use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessError;
use crate::operation::ResolveData;
use crate::transaction::{Transaction, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    resolve_data: ResolveData,
) -> Result<(), ProcessError> {
    let transaction_opt = transactions.get_mut(&resolve_data.transaction_id);

    if let Some(transaction) = transaction_opt {
        if transaction.client_id != resolve_data.client_id {
            // TODO: Log different client
            return Ok(());
        }

        if transaction.status != TransactionStatus::Disputed {
            // TODO: Log different status
            return Ok(());
        }

        if let Some(account) = accounts.get_mut(&resolve_data.client_id) {
            account.available += transaction.amount;
            account.held -= transaction.amount;
            transaction.status = TransactionStatus::Resolved;
        }
    }

    Ok(())
}
