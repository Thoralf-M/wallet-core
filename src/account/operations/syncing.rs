use crate::account::{Account, AccountBalance};
use iota_client::bee_message::{address::Address, output::Output};

use serde::{Deserialize, Serialize};

use std::time::Instant;

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

pub async fn sync_account(account: &Account, options: SyncOptions) -> crate::Result<AccountBalance> {
    log::debug!("[SYNC] start syncing");
    let syc_start_time = Instant::now();
    // ignore outputs from other networks
    // sync_addresses_balance().await?;
    // get outputs only for addresses that have != 0 as balance
    // sync_addresses_outputs().await?;
    // check if outputs are unspent, rebroadcast, reattach...
    // sync_transactions(){
    // retry(message_id, sync: false)
    // }.await?;
    // only when actively called or also in the background syncing?
    // consolidate_outputs().await?;
    log::debug!("[SYNC] finished syncing in {:.2?}", syc_start_time.elapsed());
    Ok(AccountBalance { total: 0, available: 0 })
}
async fn sync_addresses_balance(account: &Account) -> crate::Result<Vec<Address>> {
    Ok(vec![])
}
async fn sync_addresses_outputs(account: &Account) -> crate::Result<Vec<Output>> {
    Ok(vec![])
}
async fn sync_transactions(account: &Account) -> crate::Result<()> {
    Ok(())
}
