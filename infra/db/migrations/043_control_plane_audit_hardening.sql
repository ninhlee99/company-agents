CREATE OR REPLACE FUNCTION reject_control_plane_audit_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'control_plane_audit_log is append-only';
END;
$$;

DROP TRIGGER IF EXISTS control_plane_audit_log_no_update ON control_plane_audit_log;
DROP TRIGGER IF EXISTS control_plane_audit_log_no_delete ON control_plane_audit_log;

CREATE TRIGGER control_plane_audit_log_no_update
BEFORE UPDATE ON control_plane_audit_log
FOR EACH ROW EXECUTE FUNCTION reject_control_plane_audit_mutation();

CREATE TRIGGER control_plane_audit_log_no_delete
BEFORE DELETE ON control_plane_audit_log
FOR EACH ROW EXECUTE FUNCTION reject_control_plane_audit_mutation();

CREATE INDEX IF NOT EXISTS idx_control_plane_audit_company_outcome_time
  ON control_plane_audit_log(company_id, outcome, created_at DESC);
