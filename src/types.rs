use serde::Deserialize;

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct ClientId(u16);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(Deserialize, Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct TransactionId(u32);

pub type Amount = rust_decimal::Decimal;
