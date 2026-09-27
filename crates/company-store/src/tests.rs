use super::CompanyStore;
use agent_runtime::types::{
    ActionKind, AgentRole, AgentRunResult, CompanySnapshot, GovernorDecision, Permission, Proposal,
    RiskTier,
};
use economic_core::{CompanyStatus, LedgerEntry, LedgerTransaction};

fn snapshot(company_id: &str) -> CompanySnapshot {
    CompanySnapshot {
        company_id: company_id.into(),
        cash_minor: 10_000,
        revenue_minor: 5_000,
        expenses_minor: 2_000,
        liabilities_minor: 500,
        assets_minor: 10_000,
        runway_days: 40,
        status: CompanyStatus::Growth,
        budget_remaining_minor: 1_000,
        experiment_budget_minor: 100,
        content_cost_minor: 20,
        content_revenue_minor: 50,
        backlog: 2,
        capacity: 3,
        conversion_bps: 200,
        audience_growth_bps: 50,
        hiring_need: 0,
    }
}

fn report_result() -> AgentRunResult {
    let proposal = Proposal {
        agent: AgentRole::Analyst,
        objective: "verify".into(),
        action: ActionKind::ProduceReport,
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: RiskTier::Low,
        confidence_bps: 9_000,
        evidence: vec!["integration-test".into()],
        rationale: "persist".into(),
        reversible: true,
        requested_permission: Permission::Propose,
    };
    AgentRunResult {
        agent: AgentRole::Analyst,
        proposal: proposal.clone(),
        governance: Some(agent_runtime::types::GovernedProposal {
            proposal,
            decision: GovernorDecision::Approve,
            reason: "test".into(),
        }),
    }
}

async fn connect_store() -> Option<CompanyStore> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    let store = CompanyStore::connect(&database_url).await.ok()?;
    store.migrate().await.ok()?;
    Some(store)
}

#[tokio::test]
async fn postgres_round_trip_is_idempotent_and_persists_authoritative_cycle() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Integration Test Company", "USD")
        .await
        .unwrap();

    let initial = snapshot(&company_id);
    let result = report_result();
    let first = store
        .persist_and_execute_cycle_with_id(&initial, std::slice::from_ref(&result), "stable-cycle")
        .await
        .unwrap();
    let second = store
        .persist_and_execute_cycle_with_id(&initial, &[result], "stable-cycle")
        .await
        .unwrap();

    assert_eq!(first, second);
    let loaded = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(loaded, first.snapshot);
}

#[tokio::test]
async fn approved_experiment_changes_persisted_company_state() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Execution Test", "USD")
        .await
        .unwrap();

    let mut initial = snapshot(&company_id);
    initial.status = CompanyStatus::Growth;

    let proposal = Proposal {
        agent: AgentRole::Experiment,
        objective: "bounded test".into(),
        action: ActionKind::CreateExperiment,
        cost_minor: 100,
        expected_revenue_minor: 200,
        risk: RiskTier::Medium,
        confidence_bps: 8_000,
        evidence: vec!["bounded".into()],
        rationale: "small experiment".into(),
        reversible: true,
        requested_permission: Permission::Propose,
    };
    let result = AgentRunResult {
        agent: AgentRole::Experiment,
        proposal: proposal.clone(),
        governance: Some(agent_runtime::types::GovernedProposal {
            proposal,
            decision: GovernorDecision::Approve,
            reason: "test".into(),
        }),
    };

    let persisted = store
        .persist_and_execute_cycle_with_id(&initial, &[result], "experiment-cycle")
        .await
        .unwrap();

    assert_eq!(persisted.snapshot.cash_minor, 9_900);
    assert_eq!(persisted.snapshot.experiment_budget_minor, 0);
    assert_eq!(
        store
            .load_snapshot(&company_id)
            .await
            .unwrap()
            .unwrap()
            .cash_minor,
        9_900
    );
}

#[tokio::test]
async fn cost_control_status_uses_canonical_database_value() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Status Test", "USD")
        .await
        .unwrap();

    let mut value = snapshot(&company_id);
    value.status = CompanyStatus::CostControl;
    store.save_snapshot(&value).await.unwrap();

    let client = store.client.lock().await;
    let row = client
        .query_one(
            "SELECT status FROM companies WHERE id = $1",
            &[&uuid::Uuid::parse_str(&company_id).unwrap()],
        )
        .await
        .unwrap();
    let status: String = row.get(0);
    assert_eq!(status, "COST_CONTROL");
}

#[tokio::test]
async fn ledger_transaction_is_idempotent_and_balanced() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4();
    store
        .ensure_company(&company_id.to_string(), "Ledger Test", "USD")
        .await
        .unwrap();

    let cash_account = uuid::Uuid::new_v4();
    let revenue_account = uuid::Uuid::new_v4();
    {
        let client = store.client.lock().await;
        client
            .execute(
                "INSERT INTO ledger_accounts
                 (id, company_id, code, name, account_type, currency)
                 VALUES ($1,$2,'CASH','Cash','ASSET','USD'),
                        ($3,$2,'REV','Revenue','REVENUE','USD')",
                &[&cash_account, &company_id, &revenue_account],
            )
            .await
            .unwrap();
    }

    let transaction = LedgerTransaction {
        id: uuid::Uuid::new_v4().to_string(),
        description: "affiliate revenue".into(),
        entries: vec![
            LedgerEntry {
                account_id: cash_account.to_string(),
                debit_minor: 500,
                credit_minor: 0,
                currency: "USD".into(),
            },
            LedgerEntry {
                account_id: revenue_account.to_string(),
                debit_minor: 0,
                credit_minor: 500,
                currency: "USD".into(),
            },
        ],
    };

    store
        .append_ledger_transaction(&company_id.to_string(), &transaction)
        .await
        .unwrap();
    store
        .append_ledger_transaction(&company_id.to_string(), &transaction)
        .await
        .unwrap();

    let client = store.client.lock().await;
    let count: i64 = client
        .query_one(
            "SELECT count(*) FROM ledger_transactions WHERE company_id = $1",
            &[&company_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 1);
}

#[tokio::test]
async fn durable_agent_memory_round_trips_and_rate_limit_is_enforced() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Memory Test", "USD")
        .await
        .unwrap();

    store
        .upsert_agent_memory(
            &company_id,
            AgentRole::Analyst,
            "lesson",
            &serde_json::json!({"finding":"quality evidence matters"}),
            9500,
            80,
        )
        .await
        .unwrap();

    let memory = store
        .load_agent_memory(&company_id, AgentRole::Analyst, 10)
        .await
        .unwrap();
    assert_eq!(memory.len(), 1);
    assert_eq!(memory[0].key, "lesson");

    let first = store
        .claim_agent_run_slots(&company_id, &[AgentRole::Analyst], 60, 1)
        .await
        .unwrap();
    let second = store
        .claim_agent_run_slots(&company_id, &[AgentRole::Analyst], 60, 1)
        .await
        .unwrap();
    assert_eq!(first, vec![AgentRole::Analyst]);
    assert!(second.is_empty());
}

#[tokio::test]
async fn invalid_company_id_is_rejected() {
    let Some(store) = connect_store().await else {
        return;
    };
    assert!(store
        .ensure_company("not-a-uuid", "bad", "USD")
        .await
        .is_err());
}

#[tokio::test]
async fn scheduler_lease_recovery_reuses_same_run_token() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Scheduler Recovery", "USD")
        .await
        .unwrap();
    store
        .ensure_recurring_job(&company_id, "agent_cycle", 15)
        .await
        .unwrap();

    let first = store
        .claim_due_job(&company_id, "agent_cycle")
        .await
        .unwrap()
        .expect("job should be claimable");
    store.release_job_after_failure(first.0).await.unwrap();

    let second = store
        .claim_due_job(&company_id, "agent_cycle")
        .await
        .unwrap()
        .expect("released job should be recoverable");

    assert_eq!(first.0, second.0);
    assert_eq!(first.1, second.1);
    store
        .complete_job(second.0, uuid::Uuid::new_v4())
        .await
        .unwrap();
}

#[tokio::test]
async fn durable_agent_memory_round_trips_and_is_bounded() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Memory Test", "USD")
        .await
        .unwrap();

    store
        .upsert_agent_memory(
            &company_id,
            AgentRole::Analyst,
            "unit_economics",
            &serde_json::json!({"signal":"positive","value":123}),
            9_500,
            80,
        )
        .await
        .unwrap();

    let memory = store
        .load_agent_memory(&company_id, AgentRole::Analyst, 20)
        .await
        .unwrap();
    assert_eq!(memory.len(), 1);
    assert_eq!(memory[0].key, "unit_economics");
    assert_eq!(memory[0].confidence_bps, 9_500);
    assert_eq!(memory[0].importance, 80);

    let oversized = serde_json::json!({"payload":"x".repeat(20_000)});
    assert!(store
        .upsert_agent_memory(
            &company_id,
            AgentRole::Analyst,
            "too_large",
            &oversized,
            5_000,
            10,
        )
        .await
        .is_err());
}

#[tokio::test]
async fn durable_agent_rate_limit_allows_once_then_blocks_until_window() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Rate Limit Test", "USD")
        .await
        .unwrap();

    let roles = [AgentRole::Analyst];
    let first = store
        .claim_agent_run_slots(&company_id, &roles, 300, 1)
        .await
        .unwrap();
    let second = store
        .claim_agent_run_slots(&company_id, &roles, 300, 1)
        .await
        .unwrap();

    assert_eq!(first, roles);
    assert!(second.is_empty());
}
