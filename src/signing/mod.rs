use crate::account::Account;
use iota_client::bee_message::{address::Address, input::Input};

use getset::Getters;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use std::{collections::HashMap, path::Path, sync::Arc};

#[cfg(feature = "mnemonic")]
pub(crate) mod mnemonic;

type SignerHandle = Arc<Mutex<Box<dyn Signer + Sync + Send>>>;
type Signers = Arc<Mutex<HashMap<SignerType, SignerHandle>>>;
static SIGNERS_INSTANCE: OnceCell<Signers> = OnceCell::new();

/// The signer types.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
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
    /// Mnemonic, not recommended since it's not as secure as Stronghold or Ledger
    #[cfg(feature = "mnemonic")]
    Mnemonic,
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

fn default_signers() -> Signers {
    let mut signers = HashMap::new();

    #[cfg(feature = "stronghold")]
    {
        signers.insert(
            SignerType::Stronghold,
            Arc::new(Mutex::new(
                Box::new(self::stronghold::StrongholdSigner::default()) as Box<dyn Signer + Sync + Send>
            )),
        );
    }

    #[cfg(feature = "ledger-nano")]
    {
        signers.insert(
            SignerType::LedgerNano,
            Arc::new(Mutex::new(Box::new(ledger::LedgerNanoSigner {
                is_simulator: false,
                ..Default::default()
            }) as Box<dyn Signer + Sync + Send>)),
        );
    }

    #[cfg(feature = "ledger-nano-simulator")]
    {
        signers.insert(
            SignerType::LedgerNanoSimulator,
            Arc::new(Mutex::new(Box::new(ledger::LedgerNanoSigner {
                is_simulator: true,
                ..Default::default()
            }) as Box<dyn Signer + Sync + Send>)),
        );
    }

    #[cfg(feature = "mnemonic")]
    {
        signers.insert(
            SignerType::Mnemonic,
            Arc::new(Mutex::new(
                Box::new(self::mnemonic::MnemonicSigner::default()) as Box<dyn Signer + Sync + Send>
            )),
        );
    }
    signers.insert(
        SignerType::Mnemonic,
        Arc::new(Mutex::new(
            Box::new(self::mnemonic::MnemonicSigner::default()) as Box<dyn Signer + Sync + Send>
        )),
    );

    Arc::new(Mutex::new(signers))
}

/// Sets the signer interface for the given type.
pub async fn set_signer<S: Signer + Sync + Send + 'static>(signer_type: SignerType, signer: S) {
    SIGNERS_INSTANCE
        .get_or_init(default_signers)
        .lock()
        .await
        .insert(signer_type, Arc::new(Mutex::new(Box::new(signer))));
}

/// Gets the signer interface.
pub(crate) async fn get_signer(signer_type: &SignerType) -> SignerHandle {
    SIGNERS_INSTANCE
        .get_or_init(default_signers)
        .lock()
        .await
        .get(signer_type)
        .cloned()
        .unwrap_or_else(|| panic!("signer not initialized for type {:?}", signer_type))
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
#[derive(Debug, Getters, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
