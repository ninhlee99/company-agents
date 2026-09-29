# Commercial Sales Lifecycle

The Company OS now has durable primitives for non-affiliate monetization.

## Service proposals

`POST /api/commercial/proposals` creates an idempotent proposal in `DRAFT` state. The proposal records the customer, currency, quoted amount, validity deadline and business idempotency key.

## Sponsorships

`POST /api/commercial/sponsorships` creates a sponsorship opportunity with committed and delivered value. Delivered value cannot exceed the contracted amount.

## Invoices

`POST /api/commercial/invoices` creates an idempotent invoice with checked line totals.
`POST /api/commercial/invoices/issue` transitions a draft invoice to `ISSUED`.
`POST /api/commercial/invoice-payments` records an idempotent payment and advances the invoice through `PARTIALLY_PAID` to `PAID`.

Payments are bounded by the outstanding invoice balance and cannot be recorded against `DRAFT` or `VOID` invoices.

## Accounting

Issuing an invoice posts Accounts Receivable → Invoice Revenue. Recording a payment posts Cash → Accounts Receivable. Both postings use deterministic idempotency keys inside the same PostgreSQL transaction as the commercial state transition.

The system still does not claim that an external payment reference is proof of bank settlement; provider/bank reconciliation remains an explicit acceptance boundary.

## Customer CRM

`POST /api/customers` creates an idempotent customer record and `GET /api/customers` lists the current company customer set. Customer records support lead/active/inactive/churned lifecycle states, external references and operator notes.

All amounts use integer minor units and checked arithmetic.
