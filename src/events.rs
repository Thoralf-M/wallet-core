use crate::account::types::AddressWrapper;
use iota_client::bee_message::payload::transaction::TransactionId;

use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WalletEvent {
    BalanceChange(BalanceChangeEvent),
    TransactionInclusion(TransactionInclusionEvent),
    TransferProgress(TransferProgressEvent),
    // account index
    ConsolidationRequired(usize),
}

#[derive(Serialize, Deserialize)]
pub struct BalanceChangeEvent {
    /// Associated account.
    account_id: String,
    /// The address.
    address: AddressWrapper,
    /// The balance change data.
    balance_change: i64,
    /// Total account balance
    new_balance: u64,
    // the output/transaction?
}

#[derive(Serialize, Deserialize)]
pub struct TransactionInclusionEvent {
    transaction_id: TransactionId,
    inclusion_state: InclusionState,
}
#[derive(Serialize, Deserialize)]
pub enum InclusionState {
    Confirmed,
    Conflicting,
    Unkown, // do we need this for a case like tx created, then the wallet was offline until the node snapshotted the tx?
}

#[derive(Serialize, Deserialize)]
pub struct TransferProgressEvent {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The transfer status.
    pub status: TransferStatusType,
}

#[derive(Serialize, Deserialize)]
pub enum TransferStatusType {
    /// Syncing account.
    SyncingAccount,
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress(AddressData),
    /// Prepared transaction.
    PreparedTransaction(PreparedTransactionData),
    /// Signing the transaction.
    SigningTransaction,
    /// Performing PoW.
    PerformingPoW,
    /// Broadcasting.
    Broadcasting,
}

#[derive(Serialize, Deserialize)]
pub struct AddressConsolidationNeeded {
    /// The associated account identifier.
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// The associated address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub address: AddressWrapper,
}

#[derive(Serialize, Deserialize)]
pub struct LedgerAddressGeneration {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The transfer event type.
    pub event: AddressData,
}

/// Address event data.
#[derive(Serialize, Deserialize, Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct AddressData {
    /// The address.
    #[getset(get = "pub")]
    pub address: String,
}

/// Prepared transaction event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct PreparedTransactionData {
    /// Transaction inputs.
    pub inputs: Vec<TransactionIO>,
    /// Transaction outputs.
    pub outputs: Vec<TransactionIO>,
    /// The indexation data.
    pub data: Option<String>,
}

/// Input or output data for PreparedTransactionData
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransactionIO {
    /// Address
    pub address: String,
    /// Amount
    pub amount: u64,
    /// Remainder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remainder: Option<bool>,
}
