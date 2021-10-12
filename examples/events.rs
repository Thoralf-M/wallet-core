// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example events --release

use wallet_core::{
    account_manager::AccountManager, client::options::ClientOptionsBuilder, signing::SignerType, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let manager = AccountManager::builder().finish().await?;

    manager
        .listen(vec![], move |event| {
            println!("Received an event {:?}", event);
        })
        .await;

    // Get account or create a new one
    let account_alias = "event_account";
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

    let _address = account.generate_addresses(5, None).await?;

    let balance = account.sync(None).await?;
    println!("Balance: {:?}", balance);

    Ok(())
}
