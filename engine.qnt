module engine {
    type Account = {id: int, available: int, held: int, locked: bool}
    type Operation =
        Deposit({client_id: int, transaction_id: int, amount: int})
        | Withdrawal({client_id: int, transaction_id: int, amount: int})
        | Dispute({client_id: int, transaction_id: int})
        | Resolve({client_id: int, transaction_id: int})
        | Chargeback({client_id: int, transaction_id: int})
    type TransactionStatus = Completed | Disputed | Resolved | ChargedBack
    type TransactionKind = Credit | Debit
    type Transaction = {id: int, amount: int, status: TransactionStatus, client_id: int, kind: TransactionKind}

    var accounts: int -> Account
    var transactions: int -> Transaction
    var records: List[Operation]

    pure val CLIENTS = 1.to(5)
    pure val TRANSACTION_IDS = 1.to(20)
    pure val AMOUNTS = 1.to(100)

    def total(account: Account): int = account.available + account.held

    def account(client_id: int): Account =
        if (accounts.keys().contains(client_id)) accounts.get(client_id)
        else {id: client_id, available: 0, held: 0, locked: false}

    def isRecorded(transaction_id: int): bool = transactions.keys().contains(transaction_id)

    def owns(client_id: int, transaction_id: int): bool =
        isRecorded(transaction_id) and transactions.get(transaction_id).client_id == client_id

    action unchanged = all {
        accounts' = accounts,
        transactions' = transactions,
        records' = records
    }

    action init = all {
        accounts' = Map(),
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
                d.amount > 0,
                not(isRecorded(d.transaction_id)),
                not(account(d.client_id).locked),
                accounts' = accounts.put(d.client_id, {...account(d.client_id), available: account(d.client_id).available + d.amount}),
                transactions' = transactions.put(d.transaction_id, {id: d.transaction_id, amount: d.amount, status: Completed, client_id: d.client_id, kind: Credit}),
                records' = records.append(operation)
            }
            | Withdrawal(w) => all {
                w.amount > 0,
                not(isRecorded(w.transaction_id)),
                not(account(w.client_id).locked),
                account(w.client_id).available >= w.amount,
                accounts' = accounts.put(w.client_id, {...account(w.client_id), available: account(w.client_id).available - w.amount}),
                transactions' = transactions.put(w.transaction_id, {id: w.transaction_id, amount: w.amount, status: Completed, client_id: w.client_id, kind: Debit}),
                records' = records.append(operation)
            }
            | Dispute(d) => all {
                owns(d.client_id, d.transaction_id),
                transactions.get(d.transaction_id).status == Completed,
                transactions.get(d.transaction_id).kind == Credit,
                val amount = transactions.get(d.transaction_id).amount
                all {
                    accounts' = accounts.setBy(d.client_id, old => {...old, available: old.available - amount, held: old.held + amount}),
                    transactions' = transactions.setBy(d.transaction_id, old => {...old, status: Disputed}),
                    records' = records.append(operation)
                }
            }
            | Resolve(r) => all {
                owns(r.client_id, r.transaction_id),
                transactions.get(r.transaction_id).status == Disputed,
                val amount = transactions.get(r.transaction_id).amount
                all {
                    accounts' = accounts.setBy(r.client_id, old => {...old, available: old.available + amount, held: old.held - amount}),
                    transactions' = transactions.setBy(r.transaction_id, old => {...old, status: Resolved}),
                    records' = records.append(operation)
                }
            }
            | Chargeback(c) => all {
                owns(c.client_id, c.transaction_id),
                transactions.get(c.transaction_id).status == Disputed,
                val amount = transactions.get(c.transaction_id).amount
                all {
                    accounts' = accounts.setBy(c.client_id, old => {...old, held: old.held - amount, locked: true}),
                    transactions' = transactions.setBy(c.transaction_id, old => {...old, status: ChargedBack}),
                    records' = records.append(operation)
                }
            }
        }
    }

    action step = {
        nondet client_id = CLIENTS.oneOf()
        nondet transaction_id = TRANSACTION_IDS.oneOf()
        nondet amount = AMOUNTS.oneOf()
        any {
            deposit(client_id, transaction_id, amount),
            withdraw(client_id, transaction_id, amount),
            dispute(client_id, transaction_id),
            resolve(client_id, transaction_id),
            chargeback(client_id, transaction_id),
        }
    }

    def clientTransactions(client_id: int): Set[Transaction] =
        transactions.keys().map(id => transactions.get(id)).filter(t => t.client_id == client_id)

    def sumAmounts(txs: Set[Transaction]): int = txs.fold(0, (sum, t) => sum + t.amount)

    val heldMatchesDisputes = accounts.keys().forall(id =>
        accounts.get(id).held == sumAmounts(clientTransactions(id).filter(t => t.status == Disputed))
    )

    val totalMatchesLedger = accounts.keys().forall(id => {
        val txs = clientTransactions(id)
        total(accounts.get(id)) ==
            sumAmounts(txs.filter(t => t.kind == Credit and t.status != ChargedBack))
            - sumAmounts(txs.filter(t => t.kind == Debit))
    })

    val lockedIffChargedBack = accounts.keys().forall(id =>
        accounts.get(id).locked == clientTransactions(id).exists(t => t.status == ChargedBack)
    )

    val noNegativeHeld = accounts.keys().forall(id => accounts.get(id).held >= 0)

    val onlyCreditsDisputed = transactions.keys().forall(id =>
        transactions.get(id).status != Completed implies transactions.get(id).kind == Credit
    )

    val transactionsHaveAccounts = transactions.keys().forall(id =>
        accounts.keys().contains(transactions.get(id).client_id)
    )

    val everyInvariant = all {
        heldMatchesDisputes,
        totalMatchesLedger,
        lockedIffChargedBack,
        noNegativeHeld,
        onlyCreditsDisputed,
        transactionsHaveAccounts,
    }
}
