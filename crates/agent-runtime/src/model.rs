#![forbid(unsafe_code)]

use async_trait::async_trait;
use reqwest::{Client, StatusCode, Url};
use crate::types::AgentRole;
use serde_json::{json, Value};
use uuid::Uuid;
use std::{
    env,
    fmt,
    fs,
    path::Path,
    sync::Arc,
    time::Duration,
};

const MAX_RESPONSE_BYTES: usize = 1_048_576;
const MAX_ERROR_BYTES: usize = 4_096;
const MAX_WEB_RELAY_REQUEST_BYTES: usize = 512 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug, Clone)]
pub enum ModelError {
    MissingConfiguration,
    Transport(String),
    InvalidResponse(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConfiguration => write!(f, "model configuration is missing"),
            Self::Transport(value) => write!(f, "model transport error: {value}"),
            Self::InvalidResponse(value) => write!(f, "model response error: {value}"),
        }
    }
}

impl std::error::Error for ModelError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRoutingMode {
    Off,
    Shadow,
}

impl ModelRoutingMode {
    pub fn from_env() -> Self {
        match env::var("MODEL_ROUTER_MODE")
            .unwrap_or_else(|_| "shadow".into())
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "off" | "disabled" => Self::Off,
            _ => Self::Shadow,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTaskClass {
    Fast,
    Standard,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareTier {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRequestMetadata {
    pub agent: AgentRole,
    pub system_bytes: usize,
    pub user_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRouteDecision {
    pub task: ModelTaskClass,
    pub hardware: HardwareTier,
    pub recommended_provider: Option<String>,
    pub applied: bool,
}

pub fn classify_model_task(
    agent: AgentRole,
    system_bytes: usize,
    user_bytes: usize,
) -> ModelTaskClass {
    if system_bytes.max(user_bytes) >= 12_000 {
        return ModelTaskClass::Deep;
    }

    match agent {
        AgentRole::CEO | AgentRole::CFO | AgentRole::Analyst => ModelTaskClass::Deep,
        AgentRole::Experiment => ModelTaskClass::Fast,
        _ => ModelTaskClass::Standard,
    }
}

pub fn hardware_tier_for_cores(cores: usize) -> HardwareTier {
    match cores {
        0..=2 => HardwareTier::Small,
        3..=8 => HardwareTier::Medium,
        _ => HardwareTier::Large,
    }
}

fn detected_hardware_tier() -> HardwareTier {
    std::thread::available_parallelism()
        .map(|value| hardware_tier_for_cores(value.get()))
        .unwrap_or(HardwareTier::Medium)
}

fn is_local_provider(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "ollama" | "local" | "mock"
    )
}

fn is_remote_api_provider(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "gemini" | "openai" | "chatgpt" | "anthropic" | "claude"
    )
}

fn recommended_provider(
    providers: &[String],
    task: ModelTaskClass,
    hardware: HardwareTier,
) -> Option<String> {
    match task {
        ModelTaskClass::Fast => providers
            .iter()
            .find(|name| is_local_provider(name))
            .cloned(),
        ModelTaskClass::Deep if hardware == HardwareTier::Small => providers
            .iter()
            .find(|name| is_remote_api_provider(name))
            .cloned()
            .or_else(|| providers.first().cloned()),
        ModelTaskClass::Deep => providers
            .iter()
            .find(|name| !is_local_provider(name))
            .cloned()
            .or_else(|| providers.first().cloned()),
        ModelTaskClass::Standard => providers.first().cloned(),
    }
}

pub fn route_model_request(
    providers: &[String],
    metadata: &ModelRequestMetadata,
    _mode: ModelRoutingMode,
) -> ModelRouteDecision {
    let task = classify_model_task(metadata.agent, metadata.system_bytes, metadata.user_bytes);
    let hardware = detected_hardware_tier();
    ModelRouteDecision {
        task,
        hardware,
        recommended_provider: recommended_provider(providers, task, hardware),
        applied: false,
    }
}

#[async_trait]
pub trait Model: Send + Sync {
    async fn propose_json(&self, system: &str, user: &str) -> Result<Value, ModelError>;

    async fn propose_json_with_metadata(
        &self,
        system: &str,
        user: &str,
        _metadata: ModelRequestMetadata,
    ) -> Result<Value, ModelError> {
        self.propose_json(system, user).await
    }
}

pub struct MockModel;

#[async_trait]
impl Model for MockModel {
    async fn propose_json(&self, system: &str, _user: &str) -> Result<Value, ModelError> {
        Ok(json!({
            "role": system.lines().next().unwrap_or_default(),
            "summary": "mock provider; deterministic agent policy remains authoritative",
            "confidence": 0.5
        }))
    }
}

pub struct FailClosedModel {
    reason: String,
}

impl FailClosedModel {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl Model for FailClosedModel {
    async fn propose_json(&self, _system: &str, _user: &str) -> Result<Value, ModelError> {
        Err(ModelError::Transport(format!(
            "LLM unavailable: {}",
            self.reason
        )))
    }
}

fn secret_from_env(name: &str) -> Result<String, ModelError> {
    let direct = env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty());
    let file = env::var(format!("{name}_FILE"))
        .ok()
        .filter(|value| !value.trim().is_empty());

    if direct.is_some() && file.is_some() {
        return Err(ModelError::MissingConfiguration);
    }

    let raw = match (direct, file) {
        (Some(value), None) => value,
        (None, Some(path)) => {
            let path = Path::new(path.trim());
            if path.is_dir() {
                return Err(ModelError::MissingConfiguration);
            }
            let metadata =
                fs::metadata(path).map_err(|_| ModelError::MissingConfiguration)?;
            if metadata.len() > 16 * 1024 {
                return Err(ModelError::MissingConfiguration);
            }
            fs::read_to_string(path).map_err(|_| ModelError::MissingConfiguration)?
        }
        _ => return Err(ModelError::MissingConfiguration),
    };

    let value = raw.trim().to_owned();
    if value.is_empty() {
        return Err(ModelError::MissingConfiguration);
    }
    Ok(value)
}

fn build_client(user_agent: &'static str, timeout: Duration) -> Result<Client, ModelError> {
    Client::builder()
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(user_agent)
        .build()
        .map_err(|error| ModelError::Transport(error.to_string()))
}

async fn bounded_body(response: reqwest::Response) -> Result<Vec<u8>, ModelError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(ModelError::InvalidResponse(
            "provider response exceeds 1 MiB safety limit".into(),
        ));
    }

    let body = response
        .bytes()
        .await
        .map_err(|error| ModelError::Transport(error.to_string()))?;

    if body.len() > MAX_RESPONSE_BYTES {
        return Err(ModelError::InvalidResponse(
            "provider response exceeds 1 MiB safety limit".into(),
        ));
    }

    Ok(body.to_vec())
}

fn bounded_provider_error(
    status: StatusCode,
    body: &[u8],
    secrets: &[&str],
) -> ModelError {
    let bounded = &body[..body.len().min(MAX_ERROR_BYTES)];
    let mut text = String::from_utf8_lossy(bounded).to_string();

    for secret in secrets.iter().filter(|value| !value.is_empty()) {
        text = text.replace(secret, "[REDACTED]");
    }

    ModelError::Transport(format!("HTTP {status}: {text}"))
}

fn web_relay_request_id() -> String {
    format!("llm-web:{}", Uuid::new_v4())
}

fn parse_json_text(value: &str) -> Result<Value, ModelError> {
    let trimmed = value.trim();
    let normalized = trimmed
        .strip_prefix("json:")
        .map(str::trim)
        .unwrap_or(trimmed);

    serde_json::from_str(normalized).map_err(|error| {
        ModelError::InvalidResponse(format!(
            "LLM output is not valid JSON: {error}"
        ))
    })
}

fn validate_web_relay_url(value: &str) -> Result<(), ModelError> {
    let parsed = Url::parse(value).map_err(|_| ModelError::MissingConfiguration)?;
    let host = parsed.host_str().unwrap_or_default();

    if parsed.scheme() == "https"
        || matches!(host, "127.0.0.1" | "localhost" | "::1")
    {
        return Ok(());
    }

    let allowed_hosts = env::var("LLM_WEB_RELAY_ALLOW_HTTP_HOSTS")
        .unwrap_or_default();

    if parsed.scheme() == "http"
        && allowed_hosts
            .split(',')
            .map(str::trim)
            .any(|allowed| !allowed.is_empty() && allowed == host)
    {
        return Ok(());
    }

    Err(ModelError::MissingConfiguration)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebBackend {
    Gemini,
    ChatGpt,
    Claude,
}

impl WebBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gemini => "gemini-web",
            Self::ChatGpt => "chatgpt-web",
            Self::Claude => "claude-web",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "gemini-web" | "gemini" => Some(Self::Gemini),
            "chatgpt-web" | "chatgpt" => Some(Self::ChatGpt),
            "claude-web" | "claude" => Some(Self::Claude),
            _ => None,
        }
    }
}

pub struct OpenAiCompatibleModel {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl OpenAiCompatibleModel {
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        model: String,
    ) -> Result<Self, ModelError> {
        let client =
            build_client("company-agents-compatible/0.5", Duration::from_secs(30))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
            model,
        })
    }
}

#[async_trait]
impl Model for OpenAiCompatibleModel {
    async fn propose_json(
        &self,
        system: &str,
        user: &str,
    ) -> Result<Value, ModelError> {
        let body = json!({
            "model": self.model,
            "temperature": 0,
            "stream": false,
            "messages": [
                {
                    "role": "system",
                    "content": format!(
                        "{system}\n\nReturn exactly one JSON object. Do not execute tools."
                    )
                },
                {
                    "role": "user",
                    "content": user
                }
            ]
        });

        let mut request = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&body);

        if let Some(api_key) = self.api_key.as_deref().filter(|value| !value.is_empty()) {
            request = request.bearer_auth(api_key);
        }

        let response = request
            .send()
            .await
            .map_err(|error| ModelError::Transport(error.to_string()))?;

        let status = response.status();
        let raw = bounded_body(response).await?;

        if !status.is_success() {
            return Err(bounded_provider_error(
                status,
                &raw,
                self.api_key.as_deref().into_iter().collect::<Vec<_>>().as_slice(),
            ));
        }

        let envelope: Value = serde_json::from_slice(&raw).map_err(|error| {
            ModelError::InvalidResponse(format!(
                "provider response is not JSON: {error}"
            ))
        })?;

        let content = envelope
            .get("choices")
            .and_then(|value| value.get(0))
            .and_then(|value| value.get("message"))
            .and_then(|value| value.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::InvalidResponse(
                    "missing choices[0].message.content".into(),
                )
            })?;

        parse_json_text(content)
    }
}

pub struct OllamaModel(OpenAiCompatibleModel);

impl OllamaModel {
    pub fn from_env() -> Result<Self, ModelError> {
        Ok(Self(OpenAiCompatibleModel::new(
            env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434/v1".into()),
            Some("ollama".into()),
            env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3:4b".into()),
        )?))
    }
}

#[async_trait]
impl Model for OllamaModel {
    async fn propose_json(
        &self,
        system: &str,
        user: &str,
    ) -> Result<Value, ModelError> {
        self.0.propose_json(system, user).await
    }
}

pub struct GeminiInteractionsModel {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiInteractionsModel {
    pub fn from_env() -> Result<Self, ModelError> {
        Ok(Self {
            client: build_client(
                "company-agents-gemini/0.5",
                DEFAULT_TIMEOUT,
            )?,
            api_key: secret_from_env("GEMINI_API_KEY")
                .or_else(|_| secret_from_env("LLM_API_KEY"))?,
            model: env::var("GEMINI_MODEL")
                .unwrap_or_else(|_| "gemini-3.8-flash".into()),
        })
    }
}

#[async_trait]
impl Model for GeminiInteractionsModel {
    async fn propose_json(
        &self,
        system: &str,
        user: &str,
    ) -> Result<Value, ModelError> {
        let body = json!({
            "model": self.model,
            "input": user,
            "system_instruction": system,
            "generation_config": {
                "temperature": 0,
                "max_output_tokens": 4096
            },
            "response_format": {
                "type": "text",
                "mime_type": "application/json"
            }
        });

        let response = self
            .client
            .post("https://generativelanguage.googleapis.com/v1beta/interactions")
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| ModelError::Transport(error.to_string()))?;

        let status = response.status();
        let raw = bounded_body(response).await?;

        if !status.is_success() {
            return Err(bounded_provider_error(
                status,
                &raw,
                &[self.api_key.as_str()],
            ));
        }

        let envelope: Value = serde_json::from_slice(&raw).map_err(|error| {
            ModelError::InvalidResponse(format!(
                "Gemini response is not JSON: {error}"
            ))
        })?;

        let content = envelope
            .get("output_text")
            .and_then(Value::as_str)
            .or_else(|| {
                envelope
                    .get("steps")
                    .and_then(Value::as_array)
                    .and_then(|steps| {
                        steps.iter().rev().find_map(|step| {
                            if step.get("type").and_then(Value::as_str)
                                == Some("model_output")
                            {
                                step.get("content")
                                    .and_then(Value::as_array)
                                    .and_then(|items| {
                                        items.iter().rev().find_map(|item| {
                                            item.get("text")
                                                .and_then(Value::as_str)
                                        })
                                    })
                            } else {
                                None
                            }
                        })
                    })
            })
            .ok_or_else(|| {
                ModelError::InvalidResponse(
                    "missing Gemini Interactions output".into(),
                )
            })?;

        parse_json_text(content)
    }
}

pub struct OpenAiResponsesModel {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiResponsesModel {
    pub fn from_env() -> Result<Self, ModelError> {
        Ok(Self {
            client: build_client(
                "company-agents-openai/0.5",
                DEFAULT_TIMEOUT,
            )?,
            api_key: secret_from_env("OPENAI_API_KEY")
                .or_else(|_| secret_from_env("LLM_API_KEY"))?,
            model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5".into()),
        })
    }
}

#[async_trait]
impl Model for OpenAiResponsesModel {
    async fn propose_json(
        &self,
        system: &str,
        user: &str,
    ) -> Result<Value, ModelError> {
        let body = json!({
            "model": self.model,
            "instructions": format!(
                "{system}\n\nReturn exactly one JSON object. Do not execute tools."
            ),
            "input": user,
            "store": false,
            "text": {
                "format": {
                    "type": "json_object"
                }
            }
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| ModelError::Transport(error.to_string()))?;

        let status = response.status();
        let raw = bounded_body(response).await?;

        if !status.is_success() {
            return Err(bounded_provider_error(
                status,
                &raw,
                &[self.api_key.as_str()],
            ));
        }

        let envelope: Value = serde_json::from_slice(&raw).map_err(|error| {
            ModelError::InvalidResponse(format!(
                "OpenAI response is not JSON: {error}"
            ))
        })?;

        let content = envelope
            .get("output_text")
            .and_then(Value::as_str)
            .or_else(|| {
                envelope
                    .get("output")
                    .and_then(Value::as_array)
                    .and_then(|items| {
                        items.iter().find_map(|item| {
                            item.get("content")
                                .and_then(Value::as_array)
                                .and_then(|blocks| {
                                    blocks.iter().find_map(|block| {
                                        block.get("text")
                                            .and_then(Value::as_str)
                                    })
                                })
                        })
                    })
            })
            .ok_or_else(|| {
                ModelError::InvalidResponse(
                    "missing OpenAI Responses output".into(),
                )
            })?;

        parse_json_text(content)
    }
}

pub struct AnthropicMessagesModel {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicMessagesModel {
    pub fn from_env() -> Result<Self, ModelError> {
        Ok(Self {
            client: build_client(
                "company-agents-anthropic/0.5",
                DEFAULT_TIMEOUT,
            )?,
            api_key: secret_from_env("ANTHROPIC_API_KEY")
                .or_else(|_| secret_from_env("LLM_API_KEY"))?,
            model: env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-opus-4-8".into()),
        })
    }
}

#[async_trait]
impl Model for AnthropicMessagesModel {
    async fn propose_json(
        &self,
        system: &str,
        user: &str,
    ) -> Result<Value, ModelError> {
        let body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": format!(
                "{system}\n\nReturn exactly one JSON object. Do not execute tools."
            ),
            "messages": [
                {
                    "role": "user",
                    "content": user
                }
            ]
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|error| ModelError::Transport(error.to_string()))?;

        let status = response.status();
        let raw = bounded_body(response).await?;

        if !status.is_success() {
            return Err(bounded_provider_error(
                status,
                &raw,
                &[self.api_key.as_str()],
            ));
        }

        let envelope: Value = serde_json::from_slice(&raw).map_err(|error| {
            ModelError::InvalidResponse(format!(
                "Anthropic response is not JSON: {error}"
            ))
        })?;

        let content = envelope
            .get("content")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find_map(|item| {
                    if item.get("type").and_then(Value::as_str) == Some("text") {
                        item.get("text").and_then(Value::as_str)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                ModelError::InvalidResponse(
                    "missing Anthropic text content".into(),
                )
            })?;

        parse_json_text(content)
    }
}

pub struct WebRelayModel {
    client: Client,
    url: String,
    token: String,
    backend: WebBackend,
    model: String,
}

impl WebRelayModel {
    pub fn from_env() -> Result<Self, ModelError> {
        Self::from_env_with_backend(None)
    }

    pub fn from_env_with_backend(
        default_backend: Option<WebBackend>,
    ) -> Result<Self, ModelError> {
        let url =
            env::var("LLM_WEB_RELAY_URL").map_err(|_| ModelError::MissingConfiguration)?;
        let token = secret_from_env("LLM_WEB_RELAY_TOKEN")
            .or_else(|_| secret_from_env("LLM_WEB_RELAY_SECRET"))?;

        validate_web_relay_url(&url)?;

        let configured_backend_raw = env::var("LLM_WEB_RELAY_BACKEND")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let configured_backend = configured_backend_raw
            .as_deref()
            .and_then(WebBackend::parse);

        if configured_backend_raw.is_some() && configured_backend.is_none() {
            return Err(ModelError::MissingConfiguration);
        }

        let backend = match default_backend {
            Some(forced) => forced,
            None => configured_backend.unwrap_or(WebBackend::Gemini),
        };

        Ok(Self {
            client: build_client(
                "company-agents-web-relay/0.5",
                Duration::from_secs(60),
            )?,
            url,
            token,
            backend,
            model: env::var("LLM_WEB_RELAY_MODEL")
                .unwrap_or_else(|_| "web-session".into()),
        })
    }
}

#[async_trait]
impl Model for WebRelayModel {
    async fn propose_json(
        &self,
        system: &str,
        user: &str,
    ) -> Result<Value, ModelError> {
        let request_id = web_relay_request_id();
        let body = json!({
            "protocol_version": 1,
            "backend": self.backend.as_str(),
            "model": self.model,
            "system": system,
            "user": user,
            "response_format": "json_object",
            "allow_tools": false,
            "idempotency_key": request_id
        });

        let encoded = serde_json::to_vec(&body)
            .map_err(|error| ModelError::InvalidResponse(error.to_string()))?;
        if encoded.len() > MAX_WEB_RELAY_REQUEST_BYTES {
            return Err(ModelError::InvalidResponse(
                "web relay request exceeds 512 KiB".into(),
            ));
        }

        let mut response = None;
        for attempt in 0..3_u32 {
            let candidate = self
                .client
                .post(&self.url)
                .bearer_auth(&self.token)
                .json(&body)
                .send()
                .await
                .map_err(|error| ModelError::Transport(error.to_string()))?;
            if (candidate.status() == StatusCode::TOO_MANY_REQUESTS
                || candidate.status().is_server_error())
                && attempt < 2
            {
                let retry_after_ms = candidate
                    .headers()
                    .get("retry-after")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|seconds| seconds.saturating_mul(1_000))
                    .unwrap_or(250 * (1_u64 << attempt));
                tokio::time::sleep(Duration::from_millis(
                    retry_after_ms.clamp(100, 5_000),
                ))
                .await;
                continue;
            }
            response = Some(candidate);
            break;
        }
        let response = response.ok_or_else(|| {
            ModelError::Transport("web relay retry loop exhausted".into())
        })?;

        let status = response.status();
        let raw = bounded_body(response).await?;

        if !status.is_success() {
            return Err(bounded_provider_error(
                status,
                &raw,
                &[self.token.as_str()],
            ));
        }

        let envelope: Value = serde_json::from_slice(&raw).map_err(|error| {
            ModelError::InvalidResponse(format!(
                "web relay response is not JSON: {error}"
            ))
        })?;

        let output = envelope
            .get("output")
            .or_else(|| envelope.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::InvalidResponse("web relay missing output".into())
            })?;

        parse_json_text(output)
    }
}

pub struct FallbackModel {
    providers: Vec<(String, Arc<dyn Model>)>,
}

impl FallbackModel {
    pub fn new(providers: Vec<(String, Arc<dyn Model>)>) -> Result<Self, ModelError> {
        if providers.is_empty() {
            return Err(ModelError::MissingConfiguration);
        }
        Ok(Self { providers })
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers
            .iter()
            .map(|(name, _)| name.clone())
            .collect()
    }
}

pub struct RoutingModel {
    inner: Arc<FallbackModel>,
    mode: ModelRoutingMode,
}

impl RoutingModel {
    pub fn new(inner: Arc<FallbackModel>, mode: ModelRoutingMode) -> Self {
        Self { inner, mode }
    }
}

#[async_trait]
impl Model for RoutingModel {
    async fn propose_json(&self, system: &str, user: &str) -> Result<Value, ModelError> {
        self.inner.propose_json(system, user).await
    }

    async fn propose_json_with_metadata(
        &self,
        system: &str,
        user: &str,
        metadata: ModelRequestMetadata,
    ) -> Result<Value, ModelError> {
        let decision = route_model_request(&self.inner.provider_names(), &metadata, self.mode);
        if self.mode == ModelRoutingMode::Shadow {
            tracing::debug!(
                task=?decision.task,
                hardware=?decision.hardware,
                recommended_provider=?decision.recommended_provider,
                agent=?metadata.agent,
                "model routing shadow decision"
            );
        }
        self.inner.propose_json(system, user).await
    }
}

#[async_trait]
impl Model for FallbackModel {
    async fn propose_json(
        &self,
        system: &str,
        user: &str,
    ) -> Result<Value, ModelError> {
        let mut failures = Vec::new();

        for (name, provider) in &self.providers {
            match provider.propose_json(system, user).await {
                Ok(value) => return Ok(value),
                Err(error) => failures.push(format!("{name}: {error}")),
            }
        }

        Err(ModelError::Transport(format!(
            "all configured LLM providers failed: {}",
            failures.join(" | ")
        )))
    }
}

fn build_provider(name: &str) -> Result<Arc<dyn Model>, ModelError> {
    match name.trim().to_ascii_lowercase().as_str() {
        "mock" => Ok(Arc::new(MockModel)),
        "ollama" | "local" => Ok(Arc::new(OllamaModel::from_env()?)),
        "gemini" => Ok(Arc::new(GeminiInteractionsModel::from_env()?)),
        "openai" | "chatgpt" => Ok(Arc::new(OpenAiResponsesModel::from_env()?)),
        "anthropic" | "claude" => Ok(Arc::new(AnthropicMessagesModel::from_env()?)),
        "web" | "web-relay" => Ok(Arc::new(WebRelayModel::from_env()?)),
        "chatgpt-subscription" | "chatgpt-cli" => Ok(Arc::new(
            WebRelayModel::from_env_with_backend(Some(WebBackend::ChatGpt))?
        )),
        "claude-subscription" | "claude-code" => Ok(Arc::new(
            WebRelayModel::from_env_with_backend(Some(WebBackend::Claude))?
        )),
        "gemini-web" => Ok(Arc::new(
            WebRelayModel::from_env_with_backend(Some(WebBackend::Gemini))?
        )),
        "chatgpt-web" => Ok(Arc::new(
            WebRelayModel::from_env_with_backend(Some(WebBackend::ChatGpt))?
        )),
        "claude-web" => Ok(Arc::new(
            WebRelayModel::from_env_with_backend(Some(WebBackend::Claude))?
        )),
        "openai-compatible" => {
            let base_url =
                env::var("LLM_BASE_URL").map_err(|_| ModelError::MissingConfiguration)?;
            let model =
                env::var("LLM_MODEL").map_err(|_| ModelError::MissingConfiguration)?;
            Ok(Arc::new(OpenAiCompatibleModel::new(
                base_url,
                secret_from_env("LLM_API_KEY").ok(),
                model,
            )?))
        }
        other => Err(ModelError::Transport(format!(
            "unknown LLM provider '{other}'"
        ))),
    }
}

pub fn model_from_env() -> Box<dyn Model> {
    let primary =
        env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".into());
    let mut names = vec![primary];

    if let Ok(fallbacks) = env::var("LLM_FALLBACKS") {
        names.extend(
            fallbacks
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
        );
    }

    let strict_primary = env::var("LLM_STRICT_CONFIG")
        .ok()
        .is_some_and(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        });

    let mut providers = Vec::new();

    for (index, name) in names.into_iter().enumerate() {
        match build_provider(&name) {
            Ok(provider) => providers.push((name, provider)),
            Err(error) if index == 0 && strict_primary => {
                return Box::new(FailClosedModel::new(error.to_string()));
            }
            Err(_error) => {}
        }
    }

    let fallback = match FallbackModel::new(providers) {
        Ok(provider) => Arc::new(provider),
        Err(error) => return Box::new(FailClosedModel::new(error.to_string())),
    };

    match ModelRoutingMode::from_env() {
        ModelRoutingMode::Off => Box::new(fallback),
        ModelRoutingMode::Shadow => {
            Box::new(RoutingModel::new(fallback, ModelRoutingMode::Shadow))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_classifies_tasks_deterministically() {
        assert_eq!(
            classify_model_task(AgentRole::CEO, 100, 100),
            ModelTaskClass::Deep
        );
        assert_eq!(
            classify_model_task(AgentRole::Experiment, 100, 100),
            ModelTaskClass::Fast
        );
        assert_eq!(
            classify_model_task(AgentRole::Growth, 100, 100),
            ModelTaskClass::Standard
        );
        assert_eq!(
            classify_model_task(AgentRole::Growth, 13_000, 100),
            ModelTaskClass::Deep
        );
    }

    #[test]
    fn hardware_tier_respects_core_count() {
        assert_eq!(hardware_tier_for_cores(1), HardwareTier::Small);
        assert_eq!(hardware_tier_for_cores(4), HardwareTier::Medium);
        assert_eq!(hardware_tier_for_cores(16), HardwareTier::Large);
    }

    #[test]
    fn routing_recommends_local_for_fast_work_when_available() {
        let providers = vec!["gemini".into(), "ollama".into()];
        let meta = ModelRequestMetadata {
            agent: AgentRole::Experiment,
            system_bytes: 100,
            user_bytes: 100,
        };
        let decision = route_model_request(&providers, &meta, ModelRoutingMode::Shadow);
        assert_eq!(decision.task, ModelTaskClass::Fast);
        assert_eq!(decision.recommended_provider.as_deref(), Some("ollama"));
        assert!(!decision.applied);
    }

    #[test]
    fn routing_prefers_remote_for_deep_work_on_small_hardware() {
        let providers = vec!["ollama".into(), "gemini".into(), "anthropic".into()];
        let meta = ModelRequestMetadata {
            agent: AgentRole::CEO,
            system_bytes: 100,
            user_bytes: 100,
        };
        let task = classify_model_task(meta.agent, meta.system_bytes, meta.user_bytes);
        assert_eq!(task, ModelTaskClass::Deep);
        let recommended = recommended_provider(&providers, task, HardwareTier::Small);
        assert_eq!(recommended.as_deref(), Some("gemini"));
    }

    #[tokio::test]
    async fn mock_model_is_available_without_external_services() {
        let value = MockModel.propose_json("CEO", "{}").await.unwrap();
        assert_eq!(value["confidence"], 0.5);
    }

    #[test]
    fn web_backend_aliases_are_canonical() {
        assert_eq!(
            WebBackend::parse("gemini-web"),
            Some(WebBackend::Gemini)
        );
        assert_eq!(
            WebBackend::parse("chatgpt-web"),
            Some(WebBackend::ChatGpt)
        );
        assert_eq!(
            WebBackend::parse("claude-web"),
            Some(WebBackend::Claude)
        );
        assert_eq!(WebBackend::parse("unknown"), None);
    }

    #[test]
    fn provider_aliases_cover_subscription_and_web_routes() {
        assert_eq!(WebBackend::parse("chatgpt-subscription"), None);
        assert_eq!(WebBackend::parse("chatgpt-web"), Some(WebBackend::ChatGpt));
        assert_eq!(WebBackend::parse("claude-subscription"), None);
        assert_eq!(WebBackend::parse("claude-web"), Some(WebBackend::Claude));
        assert_eq!(WebBackend::parse("gemini-web"), Some(WebBackend::Gemini));
    }

    #[test]
    fn web_relay_url_requires_tls_unless_loopback_or_allowlisted() {
        assert!(validate_web_relay_url(
            "https://relay.example.internal/v1/generate"
        )
        .is_ok());
        assert!(validate_web_relay_url(
            "http://127.0.0.1:9010/v1/generate"
        )
        .is_ok());
        assert!(validate_web_relay_url(
            "http://relay.example.internal/v1/generate"
        )
        .is_err());
    }

    #[test]
    fn web_relay_request_ids_are_unique() {
        let a = web_relay_request_id();
        let b = web_relay_request_id();
        assert_ne!(a, b);
        assert!(a.starts_with("llm-web:"));
        assert!(b.starts_with("llm-web:"));
    }

    #[test]
    fn parse_json_text_accepts_plain_json() {
        let value = parse_json_text("{\"ok\":true}").unwrap();
        assert_eq!(value["ok"], true);
    }

    #[tokio::test]
    async fn fallback_uses_first_healthy_provider() {
        struct Bad;

        #[async_trait]
        impl Model for Bad {
            async fn propose_json(
                &self,
                _system: &str,
                _user: &str,
            ) -> Result<Value, ModelError> {
                Err(ModelError::Transport("down".into()))
            }
        }

        struct Good;

        #[async_trait]
        impl Model for Good {
            async fn propose_json(
                &self,
                _system: &str,
                _user: &str,
            ) -> Result<Value, ModelError> {
                Ok(json!({"ok": true}))
            }
        }

        let fallback = FallbackModel::new(vec![
            ("bad".into(), Arc::new(Bad)),
            ("good".into(), Arc::new(Good)),
        ])
        .unwrap();

        assert_eq!(
            fallback.propose_json("x", "y").await.unwrap()["ok"],
            true
        );
        assert_eq!(
            fallback.provider_names(),
            vec!["bad".to_string(), "good".to_string()]
        );
    }

    #[tokio::test]
    async fn fallback_fails_closed_when_every_provider_fails() {
        struct Bad;

        #[async_trait]
        impl Model for Bad {
            async fn propose_json(
                &self,
                _system: &str,
                _user: &str,
            ) -> Result<Value, ModelError> {
                Err(ModelError::InvalidResponse("bad".into()))
            }
        }

        let fallback =
            FallbackModel::new(vec![("bad".into(), Arc::new(Bad))]).unwrap();
        assert!(matches!(
            fallback.propose_json("x", "y").await,
            Err(ModelError::Transport(_))
        ));
    }
}
