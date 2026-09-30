# Veridara AI — Capability Matrix

**Core platform:** Autonomous Company OS  
**Positioning:** AI-native operating company with controlled autonomy

## Strict production review

This document intentionally separates implemented behavior from contracts, simulations and environment-dependent integrations.

| Capability | Current status | What is actually true |
|---|---|---|
| Agent decision cycle | Implemented | Agents call the configured model, produce typed proposals, and pass deterministic Governor/execution checks. |
| Agent memory | Implemented | Durable per-agent memory exists with bounded size/retention semantics. |
| Agent model-call rate limiting | Implemented | Persistent rate windows are checked before model calls. |
| Governance / least privilege | Implemented | Role/tool matrix, proposal validation and re-governance are enforced deterministically. |
| Economic core | Implemented | Checked arithmetic, company state/runway guards and deterministic execution are present. |
| Double-entry ledger | Implemented | Ledger transactions/entries are immutable and database validation enforces balance/company/currency invariants. |
| Scheduler | Implemented | Database leases and replay-safe run tokens are used for recurring cycles. |
| Outbox | Implemented | Durable events have leases, bounded retries and signed HTTPS webhook delivery. |
| Web research / web-session relay | Environment-gated | Browser/web relay protocols and workers exist, but usable capability depends on configured relay/browser credentials. |
| Affiliate product discovery | Environment-gated | Mock, Awin and TikTok Shop adapters exist; real data requires operator credentials/contracts. |
| Affiliate attribution/reconciliation | Implemented + environment-gated | Click/conversion state and provider verification/accounting are durable; real provider verification requires provider data. |
| Media production | Implemented | Media jobs run through isolated FFmpeg/FFprobe QA in the media worker. |
| Publishing approval contract | Implemented | Publish intent, approval, lease, completion and revocation are durable and guarded. |
| TikTok publishing adapter | Implemented + webhook reconciliation, environment-gated | Approved TikTok intents can be executed through the Content Posting API adapter; publish IDs are persisted, signed TikTok webhooks are verified with replay protection, terminal outcomes reconcile idempotently, and status polling remains available as fallback. Real use requires valid TikTok authorization, app approval/audit and operator configuration. |
| TikTok LIVE engine | Implemented + durable event persistence + governed stream publisher | Session/event contracts, durable idempotent gift/comment/follow/share/like ingestion, gift-statement reconciliation, AI engagement modes and an optional FFmpeg RTMP(S) publisher with hot-reload overlay are implemented. Actual TikTok account transport, gift events and platform PK control remain provider/account gated. |
| External email/message sending | Implemented + environment-gated | Governed outbound email has durable approval/consent evidence, outbox execution, Resend provider idempotency, provider references and failure tracking; real sending still requires verified provider configuration. |
| External payment execution | Governed + environment-gated | Payment intents, explicit approval evidence, idempotent execution intents, provider adapter boundary, execution evidence and reconciliation linkage exist; only the deterministic simulated provider is enabled by default. Real bank/card/processor settlement remains a separately gated adapter. |
| Commercial proposals | Implemented | Durable proposal creation plus deterministic proposal lifecycle transitions and pipeline reads are implemented. External contract execution remains human-governed. |
| Sponsorship management | Implemented baseline | Deterministic sponsorship lifecycle transitions, bounded delivery evidence and durable outbox events are implemented; external contracting/settlement remain governed boundaries. |
| Invoicing | Implemented | Invoice creation, issuance and payment lifecycle are durable and idempotent. |
| Invoice accounting | Implemented | Issuance posts AR → revenue; payment posts cash → AR inside the same transaction. |
| Customer CRM | Implemented baseline | Idempotent customer creation and listing with lifecycle/status metadata are available. |
| HR / payroll economics | Implemented baseline | Employees, payroll obligations and accounting primitives exist; external payroll execution is not integrated. |
| Business-unit economics | Implemented baseline | Units and portfolio metrics exist; automatic capital allocation is still gated. |
| Portfolio autonomy | Policy implemented; execution gated | A deterministic evidence-backed reinvest/hold/reduce/close policy now exists with liquidity protection and hard allocation caps. It produces decisions only; persisted execution and real capital movement remain separately gated. |
| Human approval / material side effects | Implemented | Material publishing and other sensitive actions are designed to remain explicitly gated. |
| Control-plane authentication | Implemented baseline | Bearer token authentication is required by default for non-health endpoints, with constant-time comparison. |
| Multi-user identity / RBAC / SSO | NOT achieved | Authentication is a shared control-plane token, not an operator identity system. |
| Multi-tenant SaaS isolation | NOT achieved | The runtime is company-scoped by deployment configuration, not a full user/tenant authorization model. |
| Observability | Implemented baseline | Health/readiness and Prometheus-style counters exist. Distributed tracing/load/chaos acceptance is still environment-dependent. |
| Disaster recovery | Implemented baseline | Backup/restore drill automation exists; production-scale recovery evidence is still environment-dependent. |
| Model evaluation / routing | Evidence capture implemented; routing gate remains | Per-cycle agent evaluation evidence, governance decisions, confidence, evidence count, cost, expected revenue and observed outcomes are persisted. A production benchmark matrix, complexity router and hardware-aware selection loop are still required. |
| Autonomous hiring/payroll execution | NOT achieved | Economic primitives and proposals exist; real external hiring/payroll actions remain gated. |
| Autonomous company operation with no human | NOT achieved | The architecture is a controlled autonomy foundation. Real external credentials, platform adapters and production acceptance are still required. |

## Hard conclusion

The repository is substantially beyond a toy multi-agent demo: the strongest implemented areas are deterministic governance, economic state, durable persistence, recovery-oriented workers, affiliate accounting, media processing and now commercial receivables/CRM.

It should not be described as an AI company that can independently operate every external business function yet. The largest remaining capability gap is the side-effect layer: real platform publishing, communications, payment rails, identity/RBAC and evidence-backed portfolio control.

## Next implementation priority

1. Complete TikTok OAuth/token lifecycle management and durable background polling/renewal workers.
2. Complete operator identity/RBAC and tenant isolation; treat the current role/audit work as a control-plane foundation rather than SaaS-grade identity.
3. Add a production model benchmark matrix, complexity router and hardware-aware routing in shadow mode before increasing autonomy.
4. Add evidence-backed portfolio reinvest/close decisions across real business units, with hard capital limits and human approval for material allocation.
5. Promote payment execution from simulated provider to separately reviewed real-provider adapters only after reconciliation, idempotency and recovery acceptance tests pass.
6. Add production acceptance evidence: backups/restores, load/chaos, provider failure drills, security review and revenue reconciliation.
7. Treat verified revenue and cash outcomes—not task volume or simulated activity—as the release gate for higher autonomy.


### Recent operating-control improvements
- Basic monochrome control-plane dashboard: **Implemented**
- Revenue planning target visibility: **Implemented** (planning metric only)
- Workforce/payroll/business-unit operational visibility: **Implemented**
- Verified external revenue generation: **Environment-gated / not guaranteed**
- Autonomous external publishing, messaging and payment initiation: **Publishing adapter + reconciliation implemented for TikTok; messaging and payment initiation not achieved**


### Commercial lifecycle hardening

The commercial control plane now exposes a read-only pipeline view plus deterministic proposal and sponsorship transitions. Sponsorship delivery is bounded by the contracted value and emits durable outbox events. These APIs do not execute external contracts or payment settlement; those remain provider/reconciliation boundaries.


## Procurement / Vendor Lifecycle

- Vendor records: implemented
- Purchase request creation: implemented
- Explicit approval evidence: implemented
- Delivery evidence ledger: implemented
- Automatic vendor payment/settlement: not implemented; approval and delivery do not settle funds
- Autonomous supplier contracting: not implemented; external contracts remain human-governed


## Revenue period truth

- **Ledger-backed revenue periods:** Company OS now derives MTD, trailing-30-day, and lifetime revenue from immutable revenue ledger entries rather than using the cumulative company snapshot as a monthly progress proxy.
- **Evidence count:** The dashboard exposes the number of revenue transactions supporting the MTD figure.
- **Fail-closed display:** If the ledger query is unavailable, MTD displays as zero rather than silently presenting cumulative revenue as monthly revenue.


## Contribution-margin truth

- **Expense classification:** ledger expense accounts carry VARIABLE, FIXED, or UNCLASSIFIED cost class.
- **Fail-closed economics:** MTD contribution margin is only reported when all MTD expense entries are classified; otherwise the dashboard reports Incomplete and shows the unclassified expense amount.
- **No proxy margin:** cumulative snapshot revenue is not reused as a contribution-margin input.


## Affiliate reconciliation cockpit

- MTD affiliate commission is shown from persisted conversion records.
- MTD attributed commission is aggregated from persisted attribution records.
- MTD recorded payouts are shown separately; they are not represented as bank receipt unless backed by the payout/evidence flow.
- The dashboard exposes the reconciliation variance so attribution gaps cannot be hidden inside a single revenue number.


## Experiment engine

- **Deterministic experiment policy:** hypotheses must declare control/treatment, budget, minimum observations, duration, success threshold and kill threshold.
- **Fail-closed decisioning:** invalid specifications, negative spend, budget exhaustion, insufficient observations, success, underperformance and expiry resolve deterministically.
- **Persistent evidence:** experiment definitions and observations are company-scoped and persisted before terminal decisions are returned.
- **No fabricated outcomes:** the engine evaluates supplied observations; it does not invent traffic, orders, conversion, revenue or platform metrics.

## Compliance / Policy Intelligence

- **Versioned policy evidence:** company-scoped policy snapshots persist platform, jurisdiction, version, source reference, evidence hash, observed/effective timestamps and explicit rule flags.
- **Fail-closed decisioning:** missing, inactive, future, or mismatched policy snapshots resolve to UNKNOWN/human review; missing disclosure, unverified claims, prohibited products, fake engagement and simulcast are blocked when the declared policy requires it.
- **Side-effect gate:** TikTok publishing and LIVE publisher start/externally approved LIVE session creation require an exact policy snapshot key plus evidence flags and proceed only when the deterministic compliance decision is ALLOWED.
- **Auditability:** every compliance check is idempotently persisted and exposes 24-hour ALLOWED/REVIEW/BLOCKED/UNKNOWN counts on the command center.
- **Policy activation event:** activating a new version deactivates the prior active snapshot, rejects rollback to an older effective policy, and emits `POLICY_SNAPSHOT_ACTIVATED` for downstream watchers.
- **Current boundary:** no automatic policy scraping is claimed. Operators or a verified policy watcher must supply the policy snapshot and evidence; changing platform rules do not silently become trusted.

## Trend → Opportunity → Content loop

- **Evidence-gated trend ingestion:** trend signals require a source, evidence reference, timestamp, normalized scoring inputs, confidence and policy evidence before they can enter the growth loop.
- **Deterministic opportunity scoring:** trend signals are scored from velocity, audience fit, product fit, contentability and inverse competition; low-confidence/high-score signals remain monitored instead of being auto-pursued.
- **Durable opportunity ledger:** pursued trends create idempotent, company-scoped opportunities with the exact content economics and policy evidence used to generate them.
- **Content-plan bridge:** a pursued opportunity can be materialized once into the Content Factory as a Draft; this does not publish externally or claim reach/revenue.
- **TTFC evidence:** the growth ledger stores the first content creation timestamp and derives trend-to-content elapsed seconds so growth speed can be measured instead of inferred.
- **LIVE attention controller:** each accepted LIVE event is scored for purchase intent, objections, gifts, PK moments, engagement, explicit high-value viewer evidence and safety escalation; per-session response caps/cooldowns are persisted, with human escalation for safety signals.
- **Typed outbox events:** TREND_DETECTED, OPPORTUNITY_CREATED and CONTENT_CREATED provide durable downstream hand-off points without free-form agent chat.
- **Current boundary:** trend discovery still requires an external/verified signal source, and content analytics/publishing evidence must come from separately authenticated platform boundaries.

## Learning / Failure Ledger

- **Typed learning records:** P0.5 now defines durable, company-scoped learning/failure entries with source linkage, expected vs actual outcome, impact, confidence, root cause, corrective action, reusable rule, and a deterministic follow-up decision.
- **Evidence gate:** failure records require severity and all entries require an observed actual outcome; the library rejects empty evidence and invalid confidence values rather than inventing outcomes.
- **Idempotent identity:** the database enforces a company-scoped unique `entry_key`, preventing duplicate learning records for the same source event.
- **Write-path baseline:** P0.5 introduces no update/delete workflow; the current contract is append-oriented, while database-level immutability enforcement remains a later hardening step.
- **Integration boundary:** the domain crate and durable schema are implemented, but automatic ingestion from every experiment/content/LIVE workflow is still a subsequent integration step; P0.5 does not claim those upstream systems emit learning automatically yet.

## Content Factory

- **Creative planning contract:** implemented through the company-content domain with deterministic validation for hypothesis, audience, format, product/offer references, disclosure, expected cost, maximum loss, success metric and threshold.
- **Creative primitives:** each variant persists hook, first frame, emotion, pacing, scene count, text density, voice speed, product placement, CTA, comment trigger, music style and visual style.
- **Durable content ledger:** company-scoped PostgreSQL content items are persisted with status and explicit SCALE/ITERATE/PAUSE/KILL decisions.
- **Control-plane API:** authenticated GET/POST /api/content/items is available for creating and reviewing content plans.
- **Truth boundary:** content plans do not imply rendering, publishing, reach, conversion or revenue. Those outcomes require separate verified media, platform, analytics and attribution evidence.
- **Next integration:** connect content plans to media jobs, publish intents, verified analytics, experiments and the learning ledger.

- **Content lifecycle evidence:** status transitions are governed and published/measured states require persisted evidence references.
- **Content performance evidence:** company-scoped observations persist source/evidence hash, timestamp, sample/funnel metrics, spend, commission and contribution margin with idempotent observation keys.
- **Deterministic feedback:** content observations resolve to SCALE/ITERATE/KILL using the declared loss limit and success threshold; no platform metrics are fabricated.
- **Integration boundary:** verified analytics and publisher integrations must supply the evidence; the content API itself does not claim TikTok publishing or analytics connectivity.
