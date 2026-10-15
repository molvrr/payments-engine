use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessingError;
use crate::operation::DepositData;
use crate::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    deposit_data: DepositData,
) -> Result<(), ProcessingError> {
    let DepositData {
        client_id,
        transaction_id,
        amount,
    } = deposit_data;

    if transactions.contains_key(&transaction_id) {
        return Err(ProcessingError::DuplicateTransaction(transaction_id));
    }

    let account = accounts
        .get_mut(&client_id)
        .ok_or(ProcessingError::AccountNotFound(client_id))?;

    if account.locked {
        return Err(ProcessingError::LockedAccount(client_id));
    }

    // Check the total (available + held), not only available: otherwise a
    // client with held funds could push total() past Decimal::MAX.
    let new_available = account
        .total()
        .checked_add(amount)
        .and_then(|_| account.available.checked_add(amount))
        .ok_or(ProcessingError::Overflow(client_id))?;

    account.available = new_available;

    transactions.insert(
        transaction_id,
        Transaction {
            id: transaction_id,
            amount,
            status: TransactionStatus::Completed,
            client_id,
            kind: TransactionKind::Credit,
        },
    );

    Ok(())
}
