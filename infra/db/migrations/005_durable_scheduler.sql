CREATE TABLE IF NOT EXISTS company_schedules (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  job_type text NOT NULL,
  interval_seconds bigint NOT NULL CHECK (interval_seconds >= 15),
  next_run_at timestamptz NOT NULL,
  enabled boolean NOT NULL DEFAULT true,
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, job_type)
);

CREATE INDEX IF NOT EXISTS idx_company_schedules_due
  ON company_schedules(enabled, next_run_at);
