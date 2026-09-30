#![forbid(unsafe_code)]

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{env, sync::Arc, time::{Duration, Instant}};
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

const MAX_REQUEST_BYTES: usize = 512 * 1024;
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_ERROR_BYTES: usize = 4096;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Client>>,
    api_token: String,
    worker_token: String,
    wait_timeout: Duration,
    job_ttl: Duration,
    lease: Duration,
    max_attempts: i32,
}

#[derive(Debug, Deserialize)]
struct GenerateRequest {
    protocol_version: u16,
    backend: String,
    model: String,
    system: String,
    user: String,
    response_format: String,
    allow_tools: bool,
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct GenerateResponse {
    job_id: Uuid,
    output: String,
}

#[derive(Debug, Serialize)]
struct WorkerJob {
    job_id: Uuid,
    protocol_version: u16,
    backend: String,
    model: String,
    system: String,
    user: String,
    response_format: String,
    allow_tools: bool,
    attempt: i32,
    lease_token: Uuid,
    lease_expires_in_ms: u64,
    expires_in_ms: u64,
}

#[derive(Debug, Deserialize)]
struct ClaimRequest {
    backend: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CompleteRequest {
    lease_token: Uuid,
    output: String,
}

#[derive(Debug, Deserialize)]
struct FailRequest {
    lease_token: Uuid,
    error: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
}

#[derive(Debug)]
enum ApiError {
    Unauthorized,
    BadRequest,
    NotFound,
    Conflict,
    PayloadTooLarge,
    GatewayTimeout,
    Internal,
}

impl ApiError {
    fn body(&self) -> ErrorBody {
        ErrorBody {
            error: match self {
                Self::Unauthorized => "unauthorized",
                Self::BadRequest => "bad_request",
                Self::NotFound => "not_found",
                Self::Conflict => "conflict",
                Self::PayloadTooLarge => "payload_too_large",
                Self::GatewayTimeout => "gateway_timeout",
                Self::Internal => "internal_error",
            },
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status(), Json(self.body())).into_response()
    }
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("authorization")?.to_str().ok()?;
    value.strip_prefix("Bearer ")
}

fn authorize(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let supplied = bearer(headers).ok_or(ApiError::Unauthorized)?;
    if supplied != expected {
        return Err(ApiError::Unauthorized);
    }
    Ok(())
}

fn validate_backend(backend: &str) -> bool {
    matches!(backend, "gemini-web" | "chatgpt-web" | "claude-web")
}

fn request_hash(request: &GenerateRequest) -> String {
    let canonical = format!(
        "{}\0{}\0{}\0{}\0{}\0{}",
        request.backend,
        request.model,
        request.system,
        request.user,
        request.response_format,
        request.allow_tools
    );
    format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
}

fn validate_request(request: &GenerateRequest) -> Result<(), ApiError> {
    if request.protocol_version != 1
        || !validate_backend(request.backend.as_str())
        || request.response_format != "json_object"
        || request.allow_tools
    {
        return Err(ApiError::BadRequest);
    }
    if request.model.trim().is_empty() || request.model.len() > 128 {
        return Err(ApiError::BadRequest);
    }
    if request.system.len() > 128 * 1024 || request.user.len() > 384 * 1024 {
        return Err(ApiError::PayloadTooLarge);
    }
    let total = request.system.len().saturating_add(request.user.len());
    if total > MAX_REQUEST_BYTES {
        return Err(ApiError::PayloadTooLarge);
    }
    if let Some(key) = &request.idempotency_key {
        if key.trim().is_empty() || key.len() > 128 {
            return Err(ApiError::BadRequest);
        }
    }
    Ok(())
}

fn validate_output(output: &str) -> Result<(), ApiError> {
    if output.len() > MAX_RESPONSE_BYTES {
        return Err(ApiError::PayloadTooLarge);
    }
    let value: Value = serde_json::from_str(output).map_err(|_| ApiError::BadRequest)?;
    if !value.is_object() {
        return Err(ApiError::BadRequest);
    }
    Ok(())
}

async fn generate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, ApiError> {
    authorize(&headers, &state.api_token)?;
    validate_request(&request)?;

    let request_hash = request_hash(&request);
    let idempotency_key = request
        .idempotency_key
        .unwrap_or_else(|| format!("web-relay:{}", Uuid::new_v4()));
    let job_id = Uuid::new_v4();

    {
        let client = state.db.lock().await;
        if let Some(row) = client
            .query_opt(
                "SELECT id, status, output_json, expires_at <= now(), request_hash
                   FROM llm_web_relay_jobs
                  WHERE idempotency_key = $1",
                &[&idempotency_key],
            )
            .await
            .map_err(|_| ApiError::Internal)?
        {
            let existing_id: Uuid = row.get(0);
            let status: String = row.get(1);
            let output: Option<String> = row.get(2);
            let expired: bool = row.get(3);
            let stored_hash: Option<String> = row.get(4);
            drop(client);

            if stored_hash.as_deref() != Some(request_hash.as_str()) {
                return Err(ApiError::Conflict);
            }

            if status == "SUCCEEDED" {
                return Ok(Json(GenerateResponse {
                    job_id: existing_id,
                    output: output.ok_or(ApiError::Internal)?,
                }));
            }
            if expired {
                return Err(ApiError::GatewayTimeout);
            }
            return wait_for_job(&state, existing_id).await.map(Json);
        }
    }

    {
        let client = state.db.lock().await;
        let inserted = client
            .execute(
                "INSERT INTO llm_web_relay_jobs
                 (id, idempotency_key, request_hash, backend, model, system_prompt, user_prompt,
                  response_format, allow_tools, status, attempt, max_attempts,
                  expires_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'QUEUED',0,$10,
                         now() + ($11::double precision * interval '1 second'))",
                &[
                    &job_id,
                    &idempotency_key,
                    &request_hash,
                    &request.backend,
                    &request.model,
                    &request.system,
                    &request.user,
                    &request.response_format,
                    &request.allow_tools,
                    &state.max_attempts,
                    &(state.job_ttl.as_secs_f64()),
                ],
            )
            .await;

        if inserted.is_err() {
            if let Some(row) = client
                .query_opt(
                    "SELECT id, status, output_json, expires_at <= now(), request_hash
                       FROM llm_web_relay_jobs
                      WHERE idempotency_key = $1",
                    &[&idempotency_key],
                )
                .await
                .map_err(|_| ApiError::Internal)?
            {
                let existing_id: Uuid = row.get(0);
                let status: String = row.get(1);
                let output: Option<String> = row.get(2);
                let expired: bool = row.get(3);
                let stored_hash: Option<String> = row.get(4);
                if stored_hash.as_deref() != Some(request_hash.as_str()) {
                    return Err(ApiError::Conflict);
                }

                if status == "SUCCEEDED" {
                    return Ok(Json(GenerateResponse {
                        job_id: existing_id,
                        output: output.ok_or(ApiError::Internal)?,
                    }));
                }
                if expired {
                    return Err(ApiError::GatewayTimeout);
                }
                return wait_for_job(&state, existing_id).await.map(Json);
            }

            return Err(ApiError::Conflict);
        }
    }

    wait_for_job(&state, job_id).await.map(Json)
}

async fn wait_for_job(
    state: &AppState,
    job_id: Uuid,
) -> Result<GenerateResponse, ApiError> {
    let deadline = Instant::now() + state.wait_timeout;

    loop {
        let row = {
            let client = state.db.lock().await;
            client
                .query_opt(
                    "SELECT status, output_json
                       FROM llm_web_relay_jobs
                      WHERE id = $1",
                    &[&job_id],
                )
                .await
                .map_err(|_| ApiError::Internal)?
        };

        let Some(row) = row else {
            return Err(ApiError::NotFound);
        };
        let status: String = row.get(0);

        match status.as_str() {
            "SUCCEEDED" => {
                let output: Option<String> = row.get(1);
                return Ok(GenerateResponse {
                    job_id,
                    output: output.ok_or(ApiError::Internal)?,
                });
            }
            "FAILED" | "CANCELLED" => return Err(ApiError::Conflict),
            "EXPIRED" => return Err(ApiError::GatewayTimeout),
            "QUEUED" | "RUNNING" => {}
            _ => return Err(ApiError::Internal),
        }

        if Instant::now() >= deadline {
            return Err(ApiError::GatewayTimeout);
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ClaimRequest>,
) -> Result<Json<Option<WorkerJob>>, ApiError> {
    authorize(&headers, &state.worker_token)?;

    let mut client = state.db.lock().await;
    let tx = client.transaction().await.map_err(|_| ApiError::Internal)?;

    tx.execute(
        "UPDATE llm_web_relay_jobs
            SET status='EXPIRED', lease_token=NULL, locked_until=NULL,
                error_message='job expired before completion'
          WHERE status IN ('QUEUED','RUNNING')
            AND expires_at <= now()",
        &[],
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let row = tx
        .query_opt(
            "SELECT id, backend, model, system_prompt, user_prompt,
                    response_format, allow_tools, attempt
               FROM llm_web_relay_jobs
              WHERE expires_at > now()
                AND (
                  status='QUEUED'
                  OR (status='RUNNING' AND locked_until <= now())
                )
                AND ($1::text IS NULL OR backend=$1)
              ORDER BY created_at ASC, id ASC
              FOR UPDATE SKIP LOCKED
              LIMIT 1",
            &[&request.backend],
        )
        .await
        .map_err(|_| ApiError::Internal)?;

    let Some(row) = row else {
        tx.rollback().await.map_err(|_| ApiError::Internal)?;
        return Ok(Json(None));
    };

    let job_id: Uuid = row.get(0);
    let backend: String = row.get(1);
    let model: String = row.get(2);
    let system: String = row.get(3);
    let user: String = row.get(4);
    let response_format: String = row.get(5);
    let allow_tools: bool = row.get(6);
    let attempt: i32 = row.get(7);
    let lease_token = Uuid::new_v4();

    let row = tx
        .query_one(
            "UPDATE llm_web_relay_jobs
                SET status='RUNNING',
                    attempt=attempt+1,
                    lease_token=$2,
                    locked_until=now() + ($3::double precision * interval '1 second'),
                    updated_at=now()
              WHERE id=$1
              RETURNING attempt",
            &[&job_id, &lease_token, &state.lease.as_secs_f64()],
        )
        .await
        .map_err(|_| ApiError::Internal)?;
    let attempt: i32 = row.get(0);

    let expires_ms = tx
        .query_one(
            "SELECT GREATEST(0, FLOOR(EXTRACT(EPOCH FROM (expires_at-now()))*1000))::bigint
               FROM llm_web_relay_jobs
              WHERE id=$1",
            &[&job_id],
        )
        .await
        .map_err(|_| ApiError::Internal)?
        .get::<_, i64>(0);

    tx.commit().await.map_err(|_| ApiError::Internal)?;

    Ok(Json(Some(WorkerJob {
        job_id,
        protocol_version: 1,
        backend,
        model,
        system,
        user,
        response_format,
        allow_tools,
        attempt,
        lease_token,
        lease_expires_in_ms: state.lease.as_millis().min(u64::MAX as u128) as u64,
        expires_in_ms: expires_ms.max(0) as u64,
    })))
}

async fn complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(job_id): Path<Uuid>,
    Json(request): Json<CompleteRequest>,
) -> Result<StatusCode, ApiError> {
    authorize(&headers, &state.worker_token)?;
    validate_output(&request.output)?;

    let client = state.db.lock().await;
    let changed = client
        .execute(
            "UPDATE llm_web_relay_jobs
                SET status='SUCCEEDED',
                    output_json=$3,
                    lease_token=NULL,
                    locked_until=NULL,
                    error_message=NULL,
                    updated_at=now()
              WHERE id=$1 AND status='RUNNING' AND lease_token=$2
                AND expires_at > now()",
            &[&job_id, &request.lease_token, &request.output],
        )
        .await
        .map_err(|_| ApiError::Internal)?;

    if changed == 0 {
        return Err(ApiError::Conflict);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn fail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(job_id): Path<Uuid>,
    Json(request): Json<FailRequest>,
) -> Result<StatusCode, ApiError> {
    authorize(&headers, &state.worker_token)?;
    if request.error.trim().is_empty() || request.error.len() > MAX_ERROR_BYTES {
        return Err(ApiError::BadRequest);
    }

    let client = state.db.lock().await;
    let changed = client
        .execute(
            "UPDATE llm_web_relay_jobs
                SET status=CASE WHEN attempt >= max_attempts THEN 'FAILED' ELSE 'QUEUED' END,
                    lease_token=NULL,
                    locked_until=NULL,
                    error_message=$3,
                    updated_at=now()
              WHERE id=$1 AND status='RUNNING' AND lease_token=$2",
            &[&job_id, &request.lease_token, &request.error],
        )
        .await
        .map_err(|_| ApiError::Internal)?;

    if changed == 0 {
        return Err(ApiError::Conflict);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn healthz() -> &'static str {
    "ok"
}

#[derive(Debug, Serialize)]
struct Metrics {
    queued: i64,
    running: i64,
    succeeded: i64,
    failed: i64,
    expired: i64,
}

async fn metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Metrics>, ApiError> {
    authorize(&headers, &state.api_token)?;

    let client = state.db.lock().await;
    let row = client
        .query_one(
            "SELECT
              count(*) FILTER (WHERE status='QUEUED'),
              count(*) FILTER (WHERE status='RUNNING'),
              count(*) FILTER (WHERE status='SUCCEEDED'),
              count(*) FILTER (WHERE status='FAILED'),
              count(*) FILTER (WHERE status='EXPIRED')
             FROM llm_web_relay_jobs",
            &[],
        )
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(Metrics {
        queued: row.get(0),
        running: row.get(1),
        succeeded: row.get(2),
        failed: row.get(3),
        expired: row.get(4),
    }))
}

async fn connect(url: &str) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let (client, connection) = tokio_postgres::connect(url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("llm-web-relay postgres connection error: {error}");
        }
    });
    Ok(client)
}

fn required_secret(name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let direct = env::var(name).ok().filter(|value| !value.trim().is_empty());
    let file = env::var(format!("{name}_FILE"))
        .ok()
        .filter(|value| !value.trim().is_empty());

    if direct.is_some() && file.is_some() {
        return Err(format!("{name} and {name}_FILE cannot both be set").into());
    }

    let value = match (direct, file) {
        (Some(value), None) => value,
        (None, Some(path)) => std::fs::read_to_string(path)?,
        _ => return Err(format!("{name} or {name}_FILE is required").into()),
    };

    let value = value.trim().to_owned();
    if value.len() < 16 {
        return Err(format!("{name} must contain at least 16 characters").into());
    }
    Ok(value)
}

fn parse_duration_env(name: &str, default_seconds: u64, min: u64, max: u64) -> Duration {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(min, max))
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(default_seconds))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url =
        env::var("DATABASE_URL").or_else(|_| env::var("LLM_RELAY_DATABASE_URL"))?;
    let api_token = required_secret("LLM_RELAY_API_TOKEN")?;
    let worker_token = required_secret("LLM_RELAY_WORKER_TOKEN")?;

    if api_token == worker_token {
        return Err("API and worker relay tokens must be different".into());
    }

    let client = connect(&database_url).await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/001_economic_kernel.sql"))
        .await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/002_company_execution.sql"))
        .await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/003_agent_memory_and_rate_limits.sql"))
        .await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/004_affiliate_attribution.sql"))
        .await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/005_media_jobs.sql"))
        .await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/006_affiliate_reconciliation_state.sql"))
        .await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/007_organization_payroll.sql"))
        .await?;
    client
        .batch_execute(include_str!("../../../infra/db/migrations/008_affiliate_revenue_accounting.sql"))
        .await?;
    client
.batch_execute(include_str!("../../../infra/db/migrations/009_llm_web_relay.sql"))
        .await?;
    client
        .batch_execute(include_str!(
            "../../../infra/db/migrations/010_llm_relay_request_fingerprint.sql"
        ))
        .await?;

    drop(client);

    let state = AppState {
        db: Arc::new(Mutex::new(connect(&database_url).await?)),
        api_token,
        worker_token,
        wait_timeout: parse_duration_env("LLM_RELAY_WAIT_TIMEOUT_SECONDS", 55, 5, 90),
        job_ttl: parse_duration_env("LLM_RELAY_JOB_TTL_SECONDS", 120, 30, 900),
        lease: parse_duration_env("LLM_RELAY_LEASE_SECONDS", 60, 15, 300),
        max_attempts: env::var("LLM_RELAY_MAX_ATTEMPTS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .map(|v| v.clamp(1, 10))
            .unwrap_or(3),
    };

    let bind = env::var("LLM_RELAY_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:9010".into());

    let app = Router::new()
        .route("/v1/generate", post(generate))
        .route("/v1/jobs/claim", post(claim))
        .route("/v1/jobs/{job_id}/complete", post(complete))
        .route("/v1/jobs/{job_id}/fail", post(fail))
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics))
        .layer(axum::extract::DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    println!("LLM web relay listening on {bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_supported_web_backends_are_accepted() {
        assert!(validate_backend("gemini-web"));
        assert!(validate_backend("chatgpt-web"));
        assert!(validate_backend("claude-web"));
        assert!(!validate_backend("openai"));
    }

    #[test]
    fn request_fingerprint_changes_when_payload_changes() {
        let mut first = GenerateRequest {
            protocol_version: 1,
            backend: "gemini-web".into(),
            model: "web-session".into(),
            system: "system".into(),
            user: "hello".into(),
            response_format: "json_object".into(),
            allow_tools: false,
            idempotency_key: Some("stable-key".into()),
        };
        let a = request_hash(&first);
        first.user = "different".into();
        let b = request_hash(&first);
        assert_ne!(a, b);
    }

    #[test]
    fn tools_are_always_denied() {
        let request = GenerateRequest {
            protocol_version: 1,
            backend: "gemini-web".into(),
            model: "web-session".into(),
            system: "x".into(),
            user: "y".into(),
            response_format: "json_object".into(),
            allow_tools: true,
            idempotency_key: None,
        };
        assert!(matches!(validate_request(&request), Err(ApiError::BadRequest)));
    }

    #[test]
    fn output_must_be_a_json_object() {
        assert!(validate_output(r#"{"ok":true}"#).is_ok());
        assert!(validate_output(r#"["not-object"]"#).is_err());
        assert!(validate_output("not json").is_err());
    }

    #[test]
    fn request_limits_are_enforced() {
        let request = GenerateRequest {
            protocol_version: 1,
            backend: "claude-web".into(),
            model: "web-session".into(),
            system: "x".repeat(129 * 1024),
            user: "y".into(),
            response_format: "json_object".into(),
            allow_tools: false,
            idempotency_key: None,
        };
        assert!(matches!(
            validate_request(&request),
            Err(ApiError::PayloadTooLarge)
        ));
    }
}
