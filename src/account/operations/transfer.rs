// transfer or transaction?
use crate::{
    account::{
        handle::AccountHandle,
        operations::{address_generation::AddressGenerationOptions, input_selection::select_inputs},
        types::{
            address::{AccountAddress, AddressWrapper},
            OutputData, OutputKind,
        },
        Account,
    },
    signing::{SignMessageMetadata, TransactionInput},
};

use iota_client::{
    api::finish_pow,
    bee_message::{
        address::Address,
        constants::{INPUT_OUTPUT_COUNT_MAX, INPUT_OUTPUT_COUNT_RANGE},
        input::{Input, UtxoInput},
        output::{Output, OutputId, SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput},
        payload::{
            indexation::IndexationPayload,
            transaction::{Essence, RegularEssence, TransactionPayload},
            Payload,
        },
        unlock::UnlockBlocks,
        MessageId,
    },
    common::packable::Packable,
};
use serde::{Deserialize, Serialize};

use std::str::FromStr;

const DUST_ALLOWANCE_VALUE: u64 = 1_000_000;

// clearer to have it here in transfer.rs or also mvoe it into the types folder?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOutput {
    pub address: String,
    pub amount: u64,
    #[serde(rename = "outputKind", default)]
    pub output_kind: Option<OutputKind>,
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

// Data for signing metadata
struct Remainder {
    address: Address,
    amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Function to create a transfer
pub async fn send_transfer(
    account_handle: &AccountHandle,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<MessageId> {
    log::debug!("[TRANSFER] send_transfer");
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
    let inputs = select_inputs(account_handle, amount, custom_inputs).await?;
    let (essence, inputs_for_signing, remainder) =
        create_transaction(account_handle, inputs, outputs.clone(), options).await?;
    let transaction_payload = sign_tx_essence(account_handle, essence, inputs_for_signing, remainder).await?;
    // store transaction payload to account (with db feature also store the account to the db) here before sending
    let mut account = account_handle.write().await;
    account
        .transactions
        .insert(transaction_payload.id(), transaction_payload.clone());
    drop(account);
    submit_transaction_payload(account_handle, transaction_payload).await
}
/// Function to build the transaction essence
async fn create_transaction(
    account_handle: &AccountHandle,
    inputs: Vec<OutputData>,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<(Essence, Vec<TransactionInput>, Option<Remainder>)> {
    log::debug!("[TRANSFER] create_transaction");
    let mut total_input_amount = 0;
    let mut inputs_for_essence: Vec<Input> = Vec::new();
    let mut inputs_for_signing: Vec<TransactionInput> = Vec::new();
    let account = account_handle.read().await;
    for utxo in &inputs {
        total_input_amount += utxo.amount;
        let input: Input = UtxoInput::new(utxo.transaction_id, utxo.index)?.into();
        inputs_for_essence.push(input.clone());
        // instead of finding the key_index and internal by iterating over all addresses we could also add this data to
        // the OutputData struct when syncing
        let associated_account_address = (*account
            .addresses()
            .iter()
            .filter(|a| a.address() == &utxo.address)
            .collect::<Vec<&AccountAddress>>()
            .first()
            // todo: change logic so we don't have to search the address or return an Error and don't panic
            .expect("Didn't find input address in account"))
        .clone();
        inputs_for_signing.push(TransactionInput {
            input,
            address_index: associated_account_address.key_index,
            address_internal: associated_account_address.internal,
        });
    }
    drop(account);

    let mut total_output_amount = 0;
    let mut outputs_for_essence: Vec<Output> = Vec::new();
    for output in outputs.iter() {
        let address = Address::try_from_bech32(&output.address)?;
        total_output_amount += output.amount;
        match output.output_kind {
            Some(crate::account::types::OutputKind::SignatureLockedSingle) | None => {
                outputs_for_essence.push(SignatureLockedSingleOutput::new(address, output.amount)?.into());
            }
            Some(crate::account::types::OutputKind::SignatureLockedDustAllowance) => {
                outputs_for_essence.push(SignatureLockedDustAllowanceOutput::new(address, output.amount)?.into());
            }
            _ => return Err(crate::error::Error::InvalidOutputKind("Treasury".to_string())),
        }
    }

    if total_input_amount < total_output_amount {
        return Err(crate::Error::InsufficientFunds(total_input_amount, total_output_amount));
    }
    let remainder_value = total_input_amount - total_output_amount;
    if remainder_value != 0 && remainder_value < DUST_ALLOWANCE_VALUE {
        return Err(crate::Error::LeavingDustError(format!(
            "Transaction would leave dust behind ({}i)",
            remainder_value
        )));
    }

    // Add remainder output
    let mut remainder = None;
    if remainder_value != 0 {
        let remainder_address = {
            match options.clone().unwrap_or_default().remainder_value_strategy {
                RemainderValueStrategy::ReuseAddress => inputs.first().expect("no input provided").address.clone(),
                RemainderValueStrategy::ChangeAddress => account_handle
                    .generate_addresses(
                        1,
                        Some(AddressGenerationOptions {
                            internal: true,
                            ..Default::default()
                        }),
                    )
                    .await?
                    .first()
                    .expect("Didn't generated an address")
                    .address
                    .clone(),
                RemainderValueStrategy::CustomAddress(address) => address,
            }
        };
        remainder.replace(Remainder {
            address: remainder_address.inner,
            amount: remainder_value,
        });
        outputs_for_essence
            .push(SignatureLockedDustAllowanceOutput::new(remainder_address.inner, remainder_value)?.into());
    }

    // Build transaction essence
    let mut essence_builder = RegularEssence::builder();

    // Order inputs and add them to the essence
    inputs_for_essence.sort_unstable_by_key(|a| a.pack_new());
    essence_builder = essence_builder.with_inputs(inputs_for_essence);

    // Order outputs and add them to the essence
    outputs_for_essence.sort_unstable_by_key(|a| a.pack_new());
    essence_builder = essence_builder.with_outputs(outputs_for_essence);

    // Optional add indexation payload
    if let Some(options) = options {
        if let Some(indexation) = &options.indexation {
            essence_builder = essence_builder.with_payload(Payload::Indexation(Box::new(indexation.clone())));
        }
    }

    let essence = essence_builder.finish()?;
    let essence = Essence::Regular(essence);
    Ok((essence, inputs_for_signing, remainder))
}
/// Function to sign a transaction essence
async fn sign_tx_essence(
    account: &AccountHandle,
    essence: Essence,
    mut transaction_inputs: Vec<TransactionInput>,
    remainder: Option<Remainder>,
) -> crate::Result<TransactionPayload> {
    log::debug!("[TRANSFER] sign_tx_essence");
    let account = account.read().await;
    let (remainder_deposit_address, remainder_value) = match remainder {
        Some(remainder) => (Some(remainder.address), remainder.amount),
        None => (None, 0),
    };
    let unlock_blocks = crate::signing::get_signer(account.signer_type())
        .await
        .lock()
        .await
        .sign_transaction(
            &account,
            &essence,
            &mut transaction_inputs,
            SignMessageMetadata {
                remainder_value,
                remainder_deposit_address: remainder_deposit_address.as_ref(),
                // todo: get this from the account (from the bech32_hrp of an address?) or the client
                network: crate::signing::Network::Testnet,
            },
        )
        .await?;

    // todo: validate signature after signing with the inputs
    // the public key hashes should be the same as the input address https://github.com/iotaledger/iota.rs/blob/7ba3fdd909fe5e51a9f55d47263e6191b60ade3c/iota-client/src/client.rs#L1272
    let transaction_payload = TransactionPayload::builder()
        .with_essence(essence)
        .with_unlock_blocks(UnlockBlocks::new(unlock_blocks)?)
        .finish()?;
    log::debug!("[TRANSFER] signed transaction: {:?}", transaction_payload);
    Ok(transaction_payload)
}
/// Submits a payload in a message
async fn submit_transaction_payload(
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
