use crate::account::Account;

use iota_client::bee_message::output::OutputId;

pub(crate) async fn select_inputs(
    account: &Account,
    amount: u64,
    custom_inputs: Option<Vec<OutputId>>,
) -> crate::Result<Vec<crate::account::types::OutputData>> {
    // if custom inputs are provided we should only use them (validate if we have the outputs in this account and that
    // the amount is enough) and not others
    Ok(vec![])
}
