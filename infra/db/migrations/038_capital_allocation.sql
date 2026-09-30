CREATE TABLE IF NOT EXISTS capital_allocation_plans (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  plan_key text NOT NULL,
  policy_json jsonb NOT NULL,
  total_capital_minor numeric(39,0) NOT NULL CHECK (total_capital_minor >= 0),
  planned_capital_minor numeric(39,0) NOT NULL CHECK (planned_capital_minor >= 0),
  unallocated_minor numeric(39,0) NOT NULL CHECK (unallocated_minor >= 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, plan_key)
);

CREATE TABLE IF NOT EXISTS capital_allocation_candidates (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  plan_id uuid NOT NULL,
  candidate_key text NOT NULL,
  candidate_json jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, plan_id, candidate_key),
  FOREIGN KEY (company_id, plan_id)
    REFERENCES capital_allocation_plans(company_id, id)
);

CREATE TABLE IF NOT EXISTS capital_allocation_decisions (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  plan_id uuid NOT NULL,
  candidate_id uuid NOT NULL,
  decision text NOT NULL CHECK (decision IN ('ALLOCATE','HOLD','REJECT')),
  score_bps integer NOT NULL CHECK (score_bps BETWEEN 0 AND 10000),
  allocation_minor numeric(39,0) NOT NULL CHECK (allocation_minor >= 0),
  reason text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, plan_id, candidate_id),
  FOREIGN KEY (company_id, plan_id)
    REFERENCES capital_allocation_plans(company_id, id),
  FOREIGN KEY (company_id, candidate_id)
    REFERENCES capital_allocation_candidates(company_id, id)
);

CREATE INDEX IF NOT EXISTS idx_capital_allocation_plans_company_created
  ON capital_allocation_plans(company_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_capital_allocation_decisions_company_plan
  ON capital_allocation_decisions(company_id, plan_id, score_bps DESC);
