use crate::account::{
    handle::AccountHandle,
    types::{InclusionState, Transaction},
};

use iota_client::bee_message::{
    input::Input,
    output::OutputId,
    payload::transaction::{Essence, TransactionId},
};

// ignore outputs and transactions from other networks
// check if outputs are unspent, rebroadcast, reattach...
// also revalidate that the locked outputs needs to be there, maybe there was a conflict or the transaction got
// confirmed, then they should get removed sync_transactions(){
// retry(message_id, sync: false)
// }.await?;

/// Sync transactions and return the confirmed transaction and spent output ids that don't need to be locked anymore
pub(crate) async fn sync_transactions(
    account_handle: &AccountHandle,
) -> crate::Result<(Vec<Transaction>, Vec<OutputId>)> {
    log::debug!("[SYNC] sync pending transactions");
    let account = account_handle.read().await;
    let client_guard = crate::client::get_client(&account.client_options).await?;
    let client = client_guard.read().await;
    let network_id = client.get_network_id().await?;

    let mut confirmed_transactions = Vec::new();
    let mut spent_output_ids = Vec::new();

    for transaction_id in &account.pending_transactions {
        let mut transaction = account
            .transactions
            .get(transaction_id)
            // panic during development to easier detect if something is wrong, should be handled different later
            .expect("transaction id stored, but transaction is missing")
            .clone();
        // only check transaction from the network we're connected to
        if transaction.network_id == network_id {
            if let Ok(included_message) = client.get_included_message(&transaction.payload.id()).await {
                // udate transaction data
                transaction.message_id.replace(included_message.id().0);
                transaction.inclusion_state = InclusionState::Confirmed;

                // get spent inputs
                let Essence::Regular(essence) = transaction.payload.essence();
                for input in essence.inputs() {
                    if let Input::Utxo(input) = input {
                        spent_output_ids.push(*input.output_id());
                    }
                }
                confirmed_transactions.push(transaction);
            } else {
                // todo:
                // - check if inputs are still available, if not get status and set InclusionState to Conflicting or
                //   Unknown
                // - if inputs are still available retry transaction
            }
        }
    }

    Ok((confirmed_transactions, spent_output_ids))
}
