CREATE OR REPLACE FUNCTION validate_ledger_transaction_from_header() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
  entry_count bigint;
  debit_total numeric(39,0);
  credit_total numeric(39,0);
BEGIN
  SELECT COUNT(*), COALESCE(SUM(debit_minor),0), COALESCE(SUM(credit_minor),0)
    INTO entry_count, debit_total, credit_total
    FROM ledger_entries
   WHERE transaction_id = NEW.id;

  IF entry_count < 2 THEN
    RAISE EXCEPTION 'ledger transaction requires at least two entries';
  END IF;

  IF debit_total <> credit_total THEN
    RAISE EXCEPTION 'ledger transaction is unbalanced';
  END IF;

  RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS ledger_transaction_header_complete ON ledger_transactions;
CREATE CONSTRAINT TRIGGER ledger_transaction_header_complete
AFTER INSERT ON ledger_transactions
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_ledger_transaction_from_header();
