# Technology Stack

## Primary platform language

Rust is the primary implementation language for the Company OS.

Use Rust for:
- Company OS API and operator control plane
- Agent runtime and scheduler
- Governance and permission enforcement
- Economic/domain services
- Workflow orchestration
- External integration adapters
- Background workers
- Performance-sensitive data and media components when justified

Use Python only where it materially improves analytics, forecasting or ML workloads. Use FFmpeg as an isolated media tool.

## UI

No React and no Vue.

The initial operator UI is server-rendered HTML from Rust. Prefer ordinary HTML forms, links, tables and progressive enhancement rather than a client-side SPA.

HTMX is optional for small interactive fragments, not an application framework.

## Data and infrastructure

- PostgreSQL 17: source of truth for transactional state, accounting, events and metadata.
- pgvector: selective semantic retrieval for company knowledge.
- Redis 7: optional cache/short-lived coordination only; never the financial source of truth.
- Object storage: video/audio/images and large artifacts; PostgreSQL keeps metadata/checksums.
- FFmpeg: isolated media worker.
- Python: analytics/forecasting/ML only where it provides clear value.
- Docker Compose: local development and simulation.

## Architecture principles

1. Modular monolith first.
2. Event-driven agent waking; idle agents sleep.
3. PostgreSQL transactional outbox before adding Kafka/NATS.
4. Deterministic Rust code owns money, permissions, limits and state transitions.
5. LLMs propose typed commands; they never directly mutate critical state.
6. Large media never enters prompts or PostgreSQL rows.
7. Store structured facts and lessons rather than unbounded chat history.
8. External integrations use adapters with retries, rate limits, idempotency and audit.
9. Keep dependencies minimal and justified by measured value.

## Runtime processes

- company-os: Rust HTTP API, server-rendered operator UI, agent scheduler and control plane.
- company-worker: Rust background execution when isolation is required.
- media-worker: FFmpeg/Python/Rust process isolated from the agent runtime.
- postgres: durable state.
- redis: optional acceleration layer.

## Performance targets

Initial developer-machine targets:
- company-os idle RAM < 100 MB before database connections and model clients.
- background worker idle RAM < 150 MB.
- event-driven wakeups; no polling loops faster than 1 second.
- batch analytics queries; avoid N+1 queries.
- isolate CPU/GPU-heavy media work.
- measure before introducing caches, queues or service splits.
