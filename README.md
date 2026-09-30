# VERIDARA AI

**Veridara AI — Autonomous Business Intelligence & Operations Company**

> **Build. Operate. Learn. Compound.**

Veridara AI is an AI-native company built around an internal Autonomous Company OS: a governed system of agents, economics, memory, operations and commercial workflows.

The initial business model is an AI-native media and creator company. The deeper goal is a reusable economic operating system in which AI agents can research opportunities, create businesses, allocate capital, hire, execute, learn from outcomes, scale winners, shut down losers, and eventually enter distress or bankruptcy.

## AI model strategy

The default AI provider is **local Ollama**, so the project can run without a paid API key.

The Docker setup starts Ollama and pulls a configurable local model automatically. Default:

    qwen3:4b

Ollama exposes an OpenAI-compatible API on port 11434, which the Rust Agent Runtime uses without requiring a real API token.

A smaller machine can choose:

    OLLAMA_MODEL=qwen3:1.7b

The Qwen3 family currently includes compact local variants; Ollama lists qwen3:1.7b at about 1.4GB and qwen3:4b at about 2.5GB. Choose based on available RAM/compute.

Optional remote provider:

    LLM_PROVIDER=gemini
    GEMINI_API_KEY=...
    GEMINI_MODEL=gemini-3.6-flash

Google currently provides a Free Tier for selected Gemini API models. Free-tier limits apply, and Google states that free-tier content may be used to improve its products, so local Ollama is preferred for private company data.

## Company identity

**Company:** Veridara AI  
**Operating model:** AI-native company / agent workforce  
**Core platform:** Autonomous Company OS  
**Initial commercial engines:** Media & creator commerce, affiliate commerce, B2B services, sponsorships and commercial partnerships  
**Operating principle:** AI proposes and operates within policy; deterministic systems govern money, permissions and material side effects.

The name **Veridara** is an invented brand chosen to evoke verified intelligence, direction and durable execution. It is a brand choice, not a claim of trademark availability.

## Primary implementation

The production Company OS is Rust. Rust is also the default language for workers and critical domain crates; Go/Python/Node are not required by the production path unless a future workload proves a clear advantage.

The repository also contains an optional React/Vite + Express simulation UI under `src/` and `server.ts`. That surface is simulation-only and must not be treated as the durable company ledger or evidence of real cash, revenue, inventory, payouts or provider success.

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

## Run locally

Requirements:
- Git
- Docker

Clone:

    git clone https://github.com/ninhlee99/company-agents.git
    cd company-agents

Create configuration:

    cp .env.example .env

Start everything:

    docker compose up --build

The first start downloads the configured local Ollama model. This is a one-time local model download stored in the Docker volume.

Open:

    http://localhost:8080

The control plane is authenticated by default. Set CONTROL_PLANE_TOKEN to a random secret of at least 32 bytes and send it as Authorization: Bearer <token>. For an intentionally isolated local-only development instance, CONTROL_PLANE_AUTH_DISABLED=true can be used.

No paid API key is required.

## React/Express simulation UI

For UI prototyping only:

    npm install
    npm start

The simulation server defaults to read-only behavior and is refused as a production company backend. Synthetic mutations require `ALLOW_SIMULATED_ACTIONS=true`; the UI marks synthetic state explicitly.

## Run with an already-installed Ollama

Install Ollama on your host, then pull a model:

    ollama run qwen3:4b

Set in .env:

    LLM_PROVIDER=ollama
    OLLAMA_BASE_URL=http://host.docker.internal:11434/v1
    OLLAMA_MODEL=qwen3:4b

Then start the Company OS. This avoids running a second Ollama server in Docker.

## Native Rust

Requirements:
- Rust stable

    cargo run -p company-os

Tests:

    cargo test --workspace

Quality:

    cargo clippy --workspace --all-targets --all-features -- -D warnings

Resource tuning:

    TOKIO_WORKER_THREADS=4

The Company OS defaults to a bounded Tokio worker count to reduce idle thread-stack memory on large hosts. Increase it only when concurrency measurements justify the extra CPU/RAM.

## Runtime API

- GET / — operator dashboard
- GET /healthz — health check
- GET /api/agents — latest agent proposals and Governor decisions
- POST /api/run — run one complete decision cycle
- POST /run — run one cycle and return to dashboard

The current runtime is a controlled Agent Company OS. Deterministic governance, economic execution, durable scheduling, memory, affiliate accounting, media processing, customer CRM, commercial receivables and a governed Resend email path are implemented. Real platform publishing, other external messaging providers, payment-rail execution, multi-user identity/RBAC and unsupervised portfolio control remain explicit integration/acceptance gates.

See docs/CAPABILITY_MATRIX.md for the strict capability boundary.


## Verification
Run `bash scripts/verify.sh` after cloning to execute formatting, workspace tests and Clippy gates.


## Operating model

The revenue-first autonomous operating loop and production boundaries are documented in `docs/AUTONOMOUS_OPERATING_MODEL.md`.
