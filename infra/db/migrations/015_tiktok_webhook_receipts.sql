CREATE TABLE IF NOT EXISTS tiktok_webhook_receipts (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  event_key text NOT NULL,
  event_name text NOT NULL,
  publish_id text,
  payload_hash char(64) NOT NULL,
  received_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, event_key)
);

CREATE INDEX IF NOT EXISTS idx_tiktok_webhook_receipts_publish
  ON tiktok_webhook_receipts(company_id, publish_id);
