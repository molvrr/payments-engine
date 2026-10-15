use crate::types::Amount;
use crate::types::ClientId;
use crate::types::TransactionId;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "RawRecord")]
pub enum Operation {
    Deposit(DepositData),
    Withdrawal(WithdrawalData),
    Dispute(DisputeData),
    Resolve(ResolveData),
    Chargeback(ChargebackData),
}

#[derive(Deserialize)]
struct RawRecord {
    #[serde(rename = "type")]
    kind: RawKind,
    #[serde(alias = "client")]
    client_id: ClientId,
    tx: TransactionId,
    amount: Option<Amount>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum RawKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<RawRecord> for Operation {
    type Error = String;

    fn try_from(r: RawRecord) -> Result<Self, Self::Error> {
        let RawRecord {
            kind,
            client_id,
            tx: transaction_id,
            amount,
        } = r;

        Ok(match (kind, amount) {
            (RawKind::Deposit, Some(amount)) => {
                if amount <= 0.into() {
                    return Err(format!("amount can't be less than/equal to zero"));
                }

                Operation::Deposit(DepositData {
                    client_id,
                    transaction_id,
                    amount,
                })
            }
            (RawKind::Withdrawal, Some(amount)) => {
                if amount <= 0.into() {
                    return Err(format!("amount can't be less than/equal to zero"));
                }

                Operation::Withdrawal(WithdrawalData {
                    client_id,
                    transaction_id,
                    amount,
                })
            }
            (RawKind::Dispute, None) => Operation::Dispute(DisputeData {
                client_id,
                transaction_id,
            }),
            (RawKind::Resolve, None) => Operation::Resolve(ResolveData {
                client_id,
                transaction_id,
            }),
            (RawKind::Chargeback, None) => Operation::Chargeback(ChargebackData {
                client_id,
                transaction_id,
            }),
            (kind @ (RawKind::Deposit | RawKind::Withdrawal), None) => {
                return Err(format!("{kind:?} is missing an amount"));
            }
            (kind, Some(_)) => return Err(format!("{kind:?} must not have an amount")),
        })
    }
}

impl Operation {
    pub fn client_id(&self) -> ClientId {
        match self {
            Operation::Deposit(d) => d.client_id,
            Operation::Withdrawal(w) => w.client_id,
            Operation::Dispute(d) => d.client_id,
            Operation::Resolve(r) => r.client_id,
            Operation::Chargeback(c) => c.client_id,
        }
    }
}

pub struct DepositData {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
    pub amount: Amount,
}

pub struct WithdrawalData {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
    pub amount: Amount,
}

pub struct DisputeData {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}

pub struct ResolveData {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}

pub struct ChargebackData {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}
