CREATE TABLE IF NOT EXISTS autonomy_simulations (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  idempotency_key text NOT NULL,
  proposal jsonb NOT NULL,
  decision text NOT NULL,
  ceiling text NOT NULL,
  required_level text NOT NULL,
  reason text NOT NULL,
  simulation jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_autonomy_simulations_company_time
  ON autonomy_simulations(company_id, created_at DESC);

CREATE OR REPLACE FUNCTION reject_autonomy_simulation_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'autonomy_simulations is append-only';
END;
$$;

DROP TRIGGER IF EXISTS autonomy_simulations_no_update ON autonomy_simulations;
DROP TRIGGER IF EXISTS autonomy_simulations_no_delete ON autonomy_simulations;
CREATE TRIGGER autonomy_simulations_no_update
BEFORE UPDATE ON autonomy_simulations
FOR EACH ROW EXECUTE FUNCTION reject_autonomy_simulation_mutation();
CREATE TRIGGER autonomy_simulations_no_delete
BEFORE DELETE ON autonomy_simulations
FOR EACH ROW EXECUTE FUNCTION reject_autonomy_simulation_mutation();
