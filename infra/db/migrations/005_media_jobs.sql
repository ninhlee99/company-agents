CREATE TABLE IF NOT EXISTS media_jobs (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  input_path text NOT NULL,
  output_path text NOT NULL,
  format text NOT NULL CHECK (format IN ('Mp4H264','WebMvp9')),
  width int NOT NULL CHECK (width BETWEEN 16 AND 8192),
  height int NOT NULL CHECK (height BETWEEN 16 AND 8192),
  fps int NOT NULL CHECK (fps BETWEEN 1 AND 120),
  max_duration_seconds int NOT NULL CHECK (max_duration_seconds BETWEEN 1 AND 86400),
  normalize_audio boolean NOT NULL,
  status text NOT NULL CHECK (status IN ('QUEUED','RUNNING','SUCCEEDED','FAILED','QA_FAILED')),
  attempts int NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  locked_until timestamptz,
  last_error text,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_media_jobs_queue
  ON media_jobs(status, locked_until, created_at);
