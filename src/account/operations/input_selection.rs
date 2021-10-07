use crate::account::{handle::AccountHandle, Account};

use iota_client::bee_message::output::OutputId;

const DUST_ALLOWANCE_VALUE: u64 = 1_000_000;

/// Selects inputs for a transaction
pub(crate) async fn select_inputs(
    account_handle: &AccountHandle,
    amount_to_send: u64,
    custom_inputs: Option<Vec<OutputId>>,
) -> crate::Result<Vec<crate::account::types::OutputData>> {
    log::debug!("[TRANSFER] select_inputs");
    let account = account_handle.read().await;
    // todo: if custom inputs are provided we should only use them (validate if we have the outputs in this account and
    // that the amount is enough) and not others

    let mut available_outputs = Vec::new();
    for (address, outputs) in account.outputs.iter() {
        for output in outputs {
            // todo check if not in pending transaction (locked_outputs) and if from the correct network
            if !output.is_spent {
                available_outputs.push(output);
            }
        }
    }

    let mut sum = 0;
    let selected_outputs = available_outputs
        .into_iter()
        // todo: add dust_allowance_outputs only at the end so we don't try to move them when we might have still dust
        // on the address .chain(available_dust_allowance_utxos.into_iter())
        .take_while(|input| {
            let value = input.amount;
            let old_sum = sum;
            sum += value;
            old_sum < amount_to_send || (old_sum - amount_to_send < DUST_ALLOWANCE_VALUE && old_sum != amount_to_send)
        })
        .cloned()
        .collect();
    Ok(selected_outputs)
}
