# Agent Testing Standard

## Principle
An agent is not production-ready because it succeeds on happy-path prompts. It is production-ready only after deterministic, adversarial, economic and operational tests pass.

## Current implemented test layers

The Rust workspace currently covers:
1. Economic invariant tests for money, ledger balance, budget limits and bankruptcy guards.
2. Governor policy tests for bounded approval, material escalation, self-permission escalation, bankrupt-state blocking and liquidity limits.
3. Agent contract tests for every operating agent.
4. Model outage tests proving fail-closed behavior.
5. Bankruptcy scenario tests proving discretionary actions are rejected.
6. Stress tests running 256 complete multi-agent cycles with bounded concurrency.
7. CI configuration for workspace tests and clippy.

The suite is intentionally deterministic and uses a safe Mock Model so agent policy tests do not depend on a live LLM.

## Test layers
1. Unit tests: schemas, calculations, state transitions.
2. Contract tests: agent proposals and integration adapter contracts.
3. Scenario tests: realistic business workflows.
4. Property-based tests: invariants over generated inputs.
5. Adversarial tests: prompt injection, conflicting instructions, poisoned data, misleading metrics.
6. Fault injection: timeouts, duplicates, stale data, partial failures.
7. Economic tests: budget exhaustion, negative margin, cash crisis, insolvency.
8. Security tests: least privilege, secret leakage, unauthorized commands.
9. Load tests: concurrency, queue growth, database contention.
10. Recovery tests: restart, replay, idempotent execution.
11. Regression tests: every production incident becomes a permanent test.

## Required invariants
- Ledger transactions balance.
- Money cannot be created by an agent.
- Duplicate commands are idempotent.
- Budgets cannot be exceeded.
- Closed/bankrupt companies cannot execute prohibited spending.
- Permissions cannot be self-escalated.
- Audit events are immutable.
- Verified revenue is distinguishable from estimates.
- Failed external calls do not create false success.

## Release gates
Every agent must pass:
- 100% critical invariant tests
- zero unauthorized material actions in adversarial suite
- deterministic replay for core workflows
- bounded resource usage
- documented failure modes
- rollback/recovery test.

## Current release boundary

The implemented Agent Runtime is a safe proposal/governance runtime. It does **not** yet claim production autonomy over real money, payroll, platform accounts or external publishing.

Real execution is unlocked only after:
- durable persistence
- typed tool registry
- real adapter contract tests
- secrets isolation
- platform/legal review
- simulator acceptance
- shadow-mode evaluation
- chaos and recovery gates.
