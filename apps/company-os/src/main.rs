mod publishing;
mod live;
use affiliate_intelligence::{
    search as search_affiliate, AffiliateProvider, AggregateAffiliateProvider, AwinProvider,
    MockProvider, ProductSearchQuery, SearchResponse, TikTokShopProvider,
};
use agent_runtime::{model_from_env, AgentRunResult, AgentRuntime, CompanySnapshot};
use axum::{
    extract::{Form, Query, State, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use company_store::{CompanyStore, PersistedCycle};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock};
use subtle::ConstantTimeEq;
use tiktok_live_streaming::LiveStreamController;

#[derive(Default)]
struct RuntimeMetrics {
    cycles_started_total: AtomicU64,
    cycles_succeeded_total: AtomicU64,
    cycles_failed_total: AtomicU64,
    affiliate_search_total: AtomicU64,
    affiliate_search_failed_total: AtomicU64,
    last_cycle_latency_ms: AtomicU64,
}

impl RuntimeMetrics {
    fn render_prometheus(&self) -> String {
        format!(
            concat!(
                "# HELP company_cycles_started_total Agent cycles started.\n",
                "# TYPE company_cycles_started_total counter\n",
                "company_cycles_started_total {}\n",
                "# HELP company_cycles_succeeded_total Agent cycles completed successfully.\n",
                "# TYPE company_cycles_succeeded_total counter\n",
                "company_cycles_succeeded_total {}\n",
                "# HELP company_cycles_failed_total Agent cycles failed.\n",
                "# TYPE company_cycles_failed_total counter\n",
                "company_cycles_failed_total {}\n",
                "# HELP affiliate_search_total Affiliate searches received.\n",
                "# TYPE affiliate_search_total counter\n",
                "affiliate_search_total {}\n",
                "# HELP affiliate_search_failed_total Affiliate searches rejected or failed.\n",
                "# TYPE affiliate_search_failed_total counter\n",
                "affiliate_search_failed_total {}\n",
                "# HELP company_cycle_last_latency_ms Last successful cycle latency in milliseconds.\n",
                "# TYPE company_cycle_last_latency_ms gauge\n",
                "company_cycle_last_latency_ms {}\n"
            ),
            self.cycles_started_total.load(Ordering::Relaxed),
            self.cycles_succeeded_total.load(Ordering::Relaxed),
            self.cycles_failed_total.load(Ordering::Relaxed),
            self.affiliate_search_total.load(Ordering::Relaxed),
            self.affiliate_search_failed_total.load(Ordering::Relaxed),
            self.last_cycle_latency_ms.load(Ordering::Relaxed),
        )
    }
}

#[derive(Clone)]
struct AppState {
    runtime: Arc<AgentRuntime>,
    company: Arc<RwLock<CompanySnapshot>>,
    latest: Arc<RwLock<Vec<AgentRunResult>>>,
    latest_cycle: Arc<RwLock<Option<PersistedCycle>>>,
    store: Arc<CompanyStore>,
    affiliate: Arc<dyn AffiliateProvider>,
    cycle_lock: Arc<Mutex<()>>,
    company_id: String,
    currency: String,
    metrics: Arc<RuntimeMetrics>,
    live_stream: Option<Arc<LiveStreamController>>,
}

#[derive(Debug, Serialize)]
struct CycleResponse {
    snapshot: CompanySnapshot,
    results: Vec<AgentRunResult>,
    receipts: Vec<company_execution::ExecutionReceipt>,
}

#[derive(Debug, Deserialize)]
struct PublishApproveRequest {
    intent_id: String,
    approved_by: String,
    ttl_seconds: i64,
}
#[derive(Debug, Deserialize)]
struct PublishClaimRequest {
    intent_id: String,
    approval_token: String,
    lease_seconds: i64,
}
#[derive(Debug, Deserialize)]
struct PublishCompleteRequest {
    intent_id: String,
    execution_token: String,
    success: bool,
    external_reference: Option<String>,
    error_message: Option<String>,
}
#[derive(Debug, Deserialize)]
struct PublishRevokeRequest {
    intent_id: String,
}

#[derive(Debug, Deserialize)]
struct AffiliateConversionRequest {
    event: affiliate_attribution::ConversionEvent,
    model: affiliate_attribution::AttributionModel,
}

#[derive(Debug, Deserialize)]
struct GrowthTrendRequest {
    signal: company_growth::TrendSignal,
}

#[derive(Debug, Deserialize)]
struct GrowthContentRequest {
    opportunity_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
struct ContentCreateRequest {
    brief: company_content::ContentBrief,
    variant: company_content::CreativeVariant,
}

#[derive(Debug, Deserialize)]
struct ContentObservationRequest {
    observation: company_content::ContentObservation,
}

#[derive(Debug, Deserialize)]
struct ContentStatusTransitionRequest {
    content_id: uuid::Uuid,
    next: company_content::ContentStatus,
    evidence_ref: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct AffiliateSearchParams {
    category: Option<String>,
    keywords: Option<String>,
    currency: Option<String>,
    min_price_minor: Option<i128>,
    max_price_minor: Option<i128>,
    min_commission_bps: Option<u32>,
    require_coupon: Option<bool>,
    require_coupon_code: Option<bool>,
    min_rating_bps: Option<u32>,
    min_reviews: Option<u64>,
    in_stock_only: Option<bool>,
    max_results: Option<usize>,
    as_of_date: Option<String>,
}

#[derive(Debug, Deserialize)] struct ServiceProposalRequest { customer_id: uuid::Uuid, title: String, currency: String, total_minor: i128, valid_until_epoch: i64, idempotency_key: String }
#[derive(Debug, Deserialize)] struct SponsorshipRequest { customer_id: uuid::Uuid, title: String, currency: String, committed_minor: i128 }
#[derive(Debug, Deserialize)] struct InvoiceLineRequest { description: String, quantity: u32, unit_price_minor: i128 }
#[derive(Debug, Deserialize)] struct InvoiceRequest { customer_id: uuid::Uuid, currency: String, due_epoch: i64, idempotency_key: String, lines: Vec<InvoiceLineRequest> }
#[derive(Debug, Deserialize)] struct InvoiceIssueRequest { invoice_id: uuid::Uuid }
#[derive(Debug, Deserialize)] struct PaymentReconciliationEvidenceRequest { invoice_id: uuid::Uuid, provider: String, provider_event_id: String, external_ref: Option<String>, amount_minor: i128, currency: String, observed_at_epoch: i64, evidence_hash: String }
#[derive(Debug, Deserialize)] struct InvoicePaymentRequest { invoice_id: uuid::Uuid, payment_id: uuid::Uuid, amount_minor: i128, occurred_at_epoch: i64, external_ref: Option<String> }
#[derive(Debug, Deserialize)] struct ProposalTransitionRequest { proposal_id: uuid::Uuid, status: commercial_sales::ProposalStatus }
#[derive(Debug, Deserialize)] struct SponsorshipTransitionRequest { sponsorship_id: uuid::Uuid, status: String }
#[derive(Debug, Deserialize)] struct SponsorshipDeliveryRequest { sponsorship_id: uuid::Uuid, delivered_minor: i128 }
#[derive(Debug, Deserialize)] struct CustomerRequest { name: String, email: Option<String>, external_ref: Option<String>, status: Option<String>, notes: Option<String>, idempotency_key: String }
#[derive(Debug, Deserialize)] struct CustomerSuccessTaskRequest { customer_id: uuid::Uuid, task_type:String, due_at_epoch:i64, owner:Option<String>, notes:Option<String>, idempotency_key:String }
#[derive(Debug, Deserialize)] struct CustomerSuccessCompleteRequest { task_id:uuid::Uuid, outcome:String }
#[derive(Debug, Deserialize)] struct VendorRequest { legal_name:String, contact_email:Option<String>, currency:String, tax_ref:Option<String>, idempotency_key:String }
#[derive(Debug, Deserialize)] struct PurchaseRequest { vendor_id:uuid::Uuid, title:String, currency:String, amount_minor:i128, requester:String, idempotency_key:String }
#[derive(Debug, Deserialize)] struct PurchaseApproveRequest { request_id:uuid::Uuid, approved_by:String, approval_reference:String }
#[derive(Debug, Deserialize)] struct VendorDeliveryRequest { purchase_request_id:uuid::Uuid, external_ref:Option<String>, received_at_epoch:i64, evidence_hash:String }

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\\', "&#92;")
}

fn format_minor(value: i128, currency: &str) -> String {
    let negative = value < 0;
    let absolute = value.unsigned_abs();
    let zero_decimal = matches!(
        currency.to_ascii_uppercase().as_str(),
        "VND" | "JPY" | "KRW" | "IDR"
    );
    let (whole, fraction, digits) = if zero_decimal {
        (absolute, 0, 0)
    } else {
        (absolute / 100, absolute % 100, 2)
    };
    let rendered = if digits == 0 {
        whole.to_string()
    } else {
        format!("{}.{:02}", whole, fraction)
    };
    let prefix = if negative { "-" } else { "" };
    format!("{}{} {}", prefix, currency.to_ascii_uppercase(), rendered)
}

fn seed_company(company_id: String) -> CompanySnapshot {
    CompanySnapshot {
        company_id,
        cash_minor: 100_000,
        revenue_minor: 25_000,
        expenses_minor: 15_000,
        liabilities_minor: 0,
        assets_minor: 100_000,
        runway_days: 90,
        status: economic_core::CompanyStatus::Active,
        budget_remaining_minor: 10_000,
        experiment_budget_minor: 1_000,
        content_cost_minor: 500,
        content_revenue_minor: 900,
        backlog: 8,
        capacity: 10,
        conversion_bps: 220,
        audience_growth_bps: 120,
        hiring_need: 0,
    }
}

async fn run_cycle(
    state: &AppState,
    cycle_id: &str,
) -> Result<CycleResponse, Box<dyn std::error::Error + Send + Sync>> {
    let _cycle_guard = state.cycle_lock.lock().await;
    let started = Instant::now();
    state
        .metrics
        .cycles_started_total
        .fetch_add(1, Ordering::Relaxed);

    let company = state.company.read().await.clone();

    let runtime_state: Arc<dyn agent_runtime::agent::AgentStateProvider> = state.store.clone();
    let results = state
        .runtime
        .run_all_with_state(company.clone(), Some(runtime_state))
        .await;

    let persisted = state
        .store
        .persist_and_execute_cycle_with_id(&company, &results, cycle_id)
        .await?;

    *state.company.write().await = persisted.snapshot.clone();
    *state.latest.write().await = persisted.results.clone();
    *state.latest_cycle.write().await = Some(persisted.clone());
    state
        .metrics
        .cycles_succeeded_total
        .fetch_add(1, Ordering::Relaxed);
    state.metrics.last_cycle_latency_ms.store(
        started.elapsed().as_millis().min(u64::MAX as u128) as u64,
        Ordering::Relaxed,
    );

    Ok(CycleResponse {
        snapshot: persisted.snapshot,
        results: persisted.results,
        receipts: persisted.receipts,
    })
}

fn affiliate_query(params: AffiliateSearchParams) -> ProductSearchQuery {
    let keywords = params
        .keywords
        .unwrap_or_default()
        .split_whitespace()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .take(12)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    ProductSearchQuery {
        category: params
            .category
            .map(|v| v.trim().to_owned())
            .filter(|v| !v.is_empty()),
        keywords,
        currency: params
            .currency
            .map(|v| v.to_ascii_uppercase())
            .filter(|v| v.len() == 3),
        min_price_minor: params.min_price_minor,
        max_price_minor: params.max_price_minor,
        min_commission_bps: params.min_commission_bps,
        require_coupon: params.require_coupon.unwrap_or(false),
        require_coupon_code: params.require_coupon_code.unwrap_or(false),
        min_rating_bps: params.min_rating_bps,
        min_reviews: params.min_reviews,
        in_stock_only: params.in_stock_only.unwrap_or(true),
        max_results: params.max_results.unwrap_or(20),
        as_of_date: params
            .as_of_date
            .or_else(|| Some(time::OffsetDateTime::now_utc().date().to_string())),
    }
}

#[derive(Debug, Deserialize, Default)]
struct LiveUiQuery {
    live_error: Option<String>,
    live_status: Option<String>,
}

fn live_feedback(query: &LiveUiQuery) -> String {
    let message = match query.live_error.as_deref() {
        Some("session_create_failed") => Some("Could not create the LIVE session. Check the selected mode and server logs."),
        Some("stream_not_enabled") => Some("LIVE publishing is not enabled. External publishing remains disabled until the required configuration is present."),
        Some("stream_start_failed") => Some("The LIVE stream could not be started. Check the publisher configuration and logs."),
        Some("stream_stop_failed") => Some("The LIVE stream could not be stopped cleanly. Check the publisher status and logs."),
        _ => None,
    };
    if let Some(message) = message {
        return format!(r#"<div class="card" style="border-color:#7f1d1d"><strong>LIVE action needs attention</strong><p class="muted">{}</p></div>"#, message);
    }
    match query.live_status.as_deref() {
        Some("session_created") => r#"<div class="card"><strong>LIVE session created</strong><p class="muted">The session is persisted. Creating a session does not publish externally.</p></div>"#.into(),
        Some("stream_started") => r#"<div class="card"><strong>LIVE stream started</strong><p class="muted">The governed publisher accepted the start request.</p></div>"#.into(),
        Some("stream_stopped") => r#"<div class="card"><strong>LIVE stream stopped</strong></div>"#.into(),
        _ => String::new(),
    }
}

async fn index(
    State(state): State<AppState>,
    Query(query): Query<LiveUiQuery>,
) -> Html<String> {
    let company = state.company.read().await.clone();
    let workforce = state.store.list_employees(&state.company_id).await.unwrap_or_default();
    let business_units = state.store.list_business_units(&state.company_id).await.unwrap_or_default();
    let payroll_due = state.store.payroll_due(&state.company_id, 100).await.unwrap_or_default();
    let latest = state.latest.read().await;
    let latest_cycle = state.latest_cycle.read().await;
    let mut rows = String::new();

    for item in latest.iter() {
        let decision = item
            .governance
            .as_ref()
            .map(|g| format!("{:?}", g.decision))
            .unwrap_or_else(|| "—".into());
        let execution = latest_cycle
            .as_ref()
            .and_then(|c| {
                c.receipts
                    .iter()
                    .find(|r| r.agent == item.agent && r.action == item.proposal.action)
            })
            .map(|r| format!("{:?}", r.status))
            .unwrap_or_else(|| "—".into());

        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td></tr>",
            item.agent.as_str(),
            decision,
            execution,
            item.proposal.action
        ));
    }

    if rows.is_empty() {
        rows.push_str(r#"<tr><td colspan="4">No cycle has run yet.</td></tr>"#);
    }

    let target_minor = std::env::var("MONTHLY_REVENUE_TARGET_MINOR")
        .ok()
        .and_then(|v| v.parse::<i128>().ok())
        .filter(|v| *v > 0)
        .unwrap_or_else(|| if state.currency.eq_ignore_ascii_case("VND") { 50_000_000 } else { 500_000 });
    let revenue_periods = state
        .store
        .revenue_period_metrics(&state.company_id)
        .await
        .unwrap_or_else(|error| {
            tracing::warn!(%error, "revenue period metrics unavailable");
            company_store::RevenuePeriodMetrics {
                month_to_date_minor: 0,
                last_30_days_minor: 0,
                lifetime_minor: 0,
                revenue_transaction_count: 0,
            }
        });
    let contribution_margin = state
        .store
        .contribution_margin_metrics(&state.company_id)
        .await
        .unwrap_or_else(|error| {
            tracing::warn!(%error, "contribution margin metrics unavailable");
            company_store::ContributionMarginMetrics {
                month_to_date_revenue_minor: 0,
                month_to_date_variable_cost_minor: 0,
                month_to_date_contribution_margin_minor: None,
                unclassified_expense_minor: 0,
                unclassified_expense_entry_count: 0,
                variable_cost_transaction_count: 0,
            }
        });
    let affiliate_reconciliation = state
        .store
        .affiliate_reconciliation_metrics(&state.company_id)
        .await
        .unwrap_or_else(|error| {
            tracing::warn!(%error, "affiliate reconciliation metrics unavailable");
            company_store::AffiliateReconciliationMetrics {
                reported_commission_mtd_minor: 0,
                attributed_commission_mtd_minor: 0,
                recorded_payout_mtd_minor: 0,
                variance_mtd_minor: 0,
                conversion_count_mtd: 0,
                verified_conversion_count_mtd: 0,
                partial_or_rejected_count_mtd: 0,
            }
        });
    let growth_opportunities = state
        .store
        .list_growth_opportunities(&state.company_id, 5)
        .await
        .unwrap_or_default();
    let mut growth_html = String::new();
    for record in &growth_opportunities {
        let score_pct = record.opportunity.score_bps / 100;
        let status = match record.status {
            company_growth::OpportunityStatus::Ready => "READY",
            company_growth::OpportunityStatus::ContentCreated => "CONTENT CREATED",
        };
        growth_html.push_str(&format!(
            r#"<div style="display:flex;justify-content:space-between;gap:12px;padding:10px 0;border-bottom:1px solid #26304a"><div><strong>{}</strong><div class="muted">{} · {}% confidence</div></div><div class="metric" style="font-size:18px">{}</div></div>"#,
            escape_html(&record.opportunity.title),
            status,
            record.opportunity.confidence_bps / 100,
            score_pct
        ));
    }
    if growth_html.is_empty() {
        growth_html.push_str(r#"<p class="muted">No evidence-backed opportunities have been accepted yet. Ingest a verified trend signal first.</p>"#);
    }

    let contribution_margin_label = contribution_margin
        .month_to_date_contribution_margin_minor
        .map(|value| format_minor(value, &state.currency))
        .unwrap_or_else(|| "Incomplete".into());
    let contribution_margin_detail = if contribution_margin.unclassified_expense_entry_count > 0 {
        format!(
            "{} unclassified expense across {} entries",
            format_minor(contribution_margin.unclassified_expense_minor, &state.currency),
            contribution_margin.unclassified_expense_entry_count
        )
    } else {
        format!(
            "{} variable-cost transactions",
            contribution_margin.variable_cost_transaction_count
        )
    };
    let target_pct = if revenue_periods.month_to_date_minor > 0 {
        ((revenue_periods.month_to_date_minor as f64 / target_minor as f64) * 100.0).round().min(999.0) as u64
    } else {
        0
    };
    let capacity_pct = if company.capacity > 0 {
        ((company.backlog.max(0) as f64 / company.capacity as f64) * 100.0).round().min(999.0) as u64
    } else { 0 };
    let agent_count = latest.len();
    let active_staff = workforce.iter().filter(|e| matches!(e.status, company_organization::EmployeeStatus::Active)).count();
    let due_payroll_count = payroll_due.len();
    let business_unit_count = business_units.len();
    let cycle_state = if latest_cycle.is_some() { "active" } else { "waiting" };

    let executed = latest_cycle
        .as_ref()
        .map(|c| {
            c.receipts
                .iter()
                .filter(|r| {
                    matches!(
                        r.status,
                        company_execution::ExecutionStatus::Executed
                            | company_execution::ExecutionStatus::Noop
                    )
                })
                .count()
        })
        .unwrap_or(0);

    Html(format!(
        r#"<!doctype html>
<html lang="en"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Company OS</title>
<style>
body{{font-family:Inter,system-ui,-apple-system,sans-serif;max-width:1400px;margin:0 auto;padding:28px;background:radial-gradient(900px 420px at 80% -10%,#2b1d58,transparent 60%),#070a12;color:#f8fafc}}
.card{{border:1px solid #26304a;border-radius:16px;padding:18px;margin:0 0 14px;background:#101624;box-shadow:0 18px 50px #0004}}
.grid{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px}}
.metric{{font-size:22px;font-weight:650;margin-top:6px}}
.progress{{height:8px;background:#eee;margin-top:10px}} .progress>span{{display:block;height:8px;background:#111}}
table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:9px;border-bottom:1px solid #e5e5e5;font-size:14px}}
button{{padding:10px 14px;border:0;border-radius:9px;background:linear-gradient(135deg,#8b5cf6,#6366f1);color:#fff;cursor:pointer;font-weight:700}}
small,.muted{{color:#94a3b8}} code{{background:#f3f3f3;padding:2px 4px}}
nav{{display:flex;gap:8px;flex-wrap:wrap;margin:0 0 18px}} nav a{{color:#94a3b8;text-decoration:none;padding:8px 10px;border-radius:8px}} nav a:hover{{background:#171e30;color:#fff}}
@media(max-width:800px){{.grid{{grid-template-columns:repeat(2,minmax(0,1fr))}}}} @media(max-width:520px){{.grid{{grid-template-columns:1fr}}}}
</style></head><body>
<header><h1>Veridara AI</h1><small>Autonomous Company OS · {}</small></header>
<nav><a href="/">Overview</a><a href="/api/agents">Agents</a><a href="/api/customers">Customers</a><a href="/api/employees">Workforce</a><a href="/api/business-units">Business units</a><a href="/api/journal">Audit</a></nav>
<div class="grid">
<div class="card"><small>Cash</small><div class="metric">{}</div></div>
<div class="card"><small>Revenue MTD</small><div class="metric">{}</div><small>Ledger evidence: {} revenue transactions</small></div>
<div class="card"><small>Monthly target</small><div class="metric">{}</div><div class="progress"><span style="width:{}%"></span></div><small>{}% of planning target · last 30d: {}</small></div>
<div class="card"><small>Runway</small><div class="metric">{} days</div></div>
</div>
<div class="card"><small>Contribution margin MTD</small><div class="metric">{}</div><small>{}</small></div>
<div class="card"><small>Affiliate reconciliation MTD</small><div class="metric">{}</div><small>variance · reported · attributed · paid: {} · {} · {} · {}</small></div>
<div class="card"><h2>Growth pipeline</h2><p class="muted">Evidence-backed trend signals become scored opportunities before any content plan is created.</p>{}</div>
<div class="grid"><div class="card"><small>Status</small><div class="metric">{:?}</div></div><div class="card"><small>Agent cycle</small><div class="metric">{}</div></div><div class="card"><small>Backlog / capacity</small><div class="metric">{}%</div></div><div class="card"><small>Agent results</small><div class="metric">{}</div></div></div>
<div class="grid"><div class="card"><small>Active workforce</small><div class="metric">{}</div></div><div class="card"><small>Payroll due</small><div class="metric">{}</div></div><div class="card"><small>Business units</small><div class="metric">{}</div></div><div class="card"><small>Operating loop</small><div class="metric">observe → act → learn</div></div></div>
<div class="card"><h2>Operate</h2>
<form method="post" action="/run"><button type="submit">Run one decision cycle</button></form>
<p><small>The LLM only proposes reasoning. Governor, execution policy, idempotency and persistent state remain deterministic.</small></p></div>
<div class="card"><h2>Agents</h2>
<table><tr><th>Agent</th><th>Governor</th><th>Execution</th><th>Action</th></tr>{}</table>
</div>
{}<div class="card"><h2>LIVE Command Center</h2><p><small>Server-rendered controls; external publishing stays gated by configuration and approval.</small></p><form method="post" action="/live/session"><select name="mode" style="padding:9px;width:100%"><option>SOLO</option><option>SHOPPING</option><option>GAME</option><option>STORY</option><option>MUSIC</option><option>PK</option><option>COHOST</option></select><input name="title" value="Veridara AI LIVE" style="margin-top:8px;padding:9px;width:100%;box-sizing:border-box"><p><button type="submit">Create session</button></p></form><div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap"><form method="post" action="/live/start"><button type="submit">Start stream</button></form><form method="post" action="/live/stop"><button type="submit">Stop stream</button></form><a href="/" style="color:#94a3b8">Refresh</a></div><div class="metric">Provider-gated</div></div><div class="card"><h2>Commerce</h2>
<p>Search live Awin feed data when <code>AFFILIATE_PROVIDER=awin</code>; local mock data is used by default.</p>
<small>Example: <code>/api/affiliate/search?category=electronics&amp;min_commission_bps=1500&amp;require_coupon=true</code></small>
</div>
</body></html>"#,
        state.company_id.clone(),
        format_minor(company.cash_minor, &state.currency),
        format_minor(revenue_periods.month_to_date_minor, &state.currency),
        revenue_periods.revenue_transaction_count,
        format_minor(target_minor, &state.currency),
        target_pct,
        target_pct,
        format_minor(revenue_periods.last_30_days_minor, &state.currency),
        contribution_margin_label,
        contribution_margin_detail,
        format_minor(affiliate_reconciliation.variance_mtd_minor, &state.currency),
        format_minor(affiliate_reconciliation.reported_commission_mtd_minor, &state.currency),
        format_minor(affiliate_reconciliation.attributed_commission_mtd_minor, &state.currency),
        format_minor(affiliate_reconciliation.recorded_payout_mtd_minor, &state.currency),
        growth_html,
        company.runway_days,
        company.status,
        cycle_state,
        capacity_pct,
        agent_count,
        active_staff,
        due_payroll_count,
        business_unit_count,
        rows,
        live_feedback(&query),
    ))
}

#[derive(Debug, Deserialize)]
struct LiveControlForm {
    mode: Option<String>,
    title: Option<String>,
}

async fn live_create_html(
    State(state): State<AppState>,
    Form(form): Form<LiveControlForm>,
) -> Redirect {
    let mode = form.mode.unwrap_or_else(|| "SOLO".into());
    let title = form.title.filter(|value| !value.trim().is_empty()).unwrap_or_else(|| "Veridara AI LIVE".into());
    let request = live::CreateSessionRequest {
        title,
        mode,
        room_id: std::env::var("TIKTOK_LIVE_ROOM_ID").ok().filter(|value| !value.trim().is_empty()),
        started_at_epoch: time::OffsetDateTime::now_utc().unix_timestamp(),
        approved_for_external_publish: false,
    };
    match live::create_session(State(state), Json(request)).await {
        Ok(_) => Redirect::to("/?live_status=session_created"),
        Err(_) => Redirect::to("/?live_error=session_create_failed"),
    }
}

async fn live_start_html(State(state): State<AppState>) -> Redirect {
    match live::start_stream(State(state)).await {
        Ok(_) => Redirect::to("/?live_status=stream_started"),
        Err(StatusCode::PRECONDITION_FAILED) => Redirect::to("/?live_error=stream_not_enabled"),
        Err(_) => Redirect::to("/?live_error=stream_start_failed"),
    }
}

async fn live_stop_html(State(state): State<AppState>) -> Redirect {
    match live::stop_stream(State(state)).await {
        Ok(_) => Redirect::to("/?live_status=stream_stopped"),
        Err(StatusCode::PRECONDITION_FAILED) => Redirect::to("/?live_error=stream_not_enabled"),
        Err(_) => Redirect::to("/?live_error=stream_stop_failed"),
    }
}

async fn run_html(State(state): State<AppState>) -> (StatusCode, Html<String>) {
    match run_cycle(&state, &uuid::Uuid::new_v4().to_string()).await {
        Ok(_) => (
            StatusCode::SEE_OTHER,
            Html(r#"<meta http-equiv="refresh" content="0; url=/" />"#.into()),
        ),
        Err(error) => {
            state
                .metrics
                .cycles_failed_total
                .fetch_add(1, Ordering::Relaxed);
            tracing::error!(error = %error, "manual agent cycle failed");
            let message = error.to_string();
            let _ = state
                .store
                .record_cycle_failure(&state.company_id, &message)
                .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("cycle failed safely; inspect logs".into()),
            )
        }
    }
}

async fn run_api(State(state): State<AppState>) -> Result<Json<CycleResponse>, StatusCode> {
    run_cycle(&state, &uuid::Uuid::new_v4().to_string())
        .await
        .map(Json)
        .map_err(|error| {
            state
                .metrics
                .cycles_failed_total
                .fetch_add(1, Ordering::Relaxed);
            tracing::error!(error = %error, "api agent cycle failed");
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state
                    .store
                    .record_cycle_failure(&state.company_id, &error.to_string())
                    .await;
            });
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn agents_api(State(state): State<AppState>) -> Json<Vec<AgentRunResult>> {
    Json(state.latest.read().await.clone())
}

async fn growth_trend_api(
    State(state): State<AppState>,
    Json(request): Json<GrowthTrendRequest>,
) -> Result<Json<(company_store::GrowthTrendRecord, Option<company_store::GrowthOpportunityRecord>)>, StatusCode> {
    if request.signal.company_id.to_string() != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .record_growth_trend(&request.signal)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn growth_opportunities_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<company_store::GrowthOpportunityRecord>>, StatusCode> {
    state
        .store
        .list_growth_opportunities(&state.company_id, 100)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn growth_content_api(
    State(state): State<AppState>,
    Json(request): Json<GrowthContentRequest>,
) -> Result<Json<company_store::ContentRecord>, StatusCode> {
    state
        .store
        .create_content_from_growth_opportunity(&state.company_id, request.opportunity_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn content_create_api(
    State(state): State<AppState>,
    Json(req): Json<ContentCreateRequest>,
) -> Result<Json<company_store::ContentRecord>, StatusCode> {
    let company_id = uuid::Uuid::parse_str(&state.company_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let item = company_content::ContentItem {
        id: uuid::Uuid::new_v4(),
        company_id,
        brief: req.brief,
        variant: req.variant,
        status: company_content::ContentStatus::Draft,
        decision: None,
    };
    state.store.create_content_item(&item)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn content_list_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<company_store::ContentRecord>>, StatusCode> {
    state.store.list_content_items(&state.company_id, 200)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn content_status_transition_api(
    State(state): State<AppState>,
    Json(request): Json<ContentStatusTransitionRequest>,
) -> Result<Json<company_store::ContentRecord>, StatusCode> {
    state.store.transition_content_status(
        &state.company_id,
        request.content_id,
        request.next,
        request.evidence_ref.as_deref(),
    ).await.map(Json).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn content_observation_api(
    State(state): State<AppState>,
    Json(request): Json<ContentObservationRequest>,
) -> Result<Json<company_store::ContentObservationRecord>, StatusCode> {
    if request.observation.company_id.to_string() != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.store.record_content_observation(&request.observation)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn affiliate_search_api(
    State(state): State<AppState>,
    Query(params): Query<AffiliateSearchParams>,
) -> Result<Json<SearchResponse>, StatusCode> {
    state
        .metrics
        .affiliate_search_total
        .fetch_add(1, Ordering::Relaxed);
    match search_affiliate(state.affiliate.clone(), affiliate_query(params)).await {
        Ok(value) => Ok(Json(value)),
        Err(error) => {
            state
                .metrics
                .affiliate_search_failed_total
                .fetch_add(1, Ordering::Relaxed);
            tracing::warn!(error = %error, "affiliate search failed");
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn affiliate_click_api(
    State(state): State<AppState>,
    Json(event): Json<affiliate_attribution::ClickEvent>,
) -> Result<StatusCode, StatusCode> {
    if event.company_id != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .record_affiliate_click(&event)
        .await
        .map(|_| StatusCode::ACCEPTED)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn affiliate_conversion_api(
    State(state): State<AppState>,
    Json(request): Json<AffiliateConversionRequest>,
) -> Result<Json<affiliate_attribution::ReconciledConversion>, StatusCode> {
    if request.event.company_id != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .record_affiliate_conversion(&request.event, request.model)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn affiliate_verify_api(
    State(state): State<AppState>,
    Json(request): Json<AffiliateProviderVerificationRequest>,
) -> Result<Json<affiliate_attribution::ReconciledConversion>, StatusCode> {
    if request.conversion_id.trim().is_empty() || request.verification_source.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .verify_affiliate_conversion(
            &state.company_id,
            &request.conversion_id,
            request.status,
            request.verified_commission_minor,
            request.verified_at.as_deref(),
            &request.verification_source,
        )
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn affiliate_payout_api(
    State(state): State<AppState>,
    Json(request): Json<AffiliatePayoutRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .store
        .record_affiliate_payout(
            &state.company_id,
            &request.payout_id,
            request.amount_minor,
            &request.currency,
            &request.occurred_at,
        )
        .await
        .map(|_| StatusCode::ACCEPTED)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn publish_intent_api(
    State(state): State<AppState>,
    Json(intent): Json<publishing_contract::PublishIntent>,
) -> Result<StatusCode, StatusCode> {
    if intent.company_id != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .create_publish_intent(&intent)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn publish_approve_api(
    State(state): State<AppState>,
    Json(request): Json<PublishApproveRequest>,
) -> Result<Json<publishing_contract::PublishApproval>, StatusCode> {
    state
        .store
        .approve_publish_intent(
            &state.company_id,
            &request.intent_id,
            &request.approved_by,
            request.ttl_seconds,
        )
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn publish_claim_api(
    State(state): State<AppState>,
    Json(request): Json<PublishClaimRequest>,
) -> Result<Json<Option<publishing_contract::PublishJob>>, StatusCode> {
    state
        .store
        .claim_publish_intent(
            &state.company_id,
            &request.intent_id,
            &request.approval_token,
            request.lease_seconds,
        )
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn publish_complete_api(
    State(state): State<AppState>,
    Json(request): Json<PublishCompleteRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .store
        .complete_publish_intent(
            &state.company_id,
            &request.intent_id,
            &request.execution_token,
            request.success,
            request.external_reference.as_deref(),
            request.error_message.as_deref(),
        )
        .await
        .map(|_| StatusCode::ACCEPTED)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn publish_revoke_api(
    State(state): State<AppState>,
    Json(request): Json<PublishRevokeRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .store
        .revoke_publish_intent(&state.company_id, &request.intent_id)
        .await
        .map(|_| StatusCode::ACCEPTED)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn affiliate_performance_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    state
        .store
        .content_affiliate_performance(&state.company_id, 50)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn journal_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    state
        .store
        .recent_journal(&state.company_id, 100)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn business_units_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<company_organization::BusinessUnit>>, StatusCode> {
    state
        .store
        .list_business_units(&state.company_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn portfolio_metrics_api(
    State(state): State<AppState>,
) -> Result<Json<company_organization::PortfolioMetrics>, StatusCode> {
    state
        .store
        .portfolio_metrics(&state.company_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn employees_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<company_organization::Employee>>, StatusCode> {
    state
        .store
        .list_employees(&state.company_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn payroll_due_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<company_organization::PayrollObligation>>, StatusCode> {
    state
        .store
        .payroll_due(&state.company_id, 100)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn service_proposal_api(State(state): State<AppState>, Json(req): Json<ServiceProposalRequest>) -> Result<StatusCode, StatusCode> {
    let proposal = commercial_sales::ServiceProposal {
        id: uuid::Uuid::new_v4(), company_id: uuid::Uuid::parse_str(&state.company_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        customer_id: req.customer_id, title:req.title, currency:req.currency, total_minor:req.total_minor,
        status:commercial_sales::ProposalStatus::Draft, valid_until_epoch:req.valid_until_epoch, idempotency_key:req.idempotency_key
    };
    state.store.create_service_proposal(&proposal).await.map(|_| StatusCode::CREATED).map_err(|_| StatusCode::BAD_REQUEST)
}
async fn sponsorship_api(State(state): State<AppState>, Json(req): Json<SponsorshipRequest>) -> Result<StatusCode, StatusCode> {
    let sponsorship = commercial_sales::Sponsorship {
        id:uuid::Uuid::new_v4(), company_id:uuid::Uuid::parse_str(&state.company_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        customer_id:req.customer_id,title:req.title,currency:req.currency,committed_minor:req.committed_minor,delivered_minor:0,status:"PROSPECT".into()
    };
    state.store.create_sponsorship(&sponsorship).await.map(|_| StatusCode::CREATED).map_err(|_| StatusCode::BAD_REQUEST)
}
async fn invoice_api(State(state): State<AppState>, Json(req): Json<InvoiceRequest>) -> Result<StatusCode, StatusCode> {
    let lines: Vec<commercial_sales::InvoiceLine> = req.lines.into_iter().map(|l| commercial_sales::InvoiceLine{description:l.description,quantity:l.quantity,unit_price_minor:l.unit_price_minor}).collect();
    let total=commercial_sales::invoice_total(&lines).map_err(|_| StatusCode::BAD_REQUEST)?;
    let invoice=commercial_sales::Invoice{id:uuid::Uuid::new_v4(),company_id:uuid::Uuid::parse_str(&state.company_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,customer_id:req.customer_id,currency:req.currency,subtotal_minor:total,paid_minor:0,status:commercial_sales::InvoiceStatus::Draft,due_epoch:req.due_epoch,idempotency_key:req.idempotency_key};
    state.store.create_invoice(&invoice,&lines).await.map(|_| StatusCode::CREATED).map_err(|_| StatusCode::BAD_REQUEST)
}
async fn invoice_issue_api(State(state): State<AppState>, Json(req): Json<InvoiceIssueRequest>) -> Result<StatusCode, StatusCode> {
    state.store.issue_invoice(&state.company_id,&req.invoice_id.to_string()).await.map(|_| StatusCode::ACCEPTED).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn commercial_pipeline_api(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.store.list_commercial_pipeline(&state.company_id, 100)
        .await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn proposal_transition_api(
    State(state): State<AppState>,
    Json(req): Json<ProposalTransitionRequest>,
) -> Result<StatusCode, StatusCode> {
    state.store.transition_service_proposal(
        &state.company_id,
        &req.proposal_id.to_string(),
        req.status,
    ).await.map(|_| StatusCode::ACCEPTED).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn sponsorship_transition_api(
    State(state): State<AppState>,
    Json(req): Json<SponsorshipTransitionRequest>,
) -> Result<StatusCode, StatusCode> {
    state.store.transition_sponsorship(
        &state.company_id,
        &req.sponsorship_id.to_string(),
        &req.status,
    ).await.map(|_| StatusCode::ACCEPTED).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn sponsorship_delivery_api(
    State(state): State<AppState>,
    Json(req): Json<SponsorshipDeliveryRequest>,
) -> Result<StatusCode, StatusCode> {
    state.store.record_sponsorship_delivery(
        &state.company_id,
        &req.sponsorship_id.to_string(),
        req.delivered_minor,
    ).await.map(|_| StatusCode::ACCEPTED).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn invoice_payment_api(State(state): State<AppState>, Json(req): Json<InvoicePaymentRequest>) -> Result<Json<commercial_sales::InvoiceStatus>, StatusCode> {
    state.store.record_invoice_payment(&state.company_id,&req.invoice_id.to_string(),&req.payment_id.to_string(),req.amount_minor,req.occurred_at_epoch,req.external_ref.as_deref()).await.map(Json).map_err(|_| StatusCode::BAD_REQUEST)
}


async fn payment_reconciliation_evidence_api(
    State(state): State<AppState>,
    Json(req): Json<PaymentReconciliationEvidenceRequest>,
) -> Result<Json<String>, StatusCode> {
    state.store.record_payment_reconciliation_evidence(
        &state.company_id, &req.invoice_id.to_string(), &req.provider,
        &req.provider_event_id, req.external_ref.as_deref(), req.amount_minor,
        &req.currency, req.observed_at_epoch, &req.evidence_hash,
    ).await.map(Json).map_err(|_| StatusCode::BAD_REQUEST)
}

fn control_plane_auth_disabled() -> bool {
    std::env::var("CONTROL_PLANE_AUTH_DISABLED")
        .ok()
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
}

async fn require_control_plane_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    if matches!(path, "/healthz" | "/readyz" | "/metrics" | "/api/publishing/tiktok/webhook") {
        return Ok(next.run(request).await);
    }
    if control_plane_auth_disabled() {
        return Ok(next.run(request).await);
    }

    let expected = std::env::var("CONTROL_PLANE_TOKEN").map_err(|_| StatusCode::UNAUTHORIZED)?;
    let provided = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty());

    let valid = provided
        .map(|value| bool::from(expected.as_bytes().ct_eq(value.as_bytes())))
        .unwrap_or(false);

    if valid {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}


async fn vendor_api(
    State(state): State<AppState>,
    Json(req): Json<VendorRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.store.create_vendor(
        &state.company_id, uuid::Uuid::new_v4(), &req.legal_name,
        req.contact_email.as_deref(), &req.currency, req.tax_ref.as_deref(),
        &req.idempotency_key,
    ).await.map(Json).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn vendors_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    state.store.list_vendors(&state.company_id, 200).await
        .map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn purchase_request_api(
    State(state): State<AppState>,
    Json(req): Json<PurchaseRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.store.create_purchase_request(
        &state.company_id, uuid::Uuid::new_v4(), req.vendor_id, &req.title,
        &req.currency, req.amount_minor, &req.requester, &req.idempotency_key,
    ).await.map(Json).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn purchase_approve_api(
    State(state): State<AppState>,
    Json(req): Json<PurchaseApproveRequest>,
) -> Result<StatusCode, StatusCode> {
    state.store.approve_purchase_request(
        &state.company_id, &req.request_id.to_string(), &req.approved_by, &req.approval_reference,
    ).await.map(|_| StatusCode::ACCEPTED).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn vendor_delivery_api(
    State(state): State<AppState>,
    Json(req): Json<VendorDeliveryRequest>,
) -> Result<StatusCode, StatusCode> {
    state.store.record_vendor_delivery(
        &state.company_id, uuid::Uuid::new_v4(), &req.purchase_request_id.to_string(),
        req.external_ref.as_deref(), req.received_at_epoch, &req.evidence_hash,
    ).await.map(|_| StatusCode::ACCEPTED).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn customer_api(
    State(state): State<AppState>,
    Json(req): Json<CustomerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = req.status.unwrap_or_else(|| "LEAD".into());
    state.store.create_customer(
        &state.company_id,
        uuid::Uuid::new_v4(),
        &req.name,
        req.email.as_deref(),
        req.external_ref.as_deref(),
        &status,
        req.notes.as_deref(),
        &req.idempotency_key,
    ).await.map(Json).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn customers_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    state.store.list_customers(&state.company_id, 200).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, &'static str) {
    match state.store.ready(&state.company_id).await {
        Ok(()) => (StatusCode::OK, "ready"),
        Err(error) => {
            tracing::warn!(error = %error, "Company OS readiness check failed");
            (StatusCode::SERVICE_UNAVAILABLE, "not ready")
        }
    }
}

async fn metrics(
    State(state): State<AppState>,
) -> (
    StatusCode,
    [(axum::http::HeaderName, &'static str); 1],
    String,
) {
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        state.metrics.render_prometheus(),
    )
}

fn runtime_worker_threads() -> usize {
    std::env::var("TOKIO_WORKER_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| (1..=64).contains(value))
        .unwrap_or_else(|| std::thread::available_parallelism().map(|value| value.get().min(8)).unwrap_or(2))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let worker_threads = runtime_worker_threads();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()?
        .block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let company_id = std::env::var("COMPANY_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".into());
    let company_name = std::env::var("COMPANY_NAME").unwrap_or_else(|_| "Demo Company".into());
    let currency = std::env::var("COMPANY_CURRENCY").unwrap_or_else(|_| "USD".into());

    let store = Arc::new(CompanyStore::connect(&database_url).await?);
    store.migrate().await?;
    store
        .ensure_company(&company_id, &company_name, &currency)
        .await?;

    let company = match store.load_snapshot(&company_id).await? {
        Some(snapshot) => snapshot,
        None => {
            let snapshot = seed_company(company_id.clone());
            store.save_snapshot(&snapshot).await?;
            snapshot
        }
    };

    let affiliate_names = std::env::var("AFFILIATE_PROVIDERS")
        .or_else(|_| std::env::var("AFFILIATE_PROVIDER"))
        .unwrap_or_else(|_| "mock".into());
    let strict_affiliate = std::env::var("AFFILIATE_STRICT_PROVIDERS")
        .ok()
        .is_some_and(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        });

    let mut affiliate_providers: Vec<Arc<dyn AffiliateProvider>> = Vec::new();

    for name in affiliate_names
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let provider = match name.to_ascii_lowercase().as_str() {
            "mock" => Ok(Arc::new(MockProvider::default()) as Arc<dyn AffiliateProvider>),
            "awin" => AwinProvider::from_env()
                .map(|provider| Arc::new(provider) as Arc<dyn AffiliateProvider>)
                .map_err(|error| error.to_string()),
            "tiktok" | "tiktok_shop" | "tiktok-shop" => TikTokShopProvider::from_env()
                .map(|provider| Arc::new(provider) as Arc<dyn AffiliateProvider>)
                .map_err(|error| error.to_string()),
            other => Err(format!("unknown affiliate provider {other}")),
        };

        match provider {
            Ok(provider) => affiliate_providers.push(provider),
            Err(error) if strict_affiliate => return Err(error.into()),
            Err(error) => tracing::warn!(provider = name, error = %error, "affiliate provider skipped"),
        }
    }

    if affiliate_providers.is_empty() {
        return Err("no usable affiliate providers configured".into());
    }

    let affiliate: Arc<dyn AffiliateProvider> =
        Arc::new(AggregateAffiliateProvider::new(affiliate_providers)?);

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,company_os=info".into()),
        )
        .try_init()
        .ok();

    let runtime = Arc::new(AgentRuntime::new(model_from_env()));
    let live_stream_enabled = std::env::var("TIKTOK_LIVE_ENABLED")
        .ok()
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"));
    let live_stream = if live_stream_enabled {
        Some(Arc::new(LiveStreamController::from_env().map_err(|error| format!("TikTok LIVE configuration error: {error}"))?))
    } else {
        None
    };
    let state = AppState {
        runtime,
        company: Arc::new(RwLock::new(company)),
        latest: Arc::new(RwLock::new(Vec::new())),
        latest_cycle: Arc::new(RwLock::new(None)),
        store: store.clone(),
        affiliate,
        cycle_lock: Arc::new(Mutex::new(())),
        company_id: company_id.clone(),
        currency: currency.clone(),
        metrics: Arc::new(RuntimeMetrics::default()),
        live_stream,
    };

    let interval_secs = std::env::var("AGENT_CYCLE_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v >= 15)
        .unwrap_or(300);
    store
        .ensure_recurring_job(&company_id, "agent_cycle", interval_secs as i64)
        .await?;

    let background = state.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(15));
        loop {
            ticker.tick().await;
            let claimed = background
                .store
                .claim_due_job(&background.company_id, "agent_cycle")
                .await;

            match claimed {
                Ok(Some((job_id, run_token))) => {
                    match run_cycle(&background, &run_token.to_string()).await {
                        Ok(_) => {
                            if let Err(error) = background
                                .store
                                .complete_job(job_id, uuid::Uuid::new_v4())
                                .await
                            {
                                eprintln!("scheduler completion error: {error}");
                            }
                        }
                        Err(error) => {
                            eprintln!("agent cycle error: {error}");
                            let _ = background
                                .store
                                .record_cycle_failure(&background.company_id, &error.to_string())
                                .await;
                            if let Err(release_error) =
                                background.store.release_job_after_failure(job_id).await
                            {
                                eprintln!("scheduler recovery error: {release_error}");
                            }
                        }
                    }
                }
                Ok(None) => {}
                Err(error) => eprintln!("scheduler claim error: {error}"),
            }
        }
    });

    if !control_plane_auth_disabled() {
        let token = std::env::var("CONTROL_PLANE_TOKEN")
            .map_err(|_| "CONTROL_PLANE_TOKEN is required unless CONTROL_PLANE_AUTH_DISABLED=true")?;
        if token.len() < 32 {
            return Err("CONTROL_PLANE_TOKEN must be at least 32 bytes".into());
        }
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/run", post(run_html))
        .route("/live/session", post(live_create_html))
        .route("/live/start", post(live_start_html))
        .route("/live/stop", post(live_stop_html))
        .route("/api/run", post(run_api))
        .route("/api/agents", get(agents_api))
        .route("/api/content/items", get(content_list_api).post(content_create_api))
        .route("/api/content/observations", post(content_observation_api))
        .route("/api/content/status", post(content_status_transition_api))
        .route("/api/growth/trends", get(growth_trends_api).post(growth_trend_api))
        .route("/api/growth/opportunities", get(growth_opportunities_api))
        .route("/api/growth/content", post(growth_content_api))
        .route("/api/affiliate/search", get(affiliate_search_api))
        .route("/api/affiliate/click", post(affiliate_click_api))
        .route("/api/affiliate/conversion", post(affiliate_conversion_api))
        .route("/api/affiliate/verify", post(affiliate_verify_api))
        .route("/api/affiliate/payout", post(affiliate_payout_api))
        .route("/api/affiliate/performance", get(affiliate_performance_api))
        .route("/api/publishing/intents", post(publish_intent_api))
        .route("/api/publishing/intents/approve", post(publish_approve_api))
        .route("/api/publishing/intents/claim", post(publish_claim_api))
        .route("/api/publishing/intents/complete", post(publish_complete_api))
        .route("/api/publishing/intents/revoke", post(publish_revoke_api))
        .route("/api/publishing/tiktok/execute", post(publishing::execute_tiktok))
        .route("/api/publishing/tiktok/status", post(publishing::tiktok_status))
        .route("/api/publishing/tiktok/webhook", post(publishing::tiktok_webhook))
        .route("/api/live/sessions", post(live::create_session))
        .route("/api/live/sessions/:session_id/events", post(live::record_event))
        .route("/api/live/sessions/:session_id/summary", get(live::summary))
        .route("/api/live/sessions/:session_id/reconcile-gifts", post(live::reconcile_gifts))
        .route("/api/live/stream/start", post(live::start_stream))
        .route("/api/live/stream/overlay", post(live::update_overlay))
        .route("/api/live/stream/stop", post(live::stop_stream))
        .route("/api/live/stream/status", get(live::stream_status))
        .route("/api/customers", get(customers_api).post(customer_api))
        .route("/api/vendors", get(vendors_api).post(vendor_api))
        .route("/api/procurement/requests", post(purchase_request_api))
        .route("/api/procurement/requests/approve", post(purchase_approve_api))
        .route("/api/procurement/deliveries", post(vendor_delivery_api))
        .route("/api/employees", get(employees_api))
        .route("/api/payroll/due", get(payroll_due_api))
        .route("/api/commercial/proposals", post(service_proposal_api))
        .route("/api/commercial/pipeline", get(commercial_pipeline_api))
        .route("/api/commercial/proposals/transition", post(proposal_transition_api))
        .route("/api/commercial/sponsorships/transition", post(sponsorship_transition_api))
        .route("/api/commercial/sponsorships/delivery", post(sponsorship_delivery_api))
        .route("/api/commercial/sponsorships", post(sponsorship_api))
        .route("/api/commercial/invoices", post(invoice_api))
        .route("/api/commercial/invoices/issue", post(invoice_issue_api))
        .route("/api/commercial/invoice-payments", post(invoice_payment_api))
        .route("/api/commercial/payments/reconcile", post(payment_reconciliation_evidence_api))
        .route("/api/business-units", get(business_units_api))
        .route("/api/portfolio/metrics", get(portfolio_metrics_api))
        .route("/api/journal", get(journal_api))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .with_state(state)
        .layer(middleware::from_fn(require_control_plane_auth));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8080)).await?;

    println!("Company OS listening on http://localhost:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
