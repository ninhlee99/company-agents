# Company Agents

Autonomous Company OS — an AI-native operating system for building and operating a self-sustaining company.

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

No paid API key is required.

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

## Runtime API

- GET / — operator dashboard
- GET /healthz — health check
- GET /api/agents — latest agent proposals and Governor decisions
- POST /api/run — run one complete decision cycle
- POST /run — run one cycle and return to dashboard

The current runtime is a safe agent/governance bootstrap. Real money, platform publishing and external account control remain behind later integration and acceptance gates.
