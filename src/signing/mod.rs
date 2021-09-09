/// Signer interface.
#[async_trait::async_trait]
pub trait Signer {
    /// Get the ledger status.
    async fn get_ledger_status(&self, is_simulator: bool) -> crate::LedgerStatus;
    /// Initialises a mnemonic.
    async fn store_mnemonic(&mut self, storage_path: &Path, mnemonic: String) -> crate::Result<()>;
    /// Generates an address.
    async fn generate_address(
        &mut self,
        account: &Account,
        index: usize,
        internal: bool,
        metadata: GenerateAddressMetadata,
    ) -> crate::Result<IotaAddress>;
    /// Signs transaction essence.
    async fn sign_transaction<'a>(
        &mut self,
        account: &Account,
        essence: &iota_client::bee_message::prelude::Essence,
        inputs: &mut Vec<TransactionInput>,
        metadata: SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota_client::bee_message::prelude::UnlockBlock>>;
}