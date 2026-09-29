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

## Accounting boundary

These endpoints model the commercial lifecycle and receivable state. They do not invent bank settlement: an external payment reference is accepted as evidence, while cash/ledger settlement remains a separate governed accounting operation.

All amounts use integer minor units and checked arithmetic.
