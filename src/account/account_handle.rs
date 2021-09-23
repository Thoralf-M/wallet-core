use crate::{
    account::{
        operations::{
            address_generation,
            syncing::{sync_account, SyncOptions},
            transfer::{send_transfer, TransferOptions, TransferOutput},
        },
        types::{address::AccountAddress, AccountBalance, Transaction},
        Account,
    },
    client::ClientOptions,
};

use iota_client::bee_message::MessageId;
use tokio::sync::RwLock;

use std::{ops::Deref, str::FromStr, sync::Arc};

/// A thread guard over an account, so we can lock the account during operations.
#[derive(Debug, Clone)]
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
        sync_account(&self, &options.unwrap_or_default()).await
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
        send_transfer(&account, outputs, options).await
    }

    pub async fn retry(message_id: MessageId, sync: bool) -> crate::Result<MessageId> {
        Ok(MessageId::from_str("")?)
    }

    pub async fn generate_addresses(&self, amount: usize) -> crate::Result<Vec<AccountAddress>> {
        address_generation::generate_addresses(&self, amount).await
    }

    pub async fn list_addresses(&self) -> crate::Result<Vec<AccountAddress>> {
        let account = self.read().await;
        Ok(account.addresses().to_vec())
    }

    pub fn balance() -> crate::Result<AccountBalance> {
        Ok(AccountBalance { total: 0, available: 0 })
    }

    // Should only be called from the AccountManager so all accounts use the same options
    pub(crate) async fn set_client_options(options: ClientOptions) -> crate::Result<()> {
        Ok(())
    }
}

impl Deref for AccountHandle {
    type Target = RwLock<Account>;
    fn deref(&self) -> &Self::Target {
        self.account.deref()
    }
}
