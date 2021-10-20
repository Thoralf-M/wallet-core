use crate::account::{
    handle::AccountHandle,
    operations::transfer::send_transfer,
    types::{address::AddressWithBalance, OutputData, OutputKind},
    TransferOptions, TransferOutput,
};
#[cfg(feature = "events")]
use crate::events::{
    types::{Event, TransferProgressEvent, WalletEvent, WalletEventType},
    EventEmitter,
};

use iota_client::bee_message::{
    constants::{INPUT_OUTPUT_COUNT_MAX, INPUT_OUTPUT_COUNT_RANGE},
    output::OutputId,
    payload::transaction::TransactionId,
    MessageId,
};

/// Consolidates outputs from an account by sending them to the same address again if the output amount is >= the
/// output_consolidation_threshold
pub(crate) async fn consolidate_outputs(
    account_handle: &AccountHandle,
) -> crate::Result<Vec<(Option<MessageId>, TransactionId)>> {
    let account = account_handle.read().await;
    let output_consolidation_threshold = account.account_options.output_consolidation_threshold;
    let addresses_that_need_consolidation: Vec<&AddressWithBalance> = account
        .addresses_with_balance
        .iter()
        .filter(|a| a.output_ids.len() >= output_consolidation_threshold)
        .collect();

    if addresses_that_need_consolidation.is_empty() {
        log::debug!("[OUTPUT_CONSOLIDATION] no consolidation needed");
        return Ok(Vec::new());
    }
    log::debug!("[OUTPUT_CONSOLIDATION] consolidating outputs if needed");
    let client_guard = crate::client::get_client(&account.client_options).await?;
    let bech32_hrp = client_guard.read().await.get_bech32_hrp().await?;
    // Get outputs for the consoldation
    let mut outputs_to_consolidate: Vec<Vec<OutputData>> = Vec::new();
    for address in addresses_that_need_consolidation {
        let mut unspent_outputs = Vec::new();
        for output_id in &address.output_ids {
            if !account.locked_outputs.contains(output_id) {
                if let Some(output) = account.outputs.get(output_id) {
                    // only consolidate SignatureLockedSingle outputs so we can't get problems with the dust protection
                    if !output.is_spent && output.kind == OutputKind::SignatureLockedSingle {
                        unspent_outputs.push(output.clone());
                    }
                }
            }
        }
        // only consolidate if the unlocked outputs are >= output_consolidation_threshold
        if unspent_outputs.len() >= output_consolidation_threshold {
            log::debug!(
                "[OUTPUT_CONSOLIDATION] {} has {} unspent outputs",
                address.address.to_bech32(),
                unspent_outputs.len()
            );
            outputs_to_consolidate.push(unspent_outputs);
        }
    }
    drop(account);

    if outputs_to_consolidate.is_empty() {
        log::debug!("[OUTPUT_CONSOLIDATION] no consolidation needed");
    }
    let mut consolidation_results = Vec::new();
    for outputs_on_one_address in outputs_to_consolidate {
        for outputs in outputs_on_one_address.chunks(output_consolidation_threshold) {
            let output_sum = outputs.iter().map(|o| o.amount).sum();
            let consolidation_output = vec![TransferOutput {
                address: outputs[0].address.to_bech32(&bech32_hrp),
                amount: output_sum,
                output_kind: None,
            }];
            match send_transfer(
                account_handle,
                consolidation_output,
                Some(TransferOptions {
                    skip_sync: true,
                    custom_inputs: Some(
                        outputs
                            .iter()
                            .map(|o| Ok(OutputId::new(o.transaction_id, o.index)?))
                            .collect::<crate::Result<Vec<OutputId>>>()?,
                    ),
                    ..Default::default()
                }),
            )
            .await
            {
                Ok(res) => {
                    log::debug!(
                        "[OUTPUT_CONSOLIDATION] Consolidation transaction sent: msg_id: {:?} tx_id: {:?}",
                        res.0,
                        res.1
                    );
                    consolidation_results.push(res);
                }
                Err(e) => log::debug!("Consolidation error: {}", e),
            };
        }
    }

    Ok(consolidation_results)
}
