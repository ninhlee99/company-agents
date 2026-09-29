CREATE TABLE IF NOT EXISTS policy_snapshots (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  policy_key text NOT NULL,
  platform text NOT NULL,
  jurisdiction text NOT NULL,
  version text NOT NULL,
  source_reference text NOT NULL,
  evidence_hash text NOT NULL,
  observed_at_epoch bigint NOT NULL CHECK (observed_at_epoch > 0),
  effective_at_epoch bigint NOT NULL CHECK (effective_at_epoch > 0),
  active boolean NOT NULL DEFAULT true,
  rules_json jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, policy_key, version)
);

CREATE INDEX IF NOT EXISTS idx_policy_snapshots_active
  ON policy_snapshots(company_id, policy_key, platform, jurisdiction, active, effective_at_epoch DESC);

CREATE TABLE IF NOT EXISTS compliance_checks (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  policy_snapshot_id uuid REFERENCES policy_snapshots(id),
  surface text NOT NULL CHECK (surface IN ('CONTENT','AFFILIATE','LIVE','ADVERTISING','COPYRIGHT','PRODUCT_ELIGIBILITY','CLAIMS')),
  policy_key text NOT NULL,
  policy_snapshot_key text NOT NULL,
  input_hash text NOT NULL,
  decision text NOT NULL CHECK (decision IN ('ALLOWED','REVIEW','BLOCKED','UNKNOWN')),
  reason text NOT NULL CHECK (reason IN ('POLICY_UNAVAILABLE','MISSING_POLICY_EVIDENCE','MISSING_DISCLOSURE','PROHIBITED_PRODUCT','UNSUPPORTED_PRODUCT','UNVERIFIED_CLAIM','FAKE_ENGAGEMENT','SIMULCAST','MISSING_RIGHTS_EVIDENCE','HUMAN_REVIEW_REQUIRED','ALLOWED_BY_POLICY')),
  evidence_ref text NOT NULL,
  requires_human boolean NOT NULL DEFAULT false,
  checked_at_epoch bigint NOT NULL CHECK (checked_at_epoch > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, policy_key, policy_snapshot_key, input_hash)
);

CREATE INDEX IF NOT EXISTS idx_compliance_checks_company_time
  ON compliance_checks(company_id, checked_at_epoch DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_checks_company_decision
  ON compliance_checks(company_id, decision, checked_at_epoch DESC);
