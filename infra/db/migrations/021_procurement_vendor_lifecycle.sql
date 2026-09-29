CREATE TABLE IF NOT EXISTS vendors (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  legal_name text NOT NULL,
  contact_email text,
  currency char(3) NOT NULL,
  tax_ref text,
  status text NOT NULL CHECK (status IN ('PROSPECT','ACTIVE','SUSPENDED','CLOSED')),
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_vendors_company_status
  ON vendors(company_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS purchase_requests (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  vendor_id uuid NOT NULL REFERENCES vendors(id),
  title text NOT NULL,
  currency char(3) NOT NULL,
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor > 0),
  requester text NOT NULL,
  status text NOT NULL CHECK (status IN ('DRAFT','PENDING_APPROVAL','APPROVED','REJECTED','ORDERED','RECEIVED','CANCELLED')),
  approval_reference text,
  approved_by text,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_purchase_requests_company_status
  ON purchase_requests(company_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS vendor_deliveries (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  purchase_request_id uuid NOT NULL REFERENCES purchase_requests(id),
  external_ref text,
  received_at_epoch bigint NOT NULL,
  evidence_hash text NOT NULL,
  status text NOT NULL CHECK (status IN ('ACCEPTED','REJECTED')),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vendor_deliveries_request
  ON vendor_deliveries(company_id, purchase_request_id, created_at DESC);
