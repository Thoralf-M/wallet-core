use crate::account::Account;
use getset::Getters;
use iota_client::bee_message::address::Address;
use std::fs::Path;

pub enum SignerType {
    /// Stronghold signer.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    Stronghold,
    /// Ledger Device
    #[cfg(feature = "ledger-nano")]
    LedgerNano,
    /// Ledger Speculos Simulator
    #[cfg(feature = "ledger-nano-simulator")]
    LedgerNanoSimulator,
    /// Custom signer with its identifier.
    Custom(String),
}
/// Signer interface.
#[async_trait::async_trait]
pub trait Signer {
    /// Get the ledger status.
    async fn get_ledger_status(&self, is_simulator: bool) -> LedgerStatus;
    /// Initialises a mnemonic.
    async fn store_mnemonic(&mut self, storage_path: &Path, mnemonic: String) -> crate::Result<()>;
    /// Generates an address.
    async fn generate_address(
        &mut self,
        account: &Account,
        index: usize,
        internal: bool,
        metadata: GenerateAddressMetadata,
    ) -> crate::Result<Address>;
    /// Signs transaction essence.
    async fn sign_transaction<'a>(
        &mut self,
        account: &Account,
        essence: &iota_client::bee_message::prelude::Essence,
        inputs: &mut Vec<TransactionInput>,
        metadata: SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota_client::bee_message::prelude::UnlockBlock>>;
}

/// Metadata provided to [sign_message](trait.Signer.html#method.sign_message).
#[derive(Getters)]
#[getset(get = "pub")]
pub struct SignMessageMetadata<'a> {
    /// The transfer's address that has remainder value if any.
    pub remainder_address: Option<&'a Address>,
    /// The transfer's remainder value.
    pub remainder_value: u64,
    /// The transfer's deposit address for the remainder value if any.
    pub remainder_deposit_address: Option<&'a Address>,
    /// The network which is used so the correct BIP32 path is used for the ledger. Debug mode starts with 44'/1' and
    /// in mainnet-mode it's 44'/4218'
    pub network: Network,
}

/// Metadata provided to [generate_address](trait.Signer.html#method.generate_address).
#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct GenerateAddressMetadata {
    /// Indicates that the address is being generated as part of the account syncing process.
    /// This means that the account might not be saved.
    /// If it is false, the prompt will be displayed on ledger devices.
    pub syncing: bool,
    /// The network which is used so the correct BIP32 path is used for the ledger. Debug mode starts with 44'/1' and
    /// in mainnet-mode it's 44'/4218'
    pub network: Network,
}

/// Network enum for ledger metadata
#[derive(Debug, Clone, PartialEq)]
pub enum Network {
    /// Mainnet
    Mainnet,
    /// Testnet
    Testnet,
}

/// The Ledger device status.
#[derive(Debug, ::serde::Serialize)]
pub struct LedgerApp {
    /// Opened app name.
    name: String,
    /// Opened app version.
    version: String,
}

/// The Ledger device status.
#[derive(Debug, ::serde::Serialize)]
pub struct LedgerStatus {
    /// Ledger is available and ready to be used.
    connected: bool,
    /// Ledger is connected and locked.
    locked: bool,
    /// Ledger opened app.
    app: Option<LedgerApp>,
}

/// One of the transaction inputs and its address information needed for signing it.
pub struct TransactionInput {
    /// The input.
    pub input: Input,
    /// Input's address index.
    pub address_index: usize,
    /// Whether the input address is a change address or a public address.
    pub address_internal: bool,
}
