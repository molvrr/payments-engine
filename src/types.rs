use serde::Deserialize;

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone)]
pub struct ClientId(u16);

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone)]
pub struct TransactionId(u32);

pub type Amount = rust_decimal::Decimal;
