use crate::{
    account_manager::AccountManager,
    client::options::{ClientOptions, ClientOptionsBuilder},
};

use tokio::sync::RwLock;

#[cfg(feature = "storage")]
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc};

pub struct AccountManagerBuilder {
    #[cfg(feature = "storage")]
    storage_options: Option<StorageOptions>,
    client_options: ClientOptions,
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
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }
    pub async fn finish(self) -> crate::Result<AccountManager> {
        Ok(AccountManager {
            accounts: Arc::new(RwLock::new(Vec::new())),
            background_syncing_enabled: Arc::new(AtomicBool::new(true)),
            version: 1,
        })
    }
}
