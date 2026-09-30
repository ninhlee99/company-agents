CREATE OR REPLACE FUNCTION reject_learning_entry_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'learning_entries is append-only';
END;
$$;

DROP TRIGGER IF EXISTS learning_entries_no_update ON learning_entries;
DROP TRIGGER IF EXISTS learning_entries_no_delete ON learning_entries;

CREATE TRIGGER learning_entries_no_update
BEFORE UPDATE ON learning_entries
FOR EACH ROW EXECUTE FUNCTION reject_learning_entry_mutation();

CREATE TRIGGER learning_entries_no_delete
BEFORE DELETE ON learning_entries
FOR EACH ROW EXECUTE FUNCTION reject_learning_entry_mutation();
