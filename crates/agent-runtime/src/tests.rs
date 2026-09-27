use crate::{
    agent::{Agent, AgentContext},
    model::{Model, MockModel, ModelError},
    roles::{AnalystAgent, CeoAgent, CfoAgent, ContentAgent, CooAgent, ExperimentAgent, GrowthAgent, RecruiterAgent},
    runtime::AgentRuntime,
    types::{ActionKind, AgentRole, CompanySnapshot, GovernorDecision, Permission, Proposal, RiskTier},
};
use async_trait::async_trait;
use economic_core::CompanyStatus;
use std::sync::Arc;

fn healthy_company() -> CompanySnapshot {
    CompanySnapshot {
        company_id: "test-company".into(),
        cash_minor: 100_000,
        revenue_minor: 25_000,
        expenses_minor: 15_000,
        liabilities_minor: 0,
        assets_minor: 100_000,
        runway_days: 90,
        status: CompanyStatus::Active,
        budget_remaining_minor: 10_000,
        experiment_budget_minor: 1_000,
        content_cost_minor: 500,
        content_revenue_minor: 900,
        backlog: 8,
        capacity: 10,
        conversion_bps: 220,
        audience_growth_bps: 120,
        hiring_need: 0,
    }
}

#[tokio::test]
async fn every_operating_agent_has_a_valid_contract() {
    let ctx = AgentContext { company: healthy_company() };
    let model: Arc<dyn Model> = Arc::new(MockModel);
    let agents: Vec<Arc<dyn Agent>> = vec![
        Arc::new(CeoAgent), Arc::new(CfoAgent), Arc::new(CooAgent), Arc::new(GrowthAgent),
        Arc::new(ContentAgent), Arc::new(RecruiterAgent), Arc::new(AnalystAgent), Arc::new(ExperimentAgent),
    ];

    let roles: Vec<AgentRole> = agents.iter().map(|a| a.role()).collect();
    assert_eq!(
        roles,
        vec![
            AgentRole::CEO, AgentRole::CFO, AgentRole::COO, AgentRole::Growth,
            AgentRole::Content, AgentRole::Recruiter, AgentRole::Analyst, AgentRole::Experiment
        ]
    );

    for agent in agents {
        let proposal = agent.propose(&ctx, model.clone()).await.expect("agent must fail safely only on model error");
        assert_eq!(proposal.agent, agent.role());
        assert!(!proposal.objective.trim().is_empty());
        assert!(!proposal.rationale.trim().is_empty());
        assert!(proposal.confidence_bps <= 10_000);
        assert!(proposal.cost_minor >= 0);
        assert!(proposal.expected_revenue_minor >= 0);
        assert_eq!(proposal.requested_permission, Permission::Propose);
    }
}

#[tokio::test]
async fn agents_fail_closed_when_model_is_unavailable() {
    struct FailingModel;
    #[async_trait]
    impl Model for FailingModel {
        async fn propose_json(&self, _: &str, _: &str) -> Result<serde_json::Value, ModelError> {
            Err(ModelError::Transport("simulated outage".into()))
        }
    }

    let runtime = AgentRuntime::new(Box::new(FailingModel));
    let results = runtime.run_all(healthy_company()).await;
    assert_eq!(results.len(), 8);
    for result in results {
        assert_eq!(result.proposal.action, ActionKind::EscalateIncident);
        let governed = result.governance.expect("governance result");
        assert_eq!(governed.decision, GovernorDecision::Escalate);
    }
}

#[tokio::test]
async fn bankrupt_company_cannot_trigger_discretionary_spend() {
    let runtime = AgentRuntime::new(Box::new(MockModel));
    let company = CompanySnapshot {
        status: CompanyStatus::Bankrupt,
        cash_minor: 0,
        budget_remaining_minor: 0,
        experiment_budget_minor: 0,
        ..healthy_company()
    };

    for result in runtime.run_all(company).await {
        let governed = result.governance.expect("governance result");
        if result.proposal.action != ActionKind::ProduceReport && result.proposal.action != ActionKind::EscalateIncident {
            assert_eq!(governed.decision, GovernorDecision::Reject);
        }
    }
}

#[tokio::test]
async fn stress_256_cycles_remain_bounded_and_deterministic() {
    let runtime = AgentRuntime::new(Box::new(MockModel));
    let company = healthy_company();

    for _ in 0..256 {
        let results = runtime.run_all(company.clone()).await;
        assert_eq!(results.len(), 8);
        for result in results {
            assert!(matches!(
                result.governance.unwrap().decision,
                GovernorDecision::Approve | GovernorDecision::Escalate | GovernorDecision::Reject | GovernorDecision::RequestRevision
            ));
        }
    }
}

#[tokio::test]
async fn negative_or_self_granted_permissions_never_execute() {
    let runtime = AgentRuntime::new(Box::new(MockModel));
    let company = healthy_company();
    let mut results = runtime.run_all(company).await;
    assert_eq!(results.len(), 8);

    let malicious = Proposal {
        agent: AgentRole::Growth,
        objective: "malicious".into(),
        action: ActionKind::AllocateExperimentBudget,
        cost_minor: 1,
        expected_revenue_minor: 100,
        risk: RiskTier::Low,
        confidence_bps: 10_000,
        evidence: vec!["untrusted".into()],
        rationale: "attempt permission escalation".into(),
        reversible: true,
        requested_permission: Permission::ExecuteMaterial,
    };

    let governor = crate::governor::Governor;
    let decision = governor.evaluate(malicious, &healthy_company());
    assert_eq!(decision.decision, GovernorDecision::Reject);

    for result in results.iter_mut() {
        assert!(!matches!(
            result.proposal.action,
            ActionKind::EscalateIncident if result.proposal.cost_minor < 0
        ));
    }
}
