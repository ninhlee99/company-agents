use agent_runtime::{
    types::{AgentRole, ActionKind, GovernorDecision, Permission, Proposal, RiskTier, AgentRunResult, CompanySnapshot},
};
use economic_core::CompanyStatus;
use super::CompanyStore;

#[tokio::test]
async fn postgres_round_trip_persists_snapshot_and_agent_runs() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return,
    };

    let store = CompanyStore::connect(&database_url).await.unwrap();
    store.migrate().await.unwrap();

    let company_id = uuid::Uuid::new_v4().to_string();
    store.ensure_company(&company_id, "Integration Test Company", "USD").await.unwrap();

    let snapshot = CompanySnapshot {
        company_id: company_id.clone(),
        cash_minor: 10_000,
        revenue_minor: 5_000,
        expenses_minor: 2_000,
        liabilities_minor: 500,
        assets_minor: 10_000,
        runway_days: 40,
        status: CompanyStatus::Warning,
        budget_remaining_minor: 1_000,
        experiment_budget_minor: 100,
        content_cost_minor: 20,
        content_revenue_minor: 50,
        backlog: 2,
        capacity: 3,
        conversion_bps: 200,
        audience_growth_bps: 50,
        hiring_need: 0,
    };

    let proposal = Proposal {
        agent: AgentRole::Analyst,
        objective: "verify".into(),
        action: ActionKind::ProduceReport,
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: RiskTier::Low,
        confidence_bps: 9000,
        evidence: vec!["integration-test".into()],
        rationale: "persist".into(),
        reversible: true,
        requested_permission: Permission::Propose,
    };

    let result = AgentRunResult {
        agent: AgentRole::Analyst,
        proposal,
        governance: Some(agent_runtime::types::GovernedProposal {
            proposal: Proposal {
                agent: AgentRole::Analyst,
                objective: "verify".into(),
                action: ActionKind::ProduceReport,
                cost_minor: 0,
                expected_revenue_minor: 0,
                risk: RiskTier::Low,
                confidence_bps: 9000,
                evidence: vec!["integration-test".into()],
                rationale: "persist".into(),
                reversible: true,
                requested_permission: Permission::Propose,
            },
            decision: GovernorDecision::Approve,
            reason: "test".into(),
        }),
    };

    store.persist_cycle(&snapshot, &[result]).await.unwrap();
    let loaded = store.load_snapshot(&company_id).await.unwrap().unwrap();

    assert_eq!(loaded.company_id, company_id);
    assert_eq!(loaded.cash_minor, 10_000);
    assert_eq!(loaded.revenue_minor, 5_000);
}

#[tokio::test]
async fn invalid_company_id_is_rejected() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return,
    };

    let store = CompanyStore::connect(&database_url).await.unwrap();
    assert!(store.ensure_company("not-a-uuid", "bad", "USD").await.is_err());
}
