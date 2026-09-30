#![forbid(unsafe_code)]

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use rand::RngCore;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{env, fmt, time::Duration};
use url::Url;
use uuid::Uuid;

const DEFAULT_AUTH_URL: &str = "https://www.tiktok.com/v2/auth/authorize/";
const DEFAULT_TOKEN_URL: &str = "https://open.tiktokapis.com/v2/oauth/token/";
const DEFAULT_REVOKE_URL: &str = "https://open.tiktokapis.com/v2/oauth/revoke/";
const MAX_RESPONSE_BYTES: usize = 128 * 1024;
const STATE_TTL_SECONDS: i64 = 600;

#[derive(Debug, Clone)]
pub enum AuthError {
    Configuration(String),
    InvalidRequest(String),
    Unauthorized,
    RateLimited,
    Provider(String),
    Transport(String),
    Crypto(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(v) => write!(f, "TikTok auth configuration error: {v}"),
            Self::InvalidRequest(v) => write!(f, "TikTok auth request error: {v}"),
            Self::Unauthorized => write!(f, "TikTok authorization failed"),
            Self::RateLimited => write!(f, "TikTok authorization endpoint rate limited"),
            Self::Provider(v) => write!(f, "TikTok auth provider error: {v}"),
            Self::Transport(v) => write!(f, "TikTok auth transport error: {v}"),
            Self::Crypto(v) => write!(f, "TikTok token encryption error: {v}"),
        }
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_key: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: String,
    pub authorize_url: String,
    pub token_url: String,
    pub revoke_url: String,
}

impl OAuthConfig {
    pub fn from_env() -> Result<Self, AuthError> {
        let client_key = required_env("TIKTOK_CLIENT_KEY")?;
        let client_secret = required_env("TIKTOK_CLIENT_SECRET")?;
        let redirect_uri = required_env("TIKTOK_OAUTH_REDIRECT_URI")?;
        let scopes = env::var("TIKTOK_OAUTH_SCOPES")
            .unwrap_or_else(|_| "user.info.basic,video.publish,video.upload".into())
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(",");
        if scopes.is_empty() {
            return Err(AuthError::Configuration("TIKTOK_OAUTH_SCOPES cannot be empty".into()));
        }

        let redirect = Url::parse(&redirect_uri)
            .map_err(|_| AuthError::Configuration("TIKTOK_OAUTH_REDIRECT_URI must be an absolute URL".into()))?;
        if redirect.scheme() != "https" || redirect.query().is_some() || redirect.fragment().is_some() {
            return Err(AuthError::Configuration(
                "TikTok web redirect URI must use https and contain no query or fragment".into(),
            ));
        }

        Ok(Self {
            client_key,
            client_secret,
            redirect_uri,
            scopes,
            authorize_url: env::var("TIKTOK_OAUTH_AUTHORIZE_URL").unwrap_or_else(|_| DEFAULT_AUTH_URL.into()),
            token_url: env::var("TIKTOK_OAUTH_TOKEN_URL").unwrap_or_else(|_| DEFAULT_TOKEN_URL.into()),
            revoke_url: env::var("TIKTOK_OAUTH_REVOKE_URL").unwrap_or_else(|_| DEFAULT_REVOKE_URL.into()),
        })
    }

    pub fn default_state_ttl_seconds() -> i64 {
        STATE_TTL_SECONDS
    }

    pub fn authorize_url(&self, state: &str) -> Result<String, AuthError> {
        if state.trim().is_empty() || state.len() > 256 {
            return Err(AuthError::InvalidRequest("OAuth state is invalid".into()));
        }
        let mut url = Url::parse(&self.authorize_url)
            .map_err(|_| AuthError::Configuration("TIKTOK_OAUTH_AUTHORIZE_URL is invalid".into()))?;
        {
            let mut query = url.query_pairs_mut();
            query
                .append_pair("client_key", &self.client_key)
                .append_pair("scope", &self.scopes)
                .append_pair("response_type", "code")
                .append_pair("redirect_uri", &self.redirect_uri)
                .append_pair("state", state);
        }
        Ok(url.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenSet {
    pub open_id: String,
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_expires_in: i64,
    pub scope: String,
    pub token_type: String,
}

impl TokenSet {
    pub fn validate(&self) -> Result<(), AuthError> {
        if self.open_id.trim().is_empty() || self.open_id.len() > 256
            || self.access_token.trim().is_empty()
            || self.refresh_token.trim().is_empty()
        {
            return Err(AuthError::Provider("TikTok token response is missing identifiers or tokens".into()));
        }
        if !(1..=31_536_000).contains(&self.expires_in)
            || !(1..=63_072_000).contains(&self.refresh_expires_in)
        {
            return Err(AuthError::Provider("TikTok token expiry is outside supported bounds".into()));
        }
        if self.scope.len() > 4_096 || self.token_type.len() > 32 {
            return Err(AuthError::Provider("TikTok token response is too large".into()));
        }
        Ok(())
    }

    pub fn access_expires_at(&self, now_epoch: i64) -> Result<i64, AuthError> {
        now_epoch
            .checked_add(self.expires_in)
            .ok_or_else(|| AuthError::InvalidRequest("access token expiry overflow".into()))
    }

    pub fn refresh_expires_at(&self, now_epoch: i64) -> Result<i64, AuthError> {
        now_epoch
            .checked_add(self.refresh_expires_in)
            .ok_or_else(|| AuthError::InvalidRequest("refresh token expiry overflow".into()))
    }
}

#[derive(Debug, Deserialize)]
struct TokenErrorResponse {
    error: Option<String>,
    error_description: Option<String>,
    log_id: Option<String>,
}

#[derive(Clone)]
pub struct TikTokOAuthClient {
    pub config: OAuthConfig,
    client: reqwest::Client,
}

impl TikTokOAuthClient {
    pub fn new(config: OAuthConfig) -> Result<Self, AuthError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| AuthError::Transport(error.to_string()))?;
        Ok(Self { config, client })
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenSet, AuthError> {
        if code.trim().is_empty() || code.len() > 4096 {
            return Err(AuthError::InvalidRequest("authorization code is invalid".into()));
        }
        self.token_request(&[
            ("client_key", self.config.client_key.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.config.redirect_uri.as_str()),
        ]).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenSet, AuthError> {
        if refresh_token.trim().is_empty() || refresh_token.len() > 8192 {
            return Err(AuthError::InvalidRequest("refresh token is invalid".into()));
        }
        self.token_request(&[
            ("client_key", self.config.client_key.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ]).await
    }

    pub async fn revoke(&self, access_token: &str) -> Result<(), AuthError> {
        if access_token.trim().is_empty() || access_token.len() > 8192 {
            return Err(AuthError::InvalidRequest("access token is invalid".into()));
        }
        let response = self.client
            .post(&self.config.revoke_url)
            .form(&[
                ("client_key", self.config.client_key.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("token", access_token),
            ])
            .send()
            .await
            .map_err(|error| AuthError::Transport(error.to_string()))?;
        let status = response.status();
        let bytes = bounded_body_async(response).await?;
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(AuthError::Unauthorized);
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AuthError::RateLimited);
        }
        if !status.is_success() {
            return Err(provider_error(status, &bytes, &[self.config.client_secret.as_str()]));
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| AuthError::Provider(format!("invalid revoke response: {error}")))?;
        if value.get("error").and_then(|v| v.as_str()).is_some_and(|v| v != "ok") {
            return Err(AuthError::Provider(
                value.get("error_description").and_then(|v| v.as_str()).unwrap_or("TikTok revoke failed").into(),
            ));
        }
        Ok(())
    }

    async fn token_request(&self, form: &[(&str, &str)]) -> Result<TokenSet, AuthError> {
        let response = self.client
            .post(&self.config.token_url)
            .form(form)
            .send()
            .await
            .map_err(|error| AuthError::Transport(error.to_string()))?;
        let status = response.status();
        let bytes = bounded_body_async(response).await?;
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(AuthError::Unauthorized);
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AuthError::RateLimited);
        }
        let response_text = String::from_utf8_lossy(&bytes);
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| AuthError::Provider(format!("invalid token response: {error}")))?;
        if !status.is_success() {
            return Err(provider_error(status, &bytes, &[self.config.client_secret.as_str()]));
        }
        if value.get("error").and_then(|v| v.as_str()).is_some_and(|v| v != "ok" && v != "null") {
            let detail: TokenErrorResponse = serde_json::from_value(value)
                .unwrap_or(TokenErrorResponse { error: None, error_description: None, log_id: None });
            let log = detail.log_id.unwrap_or_default();
            return Err(AuthError::Provider(format!(
                "token request failed: {} {}",
                detail.error.unwrap_or_else(|| "unknown".into()),
                detail.error_description.unwrap_or_else(|| log_or_body(log, &response_text))
            )));
        }
        let token: TokenSet = serde_json::from_value(value)
            .map_err(|error| AuthError::Provider(format!("token response schema invalid: {error}")))?;
        token.validate()?;
        Ok(token)
    }
}

async fn bounded_body_async(response: reqwest::Response) -> Result<Vec<u8>, AuthError> {
    if response.content_length().is_some_and(|length| length > MAX_RESPONSE_BYTES as u64) {
        return Err(AuthError::Provider("TikTok auth response exceeds safety limit".into()));
    }
    let body = response.bytes().await.map_err(|error| AuthError::Transport(error.to_string()))?;
    if body.len() > MAX_RESPONSE_BYTES {
        return Err(AuthError::Provider("TikTok auth response exceeds safety limit".into()));
    }
    Ok(body.to_vec())
}

fn provider_error(status: StatusCode, body: &[u8], secrets: &[&str]) -> AuthError {
    let bounded = &body[..body.len().min(4_096)];
    let mut text = String::from_utf8_lossy(bounded).to_string();
    for secret in secrets.iter().filter(|value| !value.is_empty()) {
        text = text.replace(secret, "[REDACTED]");
    }
    AuthError::Provider(format!("http {status}: {text}"))
}

fn log_or_body(log: String, body: &str) -> String {
    if log.is_empty() {
        body.chars().take(512).collect()
    } else {
        log
    }
}

fn required_env(name: &str) -> Result<String, AuthError> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_owned())
        .ok_or_else(|| AuthError::Configuration(format!("{name} is required")))
}

pub fn new_state() -> String {
    Uuid::new_v4().to_string()
}

pub fn hash_state(state: &str) -> String {
    format!("sha256:{}", Sha256::digest(state.as_bytes()).iter().map(|b| format!("{b:02x}")).collect::<String>())
}

#[derive(Clone)]
pub struct TokenCipher {
    cipher: Aes256Gcm,
}

impl TokenCipher {
    pub fn from_env() -> Result<Self, AuthError> {
        let raw = env::var("TIKTOK_TOKEN_ENCRYPTION_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| AuthError::Configuration("TIKTOK_TOKEN_ENCRYPTION_KEY is required".into()))?;
        if raw.trim().len() < 32 {
            return Err(AuthError::Configuration("TIKTOK_TOKEN_ENCRYPTION_KEY must be at least 32 characters".into()));
        }
        let digest = Sha256::digest(raw.as_bytes());
        let key = Key::<Aes256Gcm>::from_slice(&digest);
        Ok(Self { cipher: Aes256Gcm::new(key) })
    }

    pub fn encrypt(&self, company_id: Uuid, plaintext: &str) -> Result<String, AuthError> {
        if plaintext.is_empty() || plaintext.len() > 8192 {
            return Err(AuthError::Crypto("token plaintext has invalid size".into()));
        }
        let mut nonce_bytes = [0_u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let aad = company_id.as_bytes();
        let ciphertext = self.cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), aes_gcm::aead::Payload { msg: plaintext.as_bytes(), aad })
            .map_err(|_| AuthError::Crypto("token encryption failed".into()))?;
        Ok(format!(
            "v1:{}:{}",
            STANDARD_NO_PAD.encode(nonce_bytes),
            STANDARD_NO_PAD.encode(ciphertext)
        ))
    }

    pub fn decrypt(&self, company_id: Uuid, value: &str) -> Result<String, AuthError> {
        let mut parts = value.split(':');
        if parts.next() != Some("v1") {
            return Err(AuthError::Crypto("unsupported token cipher version".into()));
        }
        let nonce = parts.next().ok_or_else(|| AuthError::Crypto("token nonce missing".into()))?;
        let ciphertext = parts.next().ok_or_else(|| AuthError::Crypto("token ciphertext missing".into()))?;
        if parts.next().is_some() {
            return Err(AuthError::Crypto("token ciphertext encoding is invalid".into()));
        }
        let nonce_bytes = STANDARD_NO_PAD.decode(nonce.as_bytes())
            .map_err(|_| AuthError::Crypto("token nonce encoding is invalid".into()))?;
        let cipher_bytes = STANDARD_NO_PAD.decode(ciphertext.as_bytes())
            .map_err(|_| AuthError::Crypto("token ciphertext encoding is invalid".into()))?;
        if nonce_bytes.len() != 12 || cipher_bytes.len() > 16 * 1024 {
            return Err(AuthError::Crypto("token cipher payload is invalid".into()));
        }
        let plaintext = self.cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), aes_gcm::aead::Payload { msg: &cipher_bytes, aad: company_id.as_bytes() })
            .map_err(|_| AuthError::Crypto("token decryption failed".into()))?;
        String::from_utf8(plaintext).map_err(|_| AuthError::Crypto("token plaintext is not UTF-8".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_redirect_uri_rejects_query_and_non_https() {
        let mut config = OAuthConfig {
            client_key: "key".into(),
            client_secret: "secret".into(),
            redirect_uri: "https://example.com/callback".into(),
            scopes: "user.info.basic".into(),
            authorize_url: DEFAULT_AUTH_URL.into(),
            token_url: DEFAULT_TOKEN_URL.into(),
            revoke_url: DEFAULT_REVOKE_URL.into(),
        };
        assert!(config.authorize_url("state").is_ok());
        config.redirect_uri = "https://example.com/callback?company=x".into();
        assert!(Url::parse(&config.redirect_uri).unwrap().query().is_some());
        config.redirect_uri = "http://example.com/callback".into();
        assert_eq!(Url::parse(&config.redirect_uri).unwrap().scheme(), "http");
    }

    #[test]
    fn state_hash_is_deterministic() {
        assert_eq!(hash_state("abc"), hash_state("abc"));
        assert_ne!(hash_state("abc"), hash_state("abd"));
    }

    #[test]
    fn token_cipher_roundtrips_and_binds_company() {
        let cipher = TokenCipher {
            cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&Sha256::digest(b"unit-test"))),
        };
        let company = Uuid::new_v4();
        let encrypted = cipher.encrypt(company, "refresh-token").unwrap();
        assert_eq!(cipher.decrypt(company, &encrypted).unwrap(), "refresh-token");
        assert!(cipher.decrypt(Uuid::new_v4(), &encrypted).is_err());
    }

    #[test]
    fn token_set_expiry_is_bounded() {
        let token = TokenSet {
            open_id: "open".into(),
            access_token: "access".into(),
            expires_in: 86400,
            refresh_token: "refresh".into(),
            refresh_expires_in: 31536000,
            scope: "user.info.basic".into(),
            token_type: "Bearer".into(),
        };
        assert!(token.validate().is_ok());
        assert_eq!(token.access_expires_at(100).unwrap(), 86_500);
    }
}
