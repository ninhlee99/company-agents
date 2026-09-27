export type Money = { amountMinor: bigint; currency: string };

const CURRENCY = /^[A-Z]{3}$/;

export function money(amountMinor: bigint, currency="USD"): Money {
  if (typeof amountMinor !== "bigint") throw new Error("amountMinor must be bigint");
  if (!CURRENCY.test(currency)) throw new Error("currency must be a 3-letter ISO-style code");
  return { amountMinor, currency };
}
