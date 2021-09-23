use crate::account::{
    account_handle::AccountHandle,
    types::address::{parse_bech32_address, AccountAddress, AddressWrapper},
    Account,
};

use iota_client::Seed;

use std::collections::HashSet;

pub async fn generate_addresses(account_handle: &AccountHandle, amount: usize) -> crate::Result<Vec<AccountAddress>> {
    log::debug!("[ADDRESS GENERATION] generating {} addresses", amount);
    let mut account = account_handle.write().await;
    // todo use SignerType
    let client_guard = crate::client::get_client(&account.client_options).await?;
    let client = client_guard.read().await;

    let seed = Seed::from_bytes(&[
        37, 106, 129, 139, 42, 172, 69, 137, 65, 247, 39, 73, 133, 164, 16, 229, 127, 183, 80, 243, 163, 166, 121, 105,
        236, 229, 189, 154, 231, 238, 245, 178,
    ]);
    let addresses = client
        .get_addresses(&seed)
        .with_account_index(0)
        .with_range(0..amount)
        .finish()
        .await
        .unwrap();

    let account_addresses: Vec<AccountAddress> = addresses
        .into_iter()
        .enumerate()
        .map(|a| AccountAddress {
            address: parse_bech32_address(a.1).unwrap(),
            key_index: a.0,
            internal: false,
            outputs: HashSet::new(),
        })
        .collect();
    account.addresses.extend(account_addresses.clone());
    // store account to database if storage is used
    Ok(account_addresses)
}
