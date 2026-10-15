use crate::account::Account;
use crate::operation::Operation;
use crate::state::State;
use crate::transaction::{Transaction, TransactionKind, TransactionStatus};

pub struct Engine {
    state: State,
}

pub enum ProcessError {}

impl Engine {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub fn process(&mut self, transaction: Operation) -> Result<(), ProcessError> {
        match transaction {
            Operation::Deposit {
                client_id,
                transaction_id,
                amount,
            } => {
                let transaction = self.state.transactions.get(&transaction_id);

                if let Some(_) = transaction {
                    // TODO: Log transaction already exists
                    return Ok(());
                }

                let account = self
                    .state
                    .accounts
                    .entry(client_id)
                    .or_insert(Account::new(client_id));

                if account.locked {
                    // TODO: Log when account is locked (?)
                    return Ok(());
                }

                self.state.transactions.insert(
                    transaction_id,
                    Transaction {
                        id: transaction_id,
                        amount,
                        status: TransactionStatus::Completed,
                        client_id,
                        kind: TransactionKind::Debit,
                    },
                );

                account.available += amount;
            }
            Operation::Withdrawal {
                client_id,
                transaction_id,
                amount,
            } => {
                let transaction = self.state.transactions.get(&transaction_id);

                if let Some(_) = transaction {
                    // TODO: Log transaction already exists
                    return Ok(());
                }

                let account = self
                    .state
                    .accounts
                    .entry(client_id)
                    .or_insert(Account::new(client_id));

                if account.locked {
                    // TODO: Log when account is locked (?)
                    return Ok(());
                }

                self.state.transactions.insert(
                    transaction_id,
                    Transaction {
                        id: transaction_id,
                        amount,
                        status: TransactionStatus::Completed,
                        client_id,
                        kind: TransactionKind::Debit,
                    },
                );

                account.available -= amount;
            }
            Operation::Dispute {
                client_id,
                transaction_id,
            } => {
                let transaction_opt = self.state.transactions.get_mut(&transaction_id);

                if let Some(transaction) = transaction_opt {
                    if transaction.client_id != client_id {
                        // TODO: Log different client
                        return Ok(());
                    }

                    if transaction.status != TransactionStatus::Completed
                        || transaction.kind == TransactionKind::Debit
                    {
                        // TODO: Log transaction status
                        return Ok(());
                    }

                    if let Some(account) = self.state.accounts.get_mut(&client_id) {
                        account.available -= transaction.amount;
                        account.held += transaction.amount;
                        transaction.status = TransactionStatus::Disputed;
                    }
                }
            }
            Operation::Resolve {
                client_id,
                transaction_id,
            } => {
                let transaction_opt = self.state.transactions.get_mut(&transaction_id);

                if let Some(transaction) = transaction_opt {
                    if transaction.client_id != client_id {
                        // TODO: Log different client
                        return Ok(());
                    }

                    if transaction.status != TransactionStatus::Disputed {
                        // TODO: Log different status
                        return Ok(());
                    }
                }
            }
            Operation::Chargeback {
                client_id,
                transaction_id,
            } => todo!(),
        }
        Ok(())
    }
}
