# Implementation Plan

## Phase 1 — Economic Kernel
**Status: foundation implemented; durable execution still pending**
- Rust deterministic economic core
- immutable double-entry ledger validation
- company state and budget guards
- idempotency primitives
- PostgreSQL schema with ledger, budget, idempotency, outbox and audit tables
- bankruptcy/liquidation spending guards
- invariant tests

Remaining before Phase 1 acceptance:
- transactional repository/service layer
- atomic ledger + outbox commit
- persisted state transitions
- recovery/restore test

## Phase 2 — Simulator
**Status: next**
- deterministic seeded world
- creators, content, traffic, conversion and revenue models
- expense/payroll models
- adversarial scenarios
- replayable simulation

## Phase 3 — Agent Harness
**Status: core implemented; full persistence/tooling pending**
- typed agent contract
- OpenAI-compatible model router + safe Mock Model
- bounded concurrent Agent Runtime
- Governor policy engine
- fail-closed model outage behavior
- agent contract and stress tests

Remaining before Phase 3 acceptance:
- typed tool registry with capability enforcement
- durable agent memory
- persisted event scheduler
- per-agent durable budget/rate-limit accounting
- replayable decision journal

## Phase 4 — Agents
**Status: implemented and tested**
- Governor policy engine
- CEO
- CFO
- COO
- Growth
- Content
- Recruiter
- Analyst
- Experiment

Each operating agent has:
- role contract
- system prompt source in agents/<role>/agent.md
- deterministic economic policy
- typed proposal output
- Governor review
- model-outage fail-closed behavior

## Phase 5 — Media Factory
**Status: next**
- research
- content planning
- scripting
- rendering
- QA
- publishing adapters
- analytics
- attribution

## Phase 6 — Monetization
**Status: next**
- affiliate adapter
- sponsorship CRM
- service proposals
- invoicing
- verified revenue ingestion

## Phase 7 — Human Organization
**Status: planned**
- employee records
- payroll abstractions
- hiring workflow
- contractor workflow
- performance review

## Phase 8 — Controlled Autonomy
**Status: planned**
Shadow -> approved actions -> bounded autonomy -> broader autonomy.

## Phase 9 — Portfolio Company
**Status: planned**
Multiple creators/business units, capital allocation and automated shutdown/reinvestment.

## Phase 10 — Self-sustaining operation
**Status: planned**
Company can operate recurring cycles with human oversight focused on governance, exceptions and strategic review.

## Rule
Do not advance a phase until its previous phase passes its tests and operational acceptance criteria.
