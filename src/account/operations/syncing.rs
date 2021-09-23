use crate::account::{account_handle::AccountHandle, types::address::AccountAddress, Account, AccountBalance};
use iota_client::bee_message::{address::Address, output::Output};

use serde::{Deserialize, Serialize};

use std::time::Instant;

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
    // 0 by default
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

pub async fn sync_account(account_handle: &AccountHandle, options: &SyncOptions) -> crate::Result<AccountBalance> {
    // todo verify that no conflicts can happen when we drop the account handle in between
    log::debug!("[SYNC] start syncing");
    let syc_start_time = Instant::now();
    // ignore outputs from other networks
    let addresses_with_balance = sync_addresses_balance(account_handle, options).await?;
    // get outputs only for addresses that have != 0 as balance
    sync_addresses_outputs(account_handle, options, addresses_with_balance).await?;
    // check if outputs are unspent, rebroadcast, reattach...
    // sync_transactions(){
    // retry(message_id, sync: false)
    // }.await?;
    // only when actively called or also in the background syncing?
    // consolidate_outputs().await?;
    log::debug!("[SYNC] finished syncing in {:.2?}", syc_start_time.elapsed());
    Ok(AccountBalance { total: 0, available: 0 })
}
async fn sync_addresses_balance(
    account_handle: &AccountHandle,
    options: &SyncOptions,
) -> crate::Result<Vec<AccountAddress>> {
    log::debug!("[SYNC] start balance syncing");
    let balance_sync_start_time = Instant::now();
    let account = account_handle.read().await;
    let address_before_syncing = account.addresses().clone();

    let client_guard = crate::client::get_client(&account.client_options).await?;
    drop(account);

    let client = client_guard.read().await;

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
                    log::debug!("[SYNC] sync balance for {}", address.address().to_bech32());
                    let balance_response = client.get_address().balance(&address.address().to_bech32()).await?;
                    crate::Result::Ok((address.clone(), balance_response))
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let (address, balance_response) = res?;
            // only return addresses with balance or if we discover an account so we don't pass empty addresses around
            // which only slow the process down
            if !balance_response.balance != 0 || options.discovery {
                addresses_with_balance.push(address);
            }
        }
    }
    log::debug!(
        "[SYNC] finished balance syncing in {:.2?}",
        balance_sync_start_time.elapsed()
    );
    Ok(addresses_with_balance)
}
async fn sync_addresses_outputs(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    addresses_with_balance: Vec<AccountAddress>,
) -> crate::Result<Vec<Output>> {
    // let account = account_handle.read().await;
    // let address_before_syncing = account.addresses().clone();

    // let client_guard = crate::client::get_client(&account.client_options).await?;
    // drop(account);
    // let client = client_guard.read().await;

    // let mut found_addresses = Vec::new();
    // // We split the addresses into chunks so we don't get timeouts if we have thousands
    // for addresses_chunk in address_before_syncing
    //     .chunks(SYNC_CHUNK_SIZE)
    //     .map(|x: &[AccountAddress]| x.to_vec())
    //     .into_iter()
    // {
    //     let mut tasks = Vec::new();
    //     for address in addresses_chunk {
    //         let mut address = address.clone();
    //         // let client_options = client_options.clone();
    //         tasks.push(async move {
    //             tokio::spawn(async move {
    //                 let outputs_response = client
    //                     .get_address()
    //                     .outputs_response(&address.address().to_bech32(), Default::default())
    //                     .await
    //                     .unwrap();
    //                 crate::Result::Ok((address, outputs_response))
    //             })
    //             .await
    //         });
    //     }
    //     let results = futures::future::try_join_all(tasks).await?;
    //     let mut inserted_remainder_address = false;
    //     for res in results {
    //         let (address, outputs_response) = res?;
    //         if !outputs_response.output_ids.is_empty() || options.discovery {
    //             found_addresses.push(address);
    //         } else if !inserted_remainder_address {
    //             // We want to insert one unused address to have an unused remainder address
    //             found_addresses.push(address);
    //             inserted_remainder_address = true;
    //         }
    //     }
    // }
    Ok(vec![])
}

// have an own function to sync spent outputs? (only for history reasons)
async fn sync_addresses_spent_outputs(
    account_handle: &AccountHandle,
    options: &SyncOptions,
) -> crate::Result<Vec<Output>> {
    Ok(vec![])
}
async fn sync_transactions(account_handle: &AccountHandle) -> crate::Result<()> {
    Ok(())
}
