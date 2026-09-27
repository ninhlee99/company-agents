# Data & Analytics

## Data layers
1. Transactional: company, ledger, budgets, contracts, tasks.
2. Operational: jobs, content, publications, platform metrics.
3. Analytical: aggregates, cohorts, attribution, forecasts.
4. Knowledge: structured facts, decisions, lessons and embeddings.

## Event model
Important events are append-only and carry:
- event_id
- type
- occurred_at
- actor
- company_id
- aggregate_id
- idempotency_key
- payload
- schema_version.

## Metrics hierarchy
North-star economic measures:
- free cash flow
- contribution margin
- cash runway
- return on deployed capital.

Operating measures:
- revenue per content
- revenue per 1,000 views
- conversion
- CAC
- LTV
- creator contribution
- sponsor margin.

Attention measures are diagnostic, not the company objective.

## Attribution
Use deterministic attribution rules and record uncertainty. Never present estimated revenue as verified cash.
