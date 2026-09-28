ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS provider_verification_status text NOT NULL DEFAULT 'UNVERIFIED';

ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS provider_verified_commission_minor numeric(39,0) NOT NULL DEFAULT 0
  CHECK (provider_verified_commission_minor >= 0);

ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS provider_verified_at timestamptz;

ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS provider_verification_source text;

ALTER TABLE affiliate_conversions
  DROP CONSTRAINT IF EXISTS affiliate_conversions_provider_verification_status_check;

ALTER TABLE affiliate_conversions
  ADD CONSTRAINT affiliate_conversions_provider_verification_status_check
  CHECK (
    provider_verification_status
      IN ('UNVERIFIED','REPORTED','APPROVED','PAID','REJECTED')
  );

ALTER TABLE affiliate_revenue_recognition
  DROP CONSTRAINT IF EXISTS affiliate_revenue_recognition_status_check;

ALTER TABLE affiliate_revenue_recognition
  ADD CONSTRAINT affiliate_revenue_recognition_status_check
  CHECK (status IN ('PENDING','RECOGNIZED','REJECTED'));

CREATE INDEX IF NOT EXISTS idx_affiliate_conversions_provider_verification
  ON affiliate_conversions(company_id, provider_verification_status);
