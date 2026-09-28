CREATE TABLE IF NOT EXISTS affiliate_revenue_recognition (
  company_id uuid NOT NULL REFERENCES companies(id),
  conversion_id text NOT NULL,
  currency char(3) NOT NULL,
  recognized_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (recognized_minor >= 0),
  status text NOT NULL CHECK (status IN ('PENDING','RECOGNIZED')),
  ledger_transaction_id uuid REFERENCES ledger_transactions(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, conversion_id)
);

CREATE INDEX IF NOT EXISTS idx_affiliate_revenue_recognition_company_status
  ON affiliate_revenue_recognition(company_id, status);

CREATE TABLE IF NOT EXISTS affiliate_payouts (
  company_id uuid NOT NULL REFERENCES companies(id),
  payout_id text NOT NULL,
  currency char(3) NOT NULL,
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor > 0),
  occurred_at timestamptz NOT NULL,
  ledger_transaction_id uuid REFERENCES ledger_transactions(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, payout_id)
);
