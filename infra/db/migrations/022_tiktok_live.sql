CREATE TABLE IF NOT EXISTS tiktok_live_sessions (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  room_id text,
  title text NOT NULL,
  mode text NOT NULL,
  started_at_epoch bigint NOT NULL,
  approved_for_external_publish boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, id)
);

CREATE INDEX IF NOT EXISTS idx_tiktok_live_sessions_company_started
  ON tiktok_live_sessions(company_id, started_at_epoch DESC);

CREATE TABLE IF NOT EXISTS tiktok_live_events (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  session_id uuid NOT NULL REFERENCES tiktok_live_sessions(id),
  event_id text NOT NULL,
  room_id text NOT NULL,
  kind text NOT NULL,
  user_id text,
  display_name text,
  event_text text,
  gift_id text,
  gift_name text,
  gift_quantity bigint NOT NULL DEFAULT 0,
  gift_value_minor numeric(38,0) NOT NULL DEFAULT 0,
  currency text NOT NULL,
  pk_score bigint,
  occurred_at_epoch bigint NOT NULL,
  received_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, event_id)
);

CREATE INDEX IF NOT EXISTS idx_tiktok_live_events_session_time
  ON tiktok_live_events(company_id, session_id, occurred_at_epoch DESC);

CREATE TABLE IF NOT EXISTS tiktok_live_gift_statements (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  session_id uuid NOT NULL REFERENCES tiktok_live_sessions(id),
  statement_id text NOT NULL,
  gift_count numeric(38,0) NOT NULL,
  gross_value_minor numeric(38,0) NOT NULL,
  currency text NOT NULL,
  matched boolean NOT NULL,
  count_delta numeric(38,0) NOT NULL,
  value_delta_minor numeric(38,0) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, statement_id)
);
