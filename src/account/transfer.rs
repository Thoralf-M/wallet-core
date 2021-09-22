use crate::account::input_selection::select_inputs;
use crate::account::types::AddressWrapper;
use crate::account::types::Output;
use crate::account::types::OutputKind;
use crate::account::Account;

use iota_client::bee_message::output::OutputId;
use iota_client::bee_message::payload::indexation::IndexationPayload;
use iota_client::bee_message::payload::transaction::Essence;
use iota_client::bee_message::payload::transaction::RegularEssence;
use iota_client::bee_message::payload::transaction::TransactionPayload;
use iota_client::bee_message::payload::Payload;
use iota_client::bee_message::MessageId;

use serde::{Deserialize, Serialize};

use std::str::FromStr;

pub struct TransferOutput {
    address: String,
    amount: u64,
    output_kind: Option<OutputKind>,
}

/// The strategy to use for the remainder value management when sending funds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "strategy", content = "value")]
pub enum RemainderValueStrategy {
    /// Keep the remainder value on the source address.
    ReuseAddress,
    /// Move the remainder value to a change address.
    ChangeAddress,
    /// Move the remainder value to an address that must belong to the source account.
    #[serde(with = "crate::serde::iota_address_serde")]
    AccountAddress(AddressWrapper),
}

impl Default for RemainderValueStrategy {
    fn default() -> Self {
        Self::ChangeAddress
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOptions {
    #[serde(rename = "remainderValueStrategy", default)]
    remainder_value_strategy: RemainderValueStrategy,
    indexation: Option<IndexationPayload>,
    #[serde(rename = "skipSync", default)]
    skip_sync: bool,
    #[serde(rename = "outputKind", default)]
    output_kind: Option<OutputKind>,
    #[serde(rename = "customInputs", default)]
    custom_inputs: Option<Vec<OutputId>>,
}

pub async fn send(
    account: &Account,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<MessageId> {
    let amount = outputs.iter().map(|x| x.amount).sum();
    let custom_inputs: Option<Vec<OutputId>> = {
        if let Some(options) = options.clone() {
            if let Some(outputs) = options.custom_inputs {
                Some(outputs)
            } else {
                None
            }
        } else {
            None
        }
    };
    let inputs = select_inputs(account, amount, custom_inputs).await?;
    let essence = create_transaction(account, inputs, outputs, options).await?;
    let transaction_payload = sign_tx_essence(essence).await?;
    send_payload(Payload::Transaction(Box::new(transaction_payload))).await
}
async fn create_transaction(
    account: &Account,
    inputs: Vec<Output>,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<Essence> {
    Ok(Essence::Regular(RegularEssence::builder().finish()?))
}
async fn sign_tx_essence(essence: Essence) -> crate::Result<TransactionPayload> {
    Ok(TransactionPayload::builder().finish()?)
}
async fn send_payload(payload: Payload) -> crate::Result<MessageId> {
    Ok(MessageId::from_str("")?)
}
