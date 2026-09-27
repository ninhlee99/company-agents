# Technology Decision Matrix

## Principle

Choose the smallest, fastest, safest tool that fits the workload. Do not optimize for language purity.

The Company OS is polyglot by design, with Rust as the default for the control plane and economic core, not as a mandatory language for every component.

## Default allocation

| Area | Technology | Why |
|---|---|---|
| Company OS control plane | Rust | Low overhead, strong type and memory safety, long-running reliability |
| Economic kernel / accounting | Rust | Deterministic behavior and strong error modeling |
| Governance / permissions | Rust | Safety-critical policy enforcement |
| Agent runtime | Rust | Concurrency, resource control, long-running workers |
| Scheduler / event processing | Rust | Efficient async execution |
| Public/internal API | Rust | Same trust boundary as core services |
| Operator dashboard | Server-rendered HTML from Rust | No React/Vue and low client complexity |
| High-throughput specialized services | Rust | Use when benchmarked advantage is real |
| Simple infrastructure utilities | Go or Rust | Choose the smaller, simpler implementation |
| Data science / forecasting / ML | Python | Ecosystem and iteration speed |
| Model evaluation | Python | Fast experimentation |
| Media transformation | FFmpeg | Mature specialized implementation |
| Computer vision / ML inference | Python/C++/Rust | Select by runtime support and measured performance |
| Database | PostgreSQL | Durable source of truth |
| Vector retrieval | pgvector | Retrieval close to transactional data |
| Cache / ephemeral coordination | Redis | Optional acceleration only |
| Large media/artifacts | S3-compatible object storage | Durable blob storage |
| Messaging | PostgreSQL outbox first | Atomicity with fewer moving parts |
| Higher-scale messaging | NATS/Kafka | Introduce only when measured load justifies it |
| Observability | OpenTelemetry | Vendor-neutral telemetry |
| CI/CD | GitHub Actions | Repository-native automation |
| Deployment | Docker Compose first | Minimal operational complexity |

## Frontend constraint

No React and no Vue.

The first interface is server-rendered HTML. HTMX or tiny plain JavaScript may be used for isolated interactions, but there is no mandatory SPA.

## Rust boundary

Rust owns the safety-critical path:

`LLM proposal -> validation -> authorization -> deterministic execution -> audit -> event`

LLMs never receive direct database credentials or direct mutation access to money, permissions, or governance.

## Language introduction rule

A new language or major runtime requires a written reason covering:
1. workload,
2. measurable advantage,
3. security/operational impact,
4. deployment impact,
5. maintenance cost,
6. API/service boundary.

Avoid adding a language merely because it is fashionable or theoretically faster.
