// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example wallet --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use log::LevelFilter;
use std::time::Instant;
use wallet_core::{account_manager::AccountManager, client::ClientOptionsBuilder, signing::SignerType, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    let output_config = LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .level_filter(LevelFilter::Debug);
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config).unwrap();

    let manager = AccountManager::builder().finish().await?;
    // manager.set_stronghold_password("password").await?;

    // Get account or create a new one
    let account_alias = "logger";
    let account = match manager.get_account(account_alias.to_string()).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            manager.store_mnemonic(SignerType::Mnemonic, None).await.unwrap();
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
                .finish()
                .unwrap();
            manager
                .create_account(Some(client_options))
                // .alias(account_alias)
                // .initialise()
                .await?
        }
    };

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());

    println!("Balance: {:?}", balance);

    // let addresses = account.list_addresses().await?;
    // println!("Addresses: {}", addresses.len());

    // let address = account.generate_address().await?;
    // println!("Generated a new address: {:?}", address);

    Ok(())
}
