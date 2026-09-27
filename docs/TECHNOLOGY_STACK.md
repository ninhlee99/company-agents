# Technology Stack

## Objective
Build a fast, RAM/CPU/SSD-efficient, local-first autonomous media company platform.

## Core stack
- TypeScript + Node.js 22+: Company OS, API, agent runtime, governance and workflows.
- PostgreSQL 17: source of truth for transactional state, accounting, events and metadata.
- pgvector: semantic retrieval for structured company knowledge; keep embeddings selective.
- Redis 7: optional cache/short-lived coordination only; never the financial source of truth.
- Object storage: video, audio, images and large artifacts; PostgreSQL stores metadata and checksums.
- FFmpeg: isolated media worker for transcoding, clipping and normalization.
- Python: analytics/forecasting/ML workloads only where it provides clear value.
- pnpm workspaces: minimal monorepo management.
- Docker Compose: local development and simulation environment.

## Architecture principles
1. Modular monolith first; split services only when load or isolation requires it.
2. Event-driven waking; agents sleep when no work exists.
3. PostgreSQL transactional outbox before Kafka.
4. Deterministic code owns money, permissions, state transitions and limits.
5. LLMs propose typed commands; they never mutate critical state directly.
6. Large media never enters prompts or PostgreSQL rows.
7. Store structured facts and summaries rather than unbounded chat history.
8. Every external integration is an adapter with retries, rate limits, idempotency and audit logs.

## Runtime processes
- company-api: HTTP/API and operator UI backend.
- company-worker: event consumption, agent scheduling, governance and jobs.
- media-worker: CPU/GPU-heavy media processing.
- postgres: durable state.
- redis: optional acceleration layer.

## Performance budgets
Initial target on a developer machine:
- API idle RAM < 250 MB.
- Worker idle RAM < 350 MB.
- Media jobs isolated from agent worker.
- No polling loops faster than 1 second.
- Batch analytics queries; avoid N+1 queries.
- Use indexes only for measured access paths.
- Retain raw high-volume metrics in partitioned/aggregated storage policies.
- Compress archived events and media derivatives.
