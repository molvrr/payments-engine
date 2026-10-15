use crate::account::Account;
use crate::transaction::Transaction;
use crate::types::ClientId;
use crate::types::TransactionId;

use std::collections::HashMap;

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
