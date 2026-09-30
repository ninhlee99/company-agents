CREATE TABLE IF NOT EXISTS autonomy_control_state (
  company_id uuid PRIMARY KEY REFERENCES companies(id),
  emergency_stop_enabled boolean NOT NULL DEFAULT false,
  emergency_stop_reason text,
  emergency_stop_actor text NOT NULL DEFAULT 'system-default',
  emergency_stop_changed_at_epoch bigint NOT NULL DEFAULT 1 CHECK (emergency_stop_changed_at_epoch > 0),
  content_publish_daily numeric(39,0) NOT NULL DEFAULT 10 CHECK (content_publish_daily >= 0),
  ads_spend_daily_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (ads_spend_daily_minor >= 0),
  live_minutes_daily numeric(39,0) NOT NULL DEFAULT 60 CHECK (live_minutes_daily >= 0),
  outbound_messages_daily numeric(39,0) NOT NULL DEFAULT 100 CHECK (outbound_messages_daily >= 0),
  autonomous_capital_daily_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (autonomous_capital_daily_minor >= 0),
  updated_at_epoch bigint NOT NULL DEFAULT 1 CHECK (updated_at_epoch > 0)
);

CREATE TABLE IF NOT EXISTS autonomy_budget_usage (
  company_id uuid NOT NULL REFERENCES companies(id),
  budget_kind text NOT NULL CHECK (
    budget_kind IN (
      'CONTENT_PUBLISH',
      'ADS_SPEND',
      'LIVE_MINUTES',
      'OUTBOUND_MESSAGES',
      'AUTONOMOUS_CAPITAL'
    )
  ),
  period_start_epoch bigint NOT NULL CHECK (period_start_epoch > 0),
  used numeric(39,0) NOT NULL DEFAULT 0 CHECK (used >= 0),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, budget_kind, period_start_epoch)
);

CREATE TABLE IF NOT EXISTS autonomy_budget_consumptions (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  budget_kind text NOT NULL CHECK (
    budget_kind IN (
      'CONTENT_PUBLISH',
      'ADS_SPEND',
      'LIVE_MINUTES',
      'OUTBOUND_MESSAGES',
      'AUTONOMOUS_CAPITAL'
    )
  ),
  period_start_epoch bigint NOT NULL CHECK (period_start_epoch > 0),
  amount numeric(39,0) NOT NULL CHECK (amount > 0),
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, budget_kind, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_autonomy_budget_consumptions_company_period
  ON autonomy_budget_consumptions(company_id, period_start_epoch, budget_kind);

CREATE OR REPLACE FUNCTION reject_autonomy_budget_consumption_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'autonomy_budget_consumptions are immutable';
END;
$$;

DROP TRIGGER IF EXISTS autonomy_budget_consumptions_no_update ON autonomy_budget_consumptions;
DROP TRIGGER IF EXISTS autonomy_budget_consumptions_no_delete ON autonomy_budget_consumptions;
CREATE TRIGGER autonomy_budget_consumptions_no_update
BEFORE UPDATE ON autonomy_budget_consumptions FOR EACH ROW
EXECUTE FUNCTION reject_autonomy_budget_consumption_mutation();
CREATE TRIGGER autonomy_budget_consumptions_no_delete
BEFORE DELETE ON autonomy_budget_consumptions FOR EACH ROW
EXECUTE FUNCTION reject_autonomy_budget_consumption_mutation();
