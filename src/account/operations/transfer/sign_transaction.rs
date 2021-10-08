use crate::{
    account::{
        handle::AccountHandle,
        operations::transfer::{Remainder, TransactionPayload},
    },
    signing::{SignMessageMetadata, TransactionInput},
};

use iota_client::bee_message::{payload::transaction::Essence, unlock::UnlockBlocks};

/// Function to sign a transaction essence
pub(crate) async fn sign_tx_essence(
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
