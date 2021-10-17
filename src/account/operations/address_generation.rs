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
    let mut account = account_handle.write().await;
    let signer = crate::signing::get_signer(&account.signer_type).await;
    let mut signer = signer.lock().await;

    // get the highest index for the public or internal addresses
    let highest_current_index_plus_one = if options.internal {
        account.internal_addresses.len()
    } else {
        account.public_addresses.len()
    };

    // get bech32_hrp
    let bech32_hrp = {
        match account.public_addresses.first() {
            Some(address) => address.address.bech32_hrp.to_string(),
            // Only when we create a new account we don't have the first address and need to get the information from
            // the client Doesn't work for offline creating, should we use the network from the
            // GenerateAddressMetadata instead to use `iota` or `atoi`?
            None => {
                let client_guard = client::get_client(&account.client_options).await?;
                let bech32_hrp = client_guard.read().await.get_bech32_hrp().await?;
                bech32_hrp
            }
        }
    };
    let mut generate_addresses = Vec::new();
    for address_index in highest_current_index_plus_one..highest_current_index_plus_one + amount {
        let address = signer
            .generate_address(&account, address_index, options.internal, options.metadata.clone())
            .await?;
        generate_addresses.push(AccountAddress {
            address: AddressWrapper::new(address, bech32_hrp.clone()),
            key_index: address_index,
            internal: options.internal,
            used: false,
        });
    }

    // add addresses to the account
    if options.internal {
        account.internal_addresses.extend(generate_addresses.clone());
    } else {
        account.public_addresses.extend(generate_addresses.clone());
    };
    // todo: store account to database if storage is used
    Ok(generate_addresses)
}
