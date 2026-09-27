use async_trait::async_trait;
use serde_json::{json, Value};
use std::{env, fmt, fs, path::Path, time::Duration};

#[derive(Debug)]
pub enum ModelError {
    MissingConfiguration,
    Transport(String),
    InvalidResponse(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConfiguration => write!(f, "model configuration is missing"),
            Self::Transport(e) => write!(f, "model transport error: {e}"),
            Self::InvalidResponse(e) => write!(f, "model response error: {e}"),
        }
    }
}

#[async_trait]
pub trait Model: Send + Sync {
    async fn propose_json(&self, system: &str, user: &str) -> Result<Value, ModelError>;
}

pub struct MockModel;

#[async_trait]
impl Model for MockModel {
    async fn propose_json(&self, system: &str, _user: &str) -> Result<Value, ModelError> {
        let role = system.lines().next().unwrap_or_default();
        Ok(json!({
            "role": role,
            "summary": "Mock model: deterministic policy only; no material execution.",
            "confidence": 0.50
        }))
    }
}

pub struct OpenAiCompatibleModel {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

fn secret_from_env(name: &str) -> Result<String, ModelError> {
    let file_key = format!("{name}_FILE");
    let direct = env::var(name).ok();
    let file = env::var(&file_key).ok();

    if direct.is_some() && file.is_some() {
        return Err(ModelError::MissingConfiguration);
    }

    let value = match (direct, file) {
        (Some(value), None) => value,
        (None, Some(path)) => {
            if path.trim().is_empty() || Path::new(&path).is_dir() {
                return Err(ModelError::MissingConfiguration);
            }
            let metadata = fs::metadata(&path).map_err(|_| ModelError::MissingConfiguration)?;
            if metadata.len() > 16 * 1024 {
                return Err(ModelError::MissingConfiguration);
            }
            fs::read_to_string(&path).map_err(|_| ModelError::MissingConfiguration)?
        }
        (None, None) => return Err(ModelError::MissingConfiguration),
        _ => unreachable!(),
    };

    let trimmed = value.trim().to_owned();
    if trimmed.is_empty() {
        return Err(ModelError::MissingConfiguration);
    }
    Ok(trimmed)
}

impl OpenAiCompatibleModel {
    pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("company-agents-runtime/0.1")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
            model,
        }
    }

    fn request(&self, system: &str, user: &str) -> Value {
        json!({
            "model": self.model,
            "temperature": 0,
            "stream": false,
            "messages": [
                {"role": "system", "content": format!("{system}\n\nReturn one JSON object. Optional keys: action, objective, cost_minor, expected_revenue_minor, risk, confidence, rationale, reversible, summary. Do not execute tools.")},
                {"role": "user", "content": user}
            ]
        })
    }
}

#[async_trait]
impl Model for OpenAiCompatibleModel {
    async fn propose_json(&self, system: &str, user: &str) -> Result<Value, ModelError> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut request = self.client.post(url).json(&self.request(system, user));

        if let Some(api_key) = &self.api_key {
            if !api_key.trim().is_empty() {
                request = request.bearer_auth(api_key);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| ModelError::Transport(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(|e| ModelError::Transport(e.to_string()))?;
            let bounded = &body[..body.len().min(4_096)];
            let mut detail = String::from_utf8_lossy(bounded).to_string();
            if let Some(api_key) = &self.api_key {
                if !api_key.trim().is_empty() {
                    detail = detail.replace(api_key, "[REDACTED]");
                }
            }
            return Err(ModelError::Transport(format!("HTTP {status}: {detail}")));
        }

        if response.content_length().is_some_and(|len| len > 1_048_576) {
            return Err(ModelError::InvalidResponse(
                "model response exceeds 1 MiB safety limit".into(),
            ));
        }

        let body = response
            .bytes()
            .await
            .map_err(|e| ModelError::Transport(e.to_string()))?;

        if body.len() > 1_048_576 {
            return Err(ModelError::InvalidResponse(
                "model response exceeds 1 MiB safety limit".into(),
            ));
        }

        let envelope: Value = serde_json::from_slice(&body).map_err(|e| {
            ModelError::InvalidResponse(format!("provider response is not JSON: {e}"))
        })?;

        let content = envelope
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::InvalidResponse("missing choices[0].message.content".into())
            })?;

        let normalized = content
            .trim()
            .strip_prefix("```json")
            .and_then(|v| v.strip_suffix("```"))
            .map(str::trim)
            .unwrap_or_else(|| content.trim());

        serde_json::from_str(normalized)
            .map_err(|e| ModelError::InvalidResponse(format!("content is not valid JSON: {e}")))
    }
}

pub struct OllamaModel(OpenAiCompatibleModel);

impl OllamaModel {
    pub fn from_env() -> Self {
        let base_url =
            env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434/v1".into());
        let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3:4b".into());
        Self(OpenAiCompatibleModel::new(
            base_url,
            Some("ollama".into()),
            model,
        ))
    }
}

#[async_trait]
impl Model for OllamaModel {
    async fn propose_json(&self, system: &str, user: &str) -> Result<Value, ModelError> {
        self.0.propose_json(system, user).await
    }
}

pub struct GeminiModel(OpenAiCompatibleModel);

impl GeminiModel {
    pub fn from_env() -> Result<Self, ModelError> {
        let api_key = secret_from_env("GEMINI_API_KEY")
            .or_else(|_| secret_from_env("LLM_API_KEY"))?;

        let model = env::var("GEMINI_MODEL")
            .or_else(|_| env::var("LLM_MODEL"))
            .unwrap_or_else(|_| "gemini-3.6-flash".into());

        Ok(Self(OpenAiCompatibleModel::new(
            "https://generativelanguage.googleapis.com/v1beta/openai/".into(),
            Some(api_key),
            model,
        )))
    }
}

#[async_trait]
impl Model for GeminiModel {
    async fn propose_json(&self, system: &str, user: &str) -> Result<Value, ModelError> {
        self.0.propose_json(system, user).await
    }
}

pub fn model_from_env() -> Box<dyn Model> {
    let provider = env::var("LLM_PROVIDER")
        .unwrap_or_else(|_| "ollama".into())
        .to_ascii_lowercase();

    match provider.as_str() {
        "mock" => Box::new(MockModel),
        "ollama" | "local" => Box::new(OllamaModel::from_env()),
        "gemini" => match GeminiModel::from_env() {
            Ok(model) => Box::new(model),
            Err(_) => Box::new(MockModel),
        },
        "openai-compatible" => {
            let base_url = env::var("LLM_BASE_URL").ok();
            let key = secret_from_env("LLM_API_KEY").ok();
            let model = env::var("LLM_MODEL").ok();
            match (base_url, model) {
                (Some(base_url), Some(model)) => {
                    Box::new(OpenAiCompatibleModel::new(base_url, key, model))
                }
                _ => Box::new(MockModel),
            }
        }
        _ => Box::new(MockModel),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_model_is_always_available() {
        let result = MockModel.propose_json("CEO", "{}").await.unwrap();
        assert_eq!(result["confidence"], 0.5);
    }

    #[tokio::test]
    async fn invalid_json_is_rejected() {
        struct BadModel;
        #[async_trait]
        impl Model for BadModel {
            async fn propose_json(&self, _: &str, _: &str) -> Result<Value, ModelError> {
                Err(ModelError::InvalidResponse(
                    "content is not valid JSON".into(),
                ))
            }
        }

        let result = BadModel.propose_json("x", "y").await;
        assert!(matches!(result, Err(ModelError::InvalidResponse(_))));
    }

    #[test]
    fn defaults_to_local_ollama_configuration_shape() {
        let model = OllamaModel::from_env();
        assert_eq!(model.0.base_url, "http://127.0.0.1:11434/v1");
        assert!(!model.0.model.is_empty());
    }
}
