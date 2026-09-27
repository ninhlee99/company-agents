# Agent Testing Standard

## Principle
An agent is not production-ready because it succeeds on happy-path prompts. It is production-ready only after deterministic, adversarial, economic and operational tests pass.

## Test layers
1. Unit tests: schemas, calculations, state transitions.
2. Contract tests: tool inputs/outputs and integration adapters.
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
