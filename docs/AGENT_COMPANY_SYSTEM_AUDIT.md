# Agent Company System Audit

## Executive assessment
The repository now has a Rust-based Company OS with deterministic economic/governance boundaries, a durable control plane, a governed execution engine, and an affiliate intelligence subsystem. It is substantially beyond a proposal-only harness, but it remains controlled-runtime software rather than unrestricted production autonomy.

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

## Test volume

There are currently 40 test functions across the Rust economic core, Agent Runtime, Governor, tool registry, simulator and PostgreSQL store. In addition, the suite exercises 720 Agent × Action × CompanyStatus combinations, 2,000 randomized company snapshots and 256 full multi-agent cycles.

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
- production secret-manager integration
- audit-event ingestion/retention and operator alerting
- durable outbox worker with retry/lease semantics
- distributed scheduler/lease semantics
- per-Agent durable budget/rate-limit accounting
- automated recovery/replay verification

### P1
- richer creator/business-unit simulator with payroll and contract liabilities
- affiliate click/conversion/order attribution
- verified payout/revenue ingestion
- sponsorship CRM and service/invoicing flows
- media production + QA + publishing pipeline
- provider contract-test fixtures and real API sandbox tests
- PostgreSQL load test and backup/restore drill

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

## 2026-09 implementation checkpoint

The controlled Company OS now includes durable cycle execution, PostgreSQL-backed scheduling, transactional/idempotent ledger and state persistence, decision journaling, affiliate product intelligence and attribution, company operating aggregates, bounded FFmpeg processing, and a gated TikTok Direct Post adapter.

External side effects remain approval-bound. Provider credentials are configuration-only and must not be persisted in business state.

Validation status is tracked from GitHub Actions runs on the feature branch. No production-autonomy claim is made until unit, integration, security, load, recovery, and chaos gates are green.
