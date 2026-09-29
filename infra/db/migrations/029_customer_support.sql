CREATE TABLE IF NOT EXISTS support_cases (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  customer_id uuid REFERENCES customers(id),
  channel text NOT NULL CHECK (channel IN ('EMAIL','WEB','CHAT','SOCIAL','INTERNAL')),
  subject text NOT NULL,
  description text NOT NULL,
  priority text NOT NULL CHECK (priority IN ('LOW','NORMAL','HIGH','URGENT')),
  status text NOT NULL CHECK (status IN ('OPEN','ACKNOWLEDGED','IN_PROGRESS','WAITING_CUSTOMER','RESOLVED','CLOSED')),
  owner_agent text,
  sla_due_at_epoch bigint,
  resolved_at_epoch bigint,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,idempotency_key)
);
CREATE TABLE IF NOT EXISTS support_case_events (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  case_id uuid NOT NULL REFERENCES support_cases(id) ON DELETE CASCADE,
  event_type text NOT NULL,
  actor text NOT NULL,
  evidence_hash text,
  notes text,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS support_case_feedback (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  case_id uuid NOT NULL REFERENCES support_cases(id) ON DELETE CASCADE,
  rating smallint CHECK (rating BETWEEN 1 AND 5),
  feedback text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,case_id)
);
CREATE INDEX IF NOT EXISTS idx_support_cases_queue ON support_cases(company_id,status,priority,created_at);
CREATE INDEX IF NOT EXISTS idx_support_cases_sla ON support_cases(company_id,sla_due_at_epoch,status);
