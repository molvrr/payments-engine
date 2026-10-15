use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessingError;
use crate::operation::WithdrawalData;
use crate::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    withdrawal_data: WithdrawalData,
) -> Result<(), ProcessingError> {
    let WithdrawalData {
        client_id,
        transaction_id,
        amount,
    } = withdrawal_data;

    if transactions.contains_key(&transaction_id) {
        return Err(ProcessingError::DuplicateTransaction(transaction_id));
    }

    let account = accounts
        .get_mut(&client_id)
        .ok_or(ProcessingError::AccountNotFound(client_id))?;

    if account.locked {
        return Err(ProcessingError::LockedAccount(client_id));
    }

    if account.available < amount {
        return Err(ProcessingError::InsufficientFunds(client_id));
    }

    account.available -= amount;

    transactions.insert(
        transaction_id,
        Transaction {
            id: transaction_id,
            amount,
            status: TransactionStatus::Completed,
            client_id,
            kind: TransactionKind::Debit,
        },
    );

    Ok(())
}
