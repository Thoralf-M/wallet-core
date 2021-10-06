use crate::account::{
    operations::input_selection::select_inputs,
    types::{address::AddressWrapper, OutputData, OutputKind},
    Account,
};

use iota_client::bee_message::{
    constants::{INPUT_OUTPUT_COUNT_MAX, INPUT_OUTPUT_COUNT_RANGE},
    output::OutputId,
    payload::{
        indexation::IndexationPayload,
        transaction::{Essence, RegularEssence, TransactionPayload},
        Payload,
    },
    MessageId,
};
use serde::{Deserialize, Serialize};

use std::str::FromStr;

pub struct TransferOutput {
    address: String,
    amount: u64,
    output_kind: Option<OutputKind>,
}

#[allow(clippy::enum_variant_names)]
/// The strategy to use for the remainder value management when sending funds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "strategy", content = "value")]
pub enum RemainderValueStrategy {
    /// Keep the remainder value on the source address.
    ReuseAddress,
    /// Move the remainder value to a change address.
    ChangeAddress,
    /// Move the remainder value to any specified address.
    #[serde(with = "crate::account::types::address_serde")]
    CustomAddress(AddressWrapper),
}

impl Default for RemainderValueStrategy {
    fn default() -> Self {
        // ChangeAddress is the default because it's better for privacy than reusing an address.
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

pub async fn send_transfer(
    account: &Account,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<MessageId> {
    let amount = outputs.iter().map(|x| x.amount).sum();
    // validate outputs amount, need to be validated again in select_inputs in case we need a remainder output
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
    let inputs = select_inputs(account, amount, custom_inputs).await?;
    let essence = create_transaction(account, inputs, outputs, options).await?;
    let transaction_payload = sign_tx_essence(essence).await?;
    // store transaction payload to account (with db feature also store the account to the db) here before sending
    send_payload(Payload::Transaction(Box::new(transaction_payload))).await
}
async fn create_transaction(
    account: &Account,
    inputs: Vec<OutputData>,
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
