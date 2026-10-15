use std::fmt;

use crate::types::{ClientId, TransactionId};

/// Reasons an operation is rejected. A rejected operation leaves the
/// engine state untouched.
#[derive(Debug, PartialEq)]
pub enum ProcessingError {
    /// The transaction id was already used by a previous deposit or withdrawal.
    DuplicateTransaction(TransactionId),
    /// The account is frozen after a chargeback.
    LockedAccount(ClientId),
    /// The withdrawal amount exceeds the available funds.
    InsufficientFunds(ClientId),
    /// The referenced transaction does not exist.
    TransactionNotFound(TransactionId),
    /// The referenced transaction belongs to another client.
    ClientMismatch {
        transaction_id: TransactionId,
        client_id: ClientId,
    },
    /// Only completed deposits can be disputed.
    NotDisputable(TransactionId),
    /// Resolve/chargeback reference a transaction that is not under dispute.
    NotDisputed(TransactionId),
    /// A stored transaction references an account that does not exist.
    ///
    /// The engine never produces this state (accounts are created before
    /// their first transaction is stored), so seeing this error means an
    /// internal invariant was broken. The engine reports it instead of
    /// panicking; it is up to the consumer to decide whether to abort or to
    /// log and keep going.
    AccountNotFound(ClientId),
    /// Applying the operation would overflow an account balance.
    Overflow(ClientId),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTransaction(tx) => write!(f, "transaction {tx} already exists"),
            Self::LockedAccount(client) => write!(f, "client {client} account is locked"),
            Self::InsufficientFunds(client) => write!(f, "client {client} has insufficient funds"),
            Self::TransactionNotFound(tx) => write!(f, "transaction {tx} not found"),
            Self::ClientMismatch {
                transaction_id,
                client_id,
            } => write!(
                f,
                "transaction {transaction_id} does not belong to client {client_id}"
            ),
            Self::NotDisputable(tx) => write!(f, "transaction {tx} cannot be disputed"),
            Self::NotDisputed(tx) => write!(f, "transaction {tx} is not under dispute"),
            Self::AccountNotFound(client) => {
                write!(f, "client {client} account not found (invariant violated)")
            }
            Self::Overflow(client) => write!(f, "client {client} balance would overflow"),
        }
    }
}

impl std::error::Error for ProcessingError {}
