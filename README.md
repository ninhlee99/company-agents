# Company Agents

Autonomous Company OS — an AI-native operating system for building and operating a self-sustaining company.

The initial business model is an AI-native media and creator company. The deeper goal is a reusable economic operating system in which AI agents can research opportunities, create businesses, allocate capital, hire, execute, learn from outcomes, scale winners, shut down losers, and eventually enter distress or bankruptcy.

## Primary implementation

The core Company OS is Rust.

There is no React and no Vue. The operator dashboard is server-rendered HTML from Rust with no SPA/frontend build pipeline.

The architecture is workload-based:
- Rust: Company OS, Agent Runtime, governance, economic core, scheduler and control plane.
- Python: analytics, forecasting and ML where justified.
- FFmpeg: media transformation.
- Go: optional small infrastructure utilities where its simplicity is a better fit.
- PostgreSQL: durable source of truth.

## Implemented agents

Operating agents:
- Governor — policy and authorization engine
- CEO — strategy and capital allocation proposals
- CFO — solvency, cash and unit economics
- COO — operations and capacity
- Growth — profitable demand
- Content — content economics and experiments
- Recruiter — capacity and hiring economics
- Analyst — verified decision support
- Experiment — bounded opportunity discovery

Agents are proposal-driven. LLMs can reason, but deterministic Rust governance and economics control authorization and critical state.

## Run with Docker

You only need Git and Docker.

Clone:

    git clone https://github.com/ninhlee99/company-agents.git
    cd company-agents

Create local configuration:

    cp .env.example .env

Start everything:

    docker compose up --build

Open:

    http://localhost:8080

The default configuration uses the built-in Mock Model, so the dashboard and all agent cycles run without an LLM API key.

To use an OpenAI-compatible LLM, set these values in .env:

    LLM_API_KEY=...
    LLM_BASE_URL=https://your-provider.example/v1
    LLM_MODEL=...

Then restart:

    docker compose up --build

## Run natively

Requirements:
- Rust stable
- Docker for PostgreSQL/Redis when those services are enabled

Start the Company OS:

    cargo run -p company-os

Run the complete Rust test suite:

    cargo test --workspace

Quality checks:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

## Runtime API

- GET / — operator dashboard
- GET /healthz — health check
- GET /api/agents — latest agent proposals and Governor decisions
- POST /api/run — run one complete decision cycle
- POST /run — run one cycle and return to dashboard

The current runtime uses a deterministic demo company snapshot. PostgreSQL schema and economic migrations are present, while durable persistence and external platform integrations are wired in later milestones.
