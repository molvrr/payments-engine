#!/usr/bin/env python3
"""Generate a sample transactions CSV matching the `Transaction` enum in src/main.rs.

Columns: type, client_id, tx, amount
  - type:      deposit | withdrawal | dispute | resolve | chargeback
  - client_id: u16
  - tx:        u32 (unique for deposits/withdrawals)
  - amount:    decimal with up to 4 places (empty for dispute/resolve/chargeback)

Usage:
  python3 scripts/gen_transactions.py -n 1000 > transactions.csv
  python3 scripts/gen_transactions.py -n 1000000 --clients 500 --seed 42 -o big.csv
"""
import argparse
import csv
import random
import sys
from decimal import Decimal

U16_MAX = 2**16 - 1
U32_MAX = 2**32 - 1


def random_amount(rng: random.Random, max_units: int = 10_000) -> Decimal:
    # Up to 4 decimal places, strictly positive.
    return Decimal(rng.randint(1, max_units * 10_000)) / Decimal(10_000)


def generate(n: int, clients: int, rng: random.Random, invalid_ratio: float):
    client_ids = rng.sample(range(1, U16_MAX + 1), clients)
    next_tx = 1
    deposits: dict[int, int] = {}       # tx -> client_id (disputable)
    disputed: dict[int, int] = {}       # tx -> client_id (under dispute)

    weights = {
        "deposit": 0.55,
        "withdrawal": 0.30,
        "dispute": 0.08,
        "resolve": 0.04,
        "chargeback": 0.03,
    }
    kinds, probs = zip(*weights.items())

    for _ in range(n):
        kind = rng.choices(kinds, probs)[0]
        # Fall back to a deposit when a reference-based tx has nothing to refer to.
        if kind == "dispute" and not deposits:
            kind = "deposit"
        if kind in ("resolve", "chargeback") and not disputed:
            kind = "deposit"

        if kind in ("deposit", "withdrawal"):
            client = rng.choice(client_ids)
            tx = next_tx
            next_tx += 1
            if next_tx > U32_MAX:
                raise SystemExit("ran out of u32 transaction ids")
            if kind == "deposit":
                deposits[tx] = client
            yield (kind, client, tx, str(random_amount(rng)))
            continue

        # Occasionally emit an invalid reference (unknown tx / wrong client)
        # so the engine's error handling gets exercised.
        if rng.random() < invalid_ratio:
            yield (kind, rng.choice(client_ids), rng.randint(next_tx, U32_MAX), "")
            continue

        if kind == "dispute":
            tx = rng.choice(list(deposits))
            client = deposits.pop(tx)
            disputed[tx] = client
        else:  # resolve | chargeback
            tx = rng.choice(list(disputed))
            client = disputed.pop(tx)
            if kind == "resolve":
                deposits[tx] = client  # can be disputed again
        yield (kind, client, tx, "")


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("-n", "--count", type=int, required=True, help="number of transactions to generate")
    p.add_argument("-c", "--clients", type=int, default=10, help="number of distinct clients (default: 10)")
    p.add_argument("-s", "--seed", type=int, default=None, help="RNG seed for reproducible output")
    p.add_argument("-o", "--output", default="-", help="output file (default: stdout)")
    p.add_argument("--invalid-ratio", type=float, default=0.05,
                   help="fraction of dispute/resolve/chargeback rows referencing unknown txs (default: 0.05)")
    args = p.parse_args()

    if args.count < 0:
        p.error("--count must be >= 0")
    if not 1 <= args.clients <= U16_MAX:
        p.error(f"--clients must be between 1 and {U16_MAX}")

    rng = random.Random(args.seed)
    out = sys.stdout if args.output == "-" else open(args.output, "w", newline="")
    try:
        w = csv.writer(out, lineterminator="\n")
        w.writerow(["type", "client_id", "tx", "amount"])
        w.writerows(generate(args.count, args.clients, rng, args.invalid_ratio))
    finally:
        if out is not sys.stdout:
            out.close()


if __name__ == "__main__":
    main()
