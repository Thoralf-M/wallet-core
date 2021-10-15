use crate::account::{
    handle::AccountHandle,
    operations::syncing::{SyncOptions, SYNC_CHUNK_SIZE},
    types::address::{AccountAddress, AddressWithBalance},
};
use iota_client::bee_message::output::OutputId;

use std::{collections::HashSet, str::FromStr, time::Instant};

/// Get the balance and return only addresses with a positive balance
pub(crate) async fn get_addresses_with_balance(
    account_handle: &AccountHandle,
    options: &SyncOptions,
) -> crate::Result<Vec<AddressWithBalance>> {
    log::debug!("[SYNC] start get_addresses_with_balance");
    let balance_sync_start_time = Instant::now();
    let account = account_handle.read().await;
    let address_before_syncing = account.addresses().clone();

    let client_guard = crate::client::get_client(&account.client_options).await?;
    drop(account);

    log::debug!("[SYNC] sync balance for {} addresses", address_before_syncing.len());
    let mut addresses_with_balance = Vec::new();
    for addresses_chunk in address_before_syncing
        .chunks(SYNC_CHUNK_SIZE)
        .map(|x: &[AccountAddress]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let client_guard = client_guard.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client_guard.read().await;
                    let balance_response = client.get_address().balance(&address.address().to_bech32()).await?;
                    if balance_response.balance != 0 {
                        log::debug!(
                            "[SYNC] found {}i on {}",
                            balance_response.balance,
                            address.address().to_bech32()
                        );
                    }

                    crate::Result::Ok(AddressWithBalance {
                        address: address.address,
                        key_index: address.key_index,
                        internal: address.internal,
                        balance: balance_response.balance,
                    })
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let address = res?;
            // only return addresses with balance or if we discover an account so we don't pass empty addresses around
            // which only slows the process down
            if address.balance != 0 || options.sync_all_addresses {
                addresses_with_balance.push(address);
            }
        }
    }
    log::debug!(
        "[SYNC] finished get_addresses_with_balance in {:.2?}",
        balance_sync_start_time.elapsed()
    );
    Ok(addresses_with_balance)
}

/// Get the current output ids for provided addresses
pub(crate) async fn get_address_output_ids(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    addresses_with_balance: Vec<AddressWithBalance>,
) -> crate::Result<Vec<OutputId>> {
    log::debug!("[SYNC] start get_address_output_ids");
    let address_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    let client_guard = crate::client::get_client(&account.client_options).await?;
    drop(account);

    let mut found_outputs = Vec::new();
    // We split the addresses into chunks so we don't get timeouts if we have thousands
    for addresses_chunk in addresses_with_balance
        .chunks(SYNC_CHUNK_SIZE)
        .map(|x: &[AddressWithBalance]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let address = address.clone();
            let client_guard = client_guard.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client_guard.read().await;
                    let outputs_response = client
                        .get_address()
                        .outputs_response(&address.address().to_bech32(), Default::default())
                        .await?;
                    crate::Result::Ok((address, outputs_response))
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let (address, outputs_response) = res?;
            if !outputs_response.output_ids.is_empty() || options.sync_all_addresses {
                for output_id in &outputs_response.output_ids {
                    found_outputs.push(OutputId::from_str(output_id)?);
                }
            }
        }
    }
    log::debug!(
        "[SYNC] finished get_address_output_ids in {:.2?}",
        address_outputs_sync_start_time.elapsed()
    );
    // addresses with current outputs, historic outputs are ignored
    Ok(found_outputs)
}
