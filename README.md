# Company Agents

Autonomous Company OS — an AI-native operating system for building and operating a self-sustaining company.

The initial business model is an AI-native media and creator company. The deeper goal is a reusable economic operating system in which AI agents can research opportunities, create businesses, allocate capital, hire, execute, learn from outcomes, scale winners, shut down losers, and eventually enter distress or bankruptcy.

## Primary implementation

The production Company OS core is being built in **Rust**.

There is **no React and no Vue**. The operator dashboard is server-rendered HTML from the Rust service; no SPA or frontend build pipeline is required.

Python remains available for analytics/forecasting/ML where justified. FFmpeg remains isolated for media processing. Rust is the default for the core runtime and performance-sensitive workers.

## Core thesis

> Build a company that optimizes for sustainable free cash flow, not agent activity or vanity metrics.

## First vertical

AI-native media / creator factory:

Research → ideation → production → distribution → measurement → monetization → reinvestment.

## Current build order

Economic kernel → simulator → Rust agent runtime → governance → executive agents → media factory → monetization → controlled autonomy.

## Non-negotiable economic rules

1. Agents cannot create money.
2. Financial state is controlled by an immutable double-entry ledger.
3. Agents cannot modify their own permissions.
4. Every material action is auditable.
5. Capital allocation requires explicit authorization.
6. Views and followers are not substitutes for profit.
7. Failed businesses can be shut down.
8. If the company cannot satisfy mandatory obligations and no recovery plan exists, it enters liquidation/bankruptcy.

## Local development

Requirements:
- Rust toolchain
- Docker

Start infrastructure:

    docker compose up -d

Start the Company OS:

    cargo run -p company-os

Then open:

    http://localhost:8080

Run the Rust test suite:

    cargo test --workspace

The dashboard is intentionally minimal while the economic kernel and full agent runtime are being wired in.
