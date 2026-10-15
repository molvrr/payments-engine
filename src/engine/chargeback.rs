use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessError;
use crate::operation::ChargebackData;
use crate::transaction::{Transaction, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    chargeback_data: ChargebackData,
) -> Result<(), ProcessError> {
    let transaction_opt = transactions.get_mut(&chargeback_data.transaction_id);

    if let Some(transaction) = transaction_opt {
        if transaction.client_id != chargeback_data.client_id {
            // TODO: Log different client
            return Ok(());
        }

        if transaction.status != TransactionStatus::Disputed {
            // TODO: Log different status
            return Ok(());
        }

        if let Some(account) = accounts.get_mut(&chargeback_data.client_id) {
            account.held -= transaction.amount;
            account.locked = true;
            transaction.status = TransactionStatus::ChargedBack;
        }
    }

    Ok(())
}
