use crate::client::ClientOptions;

pub(crate) mod input_selection;
pub(crate) mod syncing;
pub(crate) mod transfer;

use transfer::{TransferOptions, TransferOutput};

use tokio::sync::RwLock;

use std::sync::Arc;
// Wrapper so we can lock the account during operations
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
}

impl AccountHandle {
    pub async fn sync(&self, options: syncing::SyncOptions) -> Result<AccountBalance> {
        syncing::sync(&self, options)
    }

    async fn consolidate_outputs(account: &Account) -> Result<Vec<Transaction>> {}

    pub async fn send(
        outputs: Vec<TransferOutput>,
        options: Option<TransferOptions>,
    ) -> Result<MessageId> {
        transfer::send()
    }

    pub async fn retry(message_id: MessageId, sync: bool) -> Result<MessageId> {}

    pub async fn generate_addresses(amount: usize) -> Result<Vec<Address>> {}
    pub async fn list_addresses() -> Result<Vec<Address>> {}
    pub fn balance() -> Result<AccountBalance> {}

    // Should only be called from the AccountManager so all accounts use the same options
    pub(crate) async fn set_client_options(options: ClientOptions) -> Result<()> {}
}

pub struct Account {
    identifier: AccountIdentifier,
    addresses: Vec<Address>,
    // stored separated from the account for performance?
    outputs: HashMap<Address, Vec<Output>>,
    // stored separated from the account for performance?
    transactions: Vec<Transaction>,
    client_options: ClientOptions,
    // sync interval, output consolidation
    account_options: AccountOptions,
}

pub enum AccountIdentifier {
    // SHA-256 hash of the first address on the seed (m/44'/0'/0'/0'/0'). Required for referencing a seed in Stronghold. The id should be provided by Stronghold.
    // can we do the hashing only during interaction with Stronghold? Then we could use the first address instead which could be useful
    Id(String),
    /// Account alias as identifier.
    Alias(String),
    /// An index identifier.
    Index(usize),
}

struct AccountBalance {
    total: u64,
    available: u64,
}

pub struct Output {
    pub transaction_id: TransactionId,
    pub message_id: MessageId,
    pub index: u16,
    pub amount: u64,
    pub is_spent: bool,
    pub address: AddressWrapper,
    pub kind: OutputKind,
}

struct Transaction {
    pub transaction: TransactionPayload,
    pub inputs: Vec<Output>,
    pub attachments: Vec<MessageId>,
    pub confirmed: bool,
    //network id to ignore outputs when set_client_options is used to switch to another network
    pub network_id: String,
}
