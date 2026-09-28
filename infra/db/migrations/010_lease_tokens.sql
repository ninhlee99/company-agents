CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE scheduled_jobs
  ADD COLUMN IF NOT EXISTS lease_token uuid;

UPDATE scheduled_jobs
   SET lease_token = gen_random_uuid()
 WHERE lease_token IS NULL;

ALTER TABLE scheduled_jobs
  ALTER COLUMN lease_token SET DEFAULT gen_random_uuid();

ALTER TABLE scheduled_jobs
  ALTER COLUMN lease_token SET NOT NULL;

ALTER TABLE media_jobs
  ADD COLUMN IF NOT EXISTS lease_token uuid;

UPDATE media_jobs
   SET lease_token = gen_random_uuid()
 WHERE status = 'RUNNING' AND lease_token IS NULL;

ALTER TABLE media_jobs
  ALTER COLUMN lease_token SET DEFAULT gen_random_uuid();

CREATE INDEX IF NOT EXISTS idx_media_jobs_lease
  ON media_jobs(company_id, status, locked_until, lease_token);

ALTER TABLE outbox_events
  ADD COLUMN IF NOT EXISTS lease_token uuid;

CREATE INDEX IF NOT EXISTS idx_outbox_events_lease
  ON outbox_events(company_id, published_at, locked_until, lease_token);
