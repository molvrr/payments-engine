use crate::types::{Amount, ClientId};

pub struct Account {
    pub id: ClientId,
    pub available: Amount,
    pub held: Amount,
    pub locked: bool,
}

impl Account {
    pub fn new(id: ClientId) -> Self {
        Self {
            id,
            available: 0.into(),
            held: 0.into(),
            locked: false,
        }
    }

    pub fn total(&self) -> Amount {
        self.available + self.held
    }
}
