ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS currency text;

UPDATE affiliate_conversions AS conversion
   SET currency = upper(company.base_currency)
  FROM companies AS company
 WHERE conversion.company_id = company.id
   AND conversion.currency IS NULL;

UPDATE affiliate_conversions
   SET currency = upper(currency)
 WHERE currency IS NOT NULL;

ALTER TABLE affiliate_conversions
  ALTER COLUMN currency SET NOT NULL;

ALTER TABLE affiliate_conversions
  DROP CONSTRAINT IF EXISTS affiliate_conversions_currency_format;

ALTER TABLE affiliate_conversions
  ADD CONSTRAINT affiliate_conversions_currency_format
  CHECK (currency ~ '^[A-Z]{3}$');

CREATE INDEX IF NOT EXISTS idx_affiliate_conversions_currency
  ON affiliate_conversions(company_id, currency);
