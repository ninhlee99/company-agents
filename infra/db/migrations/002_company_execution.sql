ALTER TABLE agent_runs
  ADD COLUMN IF NOT EXISTS idempotency_key text;

CREATE UNIQUE INDEX IF NOT EXISTS uq_agent_runs_company_idempotency
  ON agent_runs(company_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS decision_journal (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  idempotency_key text NOT NULL,
  agent_name text NOT NULL,
  action text NOT NULL,
  governor_decision text NOT NULL,
  reason text NOT NULL,
  proposal jsonb NOT NULL,
  execution jsonb,
  executed boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_decision_journal_company_time
  ON decision_journal(company_id, created_at DESC);

CREATE TABLE IF NOT EXISTS scheduled_jobs (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  job_type text NOT NULL,
  interval_seconds bigint NOT NULL CHECK (interval_seconds >= 15),
  next_run_at timestamptz NOT NULL,
  locked_until timestamptz,
  run_token uuid NOT NULL DEFAULT gen_random_uuid(),
  status text NOT NULL CHECK (status IN ('ACTIVE','PAUSED')),
  payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, job_type)
);

CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_due
  ON scheduled_jobs(status, next_run_at);

ALTER TABLE scheduled_jobs
  ADD COLUMN IF NOT EXISTS locked_until timestamptz;

ALTER TABLE scheduled_jobs
  ADD COLUMN IF NOT EXISTS run_token uuid;

UPDATE scheduled_jobs
   SET run_token = gen_random_uuid()
 WHERE run_token IS NULL;

ALTER TABLE scheduled_jobs
  ALTER COLUMN run_token SET DEFAULT gen_random_uuid();

ALTER TABLE scheduled_jobs
  ALTER COLUMN run_token SET NOT NULL;

DO $
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'companies_currency_uppercase'
      AND conrelid = 'companies'::regclass
  ) THEN
    ALTER TABLE companies
      ADD CONSTRAINT companies_currency_uppercase
      CHECK (base_currency = upper(base_currency));
  END IF;
END;
$;

CREATE OR REPLACE FUNCTION validate_ledger_transaction_presence() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM ledger_entries WHERE transaction_id = NEW.id
  ) THEN
    RAISE EXCEPTION 'ledger transaction requires entries';
  END IF;
  RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS ledger_transaction_requires_entries ON ledger_transactions;
CREATE CONSTRAINT TRIGGER ledger_transaction_requires_entries
AFTER INSERT ON ledger_transactions
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_ledger_transaction_presence();

CREATE TABLE IF NOT EXISTS company_cycle_health (
  company_id uuid PRIMARY KEY REFERENCES companies(id),
  last_started_at timestamptz,
  last_succeeded_at timestamptz,
  last_failed_at timestamptz,
  last_error text,
  consecutive_failures int NOT NULL DEFAULT 0,
  updated_at timestamptz NOT NULL DEFAULT now()
);
