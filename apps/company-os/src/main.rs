mod publishing;
mod live;
use affiliate_intelligence::{
    search as search_affiliate, AffiliateProvider, AggregateAffiliateProvider, AwinProvider,
    MockProvider, ProductSearchQuery, SearchResponse, TikTokShopProvider,
};
use agent_runtime::{model_from_env, AgentRunResult, AgentRuntime, CompanySnapshot};
use axum::{
    extract::{Form, Query, State, Request},
    http::{header, HeaderMap, HeaderValue, StatusCode},
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
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tiktok_live_streaming::LiveStreamController;

#[derive(Default)]
struct RuntimeMetrics {
    cycles_started_total: AtomicU64,
    cycles_succeeded_total: AtomicU64,
    cycles_failed_total: AtomicU64,
    affiliate_search_total: AtomicU64,
    affiliate_search_failed_total: AtomicU64,
    control_plane_requests_total: AtomicU64,
    control_plane_auth_denied_total: AtomicU64,
    control_plane_browser_csrf_denied_total: AtomicU64,
    control_plane_last_latency_ms: AtomicU64,
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
                "company_cycle_last_latency_ms {}\n",
                "# HELP company_control_plane_requests_total Control-plane requests observed by the auth middleware.\n",
                "# TYPE company_control_plane_requests_total counter\n",
                "company_control_plane_requests_total {}\n",
                "# HELP company_control_plane_auth_denied_total Control-plane requests denied by authentication or scope checks.\n",
                "# TYPE company_control_plane_auth_denied_total counter\n",
                "company_control_plane_auth_denied_total {}\n",
                "# HELP company_control_plane_browser_csrf_denied_total Browser mutations denied by CSRF validation.\n",
                "# TYPE company_control_plane_browser_csrf_denied_total counter\n",
                "company_control_plane_browser_csrf_denied_total {}\n",
                "# HELP company_control_plane_last_latency_ms Last observed protected control-plane request latency in milliseconds.\n",
                "# TYPE company_control_plane_last_latency_ms gauge\n",
                "company_control_plane_last_latency_ms {}\n"
            ),
            self.cycles_started_total.load(Ordering::Relaxed),
            self.cycles_succeeded_total.load(Ordering::Relaxed),
            self.cycles_failed_total.load(Ordering::Relaxed),
            self.affiliate_search_total.load(Ordering::Relaxed),
            self.affiliate_search_failed_total.load(Ordering::Relaxed),
            self.last_cycle_latency_ms.load(Ordering::Relaxed),
            self.control_plane_requests_total.load(Ordering::Relaxed),
            self.control_plane_auth_denied_total.load(Ordering::Relaxed),
            self.control_plane_browser_csrf_denied_total.load(Ordering::Relaxed),
            self.control_plane_last_latency_ms.load(Ordering::Relaxed),
        )
    }
}

fn observe_control_plane_request(state: &AppState, request_id: &str, started: Instant) -> HeaderValue {
    state
        .metrics
        .control_plane_requests_total
        .fetch_add(1, Ordering::Relaxed);
    let latency_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
    state
        .metrics
        .control_plane_last_latency_ms
        .store(latency_ms, Ordering::Relaxed);
    HeaderValue::from_str(request_id).unwrap_or_else(|_| HeaderValue::from_static("invalid"))
}

fn mark_control_plane_denied(state: &AppState, csrf: bool, started: Instant) {
    state
        .metrics
        .control_plane_requests_total
        .fetch_add(1, Ordering::Relaxed);
    state
        .metrics
        .control_plane_auth_denied_total
        .fetch_add(1, Ordering::Relaxed);
    let latency_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
    state
        .metrics
        .control_plane_last_latency_ms
        .store(latency_ms, Ordering::Relaxed);
    if csrf {
        state
            .metrics
            .control_plane_browser_csrf_denied_total
            .fetch_add(1, Ordering::Relaxed);
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

#[derive(Debug, Serialize, Clone)]
struct IntegrationReadiness {
    key: String,
    status: String,
    configured: bool,
    authenticated: bool,
    evidence_fresh: bool,
    reason: String,
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

#[derive(Debug, Deserialize, Default)]
struct TikTokOAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Serialize)]
struct TikTokOAuthStatusResponse {
    connected: bool,
    connection: Option<company_store::TikTokConnectionRecord>,
}

#[derive(Debug, Deserialize)]
struct AffiliateConversionRequest {
    event: affiliate_attribution::ConversionEvent,
    model: affiliate_attribution::AttributionModel,
}

#[derive(Debug, Deserialize)]
struct PolicySnapshotRequest {
    snapshot: company_compliance::PolicySnapshot,
}

#[derive(Debug, Deserialize)]
struct ComplianceCheckRequest {
    input: company_compliance::ComplianceInput,
}

#[derive(Debug, Deserialize)]
struct RevenueGraphEdgeRequest {
    edge: company_revenue_graph::RevenueGraphEdge,
}

#[derive(Debug, Deserialize, Default)]
struct ControlPlaneAuditQuery {
    #[serde(default)]
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RevenueGraphLineageQuery {
    root_type: String,
    root_ref: String,
    #[serde(default)]
    max_depth: Option<i32>,
    #[serde(default)]
    limit: Option<i64>,
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
struct CapitalAllocationRequest {
    plan_key: String,
    policy: company_capital::CapitalPolicy,
    candidates: Vec<company_capital::CapitalCandidate>,
}

#[derive(Debug, Deserialize)]
struct ContentObservationRequest {
    observation: company_content::ContentObservation,
}

#[derive(Debug, Deserialize)]
struct AutonomyControlsRequest {
    emergency_stop: bool,
    reason: Option<String>,
    actor: String,
    budgets: company_safety_controls::AutonomyBudgets,
}

#[derive(Debug, Deserialize)]
struct AutonomyBudgetConsumeRequest {
    kind: String,
    amount: i128,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
struct AutonomyAssessRequest {
    proposal: agent_runtime::types::Proposal,
    daily_burn_minor: i128,
    reserve_cash_minor: i128,
}

#[derive(Debug, Deserialize)]
struct AgentOutcomeEvidenceRequest {
    decision_journal_id: i64,
    evidence_ref: String,
    observed_revenue_delta_minor: i128,
    observed_contribution_margin_delta_minor: i128,
    observed_at_epoch: i64,
}

#[derive(Debug, Deserialize)]
struct ContentStatusTransitionRequest {
    content_id: uuid::Uuid,
    next: company_content::ContentStatus,
    evidence_ref: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct AgentEvaluationQuery {
    days: Option<i64>,
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
    let (workforce, workforce_available) =
        match state.store.list_employees(&state.company_id).await {
            Ok(value) => (value, true),
            Err(error) => {
                tracing::warn!(%error, "workforce dashboard data unavailable");
                (Vec::new(), false)
            }
        };
    let (business_units, business_units_available) =
        match state.store.list_business_units(&state.company_id).await {
            Ok(value) => (value, true),
            Err(error) => {
                tracing::warn!(%error, "business-unit dashboard data unavailable");
                (Vec::new(), false)
            }
        };
    let (payroll_due, payroll_available) =
        match state.store.payroll_due(&state.company_id, 100).await {
            Ok(value) => (value, true),
            Err(error) => {
                tracing::warn!(%error, "payroll dashboard data unavailable");
                (Vec::new(), false)
            }
        };
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

    let target_minor = configured_revenue_target_minor(&state.currency)
        .unwrap_or_else(|_| if state.currency.eq_ignore_ascii_case("VND") { 50_000_000 } else { 500_000 });
    let (revenue_periods, revenue_periods_available) =
        match state.store.revenue_period_metrics(&state.company_id).await {
            Ok(value) => (value, true),
            Err(error) => {
                tracing::warn!(%error, "revenue period metrics unavailable");
                (
                    company_store::RevenuePeriodMetrics {
                        month_to_date_minor: 0,
                        last_30_days_minor: 0,
                        lifetime_minor: 0,
                        forecast_month_minor: 0,
                        run_rate_month_minor: 0,
                        forecast_confidence_bps: 0,
                        revenue_transaction_count: 0,
                    },
                    false,
                )
            }
        };
    let (contribution_margin, contribution_margin_available) =
        match state.store.contribution_margin_metrics(&state.company_id).await {
            Ok(value) => (value, true),
            Err(error) => {
                tracing::warn!(%error, "contribution margin metrics unavailable");
                (
                    company_store::ContributionMarginMetrics {
                        month_to_date_revenue_minor: 0,
                        month_to_date_variable_cost_minor: 0,
                        month_to_date_contribution_margin_minor: None,
                        platform_fees_minor: 0,
                        affiliate_commission_minor: 0,
                        refunds_cancellations_minor: 0,
                        production_ai_cost_minor: 0,
                        ad_spend_minor: 0,
                        operating_cost_minor: 0,
                        cash_minor: 0,
                        unclassified_expense_minor: 0,
                        unclassified_expense_entry_count: 0,
                        variable_cost_transaction_count: 0,
                    },
                    false,
                )
            }
        };
    let (affiliate_reconciliation, affiliate_reconciliation_available) =
        match state
            .store
            .affiliate_reconciliation_metrics(&state.company_id)
            .await
        {
            Ok(value) => (value, true),
            Err(error) => {
                tracing::warn!(%error, "affiliate reconciliation metrics unavailable");
                (
                    company_store::AffiliateReconciliationMetrics {
                        reported_commission_mtd_minor: 0,
                        attributed_commission_mtd_minor: 0,
                        recorded_payout_mtd_minor: 0,
                        variance_mtd_minor: 0,
                        reported_attributed_variance_mtd_minor: 0,
                        attributed_paid_variance_mtd_minor: 0,
                        reported_paid_variance_mtd_minor: 0,
                        conversion_count_mtd: 0,
                        verified_conversion_count_mtd: 0,
                        partial_or_rejected_count_mtd: 0,
                    },
                    false,
                )
            }
        };
    let mut growth_data_available = true;
    let growth_opportunities = match state
        .store
        .list_growth_opportunities(&state.company_id, 5)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            growth_data_available = false;
            tracing::warn!(%error, "growth opportunity metrics unavailable");
            Vec::new()
        }
    };
    let mut growth_html = String::new();
    for record in &growth_opportunities {
        let score_pct = record.opportunity.score_bps / 100;
        let status = match record.status {
            company_growth::OpportunityStatus::Ready => "READY",
            company_growth::OpportunityStatus::ContentCreated => "CONTENT CREATED",
        };
        let ttfc = record
            .ttfc_seconds
            .map(|seconds| format!("TTFC {}s", seconds))
            .unwrap_or_else(|| "TTFC pending".into());
        growth_html.push_str(&format!(
            r#"<div style="display:flex;justify-content:space-between;gap:12px;padding:10px 0;border-bottom:1px solid #26304a"><div><strong>{}</strong><div class="muted">{} · {}% confidence · {}</div></div><div class="metric" style="font-size:18px">{}</div></div>"#,
            escape_html(&record.opportunity.title),
            status,
            record.opportunity.confidence_bps / 100,
            ttfc,
            score_pct
        ));
    }
    let mut compliance_data_available = true;
    let compliance_status = match state.store.compliance_status(&state.company_id).await {
        Ok(value) => value,
        Err(error) => {
            compliance_data_available = false;
            tracing::warn!(%error, "compliance status unavailable");
            serde_json::json!({})
        }
    };
    let compliance_html = if !compliance_data_available {
        r#"<p class="muted">Policy intelligence is unavailable. External side effects remain fail-closed.</p>"#.to_string()
    } else if let Some(policy) = compliance_status.get("latest_policy") {
        let version = policy.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
        let active = policy.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
        let counts = compliance_status.get("checks_last_24h").cloned().unwrap_or_else(|| serde_json::json!({}));
        format!(
            r#"<div class="metric">{}</div><div class="muted">{} · {} · 24h: {} allowed / {} review / {} blocked / {} unknown</div>"#,
            if active { "Policy ready" } else { "Policy inactive" },
            escape_html(version),
            escape_html(policy.get("evidence_hash").and_then(|v| v.as_str()).unwrap_or("evidence unavailable")),
            counts.get("allowed").and_then(|v| v.as_i64()).unwrap_or(0),
            counts.get("review").and_then(|v| v.as_i64()).unwrap_or(0),
            counts.get("blocked").and_then(|v| v.as_i64()).unwrap_or(0),
            counts.get("unknown").and_then(|v| v.as_i64()).unwrap_or(0),
        )
    } else {
        r#"<p class="muted">No verified policy snapshot is loaded. External publishing/LIVE launch will remain blocked.</p>"#.to_string()
    };

    if !growth_data_available {
        growth_html.push_str(r#"<p class="muted">Growth pipeline data is unavailable. The dashboard is not treating this as “no opportunities.”</p>"#);
    } else if growth_html.is_empty() {
        growth_html.push_str(r#"<p class="muted">No evidence-backed opportunities have been accepted yet. Ingest a verified trend signal first.</p>"#);
    }

    let capital_plan_html = match state
        .store
        .latest_capital_allocation_plan(&state.company_id)
        .await
    {
        Ok(Some(record)) => {
            let stop = if record.policy.emergency_stop { "blocked by emergency stop" } else { "planning only" };
            let mut rows = String::new();
            for decision in record.plan.decisions.iter().filter(|decision| decision.allocation_minor > 0).take(3) {
                rows.push_str(&format!(
                    r#"<div style="padding:9px 0;border-bottom:1px solid #26304a"><strong>{}</strong><div class="muted">{} · score {} · allocation {}</div></div>"#,
                    escape_html(&decision.candidate_id.to_string()),
                    escape_html(&decision.reason),
                    decision.score_bps,
                    format_minor(decision.allocation_minor, &state.currency)
                ));
            }
            if rows.is_empty() {
                rows.push_str(r#"<div class="muted">No candidate currently passes the allocation gates.</div>"#);
            }
            format!(
                r#"<div class="metric">{}</div><div class="muted">{} · planned {} · unallocated {}</div>{}"#,
                stop,
                record.plan.decisions.len(),
                format_minor(record.plan.planned_capital_minor, &state.currency),
                format_minor(record.plan.unallocated_minor, &state.currency),
                rows
            )
        }
        Ok(None) => r#"<p class="muted">No capital allocation plan recorded yet. The company will not move cash from this dashboard.</p>"#.into(),
        Err(error) => {
            tracing::warn!(%error, "capital allocation plan dashboard unavailable");
            r#"<p class="muted">Capital planning evidence is unavailable. No allocation is treated as approved.</p>"#.into()
        }
    };

    let revenue_mtd_label = if revenue_periods_available {
        format_minor(revenue_periods.month_to_date_minor, &state.currency)
    } else {
        "Unavailable".into()
    };
    let revenue_count_label = if revenue_periods_available {
        revenue_periods.revenue_transaction_count.to_string()
    } else {
        "Unavailable".into()
    };
    let revenue_forecast_label = if revenue_periods_available {
        format_minor(revenue_periods.forecast_month_minor, &state.currency)
    } else {
        "Unavailable".into()
    };
    let revenue_confidence_label = if revenue_periods_available {
        format!("{:.2}%", revenue_periods.forecast_confidence_bps as f64 / 100.0)
    } else {
        "Unavailable".into()
    };
    let revenue_run_rate_label = if revenue_periods_available {
        format_minor(revenue_periods.run_rate_month_minor, &state.currency)
    } else {
        "Unavailable".into()
    };
    let revenue_last_30d_label = if revenue_periods_available {
        format_minor(revenue_periods.last_30_days_minor, &state.currency)
    } else {
        "Unavailable".into()
    };

    let contribution_margin_label = if !contribution_margin_available {
        "Unavailable".into()
    } else {
        contribution_margin
            .month_to_date_contribution_margin_minor
            .map(|value| format_minor(value, &state.currency))
            .unwrap_or_else(|| "Incomplete".into())
    };
    let contribution_margin_detail = if contribution_margin_available {
        format!(
            "variable cost {} · fixed operating cost {} · cash {} · unclassified {} across {} entries",
            format_minor(contribution_margin.month_to_date_variable_cost_minor, &state.currency),
            format_minor(contribution_margin.operating_cost_minor, &state.currency),
            format_minor(contribution_margin.cash_minor, &state.currency),
            format_minor(contribution_margin.unclassified_expense_minor, &state.currency),
            contribution_margin.unclassified_expense_entry_count
        )
    } else {
        "Contribution-margin evidence is unavailable.".into()
    };

    let cm_platform_fees_label = if contribution_margin_available {
        format_minor(contribution_margin.platform_fees_minor, &state.currency)
    } else { "Unavailable".into() };
    let cm_affiliate_label = if contribution_margin_available {
        format_minor(contribution_margin.affiliate_commission_minor, &state.currency)
    } else { "Unavailable".into() };
    let cm_refunds_label = if contribution_margin_available {
        format_minor(contribution_margin.refunds_cancellations_minor, &state.currency)
    } else { "Unavailable".into() };
    let cm_production_label = if contribution_margin_available {
        format_minor(contribution_margin.production_ai_cost_minor, &state.currency)
    } else { "Unavailable".into() };
    let cm_ads_label = if contribution_margin_available {
        format_minor(contribution_margin.ad_spend_minor, &state.currency)
    } else { "Unavailable".into() };

    let affiliate_variance_label = if affiliate_reconciliation_available {
        format_minor(affiliate_reconciliation.variance_mtd_minor, &state.currency)
    } else { "Unavailable".into() };
    let affiliate_reported_label = if affiliate_reconciliation_available {
        format_minor(affiliate_reconciliation.reported_commission_mtd_minor, &state.currency)
    } else { "Unavailable".into() };
    let affiliate_attributed_label = if affiliate_reconciliation_available {
        format_minor(affiliate_reconciliation.attributed_commission_mtd_minor, &state.currency)
    } else { "Unavailable".into() };
    let affiliate_paid_label = if affiliate_reconciliation_available {
        format_minor(affiliate_reconciliation.recorded_payout_mtd_minor, &state.currency)
    } else { "Unavailable".into() };
    let affiliate_reported_attributed_label = if affiliate_reconciliation_available {
        format_minor(affiliate_reconciliation.reported_attributed_variance_mtd_minor, &state.currency)
    } else { "Unavailable".into() };
    let affiliate_attributed_paid_label = if affiliate_reconciliation_available {
        format_minor(affiliate_reconciliation.attributed_paid_variance_mtd_minor, &state.currency)
    } else { "Unavailable".into() };
    let affiliate_reported_paid_label = if affiliate_reconciliation_available {
        format_minor(affiliate_reconciliation.reported_paid_variance_mtd_minor, &state.currency)
    } else { "Unavailable".into() };
    let (budget_statuses, budget_statuses_available) = match state
        .store
        .autonomy_budget_statuses(
            &state.company_id,
            time::OffsetDateTime::now_utc().unix_timestamp(),
        )
        .await
    {
        Ok(value) => (value, true),
        Err(error) => {
            tracing::warn!(%error, "autonomy budget statuses unavailable");
            (Vec::new(), false)
        }
    };
    let mut budget_status_html = String::new();
    if !budget_statuses_available {
        budget_status_html.push_str(r#"<p class="muted">Autonomy budget evidence is unavailable. No budget headroom is inferred.</p>"#);
    }
    for status in &budget_statuses {
        let (used, remaining, limit) = match status.kind {
            company_safety_controls::BudgetKind::AdsSpend
            | company_safety_controls::BudgetKind::AutonomousCapital => (
                format_minor(status.used, &state.currency),
                format_minor(status.remaining, &state.currency),
                format_minor(status.daily_limit, &state.currency),
            ),
            _ => (
                status.used.to_string(),
                status.remaining.to_string(),
                status.daily_limit.to_string(),
            ),
        };
        budget_status_html.push_str(&format!(
            r#"<div style="padding:7px 0;border-bottom:1px solid #26304a"><span class="muted">{}</span> · used {} / limit {} · remaining {}</div>"#,
            status.kind.as_str(),
            used,
            limit,
            remaining
        ));
    }

    let autonomy_policy = autonomy_policy_from_env();
    let persistent_controls = state.store.autonomy_controls(&state.company_id).await;
    let autonomy_stop = match (
        autonomy_emergency_stop_from_env(),
        persistent_controls.as_ref().map(|record| record.controls.emergency_stop.enabled),
    ) {
        (Ok(env_stop), Ok(persisted_stop)) => Ok(env_stop || persisted_stop),
        (Err(error), _) => Err(error),
        (_, Err(error)) => Err(error.to_string()),
    };
    let safety_controls_html = match persistent_controls {
        Ok(record) => {
            let stop_label = match autonomy_stop.as_ref() {
                Ok(true) => "EMERGENCY STOP ON",
                Ok(false) => "normal",
                Err(_) => "SAFETY STATE UNKNOWN",
            };
            let stop_reason = record
                .controls
                .emergency_stop
                .reason
                .as_deref()
                .unwrap_or("no active stop reason");
            let b = &record.controls.budgets;
            format!(
                r#"<div class="card"><h2>Safety controls</h2><div class="metric">{}</div><div class="muted">{} · budgets reset daily at UTC day start</div><div class="grid" style="margin-top:10px"><div><small>Content publishes</small><div class="metric" style="font-size:18px">{}</div></div><div><small>LIVE minutes</small><div class="metric" style="font-size:18px">{}</div></div><div><small>Messages</small><div class="metric" style="font-size:18px">{}</div></div><div><small>Autonomous capital</small><div class="metric" style="font-size:18px">{}</div></div></div><div style="margin-top:10px">{}</div><small class="muted">Stop reason: {} · actor: {}</small></div>"#,
                stop_label,
                escape_html(stop_reason),
                b.content_publish_daily,
                b.live_minutes_daily,
                b.outbound_messages_daily,
                format_minor(b.autonomous_capital_daily_minor, &state.currency),
                budget_status_html,
                escape_html(stop_reason),
                escape_html(&record.controls.emergency_stop.actor),
            )
        }
        Err(error) => {
            tracing::warn!(%error, "autonomy safety controls unavailable");
            r#"<div class="card"><h2>Safety controls</h2><p class="muted">Persistent safety controls are unavailable. Autonomous side effects remain fail-closed.</p></div>"#.into()
        }
    };
    let revenue_graph_html = match state.store.revenue_graph_summary(&state.company_id).await {
        Ok(summary) => format!(
            r#"<div class="card"><h2>Revenue intelligence graph</h2><div class="metric">{}</div><p class="muted">Evidence-backed edges · {} carry economic values · latest observation: {}</p><a href="/api/revenue-graph/summary">View graph summary JSON</a></div>"#,
            summary.edge_count,
            summary.value_backed_edge_count,
            summary.latest_observed_at_epoch.map(|value| value.to_string()).unwrap_or_else(|| "none".into())
        ),
        Err(error) => {
            tracing::warn!(%error, "revenue intelligence graph unavailable");
            r#"<div class="card"><h2>Revenue intelligence graph</h2><div class="metric">Evidence unavailable</div><p class="muted">Lineage data is unavailable, so the dashboard is not showing a fabricated zero.</p></div>"#.into()
        }
    };

    let tiktok_oauth_html = match state.store.tiktok_oauth_status(&state.company_id).await {
        Ok(Some(connection)) if connection.status == "ACTIVE" => {
            let access_expires = connection.access_token_expires_at_epoch;
            format!(
                r#"<div class="card"><h2>TikTok account</h2><div class="metric">Connected</div><p class="muted">open_id: {} · access token expiry: {} · refresh token expiry: {}</p><div style="display:flex;gap:8px;flex-wrap:wrap"><form method="post" action="/api/tiktok/oauth/refresh"><button type="submit">Refresh token</button></form><form method="post" action="/api/tiktok/oauth/revoke"><button type="submit">Revoke</button></form></div></div>"#,
                escape_html(&connection.open_id),
                access_expires,
                connection.refresh_token_expires_at_epoch
            )
        }
        Ok(Some(connection)) => format!(
            r#"<div class="card"><h2>TikTok account</h2><div class="metric">{}</div><p class="muted">{}</p><a href="/api/tiktok/oauth/start">Reconnect TikTok</a></div>"#,
            escape_html(&connection.status),
            escape_html(connection.last_error.as_deref().unwrap_or("TikTok authorization needs operator action."))
        ),
        Ok(None) => r#"<div class="card"><h2>TikTok account</h2><div class="metric">Not connected</div><p class="muted">Connect a TikTok account through Login Kit before enabling database-backed publishing.</p><a href="/api/tiktok/oauth/start">Connect TikTok</a></div>"#.into(),
        Err(error) => {
            tracing::warn!(%error, "TikTok OAuth status unavailable");
            r#"<div class="card"><h2>TikTok account</h2><div class="metric">Status unavailable</div><p class="muted">The account state could not be loaded, so publishing remains fail-closed.</p></div>"#.into()
        }
    };

    let autonomy_html = match (autonomy_policy, autonomy_stop) {
        (Ok(policy), Ok(emergency_stop)) => {
            let ceiling = policy.max_level.as_str();
            let stop_label = if emergency_stop { "EMERGENCY STOP ON" } else { "normal" };
            let step_class = |level: company_autonomy::AutonomyLevel| {
                if policy.max_level == level { "autonomy-step active" } else { "autonomy-step" }
            };
            format!(
                r#"<section class="autonomy-shell"><div><div class="section-kicker">Autonomy ladder</div><h2>Observe → Recommend → Simulate → Human approve → Limited → Strategic</h2><p class="muted">Current ceiling: <strong>{}</strong> · {} · limited autonomy requires {} bps confidence, {} evidence items, reversible action, simulation and ≥ {} days runway.</p></div><div class="autonomy-steps"><span class="{}">01 Observe</span><span class="{}">02 Recommend</span><span class="{}">03 Simulate</span><span class="{}">04 Human approve</span><span class="{}">05 Limited</span><span class="{}">06 Strategic</span></div><small class="muted">External/material actions remain human-gated. The digital twin never mutates the live company state.</small></section>"#,
                ceiling,
                stop_label,
                policy.min_confidence_bps,
                policy.min_evidence_count,
                policy.min_runway_days,
                step_class(company_autonomy::AutonomyLevel::Observe),
                step_class(company_autonomy::AutonomyLevel::Recommend),
                step_class(company_autonomy::AutonomyLevel::Simulate),
                step_class(company_autonomy::AutonomyLevel::HumanApprove),
                step_class(company_autonomy::AutonomyLevel::LimitedAutonomy),
                step_class(company_autonomy::AutonomyLevel::StrategicAutonomy)
            )
        }
        _ => r#"<section class="autonomy-shell"><div class="section-kicker">Autonomy ladder</div><h2>Policy unavailable</h2><p class="muted">Autonomy policy cannot be loaded safely, so no autonomous execution ceiling is advertised.</p></section>"#.into(),
    };

    let target_pct = if revenue_periods_available && revenue_periods.month_to_date_minor > 0 {
        ((revenue_periods.month_to_date_minor as f64 / target_minor as f64) * 100.0)
            .round()
            .min(999.0) as u64
    } else {
        0
    };
    let target_pct_label = if revenue_periods_available {
        format!("{target_pct}% of planning target · MTD evidence")
    } else {
        "Planning target progress unavailable · MTD evidence unavailable".into()
    };
    let capacity_pct = if company.capacity > 0 {
        ((company.backlog.max(0) as f64 / company.capacity as f64) * 100.0).round().min(999.0) as u64
    } else { 0 };
    let agent_count = latest.len();
    let active_staff_label = if workforce_available {
        workforce
            .iter()
            .filter(|e| matches!(e.status, company_organization::EmployeeStatus::Active))
            .count()
            .to_string()
    } else {
        "Unavailable".into()
    };
    let due_payroll_label = if payroll_available {
        payroll_due.len().to_string()
    } else {
        "Unavailable".into()
    };
    let business_unit_label = if business_units_available {
        business_units.len().to_string()
    } else {
        "Unavailable".into()
    };
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

    let command_center_html = match state
        .store
        .ceo_command_center(&state.company_id, target_minor)
        .await
    {
        Ok(record) => render_ceo_command_center(&record, &state.currency),
        Err(error) => {
            tracing::warn!(%error, "CEO revenue command center unavailable");
            r#"<section class="card"><div class="section-kicker">CEO Revenue Command Center</div><h2>Evidence unavailable</h2><p class="muted">The command center cannot safely assemble a complete view right now. Missing data is not being shown as zero.</p></section>"#.into()
        }
    };

    let agent_evaluation_html = match state
        .store
        .agent_outcome_evaluations(&state.company_id, 30)
        .await
    {
        Ok(evaluations) if evaluations.is_empty() => {
            r#"<div class="card"><div class="section-kicker">Agent outcome evaluation</div><h2>Waiting for outcome evidence</h2><p class="muted">The company has not yet recorded enough executed decisions with explicit business-outcome evidence to evaluate agents.</p></div>"#.into()
        }
        Ok(evaluations) => render_agent_evaluations(&evaluations, &state.currency),
        Err(error) => {
            tracing::warn!(%error, "agent outcome evaluation unavailable");
            r#"<div class="card"><div class="section-kicker">Agent outcome evaluation</div><h2>Evidence unavailable</h2><p class="muted">Agent scorecards are unavailable right now. Missing evaluation data is not being treated as failure.</p></div>"#.into()
        }
    };

    let integration_readiness_html = render_integration_readiness(&build_integration_readiness(&state).await);

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
.section-kicker{{font-size:12px;letter-spacing:.12em;text-transform:uppercase;color:#a78bfa;font-weight:800}}
.cc-shell{{border:1px solid #33415f;border-radius:22px;padding:22px;margin:0 0 18px;background:linear-gradient(180deg,#111827,#0d1422);box-shadow:0 24px 80px #0006}}
.cc-head{{display:flex;justify-content:space-between;gap:20px;align-items:flex-start}}
.cc-head h2{{margin:6px 0 8px;font-size:28px}}
.cc-trend{{display:flex;flex-direction:column;align-items:flex-end;gap:4px;white-space:nowrap}}
.cc-trend strong{{font-size:18px}}
.cc-kpis{{display:grid;grid-template-columns:repeat(6,minmax(0,1fr));gap:10px;margin:18px 0}}
.cc-kpi{{padding:14px;border:1px solid #273550;border-radius:14px;background:#0c1320}}
.cc-kpi span,.cc-mini-grid span{{display:block;font-size:12px;color:#94a3b8}}
.cc-kpi strong{{display:block;font-size:20px;margin-top:5px}}
.cc-kpi em{{display:block;font-size:12px;color:#94a3b8;font-style:normal;margin-top:4px}}
.cc-body{{display:grid;grid-template-columns:1.25fr 1.25fr .8fr .9fr;gap:10px}}
.cc-panel{{padding:16px;border:1px solid #273550;border-radius:14px;background:#0c1320;min-width:0}}
.cc-panel h3,.cc-alerts h3{{margin:0 0 10px}}
.cc-progress{{height:8px;border-radius:999px;background:#172033;overflow:hidden;margin:12px 0}}
.cc-progress span,.cc-bar span{{display:block;height:100%;border-radius:999px;background:linear-gradient(90deg,#8b5cf6,#22d3ee)}}
.cc-chart{{display:grid;gap:7px;margin-top:14px}}
.cc-day{{display:grid;grid-template-columns:78px 1fr 96px;gap:8px;align-items:center;font-size:12px}}
.cc-day-label,.cc-day-value{{color:#cbd5e1}}
.cc-day-value{{text-align:right}}
.cc-bar{{height:7px;border-radius:999px;background:#172033;overflow:hidden}}
.cc-mini-grid{{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}}
.cc-mini-grid strong{{display:block;font-size:16px;margin-top:3px}}
.cc-bigline{{display:flex;align-items:baseline;gap:8px;margin:8px 0 16px}}
.cc-bigline strong{{font-size:27px}}
.cc-bigline span{{font-size:12px;color:#94a3b8}}
.cc-alerts{{display:grid;grid-template-columns:1.15fr 1fr 1fr;gap:10px;margin-top:10px}}
.cc-alerts>div{{padding:14px;border:1px solid #273550;border-radius:14px;background:#0c1320}}
.cc-alert{{padding:10px 0;border-bottom:1px solid #273550}}
.cc-alert:last-child{{border-bottom:0}}
.cc-alert.attention strong{{color:#fbbf24}}
.cc-alert.opportunity strong{{color:#67e8f9}}
.cc-alert.healthy strong{{color:#86efac}}
.cc-footer{{display:flex;gap:14px;flex-wrap:wrap;margin-top:12px;padding-top:12px;border-top:1px solid #273550;font-size:12px;color:#94a3b8}}
.evaluation-table{{width:100%;border-collapse:collapse}}
.evaluation-table th,.evaluation-table td{{padding:10px 8px;border-bottom:1px solid #273550;text-align:left;font-size:13px;vertical-align:top}}
.evaluation-table th{{color:#94a3b8;font-size:11px;text-transform:uppercase;letter-spacing:.08em}}
.evaluation-status{{font-weight:800}}
.evaluation-status.insufficient{{color:#fbbf24}}
.evaluation-status.partial{{color:#67e8f9}}
.evaluation-status.evaluated{{color:#86efac}}
nav{{display:flex;gap:8px;flex-wrap:wrap;margin:0 0 18px}} nav a{{color:#94a3b8;text-decoration:none;padding:8px 10px;border-radius:8px}} nav a:hover{{background:#171e30;color:#fff}}
.autonomy-shell{{border:1px solid #33415f;border-radius:18px;padding:18px;margin:0 0 18px;background:#0c1320}}
.autonomy-shell h2{{margin:5px 0 7px;font-size:20px}}
.autonomy-steps{{display:grid;grid-template-columns:repeat(6,minmax(0,1fr));gap:6px;margin:15px 0 10px}}
.autonomy-step{{padding:9px 7px;border-radius:10px;border:1px solid #273550;text-align:center;font-size:11px;color:#94a3b8;background:#101827}}
.autonomy-step.active{{color:#f8fafc;border-color:#7c3aed;background:#1a1232}}
@media(max-width:1050px){{.cc-kpis{{grid-template-columns:repeat(3,minmax(0,1fr))}}.cc-body{{grid-template-columns:repeat(2,minmax(0,1fr))}}.cc-alerts{{grid-template-columns:1fr}}}}
@media(max-width:800px){{.grid{{grid-template-columns:repeat(2,minmax(0,1fr))}}.cc-head{{flex-direction:column}}.cc-trend{{align-items:flex-start}}}}
@media(max-width:520px){{.grid{{grid-template-columns:1fr}}.cc-kpis{{grid-template-columns:1fr 1fr}}.cc-body{{grid-template-columns:1fr}}.autonomy-steps{{grid-template-columns:repeat(2,minmax(0,1fr))}}}}
</style></head><body>
<header><h1>Veridara AI</h1><small>Autonomous Company OS · {}</small></header>
<nav><a href="/auth/login">Browser sign-in</a><a href="#integrations">Integrations</a><form method="post" action="/auth/logout" style="display:inline"><button type="submit">Sign out</button></form><a href="/">Overview</a><a href="#revenue-command-center">Revenue command center</a><a href="#capital">Capital planning</a><a href="#safety">Safety controls</a><a href="#autonomy">Autonomy policy</a><a href="#agent-outcomes">Agent outcomes</a><a href="#revenue-graph">Revenue graph</a><a href="#growth">Growth pipeline</a><a href="#tiktok">TikTok LIVE</a><a href="#compliance">Policy intelligence</a></nav>
<div id="revenue-command-center">{}</div>
<div id="agent-outcomes">{}</div>
<div id="autonomy">{}</div>
<div class="grid">
<div class="card"><small>Cash</small><div class="metric">{}</div></div>
<div class="card"><small>Revenue MTD</small><div class="metric">{}</div><small>Ledger evidence: {} revenue transactions</small></div>
<div class="card"><small>Monthly target</small><div class="metric">{}</div><div class="progress"><span style="width:{}%"></span></div><small>{}</small></div>
<div class="card"><small>Revenue forecast</small><div class="metric">{}</div><small>calendar-day run-rate · confidence coverage: {}%</small></div>
<div class="card"><small>Last 30 days</small><div class="metric">{}</div><small>Monthly run-rate (30d scaled): {}</small></div>
<div class="card"><small>Runway</small><div class="metric">{} days</div></div>
</div>
<div class="card"><small>Contribution margin MTD</small><div class="metric">{}</div><small>{}</small><div class="muted" style="margin-top:8px">platform {} · affiliate commission {} · refunds/cancellations {} · production/AI {} · ads {}</div></div>
<div class="card"><small>Affiliate reconciliation MTD</small><div class="metric">{}</div><small>reported · attributed · paid: {} · {} · {}</small><div class="muted" style="margin-top:8px">reported-attributed {} · attributed-paid {} · reported-paid {}</div></div>
<div id="growth" class="card"><h2>Growth pipeline</h2><p class="muted">Evidence-backed trend signals become scored opportunities before any content plan is created.</p>{}</div>
<div id="revenue-graph">{}</div>
<div id="capital" class="card"><h2>Capital allocation</h2><p class="muted">Expected contribution, downside, speed, reversibility and evidence are evaluated before any capital movement.</p>{}</div>
<div id="safety">{}</div>
<div id="tiktok">{}</div>
<div id="integrations">{}</div>
<div id="compliance" class="card"><h2>Policy intelligence</h2><p class="muted">External content/LIVE side effects require a matching versioned policy snapshot and evidence.</p>{}</div>
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
<script>
(() => {{{{
  const csrf = document.cookie
    .split('; ')
    .find(row => row.startsWith('company_os_csrf='))
    ?.split('=')[1];
  if (!csrf) return;
  for (const form of document.querySelectorAll('form[method="post"]')) {{
    const action = form.getAttribute('action');
    if (!action || action.startsWith('/auth/session')) continue;
    const url = new URL(action, window.location.origin);
    url.searchParams.set('csrf', csrf);
    form.setAttribute('action', url.pathname + url.search + url.hash);
  }}
}}}})();</script></body></html>"#,
        state.company_id.clone(),
        command_center_html,
        agent_evaluation_html,
        autonomy_html,
        format_minor(company.cash_minor, &state.currency),
        revenue_mtd_label,
        revenue_count_label,
        format_minor(target_minor, &state.currency),
        target_pct,
        target_pct_label,
        revenue_forecast_label,
        revenue_confidence_label,
        revenue_last_30d_label,
        revenue_run_rate_label,
        contribution_margin_label,
        contribution_margin_detail,
        cm_platform_fees_label,
        cm_affiliate_label,
        cm_refunds_label,
        cm_production_label,
        cm_ads_label,
        affiliate_variance_label,
        affiliate_reported_label,
        affiliate_attributed_label,
        affiliate_paid_label,
        affiliate_reported_attributed_label,
        affiliate_attributed_paid_label,
        affiliate_reported_paid_label,
        growth_html,
        revenue_graph_html,
        capital_plan_html,
        safety_controls_html,
        tiktok_oauth_html,
        integration_readiness_html,
        compliance_html,
        company.runway_days,
        company.status,
        cycle_state,
        capacity_pct,
        agent_count,
        active_staff_label,
        due_payroll_label,
        business_unit_label,
        rows,
        live_feedback(&query),
    ))
}

fn render_ceo_command_center(
    record: &company_store::CeoCommandCenterRecord,
    currency: &str,
) -> String {
    use company_command_center::AlertKind;

    let input = &record.input;
    let summary = &record.summary;
    let trend = match summary.revenue_trend {
        company_command_center::RevenueTrend::Up => "↑ Up",
        company_command_center::RevenueTrend::Down => "↓ Down",
        company_command_center::RevenueTrend::Flat => "→ Flat",
    };
    let trend_delta = format!("{:.2}%", summary.revenue_trend_delta_bps as f64 / 100.0);
    let target_progress = summary.target_progress_bps / 100;

    let mut attention = String::new();
    let mut opportunities = String::new();
    let mut healthy = String::new();
    for alert in &summary.alerts {
        let class = match alert.kind {
            AlertKind::NeedsAttention => "attention",
            AlertKind::Opportunity => "opportunity",
            AlertKind::Healthy => "healthy",
        };
        let html = format!(
            r#"<div class="cc-alert {}"><strong>{}</strong><div class="muted">{}</div></div>"#,
            class,
            escape_html(&alert.title),
            escape_html(&alert.detail)
        );
        match alert.kind {
            AlertKind::NeedsAttention => attention.push_str(&html),
            AlertKind::Opportunity => opportunities.push_str(&html),
            AlertKind::Healthy => healthy.push_str(&html),
        }
    }
    if attention.is_empty() {
        attention = r#"<div class="muted">No active exception was raised from available evidence.</div>"#.into();
    }
    if opportunities.is_empty() {
        opportunities = r#"<div class="muted">No scored growth opportunity is ready for attention.</div>"#.into();
    }
    if healthy.is_empty() {
        healthy = r#"<div class="muted">Healthy state will appear when no exception is active.</div>"#.into();
    }

    let max_daily = input
        .daily_revenue
        .iter()
        .map(|point| point.revenue_minor)
        .max()
        .unwrap_or(0);
    let mut daily_html = String::new();
    for point in &input.daily_revenue {
        let width = if max_daily > 0 {
            point
                .revenue_minor
                .saturating_mul(100)
                .checked_div(max_daily)
                .unwrap_or(0)
                .clamp(0, 100)
        } else {
            0
        };
        daily_html.push_str(&format!(
            r#"<div class="cc-day"><div class="cc-day-label">{}</div><div class="cc-bar"><span style="width:{}%"></span></div><div class="cc-day-value">{}</div></div>"#,
            escape_html(&point.day),
            width,
            format_minor(point.revenue_minor, currency)
        ));
    }

    let employee_productivity = summary
        .revenue_mtd_per_active_employee_minor
        .map(|value| format_minor(value, currency))
        .unwrap_or_else(|| "n/a".into());
    let content_productivity = summary
        .commission_7d_per_content_minor
        .map(|value| format_minor(value, currency))
        .unwrap_or_else(|| "n/a".into());

    format!(
        r#"<section class="cc-shell">
<div class="cc-head"><div><div class="section-kicker">CEO Revenue Command Center</div><h2>What is happening with the business?</h2><p class="muted">Ledger-backed revenue, commerce, content, LIVE and policy evidence in one operating view.</p></div><div class="cc-trend"><strong>{}</strong><span class="muted">7d pulse · {}</span></div></div>
<div class="cc-kpis">
<div class="cc-kpi"><span>Cash</span><strong>{}</strong></div>
<div class="cc-kpi"><span>Revenue MTD</span><strong>{}</strong><em>{}% of target</em></div>
<div class="cc-kpi"><span>Contribution margin MTD</span><strong>{}</strong><em>{}</em></div>
<div class="cc-kpi"><span>Runway</span><strong>{}d</strong></div>
<div class="cc-kpi"><span>Affiliate orders MTD</span><strong>{}</strong><em>{}</em></div>
<div class="cc-kpi"><span>Affiliate variance</span><strong>{}</strong><em>reported − attributed</em></div>
</div>
<div class="cc-body">
<div class="cc-panel"><h3>Revenue pulse</h3><div class="cc-progress"><span style="width:{}%"></span></div><div class="muted">MTD {} · last 30d {} · lifetime {}</div><div class="cc-chart">{}</div></div>
<div class="cc-panel"><h3>Content funnel · last 7d</h3><div class="cc-mini-grid"><div><span>Views</span><strong>{}</strong></div><div><span>Clicks</span><strong>{}</strong></div><div><span>CTR</span><strong>{:.2}%</strong></div><div><span>CVR</span><strong>{:.2}%</strong></div><div><span>Conversions</span><strong>{}</strong></div><div><span>Commission</span><strong>{}</strong></div><div><span>Spend</span><strong>{}</strong></div><div><span>Margin</span><strong>{}</strong></div></div><div class="muted">Commission / content: {} · commission RPM: {}</div></div>
<div class="cc-panel"><h3>LIVE pulse · last 30d</h3><div class="cc-bigline"><strong>{}</strong><span>sessions</span></div><div class="cc-mini-grid"><div><span>Gift count</span><strong>{}</strong></div><div><span>Recorded gift value</span><strong>{}</strong></div></div><div class="muted">Gift value is not recognized company revenue.</div></div>
<div class="cc-panel"><h3>Policy & evidence</h3><div class="cc-bigline"><strong>{}</strong><span>{} / {} / {} / {} (24h)</span></div><div class="muted">Allowed / review / blocked / unknown. External side effects stay fail-closed when evidence is missing.</div></div>
</div>
<div class="cc-alerts"><div><h3>Needs attention</h3>{}</div><div><h3>Opportunities</h3>{}</div><div><h3>Healthy</h3>{}</div></div>
<div class="cc-footer"><span>Revenue / active employee: {}</span><span>Affiliate net order value MTD: {}</span><span>7d spend: {}</span><span>Payroll due: {}</span></div>
</section>"#,
        trend,
        trend_delta,
        format_minor(input.cash_minor, currency),
        format_minor(input.revenue_mtd_minor, currency),
        target_progress,
        format_minor(input.contribution_margin_mtd_minor.unwrap_or(0), currency),
        if input.contribution_margin_mtd_minor.is_some() { "evidence complete" } else { "incomplete" },
        input.runway_days,
        input.affiliate_orders_mtd,
        format_minor(input.affiliate_net_order_value_mtd_minor, currency),
        format_minor(input.affiliate_variance_mtd_minor, currency),
        summary.target_progress_bps / 100,
        format_minor(input.revenue_mtd_minor, currency),
        format_minor(input.revenue_last_30d_minor, currency),
        format_minor(input.revenue_lifetime_minor, currency),
        daily_html,
        input.content.views_7d,
        input.content.clicks_7d,
        input.content.ctr_bps as f64 / 100.0,
        input.content.cvr_bps as f64 / 100.0,
        input.content.conversions_7d,
        format_minor(input.content.commission_7d_minor, currency),
        format_minor(input.content.spend_7d_minor, currency),
        format_minor(input.content.contribution_margin_7d_minor, currency),
        content_productivity,
        format_minor(input.content.commission_rpm_minor, currency),
        input.live.sessions_30d,
        input.live.gift_count_30d,
        format_minor(input.live.gift_value_30d_minor, currency),
        if input.compliance.policy_ready { "Policy ready" } else { "Policy blocked" },
        input.compliance.allowed_24h,
        input.compliance.review_24h,
        input.compliance.blocked_24h,
        input.compliance.unknown_24h,
        attention,
        opportunities,
        healthy,
        employee_productivity,
        format_minor(input.affiliate_net_order_value_mtd_minor, currency),
        format_minor(input.content.spend_7d_minor, currency),
        input.payroll_due_count,
    )
}

fn render_agent_evaluations(
    evaluations: &[company_agent_evaluation::AgentEvaluation],
    currency: &str,
) -> String {
    let mut rows = String::new();
    for evaluation in evaluations {
        let status = match evaluation.status {
            company_agent_evaluation::EvaluationStatus::InsufficientEvidence => {
                ("Insufficient evidence", "insufficient")
            }
            company_agent_evaluation::EvaluationStatus::PartialEvidence => {
                ("Partial evidence", "partial")
            }
            company_agent_evaluation::EvaluationStatus::Evaluated => {
                ("Evaluated", "evaluated")
            }
        };
        let observed_revenue = evaluation
            .observed_revenue_delta_minor
            .map(|value| format_minor(value, currency))
            .unwrap_or_else(|| "n/a".into());
        let observed_margin = evaluation
            .observed_contribution_margin_delta_minor
            .map(|value| format_minor(value, currency))
            .unwrap_or_else(|| "n/a".into());
        let projected_return = evaluation
            .projected_return_bps
            .map(|value| format!("{:.2}%", value as f64 / 100.0))
            .unwrap_or_else(|| "n/a".into());
        let observed_return = evaluation
            .observed_return_bps
            .map(|value| format!("{:.2}%", value as f64 / 100.0))
            .unwrap_or_else(|| "n/a".into());
        rows.push_str(&format!(
            r#"<tr><td><strong>{}</strong></td><td>{}</td><td>{}</td><td>{}</td><td>{:.0}%</td><td>{:.0}%</td><td>{:.0}%</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class="evaluation-status {}">{}</td></tr>"#,
            escape_html(&evaluation.agent_name),
            evaluation.proposal_count,
            evaluation.approved_count,
            evaluation.executed_count,
            evaluation.approval_rate_bps as f64 / 100.0,
            evaluation.execution_realization_bps as f64 / 100.0,
            evaluation.outcome_evidence_coverage_bps as f64 / 100.0,
            format_minor(evaluation.observed_spend_minor, currency),
            projected_return,
            observed_revenue,
            observed_return,
            observed_margin,
            status.1,
            status.0
        ));
    }

    format!(
        r#"<div class="card"><div class="section-kicker">Agent outcome evaluation</div><h2>Business outcomes, not output volume</h2><p class="muted">Last 30 days · approval rate is descriptive; outcome evidence coverage shows how much executed work has a direct evidence trail.</p><div style="overflow:auto"><table class="evaluation-table"><tr><th>Agent</th><th>Proposals</th><th>Approved</th><th>Executed</th><th>Approval</th><th>Execution / approval</th><th>Outcome evidence</th><th>Spend</th><th>Projected return</th><th>Observed revenue Δ</th><th>Observed return</th><th>Observed CM Δ</th><th>Status</th></tr>{}</table></div><p class="muted">Projected return is proposal expectation. Observed revenue and contribution-margin deltas are shown only when explicitly evidenced against an executed decision; they are not inferred from timing alone.</p></div>"#,
        rows
    )
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

async fn build_integration_readiness(
    state: &AppState,
) -> Vec<IntegrationReadiness> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let llm_provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "gemini".into());
    let llm_provider = llm_provider.trim().to_ascii_lowercase();
    let (llm_configured, llm_authenticated, llm_status, llm_reason) = match llm_provider.as_str() {
        "mock" => (true, true, "READY", "Deterministic mock provider is configured."),
        "gemini" if std::env::var("GEMINI_API_KEY").ok().is_some_and(|v| !v.trim().is_empty()) => {
            (true, false, "CONFIGURED", "Gemini credentials are configured; external reachability is not verified here.")
        }
        "ollama" => (true, false, "CONFIGURED", "Ollama provider configuration is selected; endpoint reachability is not verified here."),
        _ => (false, false, "NOT_CONFIGURED", "The selected LLM provider is unsupported or missing required configuration."),
    };

    let affiliate_provider = std::env::var("AFFILIATE_PROVIDER")
        .or_else(|_| std::env::var("AFFILIATE_PROVIDERS"))
        .unwrap_or_else(|_| "mock".into());
    let affiliate_provider = affiliate_provider
        .split(',')
        .map(str::trim)
        .find(|value| !value.is_empty())
        .unwrap_or("mock");
    let (affiliate_status, affiliate_configured, affiliate_reason) =
        match affiliate_provider.to_ascii_lowercase().as_str() {
            "mock" => ("READY", true, "Deterministic mock affiliate provider is configured."),
            "awin" if std::env::var("AWIN_ACCESS_TOKEN").ok().is_some_and(|v| !v.trim().is_empty())
                && std::env::var("AWIN_PUBLISHER_ID").ok().is_some_and(|v| !v.trim().is_empty()) =>
            {
                ("CONFIGURED", true, "Awin credentials are configured; external feed verification is not asserted here.")
            }
            "tiktok" | "tiktok_shop" | "tiktok-shop"
                if std::env::var("TTS_APP_KEY").ok().is_some_and(|v| !v.trim().is_empty())
                    && std::env::var("TTS_APP_SECRET").ok().is_some_and(|v| !v.trim().is_empty()) =>
            {
                ("CONFIGURED", true, "TikTok Shop credentials are configured; external feed verification is not asserted here.")
            }
            _ => ("NOT_CONFIGURED", false, "Selected affiliate provider lacks the required credentials."),
        };

    let tiktok = match state.store.tiktok_oauth_status(&state.company_id).await {
        Ok(Some(connection)) if connection.status == "ACTIVE" && connection.access_token_expires_at_epoch > now => IntegrationReadiness {
            key: "tiktok_oauth".into(),
            status: "READY".into(),
            configured: true,
            authenticated: true,
            evidence_fresh: true,
            reason: "Stored TikTok OAuth connection is active and its access token has not expired.".into(),
        },
        Ok(Some(connection)) if connection.status == "ACTIVE" => IntegrationReadiness {
            key: "tiktok_oauth".into(),
            status: "ACTION_REQUIRED".into(),
            configured: true,
            authenticated: false,
            evidence_fresh: false,
            reason: "Stored TikTok OAuth connection is active but its access token has expired.".into(),
        },
        Ok(Some(connection)) => IntegrationReadiness {
            key: "tiktok_oauth".into(),
            status: "ACTION_REQUIRED".into(),
            configured: true,
            authenticated: false,
            evidence_fresh: false,
            reason: connection.last_error.unwrap_or_else(|| "TikTok OAuth connection is not active.".into()),
        },
        Ok(None) => IntegrationReadiness {
            key: "tiktok_oauth".into(),
            status: "NOT_CONFIGURED".into(),
            configured: false,
            authenticated: false,
            evidence_fresh: false,
            reason: "No TikTok OAuth connection is stored.".into(),
        },
        Err(error) => IntegrationReadiness {
            key: "tiktok_oauth".into(),
            status: "UNAVAILABLE".into(),
            configured: false,
            authenticated: false,
            evidence_fresh: false,
            reason: format!("TikTok OAuth status could not be loaded: {error}"),
        },
    };

    let live_enabled = parse_bool_env("TIKTOK_LIVE_ENABLED", false);
    let live_publisher_configured = std::env::var("TIKTOK_LIVE_STREAM_DESTINATION")
        .ok()
        .is_some_and(|v| !v.trim().is_empty());
    let live_approved = parse_bool_env("TIKTOK_LIVE_PUBLISH_APPROVED", false);
    let live = IntegrationReadiness {
        key: "tiktok_live".into(),
        status: if !live_enabled {
            "GATED".into()
        } else if live_publisher_configured && live_approved {
            "CONFIGURED".into()
        } else {
            "ACTION_REQUIRED".into()
        },
        configured: live_enabled && live_publisher_configured,
        authenticated: false,
        evidence_fresh: false,
        reason: if !live_enabled {
            "LIVE engine is disabled by configuration.".into()
        } else if live_publisher_configured && live_approved {
            "Publisher destination and publish approval flag are present; external account transport is still not verified.".into()
        } else {
            "LIVE is enabled but destination and/or publish approval evidence is incomplete.".into()
        },
    };

    let email_configured = std::env::var("RESEND_API_KEY")
        .ok()
        .is_some_and(|v| !v.trim().is_empty())
        && std::env::var("RESEND_FROM")
            .ok()
            .is_some_and(|v| !v.trim().is_empty());
    let browser_session = std::env::var("CONTROL_PLANE_BROWSER_SECRET")
        .ok()
        .is_some_and(|v| v.len() >= 32);

    let compliance = match state.store.compliance_status(&state.company_id).await {
        Ok(value) => {
            let active = value
                .get("latest_policy")
                .and_then(|policy| policy.get("active"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let evidence = value
                .get("latest_policy")
                .and_then(|policy| policy.get("evidence_hash"))
                .and_then(serde_json::Value::as_str)
                .is_some_and(|v| !v.is_empty());
            IntegrationReadiness {
                key: "compliance_policy".into(),
                status: if active && evidence { "READY" } else { "GATED" }.into(),
                configured: value.get("latest_policy").is_some(),
                authenticated: active,
                evidence_fresh: evidence,
                reason: if active && evidence {
                    "An active versioned policy snapshot with evidence is available.".into()
                } else {
                    "No active verified policy snapshot is available; external side effects remain gated.".into()
                },
            }
        }
        Err(error) => IntegrationReadiness {
            key: "compliance_policy".into(),
            status: "UNAVAILABLE".into(),
            configured: false,
            authenticated: false,
            evidence_fresh: false,
            reason: format!("Policy status could not be loaded: {error}"),
        },
    };

    vec![
        IntegrationReadiness {
            key: "llm".into(),
            status: llm_status.into(),
            configured: llm_configured,
            authenticated: llm_authenticated,
            evidence_fresh: false,
            reason: llm_reason.into(),
        },
        IntegrationReadiness {
            key: "affiliate".into(),
            status: affiliate_status.into(),
            configured: affiliate_configured,
            authenticated: affiliate_configured && affiliate_provider.eq_ignore_ascii_case("mock"),
            evidence_fresh: false,
            reason: affiliate_reason.into(),
        },
        tiktok,
        live,
        IntegrationReadiness {
            key: "outbound_email".into(),
            status: if email_configured { "CONFIGURED" } else { "NOT_CONFIGURED" }.into(),
            configured: email_configured,
            authenticated: false,
            evidence_fresh: false,
            reason: if email_configured {
                "Resend sender/key are configured; delivery acceptance is not verified here.".into()
            } else {
                "Resend API key and sender are not both configured.".into()
            },
        },
        IntegrationReadiness {
            key: "browser_session".into(),
            status: if browser_session { "READY" } else { "NOT_CONFIGURED" }.into(),
            configured: browser_session,
            authenticated: browser_session,
            evidence_fresh: false,
            reason: if browser_session {
                "Browser session signing secret is configured.".into()
            } else {
                "CONTROL_PLANE_BROWSER_SECRET is not configured; bearer API auth remains available.".into()
            },
        },
        compliance,
    ]
}

fn render_integration_readiness(items: &[IntegrationReadiness]) -> String {
    let mut rows = String::new();
    for item in items {
        rows.push_str(&format!(
            r#"<div style="display:flex;justify-content:space-between;gap:12px;padding:10px 0;border-bottom:1px solid #26304a"><div><strong>{}</strong><div class="muted">{}</div></div><div style="text-align:right"><strong>{}</strong><div class="muted">configured={} · authenticated={} · evidence_fresh={}</div></div></div>"#,
            escape_html(&item.key),
            escape_html(&item.reason),
            escape_html(&item.status),
            item.configured,
            item.authenticated,
            item.evidence_fresh
        ));
    }
    format!(
        r#"<div class="card"><h2>Integration readiness</h2><p class="muted">Configured, authenticated and evidence-fresh are separate states. No missing evidence is represented as success or zero.</p>{}<p><a href="/api/integrations/readiness">View readiness JSON</a></p></div>"#,
        rows
    )
}

async fn integrations_readiness_api(
    State(state): State<AppState>,
) -> Json<Vec<IntegrationReadiness>> {
    Json(build_integration_readiness(&state).await)
}

async fn control_plane_audit_api(
    State(state): State<AppState>,
    Query(query): Query<ControlPlaneAuditQuery>,
) -> Result<Json<Vec<company_store::ControlPlaneAuditRecord>>, StatusCode> {
    state
        .store
        .list_control_plane_audit(&state.company_id, query.limit.unwrap_or(100))
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn autonomy_controls_get_api(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let record = state
        .store
        .autonomy_controls(&state.company_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let budgets = state
        .store
        .autonomy_budget_statuses(
            &state.company_id,
            time::OffsetDateTime::now_utc().unix_timestamp(),
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let env_stop = autonomy_emergency_stop_from_env()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let effective_emergency_stop = env_stop || record.controls.emergency_stop.enabled;
    Ok(Json(serde_json::json!({
        "company_id": state.company_id,
        "controls": record.controls,
        "updated_at": record.updated_at,
        "effective_emergency_stop": effective_emergency_stop,
        "budgets": budgets,
        "identity_boundary": "actor is audit metadata; operator identity is the control-plane bearer token"
    })))
}

async fn autonomy_controls_set_api(
    State(state): State<AppState>,
    Json(request): Json<AutonomyControlsRequest>,
) -> Result<Json<company_store::AutonomyControlRecord>, StatusCode> {
    state
        .store
        .set_autonomy_controls(
            &state.company_id,
            request.emergency_stop,
            request.reason.as_deref(),
            &request.actor,
            &request.budgets,
        )
        .await
        .map(Json)
        .map_err(|error| {
            tracing::warn!(%error, "autonomy control update rejected");
            StatusCode::BAD_REQUEST
        })
}

async fn autonomy_budget_consume_api(
    State(state): State<AppState>,
    Json(request): Json<AutonomyBudgetConsumeRequest>,
) -> Result<Json<company_safety_controls::BudgetDecision>, StatusCode> {
    let kind = company_safety_controls::BudgetKind::parse(&request.kind)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
    let decision = state
        .store
        .consume_autonomy_budget(
            &state.company_id,
            kind,
            request.amount,
            &request.idempotency_key,
            now_epoch,
        )
        .await
        .map_err(|error| {
            tracing::warn!(%error, "autonomy budget consumption rejected");
            StatusCode::BAD_REQUEST
        })?;
    if decision.allowed {
        Ok(Json(decision))
    } else {
        Err(StatusCode::PRECONDITION_FAILED)
    }
}

async fn autonomy_assess_api(
    State(state): State<AppState>,
    Json(request): Json<AutonomyAssessRequest>,
) -> Result<Json<company_store::AutonomySimulationRecord>, StatusCode> {
    if request.daily_burn_minor <= 0 || request.reserve_cash_minor < 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let policy = autonomy_policy_from_env().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let env_stop = autonomy_emergency_stop_from_env()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let controls = state
        .store
        .autonomy_controls(&state.company_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let emergency_stop = env_stop || controls.controls.emergency_stop.enabled;
    let config = company_autonomy::DigitalTwinConfig {
        daily_burn_minor: request.daily_burn_minor,
        reserve_cash_minor: request.reserve_cash_minor,
    };
    state
        .store
        .assess_autonomy_for_company(
            &state.company_id,
            &request.proposal,
            policy,
            emergency_stop,
            &config,
        )
        .await
        .map(Json)
        .map_err(|error| {
            tracing::warn!(%error, "autonomy assessment unavailable");
            StatusCode::BAD_REQUEST
        })
}

async fn autonomy_policy_api(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let policy = autonomy_policy_from_env().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let env_stop = autonomy_emergency_stop_from_env()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let controls = state
        .store
        .autonomy_controls(&state.company_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let emergency_stop = env_stop || controls.controls.emergency_stop.enabled;
    Ok(Json(serde_json::json!({
        "max_level": policy.max_level.as_str(),
        "min_confidence_bps": policy.min_confidence_bps,
        "min_evidence_count": policy.min_evidence_count,
        "max_limited_cost_minor": policy.max_limited_cost_minor,
        "min_runway_days": policy.min_runway_days,
        "allow_strategic": policy.allow_strategic,
        "emergency_stop": emergency_stop,
        "material_and_external_actions_require_human_approval": true
    })))
}

async fn agent_outcome_evidence_api(
    State(state): State<AppState>,
    Json(request): Json<AgentOutcomeEvidenceRequest>,
) -> Result<Json<company_store::AgentOutcomeEvidenceRecord>, StatusCode> {
    state
        .store
        .record_agent_outcome_evidence(
            &state.company_id,
            request.decision_journal_id,
            &request.evidence_ref,
            request.observed_revenue_delta_minor,
            request.observed_contribution_margin_delta_minor,
            request.observed_at_epoch,
        )
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn agent_outcome_evaluations_api(
    State(state): State<AppState>,
    Query(query): Query<AgentEvaluationQuery>,
) -> Result<Json<Vec<company_agent_evaluation::AgentEvaluation>>, StatusCode> {
    let days = query.days.unwrap_or(30);
    state
        .store
        .agent_outcome_evaluations(&state.company_id, days)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn ceo_command_center_api(
    State(state): State<AppState>,
) -> Result<Json<company_store::CeoCommandCenterRecord>, StatusCode> {
    let target_minor = configured_revenue_target_minor(&state.currency)?;
    state
        .store
        .ceo_command_center(&state.company_id, target_minor)
        .await
        .map(Json)
        .map_err(|error| {
            tracing::warn!(%error, "CEO command center unavailable");
            StatusCode::SERVICE_UNAVAILABLE
        })
}

fn autonomy_policy_from_env() -> Result<company_autonomy::AutonomyPolicy, String> {
    let max_level = parse_autonomy_level(
        &std::env::var("AUTONOMY_MAX_LEVEL").unwrap_or_else(|_| "SIMULATE".into()),
    )?;
    let min_confidence_bps = std::env::var("AUTONOMY_MIN_CONFIDENCE_BPS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8_500);
    let min_evidence_count = std::env::var("AUTONOMY_MIN_EVIDENCE_COUNT")
        .ok()
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(2);
    let max_limited_cost_minor = std::env::var("AUTONOMY_MAX_LIMITED_COST_MINOR")
        .ok()
        .and_then(|value| value.parse::<i128>().ok())
        .unwrap_or(250);
    let min_runway_days = std::env::var("AUTONOMY_MIN_RUNWAY_DAYS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(30);
    let allow_strategic = parse_bool_env("AUTONOMY_ALLOW_STRATEGIC", false);
    company_autonomy::policy_with(
        max_level,
        min_confidence_bps,
        min_evidence_count,
        max_limited_cost_minor,
        min_runway_days,
        allow_strategic,
    )
}

fn parse_autonomy_level(value: &str) -> Result<company_autonomy::AutonomyLevel, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "OBSERVE" => Ok(company_autonomy::AutonomyLevel::Observe),
        "RECOMMEND" => Ok(company_autonomy::AutonomyLevel::Recommend),
        "SIMULATE" => Ok(company_autonomy::AutonomyLevel::Simulate),
        "HUMAN_APPROVE" | "HUMAN-APPROVE" => Ok(company_autonomy::AutonomyLevel::HumanApprove),
        "LIMITED_AUTONOMY" | "LIMITED" => Ok(company_autonomy::AutonomyLevel::LimitedAutonomy),
        "STRATEGIC_AUTONOMY" | "STRATEGIC" => Ok(company_autonomy::AutonomyLevel::StrategicAutonomy),
        _ => Err("invalid AUTONOMY_MAX_LEVEL".into()),
    }
}

fn parse_bool_env(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn autonomy_emergency_stop_from_env() -> Result<bool, String> {
    let raw = std::env::var("AUTONOMY_EMERGENCY_STOP").unwrap_or_else(|_| "false".into());
    let normalized = raw.trim().to_ascii_lowercase();
    if matches!(normalized.as_str(), "1" | "true" | "yes" | "on") {
        Ok(true)
    } else if matches!(normalized.as_str(), "0" | "false" | "no" | "off") {
        Ok(false)
    } else {
        Err("AUTONOMY_EMERGENCY_STOP must be boolean".into())
    }
}

fn configured_revenue_target_minor(currency: &str) -> Result<i128, StatusCode> {
    if let Some(value) = std::env::var("MONTHLY_REVENUE_TARGET_MINOR")
        .ok()
        .and_then(|value| value.parse::<i128>().ok())
        .filter(|value| *value > 0)
    {
        return Ok(value);
    }
    Ok(if currency.eq_ignore_ascii_case("VND") {
        50_000_000
    } else {
        500_000
    })
}

async fn agents_api(State(state): State<AppState>) -> Json<Vec<AgentRunResult>> {
    Json(state.latest.read().await.clone())
}

async fn policy_snapshot_api(
    State(state): State<AppState>,
    Json(request): Json<PolicySnapshotRequest>,
) -> Result<Json<company_compliance::PolicySnapshot>, StatusCode> {
    if request.snapshot.company_id.to_string() != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .record_policy_snapshot(&request.snapshot)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn compliance_check_api(
    State(state): State<AppState>,
    Json(request): Json<ComplianceCheckRequest>,
) -> Result<Json<company_compliance::ComplianceCheck>, StatusCode> {
    if request.input.company_id.to_string() != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .record_compliance_check(&request.input)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn tiktok_oauth_start_api(
    State(state): State<AppState>,
) -> Result<Redirect, StatusCode> {
    if !parse_bool_env("TIKTOK_OAUTH_ENABLED", false) {
        return Err(StatusCode::PRECONDITION_FAILED);
    }
    let config = company_tiktok_auth::OAuthConfig::from_env()
        .map_err(|error| {
            tracing::warn!(%error, "TikTok OAuth is not configured");
            StatusCode::SERVICE_UNAVAILABLE
        })?;
    let state_token = company_tiktok_auth::new_state();
    let state_hash = company_tiktok_auth::hash_state(&state_token);
    let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = now_epoch
        .checked_add(company_tiktok_auth::OAuthConfig::default_state_ttl_seconds())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .store
        .create_tiktok_oauth_state(
            &state.company_id,
            &state_hash,
            &config.redirect_uri,
            &config.scopes,
            expires_at,
        )
        .await
        .map_err(|error| {
            tracing::warn!(%error, "could not persist TikTok OAuth state");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let url = config.authorize_url(&state_token).map_err(|error| {
        tracing::warn!(%error, "could not build TikTok OAuth URL");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    Ok(Redirect::temporary(&url))
}

async fn tiktok_oauth_callback_api(
    State(state): State<AppState>,
    Query(query): Query<TikTokOAuthCallbackQuery>,
) -> Result<Html<String>, StatusCode> {
    let returned_state = query
        .state
        .as_deref()
        .filter(|value| !value.trim().is_empty() && value.len() <= 256)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let config = company_tiktok_auth::OAuthConfig::from_env()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
    let stored = state
        .store
        .consume_tiktok_oauth_state(
            &state.company_id,
            &company_tiktok_auth::hash_state(returned_state),
            now_epoch,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    if stored.0 != config.redirect_uri || stored.1 != config.scopes {
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Some(error) = query.error.as_deref().filter(|value| !value.trim().is_empty()) {
        let detail = query.error_description.as_deref().unwrap_or("authorization was not granted");
        return Ok(Html(format!(
            "<!doctype html><html><body><h2>TikTok connection was not completed</h2><p>{}</p><p>{}</p></body></html>",
            escape_html(error),
            escape_html(detail)
        )));
    }

    let code = query.code.as_deref().filter(|value| !value.trim().is_empty()).ok_or(StatusCode::BAD_REQUEST)?;
    let oauth = company_tiktok_auth::TikTokOAuthClient::new(config)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let token = oauth.exchange_code(code).await.map_err(|error| {
        tracing::warn!(%error, "TikTok OAuth code exchange failed");
        StatusCode::BAD_GATEWAY
    })?;
    let cipher = company_tiktok_auth::TokenCipher::from_env()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    state
        .store
        .save_tiktok_token_set(&state.company_id, &token, &cipher)
        .await
        .map_err(|error| {
            tracing::warn!(%error, "TikTok OAuth token persistence failed");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    Ok(Html(
        "<!doctype html><html><body><h2>TikTok connected</h2><p>You can close this window and return to Company OS.</p></body></html>"
            .into(),
    ))
}

async fn tiktok_oauth_status_api(
    State(state): State<AppState>,
) -> Result<Json<TikTokOAuthStatusResponse>, StatusCode> {
    let connection = state
        .store
        .tiktok_oauth_status(&state.company_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(TikTokOAuthStatusResponse {
        connected: connection.as_ref().is_some_and(|value| value.status == "ACTIVE"),
        connection,
    }))
}

async fn tiktok_refresh_with_config(
    state: &AppState,
    config: &company_tiktok_auth::OAuthConfig,
    cipher: &company_tiktok_auth::TokenCipher,
) -> Result<company_store::TikTokConnectionRecord, company_tiktok_auth::AuthError> {
    let material = state
        .store
        .tiktok_oauth_token_material(&state.company_id, cipher)
        .await
        .map_err(|error| company_tiktok_auth::AuthError::Provider(error.to_string()))?
        .ok_or_else(|| company_tiktok_auth::AuthError::Unauthorized)?;

    let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
    if material.refresh_token_expires_at_epoch <= now_epoch {
        let _ = state
            .store
            .mark_tiktok_reauth_required(&state.company_id, "TikTok refresh token has expired")
            .await;
        return Err(company_tiktok_auth::AuthError::Unauthorized);
    }

    let oauth = company_tiktok_auth::TikTokOAuthClient::new(config.clone())?;
    match oauth.refresh(&material.refresh_token).await {
        Ok(token) => state
            .store
            .save_tiktok_token_set(&state.company_id, &token, cipher)
            .await
            .map_err(|error| company_tiktok_auth::AuthError::Provider(error.to_string())),
        Err(error) => {
            let _ = state
                .store
                .mark_tiktok_reauth_required(&state.company_id, &error.to_string())
                .await;
            Err(error)
        }
    }
}

async fn tiktok_oauth_refresh_api(
    State(state): State<AppState>,
) -> Result<Json<company_store::TikTokConnectionRecord>, StatusCode> {
    if !parse_bool_env("TIKTOK_OAUTH_ENABLED", false) {
        return Err(StatusCode::PRECONDITION_FAILED);
    }
    let config = company_tiktok_auth::OAuthConfig::from_env().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let cipher = company_tiktok_auth::TokenCipher::from_env().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    tiktok_refresh_with_config(&state, &config, &cipher)
        .await
        .map(Json)
        .map_err(|error| {
            tracing::warn!(%error, "TikTok OAuth refresh failed");
            match error {
                company_tiktok_auth::AuthError::Unauthorized => StatusCode::PRECONDITION_FAILED,
                company_tiktok_auth::AuthError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
                _ => StatusCode::BAD_GATEWAY,
            }
        })
}

async fn tiktok_oauth_revoke_api(
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    if !parse_bool_env("TIKTOK_OAUTH_ENABLED", false) {
        return Err(StatusCode::PRECONDITION_FAILED);
    }
    let config = company_tiktok_auth::OAuthConfig::from_env().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let cipher = company_tiktok_auth::TokenCipher::from_env().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if let Some(material) = state
        .store
        .tiktok_oauth_token_material(&state.company_id, &cipher)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    {
        let oauth = company_tiktok_auth::TikTokOAuthClient::new(config).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        oauth.revoke(&material.access_token).await.map_err(|error| {
            tracing::warn!(%error, "TikTok OAuth revoke failed");
            match error {
                company_tiktok_auth::AuthError::Unauthorized => StatusCode::PRECONDITION_FAILED,
                company_tiktok_auth::AuthError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
                _ => StatusCode::BAD_GATEWAY,
            }
        })?;
    }
    state
        .store
        .mark_tiktok_revoked(&state.company_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(StatusCode::NO_CONTENT)
}

fn spawn_tiktok_refresh_worker(state: AppState) {
    let enabled = parse_bool_env("TIKTOK_OAUTH_ENABLED", false);
    if !enabled {
        return;
    }
    let config = match company_tiktok_auth::OAuthConfig::from_env() {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(%error, "TikTok OAuth refresh worker disabled: invalid configuration");
            return;
        }
    };
    let cipher = match company_tiktok_auth::TokenCipher::from_env() {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(%error, "TikTok OAuth refresh worker disabled: token encryption key unavailable");
            return;
        }
    };
    tokio::spawn(async move {
        if let Err(error) = state
            .store
            .ensure_recurring_job(&state.company_id, "tiktok_oauth_refresh", 300)
            .await
        {
            tracing::warn!(%error, "TikTok OAuth refresh scheduler unavailable");
            return;
        }

        loop {
            let claim = match state
                .store
                .claim_due_job(&state.company_id, "tiktok_oauth_refresh")
                .await
            {
                Ok(value) => value,
                Err(error) => {
                    tracing::warn!(%error, "TikTok OAuth refresh scheduler claim failed");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            let Some((job_id, _run_token)) = claim else {
                tokio::time::sleep(Duration::from_secs(15)).await;
                continue;
            };

            let result = async {
                let Some(material) = state
                    .store
                    .tiktok_oauth_token_material(&state.company_id, &cipher)
                    .await?
                else {
                    return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
                };

                let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
                if material.refresh_token_expires_at_epoch <= now_epoch {
                    let _ = state
                        .store
                        .mark_tiktok_reauth_required(
                            &state.company_id,
                            "TikTok refresh token has expired",
                        )
                        .await;
                    return Ok(());
                }

                if material.access_token_expires_at_epoch > now_epoch + 1_800 {
                    return Ok(());
                }

                let oauth = company_tiktok_auth::TikTokOAuthClient::new(config.clone())?;
                match oauth.refresh(&material.refresh_token).await {
                    Ok(token) => {
                        state
                            .store
                            .save_tiktok_token_set(&state.company_id, &token, &cipher)
                            .await?;
                    }
                    Err(error) => {
                        let _ = state
                            .store
                            .mark_tiktok_reauth_required(
                                &state.company_id,
                                &error.to_string(),
                            )
                            .await;
                    }
                }
                Ok(())
            }
            .await;

            if let Err(error) = result {
                tracing::warn!(%error, "TikTok OAuth refresh cycle failed");
                let _ = state.store.release_job_after_failure(job_id).await;
            } else if let Err(error) = state
                .store
                .complete_job(job_id, uuid::Uuid::new_v4())
                .await
            {
                tracing::warn!(%error, "TikTok OAuth refresh scheduler completion failed");
            }
        }
    });
}

async fn compliance_status_api(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .store
        .compliance_status(&state.company_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn revenue_graph_edge_api(
    State(state): State<AppState>,
    Json(request): Json<RevenueGraphEdgeRequest>,
) -> Result<Json<company_revenue_graph::RevenueGraphEdge>, StatusCode> {
    if request.edge.company_id.to_string() != state.company_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .store
        .record_revenue_graph_edge(&request.edge)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn revenue_graph_summary_api(
    State(state): State<AppState>,
) -> Result<Json<company_store::RevenueGraphSummary>, StatusCode> {
    state
        .store
        .revenue_graph_summary(&state.company_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

async fn revenue_graph_lineage_api(
    State(state): State<AppState>,
    Query(query): Query<RevenueGraphLineageQuery>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let root_type = company_revenue_graph::RevenueNodeType::parse(&query.root_type)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let rows = state
        .store
        .revenue_graph_lineage(
            &state.company_id,
            root_type,
            &query.root_ref,
            query.max_depth.unwrap_or(6),
            query.limit.unwrap_or(100),
        )
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(
        rows.into_iter()
            .map(|(depth, edge)| {
                serde_json::json!({
                    "depth": depth,
                    "edge": edge
                })
            })
            .collect(),
    ))
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

async fn growth_trends_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<company_store::GrowthTrendRecord>>, StatusCode> {
    state
        .store
        .list_growth_trends(&state.company_id, 100)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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

async fn create_capital_plan_api(
    State(state): State<AppState>,
    Json(request): Json<CapitalAllocationRequest>,
) -> Result<Json<company_store::CapitalAllocationRecord>, StatusCode> {
    let snapshot = state.company.read().await.clone();
    let mut policy = request.policy;
    policy.company_status = snapshot.status;
    policy.cash_available_minor = snapshot.cash_minor.max(0);
    policy.runway_days = snapshot.runway_days.max(0);
    let env_stop = autonomy_emergency_stop_from_env().unwrap_or(true);
    let controls = state
        .store
        .autonomy_controls(&state.company_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    policy.emergency_stop = policy.emergency_stop || env_stop || controls.controls.emergency_stop.enabled;
    let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
    let capital_remaining = state
        .store
        .autonomy_budget_remaining(
            &state.company_id,
            company_safety_controls::BudgetKind::AutonomousCapital,
            now_epoch,
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    policy.discretionary_budget_minor = policy.discretionary_budget_minor.min(capital_remaining);
    state
        .store
        .create_capital_allocation_plan(
            &state.company_id,
            &request.plan_key,
            &policy,
            &request.candidates,
        )
        .await
        .map(Json)
        .map_err(|error| {
            tracing::warn!(%error, "capital allocation plan rejected");
            StatusCode::BAD_REQUEST
        })
}

async fn latest_capital_plan_api(
    State(state): State<AppState>,
) -> Result<Json<Option<company_store::CapitalAllocationRecord>>, StatusCode> {
    state
        .store
        .latest_capital_allocation_plan(&state.company_id)
        .await
        .map(Json)
        .map_err(|error| {
            tracing::warn!(%error, "capital allocation plan unavailable");
            StatusCode::SERVICE_UNAVAILABLE
        })
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

fn cookie_value(request: &Request, name: &str) -> Option<String> {
    let header = request.headers().get(header::COOKIE)?.to_str().ok()?;
    header.split(';').find_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        (key == name).then(|| value.to_owned())
    })
}

fn browser_cookie_secure() -> bool {
    parse_bool_env("CONTROL_PLANE_COOKIE_SECURE", true)
}

fn browser_secret() -> Result<Vec<u8>, StatusCode> {
    let secret = std::env::var("CONTROL_PLANE_BROWSER_SECRET")
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if secret.len() < 32 {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(secret.into_bytes())
}

fn hmac_hex(secret: &[u8], message: &str) -> Result<String, StatusCode> {
    type BrowserHmac = Hmac<Sha256>;
    let mut mac = BrowserHmac::new_from_slice(secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    mac.update(message.as_bytes());
    let digest = mac.finalize().into_bytes();
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[derive(Debug, Clone)]
struct BrowserSession {
    csrf: String,
    expires_at_epoch: i64,
}

fn sign_browser_session(
    company_id: &str,
    expires_at_epoch: i64,
    csrf: &str,
    secret: &[u8],
) -> Result<String, StatusCode> {
    let payload = format!("{company_id}|{expires_at_epoch}|{csrf}");
    let signature = hmac_hex(secret, &payload)?;
    Ok(format!("v1.{expires_at_epoch}.{csrf}.{signature}"))
}

fn verify_browser_session(
    company_id: &str,
    cookie: &str,
    secret: &[u8],
) -> Option<BrowserSession> {
    let mut parts = cookie.split('.');
    let version = parts.next()?;
    let expires = parts.next()?.parse::<i64>().ok()?;
    let csrf = parts.next()?;
    let signature = parts.next()?;
    if version != "v1" || parts.next().is_some() || csrf.len() != 36 {
        return None;
    }
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if expires <= now || expires > now + 86_400 {
        return None;
    }
    let payload = format!("{company_id}|{expires}|{csrf}");
    let expected = hmac_hex(secret, &payload).ok()?;
    if !bool::from(expected.as_bytes().ct_eq(signature.as_bytes())) {
        return None;
    }
    Some(BrowserSession {
        csrf: csrf.to_owned(),
        expires_at_epoch: expires,
    })
}

fn control_plane_actor_id_for_cookie(cookie: &str) -> String {
    let digest = Sha256::digest(cookie.as_bytes());
    let fingerprint: String = digest
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("browser-session-sha256:{fingerprint}")
}

fn clear_cookie_headers(headers: &mut HeaderMap) {
    let secure = if browser_cookie_secure() { "; Secure" } else { "" };
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "company_os_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{secure}"
        ))
        .expect("static cookie header"),
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "company_os_csrf=; Path=/; SameSite=Strict; Max-Age=0{secure}"
        ))
        .expect("static cookie header"),
    );
}

fn session_cookie_headers(
    company_id: &str,
    expires_at_epoch: i64,
    csrf: &str,
    secret: &[u8],
) -> Result<HeaderMap, StatusCode> {
    let session = sign_browser_session(company_id, expires_at_epoch, csrf, secret)?;
    let secure = if browser_cookie_secure() { "; Secure" } else { "" };
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "company_os_session={session}; Path=/; HttpOnly; SameSite=Strict; Max-Age=28800{secure}"
        ))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "company_os_csrf={csrf}; Path=/; SameSite=Strict; Max-Age=28800{secure}"
        ))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    Ok(headers)
}

fn control_plane_request_id(request: &Request) -> String {
    request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

fn control_plane_auth_scope(
    method: &str,
    provided: Option<&str>,
    operator_token: &str,
    read_token: Option<&str>,
) -> Option<&'static str> {
    let provided = provided.filter(|value| !value.is_empty())?;

    if bool::from(operator_token.as_bytes().ct_eq(provided.as_bytes())) {
        return Some("operator");
    }

    if matches!(method, "GET" | "HEAD")
        && read_token.is_some_and(|token| bool::from(token.as_bytes().ct_eq(provided.as_bytes())))
    {
        return Some("read-only");
    }

    None
}

fn control_plane_actor_id(token: Option<&str>) -> String {
    match token.filter(|value| !value.is_empty()) {
        Some(token) => {
            let digest = Sha256::digest(token.as_bytes());
            let fingerprint: String = digest.iter().take(16).map(|byte| format!("{byte:02x}")).collect();
            format!("bearer-sha256:{fingerprint}")
        }
        None => "anonymous".into(),
    }
}

async fn record_control_plane_audit(
    state: &AppState,
    actor_id: &str,
    actor_role: &str,
    method: &str,
    path: &str,
    action: &str,
    outcome: &str,
    request_id: &str,
) {
    if let Err(error) = state
        .store
        .record_control_plane_audit(
            &state.company_id,
            actor_id,
            actor_role,
            method,
            path,
            action,
            outcome,
            Some(request_id),
        )
        .await
    {
        tracing::warn!(%error, %path, %outcome, "control-plane audit write failed");
    }
}

async fn require_control_plane_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let started = Instant::now();
    let request_id = control_plane_request_id(&request);
    let path = request.uri().path().to_owned();
    if matches!(
        path.as_str(),
        "/healthz"
            | "/readyz"
            | "/metrics"
            | "/api/publishing/tiktok/webhook"
            | "/api/tiktok/oauth/callback"
            | "/auth/login"
            | "/auth/session"
    ) {
        let mut response = next.run(request).await;
        if let Ok(value) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert("x-request-id", value);
        }
        return Ok(response);
    }

    let method = request.method().as_str().to_owned();
    let provided = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if control_plane_auth_disabled() {
        let actor_id = control_plane_actor_id(provided);
        let mut response = next.run(request).await;
        record_control_plane_audit(
            &state,
            &actor_id,
            "development-bypass",
            &method,
            &path,
            "CONTROL_PLANE_REQUEST",
            "ALLOWED",
            &request_id,
        )
        .await;
        let header_value = observe_control_plane_request(&state, &request_id, started);
        response.headers_mut().insert("x-request-id", header_value);
        return Ok(response);
    }

    if let Some(cookie) = cookie_value(&request, "company_os_session") {
        if let Ok(secret) = browser_secret() {
            if let Some(session) = verify_browser_session(&state.company_id, &cookie, &secret) {
                let csrf_cookie = cookie_value(&request, "company_os_csrf");
                let csrf_header = request
                    .headers()
                    .get("x-csrf-token")
                    .and_then(|value| value.to_str().ok())
                    .or_else(|| {
                        request
                            .uri()
                            .query()
                            .and_then(|query| query.split('&').find_map(|pair| {
                                let (key, value) = pair.split_once('=')?;
                                (key == "csrf").then_some(value)
                            }))
                    });
                let csrf_valid = if matches!(method.as_str(), "GET" | "HEAD" | "OPTIONS") {
                    true
                } else {
                    csrf_cookie
                        .as_deref()
                        .is_some_and(|value| bool::from(session.csrf.as_bytes().ct_eq(value.as_bytes())))
                        && csrf_header
                            .is_some_and(|value| bool::from(session.csrf.as_bytes().ct_eq(value.as_bytes())))
                };

                let actor_id = control_plane_actor_id_for_cookie(&cookie);
                if csrf_valid {
                    let mut response = next.run(request).await;
                    record_control_plane_audit(
                        &state,
                        &actor_id,
                        "operator-browser",
                        &method,
                        &path,
                        "CONTROL_PLANE_BROWSER_REQUEST",
                        "ALLOWED",
                        &request_id,
                    )
                    .await;
                    let header_value = observe_control_plane_request(&state, &request_id, started);
                    response.headers_mut().insert("x-request-id", header_value);
                    return Ok(response);
                }

                mark_control_plane_denied(&state, true, started);
                record_control_plane_audit(
                    &state,
                    &actor_id,
                    "operator-browser",
                    &method,
                    &path,
                    "CONTROL_PLANE_BROWSER_CSRF",
                    "DENIED",
                    &request_id,
                )
                .await;
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    let actor_id = control_plane_actor_id(provided);
    let operator_token = match std::env::var("CONTROL_PLANE_TOKEN") {
        Ok(value) if !value.is_empty() => value,
        _ => {
            mark_control_plane_denied(&state, false, started);
            record_control_plane_audit(
                &state,
                &actor_id,
                "operator",
                &method,
                &path,
                "CONTROL_PLANE_REQUEST",
                "DENIED",
                &request_id,
            )
            .await;
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    let read_token = std::env::var("CONTROL_PLANE_READ_TOKEN")
        .ok()
        .filter(|value| !value.is_empty());

    let role = match control_plane_auth_scope(
        &method,
        provided,
        &operator_token,
        read_token.as_deref(),
    ) {
        Some(role) => role,
        None => {
            mark_control_plane_denied(&state, false, started);
            record_control_plane_audit(
                &state,
                &actor_id,
                "operator",
                &method,
                &path,
                "CONTROL_PLANE_REQUEST",
                "DENIED",
                &request_id,
            )
            .await;
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let mut response = next.run(request).await;
    record_control_plane_audit(
        &state,
        &actor_id,
        role,
        &method,
        &path,
        "CONTROL_PLANE_REQUEST",
        "ALLOWED",
        &request_id,
    )
    .await;
    let header_value = observe_control_plane_request(&state, &request_id, started);
    response.headers_mut().insert("x-request-id", header_value);
    Ok(response)
}

async fn browser_login_page() -> Html<String> {
    Html(r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Company OS sign in</title></head><body style="font-family:system-ui;max-width:520px;margin:64px auto;padding:24px"><h1>Company OS</h1><p>Sign in to use the browser control plane. API clients should continue using Bearer authentication.</p><form method="post" action="/auth/session"><label>Control-plane token</label><input name="token" type="password" autocomplete="current-password" required minlength="32" style="display:block;width:100%;box-sizing:border-box;padding:10px;margin:8px 0 14px"><button type="submit">Sign in</button></form></body></html>"#.into())
}

#[derive(Debug, Deserialize)]
struct BrowserLoginForm {
    token: String,
}

async fn browser_login(
    State(state): State<AppState>,
    Form(form): Form<BrowserLoginForm>,
) -> Result<Response, StatusCode> {
    let operator_token = std::env::var("CONTROL_PLANE_TOKEN")
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let valid = bool::from(operator_token.as_bytes().ct_eq(form.token.as_bytes()));
    let request_id = uuid::Uuid::new_v4().to_string();
    let actor_id = control_plane_actor_id(Some(&form.token));
    if !valid {
        record_control_plane_audit(
            &state,
            &actor_id,
            "operator-browser",
            "POST",
            "/auth/session",
            "CONTROL_PLANE_BROWSER_LOGIN",
            "DENIED",
            &request_id,
        )
        .await;
        return Err(StatusCode::UNAUTHORIZED);
    }

    let secret = browser_secret()?;
    let csrf = uuid::Uuid::new_v4().to_string();
    let expires = time::OffsetDateTime::now_utc().unix_timestamp() + 8 * 60 * 60;
    let mut response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/");
    let headers = session_cookie_headers(&state.company_id, expires, &csrf, &secret)?;
    for value in headers.get_all(header::SET_COOKIE).iter() {
        response = response.header(header::SET_COOKIE, value);
    }
    let response = response.body(axum::body::Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    record_control_plane_audit(
        &state,
        &actor_id,
        "operator-browser",
        "POST",
        "/auth/session",
        "CONTROL_PLANE_BROWSER_LOGIN",
        "ALLOWED",
        &request_id,
    )
    .await;
    Ok(response)
}

async fn browser_logout(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    let request_id = control_plane_request_id(&request);
    let actor_id = cookie_value(&request, "company_os_session")
        .map(|cookie| control_plane_actor_id_for_cookie(&cookie))
        .unwrap_or_else(|| "anonymous".into());
    let mut headers = HeaderMap::new();
    clear_cookie_headers(&mut headers);
    record_control_plane_audit(
        &state,
        &actor_id,
        "operator-browser",
        "POST",
        "/auth/logout",
        "CONTROL_PLANE_BROWSER_LOGOUT",
        "ALLOWED",
        &request_id,
    )
    .await;
    let mut response = Response::builder().status(StatusCode::SEE_OTHER).header(header::LOCATION, "/auth/login");
    for value in headers.get_all(header::SET_COOKIE).iter() {
        response = response.header(header::SET_COOKIE, value);
    }
    response.body(axum::body::Body::empty()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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

#[cfg(test)]
mod control_plane_audit_tests {
    use super::*;

    #[test]
    fn control_plane_metrics_expose_request_and_denial_counters() {
        let metrics = RuntimeMetrics::default();
        metrics.control_plane_requests_total.store(3, Ordering::Relaxed);
        metrics.control_plane_auth_denied_total.store(1, Ordering::Relaxed);
        metrics.control_plane_browser_csrf_denied_total.store(1, Ordering::Relaxed);
        metrics.control_plane_last_latency_ms.store(42, Ordering::Relaxed);
        let rendered = metrics.render_prometheus();
        assert!(rendered.contains("company_control_plane_requests_total 3"));
        assert!(rendered.contains("company_control_plane_auth_denied_total 1"));
        assert!(rendered.contains("company_control_plane_browser_csrf_denied_total 1"));
        assert!(rendered.contains("company_control_plane_last_latency_ms 42"));
    }

    #[test]
    fn browser_session_round_trip_and_expiry_are_verified() {
        let secret = b"01234567890123456789012345678901";
        let csrf = uuid::Uuid::new_v4().to_string();
        let expires = time::OffsetDateTime::now_utc().unix_timestamp() + 3_600;
        let cookie = sign_browser_session("company", expires, &csrf, secret).unwrap();
        let session = verify_browser_session("company", &cookie, secret).unwrap();
        assert_eq!(session.csrf, csrf);
        assert_eq!(session.expires_at_epoch, expires);
        assert!(verify_browser_session("other", &cookie, secret).is_none());
    }

    #[test]
    fn browser_cookie_fingerprint_does_not_expose_cookie() {
        let cookie = "v1.123456.test-signature";
        let id = control_plane_actor_id_for_cookie(cookie);
        assert!(id.starts_with("browser-session-sha256:"));
        assert!(!id.contains(cookie));
    }

    #[test]
    fn readiness_renderer_keeps_state_dimensions_explicit() {
        let html = render_integration_readiness(&[IntegrationReadiness {
            key: "demo".into(),
            status: "CONFIGURED".into(),
            configured: true,
            authenticated: false,
            evidence_fresh: false,
            reason: "Credential exists; external acceptance is not verified.".into(),
        }]);
        assert!(html.contains("CONFIGURED"));
        assert!(html.contains("configured=true"));
        assert!(html.contains("authenticated=false"));
        assert!(html.contains("evidence_fresh=false"));
        assert!(html.contains("external acceptance is not verified"));
    }

    fn actor_id_is_a_non_secret_fingerprint() {
        let first = control_plane_actor_id(Some("token-value"));
        let second = control_plane_actor_id(Some("token-value"));
        assert_eq!(first, second);
        assert!(first.starts_with("bearer-sha256:"));
        assert!(!first.contains("token-value"));
        assert_eq!(control_plane_actor_id(None), "anonymous");
    }

    #[test]
    fn read_only_scope_cannot_mutate() {
        assert_eq!(
            control_plane_auth_scope(
                "GET",
                Some("read-token"),
                "operator-token",
                Some("read-token")
            ),
            Some("read-only")
        );
        assert_eq!(
            control_plane_auth_scope(
                "POST",
                Some("read-token"),
                "operator-token",
                Some("read-token")
            ),
            None
        );
        assert_eq!(
            control_plane_auth_scope(
                "POST",
                Some("operator-token"),
                "operator-token",
                Some("read-token")
            ),
            Some("operator")
        );
    }

    fn request_id_uses_safe_header_or_generates_one() {
        let request = Request::builder()
            .uri("/api/run")
            .header("x-request-id", "req-123")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(control_plane_request_id(&request), "req-123");

        let request = Request::builder()
            .uri("/api/run")
            .body(axum::body::Body::empty())
            .unwrap();
        let generated = control_plane_request_id(&request);
        assert!(!generated.is_empty());
    }
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

    spawn_tiktok_refresh_worker(state.clone());

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
        if let Ok(read_token) = std::env::var("CONTROL_PLANE_READ_TOKEN") {
            if !read_token.is_empty() && read_token.len() < 32 {
                return Err("CONTROL_PLANE_READ_TOKEN must be at least 32 bytes".into());
            }
        }
    }

    let app = Router::new()
        .route("/auth/login", get(browser_login_page))
        .route("/auth/session", post(browser_login))
        .route("/auth/logout", post(browser_logout))
        .route("/", get(index))
        .route("/run", post(run_html))
        .route("/live/session", post(live_create_html))
        .route("/live/start", post(live_start_html))
        .route("/live/stop", post(live_stop_html))
        .route("/api/run", post(run_api))
        .route("/api/ceo/command-center", get(ceo_command_center_api))
        .route("/api/agents", get(agents_api))
        .route("/api/agents/outcome-evidence", post(agent_outcome_evidence_api))
        .route("/api/agents/evaluation", get(agent_outcome_evaluations_api))
        .route("/api/control-plane/audit", get(control_plane_audit_api))
        .route("/api/integrations/readiness", get(integrations_readiness_api))
        .route("/api/autonomy/policy", get(autonomy_policy_api))
        .route("/api/autonomy/controls", get(autonomy_controls_get_api).post(autonomy_controls_set_api))
        .route("/api/autonomy/budget/consume", post(autonomy_budget_consume_api))
        .route("/api/autonomy/assess", post(autonomy_assess_api))
        .route("/api/content/items", get(content_list_api).post(content_create_api))
        .route("/api/content/observations", post(content_observation_api))
        .route("/api/content/status", post(content_status_transition_api))
        .route("/api/growth/trends", get(growth_trends_api).post(growth_trend_api))
        .route("/api/revenue-graph/edges", post(revenue_graph_edge_api))
        .route("/api/revenue-graph/summary", get(revenue_graph_summary_api))
        .route("/api/revenue-graph/lineage", get(revenue_graph_lineage_api))
        .route("/api/growth/opportunities", get(growth_opportunities_api))
        .route("/api/growth/content", post(growth_content_api))
        .route("/api/capital/plan", get(latest_capital_plan_api).post(create_capital_plan_api))
        .route("/api/compliance/policies", post(policy_snapshot_api))
        .route("/api/compliance/checks", post(compliance_check_api))
        .route("/api/compliance/status", get(compliance_status_api))
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
        .route("/api/tiktok/oauth/start", get(tiktok_oauth_start_api))
        .route("/api/tiktok/oauth/callback", get(tiktok_oauth_callback_api))
        .route("/api/tiktok/oauth/status", get(tiktok_oauth_status_api))
        .route("/api/tiktok/oauth/refresh", post(tiktok_oauth_refresh_api))
        .route("/api/tiktok/oauth/revoke", post(tiktok_oauth_revoke_api))
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
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, require_control_plane_auth));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8080)).await?;

    println!("Company OS listening on http://localhost:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
