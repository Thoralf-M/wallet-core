use crate::{
    account::{
        handle::AccountHandle,
        operations::{
            address_generation::AddressGenerationOptions,
            transfer::{Remainder, RemainderValueStrategy, TransferOptions, TransferOutput, DUST_ALLOWANCE_VALUE},
        },
        types::{address::AccountAddress, OutputData, OutputKind},
    },
    signing::TransactionInput,
};

use iota_client::{
    bee_message::{
        address::Address,
        input::{Input, UtxoInput},
        output::{Output, SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput},
        payload::{
            transaction::{Essence, RegularEssence},
            Payload,
        },
    },
    common::packable::Packable,
};

/// Function to build the transaction essence
pub(crate) async fn create_transaction(
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
        let options_ = options.clone().unwrap_or_default();
        let remainder_address = {
            match options_.remainder_value_strategy {
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
        match options_.remainder_output_kind {
            Some(OutputKind::SignatureLockedDustAllowance) => outputs_for_essence
                .push(SignatureLockedDustAllowanceOutput::new(remainder_address.inner, remainder_value)?.into()),
            _ => outputs_for_essence
                .push(SignatureLockedSingleOutput::new(remainder_address.inner, remainder_value)?.into()),
        }
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
