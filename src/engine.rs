mod chargeback;
mod deposit;
mod dispute;
mod resolve;
mod withdrawal;

use std::collections::HashMap;

use crate::account::Account;
use crate::operation::Operation;
use crate::transaction::Transaction;
use crate::types::{ClientId, TransactionId};

pub struct State {
    pub accounts: HashMap<ClientId, Account>,
    pub transactions: HashMap<TransactionId, Transaction>,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }
}

pub struct Engine {
    pub state: State,
}

#[derive(Debug)]
pub enum ProcessError {}

impl Engine {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub fn process(&mut self, transaction: Operation) -> Result<(), ProcessError> {
        match transaction {
            Operation::Deposit(deposit_data) => deposit::run(
                &mut self.state.transactions,
                &mut self.state.accounts,
                deposit_data,
            ),
            Operation::Withdrawal(withdrawal_data) => withdrawal::run(
                &mut self.state.transactions,
                &mut self.state.accounts,
                withdrawal_data,
            ),
            Operation::Dispute(dispute_data) => dispute::run(
                &mut self.state.transactions,
                &mut self.state.accounts,
                dispute_data,
            ),
            Operation::Resolve(resolve_data) => resolve::run(
                &mut self.state.transactions,
                &mut self.state.accounts,
                resolve_data,
            ),
            Operation::Chargeback(chargeback_data) => chargeback::run(
                &mut self.state.transactions,
                &mut self.state.accounts,
                chargeback_data,
            ),
        }
    }
}
