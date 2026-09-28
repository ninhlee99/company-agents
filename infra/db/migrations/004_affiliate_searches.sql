CREATE TABLE IF NOT EXISTS affiliate_searches (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  query_json jsonb NOT NULL,
  result_json jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_affiliate_searches_company_time
  ON affiliate_searches(company_id, created_at DESC);
