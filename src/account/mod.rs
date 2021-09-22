pub(crate) mod input_selection;
pub(crate) mod syncing;
pub(crate) mod transfer;
pub(crate) mod types;

use crate::account::types::AddressWrapper;
use crate::account::types::{AccountBalance, Output, Transaction};
use crate::client::ClientOptions;
use crate::client::ClientOptionsBuilder;
use crate::signing::SignerType;
use syncing::SyncOptions;
use transfer::{TransferOptions, TransferOutput};

use getset::{Getters, Setters};
use iota_client::bee_message::address::Address;
use iota_client::bee_message::MessageId;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

// Wrapper so we can lock the account during operations
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
}

impl AccountHandle {
    pub(crate) fn new(account: Account) -> Self {
        Self {
            account: Arc::new(RwLock::new(account)),
        }
    }

    pub async fn sync(&self, options: Option<SyncOptions>) -> crate::Result<AccountBalance> {
        let account = self.account.write().await;
        syncing::sync(&account, options.unwrap_or_default()).await
    }

    async fn consolidate_outputs(account: &Account) -> crate::Result<Vec<Transaction>> {
        Ok(vec![])
    }

    pub async fn send(
        &self,
        outputs: Vec<TransferOutput>,
        options: Option<TransferOptions>,
    ) -> crate::Result<MessageId> {
        let account = self.account.write().await;
        transfer::send(&account, outputs, options).await
    }

    pub async fn retry(message_id: MessageId, sync: bool) -> crate::Result<MessageId> {
        Ok(MessageId::from_str("")?)
    }

    pub async fn generate_addresses(amount: usize) -> crate::Result<Vec<AddressWrapper>> {
        Ok(vec![])
    }

    pub async fn list_addresses() -> crate::Result<Vec<AddressWrapper>> {
        Ok(vec![])
    }

    pub fn balance() -> crate::Result<AccountBalance> {
        Ok(AccountBalance {
            total: 0,
            available: 0,
        })
    }

    // Should only be called from the AccountManager so all accounts use the same options
    pub(crate) async fn set_client_options(options: ClientOptions) -> crate::Result<()> {
        Ok(())
    }
}

/// Account definition.
#[derive(Debug, Setters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    #[getset(set = "pub(crate)")]
    id: String,
    /// The account index
    index: usize,
    /// The account alias.
    alias: String,
    /// The account's signer type.
    #[serde(rename = "signerType")]
    signer_type: SignerType,
    addresses: Vec<AddressWrapper>,
    // stored separated from the account for performance?
    outputs: HashMap<Address, Vec<Output>>,
    // stored separated from the account for performance?
    transactions: Vec<Transaction>,
    client_options: ClientOptions,
    // sync interval, output consolidation
    account_options: AccountOptions,
}
impl Account {
    pub fn new(index: usize) -> AccountBuilder {
        AccountBuilder::new(index)
    }
}

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

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct AccountOptions {
    pub(crate) output_consolidation_threshold: usize,
    pub(crate) automatic_output_consolidation: bool,
    // #[cfg(feature = "storage")]
    // pub(crate) persist_events: bool,
}
