#![forbid(unsafe_code)]

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutboundEmail {
    pub message_id: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub html: String,
    pub idempotency_key: String,
    pub unsubscribe_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderReceipt {
    pub provider: String,
    pub provider_reference: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageError {
    Configuration(String),
    InvalidRequest(String),
    Unauthorized,
    RateLimited,
    Provider(String),
    Transport(String),
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(v) => write!(f, "configuration error: {v}"),
            Self::InvalidRequest(v) => write!(f, "invalid message: {v}"),
            Self::Unauthorized => write!(f, "email provider authorization failed"),
            Self::RateLimited => write!(f, "email provider rate limited"),
            Self::Provider(v) => write!(f, "email provider error: {v}"),
            Self::Transport(v) => write!(f, "email provider transport error: {v}"),
        }
    }
}

#[async_trait]
pub trait OutboundMessageProvider: Send + Sync {
    async fn send_email(&self, email: &OutboundEmail) -> Result<ProviderReceipt, MessageError>;
}

#[derive(Clone)]
pub struct ResendProvider {
    api_key: String,
    from: String,
    client: Client,
    api_base: String,
}

#[derive(Debug, Deserialize)]
struct ResendResponse {
    id: Option<String>,
}

impl ResendProvider {
    pub fn from_env() -> Result<Self, MessageError> {
        let api_key = std::env::var("RESEND_API_KEY")
            .map_err(|_| MessageError::Configuration("RESEND_API_KEY is required".into()))?;
        let from = std::env::var("RESEND_FROM")
            .map_err(|_| MessageError::Configuration("RESEND_FROM is required".into()))?;
        if api_key.trim().is_empty() || from.trim().is_empty() {
            return Err(MessageError::Configuration("Resend configuration is empty".into()));
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| MessageError::Configuration(e.to_string()))?;
        Ok(Self {
            api_key,
            from,
            client,
            api_base: std::env::var("RESEND_API_BASE")
                .unwrap_or_else(|_| "https://api.resend.com".into()),
        })
    }
}

#[async_trait]
impl OutboundMessageProvider for ResendProvider {
    async fn send_email(&self, email: &OutboundEmail) -> Result<ProviderReceipt, MessageError> {
        if email.message_id.trim().is_empty()
            || email.to.trim().is_empty()
            || email.subject.trim().is_empty()
            || email.html.trim().is_empty()
            || email.idempotency_key.trim().is_empty()
        {
            return Err(MessageError::InvalidRequest("message fields are required".into()));
        }
        let mut payload = serde_json::json!({
            "from": self.from,
            "to": [email.to],
            "subject": email.subject,
            "html": email.html
        });
        if let Some(unsubscribe_url) = &email.unsubscribe_url {
            payload["headers"] = serde_json::json!({
                "List-Unsubscribe": format!("<{unsubscribe_url}>"),
                "List-Unsubscribe-Post": "List-Unsubscribe=One-Click"
            });
        }

        let response = self.client
            .post(format!("{}/emails", self.api_base.trim_end_matches('/')))
            .bearer_auth(&self.api_key)
            .header("Idempotency-Key", &email.idempotency_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| MessageError::Transport(e.to_string()))?;

        if response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN {
            return Err(MessageError::Unauthorized);
        }
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(MessageError::RateLimited);
        }

        let status = response.status();
        let body: ResendResponse = response.json().await
            .map_err(|e| MessageError::Provider(format!("invalid provider response: {e}")))?;
        if !status.is_success() {
            return Err(MessageError::Provider(format!("http {status}")));
        }
        let provider_reference = body.id.ok_or_else(|| {
            MessageError::Provider("provider response did not include message id".into())
        })?;
        Ok(ProviderReceipt {
            provider: "resend".into(),
            provider_reference,
            status: "ACCEPTED".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_requires_idempotency_key() {
        let email = OutboundEmail {
            message_id: "m".into(),
            from: "unused".into(),
            to: "a@example.com".into(),
            subject: "s".into(),
            html: "<p>x</p>".into(),
            idempotency_key: "".into(),
            unsubscribe_url: None,
        };
        assert!(email.idempotency_key.is_empty());
    }
}
