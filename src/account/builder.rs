use crate::{
    account::{Account, AccountOptions},
    client::options::{ClientOptions, ClientOptionsBuilder},
    signing::SignerType,
};

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

pub struct AccountBuilder {
    index: usize,
    client_options: Option<ClientOptions>,
    signer_type: SignerType,
}

impl AccountBuilder {
    /// Create an IOTA client builder
    pub fn new(index: usize) -> Self {
        Self {
            index,
            client_options: None,
            #[cfg(feature = "stronghold")]
            signer_type: SignerType::Stronghold,
            #[cfg(all(feature = "mnemonic", not(feature = "stronghold")))]
            signer_type: SignerType::Mnemonic,
            #[cfg(not(any(feature = "stronghold", feature = "mnemonic")))]
            signer_type: SignerType::Mnemonic,
        }
    }
    /// Set the client options
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
    pub fn finish(&self) -> crate::Result<Account> {
        Ok(Account {
            id: self.index.to_string(),
            index: self.index,
            alias: self.index.to_string(),
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
        })
    }
}
