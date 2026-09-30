CREATE TABLE IF NOT EXISTS content_items (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  hypothesis text NOT NULL,
  audience text NOT NULL,
  format text NOT NULL CHECK (format IN ('SHORT_VIDEO','LIVE_SEGMENT','STORY','CAROUSEL')),
  product_ref text,
  offer_ref text,
  disclosure_required boolean NOT NULL,
  expected_cost_minor numeric(39,0) NOT NULL CHECK (expected_cost_minor >= 0),
  max_loss_minor numeric(39,0) NOT NULL CHECK (max_loss_minor >= expected_cost_minor),
  max_duration_seconds bigint NOT NULL CHECK (max_duration_seconds BETWEEN 1 AND 86400),
  success_metric text NOT NULL CHECK (success_metric IN ('VIEWS','CLICK_THROUGH_RATE','CONVERSION_RATE','COMMISSION','CONTRIBUTION_MARGIN')),
  success_threshold_bps integer NOT NULL CHECK (success_threshold_bps BETWEEN 0 AND 10000),
  variant_key text NOT NULL,
  hook text NOT NULL,
  first_frame text NOT NULL,
  emotion text NOT NULL,
  pacing text NOT NULL,
  scene_count integer NOT NULL CHECK (scene_count BETWEEN 1 AND 60),
  text_density text NOT NULL,
  voice_speed text NOT NULL,
  product_placement text NOT NULL,
  cta text NOT NULL,
  comment_trigger text NOT NULL,
  music_style text NOT NULL,
  visual_style text NOT NULL,
  status text NOT NULL CHECK (status IN ('DRAFT','APPROVED','RENDERED','PUBLISHED','MEASURED','PAUSED','KILLED')),
  decision text CHECK (decision IN ('SCALE','ITERATE','PAUSE','KILL')),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_content_items_company_variant
  ON content_items(company_id, variant_key);

CREATE INDEX IF NOT EXISTS idx_content_items_company_status
  ON content_items(company_id, status, created_at DESC);
