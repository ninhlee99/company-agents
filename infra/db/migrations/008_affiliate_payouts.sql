CREATE TABLE IF NOT EXISTS affiliate_payouts (
  company_id uuid NOT NULL REFERENCES companies(id),
  payout_id text NOT NULL,
  occurred_at text NOT NULL,
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor >= 0),
  currency text NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
  source text NOT NULL,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, payout_id),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_affiliate_payouts_time
  ON affiliate_payouts(company_id, occurred_at DESC);
