ALTER TABLE content_items
  ADD COLUMN IF NOT EXISTS status_evidence_ref text;

CREATE INDEX IF NOT EXISTS idx_content_items_company_evidence
  ON content_items(company_id, status, status_evidence_ref)
  WHERE status_evidence_ref IS NOT NULL;
