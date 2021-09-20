use iota_client::bee_message::address::Address;
use iota_client::bee_message::payload::transaction::TransactionId;
use iota_client::bee_message::payload::transaction::TransactionPayload;
use iota_client::bee_message::MessageId;
use serde::{Deserialize, Serialize};

use std::{hash::Hash, str::FromStr};

#[derive(Serialize, Deserialize)]
pub enum AccountIdentifier {
    // SHA-256 hash of the first address on the seed (m/44'/0'/0'/0'/0'). Required for referencing a seed in Stronghold. The id should be provided by Stronghold.
    // can we do the hashing only during interaction with Stronghold? Then we could use the first address instead which could be useful
    Id(String),
    /// Account alias as identifier.
    Alias(String),
    /// An index identifier.
    Index(usize),
}

#[derive(Serialize, Deserialize)]
pub struct AccountBalance {
    total: u64,
    available: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub transaction_id: TransactionId,
    pub message_id: MessageId,
    pub index: u16,
    pub amount: u64,
    pub is_spent: bool,
    pub address: AddressWrapper,
    pub kind: OutputKind,
}

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub transaction: TransactionPayload,
    pub inputs: Vec<Output>,
    pub attachments: Vec<MessageId>,
    pub confirmed: bool,
    //network id to ignore outputs when set_client_options is used to switch to another network
    pub network_id: String,
}

/// An address and its network type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AddressWrapper {
    pub(crate) inner: Address,
    pub(crate) bech32_hrp: String,
}

/// The address output kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub enum OutputKind {
    /// SignatureLockedSingle output.
    SignatureLockedSingle,
    /// Dust allowance output.
    SignatureLockedDustAllowance,
    /// Treasury output.
    Treasury,
}

impl FromStr for OutputKind {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "SignatureLockedSingle" => Self::SignatureLockedSingle,
            "SignatureLockedDustAllowance" => Self::SignatureLockedDustAllowance,
            "Treasury" => Self::Treasury,
            _ => return Err(crate::Error::InvalidOutputKind(s.to_string())),
        };
        Ok(kind)
    }
}
