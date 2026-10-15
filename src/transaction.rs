use crate::types::{Amount, ClientId};

#[derive(PartialEq, Debug)]
pub enum TransactionStatus {
    Completed,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(PartialEq, Debug)]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
}

pub struct Transaction {
    pub amount: Amount,
    pub status: TransactionStatus,
    pub client_id: ClientId,
    pub kind: TransactionKind,
}
