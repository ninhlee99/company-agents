CREATE TABLE IF NOT EXISTS control_plane_audit_log (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  actor_id text NOT NULL,
  actor_role text NOT NULL,
  method text NOT NULL,
  path text NOT NULL,
  action text NOT NULL,
  outcome text NOT NULL CHECK (outcome IN ('ALLOWED','DENIED')),
  request_id text,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_control_plane_audit_company_time
  ON control_plane_audit_log(company_id, created_at DESC);
