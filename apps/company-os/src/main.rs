use affiliate_intelligence::{
    search as search_affiliate, AffiliateProvider, AwinProvider, MockProvider, ProductSearchQuery,
    SearchResponse,
};
use agent_runtime::{model_from_env, AgentRunResult, AgentRuntime, CompanySnapshot};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use company_store::{CompanyStore, PersistedCycle};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, RwLock};

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
}

#[derive(Debug, Serialize)]
struct CycleResponse {
    snapshot: CompanySnapshot,
    results: Vec<AgentRunResult>,
    receipts: Vec<company_execution::ExecutionReceipt>,
}

#[derive(Debug, Deserialize)]
struct AffiliateConversionRequest {
    event: affiliate_attribution::ConversionEvent,
    model: affiliate_attribution::AttributionModel,
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
    min_rating_bps: Option<u32>,
    min_reviews: Option<u64>,
    in_stock_only: Option<bool>,
    max_results: Option<usize>,
    as_of_date: Option<String>,
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
    let company = state.company.read().await.clone();

    let memory_limit = std::env::var("AGENT_MEMORY_LIMIT")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .map(|value| value.clamp(1, 100))
        .unwrap_or(20);
    let memory = state
        .store
        .load_all_agent_memory(&state.company_id, memory_limit)
        .await?;

    let rate_window_seconds = std::env::var("AGENT_RATE_WINDOW_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .map(|value| value.clamp(15, 86_400))
        .unwrap_or(300);
    let max_calls = std::env::var("AGENT_MAX_CALLS_PER_WINDOW")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .map(|value| value.clamp(1, 100))
        .unwrap_or(1);

    let allowed_roles = state
        .store
        .claim_agent_run_slots(
            &state.company_id,
            &state.runtime.agent_roles(),
            rate_window_seconds,
            max_calls,
        )
        .await?;

    let timeout_ms = std::env::var("MODEL_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (250..=120_000).contains(value))
        .unwrap_or(15_000);

    let results = state
        .runtime
        .run_roles_with_memory(
            company.clone(),
            Duration::from_millis(timeout_ms),
            allowed_roles,
            &memory,
        )
        .await;
    let persisted = state
        .store
        .persist_and_execute_cycle_with_id(&company, &results, cycle_id)
        .await?;

    *state.company.write().await = persisted.snapshot.clone();
    *state.latest.write().await = persisted.results.clone();
    *state.latest_cycle.write().await = Some(persisted.clone());

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
        min_rating_bps: params.min_rating_bps,
        min_reviews: params.min_reviews,
        in_stock_only: params.in_stock_only.unwrap_or(true),
        max_results: params.max_results.unwrap_or(20),
        as_of_date: params
            .as_of_date
            .or_else(|| Some(time::OffsetDateTime::now_utc().date().to_string())),
    }
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let company = state.company.read().await.clone();
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
body{{font-family:system-ui,sans-serif;max-width:1200px;margin:40px auto;padding:0 20px;background:#fafafa}}
.card{{background:#fff;border:1px solid #ddd;border-radius:12px;padding:20px;margin:16px 0}}
.grid{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}}
table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:10px;border-bottom:1px solid #eee}}
button{{padding:10px 14px;border:1px solid #bbb;border-radius:8px;background:#fff;cursor:pointer}}
small{{color:#666}}
code{{background:#f2f2f2;padding:2px 5px;border-radius:5px}}
</style></head><body>
<h1>Company OS</h1>
<small>Rust control plane • deterministic Governor + bounded execution + affiliate intelligence</small>
<div class="grid">
<div class="card"><strong>Cash</strong><div>{}</div></div>
<div class="card"><strong>Revenue</strong><div>{}</div></div>
<div class="card"><strong>Expenses</strong><div>{}</div></div>
<div class="card"><strong>Runway</strong><div>{}</div></div>
</div>
<div class="card"><strong>Status:</strong> {:?} &nbsp; <strong>Executed/Noop:</strong> {}</div>
<div class="card"><h2>Run agents</h2>
<form method="post" action="/run"><button type="submit">Run one decision cycle</button></form>
<p><small>The LLM only proposes reasoning. Governor, execution policy, idempotency and persistent state remain deterministic.</small></p></div>
<div class="card"><h2>Agents</h2>
<table><tr><th>Agent</th><th>Governor</th><th>Execution</th><th>Action</th></tr>{}</table>
</div>
<div class="card"><h2>Affiliate Intelligence</h2>
<p>Search live Awin feed data when <code>AFFILIATE_PROVIDER=awin</code>; local mock data is used by default.</p>
<small>Example: <code>/api/affiliate/search?category=electronics&amp;min_commission_bps=1500&amp;require_coupon=true</code></small>
</div>
</body></html>"#,
        format_minor(company.cash_minor, &state.currency),
        format_minor(company.revenue_minor, &state.currency),
        format_minor(company.expenses_minor, &state.currency),
        company.runway_days,
        company.status,
        executed,
        rows,
    ))
}

async fn run_html(State(state): State<AppState>) -> (StatusCode, Html<String>) {
    match run_cycle(&state, &uuid::Uuid::new_v4().to_string()).await {
        Ok(_) => (
            StatusCode::SEE_OTHER,
            Html(r#"<meta http-equiv="refresh" content="0; url=/" />"#.into()),
        ),
        Err(error) => {
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

async fn affiliate_search_api(
    State(state): State<AppState>,
    Query(params): Query<AffiliateSearchParams>,
) -> Result<Json<SearchResponse>, StatusCode> {
    search_affiliate(state.affiliate.clone(), affiliate_query(params))
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
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

async fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let affiliate_mode = std::env::var("AFFILIATE_PROVIDER").unwrap_or_else(|_| "mock".into());
    let affiliate: Arc<dyn AffiliateProvider> = match affiliate_mode.to_ascii_lowercase().as_str() {
        "mock" => Arc::new(MockProvider::default()),
        "awin" => Arc::new(AwinProvider::from_env()?),
        other => return Err(format!("unknown AFFILIATE_PROVIDER={other}").into()),
    };

    let runtime = Arc::new(AgentRuntime::new(model_from_env()));
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

    let app = Router::new()
        .route("/", get(index))
        .route("/run", post(run_html))
        .route("/api/run", post(run_api))
        .route("/api/agents", get(agents_api))
        .route("/api/affiliate/search", get(affiliate_search_api))
        .route("/api/affiliate/click", post(affiliate_click_api))
        .route("/api/affiliate/conversion", post(affiliate_conversion_api))
        .route("/api/affiliate/performance", get(affiliate_performance_api))
        .route("/api/journal", get(journal_api))
        .route("/healthz", get(healthz))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8080)).await?;

    println!("Company OS listening on http://localhost:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
