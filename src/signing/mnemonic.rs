// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use core::convert::TryInto;
use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    keys::{
        bip39::{mnemonic_to_seed, wordlist},
        slip10::{Chain, Curve, Seed},
    },
};
use iota_client::{
    bee_message::{
        prelude::{Address, Ed25519Address},
        unlock::{ReferenceUnlock, UnlockBlock},
    },
    Client,
};
use once_cell::sync::OnceCell;

use std::{
    collections::HashMap,
    ops::Range,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct MnemonicSigner;

static MNEMONIC_SEED: OnceCell<[u8; 64]> = OnceCell::new();

/// Sets the mnemonic
pub fn set_mnemonic(mnemonic: String) -> crate::Result<()> {
    // first we check if the mnemonic is valid to give meaningful errors
    wordlist::verify(&mnemonic, &wordlist::ENGLISH).map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;

    let mut mnemonic_seed = [0u8; 64];
    mnemonic_to_seed(&mnemonic, "", &mut mnemonic_seed);
    MNEMONIC_SEED.set(mnemonic_seed).expect("Coudln't set mnemonic seed");
    Ok(())
}

/// Gets the mnemonic
pub(crate) fn get_mnemonic_seed() -> crate::Result<Seed> {
    Ok(Seed::from_bytes(
        MNEMONIC_SEED.get().expect("Couldn't get mnemonic seed"),
    ))
}

fn generate_address(seed: &Seed, account_index: u32, address_index: u32, internal: bool) -> crate::Result<Address> {
    // 44 is for BIP 44 (HD wallets) and 4218 is the registered index for IOTA https://github.com/satoshilabs/slips/blob/master/slip-0044.md
    let chain = Chain::from_u32_hardened(vec![44, 4218, account_index, internal as u32, address_index]);
    let public_key = seed
        .derive(Curve::Ed25519, &chain)?
        .secret_key()
        .public_key()
        .to_bytes();
    // Hash the public key to get the address
    let result = Blake2b256::digest(&public_key)
        .try_into()
        .map_err(|_e| crate::Error::Blake2b256("Hashing the public key while generating the address failed."));

    Ok(Address::Ed25519(Ed25519Address::new(result?)))
}

#[async_trait::async_trait]
impl crate::signing::Signer for MnemonicSigner {
    async fn get_ledger_status(&self, _is_simulator: bool) -> crate::signing::LedgerStatus {
        // dummy status, function is only required in the trait because we need it for the LedgerSigner
        crate::signing::LedgerStatus {
            connected: false,
            locked: false,
            app: None,
        }
    }

    async fn store_mnemonic(&mut self, storage_path: &Path, mnemonic: String) -> crate::Result<()> {
        set_mnemonic(mnemonic)?;
        Ok(())
    }

    async fn generate_address(
        &mut self,
        account: &Account,
        address_index: usize,
        internal: bool,
        _: super::GenerateAddressMetadata,
    ) -> crate::Result<iota_client::bee_message::address::Address> {
        let seed = get_mnemonic_seed()?;
        generate_address(
            &seed,
            (*account.index()).try_into()?,
            address_index.try_into()?,
            internal,
        )
    }

    async fn sign_transaction<'a>(
        &mut self,
        account: &Account,
        essence: &iota_client::bee_message::prelude::Essence,
        inputs: &mut Vec<super::TransactionInput>,
        _: super::SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota_client::bee_message::unlock::UnlockBlock>> {
        // todo implement signing transaction

        //     let mut unlock_blocks = vec![];
        //     let mut signature_indexes = HashMap::<String, usize>::new();
        //     inputs.sort_by(|a, b| a.input.cmp(&b.input));

        //     for (current_block_index, recorder) in inputs.iter().enumerate() {
        //         let signature_index = format!("{}{}", recorder.address_index, recorder.address_internal);
        //         // Check if current path is same as previous path
        //         // If so, add a reference unlock block
        //         if let Some(block_index) = signature_indexes.get(&signature_index) {
        //             unlock_blocks.push(UnlockBlock::Reference(ReferenceUnlock::new(*block_index as u16)?));
        //         } else {
        //             // If not, we should create a signature unlock block
        //             let signature = crate::stronghold::sign_transaction(
        //                 &stronghold_path(account.storage_path()).await?,
        //                 &essence.hash(),
        //                 *account.index(),
        //                 recorder.address_index,
        //                 recorder.address_internal,
        //             )
        //             .await?;
        //             unlock_blocks.push(UnlockBlock::Signature(signature.into()));
        //             signature_indexes.insert(signature_index, current_block_index);
        //         }
        //     }
        //     Ok(unlock_blocks)
        return Err(crate::Error::InvalidTransactionId);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn set_get_mnemonic() {
        let mnemonic = iota_client::Client::generate_mnemonic().unwrap();
        let mut mnemonic_seed = [0u8; 64];
        crypto::keys::bip39::mnemonic_to_seed(&mnemonic, "", &mut mnemonic_seed);
        let set_mnemonic = super::set_mnemonic(mnemonic.clone()).unwrap();
        let get_mnemonic_seed = super::get_mnemonic_seed().unwrap();
        // we can't compare `Seed`, that's why we generate an address and compare if it's the same
        assert_eq!(
            super::generate_address(&crypto::keys::slip10::Seed::from_bytes(&mnemonic_seed), 0, 0, false).unwrap(),
            super::generate_address(&get_mnemonic_seed, 0, 0, false).unwrap()
        );
    }

    #[tokio::test]
    async fn addresses() {
        use crate::{
            account::builder::AccountBuilder,
            signing::{GenerateAddressMetadata, Network, Signer},
        };

        use std::path::Path;

        let mnemonic = "until fire hat mountain zoo grocery real deny advance change marble taste goat ivory wheat bubble panic banner tattoo client ticket action race rocket".to_string();
        super::MnemonicSigner
            .store_mnemonic(&Path::new(""), mnemonic)
            .await
            .unwrap();
        let account = AccountBuilder::new(0).finish().unwrap();
        let address = super::MnemonicSigner
            .generate_address(
                &account,
                0,
                false,
                GenerateAddressMetadata {
                    syncing: false,
                    network: Network::Testnet,
                },
            )
            .await
            .unwrap();

        assert_eq!(
            address.to_bech32("atoi"),
            "atoi1qq42e54sldwkg8jnd87hq6t82pcxllquwkfs94k2esejfxm7fpl4k5k9gy0".to_string()
        );
    }
}
