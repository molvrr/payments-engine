use crate::types::Amount;
use crate::types::ClientId;
use crate::types::TransactionId;
use serde::Deserialize;
use std::fmt;

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

/// Why a well-formed CSV row is still not a valid operation.
#[derive(Debug, PartialEq)]
pub enum InvalidOperation {
    /// Deposits and withdrawals must move a strictly positive amount.
    NonPositiveAmount,
    /// Deposits and withdrawals must carry an amount.
    MissingAmount(RawKind),
}

impl fmt::Display for InvalidOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveAmount => f.write_str("amount can't be less than/equal to zero"),
            Self::MissingAmount(kind) => write!(f, "{kind:?} is missing an amount"),
        }
    }
}

impl std::error::Error for InvalidOperation {}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RawKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<RawRecord> for Operation {
    type Error = InvalidOperation;

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
                    return Err(InvalidOperation::NonPositiveAmount);
                }

                Operation::Deposit(DepositData {
                    client_id,
                    transaction_id,
                    amount,
                })
            }
            (RawKind::Withdrawal, Some(amount)) => {
                if amount <= 0.into() {
                    return Err(InvalidOperation::NonPositiveAmount);
                }

                Operation::Withdrawal(WithdrawalData {
                    client_id,
                    transaction_id,
                    amount,
                })
            }
            // Disputes, resolves and chargebacks refer to an existing
            // transaction, so any amount on them is ignored.
            (RawKind::Dispute, _) => Operation::Dispute(DisputeData {
                client_id,
                transaction_id,
            }),
            (RawKind::Resolve, _) => Operation::Resolve(ResolveData {
                client_id,
                transaction_id,
            }),
            (RawKind::Chargeback, _) => Operation::Chargeback(ChargebackData {
                client_id,
                transaction_id,
            }),
            (kind @ (RawKind::Deposit | RawKind::Withdrawal), None) => {
                return Err(InvalidOperation::MissingAmount(kind));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_io::operations_from_reader;

    fn parse_one(row: &str) -> Result<Operation, csv::Error> {
        let input = format!("type,client,tx,amount\n{row}\n");
        let mut operations: Vec<_> = operations_from_reader(input.as_bytes()).collect();
        assert_eq!(operations.len(), 1, "expected exactly one record");
        operations.remove(0)
    }

    #[track_caller]
    fn parse_ok(row: &str) -> Operation {
        parse_one(row).unwrap_or_else(|err| panic!("{row:?} should parse: {err}"))
    }

    #[track_caller]
    fn parse_err(row: &str) -> String {
        match parse_one(row) {
            Ok(_) => panic!("{row:?} should be rejected"),
            Err(err) => err.to_string(),
        }
    }

    fn amount(value: &str) -> Amount {
        value.parse().unwrap()
    }

    #[test]
    fn parses_deposit() {
        let Operation::Deposit(d) = parse_ok("deposit,1,2,1.5") else {
            panic!("expected a deposit");
        };
        assert_eq!(
            (d.client_id, d.transaction_id, d.amount),
            (ClientId(1), TransactionId(2), amount("1.5"))
        );
    }

    #[test]
    fn parses_withdrawal() {
        let Operation::Withdrawal(w) = parse_ok("withdrawal,3,4,0.25") else {
            panic!("expected a withdrawal");
        };
        assert_eq!(
            (w.client_id, w.transaction_id, w.amount),
            (ClientId(3), TransactionId(4), amount("0.25"))
        );
    }

    /// Disputes, resolves and chargebacks parse the same whether the amount
    /// column is missing, empty or filled in (the amount is ignored).
    #[test]
    fn dispute_resolve_and_chargeback_ignore_the_amount_column() {
        let reference = (ClientId(1), TransactionId(2));
        for amount_column in ["", ",", ",1.0"] {
            let row = |kind: &str| format!("{kind},1,2{amount_column}");

            let Operation::Dispute(d) = parse_ok(&row("dispute")) else {
                panic!("expected a dispute for {amount_column:?}");
            };
            let Operation::Resolve(r) = parse_ok(&row("resolve")) else {
                panic!("expected a resolve for {amount_column:?}");
            };
            let Operation::Chargeback(c) = parse_ok(&row("chargeback")) else {
                panic!("expected a chargeback for {amount_column:?}");
            };
            assert_eq!((d.client_id, d.transaction_id), reference);
            assert_eq!((r.client_id, r.transaction_id), reference);
            assert_eq!((c.client_id, c.transaction_id), reference);
        }
    }

    #[test]
    fn accepts_whitespace_around_headers_and_values() {
        let input = "type, client, tx, amount\n  deposit ,  1 , 1 ,  1.0  \n dispute , 1, 1, \n";
        let operations: Vec<_> = operations_from_reader(input.as_bytes()).collect();
        assert!(matches!(operations[0], Ok(Operation::Deposit(_))));
        assert!(matches!(operations[1], Ok(Operation::Dispute(_))));
    }

    #[test]
    fn keeps_four_decimal_places() {
        let Operation::Deposit(d) = parse_ok("deposit,1,1,1.2345") else {
            panic!("expected a deposit");
        };
        assert_eq!(d.amount, Amount::new(12345, 4));

        let Operation::Deposit(d) = parse_ok("deposit,1,1,0.0001") else {
            panic!("expected a deposit");
        };
        assert_eq!(d.amount, Amount::new(1, 4));
    }

    #[test]
    fn accepts_maximum_client_and_transaction_ids() {
        let Operation::Deposit(d) = parse_ok("deposit,65535,4294967295,1") else {
            panic!("expected a deposit");
        };
        assert_eq!(
            (d.client_id, d.transaction_id),
            (ClientId(u16::MAX), TransactionId(u32::MAX))
        );
    }

    #[test]
    fn rejects_deposit_and_withdrawal_without_amount() {
        assert!(parse_err("deposit,1,1,").contains("missing an amount"));
        assert!(parse_err("withdrawal,1,1").contains("missing an amount"));
    }

    #[test]
    fn rejects_non_positive_amounts() {
        assert!(parse_err("deposit,1,1,0").contains("less than/equal to zero"));
        assert!(parse_err("deposit,1,1,-1.0").contains("less than/equal to zero"));
        assert!(parse_err("withdrawal,1,1,0.0000").contains("less than/equal to zero"));
        assert!(parse_err("withdrawal,1,1,-0.5").contains("less than/equal to zero"));
    }

    #[test]
    fn rejects_invalid_fields() {
        parse_err("transfer,1,1,1.0"); // unknown type
        parse_err("Deposit,1,1,1.0"); // types are lowercase
        parse_err("deposit,65536,1,1.0"); // client out of u16 range
        parse_err("deposit,-1,1,1.0"); // negative client
        parse_err("deposit,1,4294967296,1.0"); // tx out of u32 range
        parse_err("deposit,1,1,abc"); // not a number
        parse_err("deposit,,1,1.0"); // missing client
    }

    #[test]
    fn malformed_rows_do_not_stop_the_stream() {
        let input = "type,client,tx,amount\ndeposit,1,1,1.0\nbogus,row\ndeposit,1,2,2.0\n";
        let operations: Vec<_> = operations_from_reader(input.as_bytes()).collect();
        assert_eq!(operations.len(), 3);
        assert!(operations[0].is_ok());
        assert!(operations[1].is_err());
        assert!(operations[2].is_ok());
    }
}
