//! Unit tests for the engine's state transitions.
//!
//! Operations are built directly (bypassing CSV parsing) so each test
//! exercises exactly one rule of the spec. The randomized test at the bottom
//! checks the ledger invariants from `engine.qnt` after every step.

use super::{Engine, ProcessingError};
use crate::operation::{
    ChargebackData, DepositData, DisputeData, Operation, ResolveData, WithdrawalData,
};
use crate::transaction::{TransactionKind, TransactionStatus};
use crate::types::{Amount, ClientId, TransactionId};

// ---------------------------------------------------------------- helpers

fn amount(value: &str) -> Amount {
    value.parse().expect("test amounts must be valid decimals")
}

fn deposit_amount(client: u16, tx: u32, amount: Amount) -> Operation {
    Operation::Deposit(DepositData {
        client_id: ClientId(client),
        transaction_id: TransactionId(tx),
        amount,
    })
}

fn deposit(client: u16, tx: u32, value: &str) -> Operation {
    deposit_amount(client, tx, amount(value))
}

fn withdrawal_amount(client: u16, tx: u32, amount: Amount) -> Operation {
    Operation::Withdrawal(WithdrawalData {
        client_id: ClientId(client),
        transaction_id: TransactionId(tx),
        amount,
    })
}

fn withdrawal(client: u16, tx: u32, value: &str) -> Operation {
    withdrawal_amount(client, tx, amount(value))
}

fn dispute(client: u16, tx: u32) -> Operation {
    Operation::Dispute(DisputeData {
        client_id: ClientId(client),
        transaction_id: TransactionId(tx),
    })
}

fn resolve(client: u16, tx: u32) -> Operation {
    Operation::Resolve(ResolveData {
        client_id: ClientId(client),
        transaction_id: TransactionId(tx),
    })
}

fn chargeback(client: u16, tx: u32) -> Operation {
    Operation::Chargeback(ChargebackData {
        client_id: ClientId(client),
        transaction_id: TransactionId(tx),
    })
}

/// Builds an engine by applying operations that are all expected to succeed.
#[track_caller]
fn engine_with(operations: impl IntoIterator<Item = Operation>) -> Engine {
    let mut engine = Engine::default();
    for operation in operations {
        engine
            .process(operation)
            .expect("setup operations must be accepted");
    }
    engine
}

/// Asserts the balances of a client: (available, held, total, locked).
#[track_caller]
fn assert_account(
    engine: &Engine,
    client: u16,
    available: &str,
    held: &str,
    total: &str,
    locked: bool,
) {
    let account = engine
        .state
        .accounts
        .get(&ClientId(client))
        .unwrap_or_else(|| panic!("client {client} should have an account"));
    assert_eq!(
        account.available,
        amount(available),
        "available of client {client}"
    );
    assert_eq!(account.held, amount(held), "held of client {client}");
    assert_eq!(account.total(), amount(total), "total of client {client}");
    assert_eq!(account.locked, locked, "locked of client {client}");
}

#[track_caller]
fn status(engine: &Engine, tx: u32) -> &TransactionStatus {
    &engine
        .state
        .transactions
        .get(&TransactionId(tx))
        .unwrap_or_else(|| panic!("transaction {tx} should be recorded"))
        .status
}

type AccountRow = (u16, Amount, Amount, bool);
type TransactionRow = (u32, u16, Amount, String);

/// Comparable copy of the whole engine state.
fn snapshot(engine: &Engine) -> (Vec<AccountRow>, Vec<TransactionRow>) {
    let mut accounts: Vec<AccountRow> = engine
        .state
        .accounts
        .values()
        .map(|a| (a.id.0, a.available, a.held, a.locked))
        .collect();
    accounts.sort_by_key(|row| row.0);

    let mut transactions: Vec<TransactionRow> = engine
        .state
        .transactions
        .iter()
        .map(|(id, t)| {
            (
                id.0,
                t.client_id.0,
                t.amount,
                format!("{:?}/{:?}", t.kind, t.status),
            )
        })
        .collect();
    transactions.sort_by_key(|row| row.0);

    (accounts, transactions)
}

/// Processes an operation and, if it is rejected, checks that the only
/// effect was (possibly) creating an empty account for the client.
#[track_caller]
fn process_checked(engine: &mut Engine, operation: Operation) -> Result<(), ProcessingError> {
    let client = operation.client_id();
    let had_account = engine.state.accounts.contains_key(&client);
    let before = snapshot(engine);

    let result = engine.process(operation);

    let (mut accounts_after, transactions_after) = snapshot(engine);
    let position = accounts_after
        .iter()
        .position(|row| row.0 == client.0)
        .expect("every processed operation leaves an account for its client");

    if result.is_err() {
        if !had_account {
            assert_eq!(
                accounts_after.remove(position),
                (client.0, Amount::ZERO, Amount::ZERO, false),
                "a rejected operation may only create an empty account"
            );
        }
        assert_eq!(
            before,
            (accounts_after, transactions_after),
            "a rejected operation must not change the state"
        );
    }

    result
}

#[track_caller]
fn assert_rejected(engine: &mut Engine, operation: Operation, expected: ProcessingError) {
    assert_eq!(process_checked(engine, operation), Err(expected));
}

// ---------------------------------------------------------------- deposit

#[test]
fn deposit_credits_available_and_total() {
    let engine = engine_with([deposit(1, 1, "1.5"), deposit(1, 2, "2.25")]);

    assert_account(&engine, 1, "3.75", "0", "3.75", false);
    assert_eq!(status(&engine, 1), &TransactionStatus::Completed);
}

#[test]
fn deposit_with_duplicate_transaction_id_is_rejected() {
    let mut engine = engine_with([deposit(1, 1, "1")]);

    assert_rejected(
        &mut engine,
        deposit(1, 1, "5"),
        ProcessingError::DuplicateTransaction(TransactionId(1)),
    );
    // Transaction ids are global, so another client cannot reuse it either.
    assert_rejected(
        &mut engine,
        deposit(2, 1, "5"),
        ProcessingError::DuplicateTransaction(TransactionId(1)),
    );
    assert_account(&engine, 1, "1", "0", "1", false);
}

#[test]
fn deposit_into_locked_account_is_rejected() {
    let mut engine = engine_with([deposit(1, 1, "1"), dispute(1, 1), chargeback(1, 1)]);

    assert_rejected(
        &mut engine,
        deposit(1, 2, "5"),
        ProcessingError::LockedAccount(ClientId(1)),
    );
    assert_account(&engine, 1, "0", "0", "0", true);
}

#[test]
fn deposit_that_overflows_available_is_rejected() {
    let mut engine = engine_with([deposit_amount(1, 1, Amount::MAX)]);

    assert_rejected(
        &mut engine,
        deposit(1, 2, "1"),
        ProcessingError::Overflow(ClientId(1)),
    );
}

#[test]
fn deposit_that_overflows_total_through_held_funds_is_rejected() {
    // available = 0 but held = MAX, so any further deposit overflows total().
    let mut engine = engine_with([deposit_amount(1, 1, Amount::MAX), dispute(1, 1)]);

    assert_rejected(
        &mut engine,
        deposit(1, 2, "1"),
        ProcessingError::Overflow(ClientId(1)),
    );
}

// ---------------------------------------------------------------- withdrawal

#[test]
fn withdrawal_debits_available_and_total() {
    let engine = engine_with([deposit(1, 1, "10"), withdrawal(1, 2, "3.3333")]);

    assert_account(&engine, 1, "6.6667", "0", "6.6667", false);
}

#[test]
fn withdrawal_of_entire_balance_is_allowed() {
    let engine = engine_with([deposit(1, 1, "2.5"), withdrawal(1, 2, "2.5")]);

    assert_account(&engine, 1, "0", "0", "0", false);
}

#[test]
fn withdrawal_with_insufficient_funds_is_rejected_and_not_recorded() {
    let mut engine = engine_with([deposit(1, 1, "1")]);

    assert_rejected(
        &mut engine,
        withdrawal(1, 2, "1.0001"),
        ProcessingError::InsufficientFunds(ClientId(1)),
    );
    assert_account(&engine, 1, "1", "0", "1", false);

    // The failed withdrawal was not recorded, so its id is still free.
    process_checked(&mut engine, deposit(1, 2, "1")).expect("tx 2 was never recorded");
}

#[test]
fn withdrawal_from_unknown_client_is_rejected_but_creates_the_account() {
    let mut engine = Engine::default();

    assert_rejected(
        &mut engine,
        withdrawal(2, 1, "1"),
        ProcessingError::InsufficientFunds(ClientId(2)),
    );
    assert_account(&engine, 2, "0", "0", "0", false);
}

#[test]
fn withdrawal_cannot_use_held_funds() {
    let mut engine = engine_with([deposit(1, 1, "10"), deposit(1, 2, "5"), dispute(1, 1)]);

    assert_rejected(
        &mut engine,
        withdrawal(1, 3, "6"),
        ProcessingError::InsufficientFunds(ClientId(1)),
    );
    process_checked(&mut engine, withdrawal(1, 4, "5")).expect("5 is available");
    assert_account(&engine, 1, "0", "10", "10", false);
}

#[test]
fn withdrawal_from_locked_account_is_rejected() {
    let mut engine = engine_with([
        deposit(1, 1, "10"),
        deposit(1, 2, "5"),
        dispute(1, 2),
        chargeback(1, 2),
    ]);

    assert_rejected(
        &mut engine,
        withdrawal(1, 3, "1"),
        ProcessingError::LockedAccount(ClientId(1)),
    );
    assert_account(&engine, 1, "10", "0", "10", true);
}

#[test]
fn withdrawal_with_duplicate_transaction_id_is_rejected() {
    let mut engine = engine_with([deposit(1, 1, "10")]);

    assert_rejected(
        &mut engine,
        withdrawal(1, 1, "1"),
        ProcessingError::DuplicateTransaction(TransactionId(1)),
    );
}

// ---------------------------------------------------------------- dispute

#[test]
fn dispute_moves_funds_from_available_to_held() {
    let engine = engine_with([deposit(1, 1, "10"), deposit(1, 2, "2.5"), dispute(1, 1)]);

    assert_account(&engine, 1, "2.5", "10", "12.5", false);
    assert_eq!(status(&engine, 1), &TransactionStatus::Disputed);
    assert_eq!(status(&engine, 2), &TransactionStatus::Completed);
}

#[test]
fn dispute_of_withdrawal_is_rejected() {
    let mut engine = engine_with([deposit(1, 1, "10"), withdrawal(1, 2, "4")]);

    assert_rejected(
        &mut engine,
        dispute(1, 2),
        ProcessingError::NotDisputable(TransactionId(2)),
    );
}

#[test]
fn dispute_after_funds_were_withdrawn_makes_available_negative() {
    // The fraud scenario: deposit, withdraw, then dispute the deposit.
    let engine = engine_with([deposit(1, 1, "10"), withdrawal(1, 2, "7"), dispute(1, 1)]);

    assert_account(&engine, 1, "-7", "10", "3", false);
}

#[test]
fn dispute_is_allowed_on_a_locked_account() {
    let engine = engine_with([
        deposit(1, 1, "10"),
        deposit(1, 2, "5"),
        dispute(1, 1),
        chargeback(1, 1),
        dispute(1, 2),
    ]);

    assert_account(&engine, 1, "0", "5", "5", true);
}

#[test]
fn dispute_that_overflows_held_is_rejected() {
    // held can exceed total once available goes negative.
    let mut engine = engine_with([
        deposit_amount(1, 1, Amount::MAX),
        withdrawal_amount(1, 2, Amount::MAX),
        deposit_amount(1, 3, Amount::MAX),
        dispute(1, 1),
    ]);

    assert_rejected(
        &mut engine,
        dispute(1, 3),
        ProcessingError::Overflow(ClientId(1)),
    );
}

// ---------------------------------------------------------------- resolve

#[test]
fn resolve_releases_held_funds() {
    let engine = engine_with([
        deposit(1, 1, "10"),
        deposit(1, 2, "1"),
        dispute(1, 1),
        resolve(1, 1),
    ]);

    assert_account(&engine, 1, "11", "0", "11", false);
    assert_eq!(status(&engine, 1), &TransactionStatus::Resolved);
}

// ---------------------------------------------------------------- chargeback

#[test]
fn chargeback_removes_held_funds_and_locks_the_account() {
    let engine = engine_with([
        deposit(1, 1, "10"),
        deposit(1, 2, "3"),
        dispute(1, 1),
        chargeback(1, 1),
    ]);

    assert_account(&engine, 1, "3", "0", "3", true);
    assert_eq!(status(&engine, 1), &TransactionStatus::ChargedBack);
}

#[test]
fn chargeback_after_withdrawal_leaves_a_negative_balance() {
    let engine = engine_with([
        deposit(1, 1, "10"),
        withdrawal(1, 2, "7"),
        dispute(1, 1),
        chargeback(1, 1),
    ]);

    assert_account(&engine, 1, "-7", "0", "-7", true);
}

#[test]
fn chargeback_only_reverses_the_disputed_transaction() {
    let engine = engine_with([
        deposit(1, 1, "10"),
        deposit(1, 2, "20"),
        dispute(1, 1),
        dispute(1, 2),
        chargeback(1, 2),
    ]);

    assert_account(&engine, 1, "0", "10", "10", true);
    assert_eq!(status(&engine, 1), &TransactionStatus::Disputed);
}

#[test]
fn chargeback_on_an_already_locked_account_is_applied() {
    let engine = engine_with([
        deposit(1, 1, "10"),
        deposit(1, 2, "20"),
        dispute(1, 1),
        dispute(1, 2),
        chargeback(1, 1),
        chargeback(1, 2),
    ]);

    assert_account(&engine, 1, "0", "0", "0", true);
    assert_eq!(status(&engine, 2), &TransactionStatus::ChargedBack);
}

// ---------------------------------------------------------------- references
//
// Dispute, resolve and chargeback each live in their own module and run their
// own checks, so every rule below is exercised for all three.

#[test]
fn references_to_unknown_transactions_are_ignored() {
    for operation in [dispute(1, 99), resolve(1, 99), chargeback(1, 99)] {
        let mut engine = engine_with([deposit(1, 1, "10")]);
        assert_rejected(
            &mut engine,
            operation,
            ProcessingError::TransactionNotFound(TransactionId(99)),
        );
    }
}

#[test]
fn references_to_another_clients_transaction_are_rejected() {
    // tx 1 is disputed, so resolve and chargeback would otherwise be accepted;
    // dispute gets a fresh, undisputed deposit for the same reason.
    let cases = [
        (vec![deposit(1, 1, "10")], dispute(2, 1)),
        (vec![deposit(1, 1, "10"), dispute(1, 1)], resolve(2, 1)),
        (vec![deposit(1, 1, "10"), dispute(1, 1)], chargeback(2, 1)),
    ];
    for (setup, operation) in cases {
        let mut engine = engine_with(setup);
        assert_rejected(
            &mut engine,
            operation,
            ProcessingError::ClientMismatch {
                transaction_id: TransactionId(1),
                client_id: ClientId(2),
            },
        );
    }
}

/// Transactions move Completed → Disputed → Resolved | ChargedBack and never
/// go back. Every other transition is rejected.
#[test]
fn invalid_status_transitions_are_rejected() {
    use ProcessingError::{NotDisputable, NotDisputed};

    let completed = || vec![deposit(1, 1, "10")];
    let disputed = || vec![deposit(1, 1, "10"), dispute(1, 1)];
    let resolved = || vec![deposit(1, 1, "10"), dispute(1, 1), resolve(1, 1)];
    let charged_back = || vec![deposit(1, 1, "10"), dispute(1, 1), chargeback(1, 1)];
    let tx = TransactionId(1);

    let cases = [
        (
            "dispute twice",
            disputed(),
            dispute(1, 1),
            NotDisputable(tx),
        ),
        (
            "dispute after resolve",
            resolved(),
            dispute(1, 1),
            NotDisputable(tx),
        ),
        (
            "dispute after chargeback",
            charged_back(),
            dispute(1, 1),
            NotDisputable(tx),
        ),
        (
            "resolve undisputed",
            completed(),
            resolve(1, 1),
            NotDisputed(tx),
        ),
        ("resolve twice", resolved(), resolve(1, 1), NotDisputed(tx)),
        (
            "resolve after chargeback",
            charged_back(),
            resolve(1, 1),
            NotDisputed(tx),
        ),
        (
            "chargeback undisputed",
            completed(),
            chargeback(1, 1),
            NotDisputed(tx),
        ),
        (
            "chargeback after resolve",
            resolved(),
            chargeback(1, 1),
            NotDisputed(tx),
        ),
        (
            "chargeback twice",
            charged_back(),
            chargeback(1, 1),
            NotDisputed(tx),
        ),
    ];
    for (name, setup, operation, expected) in cases {
        let mut engine = engine_with(setup);
        assert_eq!(
            process_checked(&mut engine, operation),
            Err(expected),
            "{name}"
        );
    }
}

// ---------------------------------------------------------------- general

#[test]
fn clients_are_independent() {
    let engine = engine_with([
        deposit(2, 10, "5"),
        deposit(1, 3, "1"),
        dispute(2, 10),
        chargeback(2, 10),
        withdrawal(1, 4, "0.5"),
    ]);

    assert_account(&engine, 1, "0.5", "0", "0.5", false);
    assert_account(&engine, 2, "0", "0", "0", true);
}

// ---------------------------------------------------------------- invariants

/// Same invariants as `everyInvariant` in engine.qnt.
#[track_caller]
fn assert_invariants(engine: &Engine) {
    let transactions = &engine.state.transactions;

    for (id, transaction) in transactions {
        assert!(
            engine.state.accounts.contains_key(&transaction.client_id),
            "transactionsHaveAccounts: tx {id} has no account"
        );
        assert!(
            transaction.status == TransactionStatus::Completed
                || transaction.kind == TransactionKind::Deposit,
            "onlyDepositsDisputed: tx {} is a {:?} in status {:?}",
            id,
            transaction.kind,
            transaction.status
        );
    }

    for account in engine.state.accounts.values() {
        let own = || transactions.values().filter(|t| t.client_id == account.id);
        let sum = |filter: &dyn Fn(&&crate::transaction::Transaction) -> bool| -> Amount {
            own().filter(filter).map(|t| t.amount).sum()
        };

        let disputed = sum(&|t| t.status == TransactionStatus::Disputed);
        let kept_deposits = sum(&|t| {
            t.kind == TransactionKind::Deposit && t.status != TransactionStatus::ChargedBack
        });
        let withdrawals = sum(&|t| t.kind == TransactionKind::Withdrawal);
        let charged_back = own().any(|t| t.status == TransactionStatus::ChargedBack);

        assert_eq!(
            account.held, disputed,
            "heldMatchesDisputes for client {}",
            account.id
        );
        assert_eq!(
            account.total(),
            kept_deposits - withdrawals,
            "totalMatchesLedger for client {}",
            account.id
        );
        assert_eq!(
            account.locked, charged_back,
            "lockedIffChargedBack for client {}",
            account.id
        );
        assert!(
            account.held >= Amount::ZERO,
            "noNegativeHeld for client {}",
            account.id
        );
    }
}

/// Small deterministic PRNG (xorshift64*) so the test needs no extra crates
/// and failures are reproducible from the seed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % bound
    }

    fn operation(&mut self) -> Operation {
        // Small id spaces force collisions, so disputes, resolves and
        // chargebacks frequently hit real transactions.
        let client = self.below(3) as u16 + 1;
        let tx = self.below(12) as u32 + 1;
        let amount = Amount::new(self.below(1_000_000) as i64 + 1, 4);
        match self.below(5) {
            0 => deposit_amount(client, tx, amount),
            1 => withdrawal_amount(client, tx, amount),
            2 => dispute(client, tx),
            3 => resolve(client, tx),
            _ => chargeback(client, tx),
        }
    }
}

#[test]
fn random_operation_sequences_preserve_ledger_invariants() {
    let mut accepted = 0;
    for seed in 0..500 {
        let mut rng = Rng::new(seed);
        let mut engine = Engine::default();
        for _ in 0..100 {
            if process_checked(&mut engine, rng.operation()).is_ok() {
                accepted += 1;
            }
            assert_invariants(&engine);
        }
    }
    // Guard against the generator degenerating into all-rejected sequences.
    assert!(accepted > 5_000, "only {accepted} operations were accepted");
}
