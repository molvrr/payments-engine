use crate::types::{Amount, ClientId, TransactionId};

#[derive(PartialEq)]
pub enum TransactionStatus {
    Completed,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(PartialEq)]
pub enum TransactionKind {
    Debit,
    Credit,
}

pub struct Transaction {
    pub id: TransactionId,
    pub amount: Amount,
    pub status: TransactionStatus,
    pub client_id: ClientId,
    pub kind: TransactionKind,
}
