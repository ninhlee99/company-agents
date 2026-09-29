CREATE TABLE IF NOT EXISTS compliance_obligations (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  title text NOT NULL,
  obligation_type text NOT NULL,
  jurisdiction text,
  status text NOT NULL CHECK (status IN ('OPEN','IN_REVIEW','SATISFIED','WAIVED','OVERDUE')),
  due_at_epoch bigint NOT NULL CHECK (due_at_epoch > 0),
  owner text,
  source_reference text,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,idempotency_key)
);
CREATE TABLE IF NOT EXISTS compliance_evidence (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  obligation_id uuid NOT NULL REFERENCES compliance_obligations(id) ON DELETE CASCADE,
  evidence_type text NOT NULL,
  evidence_hash text NOT NULL,
  reference text,
  submitted_by text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS compliance_approvals (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  obligation_id uuid NOT NULL REFERENCES compliance_obligations(id) ON DELETE CASCADE,
  decision text NOT NULL CHECK (decision IN ('APPROVED','REJECTED')),
  approver text NOT NULL,
  approval_reference text NOT NULL,
  notes text,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS compliance_audit_events (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  obligation_id uuid REFERENCES compliance_obligations(id) ON DELETE CASCADE,
  event_type text NOT NULL,
  actor text NOT NULL,
  evidence_hash text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_compliance_obligations_due ON compliance_obligations(company_id,status,due_at_epoch);
CREATE INDEX IF NOT EXISTS idx_compliance_evidence_obligation ON compliance_evidence(obligation_id,created_at DESC);
CREATE INDEX IF NOT EXISTS idx_compliance_audit_obligation ON compliance_audit_events(obligation_id,created_at DESC);

CREATE TABLE IF NOT EXISTS compliance_revenue_links (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  obligation_id uuid NOT NULL REFERENCES compliance_obligations(id) ON DELETE CASCADE,
  invoice_id uuid NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
  linked_by text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,obligation_id,invoice_id)
);
CREATE TABLE IF NOT EXISTS compliance_payment_links (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  obligation_id uuid NOT NULL REFERENCES compliance_obligations(id) ON DELETE CASCADE,
  invoice_id uuid NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
  linked_by text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,obligation_id,invoice_id)
);
