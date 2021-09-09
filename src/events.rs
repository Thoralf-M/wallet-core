
pub enum WalletEvent{
    BalanceChange(BalanceChangeEvent),
    TransactionInclusion(TransactionInclusionEvent),
    TransferProgress(TransferProgressEvent),
    ConsolidationRequired(Account_Id),
}

pub struct BalanceChangeEvent {
    /// Associated account.
    account_id: String,
    /// The address.
    address: AddressWrapper,
    /// The balance change data.
    balance_change: i64,
    /// the output/transaction?
}

pub struct TransactionInclusionEvent {
    transaction_id: TransactionId,
    inclusion_state: InclusionState
}

pub enum InclusionState {
    Confirmed,
    Conflicting,
    Unkown // do we need this for a case like tx created, then the wallet was offline until the node snapshotted the tx?
}

pub struct TransferProgress {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The transfer event type.
    pub event: TransferProgressType,
}

pub enum TransferProgressType {
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