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

Rust owns the safety-critical path. Go can be used for isolated operational services or infrastructure utilities. Python is isolated to data/ML workloads.

## Repository layout

```
apps/
  company-os/            # Rust control plane
workers/
  company-worker/        # Rust durable background work
  analytics-worker/      # Python analytics/forecasting/ML
  media-worker/          # FFmpeg + optional Rust/Python orchestration
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
