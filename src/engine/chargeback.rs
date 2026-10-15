use std::collections::HashMap;

use crate::account::Account;
use crate::engine::ProcessingError;
use crate::operation::ChargebackData;
use crate::transaction::{Transaction, TransactionStatus};
use crate::types::{ClientId, TransactionId};

pub fn run(
    transactions: &mut HashMap<TransactionId, Transaction>,
    accounts: &mut HashMap<ClientId, Account>,
    chargeback_data: ChargebackData,
) -> Result<(), ProcessingError> {
    let transaction = transactions
        .get_mut(&chargeback_data.transaction_id)
        .ok_or(ProcessingError::TransactionNotFound(
            chargeback_data.transaction_id,
        ))?;

    if transaction.client_id != chargeback_data.client_id {
        return Err(ProcessingError::ClientMismatch {
            transaction_id: chargeback_data.transaction_id,
            client_id: chargeback_data.client_id,
        });
    }

    if transaction.status != TransactionStatus::Disputed {
        return Err(ProcessingError::NotDisputed(chargeback_data.transaction_id));
    }

    let account = accounts
        .get_mut(&chargeback_data.client_id)
        .ok_or(ProcessingError::AccountNotFound(chargeback_data.client_id))?;

    // Cannot underflow: held always includes this disputed amount.
    account.held -= transaction.amount;
    account.locked = true;
    transaction.status = TransactionStatus::ChargedBack;

    Ok(())
}
