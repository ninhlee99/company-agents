CREATE TABLE IF NOT EXISTS tiktok_connections (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  open_id text NOT NULL,
  access_token text NOT NULL,
  refresh_token text NOT NULL,
  access_token_expires_at timestamptz NOT NULL,
  refresh_token_expires_at timestamptz NOT NULL,
  scopes text NOT NULL DEFAULT '',
  token_type text NOT NULL DEFAULT 'Bearer',
  status text NOT NULL DEFAULT 'ACTIVE',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, open_id)
);

CREATE INDEX IF NOT EXISTS idx_tiktok_connections_refresh
  ON tiktok_connections(company_id, access_token_expires_at, status);
