CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS companies (
  id uuid PRIMARY KEY,
  name text NOT NULL,
  status text NOT NULL CHECK (status IN ('ACTIVE','GROWTH','WARNING','COST_CONTROL','DISTRESS','EMERGENCY','LIQUIDATION','BANKRUPT')),
  base_currency char(3) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ledger_accounts (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  code text NOT NULL,
  name text NOT NULL,
  account_type text NOT NULL CHECK (account_type IN ('ASSET','LIABILITY','EQUITY','REVENUE','EXPENSE')),
  currency char(3) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS ledger_transactions (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  description text NOT NULL,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS ledger_entries (
  id bigserial PRIMARY KEY,
  transaction_id uuid NOT NULL REFERENCES ledger_transactions(id),
  account_id uuid NOT NULL REFERENCES ledger_accounts(id),
  debit_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (debit_minor >= 0),
  credit_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (credit_minor >= 0),
  currency char(3) NOT NULL,
  CHECK ((debit_minor > 0 AND credit_minor = 0) OR (credit_minor > 0 AND debit_minor = 0))
);

CREATE INDEX IF NOT EXISTS idx_ledger_entries_transaction ON ledger_entries(transaction_id);
CREATE INDEX IF NOT EXISTS idx_ledger_entries_account ON ledger_entries(account_id);

CREATE TABLE IF NOT EXISTS budgets (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  name text NOT NULL,
  currency char(3) NOT NULL,
  limit_minor numeric(39,0) NOT NULL CHECK (limit_minor >= 0),
  spent_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (spent_minor >= 0),
  active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  CHECK (spent_minor <= limit_minor)
);

CREATE TABLE IF NOT EXISTS idempotency_keys (
  company_id uuid NOT NULL REFERENCES companies(id),
  key text NOT NULL,
  command_type text NOT NULL,
  status text NOT NULL CHECK (status IN ('PROCESSING','SUCCEEDED','FAILED')),
  response_json jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (company_id, key)
);

CREATE TABLE IF NOT EXISTS outbox_events (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  event_type text NOT NULL,
  aggregate_id text,
  idempotency_key text NOT NULL,
  schema_version int NOT NULL DEFAULT 1,
  payload jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  published_at timestamptz,
  UNIQUE(company_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS audit_log (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  actor_type text NOT NULL,
  actor_id text NOT NULL,
  action text NOT NULL,
  resource_type text NOT NULL,
  resource_id text,
  decision text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION reject_ledger_entry_update() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'ledger_entries are immutable'; END; $$;
CREATE OR REPLACE FUNCTION reject_ledger_entry_delete() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'ledger_entries are immutable'; END; $$;
DROP TRIGGER IF EXISTS ledger_entries_no_update ON ledger_entries;
DROP TRIGGER IF EXISTS ledger_entries_no_delete ON ledger_entries;
CREATE TRIGGER ledger_entries_no_update BEFORE UPDATE ON ledger_entries FOR EACH ROW EXECUTE FUNCTION reject_ledger_entry_update();
CREATE TRIGGER ledger_entries_no_delete BEFORE DELETE ON ledger_entries FOR EACH ROW EXECUTE FUNCTION reject_ledger_entry_delete();


CREATE OR REPLACE FUNCTION validate_ledger_transaction() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
  entry_count bigint;
  debit_total numeric(39,0);
  credit_total numeric(39,0);
  currency_count bigint;
BEGIN
  SELECT COUNT(*), COALESCE(SUM(debit_minor),0), COALESCE(SUM(credit_minor),0), COUNT(DISTINCT currency)
    INTO entry_count, debit_total, credit_total, currency_count
    FROM ledger_entries
   WHERE transaction_id = NEW.transaction_id;

  IF entry_count < 2 THEN
    RAISE EXCEPTION 'ledger transaction requires at least two entries';
  END IF;

  IF debit_total <> credit_total THEN
    RAISE EXCEPTION 'ledger transaction is unbalanced';
  END IF;

  IF currency_count <> 1 THEN
    RAISE EXCEPTION 'ledger transaction has mixed currencies';
  END IF;

  IF EXISTS (
    SELECT 1
      FROM ledger_entries e
      JOIN ledger_accounts a ON a.id = e.account_id
      JOIN ledger_transactions t ON t.id = e.transaction_id
     WHERE e.transaction_id = NEW.transaction_id
  IF EXISTS (
    SELECT 1
      FROM ledger_entries e
      JOIN ledger_accounts a ON a.id = e.account_id
      JOIN ledger_transactions t ON t.id = e.transaction_id
     WHERE e.transaction_id = NEW.transaction_id
       AND a.company_id <> t.company_id
  ) THEN
    RAISE EXCEPTION 'ledger entry/account company mismatch';
  END IF;

  IF EXISTS (
    SELECT 1
      FROM ledger_entries e
      JOIN ledger_accounts a ON a.id = e.account_id
     WHERE e.transaction_id = NEW.transaction_id
       AND e.currency <> a.currency
  ) THEN
    RAISE EXCEPTION 'ledger entry/account currency mismatch';
  END IF;

  RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS ledger_transaction_balanced ON ledger_entries;
CREATE CONSTRAINT TRIGGER ledger_transaction_balanced
AFTER INSERT ON ledger_entries
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_ledger_transaction();

CREATE OR REPLACE FUNCTION reject_ledger_transaction_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'ledger_transactions are immutable';
END;
$$;

DROP TRIGGER IF EXISTS ledger_transactions_no_update ON ledger_transactions;
DROP TRIGGER IF EXISTS ledger_transactions_no_delete ON ledger_transactions;
CREATE TRIGGER ledger_transactions_no_update BEFORE UPDATE ON ledger_transactions FOR EACH ROW EXECUTE FUNCTION reject_ledger_transaction_mutation();
CREATE TRIGGER ledger_transactions_no_delete BEFORE DELETE ON ledger_transactions FOR EACH ROW EXECUTE FUNCTION reject_ledger_transaction_mutation();

CREATE OR REPLACE FUNCTION reject_audit_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'audit_log is append-only';
END;
$$;

DROP TRIGGER IF EXISTS audit_log_no_update ON audit_log;
DROP TRIGGER IF EXISTS audit_log_no_delete ON audit_log;
CREATE TRIGGER audit_log_no_update BEFORE UPDATE ON audit_log FOR EACH ROW EXECUTE FUNCTION reject_audit_mutation();
CREATE TRIGGER audit_log_no_delete BEFORE DELETE ON audit_log FOR EACH ROW EXECUTE FUNCTION reject_audit_mutation();


CREATE TABLE IF NOT EXISTS company_state_snapshots (
  company_id uuid PRIMARY KEY REFERENCES companies(id),
  state jsonb NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS agent_runs (
  id bigserial PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  agent_name text NOT NULL,
  payload jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_agent_runs_company_time
  ON agent_runs(company_id, created_at DESC);
