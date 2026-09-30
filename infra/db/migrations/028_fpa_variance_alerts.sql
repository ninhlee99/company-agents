CREATE TABLE IF NOT EXISTS financial_alerts (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  alert_type text NOT NULL CHECK (alert_type IN ('CASH_RUNWAY','FORECAST_VARIANCE')),
  severity text NOT NULL CHECK (severity IN ('INFO','WARNING','CRITICAL')),
  period_start_epoch bigint,
  metric_name text NOT NULL,
  actual_minor numeric(39,0),
  forecast_minor numeric(39,0),
  variance_minor numeric(39,0),
  threshold_minor numeric(39,0),
  message text NOT NULL,
  status text NOT NULL CHECK (status IN ('OPEN','ACKNOWLEDGED','RESOLVED')),
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  resolved_at timestamptz,
  UNIQUE(company_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_financial_alerts_company_status ON financial_alerts(company_id,status,created_at DESC);
