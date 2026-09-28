ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS reconciliation_status text;

ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS reconciliation_variance_minor numeric(39,0) NOT NULL DEFAULT 0;

UPDATE affiliate_conversions c
   SET reconciliation_variance_minor =
         c.commission_minor -
         COALESCE((
           SELECT SUM(a.attributed_commission_minor)
             FROM affiliate_attributions a
            WHERE a.company_id = c.company_id
              AND a.conversion_id = c.conversion_id
         ), 0),
       reconciliation_status =
         CASE
           WHEN c.commission_minor =
                COALESCE((
                  SELECT SUM(a.attributed_commission_minor)
                    FROM affiliate_attributions a
                   WHERE a.company_id = c.company_id
                     AND a.conversion_id = c.conversion_id
                ), 0)
           THEN 'VERIFIED'
           ELSE 'PARTIAL'
         END
 WHERE c.reconciliation_status IS NULL;

ALTER TABLE affiliate_conversions
  ALTER COLUMN reconciliation_status SET DEFAULT 'PARTIAL';

ALTER TABLE affiliate_conversions
  ALTER COLUMN reconciliation_status SET NOT NULL;

ALTER TABLE affiliate_conversions
  DROP CONSTRAINT IF EXISTS affiliate_conversions_reconciliation_status_check;

ALTER TABLE affiliate_conversions
  ADD CONSTRAINT affiliate_conversions_reconciliation_status_check
  CHECK (reconciliation_status IN ('VERIFIED','PARTIAL','REJECTED'));
