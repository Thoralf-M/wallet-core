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
    pub(crate) signer_type: SignerType,
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    /// Create a new account
    pub fn create_account(&self) -> AccountBuilder {
        log::debug!("creating account");
        AccountBuilder::new(self.accounts.clone(), self.signer_type.clone())
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

    /// Find accounts with balances
    /// `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address
    /// has balance, the counter is reset
    /// `account_gap_limit` defines how many accounts without balance will be
    /// checked, if an account has balance, the counter is reset
    pub async fn recover_accounts(
        &self,
        address_gap_limit: usize,
        account_gap_limit: usize,
    ) -> crate::Result<Vec<AccountHandle>> {
        log::debug!("[recover_accounts]");
        let mut old_accounts = Vec::new();
        let old_accounts_len = self.accounts.read().await.len();
        if old_accounts_len != 0 {
            // Search for addresses in current accounts, rev() because we do that later with the new accounts and want
            // to have it all ordered at the end
            for account in self.accounts.read().await.iter() {
                account.search_addresses_with_funds(address_gap_limit).await?;
                old_accounts.push(account.clone());
            }
        }
        // Count accounts with zero balances in a row
        let mut zero_balance_accounts_in_row = 0;
        let mut generated_accounts = Vec::new();
        loop {
            log::debug!("[recover_accounts] generating new account");
            let new_account = self.create_account().finish().await?;
            let account_balance = new_account.search_addresses_with_funds(address_gap_limit).await?;
            generated_accounts.push((new_account, account_balance.clone()));
            if account_balance.total == 0 {
                zero_balance_accounts_in_row += 1;
                if zero_balance_accounts_in_row >= account_gap_limit {
                    break;
                }
            } else {
                // reset if we found an account with balance
                zero_balance_accounts_in_row = 0;
            }
        }
        // delete accounts without balance
        let mut new_accounts = Vec::new();
        // iterate reversed to ignore all latest accounts that have no balance, but add all accounts that are below one
        // with balance
        for (account_handle, account_balance) in generated_accounts.iter().rev() {
            let account = account_handle.read().await;
            if !new_accounts.is_empty() || account_balance.total != 0 {
                new_accounts.push(account_handle.clone());
            }
        }
        new_accounts.reverse();

        let mut accounts = self.accounts.write().await;
        old_accounts.append(&mut new_accounts);
        *accounts = old_accounts;
        drop(accounts);

        Ok(self.accounts.read().await.clone())
    }

    /// Sets the client options for all accounts, syncs them and sets the new bech32_hrp
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        log::debug!("[set_client_options]");
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

    /// Sets the mnemonic for the signer, if none was provided, a random Bip39 mnemonic will be generated with the
    /// English word list and returned. Apart from a Stronghold backup it's the only way to recover funds, so save
    /// it securely. If you lose it, you potentially lose everything. With Stronghold this function needs to be
    /// called onnly once to initialize it, later the Stronghold password is required to use it.
    pub async fn store_mnemonic(&self, mnemonic: Option<String>) -> crate::Result<String> {
        let signer = crate::signing::get_signer().await;
        let mut signer = signer.lock().await;
        let mnemonic = match mnemonic {
            Some(m) => {
                self.verify_mnemonic(&m)?;
                m
            }
            None => self.generate_mnemonic()?,
        };
        signer
            .store_mnemonic(std::path::Path::new(""), mnemonic.clone())
            .await?;
        Ok(mnemonic)
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

#[cfg(test)]
mod tests {
    use crate::{account_manager::AccountManager, client::options::ClientOptionsBuilder, Result};
    // can't be run together with all other tests because there can be only one mnemonic at a time
    #[ignore]
    #[tokio::test]
    async fn account_recovery_empty() -> Result<()> {
        let client_options = ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
            .with_node_sync_disabled()
            .finish()?;

        let manager = AccountManager::builder()
            .with_client_options(client_options)
            .finish()
            .await?;

        // mnemonic without balance
        let mnemonic = "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string();
        manager.store_mnemonic(Some(mnemonic)).await?;

        let accounts = manager.recover_accounts(2, 2).await?;
        // accounts should be empty if no account was created before and no account was found with balance
        assert_eq!(0, accounts.len());
        Ok(())
    }

    // can't be run together with all other tests because there can be only one mnemonic at a time
    #[ignore]
    #[tokio::test]
    async fn account_recovery_existing_accounts() -> Result<()> {
        let client_options = ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
            .with_node_sync_disabled()
            .finish()?;

        let manager = AccountManager::builder()
            .with_client_options(client_options)
            .finish()
            .await?;

        // mnemonic without balance
        let mnemonic = "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string();
        manager.store_mnemonic(Some(mnemonic)).await?;

        // create two accounts
        manager.create_account().finish().await?;
        manager.create_account().finish().await?;

        let accounts = manager.recover_accounts(2, 2).await?;

        // accounts should still be ordered
        for (index, account) in accounts.iter().enumerate() {
            assert_eq!(&index, account.read().await.index());
        }
        // accounts should be 2 because we created 2 accounts before and no new account was found with balance
        assert_eq!(2, accounts.len());
        Ok(())
    }

    // can't be run together with all other tests because there can be only one mnemonic at a time
    #[ignore]
    #[tokio::test]
    async fn account_recovery_with_balance() -> Result<()> {
        let client_options = ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
            .with_node_sync_disabled()
            .finish()?;

        let manager = AccountManager::builder()
            .with_client_options(client_options)
            .finish()
            .await?;

        // mnemonic with balance on account with index 2 and address key_index 2 on the public address
        // atoi1qqt9tygh7h7s3l66m242hee6zwp98x90trejt9zya4vcnf5u34yluws9af6
        let mnemonic = "merit blame slam front add unknown winner wait matrix carbon lion cram picnic mushroom turn stadium bright wheel open tragic liar will law time".to_string();
        manager.store_mnemonic(Some(mnemonic)).await?;

        // create one account
        manager.create_account().finish().await?;

        let accounts = manager.recover_accounts(3, 2).await?;

        // accounts should still be ordered
        for (index, account) in accounts.iter().enumerate() {
            assert_eq!(&index, account.read().await.index());
        }
        // accounts should be 3 because account with index 2 has balance
        assert_eq!(3, accounts.len());

        let account_with_balance = accounts[2].read().await;
        // should have 3 addresses, index 0, 1, 2, all above should be deleted again
        assert_eq!(3, account_with_balance.public_addresses.len());
        Ok(())
    }

    // can't be run together with all other tests because there can be only one mnemonic at a time
    #[ignore]
    #[tokio::test]
    async fn account_recovery_with_balance_and_empty_addresses() -> Result<()> {
        let client_options = ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
            .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
            .with_node_sync_disabled()
            .finish()?;

        let manager = AccountManager::builder()
            .with_client_options(client_options)
            .finish()
            .await?;

        // mnemonic with balance on account with index 2 and address key_index 2 on the public address
        // atoi1qqt9tygh7h7s3l66m242hee6zwp98x90trejt9zya4vcnf5u34yluws9af6
        let mnemonic = "merit blame slam front add unknown winner wait matrix carbon lion cram picnic mushroom turn stadium bright wheel open tragic liar will law time".to_string();
        manager.store_mnemonic(Some(mnemonic)).await?;

        // create one account
        manager.create_account().finish().await?;
        manager.create_account().finish().await?;
        let account = manager.create_account().finish().await?;
        let addresses = account.generate_addresses(5, None).await?;

        let accounts = manager.recover_accounts(3, 2).await?;

        // accounts should still be ordered
        for (index, account) in accounts.iter().enumerate() {
            assert_eq!(&index, account.read().await.index());
        }
        // accounts should be 3 because account with index 2 has balance
        assert_eq!(3, accounts.len());

        let account_with_balance = accounts[2].read().await;
        // should have 10 addresses, because we generated 10 before, even thought they don't all have funds
        assert_eq!(5, account_with_balance.public_addresses.len());
        Ok(())
    }
}
