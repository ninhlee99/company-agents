CREATE TABLE IF NOT EXISTS publish_intents (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  content_id text NOT NULL,
  platform text NOT NULL CHECK (platform IN ('TIKTOK','YOUTUBE','INSTAGRAM','FACEBOOK')),
  media_uri text NOT NULL,
  title text NOT NULL,
  caption text NOT NULL,
  scheduled_at timestamptz,
  content_hash char(64) NOT NULL,
  created_by text NOT NULL,
  idempotency_key text NOT NULL,
  status text NOT NULL CHECK (
    status IN ('DRAFT','APPROVED','RUNNING','SUCCEEDED','FAILED','REVOKED')
  ),
  approval_token_hash text,
  approval_expires_at timestamptz,
  approved_by text,
  execution_token uuid,
  locked_until timestamptz,
  attempts int NOT NULL DEFAULT 0 CHECK (attempts >= 0 AND attempts <= 10),
  external_reference text,
  error_message text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_publish_intents_queue
  ON publish_intents(company_id, status, scheduled_at, created_at);

CREATE INDEX IF NOT EXISTS idx_publish_intents_running
  ON publish_intents(status, locked_until);

CREATE OR REPLACE FUNCTION touch_publish_intent_updated_at() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_publish_intents_updated_at ON publish_intents;
CREATE TRIGGER trg_publish_intents_updated_at
BEFORE UPDATE ON publish_intents
FOR EACH ROW EXECUTE FUNCTION touch_publish_intent_updated_at();
