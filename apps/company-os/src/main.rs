use affiliate_intelligence::{
    now_epoch, AffiliateIntelligence, AffiliateSearchResult, AwinCsvProvider, ProductSearchQuery,
    ProviderError, TikTokShopOpenCollaborationProvider,
};
use agent_runtime::{
    model_from_env, AgentRunResult, AgentRuntime, CompanySnapshot, ExecutionEngine,
    ExecutionOutcome,
};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use company_domain::{ContentAsset, Contract, CreatorUnit, Employee, Experiment, Task};
use company_store::CompanyStore;
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    runtime: Arc<AgentRuntime>,
    company: Arc<RwLock<CompanySnapshot>>,
    latest: Arc<RwLock<Vec<AgentRunResult>>>,
    latest_outcomes: Arc<RwLock<Vec<ExecutionOutcome>>>,
    store: Arc<CompanyStore>,
    cycle_lock: Arc<Mutex<()>>,
    affiliate: Option<Arc<AffiliateIntelligence>>,
}

#[derive(Debug, Deserialize)]
struct AffiliateQueryParams {
    category: Option<String>,
    keywords: Option<String>,
    currency: Option<String>,
    min_commission_bps: Option<u32>,
    require_coupon: Option<bool>,
    min_rating_bps: Option<u32>,
    min_reviews: Option<u64>,
    min_stock: Option<u64>,
    max_price_minor: Option<i128>,
    max_content_cost_minor: Option<i128>,
    limit: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
struct ValueError {
    error: String,
}

fn format_minor(value: i128) -> String {
    let negative = value < 0;
    let absolute = value.unsigned_abs();
    let whole = absolute / 100;
    let cents = absolute % 100;
    let rendered = format!("{}.{:02}", whole, cents);
    if negative {
        format!("-{}", rendered)
    } else {
        rendered
    }
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

fn build_affiliate_intelligence() -> Option<Arc<AffiliateIntelligence>> {
    let mut providers: Vec<Box<dyn affiliate_intelligence::AffiliateProvider>> = Vec::new();

    if std::env::var("AWIN_FEED_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .is_some()
    {
        match AwinCsvProvider::from_env() {
            Ok(provider) => providers.push(Box::new(provider)),
            Err(error) => eprintln!("Awin disabled: {error}"),
        }
    }

    if std::env::var("TIKTOK_APP_KEY").is_ok() {
        match TikTokShopOpenCollaborationProvider::from_env() {
            Ok(provider) => providers.push(Box::new(provider)),
            Err(error) => eprintln!("TikTok Shop disabled: {error}"),
        }
    }

    if providers.is_empty() {
        None
    } else {
        Some(Arc::new(AffiliateIntelligence::new(providers)))
    }
}

async fn run_cycle_with_id(
    state: &AppState,
    cycle_id: String,
) -> Result<Vec<AgentRunResult>, Box<dyn std::error::Error + Send + Sync>> {
    let _guard = state.cycle_lock.lock().await;

    if let Some(existing) = state.store.load_cycle_results(&cycle_id).await? {
        let company_id = state.company.read().await.company_id.clone();
        if let Some(snapshot) = state.store.load_snapshot(&company_id).await? {
            *state.company.write().await = snapshot;
        }
        *state.latest.write().await = existing.clone();
        return Ok(existing);
    }

    let company = state.company.read().await.clone();
    let mut results = state.runtime.run_all(company.clone()).await;
    let mut next_company = company;
    let outcomes = ExecutionEngine::default().execute_batch(&mut next_company, &mut results);

    let persisted = state
        .store
        .persist_decision_cycle(&next_company, &cycle_id, &results, &outcomes)
        .await?;

    match persisted {
        company_store::PersistCycleResult::AlreadyProcessed => {
            if let Some(existing) = state.store.load_cycle_results(&cycle_id).await? {
                *state.latest.write().await = existing.clone();
                if let Some(snapshot) = state.store.load_snapshot(&next_company.company_id).await? {
                    *state.company.write().await = snapshot;
                }
                return Ok(existing);
            }
            Err("cycle was already processed but durable results are unavailable".into())
        }
        company_store::PersistCycleResult::InProgress => {
            Err("cycle is already being processed by another Company OS instance".into())
        }
        company_store::PersistCycleResult::Committed => {
            *state.company.write().await = next_company;
            *state.latest.write().await = results.clone();
            *state.latest_outcomes.write().await = outcomes;
            Ok(results)
        }
    }
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let company = state.company.read().await.clone();
    let latest = state.latest.read().await;
    let outcomes = state.latest_outcomes.read().await;
    let mut rows = String::new();

    for (index, item) in latest.iter().enumerate() {
        let decision = item
            .governance
            .as_ref()
            .map(|g| format!("{:?}", g.decision))
            .unwrap_or_else(|| "—".into());
        let executed = outcomes.get(index).map(|o| o.executed).unwrap_or(false);

        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td></tr>",
            item.agent.as_str(),
            if executed { "EXECUTED" } else { "NOT EXECUTED" },
            decision,
            item.proposal.action
        ));
    }

    if rows.is_empty() {
        rows.push_str(r#"<tr><td colspan="4">No cycle has run yet.</td></tr>"#);
    }

    Html(format!(
        r#"<!doctype html>
<html lang="en"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Company OS</title>
<style>
body{{font-family:system-ui,sans-serif;max-width:1150px;margin:40px auto;padding:0 20px;background:#fafafa}}
.card{{background:#fff;border:1px solid #ddd;border-radius:12px;padding:20px;margin:16px 0}}
.grid{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}}
table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:10px;border-bottom:1px solid #eee}}
button{{padding:10px 14px;border:1px solid #bbb;border-radius:8px;background:#fff;cursor:pointer}}
small{{color:#666}}
</style></head><body>
<h1>Company OS</h1>
<small>Rust control plane • governed execution • durable decision journal • affiliate intelligence</small>
<div class="grid">
<div class="card"><strong>Cash</strong><div>{}</div></div>
<div class="card"><strong>Revenue</strong><div>{}</div></div>
<div class="card"><strong>Expenses</strong><div>{}</div></div>
<div class="card"><strong>Runway</strong><div>{}</div></div>
</div>
<div class="card"><h2>Run agents</h2>
<form method="post" action="/run"><button type="submit">Run one decision cycle</button></form>
<p><small>LLM reasoning is advisory metadata. Governor and deterministic execution remain authoritative.</small></p></div>
<div class="card"><h2>Agents</h2>
<table><tr><th>Agent</th><th>Execution</th><th>Governor</th><th>Action</th></tr>{}</table>
</div>
</body></html>"#,
        format_minor(company.cash_minor),
        format_minor(company.revenue_minor),
        format_minor(company.expenses_minor),
        company.runway_days,
        rows,
    ))
}

async fn run_html(State(state): State<AppState>) -> (StatusCode, Html<String>) {
    match run_cycle(&state).await {
        Ok(_) => (
            StatusCode::SEE_OTHER,
            Html(r#"<meta http-equiv="refresh" content="0; url=/" />"#.into()),
        ),
        Err(error) => {
            eprintln!("cycle error: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("cycle failed safely; inspect logs".into()),
            )
        }
    }
}

async fn run_api(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AgentRunResult>>, (StatusCode, Json<ValueError>)> {
    let cycle_id = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    if cycle_id.len() > 200 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ValueError {
                error: "Idempotency-Key must be <= 200 characters".into(),
            }),
        ));
    }

    run_cycle_with_id(&state, cycle_id)
        .await
        .map(Json)
        .map_err(|error| {
            eprintln!("api cycle error: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ValueError {
                    error: "agent cycle failed safely".into(),
                }),
            )
        })
}

async fn agents_api(State(state): State<AppState>) -> Json<Vec<AgentRunResult>> {
    Json(state.latest.read().await.clone())
}

async fn save_business_unit(
    State(state): State<AppState>,
    Json(unit): Json<company_domain::BusinessUnit>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_business_unit(&company_id, &unit)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_customer(
    State(state): State<AppState>,
    Json(customer): Json<company_domain::Customer>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_customer(&company_id, &customer)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_product(
    State(state): State<AppState>,
    Json(product): Json<company_domain::Product>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_product(&company_id, &product)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_payroll(
    State(state): State<AppState>,
    Json(payroll): Json<company_domain::PayrollRun>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_payroll_run(&company_id, &payroll)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_creator(
    State(state): State<AppState>,
    Json(creator): Json<CreatorUnit>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_creator(&company_id, &creator)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_content_asset(
    State(state): State<AppState>,
    Json(content): Json<ContentAsset>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_content_asset(&company_id, &content)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_experiment(
    State(state): State<AppState>,
    Json(experiment): Json<Experiment>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_experiment(&company_id, &experiment)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_employee(
    State(state): State<AppState>,
    Json(employee): Json<Employee>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_employee(&company_id, &employee)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_contract(
    State(state): State<AppState>,
    Json(contract): Json<Contract>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_contract(&company_id, &contract)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn save_task(
    State(state): State<AppState>,
    Json(task): Json<Task>,
) -> Result<StatusCode, (StatusCode, Json<ValueError>)> {
    let company_id = state.company.read().await.company_id.clone();
    state
        .store
        .save_task(&company_id, &task)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ValueError {
                    error: e.to_string(),
                }),
            )
        })
}

async fn affiliate_search(
    State(state): State<AppState>,
    Query(params): Query<AffiliateQueryParams>,
) -> Result<Json<AffiliateSearchResult>, (StatusCode, Json<ValueError>)> {
    let intelligence = state.affiliate.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ValueError {
                error: "no affiliate provider is configured".into(),
            }),
        )
    })?;

    let query = ProductSearchQuery {
        category: params.category,
        keywords: params
            .keywords
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_string)
            .collect(),
        currency: params.currency.map(|v| v.to_ascii_uppercase()),
        min_commission_bps: params.min_commission_bps,
        require_coupon: params.require_coupon.unwrap_or(false),
        min_rating_bps: params.min_rating_bps,
        min_reviews: params.min_reviews,
        min_stock: params.min_stock,
        max_price_minor: params.max_price_minor,
        max_content_cost_minor: params.max_content_cost_minor.unwrap_or(0).max(0),
        limit: params.limit.unwrap_or(20),
        now_epoch: now_epoch(),
    };

    match intelligence.search(query.clone()).await {
        Ok(result) => {
            let company_id = state.company.read().await.company_id.clone();
            if let Err(error) = state
                .store
                .record_affiliate_search(&company_id, &query, &result)
                .await
            {
                eprintln!("affiliate search persistence error: {error}");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ValueError {
                        error: "affiliate research could not be persisted".into(),
                    }),
                ));
            }
            Ok(Json(result))
        }
        Err(error) => {
            let status = match &error {
                ProviderError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                ProviderError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
                ProviderError::Configuration(_) => StatusCode::BAD_REQUEST,
                ProviderError::InvalidData(_) | ProviderError::Transport(_) => {
                    StatusCode::BAD_GATEWAY
                }
            };
            Err((
                status,
                Json(ValueError {
                    error: error.to_string(),
                }),
            ))
        }
    }
}

async fn affiliate_providers(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(
        state
            .affiliate
            .as_ref()
            .map(|a| a.provider_names())
            .unwrap_or_default(),
    )
}

async fn outbox_worker(
    store: Arc<CompanyStore>,
    client: reqwest::Client,
    webhook_url: Option<String>,
    bearer: Option<String>,
    owner: String,
) {
    loop {
        if let Some(url) = webhook_url.as_deref() {
            match store.claim_outbox_events(&owner, 10, 120).await {
                Ok(events) => {
                    for event in events {
                        let mut request = client.post(url).json(&event);
                        if let Some(token) = bearer.as_deref() {
                            request = request.bearer_auth(token);
                        }
                        let result = request.send().await.and_then(|response| {
                            if response.status().is_success() {
                                Ok(())
                            } else {
                                Err(reqwest::Error::from(
                                    reqwest::StatusCode::INTERNAL_SERVER_ERROR
                                ))
                            }
                        });

                        match result {
                            Ok(()) => {
                                let _ = store.mark_outbox_published(event.id).await;
                            }
                            Err(error) => {
                                let _ = store.fail_outbox_event(
                                    event.id,
                                    &owner,
                                    &error.to_string(),
                                    30,
                                ).await;
                            }
                        }
                    }
                }
                Err(error) => eprintln!("outbox claim error: {error}"),
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let company_id = std::env::var("COMPANY_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".into());
    let company_name = std::env::var("COMPANY_NAME").unwrap_or_else(|_| "Demo Company".into());
    let currency = std::env::var("COMPANY_CURRENCY").unwrap_or_else(|_| "USD".into());

    let store = Arc::new(CompanyStore::connect(&database_url).await?);
    store.migrate().await?;
    let recovered = store.recover_stale_cycles(900).await?;
    if recovered > 0 {
        eprintln!("recovered {recovered} stale control-plane records");
    }
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

    let runtime = Arc::new(AgentRuntime::new(model_from_env()));
    let state = AppState {
        runtime,
        company: Arc::new(RwLock::new(company)),
        latest: Arc::new(RwLock::new(Vec::new())),
        latest_outcomes: Arc::new(RwLock::new(Vec::new())),
        store,
        cycle_lock: Arc::new(Mutex::new(())),
        affiliate: build_affiliate_intelligence(),
    };

    let interval_secs = std::env::var("AGENT_CYCLE_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v >= 15)
        .unwrap_or(300);
    state
        .store
        .ensure_cycle_schedule(&company_id, interval_secs as i64)
        .await?;

    let outbox_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let outbox_store = state.store.clone();
    let outbox_url = std::env::var("OUTBOX_WEBHOOK_URL").ok().filter(|v| !v.trim().is_empty());
    let outbox_bearer = std::env::var("OUTBOX_WEBHOOK_BEARER").ok().filter(|v| !v.trim().is_empty());
    let outbox_owner = format!("company-os:{}", Uuid::new_v4());
    tokio::spawn(outbox_worker(
        outbox_store,
        outbox_client,
        outbox_url,
        outbox_bearer,
        outbox_owner,
    ));

    let background = state.clone();
    let scheduled_company_id = company_id.clone();
    tokio::spawn(async move {
        loop {
            if let Err(error) = background.store.recover_stale_cycles(900).await {
                eprintln!("scheduler recovery error: {error}");
            }

            match background
                .store
                .claim_due_cycle_for(&scheduled_company_id)
                .await
            {
                Ok(true) => {
                    if let Err(error) = run_cycle(&background).await {
                        eprintln!("scheduled agent cycle error: {error}");
                        if let Err(retry_error) = background
                            .store
                            .retry_cycle_schedule(&scheduled_company_id, 30)
                            .await
                        {
                            eprintln!("scheduler retry error: {retry_error}");
                        }
                    }
                }
                Ok(false) => {}
                Err(error) => eprintln!("scheduler error: {error}"),
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/run", post(run_html))
        .route("/api/run", post(run_api))
        .route("/api/agents", get(agents_api))
        .route("/api/business-units", post(save_business_unit))
        .route("/api/customers", post(save_customer))
        .route("/api/products", post(save_product))
        .route("/api/payroll", post(save_payroll))
        .route("/api/creators", post(save_creator))
        .route("/api/content", post(save_content_asset))
        .route("/api/experiments", post(save_experiment))
        .route("/api/employees", post(save_employee))
        .route("/api/contracts", post(save_contract))
        .route("/api/tasks", post(save_task))
        .route("/api/affiliate/search", get(affiliate_search))
        .route("/api/affiliate/providers", get(affiliate_providers))
        .route("/healthz", get(healthz))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 8080)).await?;

    println!("Company OS listening on http://127.0.0.1:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
