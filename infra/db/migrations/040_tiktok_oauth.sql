CREATE TABLE IF NOT EXISTS tiktok_oauth_states (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  state_hash text NOT NULL,
  redirect_uri text NOT NULL,
  scopes text NOT NULL,
  expires_at_epoch bigint NOT NULL CHECK (expires_at_epoch > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, state_hash)
);

CREATE INDEX IF NOT EXISTS idx_tiktok_oauth_states_expiry
  ON tiktok_oauth_states(company_id, expires_at_epoch);

CREATE TABLE IF NOT EXISTS tiktok_oauth_connections (
  company_id uuid PRIMARY KEY REFERENCES companies(id),
  open_id text NOT NULL,
  encrypted_access_token text,
  encrypted_refresh_token text,
  access_token_expires_at_epoch bigint,
  refresh_token_expires_at_epoch bigint,
  scopes text NOT NULL DEFAULT '',
  token_type text NOT NULL DEFAULT 'Bearer',
  status text NOT NULL DEFAULT 'ACTIVE'
    CHECK (status IN ('ACTIVE','REVOKED','REAUTH_REQUIRED')),
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CHECK (
    status <> 'ACTIVE'
    OR (
      encrypted_access_token IS NOT NULL
      AND encrypted_refresh_token IS NOT NULL
      AND access_token_expires_at_epoch IS NOT NULL
      AND refresh_token_expires_at_epoch IS NOT NULL
    )
  ),
  CHECK (
    access_token_expires_at_epoch IS NULL
    OR access_token_expires_at_epoch > 0
  ),
  CHECK (
    refresh_token_expires_at_epoch IS NULL
    OR refresh_token_expires_at_epoch > 0
  )
);

CREATE INDEX IF NOT EXISTS idx_tiktok_oauth_connections_refresh
  ON tiktok_oauth_connections(company_id, status, access_token_expires_at_epoch);

CREATE INDEX IF NOT EXISTS idx_tiktok_oauth_connections_refresh_expiry
  ON tiktok_oauth_connections(company_id, status, refresh_token_expires_at_epoch);
