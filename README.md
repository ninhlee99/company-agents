# Company Agents

Autonomous Company OS — an AI-native operating system for building and operating a self-sustaining company.

The initial business model is an AI-native media and creator company. The deeper goal is a reusable economic operating system in which AI agents can research opportunities, create businesses, allocate capital, hire, execute, learn from outcomes, scale winners, shut down losers, and eventually enter distress or bankruptcy.

## Primary implementation

The production platform is being built in **Go**.

There is **no React and no Vue**. The operator dashboard is server-rendered by Go using html/template, keeping the runtime small and operationally simple.

Rust is optional and introduced only for measured CPU/memory-intensive components where it provides a clear advantage.

## Core thesis

> Build a company that optimizes for sustainable free cash flow, not agent activity or vanity metrics.

## First vertical

AI-native media / creator factory:

Research → ideation → production → distribution → measurement → monetization → reinvestment.

## Current build order

Economic kernel → simulator → Go agent runtime → governance → executive agents → media factory → monetization → controlled autonomy.

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
- Go
- Docker

Start infrastructure:

    docker compose up -d

Start the Company OS:

    go run ./cmd/company-os

Then open:

    http://localhost:8080

The current dashboard is intentionally minimal while the economic kernel and agent runtime are being wired in.
