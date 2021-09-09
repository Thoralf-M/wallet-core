pub fn generate_mnemonic(&self) -> crate::Result<String> {}

pub struct AccountManagerBuilder {
    storage: Option<PathBuf,
    storage_file_name: Option<String>,
    account_options: AccountOptions,
}

impl AccountManagerBuilder {
    #[cfg(feature = "storage")]
    pub fn with_storage()
}

pub struct AccountManager {
    accounts: Arc<RwLock<AccountHandle>>,
    background_syncing_enabled: Arc<AtomicBool>,
}

impl AccountManager {
    pub async fn create_account(AccountOptions) -> Result<AccountHandle>{}
    // can create_account be merged into get_account?
    pub async fn get_account(Identifier, Option<AccountOptions>) -> Result<AccountHandle>{}
    pub async fn get_accounts() -> Result<Vec<AccountHandle>>{}
    pub async fn delete_account(AccountOptions) -> Result<AccountHandle>{}
    // search balance, recovery from mnemonic or balance finder
    pub async fn search_accounts(addresses_per_account, account_start_index) -> Result<AccountHandle>{}

    pub async fn set_client_options(ClientOptions) -> Result<()>{}

    pub fn start_background_syncing(SyncOptions) -> Result<()>{}
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
pub struct SyncOptions{
    output_consolidation_threshold: usize,
    automatic_output_consolidation: bool,
    // 0 by default
    address_index: usize,
    // 0 by default, no new address should be generated during syncing
    gap_limit: usize,
}