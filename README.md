# Payments Engine
A simple payments engine.

## Usage
```sh
cargo run -- transactions.csv > accounts.csv   # run the engine
cargo test                                     # run the test suite
make                                           # generate sample data and run the engine on it
```
`make` uses [`scripts/gen_transactions.py`](scripts/gen_transactions.py) to generate `transactions.csv` (10,000 rows, 5,000 clients), then writes `accounts.csv`.

Rows that fail to parse or are rejected by the engine are reported on stderr. stdout only ever contains the accounts CSV.

## Design
```
CSV rows ──► csv_io::read_operations ──► Operation ──► Engine::process ──► csv_io::write_accounts ──► stdout
              (streaming, one row          (validated     (pure state
               at a time)                   at parse time)  machine)
```
- **Invalid rows are rejected at parse time.** `Operation` is built from the raw row through `TryFrom`, which fails with an `InvalidOperation` error. Deposits and withdrawals must have a strictly positive amount. Disputes, resolves and chargebacks refer to an existing transaction, so any amount on them is ignored. A malformed row is rejected before it reaches the engine, so the engine never has to handle one.
- **Separate id types.** `ClientId(u16)` and `TransactionId(u32)` are distinct types, so one can't be passed where the other is expected.
- **Exact decimal amounts.** Amounts use `rust_decimal::Decimal`, never floats, so binary approximations don't cause accumulating rounding errors.
- **One module per operation.** Each operation lives in its own file under `src/engine/` (`deposit.rs`, `withdrawal.rs`, `dispute.rs`, `resolve.rs`, `chargeback.rs`). Each one validates first and changes state only if every check passes.
- **Sans-IO.** The engine does no I/O; see [Efficiency](#efficiency).

## Assumptions
- **Only deposits can be disputed.** Disputing a withdrawal is rejected: the funds have already left the system, so there is nothing to hold.
- **Client and transaction must match.** A dispute, resolve or chargeback must name the client that owns the transaction. Otherwise it is rejected.
- **One dispute per transaction.** Each transaction moves `Completed → Disputed → Resolved | ChargedBack` and never goes back. A resolved or charged-back transaction can't be disputed again.
- **Locked accounts.** A locked account rejects new deposits and withdrawals. Disputes, resolves and chargebacks on its earlier deposits are still processed, since the bank still has to settle them.
- **Negative balances are allowed.** If a client deposits, withdraws, then disputes the deposit, `available` goes negative. If that dispute ends in a chargeback, `total` goes negative too.
- **Transaction ids are unique across clients.** A deposit or withdrawal that reuses an existing id is rejected. A rejected withdrawal (e.g. insufficient funds) is not recorded, so its id stays free.
- **Errors from the data source are ignored.** References to unknown transactions are ignored, as the spec says, and so are rows that break the rules above. These are reported on stderr and processing continues.

## Invariants
These hold after every operation:

1. **Held matches disputes:** `held` equals the sum of the client's deposits currently under dispute.
2. **Total matches the ledger:** `total` equals the client's deposits that weren't charged back, minus their withdrawals.
3. **Locked iff charged back:** an account is locked if and only if one of its transactions was charged back.
4. **Held is never negative.**
5. **Only deposits leave `Completed`:** withdrawals are never disputed.
6. **Every transaction has an account.**

They are written down in a [Quint](https://quint-lang.org) spec, [`engine.qnt`](engine.qnt), which models the same accept/reject rules as the Rust engine. To check them:
```sh
quint run    engine.qnt --main=engine --invariant=everyInvariant   # random simulation
quint verify engine.qnt --main=engine --invariant=everyInvariant   # bounded model checking (Apalache)
```
The same invariants are also checked in Rust (see [Correctness and testing](#correctness-and-testing)).

## Correctness and testing
`cargo test` runs:

- **Engine unit tests** ([`src/engine/tests.rs`](src/engine/tests.rs)). At least one test per rule in the spec and per assumption above, including overflow cases. Every rejected operation is also checked to change nothing, except that it may create an empty account.
- **Randomized invariant test.** Runs 500 seeded random sequences of 100 operations each over a small set of client and transaction ids, so disputes, resolves and chargebacks often hit real transactions. After every step it checks the invariants listed above, the same ones as `everyInvariant` in `engine.qnt`. It uses a small built-in random number generator, so it needs no extra dependencies and a failing seed can be rerun.
- **Parsing tests** (`src/operation.rs`): whitespace, a missing trailing amount column, four-decimal precision, maximum ids, invalid amounts and types, and malformed rows not stopping the rest of the file.
- **Output tests** (`src/csv_io.rs`): the format of the accounts CSV.
- **CLI tests** ([`tests/cli.rs`](tests/cli.rs)). These run the real binary on the input files in [`tests/fixtures/`](tests/fixtures) and compare against each case's `expected.csv`. The comparison ignores spacing, decimal formatting and row order, as the spec allows. The cases are: the spec example, dispute/resolve, chargeback, references to missing or wrong transactions, malformed rows, and precision.

## Error handling and safety
- **Three error types:**
  - `CsvIoError` (can't open the input, can't write the output) is fatal. The program prints it and exits with a non-zero code.
  - `InvalidOperation` (a well-formed row that isn't a valid operation, e.g. a deposit with no amount or a non-positive amount) is per row. Together with CSV errors (unknown type, out-of-range ids, unparsable amount), it makes a row malformed.
  - `ProcessingError` (a rejected operation) is per operation. It is reported on stderr and processing continues with the next row. A rejected operation leaves the engine's state unchanged.
- **Malformed rows** are reported and skipped. The rest of the file is still processed.
- **Error reports are buffered.** stderr is locked once and wrapped in a `BufWriter`, so a file with many bad rows doesn't pay one system call per message. A failure to write to stderr is ignored on purpose: losing a diagnostic must not stop processing or change the accounts written to stdout.
- **No panics on bad input.** Overflow is checked wherever it can happen: a deposit can overflow `available` or `total`, and a dispute can overflow `held` once `available` is negative. Those cases return `ProcessingError::Overflow`. Resolve and chargeback only move or remove funds already counted in `held`, so they can't overflow.
- **Broken invariants are reported, not panicked on.** `AccountNotFound` should never happen, because the engine creates the account first. If it ever did, the engine returns it as an error instead of panicking.
- **No `unsafe`, and no `unwrap`/`expect` outside tests.**

## Efficiency
- **Streaming:** rows are read and processed one at a time. The input file is never fully loaded into memory. Error messages are buffered too; on a 3M-row file with ~350k rejected rows this cut the run time from 5.6s to 1.7s.
- **Memory:** accounts are bounded by u16 (65,536 clients). Transactions are the real cost: every deposit and withdrawal is kept so it can be disputed later, which can reach about 4 billion entries with u32 ids. Possible improvements: keep only deposits (the only disputable kind), or move old transactions to disk.
- **Sans-IO:** the engine does no I/O. It only turns operations into state changes. Reading and writing are kept at the edges (`csv_io`, `main.rs`). `csv_io::operations_from_reader` accepts any `std::io::Read`, so the same engine works with a file, stdin, a test buffer, or a socket, and supporting a new source only means writing a new reader.
- **Concurrent TCP streams:** because the engine does no I/O, it doesn't care where operations come from or which runtime delivers them. A server would read each connection on its own task or thread (for example, a tokio task parsing rows with `csv-async`). The connections would send operations over channels to N engine workers, with clients assigned to workers by some sharding logic. Each worker owns its clients' accounts, so no locking is needed. Trade-off: transaction ids are only checked for duplicates within each worker. In production, transactions wouldn't live only in memory. They would be stored in durable storage, such as a database with a unique constraint on the transaction id. That makes the duplicate check apply across all workers, keeps the history through restarts and crashes, and limits memory to a cache of recent transactions.

## AI Usage
This project was made with AI assistance (Claude Opus 5.5 model on Pi) at the following steps:
- Code review
- Simple mechanical changes
- [Sample data generator script](scripts/gen_transactions.py)
- Test suite
- Documentation review

Transcripts can be found at `./transcripts`
