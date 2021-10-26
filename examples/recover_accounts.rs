// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example recover_accounts --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use log::LevelFilter;
use std::time::Instant;
use wallet_core::{account_manager::AccountManager, client::options::ClientOptionsBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    let output_config = LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .level_filter(LevelFilter::Debug);
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config).unwrap();

    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
        // .with_node("https://chrysalis-nodes.iota.org/")?
        // .with_node("http://localhost:14265")?
        .with_node_sync_disabled()
        .finish()
        .unwrap();

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .finish()
        .await?;
    // manager.set_stronghold_password("password").await?;

    // Get account or create a new one
    let account_alias = "logger";
    let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;

    let accounts = manager.recover_accounts(account_alias).await?;

    let accounts = manager.get_accounts().await?;
    println!("Accounts: {:?}", accounts);

    // get latest account
    let account = accounts[accounts.len() - 1];
    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    let addresses_with_balance = account.list_addresses_with_balance().await?;
    println!("Addresses with balance: {}", addresses_with_balance.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    Ok(())
}
