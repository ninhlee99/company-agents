# Architecture

## Layers

1. Governance — constitution, policies, permissions, approval thresholds
2. Company domain — company, business units, people, customers, products
3. Economics — immutable ledger, treasury, budgets, revenue, costs, runway
4. Agent runtime — planning, typed tools, memory, evaluation
5. Workflow — durable event-driven jobs
6. Integrations — media platforms, CRM, payments, cloud, communications
7. Intelligence — analytics, forecasting, experimentation
8. UI — server-rendered company cockpit

## Runtime strategy

The core runtime is Rust, with specialized workers chosen by workload.

`event -> scheduler -> agent -> typed proposal -> governance -> deterministic executor -> audit -> event`

Rust owns the safety-critical path. Go and Python are optional escape hatches, not runtime requirements. Introduce them only when a measured workload justifies the extra runtime/toolchain footprint.

## Repository layout

```
apps/
  company-os/            # Rust control plane
  media-worker/          # Rust worker around FFmpeg
  llm-web-relay/         # Rust browser/web relay control plane
  llm-web-worker/        # Rust relay worker
  outbox-worker/         # Rust durable outbound delivery worker
crates/
  economic-core/         # Rust deterministic economics
  agent-runtime/         # Rust Agent Runtime
  governance/            # Rust policies/permissions
  event-bus/             # Rust event/outbox layer
  integrations/          # Rust adapter interfaces and selected implementations
web/
  templates/             # server-rendered HTML
  static/                # CSS + tiny optional JS/HTMX
agents/                  # role definitions
skills/                  # reusable skills
simulation/              # deterministic simulator
infra/                   # migrations, containers, deployment
docs/
```

## Critical boundary

Agents never receive unrestricted database credentials.

Tool calls are narrow and typed. Example capabilities:
- `request_expense`
- `propose_hire`
- `create_experiment`
- `publish_content`
- `create_customer_followup`

LLMs can produce proposals; deterministic services decide whether those proposals are authorized and economically valid.

## External integration rule

Every external provider sits behind an adapter contract:

`Agent intent -> typed command -> policy check -> adapter -> verified result -> audit event`

No provider SDK is allowed to leak directly into Agent prompts or critical domain logic.


## Current repository shape

The checked-in implementation is a Rust-first monorepo:
- `apps/company-os`: operator control plane and agent orchestration.
- `apps/media-worker`: media processing worker; FFmpeg is the heavy native dependency.
- `apps/llm-web-relay` + `apps/llm-web-worker`: isolated web/LLM relay path.
- `apps/outbox-worker`: durable outbound delivery.
- `crates/*`: domain, governance, economics, attribution, publishing and integration contracts.
- PostgreSQL: durable state, idempotency, event/outbox evidence and audit data.
- Server-rendered HTML: intentionally avoids a permanent Node/SPA runtime for the operator cockpit.

## Resource hierarchy

The dominant resource consumers are expected to be external model inference, PostgreSQL and FFmpeg—not the Rust HTTP/control layer. Therefore optimization work should target:
1. model size/quantization and inference concurrency;
2. media resolution/FPS/codec settings and process limits;
3. database indexes, connection pooling and query shape;
4. Tokio concurrency and queue bounds;
5. only then micro-optimizations in Rust code.
