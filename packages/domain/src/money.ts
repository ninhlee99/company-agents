export type Money = { amountMinor: bigint; currency: string };

export function money(amountMinor: bigint, currency="USD"): Money {
  if (!Number.isInteger(Number(amountMinor))) throw new Error("amountMinor must be an integer");
  return { amountMinor, currency };
}