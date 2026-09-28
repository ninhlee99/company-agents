# Agent Company System Audit

## Executive assessment
The repository has a Rust-based Agent Runtime with deterministic economic and governance boundaries. It is a controlled-runtime platform with automated production acceptance gates; real-world autonomy remains gated by operator credentials, third-party account approval and production deployment evidence.

## Critical findings fixed
1. Per-Agent action capability matrix.
2. Context-aware action envelope for runway, distress, hiring, content and experiment conditions.
3. Proposal schema validation for authority, cost, risk, evidence and action.
4. Action-specific cost ceilings and minimum risk.
5. Material external publishing escalation.
6. Distress/emergency discretionary spending block.
7. Model output cannot self-escalate permissions.
8. Malicious model suggestions cannot bypass deterministic policy.
9. Bounded model timeout and fail-closed model errors.
10. Untrusted company snapshots and strict model JSON handling.
11. Bounded runtime concurrency.
12. Randomized 2,000-snapshot invariant evaluation and 256 full multi-agent stress cycles.
13. Least-privilege Tool Registry.
14. Database-side immutable ledger/audit protections and deferred balance checking.
15. Local-only Docker development bindings.
16. Deterministic execution engine with global cycle spend cap and execution-bound capability recheck.
17. PostgreSQL idempotency, decision journal, audit and outbox persistence.
18. Lease-based scheduler recovery with replay-safe run tokens.
19. Simulator reuse of the production execution engine.
20. Durable outbox dispatcher with leases, bounded retries and optional HMAC-signed HTTPS delivery.
21. Affiliate ranking, coupon checks, evidence confidence, dedupe, Awin and TikTok Shop adapters.
22. Durable Agent memory with bounded retention and untrusted-context semantics.
23. Persistent per-Agent rate windows enforced before model calls.
24. Affiliate click/conversion attribution, reconciliation and verified revenue accounting.
25. Durable media queue, isolated FFmpeg execution, FFprobe inspection and post-render QA.
26. Publishing approval, lease and idempotency contract.
27. Employee, payroll, business-unit and portfolio economics primitives.
28. LLM web-relay queue with leases, expiry, authentication and request fingerprints.
29. Database-backed readiness and Prometheus-style runtime metrics.
30. Automated CI gates for formatting, tests, Clippy, migration lint, secret hygiene and Docker builds.
31. Scheduled PostgreSQL backup/restore recovery drill.
32. Explicit production acceptance and secret-rotation standard.

## Remaining acceptance items

### P0 — environment-dependent
- Run distributed tracing/metrics against the actual multi-worker production topology.
- Perform a production-like restart/replay and chaos rehearsal.
- Validate secret rotation against the chosen external secret manager.
- Execute load tests against the actual production PostgreSQL sizing.

### P1 — business expansion
- Richer creator/business-unit simulator calibration from real observations.
- Sponsorship CRM, service proposals and invoicing.
- Broader verified revenue ingestion beyond affiliate networks.
- Additional platform publishing adapters where the operator has approved accounts.

### P2 — autonomy expansion
- Model benchmark matrix by Agent.
- Task-complexity model routing and hardware-aware selection.
- Shadow mode with real data.
- Controlled capital allocation across business units.
- Automated hiring/payroll workflows beyond the current governed primitives.

## Release rule
An Agent is production-ready only when unit, contract, scenario, adversarial, economic, security, load, recovery and chaos checks pass without unauthorized material actions and with deterministic replay evidence.

The repository should not be treated as unsupervised autonomous production until the environment-dependent gates above have been executed with real deployment infrastructure and credentials.
