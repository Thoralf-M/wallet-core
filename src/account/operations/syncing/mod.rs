pub(crate) mod addresses;
pub mod options;
pub(crate) mod outputs;

use crate::account::{
    handle::AccountHandle,
    types::{address::AccountAddress, OutputData},
    AccountBalance,
};
pub use options::SyncOptions;

use std::time::Instant;

const SYNC_CHUNK_SIZE: usize = 500;

/// Syncs an account
pub async fn sync_account(account_handle: &AccountHandle, options: &SyncOptions) -> crate::Result<AccountBalance> {
    // todo verify that no conflicts can happen when we drop the account handle in between
    log::debug!("[SYNC] start syncing");
    let syc_start_time = Instant::now();

    // we get the balance first because it's a less heavy operation for the nodes
    let addresses_with_balance = addresses::get_addresses_with_balance(account_handle, options).await?;
    log::debug!("[SYNC] found {} addresses_with_balance", addresses_with_balance.len());

    // get outputs only for addresses that have > 0 as balance
    let addresses_with_new_output_ids =
        addresses::get_address_output_ids(account_handle, options, addresses_with_balance).await?;
    // only set unspent output ids? That way we keep the account size smaller, spent outputs can still be stored also
    // from different networks, but will have no impact

    let output_responses = outputs::get_outputs(account_handle, options, addresses_with_new_output_ids.clone()).await?;
    let outputs = outputs::output_response_to_output_data(account_handle, output_responses).await?;

    // ignore outputs and transactions from other networks
    // check if outputs are unspent, rebroadcast, reattach...
    // also revalidate that the locked outputs needs to be there, maybe there was a conflict or the transaction got
    // confirmed, then they should get removed sync_transactions(){
    // retry(message_id, sync: false)
    // }.await?;
    // only when actively called or also in the background syncing?
    // consolidate_outputs().await?;

    // update account with balances, output ids, outputs
    update_account(account_handle, addresses_with_new_output_ids, outputs).await?;
    // store account with storage feature

    log::debug!("[SYNC] finished syncing in {:.2?}", syc_start_time.elapsed());
    account_handle.balance().await
}

/// Update account with newly synced data
async fn update_account(
    account_handle: &AccountHandle,
    addresses_with_new_output_ids: Vec<AccountAddress>,
    outputs: Vec<OutputData>,
) -> crate::Result<()> {
    let mut account = account_handle.write().await;
    for address in addresses_with_new_output_ids {
        let r = account
            .addresses
            .binary_search_by_key(&(address.key_index, address.internal), |a| (a.key_index, a.internal));
        if let Ok(index) = r {
            account.addresses[index].balance = address.balance;
            account.addresses[index].outputs.extend(address.outputs);
        }
    }
    for output in outputs {
        account
            .outputs
            .entry(output.address.inner)
            .and_modify(|outputs| {
                // update output or insert it
                let r = outputs
                    .binary_search_by_key(&(output.transaction_id, output.index), |a| (a.transaction_id, a.index));
                if let Ok(index) = r {
                    outputs[index] = output.clone();
                } else {
                    outputs.push(output.clone())
                }
            })
            .or_insert_with(|| vec![output]);
    }
    // println!("{:#?}", account);
    Ok(())
}

async fn sync_transactions(account_handle: &AccountHandle) -> crate::Result<()> {
    // when confirmed update balance of the addresses with the spent outputs
    Ok(())
}

// have an own function to sync spent outputs? (only for history reasons, not important now)
// async fn get_spent_outputs(
//     account_handle: &AccountHandle,
//     options: &SyncOptions,
// ) -> crate::Result<Vec<Output>> {
//     Ok(vec![])
// }
