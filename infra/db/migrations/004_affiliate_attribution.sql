CREATE TABLE IF NOT EXISTS affiliate_clicks (
  company_id uuid NOT NULL REFERENCES companies(id),
  click_id text NOT NULL,
  product_id text NOT NULL,
  advertiser_id text NOT NULL,
  content_id text NOT NULL,
  occurred_at text NOT NULL,
  source text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, click_id)
);

CREATE INDEX IF NOT EXISTS idx_affiliate_clicks_product_time
  ON affiliate_clicks(company_id, product_id, occurred_at DESC);

CREATE TABLE IF NOT EXISTS affiliate_conversions (
  company_id uuid NOT NULL REFERENCES companies(id),
  conversion_id text NOT NULL,
  click_id text,
  order_id text NOT NULL,
  product_id text NOT NULL,
  advertiser_id text NOT NULL,
  occurred_at text NOT NULL,
  order_value_minor numeric(39,0) NOT NULL CHECK (order_value_minor >= 0),
  commission_minor numeric(39,0) NOT NULL CHECK (commission_minor >= 0),
  refunded_minor numeric(39,0) NOT NULL CHECK (refunded_minor >= 0 AND refunded_minor <= order_value_minor),
  cancelled boolean NOT NULL DEFAULT false,
  source text NOT NULL,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, conversion_id),
  UNIQUE (company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_affiliate_conversions_order
  ON affiliate_conversions(company_id, order_id);

CREATE TABLE IF NOT EXISTS affiliate_attributions (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  conversion_id text NOT NULL,
  click_id text NOT NULL,
  product_id text NOT NULL,
  content_id text NOT NULL,
  attributed_order_value_minor numeric(39,0) NOT NULL CHECK (attributed_order_value_minor >= 0),
  attributed_commission_minor numeric(39,0) NOT NULL CHECK (attributed_commission_minor >= 0),
  confidence_bps int NOT NULL CHECK (confidence_bps BETWEEN 0 AND 10000),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, conversion_id, click_id)
);

CREATE INDEX IF NOT EXISTS idx_affiliate_attributions_content
  ON affiliate_attributions(company_id, content_id, created_at DESC);
