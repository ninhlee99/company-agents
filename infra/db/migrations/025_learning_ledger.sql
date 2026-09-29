CREATE TABLE IF NOT EXISTS learning_entries (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  entry_key text NOT NULL,
  source_type text NOT NULL,
  source_id text NOT NULL,
  kind text NOT NULL CHECK (kind IN ('LEARNING','FAILURE','NEAR_MISS','SUCCESS')),
  severity text NOT NULL CHECK (severity IN ('NONE','LOW','MEDIUM','HIGH','CRITICAL')),
  hypothesis text NOT NULL,
  context text NOT NULL,
  expected_outcome text NOT NULL,
  actual_outcome text NOT NULL,
  impact_minor numeric(39,0) NOT NULL,
  confidence_bps integer NOT NULL CHECK (confidence_bps BETWEEN 0 AND 10000),
  root_cause text NOT NULL,
  corrective_action text NOT NULL,
  reusable_rule text NOT NULL,
  decision text NOT NULL CHECK (decision IN ('REUSE','ADJUST','RETEST','STOP','ESCALATE')),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, entry_key)
);

CREATE INDEX IF NOT EXISTS idx_learning_entries_company_created
  ON learning_entries(company_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_learning_entries_company_kind
  ON learning_entries(company_id, kind, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_learning_entries_company_source
  ON learning_entries(company_id, source_type, source_id, created_at DESC);
