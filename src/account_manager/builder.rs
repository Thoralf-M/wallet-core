use crate::{
    account_manager::AccountManager,
    client::options::{ClientOptions, ClientOptionsBuilder},
    signing::SignerType,
};

use tokio::sync::RwLock;

#[cfg(feature = "storage")]
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc};

/// Builder for the account manager.
pub struct AccountManagerBuilder {
    #[cfg(feature = "storage")]
    storage_options: Option<StorageOptions>,
    client_options: ClientOptions,
    signer_type: SignerType,
}

#[cfg(feature = "storage")]
pub struct StorageOptions {
    storage_folder: PathBuf,
    storage_file_name: Option<String>,
    // storage: ManagerStorage,
    storage_encryption_key: Option<[u8; 32]>,
}

impl Default for AccountManagerBuilder {
    fn default() -> Self {
        Self {
            #[cfg(feature = "storage")]
            storage_options: None,
            client_options: ClientOptionsBuilder::new()
                .finish()
                .expect("default client options failed"),
            #[cfg(feature = "stronghold")]
            signer_type: SignerType::Stronghold,
            #[cfg(all(feature = "mnemonic", not(feature = "stronghold")))]
            signer_type: SignerType::Mnemonic,
            #[cfg(not(any(feature = "mnemonic", feature = "stronghold")))]
            signer_type: SignerType::Custom("Signer unintialized".to_string()),
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options = options;
        self
    }
    pub fn with_signer_type(mut self, signer_type: SignerType) -> Self {
        self.signer_type = signer_type;
        self
    }
    /// Builds the account manager
    pub async fn finish(self) -> crate::Result<AccountManager> {
        crate::signing::set_signer(self.signer_type.clone());
        crate::client::set_client(self.client_options.clone()).await?;
        Ok(AccountManager {
            accounts: Arc::new(RwLock::new(Vec::new())),
            background_syncing_enabled: Arc::new(AtomicBool::new(true)),
            client_options: Arc::new(RwLock::new(self.client_options)),
            signer_type: self.signer_type,
        })
    }
}
