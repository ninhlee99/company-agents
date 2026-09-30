CREATE TABLE IF NOT EXISTS business_units (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  currency varchar(3) NOT NULL,
  status text NOT NULL,
  cash_minor numeric(39,0) NOT NULL DEFAULT 0,
  revenue_minor numeric(39,0) NOT NULL DEFAULT 0,
  expenses_minor numeric(39,0) NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS customers (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  external_ref text,
  status text NOT NULL,
  lifetime_revenue_minor numeric(39,0) NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS products (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  category text NOT NULL,
  currency varchar(3) NOT NULL,
  price_minor numeric(39,0) NOT NULL DEFAULT 0,
  active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payroll_runs (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  period_start_epoch bigint NOT NULL,
  period_end_epoch bigint NOT NULL,
  gross_minor numeric(39,0) NOT NULL,
  employer_cost_minor numeric(39,0) NOT NULL,
  cash_due_minor numeric(39,0) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payroll_lines (
  id bigserial PRIMARY KEY,
  payroll_run_id uuid NOT NULL REFERENCES payroll_runs(id) ON DELETE CASCADE,
  employee_id uuid NOT NULL REFERENCES employees(id),
  gross_minor numeric(39,0) NOT NULL,
  employer_cost_minor numeric(39,0) NOT NULL,
  withholding_minor numeric(39,0) NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_business_units_company ON business_units(company_id);
CREATE INDEX IF NOT EXISTS idx_customers_company ON customers(company_id);
CREATE INDEX IF NOT EXISTS idx_products_company_category ON products(company_id, category);
CREATE INDEX IF NOT EXISTS idx_payroll_runs_company_period ON payroll_runs(company_id, period_start_epoch DESC);
