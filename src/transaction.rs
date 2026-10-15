use crate::types::{Amount, ClientId, TransactionId};

#[derive(PartialEq, Debug)]
pub enum TransactionStatus {
    Completed,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(PartialEq, Debug)]
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
