use crate::types::Amount;
use crate::types::ClientId;
use crate::types::TransactionId;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Operation {
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
