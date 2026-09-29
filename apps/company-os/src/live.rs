use super::AppState;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use tiktok_live_engine::{decide_response, EngagementPolicy, LiveEvent, LiveLedger, LiveMode, LiveSession, ProviderGiftStatement};
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
    if req.approved_for_external_publish {
        let approved = std::env::var("TIKTOK_LIVE_PUBLISH_APPROVED")
            .ok()
            .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"));
        let destination = std::env::var("TIKTOK_LIVE_STREAM_DESTINATION").ok();
        if !approved || !destination.as_deref().is_some_and(|value| value.starts_with("rtmp://") || value.starts_with("rtmps://")) {
            return Err(StatusCode::PRECONDITION_FAILED);
        }
    }
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
    let mut response = None;
    let mut overlay_updated = false;
    if accepted {
        if let Ok(mode) = state.store.tiktok_live_mode(&state.company_id, &session_id).await {
            if let Ok(summary) = state.store.tiktok_live_summary(&state.company_id, &session_id).await {
                let gift_count = summary.get("gift_count")
                    .and_then(|value| value.as_str())
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(0);
                let gift_value_minor = summary.get("gift_value_minor")
                    .and_then(|value| value.as_str())
                    .and_then(|value| value.parse::<u128>().ok())
                    .unwrap_or(0);
                let ledger = LiveLedger {
                    gift_count,
                    gift_value_minor,
                    ..LiveLedger::default()
                };
                let generated = decide_response(mode, &req.event, &ledger, &EngagementPolicy::default());
                if let Some(controller) = state.live_stream.as_ref() {
                    overlay_updated = controller.update_overlay(&generated.text).is_ok();
                }
                response = Some(generated);
            }
        }
    }
    Ok(Json(serde_json::json!({
        "accepted": accepted,
        "event_id": req.event.event_id,
        "response": response,
        "overlay_updated": overlay_updated,
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

#[derive(Debug, Deserialize)]
pub struct OverlayRequest {
    pub text: String,
}

pub async fn start_stream(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let controller = state.live_stream.as_ref().ok_or(StatusCode::PRECONDITION_FAILED)?;
    controller
        .start("Veridara AI LIVE — đang khởi động...")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(serde_json::json!({"running": true})))
}

pub async fn update_overlay(
    State(state): State<AppState>,
    Json(req): Json<OverlayRequest>,
) -> Result<StatusCode, StatusCode> {
    let controller = state.live_stream.as_ref().ok_or(StatusCode::PRECONDITION_FAILED)?;
    controller.update_overlay(&req.text).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stop_stream(
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let controller = state.live_stream.as_ref().ok_or(StatusCode::PRECONDITION_FAILED)?;
    controller.stop().map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stream_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let running = match state.live_stream.as_ref() {
        Some(controller) => controller.is_running().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        None => false,
    };
    Ok(Json(serde_json::json!({"enabled": state.live_stream.is_some(), "running": running})))
}
