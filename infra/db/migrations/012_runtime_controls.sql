CREATE TABLE IF NOT EXISTS agent_memory (
  company_id uuid NOT NULL REFERENCES companies(id),
  agent_name text NOT NULL,
  memory_key text NOT NULL,
  value jsonb NOT NULL,
  confidence_bps int NOT NULL DEFAULT 0 CHECK (confidence_bps BETWEEN 0 AND 10000),
  importance smallint NOT NULL DEFAULT 0 CHECK (importance BETWEEN 0 AND 100),
  expires_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, agent_name, memory_key)
);

CREATE INDEX IF NOT EXISTS idx_agent_memory_lookup
  ON agent_memory(company_id, agent_name, importance DESC, updated_at DESC);

CREATE TABLE IF NOT EXISTS agent_rate_windows (
  company_id uuid NOT NULL REFERENCES companies(id),
  agent_name text NOT NULL,
  window_started_at timestamptz NOT NULL,
  call_count int NOT NULL DEFAULT 0 CHECK (call_count >= 0),
  max_calls int NOT NULL CHECK (max_calls > 0),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, agent_name)
);

CREATE INDEX IF NOT EXISTS idx_agent_rate_windows_updated
  ON agent_rate_windows(updated_at DESC);

CREATE OR REPLACE FUNCTION touch_agent_memory_updated_at() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_agent_memory_updated_at ON agent_memory;
CREATE TRIGGER trg_agent_memory_updated_at
BEFORE UPDATE ON agent_memory
FOR EACH ROW EXECUTE FUNCTION touch_agent_memory_updated_at();
