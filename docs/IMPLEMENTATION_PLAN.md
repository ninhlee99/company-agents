# Implementation Plan

## Phase 1 — Economic Kernel
**Status: core + durable execution implemented; production acceptance pending operational drills**

Implemented:
- Rust deterministic economic core
- immutable double-entry ledger validation
- company state, runway and budget guards
- checked/saturating arithmetic and overflow rejection
- PostgreSQL ledger, budgets, idempotency, outbox and audit schema
- deferred database-side ledger completeness/balance checks
- transactional, idempotent ledger posting
- transactional decision-cycle persistence
- persisted state revisioning
- economic delta → ledger reconciliation
- restart recovery for stale control-plane records
- bankruptcy/liquidation spending guards
- invariant and adversarial tests

Remaining:
- automated backup/restore verification
- live load/concurrency test against PostgreSQL
- production secret manager integration
- operational alerting/SLOs

## Phase 2 — Simulator
**Status: deterministic simulator + execution integration + Monte Carlo implemented**

Implemented:
- seeded/replayable company world
- daily revenue/cost/cash model
- content, affiliate and sponsor signals
- Agent Runtime + Governor + Execution Engine inside simulation
- bankruptcy/cash-shock scenarios
- deterministic replay
- Monte Carlo summary across bounded trials

Remaining:
- richer creator/business-unit economics
- payroll/contract liabilities
- calibrated real-world distributions
- attribution-based revenue curves
- long-horizon portfolio simulation

## Phase 3 — Agent Harness
**Status: governed execution implemented; durable operations mostly implemented**

Implemented:
- typed Agent contract
- local Ollama + Gemini/OpenAI-compatible model adapters
- bounded concurrent Agent Runtime
- model timeout and response limits
- fail-closed model error handling
- per-Agent action capability matrix
- context-aware action envelope
- proposal validation
- execution-bound least-privilege Tool Registry
- deterministic Execution Engine with re-govern before each action
- persistent decision journal
- durable idempotency
- transactional outbox
- restart recovery of stale cycles
- idempotent API cycle execution

Implemented since initial plan:
- per-Agent persistent rate limiting in PostgreSQL
- durable Agent memory store with bounded, untrusted replay context
- company-scoped scheduler claiming with row locking / skip-locked semantics
- operational readiness and Prometheus metrics endpoints
- durable affiliate payout ingestion with ledger idempotency
- affiliate click/order ingestion and deterministic attribution rebuild API

Remaining:
- per-Agent persistent spending/budget ledgers
- full distributed lease tokens/heartbeats for long-running jobs
- OpenTelemetry tracing/export
- production secret manager integration
- operational alerting/SLOs

## Phase 4 — Agents
**Status: operating Agents implemented and adversarially tested**

Implemented:
- Governor
- CEO
- CFO
- COO
- Growth
- Content
- Recruiter
- Analyst
- Experiment
- deterministic baseline policies
- bounded LLM reasoning layer
- 720 capability/status firewall combinations
- randomized snapshot tests
- multi-cycle stress tests
- model outage/timeout containment

## Phase 5 — Media Factory
**Status: production foundation implemented; external publishing/analytics integrations remain gated**

Implemented:
- media job model and FFmpeg-based processing/QA
- durable media worker with bounded execution
- social publishing adapter with approval/HMAC gate

Remaining:
- platform metrics ingestion
- closed-loop analytics → experiment attribution automation

Target:
Research → planning → scripting → production → media QA → publishing → analytics → attribution.

## Phase 6 — Monetization
**Status: affiliate intelligence + attribution implemented; broader monetization pending**

Implemented:
- provider-neutral affiliate product model
- category/keyword/currency/commission/price/stock/quality filters
- coupon validation and expiry handling
- product quality scoring with confidence
- commercial scoring and expected contribution estimate
- GTIN/product deduplication
- deterministic ranking
- provider outage isolation when at least one provider succeeds
- Awin CSV product-feed adapter
- Awin promotion/coupon feed adapter
- TikTok Shop open-collaboration search adapter with pagination
- durable affiliate research records
- Company OS affiliate search APIs

External authorization remains required. Awin/TikTok data is never invented when the provider has not supplied it.

Implemented since initial plan:
- verified affiliate payout ingestion
- click/conversion event ingestion
- deterministic attribution rebuild API

Remaining:
- sponsorship CRM
- service proposal/invoicing
- verified accrued-revenue recognition against receivables (payout cash path is implemented)

## Phase 7 — Human Organization
**Status: planned**

Target:
employees, payroll abstractions, contractors, hiring workflow, performance review, termination.

## Phase 8 — Controlled Autonomy
**Status: partially prepared, not enabled for material external side effects**

Current:
- shadow/advisory LLM reasoning
- Governor escalation for material/irreversible actions
- internal bounded execution

Not yet enabled:
- autonomous publishing
- autonomous external messaging
- autonomous payments
- autonomous hiring
- unrestricted capital allocation

## Phase 9 — Portfolio Company
**Status: planned**

Target:
multiple creators/business units, independent P&L, capital allocation, reinvestment and shutdown/restructuring.

## Phase 10 — Self-sustaining operation
**Status: planned**

Target:
continuous economic loop with verified revenue attribution, payroll, reinvestment, hiring, portfolio management and orderly liquidation/bankruptcy.

## Release rule
Do not advance a phase until its predecessor passes unit → contract → scenario → adversarial → economic → security → load → recovery → chaos gates.


## Engineering rule

All generated LLM text remains untrusted data. Money, permissions, approvals, state transitions, external side effects, and durable accounting are owned by deterministic Rust code and transactional persistence.
