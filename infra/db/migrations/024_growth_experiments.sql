CREATE TABLE IF NOT EXISTS growth_experiments (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  hypothesis text NOT NULL,
  control_variant text NOT NULL,
  treatment_variant text NOT NULL,
  max_budget_minor numeric(39,0) NOT NULL CHECK (max_budget_minor > 0),
  min_observations bigint NOT NULL CHECK (min_observations > 0),
  duration_seconds bigint NOT NULL CHECK (duration_seconds > 0),
  success_metric_bps integer NOT NULL CHECK (success_metric_bps BETWEEN 0 AND 10000),
  kill_metric_bps integer NOT NULL CHECK (kill_metric_bps BETWEEN 0 AND 10000),
  status text NOT NULL CHECK (status IN ('PROPOSED','RUNNING','SUCCEEDED','FAILED','KILLED','EXPIRED')),
  created_at timestamptz NOT NULL DEFAULT now(),
  started_at timestamptz,
  completed_at timestamptz
);

CREATE INDEX IF NOT EXISTS idx_growth_experiments_company_status
  ON growth_experiments(company_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS growth_experiment_observations (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  experiment_id uuid NOT NULL REFERENCES growth_experiments(id) ON DELETE CASCADE,
  control_observations bigint NOT NULL CHECK (control_observations >= 0),
  treatment_observations bigint NOT NULL CHECK (treatment_observations >= 0),
  control_metric_bps bigint NOT NULL CHECK (control_metric_bps BETWEEN 0 AND 10000),
  treatment_metric_bps bigint NOT NULL CHECK (treatment_metric_bps BETWEEN 0 AND 10000),
  spend_minor numeric(39,0) NOT NULL CHECK (spend_minor >= 0),
  elapsed_seconds bigint NOT NULL CHECK (elapsed_seconds >= 0),
  decision text NOT NULL CHECK (decision IN ('CONTINUE','SUCCEED','KILL','EXPIRE')),
  observation_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, experiment_id, observation_key)
);

CREATE INDEX IF NOT EXISTS idx_growth_experiment_observations_experiment
  ON growth_experiment_observations(company_id, experiment_id, created_at DESC);
