use agent_runtime::{model_from_env, AgentRunResult, AgentRuntime, CompanySnapshot};
use axum::{extract::State, http::StatusCode, response::Html, routing::{get, post}, Json, Router};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    runtime: Arc<AgentRuntime>,
    company: CompanySnapshot,
    latest: Arc<RwLock<Vec<AgentRunResult>>>,
}

fn format_minor(value: i128) -> String {
    let negative = value < 0;
    let absolute = value.unsigned_abs();
    let whole = absolute / 100;
    let cents = absolute % 100;
    let rendered = format!("{}.{:02}", whole, cents);
    if negative { format!("-${}", rendered) } else { format!("${}", rendered) }
}
fn seed_company() -> CompanySnapshot {
    CompanySnapshot {
        company_id: std::env::var("COMPANY_ID").unwrap_or_else(|_| "demo-company".into()),
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

async fn run_cycle(state: &AppState) -> Vec<AgentRunResult> {
    let results = state.runtime.run_all(state.company.clone()).await;
    *state.latest.write().await = results.clone();
    results
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let latest = state.latest.read().await;
    let mut rows = String::new();

    for item in latest.iter() {
        let decision = item.governance
            .as_ref()
            .map(|g| format!("{:?}", g.decision))
            .unwrap_or_else(|| "—".into());

        rows.push_str(&format!(
            "<tr><td>{}</td><td>READY</td><td>{}</td><td>{:?}</td></tr>",
            item.agent.as_str(),
            decision,
            item.proposal.action
        ));
    }

    if rows.is_empty() {
        rows.push_str(r#"<tr><td colspan="4">No cycle has run yet.</td></tr>"#);
    }

    Html(format!(r#"<!doctype html>
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
<small>Rust control plane • 8 operating agents + Governor policy engine</small>
<div class="grid">
<div class="card"><strong>Cash</strong><div>${}</div></div>
<div class="card"><strong>Revenue</strong><div>${}</div></div>
<div class="card"><strong>Expenses</strong><div>${}</div></div>
<div class="card"><strong>Runway</strong><div>{}</div></div>
</div>
<div class="card"><h2>Run agents</h2>
<form method="post" action="/run"><button type="submit">Run one decision cycle</button></form>
<p><small>LLM reasoning is optional; deterministic policy and Governor remain in control.</small></p></div>
<div class="card"><h2>Agents</h2>
<table><tr><th>Agent</th><th>Status</th><th>Governor</th><th>Action</th></tr>{}</table>
</div>
</body></html>"#,
        format_minor(state.company.cash_minor),
        format_minor(state.company.revenue_minor),
        format_minor(state.company.expenses_minor),
        state.company.runway_days,
        rows,
    ))
}

async fn run_html(State(state): State<AppState>) -> (StatusCode, Html<String>) {
    let _ = run_cycle(&state).await;
    (StatusCode::SEE_OTHER, Html(r#"<meta http-equiv="refresh" content="0; url=/" />"#.into()))
}

async fn run_api(State(state): State<AppState>) -> Json<Vec<AgentRunResult>> {
    Json(run_cycle(&state).await)
}

async fn agents_api(State(state): State<AppState>) -> Json<Vec<AgentRunResult>> {
    Json(state.latest.read().await.clone())
}

async fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let runtime = Arc::new(AgentRuntime::new(model_from_env()));
    let state = AppState {
        runtime,
        company: seed_company(),
        latest: Arc::new(RwLock::new(Vec::new())),
    };

    let background = state.clone();
    let interval_secs = std::env::var("AGENT_CYCLE_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v >= 5)
        .unwrap_or(60);

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let _ = run_cycle(&background).await;
        }
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/run", post(run_html))
        .route("/api/run", post(run_api))
        .route("/api/agents", get(agents_api))
        .route("/healthz", get(healthz))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8080))
        .await
        .expect("failed to bind port 8080");

    println!("Company OS listening on http://localhost:8080");
    axum::serve(listener, app).await.expect("server failed");
}
