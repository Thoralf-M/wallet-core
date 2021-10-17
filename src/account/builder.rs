use crate::{
    account::{handle::AccountHandle, Account, AccountOptions},
    account_manager::AccountManager,
    client::options::{ClientOptions, ClientOptionsBuilder},
    signing::SignerType,
};

use tokio::sync::{RwLock, RwLockWriteGuard};

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

pub struct AccountBuilder {
    client_options: Option<ClientOptions>,
    alias: Option<String>,
    signer_type: SignerType,
    accounts: Arc<RwLock<Vec<AccountHandle>>>,
}

impl AccountBuilder {
    /// Create an IOTA client builder
    pub fn new(accounts: Arc<RwLock<Vec<AccountHandle>>>) -> Self {
        Self {
            client_options: None,
            alias: None,
            #[cfg(feature = "stronghold")]
            signer_type: SignerType::Stronghold,
            #[cfg(all(feature = "mnemonic", not(feature = "stronghold")))]
            signer_type: SignerType::Mnemonic,
            #[cfg(not(any(feature = "stronghold", feature = "mnemonic")))]
            signer_type: SignerType::Mnemonic,
            accounts,
        }
    }
    /// Set the alias
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias.replace(alias);
        self
    }
    /// Set the client options (should this only be available via the AccountManager?)
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options.replace(options);
        self
    }
    /// Set the signer type
    pub fn with_signer_type(mut self, signer_type: SignerType) -> Self {
        self.signer_type = signer_type;
        self
    }
    // Build the Account
    pub async fn finish(&self) -> crate::Result<AccountHandle> {
        let mut accounts = self.accounts.write().await;
        let index = accounts.len();
        let account = Account {
            id: index.to_string(),
            index,
            alias: self.alias.clone().unwrap_or_else(|| index.to_string()),
            signer_type: self.signer_type.clone(),
            public_addresses: Vec::new(),
            internal_addresses: Vec::new(),
            addresses_with_balance: Vec::new(),
            outputs: HashMap::new(),
            locked_outputs: HashSet::new(),
            unspent_outputs: HashMap::new(),
            transactions: HashMap::new(),
            pending_transactions: HashSet::new(),
            // default options for testing
            client_options: self.client_options.clone().unwrap_or(
                ClientOptionsBuilder::new()
                    .with_primary_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
                    .finish()?,
            ),
            // sync interval, output consolidation
            account_options: AccountOptions {
                background_syncing_interval: Duration::from_secs(5),
                output_consolidation_threshold: 100,
                automatic_output_consolidation: true,
            },
        };
        let account_handle = AccountHandle::new(account);
        accounts.push(account_handle.clone());
        Ok(account_handle)
    }
}
