CREATE TABLE IF NOT EXISTS creators (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  currency varchar(3) NOT NULL,
  status text NOT NULL,
  cash_minor numeric(39,0) NOT NULL DEFAULT 0,
  revenue_minor numeric(39,0) NOT NULL DEFAULT 0,
  expenses_minor numeric(39,0) NOT NULL DEFAULT 0,
  audience bigint NOT NULL DEFAULT 0,
  content_count bigint NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS content_assets (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  creator_id uuid NOT NULL REFERENCES creators(id),
  title text NOT NULL,
  channel text NOT NULL,
  status text NOT NULL,
  production_cost_minor numeric(39,0) NOT NULL DEFAULT 0,
  attributed_revenue_minor numeric(39,0) NOT NULL DEFAULT 0,
  affiliate_commission_minor numeric(39,0) NOT NULL DEFAULT 0,
  views bigint NOT NULL DEFAULT 0,
  clicks bigint NOT NULL DEFAULT 0,
  orders bigint NOT NULL DEFAULT 0,
  published_at_epoch bigint,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS experiments (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  hypothesis text NOT NULL,
  status text NOT NULL,
  budget_minor numeric(39,0) NOT NULL DEFAULT 0,
  spent_minor numeric(39,0) NOT NULL DEFAULT 0,
  expected_revenue_minor numeric(39,0) NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS employees (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  role text NOT NULL,
  status text NOT NULL,
  monthly_cost_minor numeric(39,0) NOT NULL DEFAULT 0,
  start_epoch bigint,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS contracts (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  counterparty text NOT NULL,
  contract_type text NOT NULL,
  status text NOT NULL,
  value_minor numeric(39,0) NOT NULL DEFAULT 0,
  start_epoch bigint NOT NULL,
  end_epoch bigint,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS tasks (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  title text NOT NULL,
  status text NOT NULL,
  priority smallint NOT NULL DEFAULT 0,
  owner_agent text,
  creator_id uuid REFERENCES creators(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_creators_company ON creators(company_id);
CREATE INDEX IF NOT EXISTS idx_content_assets_company_creator ON content_assets(company_id, creator_id);
CREATE INDEX IF NOT EXISTS idx_experiments_company ON experiments(company_id);
CREATE INDEX IF NOT EXISTS idx_employees_company_status ON employees(company_id, status);
CREATE INDEX IF NOT EXISTS idx_contracts_company_status ON contracts(company_id, status);
CREATE INDEX IF NOT EXISTS idx_tasks_company_status_priority ON tasks(company_id, status, priority DESC);
