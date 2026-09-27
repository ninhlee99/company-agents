CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE affiliate_clicks
  ADD COLUMN IF NOT EXISTS payload_hash text;
ALTER TABLE affiliate_conversions
  ADD COLUMN IF NOT EXISTS payload_hash text;
ALTER TABLE affiliate_payouts
  ADD COLUMN IF NOT EXISTS payload_hash text;
ALTER TABLE media_jobs
  ADD COLUMN IF NOT EXISTS payload_hash text;

UPDATE affiliate_clicks
   SET payload_hash = encode(digest(
       concat_ws(E'\\x1f', company_id::text, click_id, product_id, advertiser_id, content_id, occurred_at, source),
       'sha256'), 'hex')
 WHERE payload_hash IS NULL;

UPDATE affiliate_conversions
   SET payload_hash = encode(digest(
       concat_ws(E'\\x1f', company_id::text, conversion_id, coalesce(click_id,''), order_id, product_id, advertiser_id, occurred_at, currency,
                 order_value_minor::text, commission_minor::text, refunded_minor::text, cancelled::text, source),
       'sha256'), 'hex')
 WHERE payload_hash IS NULL;

UPDATE affiliate_payouts
   SET payload_hash = encode(digest(
       concat_ws(E'\\x1f', company_id::text, payout_id, occurred_at, amount_minor::text, currency, source),
       'sha256'), 'hex')
 WHERE payload_hash IS NULL;

UPDATE media_jobs
   SET payload_hash = encode(digest(
       concat_ws(E'\\x1f', company_id::text, id::text, input_path, output_path, format, width::text, height::text, fps::text,
                 max_duration_seconds::text, normalize_audio::text),
       'sha256'), 'hex')
 WHERE payload_hash IS NULL;

CREATE INDEX IF NOT EXISTS idx_affiliate_conversion_payload_hash
  ON affiliate_conversions(company_id, idempotency_key, payload_hash);
CREATE INDEX IF NOT EXISTS idx_affiliate_payout_payload_hash
  ON affiliate_payouts(company_id, idempotency_key, payload_hash);
CREATE INDEX IF NOT EXISTS idx_media_job_payload_hash
  ON media_jobs(company_id, idempotency_key, payload_hash);