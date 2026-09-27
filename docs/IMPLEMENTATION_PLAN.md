# Implementation Plan

## Phase 1 — Economic Kernel
**Status: foundation implemented; durable execution still pending**
- Rust deterministic economic core
- immutable double-entry ledger validation
- company state, runway and budget guards
- checked arithmetic / overflow protection
- PostgreSQL schema with ledger, budget, idempotency, outbox and audit tables
- database-side ledger balance/dependency checks
- bankruptcy/liquidation spending guards
- invariant tests

Remaining before Phase 1 acceptance:
- transactional repository/service layer
- atomic ledger + outbox commit
- persisted state transitions
- recovery/restore test

## Phase 2 — Simulator
**Status: initial deterministic simulator implemented**
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
- multi-run statistical evaluation
- replayable decision journal

## Phase 3 — Agent Harness
**Status: safety-focused runtime implemented; durable tooling pending**
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
- execution-bound tool registry
- durable Agent memory
- persisted event scheduler
- per-agent durable budget/rate-limit accounting
- replayable decision journal
- observability/tracing

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
**Status: planned**
Research → planning → scripting → production → QA → publishing → analytics → attribution.

## Phase 6 — Monetization
**Status: planned**
Affiliate adapters, sponsorship CRM, service proposals, invoicing and verified revenue ingestion.

## Phase 7 — Human Organization
**Status: planned**
Employees, payroll abstractions, hiring workflow, contractors and performance review.

## Phase 8 — Controlled Autonomy
**Status: planned**
Shadow → approved actions → bounded autonomy → broader autonomy.

## Phase 9 — Portfolio Company
**Status: planned**
Multiple creators/business units, capital allocation and automated shutdown/reinvestment.

## Phase 10 — Self-sustaining operation
**Status: planned**
Recurring autonomous operating cycles with human oversight focused on governance, exceptions and strategic review.

## Release rule
Do not advance a phase until its previous phase passes tests and operational acceptance criteria.
