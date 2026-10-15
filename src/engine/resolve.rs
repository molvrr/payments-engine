use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessingError;
use crate::operation::ResolveData;
use crate::transaction::{Transaction, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    resolve_data: ResolveData,
) -> Result<(), ProcessingError> {
    let transaction = transactions.get_mut(&resolve_data.transaction_id).ok_or(
        ProcessingError::TransactionNotFound(resolve_data.transaction_id),
    )?;

    if transaction.client_id != resolve_data.client_id {
        return Err(ProcessingError::ClientMismatch {
            transaction_id: resolve_data.transaction_id,
            client_id: resolve_data.client_id,
        });
    }

    if transaction.status != TransactionStatus::Disputed {
        return Err(ProcessingError::NotDisputed(resolve_data.transaction_id));
    }

    let account = accounts
        .get_mut(&resolve_data.client_id)
        .ok_or(ProcessingError::AccountNotFound(resolve_data.client_id))?;

    // Cannot overflow: moves funds from held back to available, total unchanged.
    account.available += transaction.amount;
    account.held -= transaction.amount;
    transaction.status = TransactionStatus::Resolved;

    Ok(())
}
