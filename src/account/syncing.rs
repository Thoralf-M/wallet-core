use crate::account::{Account, AccountBalance};
use iota_client::bee_message::{address::Address, output::Output};
pub struct SyncOptions {
    output_consolidation_threshold: usize,
    automatic_output_consolidation: bool,
    // 0 by default
    address_index: usize,
    // 0 by default, no new address should be generated during syncing
    gap_limit: usize,
}

pub async fn sync(account: &Account, options: SyncOptions) -> crate::Result<AccountBalance> {
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
    Ok(AccountBalance {
        total: 0,
        available: 0,
    })
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
