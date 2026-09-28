# Implementation Plan

## Phase 1 — Economic Kernel
**Status: implemented baseline + durable execution layer**
- Rust deterministic economic core
- immutable double-entry ledger validation
- company state, runway and budget guards
- checked arithmetic / overflow protection
- PostgreSQL schema with ledger, budget, idempotency, outbox and audit tables
- database-side ledger balance/dependency checks
- bankruptcy/liquidation spending guards
- invariant tests

Implemented in this baseline:
- transactional repository/service layer
- atomic Company cycle + execution + journal/outbox persistence
- persisted state transitions
- idempotent cycle and ledger commands
- lease-based scheduler recovery
- durable outbox dispatcher with leased delivery, bounded retries and HMAC-signed webhook delivery

Still required for full production acceptance:
- automated backup/restore drill
- disaster recovery rehearsal

## Phase 2 — Simulator
**Status: deterministic simulator with shared production execution path implemented**
- seeded/replayable company world
- daily revenue/cost/cash model
- content, affiliate and sponsor signals
- Agent Runtime + Governor inside simulation
- bankruptcy/cash shock scenarios
- deterministic simulation tests

Remaining before Phase 2 acceptance:
- richer creator/business-unit economics
- payroll/contract liabilities
- traffic/campaign distributions calibrated from real observations
- multi-run statistical evaluation with confidence intervals

## Phase 3 — Agent Harness
**Status: safety-focused runtime + execution-bound tooling + durable scheduler/journal/outbox dispatch implemented**
- typed Agent contract
- local Ollama + Gemini/OpenAI-compatible model adapters
- bounded concurrent Agent Runtime
- model timeout
- fail-closed model error handling
- per-Agent action capability matrix
- context-aware action envelope
- proposal validation
- least-privilege Tool Registry
- stress/adversarial/randomized tests

Remaining before Phase 3 acceptance:
- distributed tracing across workers
- secret-rotation operations

## Phase 4 — Agents
**Status: operating Agents implemented and safety-tested**
- Governor policy engine
- CEO
- CFO
- COO
- Growth
- Content
- Recruiter
- Analyst
- Experiment

Every operating Agent has:
- role prompt
- deterministic baseline policy
- bounded model suggestion layer
- typed proposal validation
- Governor review
- model-outage and timeout fail-closed behavior
- scenario and adversarial tests

## Phase 5 — Media Factory
**Status: production pipeline foundation implemented**
Research → planning → scripting → production → QA → publishing → analytics → attribution remains the operating flow. Durable media jobs, sandboxed FFmpeg execution, FFprobe inspection and post-render QA are now implemented; bounded publishing adapters remain gated.

## Phase 6 — Monetization
**Status: affiliate discovery + attribution implemented; monetization expansion next**
Affiliate product discovery, deterministic ranking, coupon validation, Awin feed/commission adapters, click/conversion attribution and content-level performance are implemented. Sponsorship CRM, service proposals, invoicing and verified revenue ingestion remain next.

## Phase 7 — Human Organization
**Status: governance primitives implemented; HR economics next**
Hiring proposals and governance exist; payroll, contractor liabilities and performance economics remain next.

## Phase 8 — Controlled Autonomy
**Status: controlled runtime implemented; autonomy expansion gated**
The system currently supports proposal → Governor → bounded execution for safe actions. External/material side effects remain explicitly gated.

## Phase 9 — Portfolio Company
**Status: planned**
Multiple creators/business units, capital allocation and automated shutdown/reinvestment.

## Phase 10 — Self-sustaining operation
**Status: planned**
Recurring autonomous operating cycles with human oversight focused on governance, exceptions and strategic review.

## Release rule
Do not advance a phase until its previous phase passes tests and operational acceptance criteria.
