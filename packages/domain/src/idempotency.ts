export class IdempotencyStore {
  private readonly seen = new Set<string>();

  claim(key: string): boolean {
    if (!key.trim()) throw new Error("Idempotency key is required");
    if (this.seen.has(key)) return false;
    this.seen.add(key);
    return true;
  }

  has(key: string): boolean {
    return this.seen.has(key);
  }
}
