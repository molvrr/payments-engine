#![allow(unused)]
use serde::Deserialize;
use std::{collections::HashMap, io::BufRead};

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone)]
struct ClientId(u16);

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone)]
struct TransactionId(u32);

type Amount = rust_decimal::Decimal;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Operation {
    Deposit {
        client_id: ClientId,
        #[serde(rename = "tx")]
        transaction_id: TransactionId,
        amount: Amount,
    },
    Withdrawal {
        client_id: ClientId,
        #[serde(rename = "tx")]
        transaction_id: TransactionId,
        amount: Amount,
    },
    Dispute {
        client_id: ClientId,
        #[serde(rename = "tx")]
        transaction_id: TransactionId,
    },
    Resolve {
        client_id: ClientId,
        #[serde(rename = "tx")]
        transaction_id: TransactionId,
    },
    Chargeback {
        client_id: ClientId,
        #[serde(rename = "tx")]
        transaction_id: TransactionId,
    },
}

#[derive(PartialEq)]
enum TransactionStatus {
    Completed,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(PartialEq)]
enum TransactionKind {
    Debit,
    Credit,
}

struct Transaction {
    id: TransactionId,
    amount: Amount,
    status: TransactionStatus,
    client_id: ClientId,
    kind: TransactionKind,
}

struct Account {
    id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Account {
    fn new(id: ClientId) -> Self {
        Self {
            id,
            available: 0.into(),
            held: 0.into(),
            locked: false,
        }
    }
}

impl Account {
    fn total(&self) -> Amount {
        self.available + self.held
    }
}

struct State {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, Transaction>,
}

impl State {
    fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }
}

struct Engine {
    state: State,
}

enum ProcessError {}

impl Engine {
    fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    fn process(&mut self, transaction: Operation) -> Result<(), ProcessError> {
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

                let mut account = self
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

                let mut account = self
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

                if let Some(mut transaction) = transaction_opt {
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

                    if let Some(mut account) = self.state.accounts.get_mut(&client_id) {
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

                if let Some(mut transaction) = transaction_opt {
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

// TODO: Don't box every single error.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(csv_file) = std::env::args().nth(1) {
        let file = std::fs::File::open(csv_file)?;
        let mut reader = csv::Reader::from_reader(file);
        let mut engine = Engine::new();

        for result in reader.deserialize::<Operation>() {
            if let Ok(transaction) = result {
            } else {
                // TODO: Log error
            }
        }
    } else {
        eprintln!("Usage should be ./binary <transactions_file.csv>");
    }

    Ok(())
}
