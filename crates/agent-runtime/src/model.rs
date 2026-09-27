use async_trait::async_trait;
use serde_json::{json, Value};
use std::{env, fmt};

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
            "summary": "Mock model: use deterministic role policy and do not execute material actions.",
            "confidence": 0.50
        }))
    }
}

pub struct OpenAiCompatibleModel {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatibleModel {
    pub fn from_env() -> Result<Self, ModelError> {
        let api_key = env::var("LLM_API_KEY").map_err(|_| ModelError::MissingConfiguration)?;
        let base_url = env::var("LLM_BASE_URL").map_err(|_| ModelError::MissingConfiguration)?;
        let model = env::var("LLM_MODEL").map_err(|_| ModelError::MissingConfiguration)?;
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
            model,
        })
    }
}

#[async_trait]
impl Model for OpenAiCompatibleModel {
    async fn propose_json(&self, system: &str, user: &str) -> Result<Value, ModelError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = json!({
            "model": self.model,
            "temperature": 0,
            "response_format": {"type": "json_object"},
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        });

        let response = self.client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Transport(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ModelError::Transport(
                response.text().await.unwrap_or_else(|_| "unknown http error".into()),
            ));
        }

        let envelope: Value = response
            .json()
            .await
            .map_err(|e| ModelError::InvalidResponse(e.to_string()))?;

        let content = envelope
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| ModelError::InvalidResponse("missing choices[0].message.content".into()))?;

        serde_json::from_str(content)
            .map_err(|e| ModelError::InvalidResponse(format!("content is not JSON: {e}")))
    }
}

pub fn model_from_env() -> Box<dyn Model> {
    if env::var("LLM_API_KEY").is_ok()
        && env::var("LLM_BASE_URL").is_ok()
        && env::var("LLM_MODEL").is_ok()
    {
        if let Ok(model) = OpenAiCompatibleModel::from_env() {
            return Box::new(model);
        }
    }
    Box::new(MockModel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_model_is_always_available() {
        let result = MockModel.propose_json("CEO", "{}").await.unwrap();
        assert_eq!(result["confidence"], 0.5);
    }
}
