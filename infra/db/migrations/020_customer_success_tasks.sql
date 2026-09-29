CREATE TABLE IF NOT EXISTS customer_success_tasks (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  customer_id uuid NOT NULL REFERENCES customers(id),
  task_type text NOT NULL CHECK (task_type IN ('ONBOARDING','HEALTH_REVIEW','RENEWAL','EXPANSION','RISK_REVIEW')),
  due_at_epoch bigint NOT NULL,
  owner text,
  status text NOT NULL CHECK (status IN ('OPEN','IN_PROGRESS','COMPLETED','CANCELLED')),
  notes text,
  outcome text,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_customer_success_due
  ON customer_success_tasks(company_id,status,due_at_epoch);
