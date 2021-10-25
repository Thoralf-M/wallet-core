pub(crate) mod builder;

#[cfg(feature = "events")]
use crate::events::types::{Event, WalletEventType};
use crate::{
    account::{
        builder::AccountBuilder, handle::AccountHandle, operations::syncing::SyncOptions, types::AccountIdentifier,
    },
    client::options::ClientOptions,
    signing::SignerType,
};
use builder::AccountManagerBuilder;

use iota_client::Client;
use tokio::sync::RwLock;

use std::sync::{atomic::AtomicBool, Arc};

/// The account manager, used to create and get accounts. One account manager can hold many accounts, but they should
/// all share the same signer type with the same seed/mnemonic.
pub struct AccountManager {
    // should we use a hashmap instead of a vec like in wallet.rs?
    pub(crate) accounts: Arc<RwLock<Vec<AccountHandle>>>,
    pub(crate) background_syncing_enabled: Arc<AtomicBool>,
    pub(crate) client_options: Arc<RwLock<ClientOptions>>,
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    /// Create a new account
    pub fn create_account(&self) -> AccountBuilder {
        log::debug!("creating account");
        AccountBuilder::new(self.accounts.clone())
    }
    // can create_account be merged into get_account?
    pub async fn get_account<I: Into<AccountIdentifier>>(&self, identifier: I) -> crate::Result<AccountHandle> {
        log::debug!("get account");
        let account_id = identifier.into();
        let accounts = self.accounts.read().await;

        match account_id {
            AccountIdentifier::Id(id) => {
                for account_handle in accounts.iter() {
                    let account = account_handle.read().await;
                    if account.id() == &id {
                        return Ok(account_handle.clone());
                    }
                }
            }
            AccountIdentifier::Index(index) => {
                for account_handle in accounts.iter() {
                    let account = account_handle.read().await;
                    if account.index() == &index {
                        return Ok(account_handle.clone());
                    }
                }
            }
            AccountIdentifier::Alias(alias) => {
                for account_handle in accounts.iter() {
                    let account = account_handle.read().await;
                    if account.alias() == &alias {
                        return Ok(account_handle.clone());
                    }
                }
            }
        };
        Err(crate::Error::AccountNotFound)
    }
    /// Get all accounts
    pub async fn get_accounts(&self) -> crate::Result<Vec<AccountHandle>> {
        Ok(self.accounts.read().await.clone())
    }

    // do want a function to delete an account? If so we have to change the account creation logic, otherwise multiple
    // accounts could get the same index /// Delete an account
    // pub async fn delete_account(&self, identifier: AccountIdentifier) -> crate::Result<()> {
    // Ok(())
    // }

    // todo: find a better name for this
    // search balance, recovery from mnemonic or balance finder
    /// Function to find balances, to recover from backups
    pub async fn search_balance(
        &self,
        addresses_per_account: usize,
        account_start_index: usize,
    ) -> crate::Result<Vec<AccountHandle>> {
        Ok(vec![])
    }

    /// Sets the client options for all accounts, syncs them and sets the new bech32_hrp
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        log::debug!("[SET_CLIENT_OPTIONS]");
        let mut client_options = self.client_options.write().await;
        *client_options = options.clone();
        crate::client::set_client(options).await?;
        let accounts = self.accounts.read().await;
        for account in accounts.iter() {
            account.update_account_with_new_client().await?;
        }
        Ok(())
    }

    pub fn start_background_syncing(&self, options: SyncOptions) -> crate::Result<()> {
        Ok(())
    }
    pub fn stop_background_syncing(&self) -> crate::Result<()> {
        Ok(())
    }

    #[cfg(feature = "events")]
    /// Listen to wallet events, empty vec will listen to all events
    pub async fn listen<F>(&self, events: Vec<WalletEventType>, handler: F)
    where
        F: Fn(&Event) + 'static + Clone + Send + Sync,
    {
        let mut emitter = crate::events::EVENT_EMITTER.lock().await;
        emitter.on(events, handler);
    }

    /// Generates a new random mnemonic.
    pub fn generate_mnemonic(&self) -> crate::Result<String> {
        Ok(Client::generate_mnemonic()?)
    }

    /// Verify that a &str is a valid mnemonic.
    pub fn verify_mnemonic(&self, mnemonic: &str) -> crate::Result<()> {
        // first we check if the mnemonic is valid to give meaningful errors
        crypto::keys::bip39::wordlist::verify(mnemonic, &crypto::keys::bip39::wordlist::ENGLISH)
            .map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;
        Ok(())
    }

    /// Sets the mnemonic for the signer
    pub async fn store_mnemonic(&self, signer_type: SignerType, mnemonic: Option<String>) -> crate::Result<()> {
        let signer = crate::signing::get_signer(&signer_type).await;
        let mut signer = signer.lock().await;
        let mnemonic = match mnemonic {
            Some(m) => {
                self.verify_mnemonic(&m)?;
                m
            }
            None => self.generate_mnemonic()?,
        };
        signer.store_mnemonic(std::path::Path::new(""), mnemonic).await?;
        Ok(())
    }

    // storage feature
    #[cfg(feature = "storage")]
    pub async fn backup<P: AsRef<Path>>(&self, destination: P, stronghold_password: String) -> crate::Result<()> {
        Ok(())
    }
    #[cfg(feature = "storage")]
    pub async fn restore_backup<S: AsRef<Path>>(&self, source: S, stronghold_password: String) -> crate::Result<()> {
        Ok(())
    }
    #[cfg(feature = "storage")]
    pub async fn delete_storage(&self) -> crate::Result<()> {
        Ok(())
    }
}
