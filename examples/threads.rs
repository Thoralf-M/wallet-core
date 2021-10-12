// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example threads --release

// In this example we will try to send transactions from multiple threads simultaneously

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

    // Get account or create a new one
    let account_alias = "thread_account";
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

    let mut threads = Vec::new();
    for n in 0..20 {
        let account_ = account.clone();
        threads.push(async move {
            tokio::spawn(async move {
                // send transaction
                let outputs = vec![TransferOutput {
                    address: "atoi1qq42e54sldwkg8jnd87hq6t82pcxllquwkfs94k2esejfxm7fpl4k5k9gy0".to_string(),
                    amount: 1_000_000,
                    // we create a dust allowance outputs so we can reuse the address even with remainder
                    output_kind: Some(OutputKind::SignatureLockedDustAllowance),
                }];
                let message_id = account_
                    .send(
                        outputs,
                        Some(TransferOptions {
                            remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
                            ..Default::default()
                        }),
                    )
                    .await?;
                println!(
                    "Message from thread {} sent: https://explorer.iota.org/devnet/message/{}",
                    n, message_id
                );
                wallet_core::Result::Ok(n)
            })
            .await
        });
    }

    let results = futures::future::try_join_all(threads).await?;
    for thread in results {
        match thread {
            Ok(res) => println!("{}", res),
            Err(e) => println!("{}", e),
        }
    }

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(())
}
