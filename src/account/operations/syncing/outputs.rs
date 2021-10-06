use crate::account::{
    handle::AccountHandle,
    operations::syncing::{SyncOptions, SYNC_CHUNK_SIZE},
    types::{
        address::{AccountAddress, AddressWrapper},
        OutputData, OutputKind,
    },
};
use iota_client::{
    bee_message::{
        address::{Address, Ed25519Address},
        input::UtxoInput,
        output::OutputId,
        payload::transaction::TransactionId,
        MessageId,
    },
    bee_rest_api::types::{
        dtos::{AddressDto, OutputDto},
        responses::OutputResponse,
    },
};

use std::{str::FromStr, time::Instant};

/// Convert OutputResponse to OutputData with the network_id added
pub(crate) async fn output_response_to_output_data(
    account_handle: &AccountHandle,
    output_responses: Vec<OutputResponse>,
) -> crate::Result<Vec<OutputData>> {
    log::debug!("[SYNC] convert output_responses");
    // store outputs with network_id
    let account = account_handle.read().await;
    let client_guard = crate::client::get_client(&account.client_options).await?;
    let network_id = client_guard.read().await.get_network_id().await?;
    let bech32_hrp = client_guard.read().await.get_bech32_hrp().await?;
    output_responses
        .into_iter()
        .map(|output| {
            let (amount, address, output_kind) = get_output_amount_and_address(&output.output)?;
            Ok(OutputData {
                transaction_id: TransactionId::from_str(&output.transaction_id)?,
                index: output.output_index,
                message_id: MessageId::from_str(&output.message_id)?,
                amount,
                is_spent: output.is_spent,
                address: AddressWrapper::new(address, bech32_hrp.clone()),
                kind: output_kind,
                network_id,
            })
        })
        .collect::<crate::Result<Vec<OutputData>>>()
}

/// Get output kind, amount and address from an OutputDto
pub(crate) fn get_output_amount_and_address(output: &OutputDto) -> crate::Result<(u64, Address, OutputKind)> {
    match output {
        OutputDto::Treasury(_) => Err(crate::Error::InvalidOutputKind("Treasury".to_string())),
        OutputDto::SignatureLockedSingle(ref r) => match &r.address {
            AddressDto::Ed25519(addr) => {
                let output_address = Address::from(Ed25519Address::from_str(&addr.address)?);
                Ok((r.amount, output_address, OutputKind::SignatureLockedSingle))
            }
        },
        OutputDto::SignatureLockedDustAllowance(ref r) => match &r.address {
            AddressDto::Ed25519(addr) => {
                let output_address = Address::from(Ed25519Address::from_str(&addr.address)?);
                Ok((r.amount, output_address, OutputKind::SignatureLockedDustAllowance))
            }
        },
    }
}

/// Get the current output ids for provided addresses
pub(crate) async fn get_outputs(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    addresses_with_output_ids: Vec<AccountAddress>,
) -> crate::Result<Vec<OutputResponse>> {
    log::debug!("[SYNC] start get_outputs");
    let get_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    let client_guard = crate::client::get_client(&account.client_options).await?;
    drop(account);

    let output_ids: Vec<OutputId> = addresses_with_output_ids
        .into_iter()
        .map(|address| address.outputs.into_iter())
        .flatten()
        .collect();

    let mut found_outputs = Vec::new();
    // We split the outputs into chunks so we don't get timeouts if we have thousands
    for output_ids_chunk in output_ids
        .chunks(SYNC_CHUNK_SIZE)
        .map(|x: &[OutputId]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for output_id in output_ids_chunk {
            let client_guard = client_guard.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client_guard.read().await;
                    let output = client.get_output(&UtxoInput::from(output_id)).await?;
                    crate::Result::Ok(output)
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let output = res?;
            found_outputs.push(output);
        }
    }

    log::debug!(
        "[SYNC] finished get_outputs in {:.2?}",
        get_outputs_sync_start_time.elapsed()
    );
    Ok(found_outputs)
}
