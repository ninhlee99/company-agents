CREATE TABLE IF NOT EXISTS affiliate_clicks (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  click_id text NOT NULL,
  content_id uuid NOT NULL,
  creator_id uuid NOT NULL,
  product_id text NOT NULL,
  occurred_at_epoch bigint NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, click_id)
);

CREATE TABLE IF NOT EXISTS affiliate_order_events (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  order_id text NOT NULL,
  click_id text,
  product_id text NOT NULL,
  gross_sales_minor numeric(39,0) NOT NULL,
  commission_minor numeric(39,0) NOT NULL,
  status text NOT NULL,
  occurred_at_epoch bigint NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, order_id, status)
);

CREATE TABLE IF NOT EXISTS affiliate_attributions (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  order_id text NOT NULL,
  content_id uuid,
  creator_id uuid,
  product_id text NOT NULL,
  gross_sales_minor numeric(39,0) NOT NULL,
  commission_minor numeric(39,0) NOT NULL,
  attribution_confidence_bps integer NOT NULL,
  attribution_reason text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, order_id)
);

CREATE INDEX IF NOT EXISTS idx_affiliate_clicks_company_product_time
  ON affiliate_clicks(company_id, product_id, occurred_at_epoch);
CREATE INDEX IF NOT EXISTS idx_affiliate_orders_company_time
  ON affiliate_order_events(company_id, occurred_at_epoch);
