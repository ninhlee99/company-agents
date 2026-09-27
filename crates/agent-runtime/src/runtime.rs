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
            .and_then(|value| value.parse::<usize>().ok())
            .map(|value| value.clamp(1, 8))
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
        self.agents.iter().map(|agent| agent.role()).collect()
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
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| (250..=120_000).contains(value))
            .unwrap_or(15_000);

        self.run_roles_with_state(
            company,
            std::time::Duration::from_millis(timeout_ms),
            self.agent_roles(),
            state,
        )
        .await
    }

    pub async fn run_all_with_timeout(
        &self,
        company: CompanySnapshot,
        model_timeout: std::time::Duration,
    ) -> Vec<AgentRunResult> {
        self.run_roles_with_state(company, model_timeout, self.agent_roles(), None)
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
        self.run_roles_internal(company, model_timeout, roles, memory, None)
            .await
    }

    async fn run_roles_with_state(
        &self,
        company: CompanySnapshot,
        model_timeout: std::time::Duration,
        roles: Vec<AgentRole>,
        state: Option<Arc<dyn AgentStateProvider>>,
    ) -> Vec<AgentRunResult> {
        if state.is_none() {
            return self
                .run_roles_with_memory(company, model_timeout, roles, &HashMap::new())
                .await;
        }

        self.run_roles_internal(company, model_timeout, roles, &HashMap::new(), state)
            .await
    }

    async fn run_roles_internal(
        &self,
        company: CompanySnapshot,
        model_timeout: std::time::Duration,
        roles: Vec<AgentRole>,
        memory: &HashMap<AgentRole, Vec<AgentMemory>>,
        state: Option<Arc<dyn AgentStateProvider>>,
    ) -> Vec<AgentRunResult> {
        if let Err(reason) = company.validate() {
            return roles
                .into_iter()
                .filter(|role| *role != AgentRole::Governor)
                .map(|role| fail_closed(role, &company, &self.governor, &reason))
                .collect();
        }

        let governor = &self.governor;
        let model = self.model.clone();
        let concurrency = self.concurrency.clone();

        let mut selected = self
            .agents
            .iter()
            .filter(|agent| roles.contains(&agent.role()))
            .cloned()
            .collect::<Vec<_>>();
        selected.sort_by_key(|agent| {
            AgentRole::ALL
                .iter()
                .position(|role| *role == agent.role())
                .unwrap_or(usize::MAX)
        });

        let futures = selected.into_iter().map(|agent| {
            let model = model.clone();
            let concurrency = concurrency.clone();
            let state = state.clone();
            let company = company.clone();
            let memory = memory.get(&agent.role()).cloned().unwrap_or_default();

            async move {
                let _permit = match concurrency.acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => {
                        return fail_closed(
                            agent.role(),
                            &company,
                            governor,
                            "agent semaphore is closed",
                        );
                    }
                };

                let effective_memory = match state.as_ref() {
                    Some(provider) => {
                        if let Err(reason) = provider
                            .admit_model_call(&company.company_id, agent.role())
                            .await
                        {
                            return fail_closed(agent.role(), &company, governor, &reason);
                        }
                        match provider
                            .load_memory(&company.company_id, agent.role())
                            .await
                        {
                            Ok(value) => memory_from_provider_value(value),
                            Err(reason) => {
                                return fail_closed(
                                    agent.role(),
                                    &company,
                                    governor,
                                    &format!("agent memory load failed: {reason}"),
                                );
                            }
                        }
                    }
                    None => memory,
                };

                let ctx = AgentContext {
                    company,
                    model_timeout,
                    memory: effective_memory,
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
                        fail_closed(agent.role(), &ctx.company, governor, &error.to_string())
                    }
                }
            }
        });

        join_all(futures).await
    }
}

fn memory_from_provider_value(value: serde_json::Value) -> Vec<AgentMemory> {
    value
        .get("items")
        .and_then(|items| items.as_array())
        .and_then(|items| serde_json::from_value(serde_json::Value::Array(items.clone())).ok())
        .unwrap_or_default()
}

fn fail_closed(
    role: AgentRole,
    company: &CompanySnapshot,
    governor: &Governor,
    reason: &str,
) -> AgentRunResult {
    let proposal = crate::types::Proposal {
        agent: role,
        objective: "agent execution blocked; escalate".into(),
        action: crate::types::ActionKind::EscalateIncident,
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: crate::types::RiskTier::Critical,
        confidence_bps: 10_000,
        evidence: vec![reason.chars().take(4_000).collect()],
        rationale: "runtime failed closed without performing an economic side effect".into(),
        reversible: true,
        requested_permission: crate::types::Permission::Propose,
    };
    let governance = governor.evaluate(proposal.clone(), company);
    AgentRunResult {
        agent: role,
        proposal,
        governance: Some(governance),
    }
}
