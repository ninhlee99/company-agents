CREATE UNIQUE INDEX IF NOT EXISTS uq_decision_journal_company_id
  ON decision_journal(company_id, id);

CREATE TABLE IF NOT EXISTS agent_outcome_evidence (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  decision_journal_id bigint NOT NULL,
  evidence_ref text NOT NULL CHECK (char_length(trim(evidence_ref)) BETWEEN 1 AND 1024),
  observed_revenue_delta_minor numeric(39,0) NOT NULL,
  observed_contribution_margin_delta_minor numeric(39,0) NOT NULL,
  observed_at_epoch bigint NOT NULL CHECK (observed_at_epoch > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (company_id, decision_journal_id)
    REFERENCES decision_journal(company_id, id),
  UNIQUE(company_id, decision_journal_id)
);

CREATE INDEX IF NOT EXISTS idx_agent_outcome_evidence_company_time
  ON agent_outcome_evidence(company_id, observed_at_epoch DESC);

CREATE INDEX IF NOT EXISTS idx_agent_outcome_evidence_company_ref
  ON agent_outcome_evidence(company_id, evidence_ref);

CREATE OR REPLACE FUNCTION reject_agent_outcome_evidence_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'agent_outcome_evidence is append-only';
END;
$$;

DROP TRIGGER IF EXISTS agent_outcome_evidence_no_update ON agent_outcome_evidence;
DROP TRIGGER IF EXISTS agent_outcome_evidence_no_delete ON agent_outcome_evidence;
CREATE TRIGGER agent_outcome_evidence_no_update
BEFORE UPDATE ON agent_outcome_evidence
FOR EACH ROW EXECUTE FUNCTION reject_agent_outcome_evidence_mutation();
CREATE TRIGGER agent_outcome_evidence_no_delete
BEFORE DELETE ON agent_outcome_evidence
FOR EACH ROW EXECUTE FUNCTION reject_agent_outcome_evidence_mutation();
