CREATE TABLE IF NOT EXISTS payment_reconciliation_evidence (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  invoice_id uuid NOT NULL REFERENCES invoices(id),
  provider text NOT NULL,
  provider_event_id text NOT NULL,
  external_ref text,
  observed_amount_minor numeric(39,0) NOT NULL CHECK (observed_amount_minor > 0),
  currency varchar(3) NOT NULL,
  observed_at_epoch bigint NOT NULL,
  evidence_hash char(64) NOT NULL,
  status text NOT NULL CHECK (status IN ('OBSERVED','APPLIED','REJECTED')),
  reason text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, provider, provider_event_id)
);
CREATE INDEX IF NOT EXISTS idx_payment_reconciliation_invoice
  ON payment_reconciliation_evidence(company_id, invoice_id, status);
