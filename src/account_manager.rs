use crate::client::ClientOptions;
use crate::account::syncing::SyncOptions;
use crate::account::{AccountIdentifier, AccountHandle};
use crate::signing::SignerType;

use tokio::sync::RwLock;

use std::{path::Path, sync::{atomic::AtomicBool, Arc}};

pub fn generate_mnemonic() -> crate::Result<String> {}

pub struct AccountManagerBuilder {
    storage: Option<PathBuf,
    storage_file_name: Option<String>,
    client_options: ClientOptions,
}

impl AccountManagerBuilder {
    // #[cfg(feature = "storage")]
    // pub fn with_storage()
}

pub struct AccountManager {
    accounts: Arc<RwLock<AccountHandle>>,
    background_syncing_enabled: Arc<AtomicBool>,
}

impl AccountManager {
    pub async fn create_account(options: Option<ClientOptions>) -> Result<AccountHandle>{}
    // can create_account be merged into get_account?
    pub async fn get_account(identifier: AccountIdentifier, options: Option<ClientOptions>) -> Result<AccountHandle>{}
    pub async fn get_accounts() -> Result<Vec<AccountHandle>>{}
    pub async fn delete_account(identifier: AccountIdentifier) -> Result<()>{}
    // search balance, recovery from mnemonic or balance finder
    pub async fn search_accounts(addresses_per_account: usize, account_start_index: usize) -> Result<Vec<AccountHandle>>{}

    pub async fn set_client_options(options: ClientOptions) -> Result<()>{}
    
    pub fn start_background_syncing(options: SyncOptions) -> Result<()>{}
    pub fn stop_background_syncing()-> Result<()>{}

    //listen to all wallet events
    pub fn listen() -> Result<Channel>{}

    // can we move this to ths builder? If mnemonic is already set, should we get an error or ignore it?
    pub async fn store_mnemonic(&self, signer_type: SignerType, mnemonic: Option<String>) -> crate::Result<()> {}

    // storage feature
    #[cfg(feature = "storage")]
    pub async fn backup<P: AsRef<Path>>(&self, destination: P, stronghold_password: String) -> crate::Result<PathBuf> {}
    pub async fn restore_backup<S: AsRef<Path>>(&self, source: S, stronghold_password: String) -> crate::Result<()> {}
    pub async fn delete_storage(self) -> Result<(), (crate::Error, Self)> {}
}
