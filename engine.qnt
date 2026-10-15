module engine {
    type Account = {id: int, available: int, held: int, locked: bool}
    type Operation =
        Deposit({client_id: int, transaction_id: int, amount: int})
        | Withdrawal({client_id: int, transaction_id: int, amount: int})
        | Dispute({client_id: int, transaction_id: int})
        | Resolve({client_id: int, transaction_id: int})
        | Chargeback({client_id: int, transaction_id: int})
    type TransactionStatus = Completed | Disputed | Resolved | ChargedBack
    type Transaction = {id: int, amount: int, status: TransactionStatus, client_id: int}

    var accounts: int -> Account
    var transactions: int -> Transaction
    var records: List[Operation]
    pure val ADDRESSES = 1.to(100)

    def total(account: Account): int = account.available + account.held

    action unchanged = all {
        accounts' = accounts,
        transactions' = transactions,
        records' = records
    }

    action init = all {
        // At the initial state, all accounts are zero
        accounts' = ADDRESSES.mapBy(id => {id: id, available: 0, held: 0, locked: false}),
        transactions' = Map(),
        records' = []
    }

    action deposit(client_id, transaction_id, amount) = process(Deposit({client_id: client_id, transaction_id: transaction_id, amount: amount}))
    action withdraw(client_id, transaction_id, amount) = process(Withdrawal({client_id: client_id, transaction_id: transaction_id, amount: amount}))
    action dispute(client_id, transaction_id) = process(Dispute({client_id: client_id, transaction_id: transaction_id}))
    action resolve(client_id, transaction_id) = process(Resolve({client_id: client_id, transaction_id: transaction_id}))
    action chargeback(client_id, transaction_id) = process(Chargeback({client_id: client_id, transaction_id: transaction_id}))

    action process(operation: Operation): bool = {
        match (operation) {
            | Deposit(d) => all {
                not(accounts.get(d.client_id).locked),
                not(transactions.keys().contains(d.transaction_id)),
                accounts' = accounts.setBy(d.client_id, (old => ({...old, available: old.available + d.amount}))),
                transactions' = transactions.put(d.transaction_id, {id: d.transaction_id, amount: d.amount, status: Completed, client_id: d.client_id}),
                records' = records.append(operation)
            }
            | Withdrawal(w) => all {
                not(accounts.get(w.client_id).locked),
                not(transactions.keys().contains(w.transaction_id)),
                accounts.get(w.client_id).available >= w.amount,
                accounts' = accounts.setBy(w.client_id, (old => ({...old, available: old.available - w.amount}))),
                transactions' = transactions,
                records' = records.append(operation)
            }
            | Dispute(d) => all {
                transactions.keys().contains(d.transaction_id),
                transactions.get(d.transaction_id).status == Completed,
                d.client_id == transactions.get(d.transaction_id).client_id,
                transactions' = transactions.setBy(d.transaction_id, (old => {...old, status: Disputed})),
                accounts' = accounts.setBy(d.client_id, (old => {...old, available: old.available - transactions.get(d.transaction_id).amount, held: old.held + transactions.get(d.transaction_id).amount})),
                records' = records.append(operation)
            }
            | Resolve(r) => all {
                transactions.keys().contains(r.transaction_id),
                transactions.get(r.transaction_id).status == Disputed,
                r.client_id == transactions.get(r.transaction_id).client_id,
                transactions' = transactions.setBy(r.transaction_id, (old => {...old, status: Resolved})),
                accounts' = accounts.setBy(r.client_id, (old => {...old, available: old.available + transactions.get(r.transaction_id).amount, held: old.held - transactions.get(r.transaction_id).amount})),
                records' = records.append(operation)
            }
            | Chargeback(c) => all {
                transactions.keys().contains(c.transaction_id),
                transactions.get(c.transaction_id).status == Disputed,
                transactions.get(c.transaction_id).client_id == c.client_id,
                accounts' = accounts.setBy(c.client_id, (old => ({ ...old, locked: true, held: old.held - transactions.get(c.transaction_id).amount}))),
                transactions' = transactions.setBy(c.transaction_id, (old => ({...old, status: ChargedBack}))),
                records' = records.append(operation)
            }
        }
    }

    action step = {
        nondet client_id = ADDRESSES.oneOf()
        nondet transaction_id = 1.to(10000).oneOf()
        nondet amount = 1.to(1000000000000000).oneOf()
        any {
            deposit(client_id, transaction_id, amount),
            withdraw(client_id, transaction_id, amount),
            resolve(client_id, transaction_id),
            dispute(client_id, transaction_id),
            chargeback(client_id, transaction_id),
        }
    }

    val heldMatchesDisputes = ADDRESSES.forall(addr => {
        val acc = accounts.get(addr)
        acc.held == transactions.keys().filter(id => transactions.get(id).client_id == addr and transactions.get(id).status == Disputed).fold(0, (sum, id) => sum + transactions.get(id).amount)
    })
    val noNegativeHeld = ADDRESSES.forall(addr => accounts.get(addr).held >= 0)
    val everyInvariant = all { heldMatchesDisputes, noNegativeHeld }
}