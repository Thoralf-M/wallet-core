use crate::{
    account::{
        operations::{
            address_generation,
            address_generation::AddressGenerationOptions,
            syncing::{sync_account, SyncOptions},
            transfer::{send_transfer, TransferOptions, TransferOutput},
        },
        types::{address::AccountAddress, AccountBalance, Transaction},
        Account,
    },
    client::options::ClientOptions,
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
        sync_account(self, &options.unwrap_or_default()).await
    }

    async fn consolidate_outputs(account: &Account) -> crate::Result<Vec<Transaction>> {
        Ok(vec![])
    }

    pub async fn send(
        &self,
        outputs: Vec<TransferOutput>,
        options: Option<TransferOptions>,
    ) -> crate::Result<MessageId> {
        // should we lock the account already already here?
        let account = self.account.write().await;
        send_transfer(&account, outputs, options).await
    }

    pub async fn retry(message_id: MessageId, sync: bool) -> crate::Result<MessageId> {
        Ok(MessageId::from_str("")?)
    }

    pub async fn generate_addresses(
        &self,
        amount: usize,
        options: Option<AddressGenerationOptions>,
    ) -> crate::Result<Vec<AccountAddress>> {
        let options = options.unwrap_or_default();
        address_generation::generate_addresses(self, amount, options).await
    }

    pub async fn list_addresses(&self) -> crate::Result<Vec<AccountAddress>> {
        let account = self.read().await;
        Ok(account.addresses().to_vec())
    }

    /// Get the total and available blance of an account
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        let account = self.account.read().await;
        let total_balance: u64 = account.addresses.iter().map(|a| a.balance()).sum();
        // for `available` get locked_outputs, sum outputs balance and substract from total_balance
        Ok(AccountBalance {
            total: total_balance,
            available: 0,
        })
    }

    // Should only be called from the AccountManager so all accounts use the same options
    pub(crate) async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        let mut account = self.account.write().await;
        account.client_options = options;
        drop(account);
        // after we set the new client options we should sync the account because the network could have changed
        // we sync with all addresses, because otherwise the balance wouldn't get updated if an address doesn't has
        // balance also in the new network
        self.sync(Some(SyncOptions {
            sync_all_addresses: true,
            ..Default::default()
        }))
        .await?;
        Ok(())
    }
}

// impl Deref so we can use `account_handle.read()` instead of `account_handle.account.read()`
impl Deref for AccountHandle {
    type Target = RwLock<Account>;
    fn deref(&self) -> &Self::Target {
        self.account.deref()
    }
}
