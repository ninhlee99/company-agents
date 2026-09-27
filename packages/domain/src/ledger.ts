export type LedgerEntry = {
  accountId: string;
  debitMinor: bigint;
  creditMinor: bigint;
  currency: string;
};

export type LedgerTransaction = {
  id: string;
  description: string;
  entries: LedgerEntry[];
};

export function validateBalancedTransaction(tx: LedgerTransaction): void {
  if (!tx.entries.length) throw new Error("Transaction requires entries");
  const debit = tx.entries.reduce((n,e)=>n+e.debitMinor,0n);
  const credit = tx.entries.reduce((n,e)=>n+e.creditMinor,0n);
  if (debit !== credit) throw new Error("Unbalanced ledger transaction");
  for (const e of tx.entries) {
    if (e.debitMinor < 0n || e.creditMinor < 0n) throw new Error("Negative ledger entry");
    if (e.debitMinor > 0n && e.creditMinor > 0n) throw new Error("Entry cannot contain both debit and credit");
  }
}