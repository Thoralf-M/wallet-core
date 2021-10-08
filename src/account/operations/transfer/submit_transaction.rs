use crate::account::{handle::AccountHandle, operations::transfer::TransactionPayload};

use iota_client::{
    api::finish_pow,
    bee_message::{payload::Payload, MessageId},
};

/// Submits a payload in a message
pub(crate) async fn submit_transaction_payload(
    account_handle: &AccountHandle,
    transaction_payload: TransactionPayload,
) -> crate::Result<MessageId> {
    log::debug!("[TRANSFER] send_payload");
    let account = account_handle.read().await;
    let client = crate::client::get_client(&account.client_options).await?;
    drop(account);
    let client = client.read().await;
    log::debug!("[TRANSFER] doing pow if local pow is used");
    let message = finish_pow(&client, Some(Payload::Transaction(Box::new(transaction_payload)))).await?;
    log::debug!("[TRANSFER] submitting message {:#?}", message);
    let message_id = client.post_message(&message).await?;
    // spawn a thread which tries to get the message confirmed?
    Ok(message_id)
}
