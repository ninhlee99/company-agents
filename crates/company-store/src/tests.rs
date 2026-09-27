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


#[tokio::test]
async fn durable_cycle_is_idempotent_and_emits_outbox() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return,
    };

    let store = CompanyStore::connect(&database_url).await.unwrap();
    store.migrate().await.unwrap();

    let company_id = uuid::Uuid::new_v4().to_string();
    store.ensure_company(&company_id, "Durability Test Company", "USD").await.unwrap();

    let snapshot = CompanySnapshot {
        company_id: company_id.clone(),
        cash_minor: 10_000,
        revenue_minor: 8_000,
        expenses_minor: 2_000,
        liabilities_minor: 0,
        assets_minor: 10_000,
        runway_days: 90,
        status: CompanyStatus::Growth,
        budget_remaining_minor: 1_000,
        experiment_budget_minor: 200,
        content_cost_minor: 20,
        content_revenue_minor: 50,
        backlog: 1,
        capacity: 3,
        conversion_bps: 250,
        audience_growth_bps: 100,
        hiring_need: 0,
    };

    let result = Proposal {
        agent: AgentRole::Analyst,
        objective: "journal".into(),
        action: ActionKind::ProduceReport,
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: RiskTier::Low,
        confidence_bps: 9_000,
        evidence: vec!["durability".into()],
        rationale: "record decision".into(),
        reversible: true,
        requested_permission: Permission::Propose,
    };
    let run = AgentRunResult {
        agent: AgentRole::Analyst,
        proposal: result.clone(),
        governance: Some(agent_runtime::GovernedProposal {
            proposal: result,
            decision: GovernorDecision::Approve,
            reason: "test".into(),
        }),
    };
    let mut runs = vec![run];
    let mut working = snapshot.clone();
    let outcomes = agent_runtime::ExecutionEngine::default().execute_batch(&mut working, &mut runs);
    let cycle_id = uuid::Uuid::new_v4().to_string();

    assert_eq!(
        store.persist_decision_cycle(&working, &cycle_id, &runs, &outcomes).await.unwrap(),
        super::PersistCycleResult::Committed
    );
    assert_eq!(
        store.persist_decision_cycle(&working, &cycle_id, &runs, &outcomes).await.unwrap(),
        super::PersistCycleResult::AlreadyProcessed
    );

    let pending = store.pending_outbox(10).await.unwrap();
    assert!(pending.iter().any(|event| event.idempotency_key == format!("cycle:{cycle_id}")));
}


#[tokio::test]
async fn cash_revenue_and_expense_reconcile_to_snapshot_and_ledger() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return,
    };

    let store = CompanyStore::connect(&database_url).await.unwrap();
    store.migrate().await.unwrap();

    let company_id = uuid::Uuid::new_v4().to_string();
    store.ensure_company(&company_id, "Ledger Flow Test Company", "USD").await.unwrap();

    let snapshot = CompanySnapshot {
        company_id: company_id.clone(),
        cash_minor: 10_000,
        revenue_minor: 1_000,
        expenses_minor: 500,
        liabilities_minor: 0,
        assets_minor: 10_000,
        runway_days: 30,
        status: CompanyStatus::Active,
        budget_remaining_minor: 1_000,
        experiment_budget_minor: 100,
        content_cost_minor: 20,
        content_revenue_minor: 50,
        backlog: 1,
        capacity: 3,
        conversion_bps: 200,
        audience_growth_bps: 50,
        hiring_need: 0,
    };
    store.save_snapshot(&snapshot).await.unwrap();

    let revenue_tx = store.record_cash_revenue(&company_id, 250, "affiliate order", "order-1").await.unwrap();
    assert!(!revenue_tx.is_nil());

    let after_revenue = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(after_revenue.cash_minor, 10_250);
    assert_eq!(after_revenue.revenue_minor, 1_250);
    assert_eq!(after_revenue.assets_minor, 10_250);

    let revenue_retry = store.record_cash_revenue(&company_id, 250, "affiliate order", "order-1").await.unwrap();
    assert_eq!(revenue_retry, revenue_tx);

    let expense_tx = store.record_cash_expense(&company_id, 75, "content production", "expense-1").await.unwrap();
    assert!(!expense_tx.is_nil());

    let after_expense = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(after_expense.cash_minor, 10_175);
    assert_eq!(after_expense.revenue_minor, 1_250);
    assert_eq!(after_expense.expenses_minor, 575);
    assert_eq!(after_expense.assets_minor, 10_175);

    let cash_id = store.account_id_by_code(&company_id, "1000").await.unwrap().unwrap();
    let revenue_id = store.account_id_by_code(&company_id, "4000").await.unwrap().unwrap();
    let tx_id = store.post_ledger_transaction(
        &company_id,
        &agent_runtime_test_transaction(cash_id, revenue_id),
        "manual-test-ledger-1",
    ).await.unwrap();
    assert!(!tx_id.is_nil());
}
fn agent_runtime_test_transaction(cash_id: uuid::Uuid, revenue_id: uuid::Uuid) -> economic_core::LedgerTransaction {
    economic_core::LedgerTransaction {
        id: uuid::Uuid::new_v4().to_string(),
        description: "manual test".into(),
        entries: vec![
            economic_core::LedgerEntry {
                account_id: cash_id.to_string(),
                debit_minor: 1,
                credit_minor: 0,
                currency: "USD".into(),
            },
            economic_core::LedgerEntry {
                account_id: revenue_id.to_string(),
                debit_minor: 0,
                credit_minor: 1,
                currency: "USD".into(),
            },
        ],
    }
}
