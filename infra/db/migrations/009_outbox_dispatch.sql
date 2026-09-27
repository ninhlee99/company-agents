ALTER TABLE outbox_events
  ADD COLUMN IF NOT EXISTS attempt_count int NOT NULL DEFAULT 0 CHECK (attempt_count >= 0);

ALTER TABLE outbox_events
  ADD COLUMN IF NOT EXISTS locked_until timestamptz;

ALTER TABLE outbox_events
  ADD COLUMN IF NOT EXISTS last_error text;

CREATE INDEX IF NOT EXISTS idx_outbox_events_dispatch
  ON outbox_events(company_id, published_at, locked_until, created_at);
