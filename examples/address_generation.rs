// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example address_generation --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use log::LevelFilter;
use std::time::Instant;
use wallet_core::{
    account_manager::AccountManager, client::options::ClientOptionsBuilder, signing::SignerType, Result,
};

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
    let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally".to_string();
    manager.store_mnemonic(SignerType::Mnemonic, Some(mnemonic)).await?;
    let account = match manager.get_account(account_alias.to_string()).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
                .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
                .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
                // .with_node("https://chrysalis-nodes.iota.org/")?
                // .with_node("http://localhost:14265")?
                .with_node_sync_disabled()
                .finish()
                .unwrap();
            manager
                .create_account(Some(client_options))
                // .alias(account_alias)
                // .initialise()
                .await?
        }
    };

    let addresses = account.generate_addresses(5, None).await?;
    let mut bech32_addresses = Vec::new();
    for address in addresses {
        bech32_addresses.push(address.address().to_bech32());
    }
    println!("Generated new addresses: {:#?}", bech32_addresses);
    // generate internal addresses because they are used for the remainder
    // let _address = account
    //     .generate_addresses(
    //         20,
    //         Some(AddressGenerationOptions {
    //             internal: true,
    //             ..Default::default()
    //         }),
    //     )
    //     .await?;

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    Ok(())
}
