use crate::{
    account::{
        handle::AccountHandle,
        types::address::{AccountAddress, AddressWrapper},
    },
    client,
    signing::{GenerateAddressMetadata, Network},
};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressGenerationOptions {
    pub internal: bool,
    pub metadata: GenerateAddressMetadata,
}

impl Default for AddressGenerationOptions {
    fn default() -> Self {
        Self {
            internal: false,
            metadata: GenerateAddressMetadata {
                syncing: false,
                network: Network::Testnet,
            },
        }
    }
}

/// Generate addresses and stores them in the account
pub async fn generate_addresses(
    account_handle: &AccountHandle,
    amount: usize,
    options: AddressGenerationOptions,
) -> crate::Result<Vec<AccountAddress>> {
    log::debug!("[ADDRESS GENERATION] generating {} addresses", amount);
    let account = account_handle.read().await;
    let signer = crate::signing::get_signer(&account.signer_type).await;
    let mut signer = signer.lock().await;

    // get the highest index for the public or internal addresses so we don't generate the same addresses twice
    let address_with_highest_index = account
        .addresses
        .iter()
        .filter(|address| address.internal == options.internal)
        .max_by_key(|a| a.key_index);
    let highest_current_index_plus_one = match address_with_highest_index {
        // increase index by one because this will be the index for the new address
        Some(address) => address.key_index + 1,
        None => 0,
    };
    // todo: read bech32_hrp from first address and only get it from the client if we don't have any address (so only
    // for the first address)
    let client_guard = client::get_client(&account.client_options).await?;
    let bech32_hrp = client_guard.read().await.get_bech32_hrp().await?;
    let mut generate_addresses = Vec::new();
    for address_index in highest_current_index_plus_one..highest_current_index_plus_one + amount {
        let address = signer
            .generate_address(&account, address_index, options.internal, options.metadata.clone())
            .await?;
        generate_addresses.push(AccountAddress {
            address: AddressWrapper::new(address, bech32_hrp.clone()),
            key_index: address_index,
            internal: options.internal,
            outputs: HashSet::new(),
            balance: 0,
            used: false,
        });
    }
    drop(account);

    let mut account = account_handle.write().await;
    account.addresses.extend(generate_addresses.clone());
    // todo: store account to database if storage is used
    Ok(generate_addresses)
}
