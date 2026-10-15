mod chargeback;
mod deposit;
mod dispute;
mod error;
mod resolve;
mod withdrawal;

use std::collections::HashMap;

use crate::account::Account;
use crate::operation::Operation;
use crate::transaction::Transaction;
use crate::types::{ClientId, TransactionId};
pub use error::ProcessingError;

#[derive(Default)]
pub struct State {
    pub accounts: HashMap<ClientId, Account>,
    pub transactions: HashMap<TransactionId, Transaction>,
}

#[derive(Default)]
pub struct Engine {
    pub state: State,
}

impl Engine {
    pub fn process(&mut self, transaction: Operation) -> Result<(), ProcessingError> {
        let client_id = transaction.client_id();
        self.state
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

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
