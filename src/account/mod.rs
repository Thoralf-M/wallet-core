pub(crate) mod account_builder;
pub(crate) mod account_handle;
pub(crate) mod operations;
pub(crate) mod types;

use crate::{
    account::types::{address::AccountAddress, AccountBalance, Output, Transaction},
    client::ClientOptions,
    signing::SignerType,
};

use getset::{Getters, Setters};
use iota_client::bee_message::address::Address;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// Account definition.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    #[getset(set = "pub(crate)")]
    id: String,
    /// The account index
    index: usize,
    /// The account alias.
    alias: String,
    /// The account's signer type.
    #[serde(rename = "signerType")]
    signer_type: SignerType,
    addresses: Vec<AccountAddress>,
    // stored separated from the account for performance?
    outputs: HashMap<Address, Vec<Output>>,
    // stored separated from the account for performance?
    transactions: Vec<Transaction>,
    client_options: ClientOptions,
    // sync interval, output consolidation
    #[getset(get = "pub(crate)")]
    account_options: AccountOptions,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct AccountOptions {
    pub(crate) output_consolidation_threshold: usize,
    pub(crate) automatic_output_consolidation: bool,
    /* #[cfg(feature = "storage")]
     * pub(crate) persist_events: bool, */
}
