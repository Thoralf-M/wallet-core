pub fn generate_mnemonic(&self) -> crate::Result<String> {}

pub struct AccountManagerBuilder {
    storage: Option<PathBuf,
    storage_file_name: Option<String>,
    account_options: AccountOptions,
}

impl AccountManagerBuilder {
    pub fn with_storage()
}


pub struct AccountManager {
    accounts: Arc<RwLock<AccountHandle>>,
}

impl AccountManager {
    pub async fn create_account(AccountOptions) -> Result<AccountHandle>{}
    // can create_account be merged into get_account?
    pub async fn get_account(Identifier, Option<AccountOptions>) -> Result<AccountHandle>{}
    pub async fn get_accounts() -> Result<Vec<AccountHandle>>{}
    pub async fn delete_account(AccountOptions) -> Result<AccountHandle>{}
    // search accounts with balance
    pub async fn restore_accounts(address_per_account, account_index_range) -> Result<AccountHandle>{}

    pub fn start_background_syncing()
    pub fn stop_background_syncing()

    //listen to all wallet events
    pub fn listen() -> Result<>

    // can we move this to ths builder? If mnemonic is already set, should we get an error or ignore it?
    pub async fn store_mnemonic(&self, signer_type: SignerType, mnemonic: Option<String>) -> crate::Result<()> {}

    // storage feature
    pub async fn backup<P: AsRef<Path>>(&self, destination: P, stronghold_password: String) -> crate::Result<PathBuf> {}
    pub async fn restore_backup<S: AsRef<Path>>(&self, source: S, stronghold_password: String) -> crate::Result<()> {
    pub async fn delete_storage(self) -> Result<(), (crate::Error, Self)> {
}