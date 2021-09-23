use crate::account::Account;

use iota_client::bee_message::output::OutputId;

pub(crate) async fn select_inputs(
    account: &Account,
    amount: u64,
    custom_inputs: Option<Vec<OutputId>>,
) -> crate::Result<Vec<crate::account::types::Output>> {
    Ok(vec![])
}
