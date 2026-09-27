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
  if (!tx.id.trim()) throw new Error("Transaction id is required");
  if (!tx.entries.length) throw new Error("Transaction requires entries");
  const currencies = new Set(tx.entries.map(e => e.currency));
  if (currencies.size !== 1) throw new Error("All ledger entries must use the same currency");
  const debit = tx.entries.reduce((n,e)=>n+e.debitMinor,0n);
  const credit = tx.entries.reduce((n,e)=>n+e.creditMinor,0n);
  if (debit !== credit) throw new Error("Unbalanced ledger transaction");
  for (const e of tx.entries) {
    if (!e.accountId.trim()) throw new Error("Ledger account id is required");
    if (e.debitMinor < 0n || e.creditMinor < 0n) throw new Error("Negative ledger entry");
    if (e.debitMinor > 0n && e.creditMinor > 0n) throw new Error("Entry cannot contain both debit and credit");
    if (e.debitMinor === 0n && e.creditMinor === 0n) throw new Error("Zero-value ledger entry");
  }
}
