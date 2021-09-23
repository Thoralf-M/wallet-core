pub(crate) mod address;
use address::{parse_bech32_address, AddressWrapper};

use iota_client::bee_message::{
    payload::transaction::{TransactionId, TransactionPayload},
    MessageId,
};
use serde::{Deserialize, Deserializer, Serialize};

use std::str::FromStr;

const ACCOUNT_ID_PREFIX: &str = "wallet-account://";

/// The account identifier.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum AccountIdentifier {
    /// An address identifier.
    #[serde(with = "crate::serde::iota_address_serde")]
    Address(AddressWrapper),
    // SHA-256 hash of the first address on the seed (m/44'/0'/0'/0'/0'). Required for referencing a seed in
    // Stronghold. The id should be provided by Stronghold. can we do the hashing only during interaction with
    // Stronghold? Then we could use the first address instead which could be useful
    Id(String),
    /// Account alias as identifier.
    Alias(String),
    /// An index identifier.
    Index(usize),
}

impl<'de> Deserialize<'de> for AccountIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(AccountIdentifier::from(s))
    }
}

// When the identifier is a string id.
impl From<&str> for AccountIdentifier {
    fn from(value: &str) -> Self {
        if let Ok(address) = parse_bech32_address(value) {
            Self::Address(address)
        } else if value.starts_with(ACCOUNT_ID_PREFIX) {
            Self::Id(value.to_string())
        } else {
            Self::Alias(value.to_string())
        }
    }
}

impl From<String> for AccountIdentifier {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for AccountIdentifier {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

// When the identifier is an index.
impl From<usize> for AccountIdentifier {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountBalance {
    pub(crate) total: u64,
    pub(crate) available: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub transaction_id: TransactionId,
    pub message_id: MessageId,
    pub index: u16,
    pub amount: u64,
    pub is_spent: bool,
    pub address: AddressWrapper,
    pub kind: OutputKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction: TransactionPayload,
    pub inputs: Vec<Output>,
    pub attachments: Vec<MessageId>,
    pub confirmed: bool,
    // network id to ignore outputs when set_client_options is used to switch to another network
    pub network_id: String,
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