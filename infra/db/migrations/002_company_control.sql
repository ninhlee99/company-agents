-- Company control-plane durability layer.
ALTER TABLE company_state_snapshots
  ADD COLUMN IF NOT EXISTS revision bigint NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS cycle_runs (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  status text NOT NULL CHECK (status IN ('PROCESSING','SUCCEEDED','FAILED')),
  response_json jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz
);

CREATE INDEX IF NOT EXISTS idx_cycle_runs_company_time
  ON cycle_runs(company_id, created_at DESC);

CREATE TABLE IF NOT EXISTS decision_journal (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  cycle_id uuid NOT NULL REFERENCES cycle_runs(id),
  agent_name text NOT NULL,
  action text NOT NULL,
  governance_decision text NOT NULL,
  execution_status text NOT NULL CHECK (execution_status IN ('EXECUTED','SKIPPED','FAILED')),
  proposal_json jsonb NOT NULL,
  execution_json jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_decision_journal_company_time
  ON decision_journal(company_id, created_at DESC);

ALTER TABLE idempotency_keys
  ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

CREATE OR REPLACE FUNCTION touch_idempotency_updated_at() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS idempotency_keys_touch_updated_at ON idempotency_keys;
CREATE TRIGGER idempotency_keys_touch_updated_at
BEFORE UPDATE ON idempotency_keys
FOR EACH ROW EXECUTE FUNCTION touch_idempotency_updated_at();
