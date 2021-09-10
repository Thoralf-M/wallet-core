use crate::account::AccountBalance;
pub struct SyncOptions {
    output_consolidation_threshold: usize,
    automatic_output_consolidation: bool,
    // 0 by default
    address_index: usize,
    // 0 by default, no new address should be generated during syncing
    gap_limit: usize,
}

pub async fn sync(options: SyncOptions) -> Result<AccountBalance>{
            // ignore outputs from other networks
            sync_addresses_balance().await?;
            // get outputs only for addresses that have != 0 as balance
            sync_addresses_outputs().await?;
            // check if outputs are unspent, rebroadcast, reattach...
            sync_transactions(){
                retry(message_id, sync: false)
            }.await?;
            // only when actively called or also in the background syncing?
            consolidate_outputs().await?;
}
async fn sync_addresses_balance(&Account) -> Result<Vec<Adress>>{}
async fn sync_addresses_outputs(&Account) -> Result<Vec<Output>>{}
async fn sync_transactions(&Account) -> Result<()>{}