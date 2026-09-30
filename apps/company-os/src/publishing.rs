#![forbid(unsafe_code)]

use async_trait::async_trait;
use axum::{body::Bytes, extract::State, http::{HeaderMap, StatusCode}, Json};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use social_publishing::{verify_tiktok_webhook_signature, ApprovalAuthority, CreatorInfo, TikTokAccessTokenProvider, TikTokPublisher, VideoPublishRequest};
use std::{path::{Path, PathBuf}, sync::Arc};
use tokio::{fs::File, io::AsyncReadExt};
use url::Url;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TikTokExecuteRequest {
    pub intent_id: String,
    pub approval_token: String,
    pub lease_seconds: i64,
    pub privacy_level: String,
    #[serde(default)]
    pub disable_duet: bool,
    #[serde(default)]
    pub disable_comment: bool,
    #[serde(default)]
    pub disable_stitch: bool,
    pub cover_timestamp_ms: Option<u64>,
    pub is_aigc: bool,
    #[serde(default)]
    pub brand_organic_toggle: bool,
    pub policy_snapshot_key: String,
    pub policy_evidence_ref: String,
    pub disclosure_present: bool,
    pub claim_evidence_present: bool,
    pub product_eligibility_verified: bool,
    pub rights_evidence_present: bool,
}

#[derive(Debug, Serialize)]
pub struct TikTokExecuteResponse {
    pub intent_id: String,
    pub execution_token: String,
    pub publish_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct TikTokStatusRequest {
    pub intent_id: String,
    pub execution_token: String,
    pub publish_id: String,
}

#[derive(Debug, Serialize)]
pub struct TikTokStatusResponse {
    pub intent_id: String,
    pub publish_id: String,
    pub provider_status: String,
    pub terminal: bool,
}

fn approval_authority() -> Result<ApprovalAuthority, String> {
    let secret = std::env::var("PUBLISH_APPROVAL_SECRET")
        .map_err(|_| "PUBLISH_APPROVAL_SECRET is required for TikTok publishing".to_owned())?;
    ApprovalAuthority::new(secret.into_bytes()).map_err(|e| e.to_string())
}

fn publisher_from_env() -> Result<TikTokPublisher, String> {
    let approval = approval_authority()?;
    TikTokPublisher::from_env(approval).map_err(|e| e.to_string())
}

#[derive(Clone)]
struct StoreTikTokAccessTokenProvider {
    store: Arc<company_store::CompanyStore>,
    oauth: company_tiktok_auth::TikTokOAuthClient,
    cipher: company_tiktok_auth::TokenCipher,
    company_id: String,
}

#[async_trait]
impl TikTokAccessTokenProvider for StoreTikTokAccessTokenProvider {
    async fn access_token(&self) -> Result<String, social_publishing::PublishError> {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let material = self
            .store
            .tiktok_oauth_token_material(&self.company_id, &self.cipher)
            .await
            .map_err(|error| social_publishing::PublishError::Provider(error.to_string()))?
            .ok_or(social_publishing::PublishError::Unauthorized)?;

        if material.refresh_token_expires_at_epoch <= now {
            let _ = self
                .store
                .mark_tiktok_reauth_required(&self.company_id, "TikTok refresh token has expired")
                .await;
            return Err(social_publishing::PublishError::Unauthorized);
        }

        if material.access_token_expires_at_epoch > now + 600 {
            return Ok(material.access_token);
        }

        match self.oauth.refresh(&material.refresh_token).await {
            Ok(token) => {
                let access_token = token.access_token.clone();
                self.store
                    .save_tiktok_token_set(&self.company_id, &token, &self.cipher)
                    .await
                    .map_err(|error| social_publishing::PublishError::Provider(error.to_string()))?;
                Ok(access_token)
            }
            Err(error) => {
                let message = error.to_string();
                let _ = self
                    .store
                    .mark_tiktok_reauth_required(&self.company_id, &message)
                    .await;
                Err(map_tiktok_auth_error(error))
            }
        }
    }
}

fn map_tiktok_auth_error(
    error: company_tiktok_auth::AuthError,
) -> social_publishing::PublishError {
    match error {
        company_tiktok_auth::AuthError::Unauthorized => social_publishing::PublishError::Unauthorized,
        company_tiktok_auth::AuthError::RateLimited => social_publishing::PublishError::RateLimited,
        other => social_publishing::PublishError::Provider(other.to_string()),
    }
}

async fn publisher(state: &crate::AppState) -> Result<TikTokPublisher, String> {
    let approval = approval_authority()?;
    let oauth_enabled = std::env::var("TIKTOK_OAUTH_ENABLED")
        .ok()
        .is_some_and(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"));
    if oauth_enabled {
        let config = company_tiktok_auth::OAuthConfig::from_env().map_err(|e| e.to_string())?;
        let cipher = company_tiktok_auth::TokenCipher::from_env().map_err(|e| e.to_string())?;
        let oauth = company_tiktok_auth::TikTokOAuthClient::new(config.clone()).map_err(|e| e.to_string())?;
        let provider = Arc::new(StoreTikTokAccessTokenProvider {
            store: state.store.clone(),
            oauth,
            cipher,
            company_id: state.company_id.clone(),
        });
        return TikTokPublisher::from_access_token_provider(
            provider,
            approval,
            std::env::var("TIKTOK_CONTENT_API_BASE")
                .unwrap_or_else(|_| "https://open.tiktokapis.com".into()),
        )
        .map_err(|e| e.to_string());
    }
    publisher_from_env()
}

fn file_path(media_uri: &str) -> Result<PathBuf, String> {
    let parsed = Url::parse(media_uri).map_err(|e| format!("media_uri is not a valid URL: {e}"))?;
    if parsed.scheme() != "file" {
        return Err("TikTok FILE_UPLOAD execution requires a file:// media_uri".into());
    }
    parsed
        .to_file_path()
        .map_err(|_| "media_uri must resolve to a local file path".into())
}

fn enforce_media_root(path: &Path) -> Result<PathBuf, String> {
    let root = std::env::var("PUBLISH_MEDIA_ROOT")
        .map_err(|_| "PUBLISH_MEDIA_ROOT is required for local media publishing".to_owned())?;
    let root = std::fs::canonicalize(root).map_err(|e| format!("invalid PUBLISH_MEDIA_ROOT: {e}"))?;
    let file = std::fs::canonicalize(path).map_err(|e| format!("media file is unavailable: {e}"))?;
    if !file.starts_with(&root) {
        return Err("media file is outside PUBLISH_MEDIA_ROOT".into());
    }
    Ok(file)
}

async fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).await.map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if read == 0 { break; }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn fail_message(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": error.to_string() })),
    )
}

pub async fn execute_tiktok(
    State(state): State<AppState>,
    Json(request): Json<TikTokExecuteRequest>,
) -> Result<Json<TikTokExecuteResponse>, (StatusCode, Json<serde_json::Value>)> {
    let compliance = state
        .store
        .check_tiktok_compliance_for_publish(
            &state.company_id,
            &request.intent_id,
            &request.policy_snapshot_key,
            &request.policy_evidence_ref,
            request.disclosure_present,
            request.claim_evidence_present,
            request.product_eligibility_verified,
            request.rights_evidence_present,
        )
        .await
        .map_err(fail_message)?;

    if compliance.decision != company_compliance::ComplianceDecision::Allowed {
        return Err((
            StatusCode::PRECONDITION_FAILED,
            Json(serde_json::json!({
                "error": "TikTok publish blocked by compliance policy",
                "decision": compliance.decision,
                "reason": compliance.reason,
                "requires_human": compliance.requires_human
            })),
        ));
    }

    let Some(job) = state
        .store
        .claim_publish_intent(
            &state.company_id,
            &request.intent_id,
            &request.approval_token,
            request.lease_seconds,
        )
        .await
        .map_err(fail_message)?
    else {
        return Err(fail_message("publish intent is not approved, not due, or already claimed"));
    };

    if job.intent.platform != publishing_contract::PublishPlatform::TikTok {
        let _ = state.store.complete_publish_intent(
            &state.company_id,
            &request.intent_id,
            &job.execution_token,
            false,
            None,
            Some("executor only supports TikTok"),
        ).await;
        return Err(fail_message("publish intent platform is not TikTok"));
    }

    let path = match file_path(&job.intent.media_uri).and_then(|p| enforce_media_root(&p)) {
        Ok(path) => path,
        Err(error) => {
            let _ = state.store.complete_publish_intent(
                &state.company_id, &request.intent_id, &job.execution_token, false, None, Some(&error)
            ).await;
            return Err(fail_message(error));
        }
    };

    let actual_hash = match sha256_file(&path).await {
        Ok(value) => value,
        Err(error) => {
            let _ = state.store.complete_publish_intent(
                &state.company_id, &request.intent_id, &job.execution_token, false, None, Some(&error)
            ).await;
            return Err(fail_message(error));
        }
    };
    if !actual_hash.eq_ignore_ascii_case(&job.intent.content_hash) {
        let error = "media content hash does not match the approved publish intent";
        let _ = state.store.complete_publish_intent(
            &state.company_id, &request.intent_id, &job.execution_token, false, None, Some(error)
        ).await;
        return Err(fail_message(error));
    }

    let publisher = publisher().map_err(fail_message)?;
    let creator = publisher.query_creator_info().await.map_err(|e| fail_message(e))?;

    let publish_request = VideoPublishRequest {
        artifact_id: job.intent.content_id.clone(),
        video_path: path,
        title: job.intent.title.clone(),
        privacy_level: request.privacy_level,
        disable_duet: request.disable_duet,
        disable_comment: request.disable_comment,
        disable_stitch: request.disable_stitch,
        cover_timestamp_ms: request.cover_timestamp_ms,
        is_aigc: request.is_aigc,
        brand_organic_toggle: request.brand_organic_toggle,
    };

    if let Err(error) = validate_creator_request(&publish_request, &creator) {
        let message = error;
        let _ = state.store.complete_publish_intent(
            &state.company_id, &request.intent_id, &job.execution_token, false, None, Some(&message)
        ).await;
        return Err(fail_message(message));
    }

    let now_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
    let budget = state
        .store
        .consume_autonomy_budget(
            &state.company_id,
            company_safety_controls::BudgetKind::ContentPublish,
            1,
            &format!(
                "content-publish:{}:{}:{}",
                state.company_id, request.intent_id, request.approval_token
            ),
            now_epoch,
        )
        .await
        .map_err(|error| fail_message(error))?;
    if !budget.allowed {
        let reason = "content publish blocked by emergency stop or daily autonomy budget";
        let _ = state.store.complete_publish_intent(
            &state.company_id,
            &request.intent_id,
            &job.execution_token,
            false,
            None,
            Some(reason),
        ).await;
        return Err((
            StatusCode::PRECONDITION_FAILED,
            Json(serde_json::json!({
                "error": reason,
                "budget": budget
            })),
        ));
    }

    let receipt = match publisher.publish_video_authorized(publish_request, &creator).await {
        Ok(receipt) => receipt,
        Err(error) => {
            let message = error.to_string();
            let _ = state.store.complete_publish_intent(
                &state.company_id, &request.intent_id, &job.execution_token, false, None, Some(&message)
            ).await;
            return Err(fail_message(message));
        }
    };

    state.store.record_publish_started(
        &state.company_id,
        &request.intent_id,
        &job.execution_token,
        &receipt.publish_id,
    ).await.map_err(fail_message)?;

    Ok(Json(TikTokExecuteResponse {
        intent_id: request.intent_id,
        execution_token: job.execution_token,
        publish_id: receipt.publish_id,
        status: receipt.status,
    }))
}

fn validate_creator_request(
    request: &VideoPublishRequest,
    creator: &CreatorInfo,
) -> Result<(), String> {
    if request.privacy_level.trim().is_empty() {
        return Err("privacy_level must be explicitly selected".into());
    }
    if !creator.privacy_level_options.iter().any(|v| v == &request.privacy_level) {
        return Err("privacy_level is not currently allowed for this creator".into());
    }
    Ok(())
}

pub async fn tiktok_status(
    State(state): State<AppState>,
    Json(request): Json<TikTokStatusRequest>,
) -> Result<Json<TikTokStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    if request.publish_id.trim().is_empty() {
        return Err(fail_message("publish_id is required"));
    }
    let publisher = publisher().map_err(fail_message)?;
    let provider_status = publisher.fetch_status(&request.publish_id).await.map_err(|e| fail_message(e))?;

    let terminal = matches!(provider_status.as_str(), "PUBLISH_COMPLETE" | "FAILED");
    if terminal {
        let success = provider_status == "PUBLISH_COMPLETE";
        let error = if success { None } else { Some(provider_status.as_str()) };
        state.store.complete_publish_intent(
            &state.company_id,
            &request.intent_id,
            &request.execution_token,
            success,
            Some(&request.publish_id),
            error,
        ).await.map_err(fail_message)?;
    }

    Ok(Json(TikTokStatusResponse {
        intent_id: request.intent_id,
        publish_id: request.publish_id,
        provider_status,
        terminal,
    }))
}

#[derive(Debug, Deserialize)]
struct TikTokWebhookEnvelope {
    event: String,
    create_time: i64,
    #[serde(default)]
    content: String,
}

pub async fn tiktok_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    if body.len() > 256 * 1024 { return Err(StatusCode::PAYLOAD_TOO_LARGE); }
    let signature = headers.get("TikTok-Signature").and_then(|v| v.to_str().ok()).ok_or(StatusCode::UNAUTHORIZED)?;
    let secret = std::env::var("TIKTOK_CLIENT_SECRET").map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    verify_tiktok_webhook_signature(secret.as_bytes(), signature, &body, 300).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let envelope: TikTokWebhookEnvelope = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    if envelope.event.trim().is_empty() || envelope.create_time <= 0 { return Err(StatusCode::BAD_REQUEST); }
    let content = if envelope.content.trim().is_empty() { serde_json::Value::Null } else { serde_json::from_str::<serde_json::Value>(&envelope.content).map_err(|_| StatusCode::BAD_REQUEST)? };
    let publish_id = content.get("publish_id").and_then(|v| v.as_str()).map(str::to_owned);
    let (success, failure_reason) = match envelope.event.as_str() {
        "post.publish.complete" | "post.publish.completed" | "video.publish.completed" => (Some(true), None),
        "post.publish.failed" | "video.upload.failed" => (Some(false), content.get("reason").and_then(|v| v.as_str()).map(str::to_owned)),
        "authorization.removed" => (None, None),
        _ => (None, None),
    };
    let payload_hash = format!("{:x}", Sha256::digest(&body));
    let event_key = format!("{}:{}", envelope.event, payload_hash);
    state.store.reconcile_tiktok_webhook(&state.company_id, &event_key, &envelope.event, publish_id.as_deref(), &payload_hash, success, failure_reason.as_deref())
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}