CREATE TABLE IF NOT EXISTS revenue_graph_edges (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  edge_key text NOT NULL,
  from_type text NOT NULL CHECK (from_type IN (
    'CASH','COMMISSION','ORDER','PRODUCT','CAMPAIGN','CONTENT','HOOK',
    'CREATOR','AUDIENCE','TRAFFIC','EXPERIMENT','DECISION','TREND'
  )),
  from_ref text NOT NULL,
  relation text NOT NULL,
  to_type text NOT NULL CHECK (to_type IN (
    'CASH','COMMISSION','ORDER','PRODUCT','CAMPAIGN','CONTENT','HOOK',
    'CREATOR','AUDIENCE','TRAFFIC','EXPERIMENT','DECISION','TREND'
  )),
  to_ref text NOT NULL,
  value_minor numeric(39,0),
  currency char(3),
  confidence_bps integer NOT NULL CHECK (confidence_bps BETWEEN 0 AND 10000),
  evidence_ref text NOT NULL,
  source text NOT NULL,
  observed_at_epoch bigint NOT NULL CHECK (observed_at_epoch > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, edge_key),
  CHECK ((value_minor IS NULL AND currency IS NULL)
      OR (value_minor IS NOT NULL AND value_minor >= 0 AND currency ~ '^[A-Z]{3}$')),
  CHECK (NOT (from_type = to_type AND from_ref = to_ref))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_revenue_graph_company_id
  ON revenue_graph_edges(company_id, id);

CREATE INDEX IF NOT EXISTS idx_revenue_graph_forward
  ON revenue_graph_edges(company_id, from_type, from_ref, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_revenue_graph_reverse
  ON revenue_graph_edges(company_id, to_type, to_ref, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_revenue_graph_evidence
  ON revenue_graph_edges(company_id, evidence_ref, created_at DESC);

CREATE OR REPLACE FUNCTION reject_revenue_graph_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'revenue_graph_edges are immutable';
END;
$$;

DROP TRIGGER IF EXISTS revenue_graph_edges_no_update ON revenue_graph_edges;
DROP TRIGGER IF EXISTS revenue_graph_edges_no_delete ON revenue_graph_edges;

CREATE TRIGGER revenue_graph_edges_no_update
BEFORE UPDATE ON revenue_graph_edges
FOR EACH ROW EXECUTE FUNCTION reject_revenue_graph_mutation();

CREATE TRIGGER revenue_graph_edges_no_delete
BEFORE DELETE ON revenue_graph_edges
FOR EACH ROW EXECUTE FUNCTION reject_revenue_graph_mutation();
