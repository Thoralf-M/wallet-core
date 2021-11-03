// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Storage adapter.
pub mod adapter;
/// Storage manager.
pub mod manager;
use adapter::StorageAdapter;

use crypto::ciphers::{chacha::XChaCha20Poly1305, traits::Aead};

use serde::Serialize;

use std::{
    collections::HashMap,
    convert::TryInto,
    io::{Read, Write},
    path::PathBuf,
};

struct Storage {
    storage_path: PathBuf,
    inner: Box<dyn StorageAdapter + Sync + Send>,
    encryption_key: Option<[u8; 32]>,
}

impl Storage {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    async fn get(&self, key: &str) -> crate::Result<String> {
        self.inner.get(key).await.and_then(|record| {
            if let Some(key) = &self.encryption_key {
                if serde_json::from_str::<Vec<u8>>(&record).is_ok() {
                    decrypt_record(&record, key)
                } else {
                    Ok(record)
                }
            } else {
                Ok(record)
            }
        })
    }

    async fn set<T: Serialize>(&mut self, key: &str, record: T) -> crate::Result<()> {
        let record = serde_json::to_string(&record)?;
        self.inner
            .set(
                key,
                if let Some(key) = &self.encryption_key {
                    let mut output = Vec::new();
                    encrypt_record(record.as_bytes(), key, &mut output)?;
                    serde_json::to_string(&output)?
                } else {
                    record
                },
            )
            .await
    }

    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        self.inner
            .batch_set(if let Some(key) = &self.encryption_key {
                let mut encrypted_records = HashMap::new();
                for (id, record) in records {
                    let mut output = Vec::new();
                    encrypt_record(record.as_bytes(), key, &mut output)?;
                    encrypted_records.insert(id, serde_json::to_string(&output)?);
                }
                encrypted_records
            } else {
                records
            })
            .await
    }

    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        self.inner.remove(key).await
    }
}

fn encrypt_record<O: Write>(record: &[u8], encryption_key: &[u8; 32], output: &mut O) -> crate::Result<()> {
    let mut nonce = [0; XChaCha20Poly1305::NONCE_LENGTH];
    crypto::utils::rand::fill(&mut nonce).map_err(|e| crate::Error::RecordEncrypt(format!("{:?}", e)))?;

    let mut tag = vec![0; XChaCha20Poly1305::TAG_LENGTH];
    let mut ciphertext = vec![0; record.len()];
    // we can unwrap here since we know the lengths are valid
    XChaCha20Poly1305::encrypt(
        encryption_key.try_into().unwrap(),
        &nonce.try_into().unwrap(),
        &[],
        record,
        &mut ciphertext,
        tag.as_mut_slice().try_into().unwrap(),
    )
    .map_err(|e| crate::Error::RecordEncrypt(format!("{:?}", e)))?;

    output.write_all(&nonce)?;
    output.write_all(&tag)?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub(crate) fn decrypt_record(record: &str, encryption_key: &[u8; 32]) -> crate::Result<String> {
    let record: Vec<u8> = serde_json::from_str(record)?;
    let mut record: &[u8] = &record;

    let mut nonce = [0; XChaCha20Poly1305::NONCE_LENGTH];
    record.read_exact(&mut nonce)?;

    let mut tag = vec![0; XChaCha20Poly1305::TAG_LENGTH];
    record.read_exact(&mut tag)?;

    let mut ct = Vec::new();
    record.read_to_end(&mut ct)?;

    let mut pt = vec![0; ct.len()];
    // we can unwrap here since we know the lengths are valid
    XChaCha20Poly1305::decrypt(
        encryption_key.try_into().unwrap(),
        &nonce.try_into().unwrap(),
        &[],
        &mut pt,
        &ct,
        tag.as_slice().try_into().unwrap(),
    )
    .map_err(|e| crate::Error::RecordDecrypt(format!("{:?}", e)))?;

    Ok(String::from_utf8_lossy(&pt).to_string())
}
