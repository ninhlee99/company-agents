CREATE TABLE IF NOT EXISTS incidents (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  title text NOT NULL,
  description text NOT NULL,
  severity text NOT NULL CHECK (severity IN ('SEV1','SEV2','SEV3','SEV4')),
  status text NOT NULL CHECK (status IN ('OPEN','ACKNOWLEDGED','MITIGATING','RESOLVED','CLOSED')),
  source text NOT NULL,
  opened_at_epoch bigint NOT NULL,
  acknowledged_at_epoch bigint,
  resolved_at_epoch bigint,
  sla_minutes integer NOT NULL CHECK (sla_minutes > 0),
  owner text,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id,idempotency_key)
);
CREATE TABLE IF NOT EXISTS incident_events (
  id uuid PRIMARY KEY,
  incident_id uuid NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
  event_type text NOT NULL,
  actor text NOT NULL,
  evidence_hash text,
  notes text,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS incident_postmortems (
  id uuid PRIMARY KEY,
  incident_id uuid NOT NULL UNIQUE REFERENCES incidents(id) ON DELETE CASCADE,
  root_cause text NOT NULL,
  corrective_actions text NOT NULL,
  prevention_actions text NOT NULL,
  learning_notes text,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_incidents_company_status ON incidents(company_id,status,severity,opened_at_epoch DESC);
