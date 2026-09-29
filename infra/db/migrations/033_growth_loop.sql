CREATE TABLE IF NOT EXISTS growth_trends (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  trend_key text NOT NULL,
  topic text NOT NULL,
  source text NOT NULL,
  evidence_ref text NOT NULL,
  observed_at_epoch bigint NOT NULL CHECK (observed_at_epoch > 0),
  velocity_bps integer NOT NULL CHECK (velocity_bps BETWEEN 0 AND 10000),
  audience_fit_bps integer NOT NULL CHECK (audience_fit_bps BETWEEN 0 AND 10000),
  product_fit_bps integer NOT NULL CHECK (product_fit_bps BETWEEN 0 AND 10000),
  contentability_bps integer NOT NULL CHECK (contentability_bps BETWEEN 0 AND 10000),
  competition_bps integer NOT NULL CHECK (competition_bps BETWEEN 0 AND 10000),
  confidence_bps integer NOT NULL CHECK (confidence_bps BETWEEN 0 AND 10000),
  product_ref text,
  offer_ref text,
  content_format text NOT NULL CHECK (content_format IN ('SHORT_VIDEO','LIVE_SEGMENT','STORY','CAROUSEL')),
  max_budget_minor numeric(39,0) NOT NULL CHECK (max_budget_minor >= 0),
  max_loss_minor numeric(39,0) NOT NULL CHECK (max_loss_minor >= max_budget_minor),
  max_duration_seconds bigint NOT NULL CHECK (max_duration_seconds BETWEEN 1 AND 86400),
  success_metric text NOT NULL CHECK (success_metric IN ('VIEWS','CLICK_THROUGH_RATE','CONVERSION_RATE','COMMISSION','CONTRIBUTION_MARGIN')),
  success_threshold_bps integer NOT NULL CHECK (success_threshold_bps BETWEEN 0 AND 10000),
  policy_evidence_ref text NOT NULL,
  score_bps integer NOT NULL CHECK (score_bps BETWEEN 0 AND 10000),
  decision text NOT NULL CHECK (decision IN ('PURSUE','MONITOR','REJECT')),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, trend_key)
);

CREATE INDEX IF NOT EXISTS idx_growth_trends_company_decision
  ON growth_trends(company_id, decision, score_bps DESC, created_at DESC);

CREATE TABLE IF NOT EXISTS growth_opportunities (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  trend_id uuid NOT NULL REFERENCES growth_trends(id),
  opportunity_key text NOT NULL,
  title text NOT NULL,
  score_bps integer NOT NULL CHECK (score_bps BETWEEN 0 AND 10000),
  confidence_bps integer NOT NULL CHECK (confidence_bps BETWEEN 0 AND 10000),
  policy_evidence_ref text NOT NULL,
  plan_json jsonb NOT NULL,
  status text NOT NULL CHECK (status IN ('READY','CONTENT_CREATED')),
  content_item_id uuid REFERENCES content_items(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, opportunity_key)
);

CREATE INDEX IF NOT EXISTS idx_growth_opportunities_company_ready
  ON growth_opportunities(company_id, status, score_bps DESC, created_at DESC);
