CREATE TABLE IF NOT EXISTS business_units (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  currency char(3) NOT NULL,
  cash_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (cash_minor >= 0),
  revenue_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (revenue_minor >= 0),
  variable_cost_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (variable_cost_minor >= 0),
  fixed_cost_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (fixed_cost_minor >= 0),
  budget_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (budget_minor >= 0),
  lifecycle text NOT NULL CHECK (lifecycle IN ('TESTING','GROWING','STABLE','DISTRESS','PAUSED','CLOSED')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, name)
);

CREATE TABLE IF NOT EXISTS employees (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  role text NOT NULL,
  monthly_cost_minor numeric(39,0) NOT NULL CHECK (monthly_cost_minor >= 0),
  currency char(3) NOT NULL,
  status text NOT NULL CHECK (status IN ('PROPOSED','ACTIVE','SUSPENDED','TERMINATED')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payroll_obligations (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  employee_id uuid NOT NULL REFERENCES employees(id),
  period text NOT NULL,
  gross_minor numeric(39,0) NOT NULL CHECK (gross_minor >= 0),
  currency char(3) NOT NULL,
  due_at timestamptz NOT NULL,
  paid_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (paid_minor >= 0 AND paid_minor <= gross_minor),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, employee_id, period)
);

CREATE INDEX IF NOT EXISTS idx_payroll_due
  ON payroll_obligations(company_id, due_at, paid_minor);

CREATE INDEX IF NOT EXISTS idx_employees_company_status
  ON employees(company_id, status);
