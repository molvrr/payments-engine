use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessError;
use crate::operation::DepositData;
use crate::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    deposit_data: DepositData,
) -> Result<(), ProcessError> {
    let transaction = transactions.get(&deposit_data.transaction_id);

    if let Some(t) = transaction {
        eprintln!("Transaction {:?} already exists", t.id);
        return Ok(());
    }

    let account = accounts
        .entry(deposit_data.client_id)
        .or_insert(Account::new(deposit_data.client_id));

    if account.locked {
        eprintln!("Client account {:?} is locked", account.id);
        return Ok(());
    }

    transactions.insert(
        deposit_data.transaction_id,
        Transaction {
            id: deposit_data.transaction_id,
            amount: deposit_data.amount,
            status: TransactionStatus::Completed,
            client_id: deposit_data.client_id,
            kind: TransactionKind::Credit,
        },
    );

    account.available += deposit_data.amount;

    Ok(())
}
