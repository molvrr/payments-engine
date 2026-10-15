use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessError;
use crate::operation::DisputeData;
use crate::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    dispute_data: DisputeData,
) -> Result<(), ProcessError> {
    let transaction_opt = transactions.get_mut(&dispute_data.transaction_id);

    if let Some(transaction) = transaction_opt {
        if transaction.client_id != dispute_data.client_id {
            // TODO: Log different client
            return Ok(());
        }

        if transaction.status != TransactionStatus::Completed
            || transaction.kind == TransactionKind::Debit
        {
            // TODO: Log transaction status
            return Ok(());
        }

        if let Some(account) = accounts.get_mut(&dispute_data.client_id) {
            account.available -= transaction.amount;
            account.held += transaction.amount;
            transaction.status = TransactionStatus::Disputed;
        }
    }

    Ok(())
}
