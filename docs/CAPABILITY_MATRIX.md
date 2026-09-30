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
| Forecast-vs-actual variance | Implemented baseline | Read-only FPA variance reports compare the latest ACTIVE forecast with same-period cash-flow observations. Missing actual evidence stays unavailable; no values are fabricated. |
| Unified event architecture | Implemented foundation | `company-domain` defines canonical event names and a versioned company event envelope with correlation/causation IDs, aggregate references, idempotency keys and validated payloads. `company-store` can persist the typed envelope idempotently through the durable outbox; existing producers are migrated incrementally. |
| Web research / web-session relay | Environment-gated | Browser/web relay protocols and workers exist, but usable capability depends on configured relay/browser credentials. |
| Affiliate product discovery | Environment-gated | Mock, Awin and TikTok Shop adapters exist; real data requires operator credentials/contracts. |
| Product freshness gate | Implemented foundation | Affiliate product search can enforce a strict source-evidence freshness window; stale, future-dated, missing or malformed timestamps are excluded when the window is enabled. |
| Competitor intelligence + content whitespace | Implemented foundation | Evidence-backed competitor observations and owned-content coverage can be evaluated deterministically into ranked whitespace gaps. The module excludes stale observations and never invents external competitor data. |
| Creator intelligence and creator/product matching | Implemented foundation | Company-scoped creator performance signals and product economics can be matched deterministically by specialty/category, audience quality, engagement, CTR, conversion, commission, rating, refunds, delivery reliability and freshness. No external creator data is inferred. |
| Ads decision engine | Implemented foundation | Caller-supplied campaign evidence is scored by incremental contribution margin after fees/refunds/production/ad spend, conversion lift, confidence and time-to-feedback. ROAS is diagnostic only; no ad platform execution or spend is performed by this layer. |
| Affiliate attribution/reconciliation | Implemented + environment-gated | Click/conversion state and provider verification/accounting are durable; real provider verification requires provider data. |
| Media production | Implemented | Media jobs run through isolated FFmpeg/FFprobe QA in the media worker. |
| Publishing approval contract | Implemented | Publish intent, approval, lease, completion and revocation are durable and guarded. |
| TikTok publishing adapter | Implemented + webhook reconciliation, environment-gated | Approved TikTok intents can be executed through the Content Posting API adapter; publish IDs are persisted, signed TikTok webhooks are verified with replay protection, terminal outcomes reconcile idempotently, and status polling remains available as fallback. Real use requires valid TikTok authorization, app approval/audit and operator configuration. |
| TikTok LIVE engine | Implemented + durable event persistence + governed stream publisher | Session/event contracts, durable idempotent gift/comment/follow/share/like ingestion, gift-statement reconciliation, AI engagement modes and an optional FFmpeg RTMP(S) publisher with hot-reload overlay are implemented. Actual TikTok account transport, gift events and platform PK control remain provider/account gated. |
| External email/message sending | Implemented + environment-gated | Governed outbound email has durable approval/consent evidence, outbox execution, Resend provider idempotency, provider references and failure tracking; real sending still requires verified provider configuration. |
| External payment execution | Governed + simulated boundary implemented | Payment intent creation, explicit approval, idempotency, simulated provider execution evidence and durable outbox lineage are implemented. Simulation never moves external funds or marks an invoice paid. Real bank/card/processor settlement remains separately gated and is not exposed as a live provider. |
| Commercial proposals | Implemented | Durable proposal creation plus deterministic proposal lifecycle transitions and pipeline reads are implemented. External contract execution remains human-governed. |
| Sponsorship management | Implemented baseline | Deterministic sponsorship lifecycle transitions, bounded delivery evidence and durable outbox events are implemented; external contracting/settlement remain governed boundaries. |
| Invoicing | Implemented | Invoice creation, issuance and payment lifecycle are durable and idempotent. |
| Invoice accounting | Implemented | Issuance posts AR → revenue; payment posts cash → AR inside the same transaction. |
| Customer CRM | Implemented baseline | Idempotent customer creation and listing with lifecycle/status metadata are available. |
| Customer intelligence / observed economics | Implemented foundation | Company-scoped customer value is summarized from durable invoices/payments/support cases: observed billed/paid amounts, outstanding balance, collection rate, payment recency and open case load. Predictive LTV is explicitly unavailable without cohort evidence. |
| HR / payroll economics | Implemented baseline | Employees, payroll obligations and accounting primitives exist; external payroll execution is not integrated. |
| Business-unit economics | Implemented baseline | Units and portfolio metrics exist; automatic capital allocation is still gated. |
| Portfolio classes | Implemented foundation | Capital candidates are deterministically classified as Scale, Maintain, Validate or Exit from bounded return/downside, confidence and evidence signals. The class annotates capital decisions; it does not authorize spend. |
| Portfolio autonomy | Policy implemented; execution gated | A deterministic evidence-backed reinvest/hold/reduce/close policy now exists with liquidity protection and hard allocation caps. It produces decisions only; persisted execution and real capital movement remain separately gated. |
| Human approval / material side effects | Implemented | Material publishing and other sensitive actions are designed to remain explicitly gated. |
| Control-plane authentication | Implemented baseline + named RBAC + audit + browser session | Bearer authentication is required by default for non-health endpoints, with constant-time comparison. Optional named principals can carry `admin`, `operator`, or `read-only` scopes and an explicit `company_id`; mismatched tenants are rejected. Legacy operator/read-only tokens remain supported. Auth ALLOW/DENY decisions are persisted to a company-scoped append-only audit log. HTML control-plane sessions use a signed HttpOnly cookie with SameSite=Strict and CSRF double-submit protection. |
| Company-owned relational isolation | Implemented hardening | Commercial, CRM, procurement, payment, LIVE, creator and task relationships now use company-aware composite foreign keys; store-layer ownership checks fail early with explicit errors. Migration validation rejects pre-existing cross-company references rather than silently repairing them. |
| Multi-user identity / RBAC / SSO | RBAC baseline; SSO not achieved | Optional named control-plane principals support `admin`, `operator`, and `read-only` scopes with fail-closed config validation, unique principal IDs/tokens and audited principal IDs. Credentials remain deployment configuration; directory/SSO-backed lifecycle and durable user identity are not implemented. |
| Multi-tenant SaaS isolation | Tenant-bound principal baseline; full SaaS isolation not achieved | Named bearer principals can be bound to an explicit `company_id`, and authenticated requests are rejected when the principal is bound to a different runtime company. Legacy shared tokens remain deployment-scoped; directory tenancy, per-tenant lifecycle and full SaaS isolation are not implemented. |
| Observability | Request correlation + trace context + bounded read-load smoke | Health/readiness and Prometheus-style counters exist. Protected control-plane requests emit a bounded `x-request-id` and normalized `x-trace-id`; structured control-plane spans carry the same correlation. CI's production-like restart smoke also runs 40 authenticated readiness reads with concurrency capped at 8. Target-topology load/chaos and external trace exporters remain environment-dependent. |
| Integration readiness | Implemented baseline | Authenticated control-plane clients can inspect deterministic readiness for LLM, affiliate, TikTok OAuth, LIVE, outbound email, browser session and compliance. Configured/authenticated/evidence-fresh remain separate signals; external reachability is not inferred. |
| Browser smoke acceptance | Implemented for simulation harness | Playwright smoke tests verify the React/Vite simulation UI renders, exposes `SIMULATION`/synthetic evidence metadata, and keeps mutating demo actions fail-closed by default. Production Rust dashboard acceptance still requires a provisioned Postgres environment. |
| Disaster recovery | Implemented baseline + restart + read-load smoke | Backup/restore drill automation exists, plus a production-like restart smoke that verifies service recovery/readiness after app/database/worker restarts and drives 40 concurrent authenticated read checks capped at 8 workers. Full production-scale recovery evidence is still environment-dependent. |
| Simulator multi-run statistical evaluation | Implemented baseline | Deterministic seeded runs now produce descriptive mean/stddev/95% intervals and p05/median/p95 for key economic outcomes plus simulated survival/bankruptcy rates. These intervals describe model-run variability, not real-world forecast confidence. |
| Model evaluation / routing | Evidence capture + shadow routing + benchmark harness | Per-cycle evaluation evidence is persisted. Shadow routing classifies task complexity, detects hardware tier and records a deterministic provider recommendation without changing actual provider selection. A CLI benchmark matrix now measures per-provider latency, success and JSON-object validity across Fast/Standard/Deep cases; active routing remains gated pending target-environment quality/cost/failure evidence. |
| Autonomous hiring/payroll execution | NOT achieved | Economic primitives and proposals exist; real external hiring/payroll actions remain gated. |
| Autonomous company operation with no human | NOT achieved | The architecture is a controlled autonomy foundation. Real external credentials, platform adapters and production acceptance are still required. |


## TikTok OAuth refresh durability

- **Durable refresh lease:** enabled refresh workers use the existing `scheduled_jobs` lease, so concurrent Company OS instances claim at most one refresh job at a time. Refresh failures release the job for bounded retry; successful/no-op cycles advance the next run.

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


### Commercial delivery reporting

- **Read-only report:** `/api/commercial/report` returns currency-scoped proposal, sponsorship-delivery and invoice collection metrics with explicit outstanding/overdue amounts. No future revenue is inferred from pipeline state.

### Commercial lifecycle hardening

- **Forecast-vs-actual cash flow:** `GET /api/fpa/forecast-variance` and `/fpa/variance` read the newest ACTIVE forecast and match the latest same-period cash-flow observation. Missing actual evidence remains `Unavailable`; variance is `actual net cash flow − forecast net cash flow` and is never treated as a forecast or guaranteed outcome.

- **Tenant-bound principals:** named RBAC principals must declare `company_id`; a principal bound to another company cannot authenticate against this runtime. This closes the identity→company boundary for the named-principal path without claiming full SaaS tenancy.

- **Named control-plane principals:** `CONTROL_PLANE_PRINCIPALS_JSON` replaces the shared-token role map when configured. Principal IDs are audited; raw credentials are never written to audit metadata. `read-only` is limited to GET/HEAD; `operator` and `admin` cover current mutating control-plane APIs.

- **Unified event contract:** `CompanyEventType` provides stable names for the P1 event set; `CompanyEventEnvelope` carries company, schema, aggregate, correlation/causation, idempotency and payload metadata. Store persistence keeps the existing durable outbox path.
- **Content producer migration:** `RENDERED → PUBLISHED` emits canonical `CONTENT_PUBLISHED` transactionally with the content status/evidence update; repeated publication is prevented by the content state machine.
- **Content creation producer migration:** growth-opportunity content creation now emits canonical `CONTENT_CREATED` through the typed envelope, preserving opportunity causation, policy evidence and the existing idempotency key.
- **Growth producer migration:** persisted growth trend detection now emits canonical `TREND_DETECTED` through the typed envelope, preserving company/trend identity, observed evidence and decision metadata.
- **Affiliate producer migration:** provider verification now emits canonical `COMMISSION_VERIFIED` through the typed envelope, preserving the existing verification idempotency key while keeping provider evidence separate from recognized revenue accounting.
- **Governance producer migration:** activating a verified policy snapshot now emits canonical `POLICY_CHANGED` through the typed envelope, preserving policy version/evidence lineage and the existing idempotency key.
- **Experiment producer:** terminal experiment decisions now emit `EXPERIMENT_COMPLETED` transactionally with the experiment observation/status, revenue-graph lineage and learning record. The event payload contains only recorded experiment evidence and the deterministic terminal decision; `CONTINUE` emits no completion event.

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
- **Fail-closed display:** If the ledger query is unavailable, MTD displays as **Unavailable** rather than zero; the dashboard never turns a data-access failure into a business fact.


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

## Emergency stop + autonomy budgets

- **Persistent emergency stop:** a company-scoped stop survives process restarts and blocks autonomous cycle side effects, TikTok publishing, LIVE starts and outbound email delivery while leaving audit/evidence/accounting paths available.
- **Daily hard budgets:** content publishes, ad spend, LIVE minutes, outbound messages and autonomous capital each have explicit daily ceilings. Zero is a valid hard block.
- **Atomic idempotent consumption:** budget consumption is checked and recorded inside a transaction with an immutable consumption ledger and durable outbox event.
- **Operational visibility:** Company OS exposes the stop state and configured daily caps; blocked actions return an explicit precondition failure rather than appearing successful.
- **Current boundary:** ad execution and fully automatic autonomous-capital execution are not yet present in the repository, so their budgets are control-plane reservations for the next execution adapters.

## Capital allocation planning

- **Evidence-backed candidate contract:** capital candidates carry an explicit evidence reference plus expected contribution, downside, capital required, time-to-feedback, reversibility, strategic value, confidence, evidence count and a hard allocation cap.
- **Deterministic portfolio plan:** the planner scores candidates using bounded return/risk/speed/quality inputs, respects company cash minus reserve and a discretionary budget, and allocates no more than hard caps.
- **Liquidity and stop gates:** emergency stop, non-operational company states and insufficient runway/evidence hold capital at zero; positive expected contribution is required before allocation.
- **Durable idempotency:** plan identity is deterministic per company + plan key and the complete input bundle is fingerprinted, so replay with different evidence fails closed instead of silently mutating a prior plan.
- **No cash movement:** this capability creates an auditable allocation plan and outbox event only. It does not move company cash or execute external investments/payments.

## TikTok OAuth / token lifecycle

- **Web OAuth flow:** Company OS can generate a TikTok Login Kit authorization URL, persist a one-time CSRF state hash, validate the callback state and registered redirect URI, and exchange the authorization code server-side.
- **Encrypted token storage:** access and refresh tokens are encrypted at rest with AES-256-GCM and company-scoped associated data; API responses never return raw tokens.
- **Refresh / rotation:** manual and background refresh are supported; the newly returned refresh token replaces the prior token so token rotation is preserved.
- **Revocation / reauth:** revocation and invalid-refresh handling move the connection to explicit `REVOKED` / `REAUTH_REQUIRED` states and leave an audit trail.
- **Publishing integration:** when `TIKTOK_OAUTH_ENABLED=true`, Content Posting API calls fetch the current token from the durable store and refresh it when near expiry; the legacy env-token path remains available when OAuth mode is disabled.
- **External prerequisites:** the TikTok developer app still needs the requested scopes, consent and an exact registered HTTPS redirect URI; Content Posting API production/public posting remains subject to TikTok's app approval/audit rules.

## Affiliate Attribution & Reconciliation

- **Three ledgers stay separate:** provider-reported commission, company-attributed commission, and recorded affiliate payout/cash receipt are exposed independently.
- **Variance directions:** reported-attributed, attributed-paid, and reported-paid variances are all explicit; no single variance is used as a proxy for the others.
- **Company scope:** reconciliation queries remain tenant/company scoped and month-to-date by ledger/provider timestamps.
- **Lineage:** affiliate graph edges preserve click/order/commission/payout evidence without silently inventing missing campaign/content provenance.

## Contribution Margin Accounting

- **Authoritative CM:** contribution margin remains `revenue - variable_cost`; category fields are explanatory and are not summed into CM again.
- **Separate categories:** platform fees, affiliate commissions, refunds/cancellations, production/AI, ad spend, fixed operating costs, and cash are exposed separately from revenue.
- **Classification boundary:** category values use explicit ledger account codes first and conservative account-name matching second; unmatched expenses remain in the unclassified bucket.
- **Cash boundary:** cash is read from the company-scoped `CASH` asset account and is never treated as revenue or contribution margin.

## Revenue Period Truth

- **Period semantics:** `RevenuePeriodMetrics` keeps month-to-date revenue, trailing-30-day revenue, lifetime revenue, a calendar-day monthly forecast, and a trailing-30-day monthly run-rate as separate values.
- **Target separation:** the configured monthly target is planning input and is never substituted for observed revenue.
- **Forecast boundary:** the monthly forecast is a deterministic calendar-day run-rate projection; its confidence field is time-coverage (elapsed-month) coverage, not a statistical guarantee.
- **No lifetime-vs-monthly comparison:** dashboard target progress uses only month-to-date observed revenue.

## Integration readiness

- **Trace context:** control-plane spans carry a normalized 32-hex trace ID from a valid W3C `traceparent`, or a deterministic SHA-256-derived fallback from `x-request-id`; successful responses return `x-request-id` and `x-trace-id`. No trace/request IDs are Prometheus labels.


- **Creator-product matching:** `POST /api/growth/creator-product-matches` accepts only caller-supplied company-scoped creator/product evidence. Results are deterministic fit/economic signals; only specialty/category-aligned, fresh and in-stock products are considered, and no external marketplace connectivity or commercial success is implied.


- **Competitor whitespace:** `POST /api/growth/competitor-whitespace` accepts only caller-supplied observations with source/evidence/timestamp metadata plus owned-content coverage. Results are deterministic priority signals; they are not claims about unobserved competitors or platform-wide market demand.

- **Control-plane request telemetry:** protected requests are correlated with `x-request-id`; auth/scope/CSRF denials increment dedicated counters and protected request latency is exposed as a low-cardinality gauge. No path/actor labels are exported to avoid cardinality and secret leakage.

- **Readiness API:** `GET /api/integrations/readiness` returns explicit `READY`, `CONFIGURED`, `NOT_CONFIGURED`, `ACTION_REQUIRED`, `GATED`, or `UNAVAILABLE` states.
- **Evidence boundary:** configuration and stored authentication are reported separately from provider reachability or business-outcome acceptance; no external success is inferred from environment variables alone.

## Control-plane audit

- **Control-plane audit:** authenticated/denied requests are durably recorded with company, coarse operator role, method/path, outcome, request ID, and non-secret bearer-token fingerprint. The system still does not provide multi-user identity, RBAC, or SSO.
- **Read-only audit feed:** `GET /api/control-plane/audit?limit=N` exposes recent company-scoped audit metadata to authenticated control-plane clients; the bearer token itself is never returned.
- **Coarse auth scope:** `CONTROL_PLANE_READ_TOKEN` may authorize only `GET/HEAD` requests; `CONTROL_PLANE_TOKEN` remains the operator credential for mutating control-plane actions. This narrows blast radius but is not multi-user RBAC.
- **Browser session:** `GET /auth/login` + `POST /auth/session` establish an 8-hour signed browser session when `CONTROL_PLANE_BROWSER_SECRET` is configured. Mutating HTML forms require a matching CSRF cookie/query token; `POST /auth/logout` clears both cookies. Browser sessions do not create per-user identity and remain shared operator sessions.

## Model routing (shadow)

- **Deterministic task classification:** model calls are classified as Fast, Standard, or Deep from agent role and bounded prompt size.
- **Hardware-aware recommendation:** local logical CPU count is mapped to Small/Medium/Large tiers.
- **Provider recommendation:** Fast favors configured local/mock providers; Deep on Small hardware favors configured remote API providers.
- **Safe rollout:** `MODEL_ROUTER_MODE=shadow` is advisory telemetry only. The existing provider/fallback order remains authoritative until benchmark and acceptance evidence justify active routing.
- **Benchmark harness:** `cargo run -p agent-runtime --bin model-benchmark` runs the Fast/Standard/Deep matrix against providers named in `LLM_BENCHMARK_PROVIDERS` (default `mock`) and outputs machine-readable observations. Latency/format evidence is collected without changing routing.
- **CI smoke gate:** the Rust workflow runs the benchmark against `mock` on every push/PR and uploads the JSON report, proving the benchmark target remains buildable without external credentials.
- **Current boundary:** the harness does not claim provider quality or cost superiority; active routing requires target-environment evidence for quality, cost, latency and failure behavior.

## Revenue Intelligence Graph

- **Evidence-backed lineage:** immutable, company-scoped edges can connect content, hooks, audience, traffic, orders, products, commissions, experiments, decisions, trends and cash settlement with evidence references and confidence.
- **Operational reads:** Company OS exposes graph summary and bounded forward lineage queries; graph data is visible as unavailable rather than silently treated as zero.
- **Automatic write points:** affiliate click/conversion/verification/payout, growth-created content, and experiment decisions can create graph edges transactionally with idempotent identity.
- **Integrity controls:** deterministic edge IDs, company-scoped uniqueness, evidence requirements, bounded numeric values and append-only triggers prevent silent mutation.
- **Current boundary:** the graph does not invent missing campaign/creator data and does not retroactively backfill historical records; existing events must be replayed through authenticated/imported source boundaries to populate lineage.

## Trend → Opportunity → Content loop

- **Evidence-gated trend ingestion:** trend signals require a source, evidence reference, timestamp, normalized scoring inputs, confidence and policy evidence before they can enter the growth loop.
- **Deterministic opportunity scoring:** trend signals are scored from velocity, audience fit, product fit, contentability and inverse competition; low-confidence/high-score signals remain monitored instead of being auto-pursued.
- **Durable opportunity ledger:** pursued trends create idempotent, company-scoped opportunities with the exact content economics and policy evidence used to generate them.
- **Content-plan bridge:** a pursued opportunity can be materialized once into the Content Factory as a Draft; this does not publish externally or claim reach/revenue.
- **TTFC evidence:** the growth ledger stores the first content creation timestamp and derives trend-to-content elapsed seconds so growth speed can be measured instead of inferred.
- **LIVE attention controller:** each accepted LIVE event is scored for purchase intent, objections, gifts, PK moments, engagement, explicit high-value viewer evidence and safety escalation; per-session response caps/cooldowns are persisted, with human escalation for safety signals.
- **LIVE learning integration:** safety escalations and high-priority responses persist a bounded policy-learning record transactionally; the record carries event/session evidence identifiers and does not recognize gifts or engagement as revenue.
- **Typed outbox events:** trend/opportunity/content events plus verified commission, affiliate conversion reconciliation, affiliate payout settlement, Agent decisions, and publish completions now use canonical typed envelopes for durable downstream hand-off without free-form agent chat.
- **Current boundary:** trend discovery still requires an external/verified trend signal source, and content analytics/publishing evidence must come from separately authenticated platform boundaries.

- **Affiliate payout event:** verified payout settlement emits `AFFILIATE_PAYOUT_SETTLED` with deterministic correlation, ledger transaction ID, currency, amount and remaining recognized receivable evidence; no payout is recognized beyond the existing ledger gate.

- **Agent decision event:** `AGENT_DECISION_RECORDED` carries cycle/proposal identity, agent/action, Governor decision/reason and execution evidence with deterministic correlation and idempotency.

- **Affiliate reconciliation event:** `AFFILIATE_CONVERSION_RECONCILED` carries conversion/order/product identity, reconciled outcome, recognized amount, variance and ledger transaction evidence under the existing conversion idempotency key.

- **Publish completion event:** `PUBLISH_INTENT_COMPLETED` unifies direct publish completion and TikTok webhook reconciliation under the same envelope boundary; external reference/status/error evidence remain source-labeled and do not imply delivery success beyond the verified provider response.

## Agent outcome evaluation

- **Outcome evidence ledger:** one immutable, company-scoped evidence record can be attached to an executed, Governor-approved decision journal entry; it carries an evidence reference plus observed revenue and contribution-margin deltas.
- **Deterministic agent scorecards:** the evaluation layer reports proposal/approval/execution counts, observed spend, projected revenue, evidence coverage, projected return and observed return without assigning revenue to agents from timing or correlation alone.
- **Evidence states:** `InsufficientEvidence`, `PartialEvidence`, and `Evaluated` describe evidence coverage; they are not an autonomous ranking of agents.
- **APIs:** `POST /api/agents/outcome-evidence` records explicit outcome evidence and `GET /api/agents/evaluation` returns the last-30-day scorecards by default.
- **Current boundary:** no automatic causal attribution from content, LIVE, affiliate or ledger revenue to an agent is claimed; a verified system must explicitly attach the outcome evidence to a decision.

## CEO Revenue Command Center

- **Ledger-backed executive view:** combines authoritative MTD/30d/lifetime revenue, contribution margin evidence, affiliate reconciliation, affiliate orders/net order value, content funnel, LIVE gift pulse, policy readiness, growth opportunities and cash/runway in one read model.
- **Consistent windows:** content metrics are evaluated on the latest seven-day observation per content item; LIVE pulse uses the last 30 days; affiliate order economics and reconciliation use the current month; revenue trend uses a complete seven-day daily series.
- **Exception-first triage:** deterministic `Needs attention`, `Opportunities`, and `Healthy` sections surface liquidity, margin evidence, reconciliation, policy and growth states without inventing missing data.
- **Truth boundaries:** gift value is explicitly not recognized company revenue; content commission is labeled as observed commission; missing command-center data renders as unavailable rather than zero.
- **API:** authenticated `GET /api/ceo/command-center` exposes the same deterministic decision view used by the dashboard.

## Autonomy ladder / Digital Twin

- **Six-level control ladder:** Observe → Recommend → Simulate → Human approve → Limited autonomy → Strategic autonomy.
- **Deterministic gate:** emergency stop blocks autonomous side effects; distress/emergency/liquidation/bankruptcy caps autonomy at recommendation/simulation; low confidence or insufficient evidence stays recommendation-only; material/external/critical-risk actions require human approval.
- **Digital Twin:** proposed actions are replayed through the pure execution model against the authoritative snapshot without mutating live state; simulated cash/runway/status/downside are returned as evidence.
- **Limited autonomy conditions:** configured ceiling must permit it, action must be reversible, cost must stay under cap, confidence/evidence gates must pass, and the Digital Twin must show non-negative cash, no bankruptcy risk and sufficient runway.
- **Durable simulations:** autonomy assessments are persisted idempotently in an append-only, company-scoped ledger and emit `AUTONOMY_ASSESSMENT_RECORDED`.
- **Safe default:** `AUTONOMY_MAX_LEVEL=SIMULATE`; strategic autonomy is disabled unless explicitly configured. External/material side effects remain human-gated.

## Learning / Failure Ledger

- **Typed learning records:** P0.5 now defines durable, company-scoped learning/failure entries with source linkage, expected vs actual outcome, impact, confidence, root cause, corrective action, reusable rule, and a deterministic follow-up decision.
- **Evidence gate:** failure records require severity and all entries require an observed actual outcome; the library rejects empty evidence and invalid confidence values rather than inventing outcomes.
- **Idempotent identity:** the database enforces a company-scoped unique `entry_key`, preventing duplicate learning records for the same source event.
- **Append-only enforcement:** learning entries are append-only at the database layer; UPDATE/DELETE attempts are rejected by a trigger.
- **Outcome integrations:** terminal experiment decisions and content observations now persist evidence-backed learning entries plus transactional `LEARNING_ENTRY_RECORDED` outbox events in the same database transaction. Material LIVE attention decisions (safety escalation or priority ≥90) are also persisted as learning/near-miss evidence; low-signal LIVE events remain out of the learning ledger.

## Content Factory

- **Creative planning contract:** implemented through the company-content domain with deterministic validation for hypothesis, audience, format, product/offer references, disclosure, expected cost, maximum loss, success metric and threshold.
- **Creative primitives:** each variant persists hook, first frame, emotion, pacing, scene count, text density, voice speed, product placement, CTA, comment trigger, music style and visual style.
- **Durable content ledger:** company-scoped PostgreSQL content items are persisted with status and explicit SCALE/ITERATE/PAUSE/KILL decisions.
- **Control-plane API:** authenticated GET/POST /api/content/items is available for creating and reviewing content plans.
- **Truth boundary:** content plans do not imply rendering, publishing, reach, conversion or revenue. Those outcomes require separate verified media, platform, analytics and attribution evidence.
- **Learning integration:** measured content observations now feed the learning ledger transactionally. Verified analytics/publisher evidence is still required at the content-observation boundary; LIVE learning remains separate.

- **Content lifecycle evidence:** status transitions are governed and published/measured states require persisted evidence references.
- **Content performance evidence:** company-scoped observations persist source/evidence hash, timestamp, sample/funnel metrics, spend, commission and contribution margin with idempotent observation keys.
- **Deterministic feedback:** content observations resolve to SCALE/ITERATE/KILL using the declared loss limit and success threshold; no platform metrics are fabricated.
- **Integration boundary:** verified analytics and publisher integrations must supply the evidence; the content API itself does not claim TikTok publishing or analytics connectivity.
