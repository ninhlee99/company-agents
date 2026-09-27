# Architecture

## Layers

1. Governance — constitution, policies, permissions, approval thresholds
2. Company domain — company, business units, people, customers, products
3. Economics — ledger, treasury, budgets, revenue, costs, runway
4. Agent runtime — planning, tool calling, memory, evaluation
5. Workflow — durable multi-step processes and scheduled loops
6. Integrations — media platforms, CRM, payments, cloud, communications
7. Intelligence — analytics, forecasting, experimentation
8. UI — company cockpit and audit views

## Runtime

Event-driven architecture with durable workflows.

Company state is read from the domain and ledger. Agents propose typed commands. Governance validates them. Commands execute deterministic business logic and emit events. Agents observe outcomes and learn.

## Suggested repository layout

apps/api
apps/web
apps/worker
apps/media-worker
packages/domain
packages/finance
packages/agents
packages/governance
packages/events
packages/workflows
packages/tools
packages/ai
packages/observability
python/analytics
python/forecasting
python/media
agents
skills
docs
simulation
tests

## Critical boundary
Agents never receive unrestricted database credentials. Tools expose narrow capabilities such as request_expense, propose_hire, create_experiment and publish_content.
