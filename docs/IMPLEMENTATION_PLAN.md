# Implementation Plan

## Phase 1 — Economic Kernel
- PostgreSQL schema
- immutable double-entry ledger
- company state
- budgets
- idempotency
- transactional outbox
- distress/bankruptcy state machine
- invariant tests.

## Phase 2 — Simulator
- deterministic seeded world
- creators, content, traffic, conversion and revenue models
- expense/payroll models
- adversarial scenarios
- replayable simulation.

## Phase 3 — Agent Harness
- typed agent contract
- model router
- tool registry
- permission engine
- budgets/rate limits
- memory
- event-driven scheduler.

## Phase 4 — Executive Agents
Governor, CFO, CEO, COO, Analyst and Experiment Agent.

## Phase 5 — Media Factory
Research, content planning, scripting, rendering, QA, publishing adapters, analytics and attribution.

## Phase 6 — Monetization
Affiliate adapter, sponsorship CRM, service proposals, invoicing and verified revenue ingestion.

## Phase 7 — Human Organization
Employee records, payroll abstractions, hiring workflow, contractor workflow and performance review.

## Phase 8 — Controlled Autonomy
Shadow -> approved actions -> bounded autonomy -> broader autonomy.

## Phase 9 — Portfolio Company
Multiple creators/business units, capital allocation and automated shutdown/reinvestment.

## Phase 10 — Self-sustaining operation
Company can operate recurring cycles with human oversight focused on governance, exceptions and strategic review.

## Rule
Do not advance a phase until its previous phase passes its tests and operational acceptance criteria.
