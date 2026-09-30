#![forbid(unsafe_code)]

use agent_runtime::types::AgentRole;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PublishPlatform {
    TikTok,
    YouTube,
    Instagram,
    Facebook,
}

impl PublishPlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TikTok => "TIKTOK",
            Self::YouTube => "YOUTUBE",
            Self::Instagram => "INSTAGRAM",
            Self::Facebook => "FACEBOOK",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "tiktok" => Some(Self::TikTok),
            "youtube" | "youtube-short" => Some(Self::YouTube),
            "instagram" | "instagram-reel" => Some(Self::Instagram),
            "facebook" | "facebook-reel" => Some(Self::Facebook),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PublishIntentStatus {
    Draft,
    Approved,
    Running,
    Succeeded,
    Failed,
    Revoked,
}

impl PublishIntentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Approved => "APPROVED",
            Self::Running => "RUNNING",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Revoked => "REVOKED",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "DRAFT" => Some(Self::Draft),
            "APPROVED" => Some(Self::Approved),
            "RUNNING" => Some(Self::Running),
            "SUCCEEDED" => Some(Self::Succeeded),
            "FAILED" => Some(Self::Failed),
            "REVOKED" => Some(Self::Revoked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishIntent {
    pub id: String,
    pub company_id: String,
    pub content_id: String,
    pub platform: PublishPlatform,
    pub media_uri: String,
    pub title: String,
    pub caption: String,
    pub scheduled_at: Option<String>,
    pub content_hash: String,
    pub created_by: AgentRole,
    pub idempotency_key: String,
}

impl PublishIntent {
    pub fn validate(&self) -> Result<(), String> {
        if Uuid::parse_str(&self.id).is_err() || Uuid::parse_str(&self.company_id).is_err() {
            return Err("publish intent identifiers must be UUIDs".into());
        }
        if self.content_id.trim().is_empty() || self.content_id.len() > 256 {
            return Err("content_id is invalid".into());
        }
        if self.media_uri.trim().is_empty() || self.media_uri.len() > 2048 {
            return Err("media_uri is invalid".into());
        }
        let scheme = self.media_uri.split(':').next().unwrap_or_default().to_ascii_lowercase();
        if !matches!(scheme.as_str(), "https" | "s3" | "file") {
            return Err("media_uri scheme must be https, s3 or file".into());
        }
        if self.title.len() > 500 || self.caption.len() > 10_000 {
            return Err("publish text is too long".into());
        }
        if self.content_hash.len() != 64 || !self.content_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("content_hash must be 64 hexadecimal characters".into());
        }
        if self.idempotency_key.trim().is_empty() || self.idempotency_key.len() > 256 {
            return Err("publish idempotency_key is invalid".into());
        }
        if let Some(scheduled_at) = self.scheduled_at.as_deref() {
            OffsetDateTime::parse(
                scheduled_at,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|error| format!("scheduled_at is invalid: {error}"))?;
        }
        if self.created_by == AgentRole::Governor {
            return Err("Governor cannot create operational publishing intents".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishApproval {
    pub intent_id: String,
    pub approval_token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishJob {
    pub intent: PublishIntent,
    pub execution_token: String,
}

pub fn new_idempotency_key(intent: &PublishIntent) -> String {
    format!("publish:{}", intent.idempotency_key)
}

pub fn generate_approval_token() -> (String, String) {
    let token = format!("pub-{}", Uuid::new_v4());
    (token.clone(), hash_secret(&token))
}

pub fn hash_secret(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}


#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::types::AgentRole;

    fn intent() -> PublishIntent {
        PublishIntent {
            id: Uuid::new_v4().to_string(),
            company_id: Uuid::new_v4().to_string(),
            content_id: "content-1".into(),
            platform: PublishPlatform::TikTok,
            media_uri: "s3://bucket/video.mp4".into(),
            title: "test".into(),
            caption: "caption".into(),
            scheduled_at: None,
            content_hash: "a".repeat(64),
            created_by: AgentRole::Content,
            idempotency_key: "content-1:tiktok:v1".into(),
        }
    }

    #[test]
    fn publish_intent_validation_is_strict() {
        assert!(intent().validate().is_ok());

        let mut bad = intent();
        bad.media_uri = "javascript:alert(1)".into();
        assert!(bad.validate().is_err());

        let mut bad_hash = intent();
        bad_hash.content_hash = "not-a-hash".into();
        assert!(bad_hash.validate().is_err());

        let mut governor = intent();
        governor.created_by = AgentRole::Governor;
        assert!(governor.validate().is_err());
    }

    #[test]
    fn approval_tokens_are_random_and_hashed() {
        let (a, ah) = generate_approval_token();
        let (b, bh) = generate_approval_token();
        assert_ne!(a, b);
        assert_ne!(ah, bh);
        assert_eq!(hash_secret(&a), ah);
        assert_eq!(hash_secret(&b), bh);
    }
}
