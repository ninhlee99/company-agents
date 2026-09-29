CREATE TABLE IF NOT EXISTS outbound_messages (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  channel text NOT NULL CHECK (channel IN ('EMAIL')),
  recipient text NOT NULL,
  subject text NOT NULL,
  html_body text NOT NULL,
  idempotency_key text NOT NULL,
  approval_reference text,
  approved_by text,
  consent_basis text,
  unsubscribe_url text,
  status text NOT NULL CHECK (status IN ('PENDING_APPROVAL','APPROVED','PROCESSING','SENT','FAILED','CANCELLED')),
  provider text,
  provider_reference text,
  attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  last_error text,
  outbox_event_id bigint REFERENCES outbox_events(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_outbound_messages_company_status
  ON outbound_messages(company_id, status, created_at);

CREATE UNIQUE INDEX IF NOT EXISTS idx_outbound_messages_provider_reference
  ON outbound_messages(company_id, provider, provider_reference)
  WHERE provider_reference IS NOT NULL;
