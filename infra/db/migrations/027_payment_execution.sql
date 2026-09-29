CREATE TABLE IF NOT EXISTS payment_execution_intents (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  invoice_id uuid NOT NULL REFERENCES invoices(id),
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor > 0),
  currency varchar(3) NOT NULL,
  provider text NOT NULL,
  payment_method_ref text NOT NULL,
  status text NOT NULL CHECK (status IN ('PENDING_APPROVAL','APPROVED','SUBMITTED','SUCCEEDED','FAILED','CANCELLED')),
  approval_reference text,
  approved_by text,
  approved_at_epoch bigint,
  submitted_at_epoch bigint,
  completed_at_epoch bigint,
  provider_execution_ref text,
  failure_reason text,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS payment_execution_evidence (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  intent_id uuid NOT NULL REFERENCES payment_execution_intents(id),
  provider text NOT NULL,
  provider_execution_ref text NOT NULL,
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor > 0),
  currency varchar(3) NOT NULL,
  observed_at_epoch bigint NOT NULL,
  evidence_hash char(64) NOT NULL,
  status text NOT NULL CHECK (status IN ('OBSERVED','RECONCILED','REJECTED')),
  reason text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, provider, provider_execution_ref)
);

CREATE INDEX IF NOT EXISTS idx_payment_execution_invoice
  ON payment_execution_intents(company_id, invoice_id, status);
CREATE INDEX IF NOT EXISTS idx_payment_execution_evidence_intent
  ON payment_execution_evidence(company_id, intent_id, status);
