use crate::{
    account::{Account, AccountOptions},
    client::{ClientOptions, ClientOptionsBuilder},
    signing::SignerType,
};

use std::collections::HashMap;

pub struct AccountBuilder {
    index: usize,
    client_options: Option<ClientOptions>,
}
impl AccountBuilder {
    /// Create an Iota client builder
    pub fn new(index: usize) -> Self {
        Self {
            index,
            client_options: None,
        }
    }
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options.replace(options);
        self
    }
    pub fn finish(&self) -> crate::Result<Account> {
        Ok(Account {
            id: self.index.to_string(),
            index: self.index,
            alias: self.index.to_string(),
            signer_type: SignerType::Custom("".to_string()),
            addresses: Vec::new(),
            outputs: HashMap::new(),
            transactions: Vec::new(),
            // default options for testing
            client_options: self.client_options.clone().unwrap_or(
                ClientOptionsBuilder::new()
                    .with_primary_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
                    .finish()?,
            ),
            // sync interval, output consolidation
            account_options: AccountOptions {
                output_consolidation_threshold: 100,
                automatic_output_consolidation: true,
            },
        })
    }
}
