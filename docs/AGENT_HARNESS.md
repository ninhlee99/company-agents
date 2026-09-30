# Agent Harness Specification

## Purpose
The harness is the runtime contract between an AI model and the company.

## Agent lifecycle
OBSERVE → PLAN → PROPOSE → AUTHORIZE → EXECUTE → VERIFY → MEASURE → LEARN.

## Harness responsibilities
- load role and skill context
- expose only authorized tools
- validate tool arguments with schemas
- enforce budgets and approval thresholds
- record every decision/tool call
- retry transient failures safely
- prevent duplicate financial actions through idempotency keys
- attach outcomes to decisions
- persist useful organizational memory

## Tool classes
READ: state, metrics, documents
PROPOSE: budgets, hiring, experiments, investments
EXECUTE: only after authorization
EXTERNAL: publish, message, contract, payment adapters

## Decision envelope
Every material proposal contains:
objective, evidence, alternatives, cost, expected value, downside, time horizon, confidence, requested authority and rollback/exit criteria.

## Safety
- no raw SQL tools for agents
- no unrestricted shell
- no arbitrary network access
- no secret exposure
- no self-permission changes
- no direct ledger mutation outside finance service
