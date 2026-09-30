#![forbid(unsafe_code)]

use hmac::{Hmac, Mac};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex, time::sleep};

type HmacSha256 = Hmac<Sha256>;

#[async_trait::async_trait]
pub trait TikTokAccessTokenProvider: Send + Sync {
    async fn access_token(&self) -> Result<String, PublishError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalReceipt {
    pub scope: String,
    pub artifact_id: String,
    pub expires_at_epoch: i64,
    signature_hex: String,
}

#[derive(Clone)]
pub struct ApprovalAuthority {
    secret: Arc<Vec<u8>>,
}

impl ApprovalAuthority {
    pub fn new(secret: impl Into<Vec<u8>>) -> Result<Self, PublishError> {
        let secret = secret.into();
        if secret.len() < 32 {
            return Err(PublishError::InvalidApproval(
                "approval secret must be at least 32 bytes".into(),
            ));
        }
        Ok(Self {
            secret: Arc::new(secret),
        })
    }

    pub fn issue(
        &self,
        scope: &str,
        artifact_id: &str,
        expires_at_epoch: i64,
    ) -> Result<ApprovalReceipt, PublishError> {
        if scope.trim().is_empty() || artifact_id.trim().is_empty() {
            return Err(PublishError::InvalidApproval(
                "scope and artifact id are required".into(),
            ));
        }
        if expires_at_epoch <= epoch_now() {
            return Err(PublishError::InvalidApproval(
                "approval is already expired".into(),
            ));
        }
        let payload = format!("{scope}|{artifact_id}|{expires_at_epoch}");
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|_| PublishError::InvalidApproval("invalid approval secret".into()))?;
        mac.update(payload.as_bytes());
        Ok(ApprovalReceipt {
            scope: scope.into(),
            artifact_id: artifact_id.into(),
            expires_at_epoch,
            signature_hex: hex(&mac.finalize().into_bytes()),
        })
    }

    pub fn verify(
        &self,
        receipt: &ApprovalReceipt,
        scope: &str,
        artifact_id: &str,
    ) -> Result<(), PublishError> {
        if receipt.scope != scope || receipt.artifact_id != artifact_id {
            return Err(PublishError::InvalidApproval(
                "approval scope/artifact mismatch".into(),
            ));
        }
        if receipt.expires_at_epoch <= epoch_now() {
            return Err(PublishError::InvalidApproval("approval is expired".into()));
        }
        let payload = format!(
            "{}|{}|{}",
            receipt.scope, receipt.artifact_id, receipt.expires_at_epoch
        );
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|_| PublishError::InvalidApproval("invalid approval secret".into()))?;
        mac.update(payload.as_bytes());
        let signature = decode_hex(&receipt.signature_hex)?;
        mac.verify_slice(&signature)
            .map_err(|_| PublishError::InvalidApproval("approval signature is invalid".into()))
    }
}

pub fn verify_tiktok_webhook_signature(
    client_secret: &[u8],
    signature_header: &str,
    body: &[u8],
    max_age_seconds: i64,
) -> Result<(), PublishError> {
    if client_secret.len() < 32 {
        return Err(PublishError::Configuration("TikTok client secret must be at least 32 bytes".into()));
    }
    let mut timestamp = None;
    let mut signature = None;
    for part in signature_header.split(',') {
        let mut pieces = part.trim().splitn(2, '=');
        match (pieces.next(), pieces.next()) {
            (Some("t"), Some(value)) => timestamp = Some(value),
            (Some("s"), Some(value)) => signature = Some(value),
            _ => {}
        }
    }
    let timestamp = timestamp
        .ok_or_else(|| PublishError::InvalidRequest("TikTok-Signature timestamp is missing".into()))?
        .parse::<i64>()
        .map_err(|_| PublishError::InvalidRequest("TikTok-Signature timestamp is invalid".into()))?;
    let signature = signature
        .ok_or_else(|| PublishError::InvalidRequest("TikTok-Signature signature is missing".into()))?;
    let now = epoch_now();
    if (now - timestamp).abs() > max_age_seconds {
        return Err(PublishError::InvalidRequest("TikTok webhook timestamp is outside the replay window".into()));
    }

    let signed_payload = format!("{timestamp}.{}", String::from_utf8_lossy(body));
    let mut mac = HmacSha256::new_from_slice(client_secret)
        .map_err(|_| PublishError::Configuration("invalid TikTok client secret".into()))?;
    mac.update(signed_payload.as_bytes());
    let expected = hex(&mac.finalize().into_bytes());
    if expected.as_bytes().ct_eq(signature.as_bytes()).into() {
        Ok(())
    } else {
        Err(PublishError::Unauthorized)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatorInfo {
    pub username: String,
    pub privacy_level_options: Vec<String>,
    pub max_video_post_duration_sec: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoPublishRequest {
    pub artifact_id: String,
    pub video_path: PathBuf,
    pub title: String,
    pub privacy_level: String,
    pub disable_duet: bool,
    pub disable_comment: bool,
    pub disable_stitch: bool,
    pub cover_timestamp_ms: Option<u64>,
    pub is_aigc: bool,
    pub brand_organic_toggle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishReceipt {
    pub artifact_id: String,
    pub publish_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishError {
    Configuration(String),
    InvalidRequest(String),
    InvalidApproval(String),
    Unauthorized,
    RateLimited,
    Provider(String),
    Transport(String),
    File(String),
    Timeout,
}

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(v) => write!(f, "configuration error: {v}"),
            Self::InvalidRequest(v) => write!(f, "invalid publish request: {v}"),
            Self::InvalidApproval(v) => write!(f, "invalid approval: {v}"),
            Self::Unauthorized => write!(f, "TikTok authorization failed"),
            Self::RateLimited => write!(f, "TikTok API rate limited"),
            Self::Provider(v) => write!(f, "TikTok provider error: {v}"),
            Self::Transport(v) => write!(f, "TikTok transport error: {v}"),
            Self::File(v) => write!(f, "media file error: {v}"),
            Self::Timeout => write!(f, "TikTok request timed out"),
        }
    }
}

#[derive(Clone)]
pub struct TikTokPublisher {
    pub access_token: String,
    pub api_base: String,
    pub client: reqwest::Client,
    pub approval: ApprovalAuthority,
    limiter: Arc<Mutex<Option<std::time::Instant>>>,
}

impl TikTokPublisher {
    pub fn from_env(approval: ApprovalAuthority) -> Result<Self, PublishError> {
        let access_token = std::env::var("TIKTOK_CONTENT_ACCESS_TOKEN").map_err(|_| {
            PublishError::Configuration("TIKTOK_CONTENT_ACCESS_TOKEN is required".into())
        })?;
        if access_token.trim().is_empty() {
            return Err(PublishError::Configuration(
                "TIKTOK_CONTENT_ACCESS_TOKEN cannot be empty".into(),
            ));
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| PublishError::Configuration(format!("HTTP client setup failed: {e}")))?;
        Ok(Self {
            access_token,
            api_base: std::env::var("TIKTOK_CONTENT_API_BASE")
                .unwrap_or_else(|_| "https://open.tiktokapis.com".into()),
            client,
            approval,
            limiter: Arc::new(Mutex::new(None)),
        })
    }

    async fn wait_for_rate_limit(&self) {
        let mut guard = self.limiter.lock().await;
        if let Some(previous) = *guard {
            let elapsed = previous.elapsed();
            if elapsed < Duration::from_secs(10) {
                sleep(Duration::from_secs(10) - elapsed).await;
            }
        }
        *guard = Some(std::time::Instant::now());
    }

    async fn post_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, PublishError> {
        self.wait_for_rate_limit().await;
        let response = self
            .client
            .post(format!("{}{}", self.api_base.trim_end_matches('/'), path))
            .bearer_auth(&self.access_token)
            .header(header::CONTENT_TYPE, "application/json; charset=UTF-8")
            .json(&body)
            .send()
            .await
            .map_err(|e| PublishError::Transport(e.to_string()))?;

        if response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
        {
            return Err(PublishError::Unauthorized);
        }
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(PublishError::RateLimited);
        }
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| PublishError::Transport(e.to_string()))?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err(PublishError::Provider("response exceeds 2 MiB".into()));
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| PublishError::Provider(format!("invalid JSON response: {e}")))?;
        if !status.is_success() {
            return Err(PublishError::Provider(format!("http {status}")));
        }
        if value
            .pointer("/error/code")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v != "ok")
        {
            return Err(PublishError::Provider(
                value
                    .pointer("/error/message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("TikTok error")
                    .to_string(),
            ));
        }
        Ok(value)
    }

    pub async fn query_creator_info(&self) -> Result<CreatorInfo, PublishError> {
        let response = self
            .post_json("/v2/post/publish/creator_info/query/", json!({}))
            .await?;
        let data = response
            .get("data")
            .ok_or_else(|| PublishError::Provider("creator info data missing".into()))?;
        Ok(CreatorInfo {
            username: data
                .get("creator_username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .into(),
            privacy_level_options: data
                .get("privacy_level_options")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            max_video_post_duration_sec: data
                .get("max_video_post_duration_sec")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        })
    }

    pub fn validate_request(
        &self,
        request: &VideoPublishRequest,
        creator: &CreatorInfo,
    ) -> Result<(), PublishError> {
        if request.artifact_id.trim().is_empty() {
            return Err(PublishError::InvalidRequest(
                "artifact id is required".into(),
            ));
        }
        let utf16_len = request.title.encode_utf16().count();
        if utf16_len > 2_200 {
            return Err(PublishError::InvalidRequest(
                "title exceeds 2200 UTF-16 code units".into(),
            ));
        }
        if !creator
            .privacy_level_options
            .iter()
            .any(|v| v == &request.privacy_level)
        {
            return Err(PublishError::InvalidRequest(
                "privacy level is not allowed for this creator".into(),
            ));
        }
        validate_local_video_path(&request.video_path)?;
        if !request
            .video_path
            .extension()
            .and_then(|v| v.to_str())
            .map(|v| v.eq_ignore_ascii_case("mp4"))
            .unwrap_or(false)
        {
            return Err(PublishError::InvalidRequest(
                "direct video publish expects an MP4 artifact".into(),
            ));
        }
        Ok(())
    }

    pub async fn publish_video(
        &self,
        request: VideoPublishRequest,
        creator: &CreatorInfo,
        approval: &ApprovalReceipt,
    ) -> Result<PublishReceipt, PublishError> {
        self.validate_request(&request, creator)?;
        self.approval
            .verify(approval, "tiktok.video.publish", &request.artifact_id)?;
        self.publish_video_authorized(request, creator).await
    }

    /// Execute a publish after the durable CompanyStore approval gate has
    /// already been consumed. The caller must have obtained a valid
    /// execution lease from CompanyStore before invoking this method.
    pub async fn publish_video_authorized(
        &self,
        request: VideoPublishRequest,
        creator: &CreatorInfo,
    ) -> Result<PublishReceipt, PublishError> {
        self.validate_request(&request, creator)?;
        let metadata = tokio::fs::metadata(&request.video_path)
            .await
            .map_err(|e| PublishError::File(e.to_string()))?;
        if metadata.len() == 0 || metadata.len() > 500 * 1024 * 1024 {
            return Err(PublishError::InvalidRequest(
                "video size must be 1 byte..=500 MiB".into(),
            ));
        }

        let chunk_size: usize = 10_000_000;
        let total_chunks = (metadata.len() as usize).div_ceil(chunk_size);

        let init = self
            .post_json(
                "/v2/post/publish/video/init/",
                json!({
                    "post_info": {
                        "title": request.title,
                        "privacy_level": request.privacy_level,
                        "disable_duet": request.disable_duet,
                        "disable_comment": request.disable_comment,
                        "disable_stitch": request.disable_stitch,
                        "video_cover_timestamp_ms": request.cover_timestamp_ms.unwrap_or(0),
                        "is_aigc": request.is_aigc,
                        "brand_organic_toggle": request.brand_organic_toggle
                    },
                    "source_info": {
                        "source": "FILE_UPLOAD",
                        "video_size": metadata.len(),
                        "chunk_size": chunk_size,
                        "total_chunk_count": total_chunks
                    }
                }),
            )
            .await?;

        let data = init
            .get("data")
            .ok_or_else(|| PublishError::Provider("publish init data missing".into()))?;
        let publish_id = data
            .get("publish_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PublishError::Provider("publish_id missing".into()))?
            .to_string();
        let upload_url = data
            .get("upload_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PublishError::Provider("upload_url missing".into()))?
            .to_string();

        upload_chunks(
            &self.client,
            &upload_url,
            &request.video_path,
            metadata.len(),
            chunk_size,
        )
        .await?;

        Ok(PublishReceipt {
            artifact_id: request.artifact_id,
            publish_id,
            status: "PROCESSING".into(),
        })
    }

    pub async fn fetch_status(&self, publish_id: &str) -> Result<String, PublishError> {
        if publish_id.trim().is_empty() {
            return Err(PublishError::InvalidRequest(
                "publish id is required".into(),
            ));
        }
        let response = self
            .post_json(
                "/v2/post/publish/status/fetch/",
                json!({ "publish_id": publish_id }),
            )
            .await?;
        Ok(response
            .pointer("/data/status")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN")
            .to_string())
    }
}

async fn upload_chunks(
    client: &reqwest::Client,
    upload_url: &str,
    path: &Path,
    total_size: u64,
    chunk_size: usize,
) -> Result<(), PublishError> {
    let parsed = url::Url::parse(upload_url)
        .map_err(|e| PublishError::Provider(format!("invalid upload URL: {e}")))?;
    if !matches!(parsed.scheme(), "https") {
        return Err(PublishError::Provider(
            "TikTok upload URL must use HTTPS".into(),
        ));
    }

    let mut file = File::open(path)
        .await
        .map_err(|e| PublishError::File(e.to_string()))?;
    let mut start = 0_u64;
    let mut buffer = vec![0_u8; chunk_size];

    while start < total_size {
        let remaining = (total_size - start) as usize;
        let target = remaining.min(chunk_size);
        file.read_exact(&mut buffer[..target])
            .await
            .map_err(|e| PublishError::File(e.to_string()))?;
        let end = start + target as u64 - 1;

        let response = client
            .put(upload_url)
            .header(header::CONTENT_TYPE, "video/mp4")
            .header(header::CONTENT_LENGTH, target)
            .header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{total_size}"),
            )
            .body(buffer[..target].to_vec())
            .send()
            .await
            .map_err(|e| PublishError::Transport(e.to_string()))?;

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(PublishError::RateLimited);
        }
        if !response.status().is_success() {
            return Err(PublishError::Provider(format!(
                "upload http {}",
                response.status()
            )));
        }
        start = end + 1;
    }

    Ok(())
}

fn validate_local_video_path(path: &Path) -> Result<(), PublishError> {
    if path.as_os_str().is_empty() {
        return Err(PublishError::InvalidRequest(
            "video path is required".into(),
        ));
    }
    let value = path.to_string_lossy();
    if value.contains(' ') || value.contains("://") {
        return Err(PublishError::InvalidRequest(
            "only local media artifacts are allowed".into(),
        ));
    }
    Ok(())
}

fn epoch_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|b| {
            [
                HEX[(b >> 4) as usize] as char,
                HEX[(b & 15) as usize] as char,
            ]
        })
        .collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, PublishError> {
    if value.len() % 2 != 0 {
        return Err(PublishError::InvalidApproval(
            "invalid signature encoding".into(),
        ));
    }
    (0..value.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&value[i..i + 2], 16)
                .map_err(|_| PublishError::InvalidApproval("invalid signature encoding".into()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn approval_is_scope_bound_and_expiring() {
        let authority = ApprovalAuthority::new(vec![7_u8; 32]).unwrap();
        let receipt = authority
            .issue("tiktok.video.publish", "asset-1", epoch_now() + 300)
            .unwrap();
        assert!(authority
            .verify(&receipt, "tiktok.video.publish", "asset-1")
            .is_ok());
        assert!(authority
            .verify(&receipt, "tiktok.video.publish", "asset-2")
            .is_err());
        assert!(authority
            .verify(&receipt, "tiktok.message.send", "asset-1")
            .is_err());
    }

    #[test]
    fn title_utf16_limit_is_enforced() {
        let approval = ApprovalAuthority::new(vec![1_u8; 32]).unwrap();
        let publisher = TikTokPublisher {
            access_token: "test".into(),
            api_base: "https://example.invalid".into(),
            client: reqwest::Client::new(),
            approval,
            limiter: Arc::new(Mutex::new(None)),
        };
        let creator = CreatorInfo {
            username: "c".into(),
            privacy_level_options: vec!["PUBLIC_TO_EVERYONE".into()],
            max_video_post_duration_sec: 300,
        };
        let too_long = "x".repeat(2_201);
        let request = VideoPublishRequest {
            artifact_id: "a".into(),
            video_path: PathBuf::from("/tmp/x.mp4"),
            title: too_long,
            privacy_level: "PUBLIC_TO_EVERYONE".into(),
            disable_duet: false,
            disable_comment: false,
            disable_stitch: false,
            cover_timestamp_ms: None,
            is_aigc: true,
            brand_organic_toggle: false,
        };
        assert!(publisher.validate_request(&request, &creator).is_err());
    }

    #[test]
    fn remote_or_wrong_extension_artifacts_are_rejected() {
        let approval = ApprovalAuthority::new(vec![1_u8; 32]).unwrap();
        let publisher = TikTokPublisher {
            access_token: "test".into(),
            api_base: "https://example.invalid".into(),
            client: reqwest::Client::new(),
            approval,
            limiter: Arc::new(Mutex::new(None)),
        };
        let creator = CreatorInfo {
            username: "c".into(),
            privacy_level_options: vec!["PUBLIC_TO_EVERYONE".into()],
            max_video_post_duration_sec: 300,
        };
        let request = VideoPublishRequest {
            artifact_id: "a".into(),
            video_path: PathBuf::from("https://example.com/a.mp4"),
            title: "ok".into(),
            privacy_level: "PUBLIC_TO_EVERYONE".into(),
            disable_duet: false,
            disable_comment: false,
            disable_stitch: false,
            cover_timestamp_ms: None,
            is_aigc: true,
            brand_organic_toggle: false,
        };
        assert!(publisher.validate_request(&request, &creator).is_err());

        let request = VideoPublishRequest {
            video_path: PathBuf::from("/tmp/a.mov"),
            ..request
        };
        assert!(publisher.validate_request(&request, &creator).is_err());
    }
}

#[cfg(test)]
mod webhook_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn webhook_signature_accepts_fresh_payload() {
        let secret = vec![7_u8; 32];
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let body = br#"{"event":"post.publish.complete"}"#;
        let mut mac = HmacSha256::new_from_slice(&secret).unwrap();
        mac.update(format!("{timestamp}.{}", String::from_utf8_lossy(body)).as_bytes());
        let header = format!("t={timestamp},s={}", hex(&mac.finalize().into_bytes()));
        assert!(verify_tiktok_webhook_signature(&secret, &header, body, 300).is_ok());
    }

    #[test]
    fn webhook_signature_rejects_stale_payload() {
        let secret = vec![7_u8; 32];
        let timestamp = epoch_now() - 301;
        let body = br#"{}"#;
        let mut mac = HmacSha256::new_from_slice(&secret).unwrap();
        mac.update(format!("{timestamp}.{}", String::from_utf8_lossy(body)).as_bytes());
        let header = format!("t={timestamp},s={}", hex(&mac.finalize().into_bytes()));
        assert!(verify_tiktok_webhook_signature(&secret, &header, body, 300).is_err());
    }
}