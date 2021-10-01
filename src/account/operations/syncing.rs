use crate::account::{handle::AccountHandle, types::address::AccountAddress, AccountBalance};
use iota_client::{
    bee_message::{input::UtxoInput, output::OutputId},
    bee_rest_api::types::responses::OutputResponse,
};

use serde::{Deserialize, Serialize};

use std::{str::FromStr, time::Instant};

const SYNC_CHUNK_SIZE: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncOptions {
    #[serde(
        rename = "outputConsolidationThreshold",
        default = "default_output_consolidation_threshold"
    )]
    output_consolidation_threshold: usize,
    #[serde(
        rename = "automaticOutputConsolidation",
        default = "default_automatic_output_consolidation"
    )]
    automatic_output_consolidation: bool,
    // 0 by default, using a higher value could result in a wrong balance since addresses with a lower index aren't
    // synced
    #[serde(rename = "addressStartIndex", default = "default_address_start_index")]
    address_start_index: usize,
    // 0 by default, no new address should be generated during syncing
    #[serde(rename = "gapLimit", default = "default_gap_limit")]
    gap_limit: usize,
    #[serde(rename = "syncSpentOutputs", default)]
    sync_spent_outputs: bool,
    // have such a field here if balance discovery is used? When we discorver an account we don't want to return all
    // empty generated addresses maybe easier to just delete empty addresses from the account at the end of the
    // discovery process?
    #[serde(rename = "discovery", default)]
    discovery: bool,
}

fn default_output_consolidation_threshold() -> usize {
    100
}

fn default_automatic_output_consolidation() -> bool {
    true
}

fn default_address_start_index() -> usize {
    0
}

fn default_gap_limit() -> usize {
    0
}

// Sync an account
pub async fn sync_account(account_handle: &AccountHandle, options: &SyncOptions) -> crate::Result<AccountBalance> {
    // todo verify that no conflicts can happen when we drop the account handle in between
    log::debug!("[SYNC] start syncing");
    let syc_start_time = Instant::now();
    // ignore outputs from other networks
    let addresses_with_balance = get_addresses_with_balance(account_handle, options).await?;
    log::debug!("[SYNC] found {} addresses_with_balance", addresses_with_balance.len());
    // could result in a wrong balance if we sync from a higher start index
    // todo still calculate it at the end with all addresses from the account and not only the new ones
    let total_balance: u64 = if options.address_start_index == 0 {
        addresses_with_balance.iter().map(|address| address.balance()).sum()
    } else {
        // synced with a higher start index, add balance from lower addresses
        // todo add balance from addresses with index < options.address_start_index
        addresses_with_balance.iter().map(|address| address.balance()).sum()
    };
    // get outputs only for addresses that have > 0 as balance
    let addresses_with_new_output_ids = get_address_output_ids(account_handle, options, addresses_with_balance).await?;
    // only set unspent output ids? That way we keep the account size smaller, spent outputs can still be stored also
    // from different networks, but will have no impact

    let outputs = get_outputs(account_handle, options, addresses_with_new_output_ids).await?;
    // store outputs with network_id

    // check if outputs are unspent, rebroadcast, reattach...
    // sync_transactions(){
    // retry(message_id, sync: false)
    // }.await?;
    // only when actively called or also in the background syncing?
    // consolidate_outputs().await?;

    // update account with balances, output ids, outputs (if already existing don't overwrite if it's locked for a
    // transaction)

    // store account with storage feature

    log::debug!("[SYNC] finished syncing in {:.2?}", syc_start_time.elapsed());
    Ok(AccountBalance {
        total: total_balance,
        available: 0,
    })
}

/// Get the balance and return only addresses with a positive balance
async fn get_addresses_with_balance(
    account_handle: &AccountHandle,
    options: &SyncOptions,
) -> crate::Result<Vec<AccountAddress>> {
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
        for mut address in addresses_chunk {
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
                    address.balance = balance_response.balance;
                    crate::Result::Ok(address)
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let address = res?;
            // only return addresses with balance or if we discover an account so we don't pass empty addresses around
            // which only slows the process down
            if address.balance != 0 || options.discovery {
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
async fn get_address_output_ids(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    addresses_with_balance: Vec<AccountAddress>,
) -> crate::Result<Vec<AccountAddress>> {
    log::debug!("[SYNC] start get_address_output_ids");
    let address_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    let client_guard = crate::client::get_client(&account.client_options).await?;
    drop(account);

    let mut found_outputs = Vec::new();
    // We split the addresses into chunks so we don't get timeouts if we have thousands
    for addresses_chunk in addresses_with_balance
        .chunks(SYNC_CHUNK_SIZE)
        .map(|x: &[AccountAddress]| x.to_vec())
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
            if !outputs_response.output_ids.is_empty() || options.discovery {
                found_outputs.push((address, outputs_response));
            }
        }
    }
    // addresses with current outputs, historic outputs are ignored
    let mut addresses_with_outputs = Vec::new();
    // update account addresses
    // todo should this happen outside of this function in sync_account (maybe the same time with balance and the
    // outputs)?
    let mut account = account_handle.write().await;
    for address in &mut account.addresses {
        if let Some(address_outputs) = found_outputs.iter().find(|a| address.address == a.0.address) {
            for output_id in &address_outputs.1.output_ids {
                address.outputs.insert(OutputId::from_str(output_id)?);
            }
            addresses_with_outputs.push(address.clone());
        }
    }
    log::debug!(
        "[SYNC] finished get_address_output_ids in {:.2?}",
        address_outputs_sync_start_time.elapsed()
    );
    Ok(addresses_with_outputs)
}

/// Get the current output ids for provided addresses
async fn get_outputs(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    addresses_with_output_ids: Vec<AccountAddress>,
) -> crate::Result<Vec<OutputResponse>> {
    log::debug!("[SYNC] start get_outputs");
    let get_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    let client_guard = crate::client::get_client(&account.client_options).await?;
    drop(account);

    let output_ids: Vec<OutputId> = addresses_with_output_ids
        .into_iter()
        .map(|address| address.outputs.into_iter())
        .flatten()
        .collect();

    let mut found_outputs = Vec::new();
    // We split the outputs into chunks so we don't get timeouts if we have thousands
    for output_ids_chunk in output_ids
        .chunks(SYNC_CHUNK_SIZE)
        .map(|x: &[OutputId]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for output_id in output_ids_chunk {
            let client_guard = client_guard.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client_guard.read().await;
                    let output = client.get_output(&UtxoInput::from(output_id)).await?;
                    crate::Result::Ok(output)
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let output = res?;
            found_outputs.push(output);
        }
    }

    log::debug!(
        "[SYNC] finished get_outputs in {:.2?}",
        get_outputs_sync_start_time.elapsed()
    );
    Ok(found_outputs)
}

async fn sync_transactions(account_handle: &AccountHandle) -> crate::Result<()> {
    Ok(())
}

// have an own function to sync spent outputs? (only for history reasons, not important now)
// async fn get_spent_outputs(
//     account_handle: &AccountHandle,
//     options: &SyncOptions,
// ) -> crate::Result<Vec<Output>> {
//     Ok(vec![])
// }
