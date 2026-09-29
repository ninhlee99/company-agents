-- Harden the customer boundary used by commercial sales.
-- Existing rows remain valid; new records may carry stable external identity and
-- idempotency metadata so API retries cannot create duplicate customers.
ALTER TABLE customers
  ADD COLUMN IF NOT EXISTS email text,
  ADD COLUMN IF NOT EXISTS notes text,
  ADD COLUMN IF NOT EXISTS idempotency_key text;

CREATE UNIQUE INDEX IF NOT EXISTS idx_customers_company_idempotency
  ON customers(company_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_customers_company_external_ref
  ON customers(company_id, external_ref)
  WHERE external_ref IS NOT NULL;
