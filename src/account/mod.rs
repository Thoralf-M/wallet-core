pub(crate) mod input_selection;
pub(crate) mod syncing;
pub(crate) mod transfer;
pub(crate) mod types;

use crate::account::types::AddressWrapper;
use crate::account::types::{AccountBalance, AccountIdentifier, Output, Transaction};
use crate::client::ClientOptions;
use transfer::{TransferOptions, TransferOutput};

use iota_client::bee_message::address::Address;
use iota_client::bee_message::MessageId;
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::sync::Arc;
// Wrapper so we can lock the account during operations
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
}

impl AccountHandle {
    pub async fn sync(&self, options: syncing::SyncOptions) -> crate::Result<AccountBalance> {
        let account = self.account.write().await;
        syncing::sync(&account, options).await
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

    pub async fn retry(message_id: MessageId, sync: bool) -> crate::Result<MessageId> {}

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

pub struct Account {
    identifier: AccountIdentifier,
    addresses: Vec<AddressWrapper>,
    // stored separated from the account for performance?
    outputs: HashMap<Address, Vec<Output>>,
    // stored separated from the account for performance?
    transactions: Vec<Transaction>,
    client_options: ClientOptions,
    // sync interval, output consolidation
    account_options: AccountOptions,
}
