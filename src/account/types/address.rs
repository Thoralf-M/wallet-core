use getset::{Getters, Setters};
use iota_client::bee_message::address::Address;
use serde::{Deserialize, Serialize, Serializer};

use std::hash::Hash;

/// An account address.
#[derive(Debug, Getters, Setters, Clone, Deserialize)]
#[getset(get = "pub")]
pub struct AccountAddress {
    /// The address.
    #[serde(with = "crate::serde::iota_address_serde")]
    address: AddressWrapper,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    #[getset(set = "pub(crate)")]
    key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    internal: bool,
    /* /// The address outputs.
     * //make this a HashSet to store the outputs separated?
     * #[getset(set = "pub(crate)")]
     * pub(crate) outputs: HashMap<OutputId, AddressOutput>, */
}

impl Serialize for AccountAddress {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct AddressDto<'a> {
            #[serde(with = "crate::serde::iota_address_serde")]
            address: &'a AddressWrapper,
            balance: u64,
            #[serde(rename = "keyIndex")]
            key_index: usize,
            internal: bool,
            // outputs: &'a HashMap<OutputId, AddressOutput>,
        }
        let address = AddressDto {
            address: &self.address,
            balance: self.balance(),
            key_index: self.key_index,
            internal: self.internal,
            // outputs: &self.outputs,
        };
        address.serialize(s)
    }
}

impl AccountAddress {
    /// Gets a new instance of the address builder.
    // #[doc(hidden)]
    // pub fn builder() -> AddressBuilder {
    //     AddressBuilder::new()
    // }

    // pub(crate) fn available_outputs(&self, sent_messages: &[Message]) -> Vec<&AddressOutput> {
    //     self.outputs
    //         .values()
    //         .filter(|o| !(o.is_spent || o.is_used(sent_messages)))
    //         .collect()
    // }

    // /// Address total balance
    // pub fn balance(&self) -> u64 {
    //     self.outputs
    //         .values()
    //         .fold(0, |acc, o| acc + if o.is_spent { 0 } else { *o.amount() })
    // }
    /// Address total balance
    pub fn balance(&self) -> u64 {
        0
    }

    // pub(crate) fn available_balance(&self, sent_messages: &[Message]) -> u64 {
    //     self.available_outputs(sent_messages)
    //         .iter()
    //         .fold(0, |acc, o| acc + *o.amount())
    // }

    // pub(crate) fn outputs_mut(&mut self) -> &mut HashMap<OutputId, AddressOutput> {
    //     &mut self.outputs
    // }

    // /// Updates the Bech32 human readable part.
    // #[doc(hidden)]
    // pub fn set_bech32_hrp(&mut self, hrp: String) {
    //     self.address.bech32_hrp = hrp.to_string();
    //     for output in self.outputs.values_mut() {
    //         output.address.bech32_hrp = hrp.to_string();
    //     }
    // }
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
