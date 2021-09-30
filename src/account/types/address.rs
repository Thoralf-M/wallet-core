use getset::{Getters, Setters};
use iota_client::bee_message::{address::Address, output::OutputId};
use serde::{Deserialize, Serialize, Serializer};

use std::{collections::HashSet, hash::Hash};

/// An account address.
#[derive(Debug, Getters, Setters, Clone, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct AccountAddress {
    /// The address.
    #[serde(with = "crate::account::types::address_serde")]
    pub(crate) address: AddressWrapper,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    #[getset(set = "pub(crate)")]
    pub(crate) key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    pub(crate) internal: bool,
    // /// The address outputs.
    // //make this a HashSet to store the outputs separated? add the network id here?
    // #[getset(set = "pub(crate)")]
    // pub(crate) outputs: HashMap<OutputId, AddressOutput>,
    // should only hold the current unspent outputs, spent outputs can be retrieved from all outputs
    pub(crate) outputs: HashSet<OutputId>,
    /// Balance
    // do we want this field? if we want to keep it, do we need to validate that the amount of the outputs matches?
    // What happens if we don't get all outputs because of the API limit?
    pub(crate) balance: u64,
    // do we want this field? Could be useful if we don't store spent output ids and because of that wouldn't know if
    // an address was used or not just by looking at it
    pub(crate) used: bool,
}

/// An address and its network type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AddressWrapper {
    pub(crate) inner: Address,
    pub(crate) bech32_hrp: String,
}

impl AsRef<Address> for AddressWrapper {
    fn as_ref(&self) -> &Address {
        &self.inner
    }
}

impl AddressWrapper {
    /// Create a new address wrapper.
    pub fn new(address: Address, bech32_hrp: String) -> Self {
        Self {
            inner: address,
            bech32_hrp,
        }
    }

    /// Encodes the address as bech32.
    pub fn to_bech32(&self) -> String {
        self.inner.to_bech32(&self.bech32_hrp)
    }

    pub(crate) fn bech32_hrp(&self) -> &str {
        &self.bech32_hrp
    }
}
/// Parses a bech32 address string.
pub fn parse_bech32_address<A: AsRef<str>>(address: A) -> crate::Result<AddressWrapper> {
    let address = address.as_ref();
    let mut tokens = address.split('1');
    let hrp = tokens.next().ok_or(crate::Error::InvalidAddress)?;
    let address = iota_client::bee_message::address::Address::try_from_bech32(address)?;
    Ok(AddressWrapper::new(address, hrp.to_string()))
}
