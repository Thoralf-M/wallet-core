use crate::account::{
    handle::AccountHandle,
    types::{OutputData, OutputKind},
    Account,
};

use iota_client::bee_message::output::OutputId;

const DUST_ALLOWANCE_VALUE: u64 = 1_000_000;

/// Selects inputs for a transaction
pub(crate) async fn select_inputs(
    account_handle: &AccountHandle,
    amount_to_send: u64,
    custom_inputs: Option<Vec<OutputId>>,
) -> crate::Result<Vec<OutputData>> {
    log::debug!("[TRANSFER] select_inputs");
    let mut account = account_handle.write().await;
    // todo: if custom inputs are provided we should only use them (validate if we have the outputs in this account and
    // that the amount is enough) and not others

    let client_guard = crate::client::get_client(&account.client_options).await?;
    let network_id = client_guard.read().await.get_network_id().await?;

    let mut signature_locked_outputs = Vec::new();
    let mut dust_allowance_outputs = Vec::new();
    for (address, outputs) in account.outputs.iter() {
        for output in outputs {
            // check if not in pending transaction (locked_outputs) and if from the correct network
            if !output.is_spent
                && !account
                    .locked_outputs
                    .contains(&OutputId::new(output.transaction_id, output.index)?)
                && output.network_id == network_id
            {
                match output.kind {
                    OutputKind::SignatureLockedSingle => signature_locked_outputs.push(output),
                    OutputKind::SignatureLockedDustAllowance => dust_allowance_outputs.push(output),
                    _ => {}
                }
            }
        }
    }

    // todo try to select matching inputs first, only if that's not possible we should select the inputs like below

    // Sort inputs so we can get the biggest inputs first and don't reach the input limit, if we don't have the
    // funds spread over too many outputs
    signature_locked_outputs.sort_by(|a, b| b.amount.cmp(&a.amount));
    dust_allowance_outputs.sort_by(|a, b| b.amount.cmp(&a.amount));

    let mut sum = 0;
    let selected_outputs: Vec<OutputData> = signature_locked_outputs
        .into_iter()
        // add dust_allowance_outputs only at the end so we don't try to move them when we might have still dust
        .chain(dust_allowance_outputs.into_iter())
        .take_while(|input| {
            let value = input.amount;
            let old_sum = sum;
            sum += value;
            old_sum < amount_to_send || (old_sum - amount_to_send < DUST_ALLOWANCE_VALUE && old_sum != amount_to_send)
        })
        .cloned()
        .collect();

    // lock outputs so they don't get used by another transaction
    for output in &selected_outputs {
        account
            .locked_outputs
            .insert(OutputId::new(output.transaction_id, output.index)?);
    }
    Ok(selected_outputs)
}
