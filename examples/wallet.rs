// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example wallet --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use log::LevelFilter;
use std::time::Instant;
use wallet_core::{
    account::TransferOutput, account_manager::AccountManager, client::options::ClientOptionsBuilder,
    signing::SignerType, Result,
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
    let mnemonic = "until fire hat mountain zoo grocery real deny advance change marble taste goat ivory wheat bubble panic banner tattoo client ticket action race rocket".to_string();
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

    // let accounts = manager.get_accounts().await?;
    // println!("Accounts: {:?}", accounts);

    let _address = account.generate_addresses(4, None).await?;
    // println!("Generated a new address: {:?}", _address);

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    // send transaction
    let outputs = vec![TransferOutput {
        address: "atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r".to_string(),
        amount: 1_000_000,
        output_kind: None,
    }];
    let message_id = account.send(outputs, None).await?;
    println!("Message sent: https://explorer.iota.org/devnet/message/{}", message_id);

    // // switch to mainnet
    // let client_options = ClientOptionsBuilder::new()
    //     .with_node("https://chrysalis-nodes.iota.org/")?
    //     .with_node("https://chrysalis-nodes.iota.cafe/")?
    //     .with_node_sync_disabled()
    //     .finish()
    //     .unwrap();
    // manager.set_client_options(client_options).await?;
    // let now = Instant::now();
    // let balance = account.sync(None).await?;
    // println!("Syncing took: {:.2?}", now.elapsed());
    // println!("Balance: {:?}", balance);

    // // switch back to testnet
    // let client_options = ClientOptionsBuilder::new()
    //     .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
    //     .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
    //     .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
    //     .with_node_sync_disabled()
    //     .finish()
    //     .unwrap();
    // manager.set_client_options(client_options).await?;
    // let now = Instant::now();
    // let balance = account.sync(None).await?;
    // println!("Syncing took: {:.2?}", now.elapsed());
    // println!("Balance: {:?}", balance);

    Ok(())
}
