use crate::{
    agent::{Agent, AgentContext},
    governor::Governor,
    model::Model,
    roles::executive_agents,
    types::{AgentRole, AgentRunResult, CompanySnapshot},
};
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct AgentRuntime {
    agents: Vec<Arc<dyn Agent>>,
    governor: Governor,
    model: Arc<dyn Model>,
    concurrency: Arc<Semaphore>,
}

impl AgentRuntime {
    pub fn new(model: Box<dyn Model>) -> Self {
        Self::new_with_concurrency(model, 4)
    }

    pub fn new_with_concurrency(model: Box<dyn Model>, max_concurrent: usize) -> Self {
        Self {
            agents: executive_agents(),
            governor: Governor,
            model: Arc::from(model),
            concurrency: Arc::new(Semaphore::new(max_concurrent.max(1))),
        }
    }

    pub fn agent_roles(&self) -> Vec<AgentRole> {
        self.agents.iter().map(|a| a.role()).collect()
    }

    pub async fn run_all(&self, company: CompanySnapshot) -> Vec<AgentRunResult> {
        self.run_all_with_timeout(company, std::time::Duration::from_millis(15_000)).await
    }

    pub async fn run_all_with_timeout(&self, company: CompanySnapshot, model_timeout: std::time::Duration) -> Vec<AgentRunResult> {
        let ctx = AgentContext { company, model_timeout };
        let governor = &self.governor;
        let model = self.model.clone();
        let concurrency = self.concurrency.clone();

        let futures = self.agents.iter().map(|agent| {
            let agent = agent.clone();
            let model = model.clone();
            let concurrency = concurrency.clone();
            let ctx = AgentContext { company: ctx.company.clone() };

            async move {
                let _permit = concurrency.acquire_owned().await.expect("agent semaphore closed");
                match agent.propose(&ctx, model).await {
                    Ok(proposal) => {
                        let governance = governor.evaluate(proposal.clone(), &ctx.company);
                        AgentRunResult { agent: agent.role(), proposal, governance: Some(governance) }
                    }
                    Err(error) => {
                        let proposal = crate::types::Proposal {
                            agent: agent.role(),
                            objective: "agent failure".into(),
                            action: crate::types::ActionKind::EscalateIncident,
                            cost_minor: 0,
                            expected_revenue_minor: 0,
                            risk: crate::types::RiskTier::Critical,
                            confidence_bps: 10_000,
                            evidence: vec![error.to_string()],
                            rationale: "agent failed safely and did not execute an action".into(),
                            reversible: true,
                            requested_permission: crate::types::Permission::Propose,
                        };
                        let governance = governor.evaluate(proposal.clone(), &ctx.company);
                        AgentRunResult { agent: agent.role(), proposal, governance: Some(governance) }
                    }
                }
            }
        });

        join_all(futures).await
    }
}
