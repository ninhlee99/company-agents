-- Durable recurring CEO exam reports.
CREATE TABLE IF NOT EXISTS ceo_exams (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  exam_key text NOT NULL,
  period_start_epoch bigint NOT NULL,
  period_end_epoch bigint NOT NULL,
  overall_status text NOT NULL CHECK (overall_status IN ('PASS','REVIEW','UNKNOWN')),
  overall_score_bps integer CHECK (overall_score_bps IS NULL OR (overall_score_bps >= 0 AND overall_score_bps <= 10000)),
  report jsonb NOT NULL,
  evidence_hash char(64) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, exam_key),
  CHECK (period_start_epoch > 0),
  CHECK (period_end_epoch > period_start_epoch)
);

CREATE INDEX IF NOT EXISTS idx_ceo_exams_company_period
  ON ceo_exams(company_id, period_end_epoch DESC, created_at DESC);
