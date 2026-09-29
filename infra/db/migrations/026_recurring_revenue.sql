CREATE TABLE IF NOT EXISTS subscription_plans (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  currency varchar(3) NOT NULL,
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor >= 0),
  interval_unit text NOT NULL CHECK (interval_unit IN ('MONTH','YEAR')),
  interval_count integer NOT NULL CHECK (interval_count > 0 AND interval_count <= 12),
  active boolean NOT NULL DEFAULT true,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,idempotency_key)
);
CREATE TABLE IF NOT EXISTS subscriptions (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  customer_id uuid NOT NULL REFERENCES customers(id),
  plan_id uuid NOT NULL REFERENCES subscription_plans(id),
  status text NOT NULL CHECK (status IN ('TRIALING','ACTIVE','PAST_DUE','PAUSED','CANCELLED')),
  started_at_epoch bigint NOT NULL,
  current_period_start_epoch bigint NOT NULL,
  current_period_end_epoch bigint NOT NULL,
  cancel_at_period_end boolean NOT NULL DEFAULT false,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,idempotency_key)
);
CREATE TABLE IF NOT EXISTS subscription_billing_periods (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  subscription_id uuid NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
  period_start_epoch bigint NOT NULL,
  period_end_epoch bigint NOT NULL,
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor >= 0),
  currency varchar(3) NOT NULL,
  status text NOT NULL CHECK (status IN ('PLANNED','INVOICED','PAID','VOID')),
  invoice_id uuid REFERENCES invoices(id),
  compliance_reference text,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,subscription_id,period_start_epoch),
  UNIQUE(company_id,idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_subscriptions_company_status ON subscriptions(company_id,status);
CREATE INDEX IF NOT EXISTS idx_billing_periods_due ON subscription_billing_periods(company_id,status,period_end_epoch);
