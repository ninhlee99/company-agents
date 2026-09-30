-- Company isolation hardening for company-owned relationships.
-- Existing data is validated explicitly; a mismatch fails startup rather than being silently repaired.

CREATE UNIQUE INDEX IF NOT EXISTS uq_customers_company_id ON customers(company_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_vendors_company_id ON vendors(company_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_purchase_requests_company_id ON purchase_requests(company_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_invoices_company_id ON invoices(company_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_payment_execution_intents_company_id ON payment_execution_intents(company_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_creators_company_id ON creators(company_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_tiktok_live_sessions_company_id ON tiktok_live_sessions(company_id, id);

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'service_proposals_customer_company_fk') THEN
    ALTER TABLE service_proposals ADD CONSTRAINT service_proposals_customer_company_fk FOREIGN KEY (company_id, customer_id) REFERENCES customers(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE service_proposals VALIDATE CONSTRAINT service_proposals_customer_company_fk;
ALTER TABLE service_proposals DROP CONSTRAINT IF EXISTS service_proposals_customer_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sponsorships_customer_company_fk') THEN
    ALTER TABLE sponsorships ADD CONSTRAINT sponsorships_customer_company_fk FOREIGN KEY (company_id, customer_id) REFERENCES customers(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE sponsorships VALIDATE CONSTRAINT sponsorships_customer_company_fk;
ALTER TABLE sponsorships DROP CONSTRAINT IF EXISTS sponsorships_customer_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'invoices_customer_company_fk') THEN
    ALTER TABLE invoices ADD CONSTRAINT invoices_customer_company_fk FOREIGN KEY (company_id, customer_id) REFERENCES customers(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE invoices VALIDATE CONSTRAINT invoices_customer_company_fk;
ALTER TABLE invoices DROP CONSTRAINT IF EXISTS invoices_customer_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'customer_success_customer_company_fk') THEN
    ALTER TABLE customer_success_tasks ADD CONSTRAINT customer_success_customer_company_fk FOREIGN KEY (company_id, customer_id) REFERENCES customers(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE customer_success_tasks VALIDATE CONSTRAINT customer_success_customer_company_fk;
ALTER TABLE customer_success_tasks DROP CONSTRAINT IF EXISTS customer_success_tasks_customer_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'purchase_requests_vendor_company_fk') THEN
    ALTER TABLE purchase_requests ADD CONSTRAINT purchase_requests_vendor_company_fk FOREIGN KEY (company_id, vendor_id) REFERENCES vendors(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE purchase_requests VALIDATE CONSTRAINT purchase_requests_vendor_company_fk;
ALTER TABLE purchase_requests DROP CONSTRAINT IF EXISTS purchase_requests_vendor_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'vendor_deliveries_request_company_fk') THEN
    ALTER TABLE vendor_deliveries ADD CONSTRAINT vendor_deliveries_request_company_fk FOREIGN KEY (company_id, purchase_request_id) REFERENCES purchase_requests(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE vendor_deliveries VALIDATE CONSTRAINT vendor_deliveries_request_company_fk;
ALTER TABLE vendor_deliveries DROP CONSTRAINT IF EXISTS vendor_deliveries_purchase_request_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'payment_reconciliation_invoice_company_fk') THEN
    ALTER TABLE payment_reconciliation_evidence ADD CONSTRAINT payment_reconciliation_invoice_company_fk FOREIGN KEY (company_id, invoice_id) REFERENCES invoices(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE payment_reconciliation_evidence VALIDATE CONSTRAINT payment_reconciliation_invoice_company_fk;
ALTER TABLE payment_reconciliation_evidence DROP CONSTRAINT IF EXISTS payment_reconciliation_evidence_invoice_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'payment_execution_invoice_company_fk') THEN
    ALTER TABLE payment_execution_intents ADD CONSTRAINT payment_execution_invoice_company_fk FOREIGN KEY (company_id, invoice_id) REFERENCES invoices(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE payment_execution_intents VALIDATE CONSTRAINT payment_execution_invoice_company_fk;
ALTER TABLE payment_execution_intents DROP CONSTRAINT IF EXISTS payment_execution_intents_invoice_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'payment_execution_evidence_intent_company_fk') THEN
    ALTER TABLE payment_execution_evidence ADD CONSTRAINT payment_execution_evidence_intent_company_fk FOREIGN KEY (company_id, intent_id) REFERENCES payment_execution_intents(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE payment_execution_evidence VALIDATE CONSTRAINT payment_execution_evidence_intent_company_fk;
ALTER TABLE payment_execution_evidence DROP CONSTRAINT IF EXISTS payment_execution_evidence_intent_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'tiktok_live_events_session_company_fk') THEN
    ALTER TABLE tiktok_live_events ADD CONSTRAINT tiktok_live_events_session_company_fk FOREIGN KEY (company_id, session_id) REFERENCES tiktok_live_sessions(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE tiktok_live_events VALIDATE CONSTRAINT tiktok_live_events_session_company_fk;
ALTER TABLE tiktok_live_events DROP CONSTRAINT IF EXISTS tiktok_live_events_session_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'tiktok_live_gift_statements_session_company_fk') THEN
    ALTER TABLE tiktok_live_gift_statements ADD CONSTRAINT tiktok_live_gift_statements_session_company_fk FOREIGN KEY (company_id, session_id) REFERENCES tiktok_live_sessions(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE tiktok_live_gift_statements VALIDATE CONSTRAINT tiktok_live_gift_statements_session_company_fk;
ALTER TABLE tiktok_live_gift_statements DROP CONSTRAINT IF EXISTS tiktok_live_gift_statements_session_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'content_assets_creator_company_fk') THEN
    ALTER TABLE content_assets ADD CONSTRAINT content_assets_creator_company_fk FOREIGN KEY (company_id, creator_id) REFERENCES creators(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE content_assets VALIDATE CONSTRAINT content_assets_creator_company_fk;
ALTER TABLE content_assets DROP CONSTRAINT IF EXISTS content_assets_creator_id_fkey;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'tasks_creator_company_fk') THEN
    ALTER TABLE tasks ADD CONSTRAINT tasks_creator_company_fk FOREIGN KEY (company_id, creator_id) REFERENCES creators(company_id, id) NOT VALID;
  END IF;
END $$;
ALTER TABLE tasks VALIDATE CONSTRAINT tasks_creator_company_fk;
ALTER TABLE tasks DROP CONSTRAINT IF EXISTS tasks_creator_id_fkey;