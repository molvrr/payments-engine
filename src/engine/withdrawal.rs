use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessError;
use crate::operation::WithdrawalData;
use crate::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    withdrawal_data: WithdrawalData,
) -> Result<(), ProcessError> {
    let transaction = transactions.get(&withdrawal_data.transaction_id);

    if let Some(t) = transaction {
        eprintln!("Transaction {:?} already exists", t.id);
        return Ok(());
    }

    let account = accounts
        .entry(withdrawal_data.client_id)
        .or_insert(Account::new(withdrawal_data.client_id));

    if account.locked {
        eprintln!("Client account {:?} is locked", account.id);
        return Ok(());
    }

    if account.available < withdrawal_data.amount {
        eprintln!("Client account {:?} has no sufficient funds", account.id);
        return Ok(());
    }

    transactions.insert(
        withdrawal_data.transaction_id,
        Transaction {
            id: withdrawal_data.transaction_id,
            amount: withdrawal_data.amount,
            status: TransactionStatus::Completed,
            client_id: withdrawal_data.client_id,
            kind: TransactionKind::Debit,
        },
    );

    account.available -= withdrawal_data.amount;
    Ok(())
}
