#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};
use crate::{
    account::{
        operations::{
            address_generation,
            address_generation::AddressGenerationOptions,
            syncing::{sync_account, SyncOptions},
            transfer::{send_transfer, TransferOptions, TransferOutput},
        },
        types::{
            address::{AccountAddress, AddressWithBalance},
            AccountBalance, OutputData, Transaction,
        },
        Account,
    },
    client::options::ClientOptions,
};

use iota_client::bee_message::{output::OutputId, MessageId};
use tokio::sync::{Mutex, RwLock};

use std::{ops::Deref, str::FromStr, sync::Arc};

/// A thread guard over an account, so we can lock the account during operations.
#[derive(Debug, Clone)]
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
    // mutex to prevent multipls sync calls at the same time, returnning the last synced result if the time was < 1?
    // second ago the u64 is a timestamp
    pub(crate) last_synced: Arc<Mutex<u128>>,
}

impl AccountHandle {
    pub(crate) fn new(account: Account) -> Self {
        Self {
            account: Arc::new(RwLock::new(account)),
            last_synced: Default::default(),
        }
    }

    /// Sync the account
    pub async fn sync(&self, options: Option<SyncOptions>) -> crate::Result<AccountBalance> {
        sync_account(self, &options.unwrap_or_default()).await
    }

    /// Consolidate outputs from addresses that have more outputs than the consolidation threshold
    async fn consolidate_outputs(account: &Account) -> crate::Result<Vec<Transaction>> {
        Ok(vec![])
    }

    /// Send a transaction
    pub async fn send(
        &self,
        outputs: Vec<TransferOutput>,
        options: Option<TransferOptions>,
    ) -> crate::Result<MessageId> {
        // sync account before sending a transaction
        #[cfg(feature = "events")]
        {
            let account_index = self.account.read().await.index;
            crate::events::EVENT_EMITTER.lock().await.emit(
                account_index,
                WalletEvent::TransferProgress(TransferProgressEvent::SyncingAccount),
            );
        }
        sync_account(
            self,
            &SyncOptions {
                automatic_output_consolidation: false,
                ..Default::default()
            },
        )
        .await?;
        send_transfer(self, outputs, options).await
    }

    /// Reattaches or promotes a message to get it confirmed
    // pub async fn retry(message_id: MessageId, sync: bool) -> crate::Result<MessageId> {
    //     Ok(MessageId::from_str("")?)
    // }

    /// Generate addresses
    pub async fn generate_addresses(
        &self,
        amount: usize,
        options: Option<AddressGenerationOptions>,
    ) -> crate::Result<Vec<AccountAddress>> {
        let options = options.unwrap_or_default();
        address_generation::generate_addresses(self, amount, options).await
    }

    /// Returns all addresses of the account
    pub async fn list_addresses(&self) -> crate::Result<Vec<AccountAddress>> {
        let account = self.read().await;
        let mut all_addresses = account.public_addresses().clone();
        all_addresses.extend(account.internal_addresses().clone());
        Ok(all_addresses.to_vec())
    }

    /// Returns only addresses of the account with balance
    pub async fn list_addresses_with_balance(&self) -> crate::Result<Vec<AddressWithBalance>> {
        let account = self.read().await;
        Ok(account.addresses_with_balance().to_vec())
    }

    /// Returns all outputs of the account
    pub async fn list_outputs(&self) -> crate::Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all unspent outputs of the account
    pub async fn list_unspent_outputs(&self) -> crate::Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.unspent_outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all transaction of the account
    pub async fn list_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction in account.transactions.values() {
            transactions.push(transaction.clone());
        }
        Ok(transactions)
    }

    /// Returns all pending transaction of the account
    pub async fn list_pending_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction_id in &account.pending_transactions {
            if let Some(transaction) = account.transactions.get(transaction_id) {
                transactions.push(transaction.clone());
            }
        }
        Ok(transactions)
    }

    /// Get the total and available blance of an account
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        log::debug!("[BALANCE] get balance");
        let account = self.account.read().await;
        let total_balance: u64 = account.addresses_with_balance.iter().map(|a| a.balance()).sum();
        // for `available` get locked_outputs, sum outputs balance and substract from total_balance
        log::debug!("[BALANCE] locked outputs: {:#?}", account.locked_outputs);
        let mut locked_balance = 0;
        for locked_output in &account.locked_outputs {
            if let Some(output) = account.unspent_outputs.get(locked_output) {
                locked_balance += output.amount;
            }
        }
        log::debug!(
            "[BALANCE] total_balance: {}, lockedbalance: {}",
            total_balance,
            locked_balance
        );
        Ok(AccountBalance {
            total: total_balance,
            available: total_balance - locked_balance,
        })
    }

    // Should only be called from the AccountManager so all accounts use the same options
    pub(crate) async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        log::debug!("[SET_CLIENT_OPTIONS]");
        let mut account = self.account.write().await;
        account.client_options = options;
        let client_guard = crate::client::get_client(&account.client_options).await?;
        let client = client_guard.read().await;
        let bech32_hrp = client.get_bech32_hrp().await?;
        drop(client);
        for address in &mut account.addresses_with_balance {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.public_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.internal_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        drop(account);
        // after we set the new client options we should sync the account because the network could have changed
        // we sync with all addresses, because otherwise the balance wouldn't get updated if an address doesn't has
        // balance also in the new network
        self.sync(Some(SyncOptions {
            sync_all_addresses: true,
            force_syncing: true,
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
