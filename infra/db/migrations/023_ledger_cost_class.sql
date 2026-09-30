ALTER TABLE ledger_accounts
  ADD COLUMN IF NOT EXISTS cost_class text;

UPDATE ledger_accounts
   SET cost_class = 'UNCLASSIFIED'
 WHERE cost_class IS NULL;

ALTER TABLE ledger_accounts
  ALTER COLUMN cost_class SET DEFAULT 'UNCLASSIFIED';

ALTER TABLE ledger_accounts
  ALTER COLUMN cost_class SET NOT NULL;

ALTER TABLE ledger_accounts
  DROP CONSTRAINT IF EXISTS ledger_accounts_cost_class_check;

ALTER TABLE ledger_accounts
  ADD CONSTRAINT ledger_accounts_cost_class_check
  CHECK (cost_class IN ('VARIABLE','FIXED','UNCLASSIFIED'));

CREATE INDEX IF NOT EXISTS idx_ledger_accounts_company_cost_class
  ON ledger_accounts(company_id, account_type, cost_class);
