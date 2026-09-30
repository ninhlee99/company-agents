ALTER TABLE financial_alerts
  ADD COLUMN IF NOT EXISTS actual_value numeric(39,0),
  ADD COLUMN IF NOT EXISTS forecast_value numeric(39,0),
  ADD COLUMN IF NOT EXISTS variance_value numeric(39,0),
  ADD COLUMN IF NOT EXISTS threshold_value numeric(39,0);
