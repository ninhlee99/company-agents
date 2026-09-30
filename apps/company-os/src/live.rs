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
    #[serde(default)]
    pub policy_snapshot_key: Option<String>,
    #[serde(default)]
    pub policy_evidence_ref: Option<String>,
    #[serde(default)]
    pub disclosure_present: bool,
    #[serde(default)]
    pub claim_evidence_present: bool,
    #[serde(default)]
    pub product_eligibility_verified: bool,
    #[serde(default)]
    pub rights_evidence_present: bool,
    #[serde(default)]
    pub simulcast: bool,
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

        let policy_snapshot_key = req.policy_snapshot_key.as_deref().ok_or(StatusCode::PRECONDITION_FAILED)?;
        let policy_evidence_ref = req.policy_evidence_ref.as_deref().ok_or(StatusCode::PRECONDITION_FAILED)?;
        let compliance = state
            .store
            .check_tiktok_compliance_for_live(
                &state.company_id,
                &req.title,
                policy_snapshot_key,
                policy_evidence_ref,
                req.disclosure_present,
                req.claim_evidence_present,
                req.product_eligibility_verified,
                req.rights_evidence_present,
                req.simulcast,
            )
            .await
            .map_err(|_| StatusCode::PRECONDITION_FAILED)?;

        if compliance.decision != company_compliance::ComplianceDecision::Allowed {
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
    let attention = if accepted {
        Some(
            state
                .store
                .record_tiktok_live_attention(&state.company_id, &session_id, &req.event)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        )
    } else {
        None
    };

    let mut response = None;
    let mut overlay_updated = false;
    if let Some(attention) = attention.as_ref() {
        if attention.action == company_live_attention::AttentionAction::Respond {
            let mode = state
                .store
                .tiktok_live_mode(&state.company_id, &session_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let summary = state
                .store
                .tiktok_live_summary(&state.company_id, &session_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let gift_count = summary_u64(&summary, "gift_count")?;
            let gift_value_minor = summary_u128(&summary, "gift_value_minor")?;
            let ledger = LiveLedger {
                gift_count,
                gift_value_minor,
                ..LiveLedger::default()
            };
            let mut generated =
                decide_response(mode, &req.event, &ledger, &EngagementPolicy::default());
            if attention.reason == company_live_attention::AttentionReason::HighValueViewer
                && matches!(generated.action, tiktok_live_engine::ResponseAction::IdlePrompt)
            {
                generated.action = tiktok_live_engine::ResponseAction::WelcomeViewer;
                generated.text = format!(
                    "Chào mừng {}! Mình sẽ ưu tiên hỗ trợ bạn đúng lúc trong LIVE.",
                    req.event.display_name.as_deref().unwrap_or("bạn")
                );
                generated.priority = attention.priority;
            }
            if let Some(controller) = state.live_stream.as_ref() {
                overlay_updated = controller.update_overlay(&generated.text).is_ok();
            }
            response = Some(generated);
        } else if attention.action == company_live_attention::AttentionAction::Escalate {
            let generated = tiktok_live_engine::LiveResponse {
                action: tiktok_live_engine::ResponseAction::SafetyEscalation,
                text: "Có tín hiệu cần moderator kiểm tra trước khi tiếp tục.".into(),
                priority: attention.priority,
                requires_human: true,
            };
            if let Some(controller) = state.live_stream.as_ref() {
                overlay_updated = controller.update_overlay(&generated.text).is_ok();
            }
            response = Some(generated);
        }
    }

    Ok(Json(serde_json::json!({
        "accepted": accepted,
        "event_id": req.event.event_id,
        "attention": attention,
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

fn summary_u64(summary: &serde_json::Value, key: &str) -> Result<u64, StatusCode> {
    match summary.get(key) {
        None => Ok(0),
        Some(value) => value
            .as_str()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .parse::<u64>()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn summary_u128(summary: &serde_json::Value, key: &str) -> Result<u128, StatusCode> {
    match summary.get(key) {
        None => Ok(0),
        Some(value) => value
            .as_str()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .parse::<u128>()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
    }
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
    let controller = state
        .live_stream
        .as_ref()
        .ok_or(StatusCode::PRECONDITION_FAILED)?;

    let policy_snapshot_key = std::env::var("TIKTOK_LIVE_POLICY_SNAPSHOT_KEY")
        .map_err(|_| StatusCode::PRECONDITION_FAILED)?;
    let policy_evidence_ref = std::env::var("TIKTOK_LIVE_POLICY_EVIDENCE_REF")
        .map_err(|_| StatusCode::PRECONDITION_FAILED)?;
    let compliance = state
        .store
        .check_tiktok_compliance_for_live(
            &state.company_id,
            "Veridara AI LIVE",
            &policy_snapshot_key,
            &policy_evidence_ref,
            env_bool("TIKTOK_LIVE_DISCLOSURE_PRESENT")?,
            env_bool("TIKTOK_LIVE_CLAIM_EVIDENCE_PRESENT")?,
            env_bool("TIKTOK_LIVE_PRODUCT_ELIGIBILITY_VERIFIED")?,
            env_bool("TIKTOK_LIVE_RIGHTS_EVIDENCE_PRESENT")?,
            env_bool("TIKTOK_LIVE_SIMULCAST")?,
        )
        .await
        .map_err(|_| StatusCode::PRECONDITION_FAILED)?;

    if compliance.decision != company_compliance::ComplianceDecision::Allowed {
        return Err(StatusCode::PRECONDITION_FAILED);
    }

    let controls = state
        .store
        .autonomy_controls(&state.company_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if controls.controls.emergency_stop.enabled {
        return Err(StatusCode::PRECONDITION_FAILED);
    }

    let reservation_minutes = std::env::var("AUTONOMY_LIVE_SESSION_RESERVATION_MINUTES")
        .ok()
        .and_then(|value| value.parse::<i128>().ok())
        .filter(|value| (1..=1_440).contains(value))
        .unwrap_or(60);
    let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
    let budget = state
        .store
        .consume_autonomy_budget(
            &state.company_id,
            company_safety_controls::BudgetKind::LiveMinutes,
            reservation_minutes,
            &format!("live-session:{}:{}", state.company_id, now_epoch),
            now_epoch,
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !budget.allowed {
        return Err(StatusCode::PRECONDITION_FAILED);
    }

    controller
        .start("Veridara AI LIVE — đang khởi động...")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(serde_json::json!({
        "running": true,
        "reserved_minutes": reservation_minutes,
        "budget": budget
    })))
}

fn env_bool(name: &str) -> Result<bool, StatusCode> {
    let value = std::env::var(name).map_err(|_| StatusCode::PRECONDITION_FAILED)?;
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(StatusCode::PRECONDITION_FAILED),
    }
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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn live_summary_gift_metrics_allow_missing_as_empty_state() {
        let summary = json!({});
        assert_eq!(summary_u64(&summary, "gift_count").unwrap(), 0);
        assert_eq!(summary_u128(&summary, "gift_value_minor").unwrap(), 0);
    }

    #[test]
    fn live_summary_gift_metrics_fail_closed_when_malformed() {
        let summary = json!({"gift_count":"oops","gift_value_minor":"100"});
        assert_eq!(summary_u64(&summary, "gift_count").unwrap_err(), StatusCode::INTERNAL_SERVER_ERROR);

        let summary = json!({"gift_count":"1","gift_value_minor":{}});
        assert_eq!(summary_u128(&summary, "gift_value_minor").unwrap_err(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn live_summary_gift_metrics_parse_valid_strings() {
        let summary = json!({"gift_count":"7","gift_value_minor":"123456"});
        assert_eq!(summary_u64(&summary, "gift_count").unwrap(), 7);
        assert_eq!(summary_u128(&summary, "gift_value_minor").unwrap(), 123456);
    }
}
