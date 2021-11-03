// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::handle::AccountHandle, account_manager::AccountManager};

/// Find accounts with balances
/// `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address
/// has balance, the counter is reset
/// `account_gap_limit` defines how many accounts without balance will be
/// checked, if an account has balance, the counter is reset
pub async fn recover_accounts(
    account_manager: &AccountManager,
    address_gap_limit: usize,
    account_gap_limit: usize,
) -> crate::Result<Vec<AccountHandle>> {
    log::debug!("[recover_accounts]");
    let mut old_accounts = Vec::new();
    let old_accounts_len = account_manager.accounts.read().await.len();
    if old_accounts_len != 0 {
        // Search for addresses in current accounts, rev() because we do that later with the new accounts and want
        // to have it all ordered at the end
        for account in account_manager.accounts.read().await.iter() {
            account.search_addresses_with_funds(address_gap_limit).await?;
            old_accounts.push(account.clone());
        }
    }
    // Count accounts with zero balances in a row
    let mut zero_balance_accounts_in_row = 0;
    let mut generated_accounts = Vec::new();
    loop {
        log::debug!("[recover_accounts] generating new account");
        let new_account = account_manager.create_account().finish().await?;
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

    let mut accounts = account_manager.accounts.write().await;
    old_accounts.append(&mut new_accounts);
    *accounts = old_accounts;
    drop(accounts);

    Ok(account_manager.accounts.read().await.clone())
}
