# Technology Stack

## Primary platform language
Go is the primary implementation language for the Company OS.

Use Go for:
- Company OS API
- Agent runtime and scheduler
- Governance and permission enforcement
- Economic/domain services
- Workflow orchestration
- Operator dashboard
- External integration adapters
- Background workers

Use Rust selectively when a component has a measured need for:
- very high-throughput or low-latency processing
- memory-tight media/data pipelines
- CPU-heavy workloads that benefit materially from Rust

Do not introduce Rust by default. A component must justify the additional language/runtime complexity with measured performance or isolation requirements.

## UI
No React and no Vue.

The initial operator UI is server-rendered HTML using Go html/template with a small amount of progressive enhancement only where useful. Prefer ordinary HTML forms, links and tables over a client-side SPA.

Possible lightweight enhancement:
- HTMX only for small interactive fragments, not as an application framework.
- Plain CSS or a small CSS layer; no mandatory frontend build pipeline.

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
4. Deterministic code owns money, permissions, limits and state transitions.
5. LLMs propose typed commands; they never directly mutate critical state.
6. Large media never enters prompts or PostgreSQL rows.
7. Store structured facts and lessons rather than unbounded chat history.
8. External integrations use adapters with retries, rate limits, idempotency and audit.
9. Keep the hot path small: standard library first, dependencies only when justified.

## Runtime processes
- company-os: Go HTTP API, server-rendered operator UI, agent scheduler and control plane.
- company-worker: Go background execution when workload isolation is required.
- media-worker: FFmpeg/Python/Rust process isolated from agent runtime.
- postgres: durable state.
- redis: optional acceleration layer.

## Performance targets
Initial developer-machine targets:
- company-os idle RAM < 100 MB before database connections and model clients.
- background worker idle RAM < 150 MB.
- agent wakeups are event driven; no polling loops faster than 1 second.
- batch analytics queries; avoid N+1 queries.
- isolate CPU/GPU-heavy media work.
- measure before introducing caches, queues or service splits.
