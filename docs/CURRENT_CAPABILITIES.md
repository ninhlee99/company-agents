# Current Company OS Capabilities

## Runtime and governance

The Company OS currently provides a Rust-first control plane with bounded concurrent agents, deterministic Governor validation, context-aware action envelopes, proposal cost/risk caps, fail-closed model handling, durable Agent memory, persistent model-call rate windows, and deterministic execution for the small set of safe economic/operational actions.

The LLM is advisory only. It cannot grant itself permissions, exceed action caps, bypass the Governor, or directly execute external/material side effects.

## Durable company state

PostgreSQL is the durable source of truth for:

- company state snapshots;
- ledger transactions and entries;
- budgets;
- agent runs;
- idempotency keys;
- decision journal;
- audit log;
- transactional outbox;
- recurring scheduler jobs;
- agent memory and rate windows;
- affiliate clicks, conversions, attribution and payouts;
- media jobs;
- business-unit portfolio snapshots.

State mutation is transactional where the operation crosses economic, journal, audit and outbox boundaries.

## Economic engine

The system supports:

- exact integer money arithmetic;
- balanced double-entry transactions;
- budget allocation and bounded experiment spend;
- cash, revenue, expense, liability and asset state;
- runway and distress status;
- bankruptcy/liquidation guardrails;
- creator and affiliate business-unit P&L;
- recurring payroll obligations;
- contract obligations;
- company portfolio cash synchronization.

## Affiliate intelligence

The affiliate engine supports provider-neutral product discovery with:

- category and keyword filters;
- currency and price filters;
- minimum commission;
- coupon requirement and coupon validity dates;
- minimum rating/review count;
- stock filters;
- source freshness filtering;
- deterministic deduplication;
- confidence scoring;
- Wilson lower-bound quality estimation;
- fixed-commission normalization when currencies match;
- deterministic ranking.

Supported provider paths:

- local Mock provider for offline tests;
- Awin product feeds;
- Awin promotion/voucher data;
- Awin commission-group enrichment;
- TikTok Shop Creator Showcase product discovery;
- composite cross-provider aggregation.

Verified affiliate conversion revenue is booked as receivable + revenue. Verified payout settlement moves receivable to cash and is idempotent.

## Media

The media pipeline supports isolated FFmpeg/FFprobe execution with:

- workspace-bound path validation;
- traversal rejection;
- symlink escape protection;
- configurable FFmpeg/FFprobe executables;
- file/duration/codec/audio QA;
- durable jobs and retries;
- terminal QA failure handling.

## Scheduler and recovery

Recurring agent cycles use:

- database-backed scheduling;
- row locking with SKIP LOCKED;
- lease-based crash recovery;
- replay tokens;
- deterministic cycle idempotency;
- failure retry;
- cycle health tracking.

The outbox has leases, attempts, exponential retry and structured acknowledgement.

Backup/recovery scripts and a documented restore drill are included.

## Observability

Company OS exposes:

- structured JSON logs;
- Prometheus-style metrics;
- cycle success/failure counters;
- affiliate search counters;
- last cycle latency;
- durable decision journal and audit history.

## External side-effect boundary

The runtime intentionally does NOT auto-send external messages, publish irreversible content, hire people, or initiate payments merely because an agent/Governor proposes them.

Those operations remain gated behind an explicit, auditable authorization layer that still needs provider-specific contracts, human/organizational policy and production recovery drills.

## What is not yet production-certified

The repository is not labeled fully autonomous production. Remaining certification work includes:

- real-world provider credential verification and scope testing;
- secret manager integration/rotation;
- distributed tracing and alerting;
- multi-run simulator calibration against observed business data;
- provider-specific affiliate attribution reconciliation at scale;
- creator/content publishing adapters with explicit authorization;
- sponsorship/service CRM and invoice lifecycle;
- payroll execution adapters;
- disaster-recovery rehearsal against production-like databases;
- broader load tests and soak tests;
- formal compliance/legal review for each market and platform.

These are certification gates, not hidden autonomous behavior.
