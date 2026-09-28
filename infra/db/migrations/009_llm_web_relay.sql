CREATE TABLE IF NOT EXISTS llm_web_relay_jobs (
  id uuid PRIMARY KEY,
  idempotency_key text NOT NULL,
  backend text NOT NULL CHECK (backend IN ('gemini-web','chatgpt-web','claude-web')),
  model text NOT NULL,
  system_prompt text NOT NULL,
  user_prompt text NOT NULL,
  response_format text NOT NULL CHECK (response_format = 'json_object'),
  allow_tools boolean NOT NULL DEFAULT false CHECK (allow_tools = false),
  status text NOT NULL CHECK (status IN ('QUEUED','RUNNING','SUCCEEDED','FAILED','EXPIRED','CANCELLED')),
  attempt int NOT NULL DEFAULT 0 CHECK (attempt >= 0),
  max_attempts int NOT NULL DEFAULT 3 CHECK (max_attempts BETWEEN 1 AND 10),
  lease_token uuid,
  locked_until timestamptz,
  expires_at timestamptz NOT NULL,
  output_json text,
  error_message text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_llm_web_relay_claim
  ON llm_web_relay_jobs(status, backend, expires_at, created_at);

CREATE INDEX IF NOT EXISTS idx_llm_web_relay_lease
  ON llm_web_relay_jobs(status, locked_until);

CREATE OR REPLACE FUNCTION touch_llm_web_relay_updated_at() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_llm_web_relay_updated_at ON llm_web_relay_jobs;
CREATE TRIGGER trg_llm_web_relay_updated_at
BEFORE UPDATE ON llm_web_relay_jobs
FOR EACH ROW EXECUTE FUNCTION touch_llm_web_relay_updated_at();
