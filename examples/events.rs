// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example events --release

use wallet_core::{
    account::{types::OutputKind, RemainderValueStrategy, TransferOptions, TransferOutput},
    account_manager::AccountManager,
    client::options::ClientOptionsBuilder,
    signing::SignerType,
    Result,
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
                .with_node_sync_disabled()
                .finish()
                .unwrap();
            manager
                .create_account()
                .with_client_options(client_options)
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let _address = account.generate_addresses(5, None).await?;

    let balance = account.sync(None).await?;
    println!("Balance: {:?}", balance);

    // send transaction
    let outputs = vec![TransferOutput {
        address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
        amount: 1_000_000,
        // we create a dust allowance outputs so we can reuse our address even with remainder
        output_kind: Some(OutputKind::SignatureLockedDustAllowance),
    }];
    // let res = account.send(outputs, None).await?;
    let res = account
        .send(
            outputs,
            Some(TransferOptions {
                remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
                ..Default::default()
            }),
        )
        .await?;
    println!(
        "Message sent: https://explorer.iota.org/devnet/message/{}",
        res.0.expect("No message created")
    );

    Ok(())
}
