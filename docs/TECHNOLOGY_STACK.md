# Technology Stack

## Strategy

This system uses a **polyglot architecture chosen by workload**.

Rust is the default for the Company OS control plane, economic core, agent runtime and governance because these are long-running, concurrency-heavy, safety-sensitive paths. Go remains available for small operational services and infrastructure tooling where its simplicity materially reduces implementation or operational cost.

Python is reserved for analytics/forecasting/ML. FFmpeg is used for media transformation. PostgreSQL is the durable source of truth.

The rule is simple: **best tool for the job, clear service boundaries, minimum unnecessary complexity**.

## Core architecture

- **Rust** — Company OS, economic kernel, Agent Runtime, scheduler, governance, permissions, APIs, workflow workers.
- **Go** — optional infrastructure/service utilities where its operational simplicity is a better fit; not required for the core.
- **Python** — analytics, forecasting, ML experimentation and model-specific pipelines.
- **FFmpeg** — video/audio transformation.
- **PostgreSQL 17** — transactional source of truth.
- **pgvector** — selective semantic retrieval.
- **Redis 7** — optional cache and ephemeral coordination only.
- **S3-compatible object storage** — media/artifact blobs.
- **OpenTelemetry** — metrics, traces and logs.

## UI

No React and no Vue.

The operator interface is server-rendered HTML from the Rust service. Ordinary HTML forms, links, tables and progressive enhancement are preferred.

HTMX or tiny amounts of plain JavaScript may be introduced only where they remove real interaction friction without turning the UI into an SPA.

## Architecture principles

1. Modular monolith first.
2. Event-driven Agent waking; idle agents sleep.
3. PostgreSQL transactional outbox before introducing a dedicated message broker.
4. Rust deterministic code owns money, permissions, limits and state transitions.
5. LLMs reason and propose typed commands; they never directly mutate critical state.
6. Large media never enters prompts or normal relational rows.
7. Store structured facts, decisions and lessons instead of unbounded chat history.
8. External integrations use narrow adapters with retry, rate limit, idempotency and audit.
9. Add a new runtime only when the workload and measured evidence justify it.

## Runtime processes

- `company-os`: Rust API, operator UI, Agent control plane and scheduler.
- `company-worker`: Rust background workers for isolated execution.
- `analytics-worker`: Python only for analytics/forecasting/ML jobs.
- `media-worker`: FFmpeg with optional Rust/Python orchestration.
- `postgres`: durable state.
- `redis`: optional acceleration.

## Resource goals

Initial targets on a developer machine:
- company-os idle RAM < 100 MB before DB/model clients,
- worker idle RAM < 150 MB,
- event-driven wakeups; no tight polling,
- batch analytics and measured indexes,
- isolate CPU/GPU-heavy workloads from the control plane.
