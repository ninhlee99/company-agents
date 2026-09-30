# Implementation Plan

## Phase 1 — Economic Kernel
**Status: implemented + operational acceptance automation**
- Deterministic economic core, immutable double-entry ledger, company state/runway guards and checked arithmetic.
- PostgreSQL ledger, budget, idempotency, audit and outbox persistence.
- Transactional cycle execution, scheduler leases, replay-safe run tokens and durable outbox delivery.
- Automated CI and weekly PostgreSQL recovery drill are now part of the release gates.

- Unified event architecture foundation: canonical event names, versioned envelope, correlation/causation metadata and idempotent outbox persistence are implemented. The terminal experiment producer now emits `EXPERIMENT_COMPLETED` transactionally; remaining producers migrate incrementally.

## Phase 2 — Simulator
**Status: implemented baseline**
- Seeded/replayable company world.
- Revenue/cost/cash model, affiliate and sponsor signals.
- Agent Runtime + Governor and bankruptcy/cash-shock scenarios.
- Business-unit, portfolio and payroll economics are represented.

**Next acceptance work:** calibrate simulator distributions from real observed outcomes and validate sensitivity across target deployment scenarios. Multi-run statistical evaluation with deterministic seed sets and descriptive 95% intervals is implemented.

## Phase 3 — Agent Harness
**Status: implemented safety baseline**
- Typed Agent contract, multi-provider LLM gateway, bounded runtime, timeouts, fail-closed errors.
- Capability matrix, context envelope, proposal validation, Tool Registry, durable memory and rate limits.
- Durable scheduler/journal/outbox and operational CI/recovery gates.

**Next acceptance work:** extend restart smoke into target-topology load/chaos runs and distributed tracing acceptance.

## Phase 4 — Agents
**Status: implemented and safety-tested**
- Governor, CEO, CFO, COO, Growth, Content, Recruiter, Analyst and Experiment.
- Every operating Agent has deterministic baseline policy, bounded model suggestions, typed validation and Governor review.

## Phase 5 — Media Factory
**Status: production pipeline foundation implemented**
Research → planning → scripting → production → QA → publishing → analytics → attribution.
Durable media jobs, sandboxed FFmpeg, FFprobe QA and publishing approval contracts are implemented. Real platform adapters remain credential/account dependent.

## Phase 6 — Monetization
**Status: affiliate + commercial sales lifecycle implemented**
- Product discovery/ranking, coupon validation, Awin/TikTok Shop adapters.
- Click/conversion attribution, provider verification, receivable recognition and payout accounting.
- Customer CRM identity/lifecycle, service proposals, sponsorships, invoices and a governed outbound email path using durable outbox delivery.
- Invoice issuance/payment now posts deterministic Accounts Receivable/Cash/Revenue ledger entries.
- Payment execution boundary: create intent → explicit approval → non-production mock execution with durable evidence/outbox only; simulated execution does not move funds or mark invoices paid.

**Next product work:** broader verified revenue ingestion and external delivery evidence. Proposal/sponsorship status workflows and currency-scoped delivery reporting are implemented.

- Competitor intelligence foundation: evidence-backed observations and owned-content coverage can be scored deterministically into content whitespace gaps.

- Creator intelligence foundation: evidence-backed creator profiles can be matched to fresh, in-stock product economics through deterministic specialty/category and performance scoring.
- Ads decision engine foundation: evidence-backed campaigns are ranked on incremental contribution margin, lift, confidence and feedback speed; execution/spend remains separately gated.
## FP&A / Cash Flow

- Forecast-vs-actual variance report: implemented baseline via `/api/fpa/forecast-variance` and `/fpa/variance`, company-scoped and evidence-aware.

## Phase 7 — Human Organization
**Status: core economics implemented**
- Hiring governance, employees, payroll obligations, accrual/payment accounting and business-unit economics.

**Next product work:** richer contractor/performance workflows.

- Control-plane RBAC baseline: named config-driven principals (`admin`/`operator`/`read-only`) are supported; SSO/directory identity and durable user lifecycle remain outside the current scope.

- Tenant binding baseline: named control-plane principals are explicitly company-scoped; legacy shared-token compatibility and deployment-scoped tenancy remain.

## Phase 8 — Controlled Autonomy
**Status: controlled runtime implemented**
Proposal → Governor → bounded execution, durable audit, replay-safe scheduling and explicit external-side-effect gates are active.

**Next acceptance work:** broaden restart/chaos to the target production topology and complete real account acceptance tests.

## Phase 9 — Portfolio Company
**Status: foundations implemented; autonomous portfolio control gated**
Business-unit and portfolio economics exist. Automated capital allocation and shutdown/reinvestment require additional production evidence.

## Phase 10 — Self-sustaining operation
**Status: gated**
Recurring autonomous operation is technically scaffolded, but unsupervised external side effects remain blocked until the production acceptance standard is satisfied.

## Release rule
Do not advance a phase until its previous phase passes tests and operational acceptance criteria.
