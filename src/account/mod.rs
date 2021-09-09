

// Wrapper so we can lock the account during operations
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
}

impl AccountHandle {
    pub async fn sync(SyncOptions) -> Result<Balance,available_balance>{
        // ignore outputs from other networks
        sync_addresses_balance()
        // get outputs only for addresses that have != 0 as balance
        sync_addresses_outputs()
        // check if outputs are unspent, rebroadcast, reattach...
        sync_transactions(){
            retry(message_id, sync: false)
        }
        // only when actively called or also in the background syncing?
        consolidate_outputs()
    }
    async fn sync_addresses_balance(&Account) -> Result<Vec<Adress>>
    async fn sync_addresses_outputs(&Account) -> Result<Vec<Output>>
    async fn sync_transactions(&Account) -> Result<Vec<Transaction>>
    async fn consolidate_outputs(&Account) -> Result<Vec<Transaction>>

    async fn select_inputs(amount) -> Result<Outputs>{}
    pub async fn send([TransferOutput(address, amount, output_type)], TransferOptions(remainder_strategy, custom_inputs)) -> Result<Message>{}
    pub async fn retry(message_id, sync: bool) -> Result<message_id>{}
    
    pub async fn generate_addresses(amount) -> Result<Vec<Address>>{}
    pub async fn list_addresses() -> Result<Vec<Address>>{}
    // Should only be called from the AccountManager
    pub(crate) async fn set_client_options(ClientOptions) -> Result<()>{}
}


pub struct Account {
    identifier: AccountIdentifier,
    addresses: Vec<Address>,
    outputs: HashMap<Address, Vec<Output>>,
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

pub struct Output {
    pub transaction_id: TransactionId,
    pub message_id: MessageId,
    pub index: u16,
    pub amount: u64,
    pub is_spent: bool,
    pub address: AddressWrapper,
    pub kind: OutputKind,
}

struct TransferOptions {
    remainder_value_strategy: RemainderValueStrategy,
    indexation: Option<IndexationDto>,
    skip_sync: bool,
    output_kind: Option<OutputKind>,
}

struct Transaction {
    pub transaction: TransactionPayload,
    pub inputs: Vec<Output>,
    pub attachments: Vec<MessageId>,
    pub confirmed: bool,
    //network id to ignore outputs when set_client_options is used to switch to another network 
    pub network_id: String,
}