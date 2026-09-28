use crate::{
    agent::{Agent, AgentContext, AgentMemory, AgentRuntimeStore},
    governor::Governor,
    model::Model,
    roles::executive_agents,
    types::{AgentRole, AgentRunResult, CompanySnapshot},
};
use futures::future::join_all;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct AgentRuntime {
    agents: Vec<Arc<dyn Agent>>,
    governor: Governor,
    model: Arc<dyn Model>,
    concurrency: Arc<Semaphore>,
    store: Option<Arc<dyn AgentRuntimeStore>>,
}

impl AgentRuntime {
    pub fn new(model: Box<dyn Model>) -> Self {
        let concurrency = concurrency_from_env();
        Self::new_with_concurrency_and_store(model, concurrency, None)
    }

    pub fn new_with_store(model: Box<dyn Model>, store: Arc<dyn AgentRuntimeStore>) -> Self {
        let concurrency = concurrency_from_env();
        Self::new_with_concurrency_and_store(model, concurrency, Some(store))
    }

    pub fn new_with_concurrency(model: Box<dyn Model>, max_concurrent: usize) -> Self {
        Self::new_with_concurrency_and_store(model, max_concurrent, None)
    }

    pub fn new_with_concurrency_and_store(
        model: Box<dyn Model>,
        max_concurrent: usize,
        store: Option<Arc<dyn AgentRuntimeStore>>,
    ) -> Self {
        Self {
            agents: executive_agents(),
            governor: Governor,
            model: Arc::from(model),
            concurrency: Arc::new(Semaphore::new(max_concurrent.clamp(1, 8))),
            store,
        }
    }

    pub fn agent_roles(&self) -> Vec<AgentRole> {
        self.agents.iter().map(|a| a.role()).collect()
    }

    pub async fn run_all(&self, company: CompanySnapshot) -> Vec<AgentRunResult> {
        let timeout_ms = std::env::var("MODEL_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| (250..=120_000).contains(v))
            .unwrap_or(15_000);

        self.run_all_with_timeout(company, std::time::Duration::from_millis(timeout_ms))
            .await
    }

    pub async fn run_all_with_timeout(
        &self,
        company: CompanySnapshot,
        model_timeout: std::time::Duration,
    ) -> Vec<AgentRunResult> {
        let ctx = AgentContext {
            company,
            model_timeout,
            memory: Vec::new(),
        };
        let governor = &self.governor;
        let model = self.model.clone();
        let concurrency = self.concurrency.clone();
        let store = self.store.clone();
        let rate_window_seconds = std::env::var("AGENT_RATE_WINDOW_SECONDS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .map(|v| v.clamp(1, 86_400))
            .unwrap_or(300);
        let max_calls = std::env::var("AGENT_MAX_CALLS_PER_WINDOW")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .map(|v| v.clamp(1, 1_000))
            .unwrap_or(1);

        let futures = self.agents.iter().map(|agent| {
            let agent = agent.clone();
            let model = model.clone();
            let concurrency = concurrency.clone();
            let governor = governor;
            let store = store.clone();
            let ctx = AgentContext {
                company: ctx.company.clone(),
                model_timeout: ctx.model_timeout,
                memory: Vec::new(),
            };

            async move {
                let _permit = match concurrency.acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => {
                        return failed_result(
                            governor,
                            agent.role(),
                            "agent semaphore is closed".into(),
                        );
                    }
                };

                if let Some(store) = store.as_ref() {
                    match store
                        .try_acquire_agent_rate(
                            &ctx.company.company_id,
                            agent.role().as_str(),
                            rate_window_seconds,
                            max_calls,
                        )
                        .await
                    {
                        Ok(true) => {}
                        Ok(false) => {
                            return failed_result(
                                governor,
                                agent.role(),
                                format!(
                                    "persistent agent rate limit reached: {} call(s)/{}s",
                                    max_calls, rate_window_seconds
                                ),
                            );
                        }
                        Err(error) => {
                            return failed_result(
                                governor,
                                agent.role(),
                                format!("durable runtime state unavailable: {error}"),
                            );
                        }
                    }
                }

                let memory = match store.as_ref() {
                    Some(store) => match store
                        .load_agent_memory(&ctx.company.company_id, agent.role().as_str(), 8)
                        .await
                    {
                        Ok(memory) => memory,
                        Err(error) => {
                            return failed_result(
                                governor,
                                agent.role(),
                                format!("agent memory unavailable: {error}"),
                            );
                        }
                    },
                    None => Vec::new(),
                };

                let agent_ctx = AgentContext { memory, ..ctx };

                match agent.propose(&agent_ctx, model).await {
                    Ok(proposal) => {
                        let governance = governor.evaluate(proposal.clone(), &agent_ctx.company);
                        let result = AgentRunResult {
                            agent: agent.role(),
                            proposal,
                            governance: Some(governance),
                        };
                        if let Some(store) = store.as_ref() {
                            let memory = AgentMemory {
                                memory_key: "latest_governed_proposal".into(),
                                value: json!({
                                    "proposal": &result.proposal,
                                    "governance": &result.governance
                                }),
                                confidence_bps: result.proposal.confidence_bps,
                                importance: 80,
                            };
                            let _ = store
                                .save_agent_memory(
                                    &agent_ctx.company.company_id,
                                    agent.role().as_str(),
                                    memory,
                                )
                                .await;
                        }
                        result
                    }
                    Err(error) => failed_result(governor, agent.role(), error.to_string()),
                }
            }
        });

        join_all(futures).await
    }
}

fn concurrency_from_env() -> usize {
    std::env::var("AGENT_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .map(|v| v.clamp(1, 8))
        .unwrap_or(2)
}

fn failed_result(governor: &Governor, role: AgentRole, reason: String) -> AgentRunResult {
    let proposal = crate::types::Proposal {
        agent: role,
        objective: "agent runtime unavailable".into(),
        action: crate::types::ActionKind::EscalateIncident,
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: crate::types::RiskTier::Critical,
        confidence_bps: 10_000,
        evidence: vec![reason],
        rationale: "agent failed safely and did not execute an action".into(),
        reversible: true,
        requested_permission: crate::types::Permission::Propose,
    };
    let governance = governor.evaluate(proposal.clone(), &CompanySnapshot {
        company_id: "runtime-failure".into(),
        cash_minor: 0,
        revenue_minor: 0,
        expenses_minor: 0,
        liabilities_minor: 0,
        assets_minor: 0,
        runway_days: 0,
        status: economic_core::CompanyStatus::Emergency,
        budget_remaining_minor: 0,
        experiment_budget_minor: 0,
        content_cost_minor: 0,
        content_revenue_minor: 0,
        backlog: 0,
        capacity: 0,
        conversion_bps: 0,
        audience_growth_bps: 0,
        hiring_need: 0,
    });
    AgentRunResult {
        agent: role,
        proposal,
        governance: Some(governance),
    }
}
