# Company Agents

Autonomous Company OS — an AI-native operating system for building and operating a self-sustaining company.

The initial business model is an AI-native media and creator company. The deeper goal is a reusable economic operating system in which AI agents can research opportunities, create businesses, allocate capital, hire, execute, learn from outcomes, scale winners, shut down losers, and eventually enter distress or bankruptcy.

## Core thesis

> Build a company that optimizes for sustainable free cash flow, not agent activity or vanity metrics.

## Architecture

- TypeScript / Node.js — Company OS, APIs, agent runtime
- Python — analytics, forecasting, media processing
- PostgreSQL + pgvector — system of record and organizational memory
- Redis / NATS — events and queues
- Temporal — durable workflows
- Next.js — operations dashboard
- Docker — local development and deployment
- OpenTelemetry — observability

## First vertical

AI-native media / creator factory:

Research → ideation → production → distribution → measurement → monetization → reinvestment.

## Non-negotiable economic rules

1. Agents cannot create money.
2. Financial state is controlled by an immutable double-entry ledger.
3. Agents cannot modify their own permissions.
4. Every material action is auditable.
5. Capital allocation requires explicit authorization.
6. Views and followers are not substitutes for profit.
7. Failed businesses can be shut down.
8. If the company cannot satisfy mandatory obligations and no recovery plan exists, it enters liquidation/bankruptcy.

## Development strategy

Build the simulator and economic engine before connecting real-world money or platform accounts.

See:
- docs/PROJECT_SPEC.md
- docs/ARCHITECTURE.md
- docs/COMPANY_CONSTITUTION.md
- docs/BUSINESS_MODEL.md
- docs/ROADMAP.md
- docs/AGENT_HARNESS.md

## Status

Specification-first foundation. Implementation should proceed vertically: ledger → simulator → governance → agents → media → external integrations.
