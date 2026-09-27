ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS reconciliation_status text;

ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS reconciliation_variance_minor numeric(39,0) NOT NULL DEFAULT 0;

UPDATE affiliate_conversions
   SET reconciliation_status = 'VERIFIED'
 WHERE reconciliation_status IS NULL;

ALTER TABLE affiliate_conversions
  ALTER COLUMN reconciliation_status SET DEFAULT 'PARTIAL';

ALTER TABLE affiliate_conversions
  DROP CONSTRAINT IF EXISTS affiliate_conversions_reconciliation_status_check;

ALTER TABLE affiliate_conversions
  ADD CONSTRAINT affiliate_conversions_reconciliation_status_check
  CHECK (reconciliation_status IN ('VERIFIED','PARTIAL','REJECTED'));
