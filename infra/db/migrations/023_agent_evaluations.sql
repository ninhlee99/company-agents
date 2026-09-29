CREATE TABLE IF NOT EXISTS agent_evaluations (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  cycle_id text NOT NULL,
  agent_role text NOT NULL,
  action text NOT NULL,
  governance_decision text NOT NULL,
  confidence_bps integer NOT NULL CHECK (confidence_bps BETWEEN 0 AND 10000),
  evidence_count integer NOT NULL CHECK (evidence_count >= 0),
  proposal_cost_minor numeric(39,0) NOT NULL CHECK (proposal_cost_minor >= 0),
  expected_revenue_minor numeric(39,0) NOT NULL CHECK (expected_revenue_minor >= 0),
  outcome text NOT NULL CHECK (outcome IN ('OBSERVED','APPROVED','REJECTED','FAILED','UNKNOWN')),
  evaluation_score_bps integer NOT NULL CHECK (evaluation_score_bps BETWEEN 0 AND 10000),
  notes text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, cycle_id, agent_role)
);

CREATE TABLE IF NOT EXISTS agent_model_routes (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  agent_role text NOT NULL,
  model_name text NOT NULL,
  reason text NOT NULL,
  active boolean NOT NULL DEFAULT true,
  approved_by text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, agent_role, model_name)
);

CREATE INDEX IF NOT EXISTS idx_agent_evaluations_company_agent
  ON agent_evaluations(company_id, agent_role, created_at DESC);
