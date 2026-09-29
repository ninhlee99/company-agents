# Veridara AI — Capability Matrix

**Core platform:** Autonomous Company OS  
**Positioning:** AI-native operating company with controlled autonomy

## Strict production review

This document intentionally separates implemented behavior from contracts, simulations and environment-dependent integrations.

| Capability | Current status | What is actually true |
|---|---|---|
| Agent decision cycle | Implemented | Agents call the configured model, produce typed proposals, and pass deterministic Governor/execution checks. |
| Agent memory | Implemented | Durable per-agent memory exists with bounded size/retention semantics. |
| Agent model-call rate limiting | Implemented | Persistent rate windows are checked before model calls. |
| Governance / least privilege | Implemented | Role/tool matrix, proposal validation and re-governance are enforced deterministically. |
| Economic core | Implemented | Checked arithmetic, company state/runway guards and deterministic execution are present. |
| Double-entry ledger | Implemented | Ledger transactions/entries are immutable and database validation enforces balance/company/currency invariants. |
| Scheduler | Implemented | Database leases and replay-safe run tokens are used for recurring cycles. |
| Outbox | Implemented | Durable events have leases, bounded retries and signed HTTPS webhook delivery. |
| Web research / web-session relay | Environment-gated | Browser/web relay protocols and workers exist, but usable capability depends on configured relay/browser credentials. |
| Affiliate product discovery | Environment-gated | Mock, Awin and TikTok Shop adapters exist; real data requires operator credentials/contracts. |
| Affiliate attribution/reconciliation | Implemented + environment-gated | Click/conversion state and provider verification/accounting are durable; real provider verification requires provider data. |
| Media production | Implemented | Media jobs run through isolated FFmpeg/FFprobe QA in the media worker. |
| Publishing approval contract | Implemented | Publish intent, approval, lease, completion and revocation are durable and guarded. |
| TikTok publishing adapter | Implemented + webhook reconciliation, environment-gated | Approved TikTok intents can be executed through the Content Posting API adapter; publish IDs are persisted, signed TikTok webhooks are verified with replay protection, terminal outcomes reconcile idempotently, and status polling remains available as fallback. Real use requires valid TikTok authorization, app approval/audit and operator configuration. |
| External email/message sending | NOT achieved | SendExternalMessage is a declared capability, but there is no production message provider executor. |
| External payment execution | NOT achieved | The system can model invoices and record externally evidenced payments, but does not initiate bank/card/Stripe settlement. |
| Commercial proposals | Implemented baseline | Durable proposal creation exists with idempotency. Status workflow endpoints are still limited. |
| Sponsorship management | Implemented baseline | Contracted/delivered value is bounded, but CRM delivery/reporting workflow is incomplete. |
| Invoicing | Implemented | Invoice creation, issuance and payment lifecycle are durable and idempotent. |
| Invoice accounting | Implemented | Issuance posts AR → revenue; payment posts cash → AR inside the same transaction. |
| Customer CRM | Implemented baseline | Idempotent customer creation and listing with lifecycle/status metadata are available. |
| HR / payroll economics | Implemented baseline | Employees, payroll obligations and accounting primitives exist; external payroll execution is not integrated. |
| Business-unit economics | Implemented baseline | Units and portfolio metrics exist; automatic capital allocation is still gated. |
| Portfolio autonomy | NOT achieved | The repository does not yet provide evidence-backed autonomous reinvest/close decisions across real business units. |
| Human approval / material side effects | Implemented | Material publishing and other sensitive actions are designed to remain explicitly gated. |
| Control-plane authentication | Implemented baseline | Bearer token authentication is required by default for non-health endpoints, with constant-time comparison. |
| Multi-user identity / RBAC / SSO | NOT achieved | Authentication is a shared control-plane token, not an operator identity system. |
| Multi-tenant SaaS isolation | NOT achieved | The runtime is company-scoped by deployment configuration, not a full user/tenant authorization model. |
| Observability | Implemented baseline | Health/readiness and Prometheus-style counters exist. Distributed tracing/load/chaos acceptance is still environment-dependent. |
| Disaster recovery | Implemented baseline | Backup/restore drill automation exists; production-scale recovery evidence is still environment-dependent. |
| Model evaluation / routing | NOT achieved | There is no production benchmark matrix, complexity router or hardware-aware model selection loop yet. |
| Autonomous hiring/payroll execution | NOT achieved | Economic primitives and proposals exist; real external hiring/payroll actions remain gated. |
| Autonomous company operation with no human | NOT achieved | The architecture is a controlled autonomy foundation. Real external credentials, platform adapters and production acceptance are still required. |

## Hard conclusion

The repository is substantially beyond a toy multi-agent demo: the strongest implemented areas are deterministic governance, economic state, durable persistence, recovery-oriented workers, affiliate accounting, media processing and now commercial receivables/CRM.

It should not be described as an AI company that can independently operate every external business function yet. The largest remaining capability gap is the side-effect layer: real platform publishing, communications, payment rails, identity/RBAC and evidence-backed portfolio control.

## Next implementation priority

1. Extend the TikTok adapter from authenticated execution to OAuth/token lifecycle management and durable background polling workers; webhook reconciliation is now implemented.
2. Build an authenticated outbound messaging adapter behind approval + outbox.
3. Add proposal/sponsorship lifecycle transitions and delivery evidence.
4. Add real payment-provider reconciliation before any payment initiation capability.
5. Add operator identity/RBAC and tenant isolation before exposing the control plane as SaaS.
6. Add benchmark/evaluation and shadow-mode evidence before increasing autonomy.


### Recent operating-control improvements
- Basic monochrome control-plane dashboard: **Implemented**
- Revenue planning target visibility: **Implemented** (planning metric only)
- Workforce/payroll/business-unit operational visibility: **Implemented**
- Verified external revenue generation: **Environment-gated / not guaranteed**
- Autonomous external publishing, messaging and payment initiation: **Publishing adapter + reconciliation implemented for TikTok; messaging and payment initiation not achieved**


### Commercial lifecycle hardening

The commercial control plane now exposes a read-only pipeline view plus deterministic proposal and sponsorship transitions. Sponsorship delivery is bounded by the contracted value and emits durable outbox events. These APIs do not execute external contracts or payment settlement; those remain provider/reconciliation boundaries.


## FP&A / Budget Controls

- Durable company budgets: implemented
- Atomic spend tracking with idempotency: implemented
- Currency and limit enforcement: implemented
- Append-only audit evidence for budget spend: implemented
- Forecast vs actual: not yet implemented
- Cash-flow forecasting / runway planning: not yet implemented


## Forecast / Cash Flow

- Versioned financial forecast plans: implemented
- Monthly/period cash-flow assumptions: implemented
- Forecast net cash-flow summary: implemented
- Idempotent cash-flow observations with evidence: implemented
- Forecast-vs-actual variance analysis: next
- Automated cash runway / liquidity alerts: next


## Recurring Revenue

- Subscription plans with monthly/yearly cadence: implemented
- Customer subscriptions and billing periods: implemented
- Idempotent recurring billing period creation: implemented
- Invoice issuance from a billing period: implemented
- Compliance reference required before invoice issuance: implemented
- Existing payment reconciliation evidence attaches to the generated invoice: supported by invoice_id linkage
- Automated payment execution/settlement: not implemented
- Automated renewal/dunning: next
