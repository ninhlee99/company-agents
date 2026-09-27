# Agent Company System Audit

## Executive assessment
The repository now has a Rust-based Agent Runtime and deterministic economic/governance boundaries. It is suitable as a controlled development/simulation platform, but it is not yet production-autonomous.

## Critical findings fixed in this pass
1. Per-Agent action capability matrix.
2. Context-aware action envelope for runway, distress, hiring, content and experiment conditions.
3. Proposal schema validation for authority, cost, risk, evidence and action.
4. Action-specific cost ceilings.
5. Action-specific minimum risk.
6. External publishing classified as material and escalated.
7. Distress/emergency discretionary spending blocked.
8. Model output cannot self-escalate permissions.
9. Malicious model suggestions cannot bypass deterministic policy.
10. Model calls have bounded timeout.
11. Company snapshots are explicitly marked as untrusted model input.
12. Invalid model JSON fails closed.
13. Agent model outage produces escalation instead of execution.
14. Bounded runtime concurrency.
15. Randomized 2,000-snapshot invariant evaluation.
16. 256 full multi-agent stress cycles.
17. Individual branch tests for every operating Agent.
18. Least-privilege Tool Registry.
19. Database-side immutable ledger/audit protections and deferred transaction-balance checking.
20. Local-only Docker service bindings for development.

## Agent coverage
| Agent | Primary invariant | Adversarial coverage |
|---|---|---|
| Governor | cannot self-authorize | self-permission, material action, distress, liquidity |
| CEO | capital bounded by state | distress guard, role/action boundary |
| CFO | liquidity first | low-runway and negative-FCF branches |
| COO | capacity before expansion | backlog/capacity branch |
| Growth | attention tied to economics | low conversion / negative growth |
| Content | negative unit economics causes redesign | loss branch |
| Recruiter | hiring requires economic headroom | hiring gate + adversarial cross-role action |
| Analyst | report-only capability | cross-role permission attack |
| Experiment | bounded reversible tests | zero-budget + randomized snapshots |

## Test categories currently represented
- Unit / invariant
- Contract
- Scenario
- Adversarial model behavior
- Fault injection: model outage
- Fault injection: model timeout
- Economic guard tests
- Permission/capability tests
- Randomized property-like testing
- Multi-agent stress
- CI lint/test gate

## Remaining production gates

### P0
- PostgreSQL-backed durable state and transaction executor
- atomic ledger + outbox transaction
- durable idempotency
- persistent Agent decision journal
- real tool executor with capability enforcement at the execution boundary
- secrets isolation
- audit ingestion
- restart/replay recovery

### P1
- deterministic business simulator with revenue, costs, payroll and bankruptcy
- external adapter contract tests
- content/media QA pipeline
- affiliate/sponsorship revenue attribution
- rate-limit/backoff and provider outage handling
- load tests against PostgreSQL
- backup/restore drill

### P2
- model benchmark matrix by Agent
- model router by task complexity and hardware profile
- shadow mode with real data
- bounded real-world publishing
- controlled capital allocation
- hiring/payroll workflows

## Release rule
An Agent is considered production-ready only when unit -> contract -> scenario -> adversarial -> economic -> security -> load -> recovery -> chaos passes without unauthorized material actions and with deterministic replay evidence.

The current repository should be treated as controlled-runtime / pre-autonomy until the P0 gates are complete.