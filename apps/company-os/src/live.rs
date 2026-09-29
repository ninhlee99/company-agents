use super::AppState;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use tiktok_live_engine::{LiveEvent, LiveMode, LiveSession, ProviderGiftStatement};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: String,
    pub mode: String,
    pub room_id: Option<String>,
    pub started_at_epoch: i64,
    pub approved_for_external_publish: bool,
}

#[derive(Debug, Deserialize)]
pub struct EventRequest {
    pub event: LiveEvent,
}

#[derive(Debug, Deserialize)]
pub struct GiftStatementRequest {
    pub statement: ProviderGiftStatement,
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<LiveSession>, StatusCode> {
    let mode = parse_mode(&req.mode).ok_or(StatusCode::BAD_REQUEST)?;
    let company_id = Uuid::parse_str(&state.company_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let session = LiveSession {
        id: Uuid::new_v4(),
        company_id,
        room_id: req.room_id,
        title: req.title,
        mode,
        started_at_epoch: req.started_at_epoch,
        approved_for_external_publish: req.approved_for_external_publish,
    };
    state.store.create_tiktok_live_session(&session).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(session))
}

pub async fn record_event(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<EventRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let accepted = state.store.record_tiktok_live_event(&state.company_id, &session_id, &req.event)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(serde_json::json!({
        "accepted": accepted,
        "event_id": req.event.event_id,
    })))
}

pub async fn summary(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.store.tiktok_live_summary(&state.company_id, &session_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn reconcile_gifts(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<GiftStatementRequest>,
) -> Result<Json<tiktok_live_engine::GiftReconciliation>, StatusCode> {
    state.store.reconcile_tiktok_live_gifts(&state.company_id, &session_id, &req.statement)
        .await
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

fn parse_mode(value: &str) -> Option<LiveMode> {
    match value.trim().to_ascii_uppercase().as_str() {
        "SOLO" => Some(LiveMode::Solo),
        "COHOST" | "CO_HOST" => Some(LiveMode::CoHost),
        "PK" => Some(LiveMode::Pk),
        "GAME" => Some(LiveMode::Game),
        "STORY" => Some(LiveMode::Story),
        "MUSIC" => Some(LiveMode::Music),
        "SHOPPING" | "SHOP" => Some(LiveMode::Shopping),
        _ => None,
    }
}
