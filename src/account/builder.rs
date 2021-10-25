#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
use crate::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
use crate::{
    account::{constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD, handle::AccountHandle, Account, AccountOptions},
    client::options::ClientOptions,
    signing::SignerType,
};

use tokio::sync::RwLock;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

/// The AccountBuilder
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
            #[cfg(not(any(feature = "mnemonic", feature = "stronghold")))]
            signer_type: SignerType::Custom("Signer unintialized".to_string()),
            accounts,
        }
    }
    /// Set the alias
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias.replace(alias);
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
        let consolidation_threshold = match self.signer_type {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            #[cfg(feature = "ledger-nano-simulator")]
            SignerType::LedgerNanoSimulator => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
        };
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
            // sync interval, output consolidation
            account_options: AccountOptions {
                background_syncing_interval: Duration::from_secs(7),
                output_consolidation_threshold: consolidation_threshold,
                automatic_output_consolidation: true,
            },
        };
        let account_handle = AccountHandle::new(account);
        accounts.push(account_handle.clone());
        Ok(account_handle)
    }
}
