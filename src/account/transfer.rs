use crate::account::input_selection::select_inputs;

pub struct TransferOutput {
    address: String,
    amount: u64,
    output_type: Option<OutputType>,
}
pub struct TransferOptions {
    remainder_value_strategy: RemainderValueStrategy,
    indexation: Option<IndexationDto>,
    skip_sync: bool,
    output_kind: Option<OutputKind>,
}

pub async fn send(
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> Result<MessageId> {
    select_inputs(amount: u64).await?;
    create_transaction().await?;
    sign_transaction().await?;
    send_transaction().await
}
async fn create_transaction() -> Result<TransactionEssence> {}
async fn sign_transaction() -> Result<Transaction> {}
async fn send_transaction() -> Result<MessageId> {}
