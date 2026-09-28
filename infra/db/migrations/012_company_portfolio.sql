CREATE TABLE IF NOT EXISTS company_portfolio_snapshots (
  company_id uuid PRIMARY KEY REFERENCES companies(id),
  schema_version int NOT NULL DEFAULT 1 CHECK (schema_version >= 1),
  state jsonb NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_company_portfolio_updated
  ON company_portfolio_snapshots(updated_at DESC);
