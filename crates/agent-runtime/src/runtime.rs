use crate::{
    agent::{Agent, AgentContext, AgentStateProvider},
    governor::Governor,
    model::Model,
    roles::executive_agents,
    types::{AgentMemory, AgentRole, AgentRunResult, CompanySnapshot},
};
use futures::future::join_all;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Semaphore;

pub struct AgentRuntime {
    agents: Vec<Arc<dyn Agent>>,
    governor: Governor,
    model: Arc<dyn Model>,
    concurrency: Arc<Semaphore>,
}

impl AgentRuntime {
    pub fn new(model: Box<dyn Model>) -> Self {
        let concurrency = std::env::var("AGENT_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .map(|v| v.clamp(1, 8))
            .unwrap_or(2);
        Self::new_with_concurrency(model, concurrency)
    }

    pub fn new_with_concurrency(model: Box<dyn Model>, max_concurrent: usize) -> Self {
        Self {
            agents: executive_agents(),
            governor: Governor,
            model: Arc::from(model),
            concurrency: Arc::new(Semaphore::new(max_concurrent.clamp(1, 8))),
        }
    }

    pub fn agent_roles(&self) -> Vec<AgentRole> {
        self.agents.iter().map(|a| a.role()).collect()
    }

    pub async fn run_all(&self, company: CompanySnapshot) -> Vec<AgentRunResult> {
        self.run_all_with_state(company, None).await
    }

    pub async fn run_all_with_state(
        &self,
        company: CompanySnapshot,
        state: Option<Arc<dyn AgentStateProvider>>,
    ) -> Vec<AgentRunResult> {
        let timeout_ms = std::env::var("MODEL_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| (250..=120_000).contains(v))
            .unwrap_or(15_000);

        self.run_all_with_memory_timeout(
            company,
            std::time::Duration::from_millis(timeout_ms),
            &HashMap::new(),
        )
        .await
    }

    pub async fn run_all_with_timeout(
        &self,
        company: CompanySnapshot,
        model_timeout: std::time::Duration,
    ) -> Vec<AgentRunResult> {
        self.run_all_with_memory_timeout(company, model_timeout, &HashMap::new())
            .await
    }

    pub async fn run_all_with_memory_timeout(
        &self,
        company: CompanySnapshot,
        model_timeout: std::time::Duration,
        memory: &HashMap<AgentRole, Vec<AgentMemory>>,
    ) -> Vec<AgentRunResult> {
        self.run_roles_with_memory(company, model_timeout, self.agent_roles(), memory)
            .await
    }

    pub async fn run_roles_with_memory(
        &self,
        company: CompanySnapshot,
        model_timeout: std::time::Duration,
        roles: Vec<AgentRole>,
        memory: &HashMap<AgentRole, Vec<AgentMemory>>,
    ) -> Vec<AgentRunResult> {
        let ctx = AgentContext {
            company,
            model_timeout,
            memory: Vec::new(),
        };
        let governor = &self.governor;
        let model = self.model.clone();
        let concurrency = self.concurrency.clone();

        let mut selected = self
            .agents
            .iter()
            .filter(|agent| roles.contains(&agent.role()))
            .collect::<Vec<_>>();
        selected.sort_by_key(|agent| AgentRole::ALL.iter().position(|role| *role == agent.role()));

        let futures = selected.into_iter().map(|agent| {
            let agent = agent.clone();
            let model = model.clone();
            let concurrency = concurrency.clone();
            let ctx = AgentContext {
                company: ctx.company.clone(),
                model_timeout: ctx.model_timeout,
                memory: memory.get(&agent.role()).cloned().unwrap_or_default(),
            };

            async move {
                let _permit = match concurrency.acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => {
                        let proposal = crate::types::Proposal {
                            agent: agent.role(),
                            objective: "agent runtime unavailable".into(),
                            action: crate::types::ActionKind::EscalateIncident,
                            cost_minor: 0,
                            expected_revenue_minor: 0,
                            risk: crate::types::RiskTier::Critical,
                            confidence_bps: 10_000,
                            evidence: vec!["agent semaphore is closed".into()],
                            rationale: "runtime failed closed".into(),
                            reversible: true,
                            requested_permission: crate::types::Permission::Propose,
                        };
                        let governance = governor.evaluate(proposal.clone(), &ctx.company);
                        return AgentRunResult {
                            agent: agent.role(),
                            proposal,
                            governance: Some(governance),
                        };
                    }
                };
                let memory = if let Some(state) = &state {
                    match state.admit_model_call(&ctx_company.company_id, agent.role()).await {
                        Ok(()) => state
                            .load_memory(&ctx_company.company_id, agent.role())
                            .await
                            .unwrap_or_else(|_| serde_json::json!({})),
                        Err(reason) => {
                            let proposal = crate::types::Proposal {
                                agent: agent.role(),
                                objective: "agent call admission denied".into(),
                                action: crate::types::ActionKind::EscalateIncident,
                                cost_minor: 0,
                                expected_revenue_minor: 0,
                                risk: crate::types::RiskTier::Critical,
                                confidence_bps: 10_000,
                                evidence: vec![reason],
                                rationale: "durable rate limit or state admission failed; execution halted".into(),
                                reversible: true,
                                requested_permission: crate::types::Permission::Propose,
                            };
                            let governance = governor.evaluate(proposal.clone(), &ctx_company);
                            return AgentRunResult {
                                agent: agent.role(),
                                proposal,
                                governance: Some(governance),
                            };
                        }
                    }
                } else {
                    serde_json::json!({})
                };
                let ctx = AgentContext {
                    company: ctx_company,
                    model_timeout: timeout,
                    memory,
                };

                match agent.propose(&ctx, model).await {
                    Ok(proposal) => {
                        let governance = governor.evaluate(proposal.clone(), &ctx.company);
                        AgentRunResult {
                            agent: agent.role(),
                            proposal,
                            governance: Some(governance),
                        }
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
                        AgentRunResult {
                            agent: agent.role(),
                            proposal,
                            governance: Some(governance),
                        }
                    }
                }
            }
        });

        join_all(futures).await
    }
}
