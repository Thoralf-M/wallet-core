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
    let client_options = account.client_options.clone();
    drop(account);
    let client = crate::client::get_client(&client_options).await?;
    let client = client.read().await;
    if *client_options.local_pow() {
        log::debug!("[TRANSFER] doing local pow");
    }
    let message = finish_pow(&client, Some(Payload::Transaction(Box::new(transaction_payload)))).await?;
    // log::debug!("[TRANSFER] submitting message {:#?}", message);
    let message_id = client.post_message(&message).await?;
    log::debug!("[TRANSFER] submitted message {}", message_id);
    // spawn a thread which tries to get the message confirmed
    tokio::spawn(async move {
        if let Ok(client) = crate::client::get_client(&client_options).await {
            let client = client.read().await;
            if let Ok(messages) = client.retry_until_included(&message_id, None, None).await {
                for msg in messages {
                    log::debug!("[TRANSFER] reattached {}, new message id {}", message_id, msg.0);
                }
            }
        }
    });
    Ok(message_id)
}
