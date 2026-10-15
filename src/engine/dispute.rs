use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessingError;
use crate::operation::DisputeData;
use crate::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    dispute_data: DisputeData,
) -> Result<(), ProcessingError> {
    let transaction = transactions.get_mut(&dispute_data.transaction_id).ok_or(
        ProcessingError::TransactionNotFound(dispute_data.transaction_id),
    )?;

    if transaction.client_id != dispute_data.client_id {
        return Err(ProcessingError::ClientMismatch {
            transaction_id: dispute_data.transaction_id,
            client_id: dispute_data.client_id,
        });
    }

    if transaction.status != TransactionStatus::Completed
        || transaction.kind != TransactionKind::Credit
    {
        return Err(ProcessingError::NotDisputable(dispute_data.transaction_id));
    }

    let account = accounts
        .get_mut(&dispute_data.client_id)
        .ok_or(ProcessingError::AccountNotFound(dispute_data.client_id))?;

    // held can exceed total when available is negative, so it can overflow.
    let (Some(available), Some(held)) = (
        account.available.checked_sub(transaction.amount),
        account.held.checked_add(transaction.amount),
    ) else {
        return Err(ProcessingError::Overflow(dispute_data.client_id));
    };

    account.available = available;
    account.held = held;
    transaction.status = TransactionStatus::Disputed;

    Ok(())
}
