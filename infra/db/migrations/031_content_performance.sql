CREATE TABLE IF NOT EXISTS content_observations (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  content_id uuid NOT NULL REFERENCES content_items(id) ON DELETE CASCADE,
  observation_key text NOT NULL,
  source text NOT NULL,
  evidence_hash text NOT NULL,
  observed_at_epoch bigint NOT NULL CHECK (observed_at_epoch > 0),
  sample_count bigint NOT NULL CHECK (sample_count > 0),
  spend_minor numeric(39,0) NOT NULL CHECK (spend_minor >= 0),
  metric_bps integer NOT NULL CHECK (metric_bps BETWEEN 0 AND 10000),
  views bigint NOT NULL CHECK (views >= 0),
  clicks bigint NOT NULL CHECK (clicks >= 0 AND clicks <= views),
  conversions bigint NOT NULL CHECK (conversions >= 0 AND conversions <= clicks),
  commission_minor numeric(39,0) NOT NULL CHECK (commission_minor >= 0),
  contribution_margin_minor numeric(39,0) NOT NULL,
  decision text NOT NULL CHECK (decision IN ('SCALE','ITERATE','PAUSE','KILL')),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, observation_key)
);

CREATE INDEX IF NOT EXISTS idx_content_observations_company_content_time
  ON content_observations(company_id, content_id, observed_at_epoch DESC);
