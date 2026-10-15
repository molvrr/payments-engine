use serde::Deserialize;

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct ClientId(pub(crate) u16);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct TransactionId(pub(crate) u32);

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Amount = rust_decimal::Decimal;
