use crate::{model::Model, types::*};
use async_trait::async_trait;
use std::{fmt, sync::Arc};

#[derive(Debug)]
pub enum AgentError {
    Model(crate::model::ModelError),
    InvalidProposal(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(e) => write!(f, "{e}"),
            Self::InvalidProposal(e) => write!(f, "invalid proposal: {e}"),
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
