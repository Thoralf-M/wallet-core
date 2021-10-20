// transfer or transaction?
mod create_transaction;
mod options;
mod sign_transaction;
pub(crate) mod submit_transaction;

use crate::account::{
    handle::AccountHandle,
    operations::input_selection::select_inputs,
    types::{address::AccountAddress, InclusionState, OutputData, Transaction},
};
pub use options::{RemainderValueStrategy, TransferOptions, TransferOutput};

use iota_client::bee_message::{
    address::Address,
    constants::{INPUT_OUTPUT_COUNT_MAX, INPUT_OUTPUT_COUNT_RANGE},
    output::OutputId,
    payload::transaction::{TransactionId, TransactionPayload},
    MessageId,
};

use std::time::{SystemTime, UNIX_EPOCH};

const DUST_ALLOWANCE_VALUE: u64 = 1_000_000;

// Data for signing metadata (used for ledger signer)
pub(crate) struct Remainder {
    address: AccountAddress,
    amount: u64,
}

/// Function to create a transfer
pub async fn send_transfer(
    account_handle: &AccountHandle,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<(Option<MessageId>, TransactionId)> {
    log::debug!("[TRANSFER] send_transfer");
    let amount = outputs.iter().map(|x| x.amount).sum();
    // validate outputs amount
    if !INPUT_OUTPUT_COUNT_RANGE.contains(&outputs.len()) {
        return Err(crate::Error::TooManyOutputs(outputs.len(), INPUT_OUTPUT_COUNT_MAX));
    }
    let custom_inputs: Option<Vec<OutputId>> = {
        if let Some(options) = options.clone() {
            // validate inputs amount
            if let Some(inputs) = &options.custom_inputs {
                if !INPUT_OUTPUT_COUNT_RANGE.contains(&inputs.len()) {
                    return Err(crate::Error::TooManyInputs(inputs.len(), INPUT_OUTPUT_COUNT_MAX));
                }
            }
            options.custom_inputs
        } else {
            None
        }
    };
    let inputs = select_inputs(account_handle, amount, custom_inputs).await?;
    // can we unlock the outputs in a better way if the transaction creation fails?
    let (essence, inputs_for_signing, remainder) =
        match create_transaction::create_transaction(account_handle, inputs.clone(), outputs.clone(), options).await {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                unlock_inputs(account_handle, inputs).await?;
                return Err(err);
            }
        };
    let transaction_payload =
        match sign_transaction::sign_tx_essence(account_handle, essence, inputs_for_signing, remainder).await {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                unlock_inputs(account_handle, inputs).await?;
                return Err(err);
            }
        };
    let transaction_id = transaction_payload.id();
    // store transaction payload to account (with db feature also store the account to the db) here before sending
    let mut account = account_handle.write().await;
    let client_guard = crate::client::get_client(&account.client_options).await?;
    let network_id = client_guard.read().await.get_network_id().await?;
    account.transactions.insert(
        transaction_id,
        Transaction {
            payload: transaction_payload.clone(),
            message_id: None,
            network_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis(),
            inclusion_state: InclusionState::Pending,
            incoming: false,
            internal: false,
        },
    );
    account.pending_transactions.insert(transaction_id);
    drop(account);
    match submit_transaction::submit_transaction_payload(account_handle, transaction_payload).await {
        Ok(message_id) => Ok((Some(message_id), transaction_id)),
        Err(_) => Ok((None, transaction_id)),
    }
}

// unlock outputs
async fn unlock_inputs(account_handle: &AccountHandle, inputs: Vec<OutputData>) -> crate::Result<()> {
    let mut account = account_handle.write().await;
    for output in &inputs {
        account
            .locked_outputs
            .remove(&OutputId::new(output.transaction_id, output.index)?);
    }
    Ok(())
}
