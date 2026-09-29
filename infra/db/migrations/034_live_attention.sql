CREATE UNIQUE INDEX IF NOT EXISTS idx_tiktok_live_sessions_company_id
  ON tiktok_live_sessions(company_id, id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_tiktok_live_events_company_id
  ON tiktok_live_events(company_id, id);

CREATE TABLE IF NOT EXISTS live_attention_decisions (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  session_id uuid NOT NULL,
  event_id text NOT NULL,
  action text NOT NULL CHECK (action IN ('RESPOND','DEFER','IGNORE','ESCALATE')),
  reason text NOT NULL CHECK (reason IN ('PURCHASE_INTENT','OBJECTION','GIFT','PK_MOMENT','HIGH_ENGAGEMENT','SAFETY_ESCALATION','COOLDOWN','RATE_LIMITED','LOW_SIGNAL')),
  priority smallint NOT NULL CHECK (priority BETWEEN 0 AND 100),
  decided_at_epoch bigint NOT NULL CHECK (decided_at_epoch > 0),
  requires_human boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (company_id, session_id) REFERENCES tiktok_live_sessions(company_id, id),
  UNIQUE(company_id, session_id, event_id)
);

CREATE INDEX IF NOT EXISTS idx_live_attention_company_session_time
  ON live_attention_decisions(company_id, session_id, decided_at_epoch DESC);

CREATE INDEX IF NOT EXISTS idx_live_attention_response_window
  ON live_attention_decisions(company_id, session_id, action, decided_at_epoch DESC);
