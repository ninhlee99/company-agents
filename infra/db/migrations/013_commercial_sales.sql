CREATE TABLE IF NOT EXISTS service_proposals (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  customer_id uuid NOT NULL REFERENCES customers(id),
  title text NOT NULL,
  currency varchar(3) NOT NULL,
  total_minor numeric(39,0) NOT NULL CHECK (total_minor >= 0),
  status text NOT NULL CHECK (status IN ('DRAFT','SENT','ACCEPTED','REJECTED','EXPIRED')),
  valid_until_epoch bigint NOT NULL,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS sponsorships (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  customer_id uuid NOT NULL REFERENCES customers(id),
  title text NOT NULL,
  currency varchar(3) NOT NULL,
  committed_minor numeric(39,0) NOT NULL CHECK (committed_minor >= 0),
  delivered_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (delivered_minor >= 0 AND delivered_minor <= committed_minor),
  status text NOT NULL CHECK (status IN ('PROSPECT','CONTRACTED','DELIVERING','COMPLETED','CANCELLED')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS invoices (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  customer_id uuid NOT NULL REFERENCES customers(id),
  currency varchar(3) NOT NULL,
  subtotal_minor numeric(39,0) NOT NULL CHECK (subtotal_minor >= 0),
  paid_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (paid_minor >= 0 AND paid_minor <= subtotal_minor),
  status text NOT NULL CHECK (status IN ('DRAFT','ISSUED','PARTIALLY_PAID','PAID','VOID')),
  due_epoch bigint NOT NULL,
  idempotency_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS invoice_lines (
  id bigserial PRIMARY KEY,
  invoice_id uuid NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
  description text NOT NULL,
  quantity integer NOT NULL CHECK (quantity > 0),
  unit_price_minor numeric(39,0) NOT NULL CHECK (unit_price_minor >= 0)
);

CREATE TABLE IF NOT EXISTS invoice_payments (
  id uuid PRIMARY KEY,
  invoice_id uuid NOT NULL REFERENCES invoices(id),
  amount_minor numeric(39,0) NOT NULL CHECK (amount_minor > 0),
  external_ref text,
  occurred_at_epoch bigint NOT NULL,
  UNIQUE(invoice_id, external_ref)
);

CREATE INDEX IF NOT EXISTS idx_service_proposals_company_status ON service_proposals(company_id, status);
CREATE INDEX IF NOT EXISTS idx_sponsorships_company_status ON sponsorships(company_id, status);
CREATE INDEX IF NOT EXISTS idx_invoices_company_status ON invoices(company_id, status);
CREATE INDEX IF NOT EXISTS idx_invoice_payments_invoice ON invoice_payments(invoice_id);
