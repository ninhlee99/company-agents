CREATE TABLE IF NOT EXISTS financial_forecasts (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  currency char(3) NOT NULL,
  horizon_months integer NOT NULL CHECK (horizon_months BETWEEN 1 AND 60),
  methodology text NOT NULL,
  status text NOT NULL CHECK (status IN ('DRAFT','ACTIVE','ARCHIVED')),
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS financial_forecast_periods (
  id uuid PRIMARY KEY,
  forecast_id uuid NOT NULL REFERENCES financial_forecasts(id) ON DELETE CASCADE,
  period_start_epoch bigint NOT NULL,
  revenue_minor numeric(39,0) NOT NULL DEFAULT 0,
  operating_inflow_minor numeric(39,0) NOT NULL DEFAULT 0,
  operating_outflow_minor numeric(39,0) NOT NULL DEFAULT 0,
  capex_minor numeric(39,0) NOT NULL DEFAULT 0,
  financing_inflow_minor numeric(39,0) NOT NULL DEFAULT 0,
  financing_outflow_minor numeric(39,0) NOT NULL DEFAULT 0,
  notes text,
  UNIQUE(forecast_id, period_start_epoch),
  CHECK (revenue_minor >= 0 AND operating_inflow_minor >= 0 AND operating_outflow_minor >= 0
         AND capex_minor >= 0 AND financing_inflow_minor >= 0 AND financing_outflow_minor >= 0)
);

CREATE TABLE IF NOT EXISTS cashflow_observations (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  period_start_epoch bigint NOT NULL,
  currency char(3) NOT NULL,
  inflow_minor numeric(39,0) NOT NULL DEFAULT 0,
  outflow_minor numeric(39,0) NOT NULL DEFAULT 0,
  closing_cash_minor numeric(39,0) NOT NULL,
  source text NOT NULL,
  evidence_hash text NOT NULL,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_forecast_periods_forecast_period
  ON financial_forecast_periods(forecast_id, period_start_epoch);
CREATE INDEX IF NOT EXISTS idx_cashflow_observations_company_period
  ON cashflow_observations(company_id, period_start_epoch DESC);
