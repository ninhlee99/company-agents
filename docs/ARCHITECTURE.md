# Architecture

## Layers

1. Governance — constitution, policies, permissions, approval thresholds
2. Company domain — company, business units, people, customers, products
3. Economics — ledger, treasury, budgets, revenue, costs, runway
4. Agent runtime — planning, typed tool calls, memory, evaluation
5. Workflow — durable event-driven jobs and scheduled loops
6. Integrations — media platforms, CRM, payments, cloud, communications
7. Intelligence — analytics, forecasting, experimentation
8. UI — Rust server-rendered company cockpit and audit views

## Runtime

The primary runtime is a Rust modular monolith.

event -> scheduler -> agent -> typed proposal -> governance -> deterministic executor -> event

Company state is read from domain services and the ledger. Agents never receive unrestricted database credentials.

## Repository layout

apps/company-os/
crates/economic-core/
internal/agent/
internal/governance/
internal/events/
internal/workflow/
internal/integrations/
internal/analytics/
web/templates/
web/static/
agents/
skills/
infra/
docs/
simulation/

## UI strategy

There is no React or Vue application. The operator UI is served directly by Rust as server-rendered HTML.

The first dashboard exposes:
- cash, revenue, expenses, free cash flow and runway
- agent status and current tasks
- pending approvals
- experiments
- content/creator performance
- incidents and audit trail

## Critical boundary

Agents never receive unrestricted database credentials. Tools expose narrow capabilities such as request_expense, propose_hire, create_experiment and publish_content.

Money, permissions and company state are enforced by deterministic Rust services and PostgreSQL constraints.
