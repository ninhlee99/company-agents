# Implementation Plan

## Phase 1 — Economic Kernel
**Status: implemented + operational acceptance automation**
- Deterministic economic core, immutable double-entry ledger, company state/runway guards and checked arithmetic.
- PostgreSQL ledger, budget, idempotency, audit and outbox persistence.
- Transactional cycle execution, scheduler leases, replay-safe run tokens and durable outbox delivery.
- Automated CI and weekly PostgreSQL recovery drill are now part of the release gates.

## Phase 2 — Simulator
**Status: implemented baseline**
- Seeded/replayable company world.
- Revenue/cost/cash model, affiliate and sponsor signals.
- Agent Runtime + Governor and bankruptcy/cash-shock scenarios.
- Business-unit, portfolio and payroll economics are represented.

**Next acceptance work:** calibrate distributions from real observations and add multi-run statistical evaluation with confidence intervals.

- Unified event architecture foundation: canonical event names, versioned envelope, correlation/causation metadata and idempotent outbox persistence are implemented; producer migration remains incremental.
## Phase 3 — Agent Harness
**Status: implemented safety baseline**
- Typed Agent contract, multi-provider LLM gateway, bounded runtime, timeouts, fail-closed errors.
- Capability matrix, context envelope, proposal validation, Tool Registry, durable memory and rate limits.
- Durable scheduler/journal/outbox and operational CI/recovery gates.

**Next acceptance work:** run distributed observability, load and chaos tests on the target deployment topology.

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
- Customer CRM identity/lifecycle, service proposals, sponsorships and invoices.
- Invoice issuance/payment now posts deterministic Accounts Receivable/Cash/Revenue ledger entries.

**Next product work:** proposal/sponsorship status workflows, richer delivery/reporting and broader verified revenue ingestion.

## Phase 7 — Human Organization
**Status: core economics implemented**
- Hiring governance, employees, payroll obligations, accrual/payment accounting and business-unit economics.

**Next product work:** richer contractor/performance workflows.

## Phase 8 — Controlled Autonomy
**Status: controlled runtime implemented**
Proposal → Governor → bounded execution, durable audit, replay-safe scheduling and explicit external-side-effect gates are active.

**Next acceptance work:** production-like restart/chaos validation and real account acceptance tests.

## Phase 9 — Portfolio Company
**Status: foundations implemented; autonomous portfolio control gated**
Business-unit and portfolio economics exist. Automated capital allocation and shutdown/reinvestment require additional production evidence.

## Phase 10 — Self-sustaining operation
**Status: gated**
Recurring autonomous operation is technically scaffolded, but unsupervised external side effects remain blocked until the production acceptance standard is satisfied.

## Release rule
Do not advance a phase until its previous phase passes tests and operational acceptance criteria.
