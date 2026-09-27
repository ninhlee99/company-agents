use crate::{
    agent::{Agent, AgentContext},
    governor::Governor,
    model::Model,
    roles::executive_agents,
    types::{AgentRole, AgentRunResult, CompanySnapshot},
};
use std::sync::Arc;

pub struct AgentRuntime {
    agents: Vec<Arc<dyn Agent>>,
    governor: Governor,
    model: Arc<dyn Model>,
}

impl AgentRuntime {
    pub fn new(model: Box<dyn Model>) -> Self {
        Self {
            agents: executive_agents(),
            governor: Governor,
            model: Arc::from(model),
        }
    }

    pub fn agent_roles(&self) -> Vec<AgentRole> {
        self.agents.iter().map(|a| a.role()).collect()
    }

    pub async fn run_all(&self, company: CompanySnapshot) -> Vec<AgentRunResult> {
        let ctx = AgentContext { company };
        let mut results = Vec::with_capacity(self.agents.len());

        for agent in &self.agents {
            match agent.propose(&ctx, self.model.clone()).await {
                Ok(proposal) => {
                    let governance = self.governor.evaluate(proposal.clone(), &ctx.company);
                    results.push(AgentRunResult {
                        agent: agent.role(),
                        proposal,
                        governance: Some(governance),
                    });
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
                    let governance = self.governor.evaluate(proposal.clone(), &ctx.company);
                    results.push(AgentRunResult { agent: agent.role(), proposal, governance: Some(governance) });
                }
            }
        }

        results
    }
}
