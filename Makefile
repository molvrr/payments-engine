accounts.csv: transactions.csv
	@cargo run -- transactions.csv > accounts.csv

transactions.csv:
	@python3 scripts/gen_transactions.py -n 10000 -c 5000 > transactions.csv
