pub(crate) mod builder;

use crate::{
    account::{
        builder::AccountBuilder, handle::AccountHandle, operations::syncing::SyncOptions, types::AccountIdentifier,
    },
    client::options::ClientOptions,
    events::WalletEvent,
    signing::SignerType,
};
use builder::AccountManagerBuilder;

use iota_client::Client;
use tokio::sync::RwLock;

use std::sync::{atomic::AtomicBool, Arc};

pub struct AccountManager {
    // should we use a hashmap instead of a vec like in wallet.rs?
    pub(crate) accounts: Arc<RwLock<Vec<AccountHandle>>>,
    pub(crate) background_syncing_enabled: Arc<AtomicBool>,
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    pub async fn create_account(&self, options: Option<ClientOptions>) -> crate::Result<AccountHandle> {
        log::debug!("creating account");
        // create account so it compiles
        let mut account_builder = AccountBuilder::new(0);
        if let Some(client_options) = options {
            account_builder = account_builder.with_client_options(client_options);
        }
        let account_handle = AccountHandle::new(account_builder.finish()?);
        let mut accounts = self.accounts.write().await;
        accounts.push(account_handle.clone());
        Ok(account_handle)
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
            AccountIdentifier::Address(address) => {
                for account_handle in accounts.iter() {
                    let account = account_handle.read().await;
                    if account.addresses().iter().any(|a| a.address() == &address) {
                        return Ok(account_handle.clone());
                    }
                }
            }
        };
        Err(crate::Error::AccountNotFound)
    }
    pub async fn get_accounts(&self) -> crate::Result<Vec<AccountHandle>> {
        Ok(self.accounts.read().await.clone())
    }
    pub async fn delete_account(&self, identifier: AccountIdentifier) -> crate::Result<()> {
        Ok(())
    }
    // search balance, recovery from mnemonic or balance finder
    pub async fn search_balance(
        &self,
        addresses_per_account: usize,
        account_start_index: usize,
    ) -> crate::Result<Vec<AccountHandle>> {
        Ok(vec![])
    }

    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        Ok(())
    }

    pub fn start_background_syncing(&self, options: SyncOptions) -> crate::Result<()> {
        Ok(())
    }
    pub fn stop_background_syncing(&self) -> crate::Result<()> {
        Ok(())
    }

    // listen to all wallet events
    // pub fn listen() -> crate::Result<(Sender<WalletEvent>, Receiver<WalletEvent>)> {}

    pub fn generate_mnemonic(&self) -> crate::Result<String> {
        Ok(Client::generate_mnemonic()?)
    }
    pub fn verify_mnemonic(&self, mnemonic: &str) -> crate::Result<()> {
        // first we check if the mnemonic is valid to give meaningful errors
        crypto::keys::bip39::wordlist::verify(mnemonic, &crypto::keys::bip39::wordlist::ENGLISH)
            .map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;
        Ok(())
    }

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
