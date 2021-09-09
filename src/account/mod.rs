

// Wrapper so we can lock the account during operations
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
}

impl AccountHandle {
    pub async fn sync() -> Result<Balance,available_balance>{
        sync_addresses_balance()
        // get outputs only for address that have != 0 as balance
        sync_addresses_outputs()
        // check if valid, rebroadcast, reattach...
        sync_transactions()
        // only when acitvely called or also in the background syncing?
        consolidate_outputs()
    }
    pub async fn send([TransferOutput(address, amount, output_type)], TransferOptions(remainder_strategy, custom_inputs)) -> Result<Message>{}
    pub async fn generate_addresses(amount) -> Result<Vec<Address>>{}
    pub async fn list_addresses() -> Result<Vec<Address>>{}

}


struct Account {
    addresses: Vec<Address>,
    outputs: Vec<Output>,
    transactions: Vec<TransactionPayload>,
    client_options: ClientOptions,
    // sync interval, output consolidation
    account_options: AccountOptions,
}