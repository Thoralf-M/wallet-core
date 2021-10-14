pub(crate) mod addresses;
pub mod options;
pub(crate) mod outputs;
pub(crate) mod transactions;

use crate::account::{
    handle::AccountHandle,
    types::{address::AccountAddress, InclusionState, OutputData, Transaction},
    AccountBalance,
};
pub use options::SyncOptions;

use iota_client::bee_message::output::OutputId;

use std::time::{Instant, SystemTime, UNIX_EPOCH};

const SYNC_CHUNK_SIZE: usize = 500;
// ms since the last sync operation
const MIN_SYNC_INTERVAL: u128 = 5000;

/// Syncs an account
pub async fn sync_account(account_handle: &AccountHandle, options: &SyncOptions) -> crate::Result<AccountBalance> {
    log::debug!("[SYNC] start syncing");
    let syc_start_time = Instant::now();

    // prevent syncing the account multiple times simultaneously
    let time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    let mut last_synced = account_handle.last_synced.lock().await;
    log::debug!("[SYNC] last time synced before {}ms", time_now - last_synced.0);
    if time_now - last_synced.0 < MIN_SYNC_INTERVAL {
        log::debug!(
            "[SYNC] synced within the latest {} ms, returning latest synced balance",
            MIN_SYNC_INTERVAL
        );
        return Ok(last_synced.1.clone());
    }

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

    let (synced_transactions, spent_output_ids) = transactions::sync_transactions(account_handle).await?;
    // only when actively called or also in the background syncing?
    // consolidate_outputs().await?;

    // update account with balances, output ids, outputs
    update_account(
        account_handle,
        addresses_with_new_output_ids,
        outputs,
        synced_transactions,
        spent_output_ids,
    )
    .await?;
    // store account with storage feature

    let account_balance = account_handle.balance().await?;
    // update last_synced mutex
    let time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    last_synced.0 = time_now;
    last_synced.1 = account_balance.clone();
    log::debug!("[SYNC] finished syncing in {:.2?}", syc_start_time.elapsed());
    Ok(account_balance)
}

/// Update account with newly synced data
async fn update_account(
    account_handle: &AccountHandle,
    addresses_with_new_output_ids: Vec<AccountAddress>,
    outputs: Vec<OutputData>,
    synced_transactions: Vec<Transaction>,
    spent_output_ids: Vec<OutputId>,
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
            .insert(OutputId::new(output.transaction_id, output.index)?, output.clone());
        if !output.is_spent {
            account
                .unspent_outputs
                .insert(OutputId::new(output.transaction_id, output.index)?, output);
        }
    }
    for transaction in synced_transactions {
        if transaction.inclusion_state == InclusionState::Confirmed {
            account.pending_transactions.remove(&transaction.payload.id());
        }
        account.transactions.insert(transaction.payload.id(), transaction);
    }
    for spent_output_id in spent_output_ids {
        if let Some(output) = account.outputs.get_mut(&spent_output_id) {
            output.is_spent = true;
        }
        if let Some(output) = account.unspent_outputs.get_mut(&spent_output_id) {
            output.is_spent = true;
        }
        account.locked_outputs.remove(&spent_output_id);
        account.unspent_outputs.remove(&spent_output_id);
        log::debug!("[SYNC] Unlocked {}", spent_output_id);
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
