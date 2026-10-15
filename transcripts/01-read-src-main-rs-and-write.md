# Read @src/main.rs and write a script for generating a csv with sample data follo

> Exported from Pi. User prompts and assistant replies are kept; tool calls, tool outputs, model reasoning and the system prompt are omitted.

## User

Read @src/main.rs and write a script for generating a csv with sample data following the type Transaction. It should take a flag indicating number of transactions

---

## Assistant

I wrote the generator at `scripts/gen_transactions.py`. It only uses the Python standard library. I ran it with 15 rows (seeded) and with 100,000 rows, and both produced well-formed CSV. I haven't tried loading that CSV in the Rust code yet.

```bash
python3 scripts/gen_transactions.py -n 1000 > transactions.csv
python3 scripts/gen_transactions.py -n 1000000 --clients 500 --seed 42 -o big.csv
```

**Flags:**
- `-n/--count` (required): number of transactions to generate.
- `-c/--clients` (default 10): number of different clients, each with a random `u16` id.
- `-s/--seed`: makes the output repeatable.
- `-o/--output`: output file; the default is stdout.
- `--invalid-ratio` (default 0.05): the share of dispute, resolve and chargeback rows that point to a transaction that doesn't exist, so your error handling gets tested.

**Output:**
- **Columns:** `type,client_id,tx,amount`, matching your `serde` names (`snake_case` tag, `tx` rename).
- **Deposits and withdrawals:** each gets a new increasing `tx` id and a positive amount with up to 4 decimal places.
- **Disputes, resolves and chargebacks:** the amount is empty. Disputes point to a real deposit by the same client. Resolves and chargebacks point to a transaction that is currently under dispute. A resolved deposit can be disputed again.
- **Withdrawals:** they don't check the client's balance, so some will overdraw, which tests that case too.

**Possible problem in `src/main.rs`:** the `csv` crate and `#[serde(tag = "type")]` (internally tagged enums) often don't work together. Every field is read as a string first, so number fields like `client_id: u16` can fail with "invalid type: string". If that happens, read each row into a flat struct (`type`, `client_id`, `tx`, `amount: Option<Decimal>`) and convert it to `Transaction` yourself.
