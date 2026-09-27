use crate::{model::Model, types::*};
use async_trait::async_trait;
use std::{fmt, sync::Arc, time::Duration};

#[derive(Debug)]
pub enum AgentError {
    Model(crate::model::ModelError),
    InvalidProposal(String),
    Timeout,
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(e) => write!(f, "{e}"),
            Self::InvalidProposal(e) => write!(f, "invalid proposal: {e}"),
            Self::Timeout => write!(f, "model call timed out"),
        }
    }
}

pub struct AgentContext {
    pub company: CompanySnapshot,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn role(&self) -> AgentRole;
    fn permission(&self) -> Permission;
    fn system_prompt(&self) -> &'static str;
    async fn propose(&self, ctx: &AgentContext, model: Arc<dyn Model>) -> Result<Proposal, AgentError>;
}

pub fn proposal_confidence(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}

pub fn model_context(ctx: &AgentContext) -> String {
    serde_json::to_string(&ctx.company).unwrap_or_else(|_| "{}".into())
}

pub async fn call_model(
    agent: &dyn Agent,
    ctx: &AgentContext,
    model: Arc<dyn Model>,
) -> Result<serde_json::Value, AgentError> {
    let timeout_ms = std::env::var("MODEL_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v >= 250 && *v <= 120_000)
        .unwrap_or(15_000);

    tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        model.propose_json(agent.system_prompt(), &model_context(ctx)),
    )
    .await
    .map_err(|_| AgentError::Timeout)?
    .map_err(AgentError::Model)
}
