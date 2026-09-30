CREATE TABLE IF NOT EXISTS browser_sessions (
  company_id UUID NOT NULL REFERENCES companies(id),
  session_fingerprint TEXT NOT NULL,
  expires_at_epoch BIGINT NOT NULL,
  revoked_at_epoch BIGINT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, session_fingerprint)
);

CREATE INDEX IF NOT EXISTS idx_browser_sessions_active
  ON browser_sessions(company_id, expires_at_epoch, revoked_at_epoch);
