use crate::account::syncing::SyncOptions;
use crate::account::{types::AccountIdentifier, Account, AccountHandle};
use crate::client::ClientOptions;
use crate::client::ClientOptionsBuilder;
use crate::events::WalletEvent;
use crate::signing::SignerType;

use iota_client::Client;
use tokio::sync::RwLock;

use std::{
    path::Path,
    path::PathBuf,
    sync::{
        atomic::AtomicBool,
        mpsc::{Receiver, Sender},
        Arc,
    },
};

pub fn generate_mnemonic() -> crate::Result<String> {
    Ok(Client::generate_mnemonic()?)
}

pub struct AccountManagerBuilder {
    storage_options: Option<StorageOptions>,
    client_options: ClientOptions,
}

pub struct StorageOptions {
    storage_folder: PathBuf,
    storage_file_name: Option<String>,
    // storage: ManagerStorage,
    storage_encryption_key: Option<[u8; 32]>,
}

impl Default for AccountManagerBuilder {
    fn default() -> Self {
        Self {
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
        })
    }
}

pub struct AccountManager {
    accounts: Arc<RwLock<Vec<AccountHandle>>>,
    background_syncing_enabled: Arc<AtomicBool>,
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    pub async fn create_account(
        &self,
        options: Option<ClientOptions>,
    ) -> crate::Result<AccountHandle> {
        // create account so it compiles
        let mut account_builder = Account::new(0);
        if let Some(client_options) = options {
            account_builder = account_builder.with_client_options(client_options);
        }
        Ok(AccountHandle::new(account_builder.finish()?))
    }
    // can create_account be merged into get_account?
    pub async fn get_account<I: Into<AccountIdentifier>>(
        &self,
        identifier: I,
    ) -> crate::Result<AccountHandle> {
        // create account so it compiles
        let account_builder = Account::new(0);
        Ok(AccountHandle::new(account_builder.finish()?))
    }
    pub async fn get_accounts(&self) -> crate::Result<Vec<AccountHandle>> {
        Ok(vec![])
    }
    pub async fn delete_account(&self, identifier: AccountIdentifier) -> crate::Result<()> {
        Ok(())
    }
    // search balance, recovery from mnemonic or balance finder
    pub async fn search_accounts(
        &self,
        addresses_per_account: usize,
        account_start_index: usize,
    ) -> crate::Result<Vec<AccountHandle>> {
        Ok(vec![])
    }

    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        Ok(())
    }

    pub fn start_background_syncing(&self, options: SyncOptions) -> crate::Result<()> {
        Ok(())
    }
    pub fn stop_background_syncing(&self) -> crate::Result<()> {
        Ok(())
    }

    //listen to all wallet events
    // pub fn listen() -> crate::Result<(Sender<WalletEvent>, Receiver<WalletEvent>)> {}

    pub async fn store_mnemonic(
        &self,
        signer_type: SignerType,
        mnemonic: Option<String>,
    ) -> crate::Result<()> {
        Ok(())
    }

    // storage feature
    #[cfg(feature = "storage")]
    pub async fn backup<P: AsRef<Path>>(
        &self,
        destination: P,
        stronghold_password: String,
    ) -> crate::Result<()> {
        Ok(())
    }
    #[cfg(feature = "storage")]
    pub async fn restore_backup<S: AsRef<Path>>(
        &self,
        source: S,
        stronghold_password: String,
    ) -> crate::Result<()> {
        Ok(())
    }
    #[cfg(feature = "storage")]
    pub async fn delete_storage(&self) -> crate::Result<()> {
        Ok(())
    }
}
