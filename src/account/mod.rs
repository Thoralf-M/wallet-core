pub(crate) mod builder;
pub(crate) mod handle;
pub(crate) mod operations;
pub(crate) mod types;
pub use operations::address_generation::AddressGenerationOptions;

use crate::{
    account::types::{address::AccountAddress, AccountBalance, OutputData},
    client::options::ClientOptions,
    signing::SignerType,
};

use getset::{Getters, Setters};
use iota_client::bee_message::{address::Address, output::OutputId, payload::transaction::TransactionId};
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

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
    outputs: HashMap<Address, Vec<OutputData>>,
    // outputs used in transactions should be locked here so they don't get used again, resulting in conflicting
    // transactions
    locked_outputs: HashSet<OutputId>,
    // stored separated from the account for performance and only the transaction id here?
    transactions: HashSet<TransactionId>,
    // Maybe pending transactions even additionally separated?
    pending_transactions: HashSet<TransactionId>,
    client_options: ClientOptions,
    // sync interval, output consolidation
    #[getset(get = "pub(crate)")]
    account_options: AccountOptions,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct AccountOptions {
    pub(crate) background_syncing_interval: Duration,
    pub(crate) output_consolidation_threshold: usize,
    pub(crate) automatic_output_consolidation: bool,
    /* #[cfg(feature = "storage")]
     * pub(crate) persist_events: bool, */
}
