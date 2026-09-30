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
async fn cycle_persists_last_decision_memory_atomically() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Memory Test", "USD")
        .await
        .unwrap();

    let initial = snapshot(&company_id);
    let result = report_result();

    store
        .persist_and_execute_cycle_with_id(&initial, &[result], "memory-cycle")
        .await
        .unwrap();

    let memory = store.load_agent_memory(
        &company_id,
        AgentRole::Analyst,
        20,
    ).await.unwrap();

    assert_eq!(memory.len(), 1);
    assert_eq!(memory[0].key, "last_decision");
    assert_eq!(memory[0].value["action"], "ProduceReport");
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
async fn affiliate_revenue_requires_provider_approval_and_reverses_on_rejection() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Affiliate Verification Test", "USD")
        .await
        .unwrap();

    let mut initial = snapshot(&company_id);
    initial.revenue_minor = 5_000;
    initial.assets_minor = 10_000;
    store.save_snapshot(&initial).await.unwrap();

    store
        .record_affiliate_click(&affiliate_attribution::ClickEvent {
            click_id: "click-verified".into(),
            company_id: company_id.clone(),
            product_id: "product-1".into(),
            advertiser_id: "advertiser-1".into(),
            content_id: "video-1".into(),
            occurred_at: "2026-09-28T09:00:00Z".into(),
            source: "affiliate-network".into(),
        })
        .await
        .unwrap();

    let conversion = affiliate_attribution::ConversionEvent {
        company_id: company_id.clone(),
        conversion_id: "conversion-verified".into(),
        click_id: Some("click-verified".into()),
        order_id: "order-verified".into(),
        product_id: "product-1".into(),
        advertiser_id: "advertiser-1".into(),
        occurred_at: "2026-09-28T09:05:00Z".into(),
        order_value_minor: 2_000,
        commission_minor: 100,
        refunded_minor: 0,
        cancelled: false,
        source: "affiliate-network".into(),
    };

    let reconciled = store
        .record_affiliate_conversion(
            &conversion,
            affiliate_attribution::AttributionModel::LastClick,
        )
        .await
        .unwrap();
    assert_eq!(
        reconciled.status,
        affiliate_attribution::ReconciliationStatus::Verified
    );
    assert_eq!(
        store
            .load_snapshot(&company_id)
            .await
            .unwrap()
            .unwrap()
            .revenue_minor,
        5_000
    );

    let approved = store
        .verify_affiliate_conversion(
            &company_id,
            "conversion-verified",
            affiliate_attribution::ProviderVerificationStatus::Approved,
            100,
            Some("2026-09-28T10:00:00Z"),
            "network-report-api",
        )
        .await
        .unwrap();
    assert_eq!(approved.net_commission_minor, 100);
    assert_eq!(
        store
            .load_snapshot(&company_id)
            .await
            .unwrap()
            .unwrap()
            .revenue_minor,
        5_100
    );

    let client = store.client.lock().await;
    let status: String = client
        .query_one(
            "SELECT status FROM affiliate_revenue_recognition
              WHERE company_id=$1 AND conversion_id=$2",
            &[&uuid::Uuid::parse_str(&company_id).unwrap(), &"conversion-verified"],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(status, "RECOGNIZED");
    drop(client);

    store
        .verify_affiliate_conversion(
            &company_id,
            "conversion-verified",
            affiliate_attribution::ProviderVerificationStatus::Rejected,
            0,
            Some("2026-09-28T11:00:00Z"),
            "network-rejection-feed",
        )
        .await
        .unwrap();

    assert_eq!(
        store
            .load_snapshot(&company_id)
            .await
            .unwrap()
            .unwrap()
            .revenue_minor,
        5_000
    );
}

#[tokio::test]
async fn publish_intent_requires_approval_and_one_time_execution_lease() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Publish Gate Test", "USD")
        .await
        .unwrap();

    let intent_id = uuid::Uuid::new_v4();
    let intent = publishing_contract::PublishIntent {
        id: intent_id.to_string(),
        company_id: company_id.clone(),
        content_id: "video-gate".into(),
        platform: publishing_contract::PublishPlatform::TikTok,
        media_uri: "s3://bucket/video.mp4".into(),
        title: "gated".into(),
        caption: "gated caption".into(),
        scheduled_at: None,
        content_hash: "b".repeat(64),
        created_by: agent_runtime::types::AgentRole::Content,
        idempotency_key: "video-gate:tiktok:v1".into(),
    };

    store.create_publish_intent(&intent).await.unwrap();

    let approval = store
        .approve_publish_intent(&company_id, &intent.id, "human-operator", 300)
        .await
        .unwrap();

    let claimed = store
        .claim_publish_intent(
            &company_id,
            &intent.id,
            &approval.approval_token,
            60,
        )
        .await
        .unwrap()
        .expect("approved intent should be claimable");

    assert_eq!(claimed.intent.content_hash, intent.content_hash);
    assert_ne!(claimed.execution_token, approval.approval_token);

    let second_claim = store
        .claim_publish_intent(
            &company_id,
            &intent.id,
            &approval.approval_token,
            60,
        )
        .await;
    assert!(second_claim.is_ok());
    assert!(second_claim.unwrap().is_none());

    store
        .complete_publish_intent(
            &company_id,
            &intent.id,
            &claimed.execution_token,
            true,
            Some("tiktok:post:123"),
            None,
        )
        .await
        .unwrap();

    let invalid_reuse = store
        .complete_publish_intent(
            &company_id,
            &intent.id,
            &claimed.execution_token,
            true,
            Some("tiktok:post:456"),
            None,
        )
        .await;
    assert!(invalid_reuse.is_err());

    let client = store.client.lock().await;
    let status: String = client
        .query_one(
            "SELECT status FROM publish_intents WHERE company_id=$1 AND id=$2",
            &[&uuid::Uuid::parse_str(&company_id).unwrap(), &intent_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(status, "SUCCEEDED");
}

#[tokio::test]
async fn scheduler_lease_expiry_reuses_same_run_token_for_replay() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Scheduler Recovery Test", "USD")
        .await
        .unwrap();

    store
        .ensure_recurring_job(&company_id, "agent_cycle", 60)
        .await
        .unwrap();

    let first = store
        .claim_due_job(&company_id, "agent_cycle")
        .await
        .unwrap()
        .expect("job should be claimable");

    let job_id = first.0;
    let first_token = first.1;

    {
        let client = store.client.lock().await;
        client
            .execute(
                "UPDATE scheduled_jobs
                    SET locked_until=now()-interval '1 second'
                  WHERE id=$1",
                &[&job_id],
            )
            .await
            .unwrap();
    }

    let second = store
        .claim_due_job(&company_id, "agent_cycle")
        .await
        .unwrap()
        .expect("expired lease should be reclaimable");

    assert_eq!(second.0, job_id);
    assert_eq!(second.1, first_token);

    store
        .complete_job(job_id, uuid::Uuid::new_v4())
        .await
        .unwrap();
}

#[tokio::test]
async fn affiliate_provider_verification_refuses_positive_unapproved_commission() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Affiliate Safety Test", "USD")
        .await
        .unwrap();

    let mut initial = snapshot(&company_id);
    initial.revenue_minor = 1_000;
    store.save_snapshot(&initial).await.unwrap();

    store
        .record_affiliate_click(&affiliate_attribution::ClickEvent {
            click_id: "click-safe".into(),
            company_id: company_id.clone(),
            product_id: "product-safe".into(),
            advertiser_id: "advertiser-safe".into(),
            content_id: "content-safe".into(),
            occurred_at: "2026-09-28T09:00:00Z".into(),
            source: "network".into(),
        })
        .await
        .unwrap();

    store
        .record_affiliate_conversion(
            &affiliate_attribution::ConversionEvent {
                company_id: company_id.clone(),
                conversion_id: "conversion-safe".into(),
                click_id: Some("click-safe".into()),
                order_id: "order-safe".into(),
                product_id: "product-safe".into(),
                advertiser_id: "advertiser-safe".into(),
                occurred_at: "2026-09-28T09:01:00Z".into(),
                order_value_minor: 1_000,
                commission_minor: 50,
                refunded_minor: 0,
                cancelled: false,
                source: "network".into(),
            },
            affiliate_attribution::AttributionModel::LastClick,
        )
        .await
        .unwrap();

    let error = store
        .verify_affiliate_conversion(
            &company_id,
            "conversion-safe",
            affiliate_attribution::ProviderVerificationStatus::Reported,
            50,
            Some("2026-09-28T09:02:00Z"),
            "network-report",
        )
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("non-approved provider verification"));
    assert_eq!(
        store
            .load_snapshot(&company_id)
            .await
            .unwrap()
            .unwrap()
            .revenue_minor,
        1_000
    );
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
async fn affiliate_partial_reconciliation_replays_exact_status_and_variance() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Affiliate Replay Test", "USD")
        .await
        .unwrap();

    let event = affiliate_attribution::ConversionEvent {
        company_id: company_id.clone(),
        conversion_id: "conversion-partial".into(),
        click_id: None,
        order_id: "order-partial".into(),
        product_id: "product-no-click".into(),
        advertiser_id: "advertiser".into(),
        occurred_at: "2026-09-27T10:00:00Z".into(),
        order_value_minor: 1_000,
        commission_minor: 100,
        refunded_minor: 0,
        cancelled: false,
        source: "integration-test".into(),
    };

    let first = store
        .record_affiliate_conversion(&event, affiliate_attribution::AttributionModel::LastClick)
        .await
        .unwrap();
    let second = store
        .record_affiliate_conversion(&event, affiliate_attribution::AttributionModel::LastClick)
        .await
        .unwrap();

    assert_eq!(first.status, affiliate_attribution::ReconciliationStatus::Partial);
    assert_eq!(first.reconciliation_variance_minor, 100);
    assert_eq!(second.status, first.status);
    assert_eq!(second.reconciliation_variance_minor, first.reconciliation_variance_minor);
}

#[tokio::test]
async fn payroll_accrual_and_payment_are_ledger_consistent() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Payroll Test", "USD")
        .await
        .unwrap();

    let initial = snapshot(&company_id);
    store.save_snapshot(&initial).await.unwrap();

    let employee = company_organization::Employee {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Operator".into(),
        role: "content".into(),
        monthly_cost_minor: 1_000,
        currency: "USD".into(),
        status: company_organization::EmployeeStatus::Active,
    };
    store.upsert_employee(&company_id, &employee).await.unwrap();

    let accrued = store
        .accrue_payroll(&company_id, "2026-09", "2026-09-30T00:00:00Z")
        .await
        .unwrap();
    assert_eq!(accrued, 1_000);

    let after_accrual = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(after_accrual.expenses_minor, 3_000);
    assert_eq!(after_accrual.liabilities_minor, 1_500);

    let due = store.payroll_due(&company_id, 10).await.unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].gross_minor, 1_000);

    let paid = store
        .pay_payroll(&company_id, &due[0].id, 600)
        .await
        .unwrap();
    assert_eq!(paid, 600);
    let after_partial = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(after_partial.cash_minor, 9_400);
    assert_eq!(after_partial.liabilities_minor, 900);

    let paid_remainder = store
        .pay_payroll(&company_id, &due[0].id, 400)
        .await
        .unwrap();
    assert_eq!(paid_remainder, 400);
    let after_full = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(after_full.cash_minor, 9_000);
    assert_eq!(after_full.liabilities_minor, 500);

    let paid_again = store
        .pay_payroll(&company_id, &due[0].id, 400)
        .await
        .unwrap();
    assert_eq!(paid_again, 0);
}

#[tokio::test]
async fn business_unit_upsert_and_portfolio_metrics_round_trip() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Business Unit Test", "USD")
        .await
        .unwrap();

    let first = company_organization::BusinessUnit {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Owned Media".into(),
        currency: "USD".into(),
        cash_minor: 5_000,
        revenue_minor: 2_000,
        variable_cost_minor: 500,
        fixed_cost_minor: 250,
        budget_minor: 1_000,
        lifecycle: company_organization::BusinessUnitLifecycle::Growing,
    };
    let second = company_organization::BusinessUnit {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Affiliate Commerce".into(),
        currency: "USD".into(),
        cash_minor: 4_000,
        revenue_minor: 1_500,
        variable_cost_minor: 300,
        fixed_cost_minor: 200,
        budget_minor: 700,
        lifecycle: company_organization::BusinessUnitLifecycle::Testing,
    };

    store.upsert_business_unit(&company_id, &first).await.unwrap();
    store.upsert_business_unit(&company_id, &second).await.unwrap();

    let units = store.list_business_units(&company_id).await.unwrap();
    assert_eq!(units.len(), 2);

    let metrics = store.portfolio_metrics(&company_id).await.unwrap();
    assert_eq!(metrics.revenue_minor, 3_500);
    assert_eq!(metrics.variable_cost_minor, 800);
    assert_eq!(metrics.fixed_cost_minor, 450);
    assert_eq!(metrics.total_cost_minor, 1_250);
}
    
#[tokio::test]
async fn verified_affiliate_conversion_becomes_receivable_then_cash() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Affiliate Accounting Test", "USD")
        .await
        .unwrap();
    store.save_snapshot(&snapshot(&company_id)).await.unwrap();

    store
        .record_affiliate_click(&affiliate_attribution::ClickEvent {
            click_id: "click-1".into(),
            company_id: company_id.clone(),
            product_id: "product-1".into(),
            advertiser_id: "advertiser-1".into(),
            content_id: "content-1".into(),
            occurred_at: "2026-09-27T09:00:00Z".into(),
            source: "test".into(),
        })
        .await
        .unwrap();

    let conversion = affiliate_attribution::ConversionEvent {
        company_id: company_id.clone(),
        conversion_id: "conversion-1".into(),
        click_id: Some("click-1".into()),
        order_id: "order-1".into(),
        product_id: "product-1".into(),
        advertiser_id: "advertiser-1".into(),
        occurred_at: "2026-09-27T10:00:00Z".into(),
        order_value_minor: 2_000,
        commission_minor: 101,
        refunded_minor: 0,
        cancelled: false,
        source: "test".into(),
    };

    let recognized = store
        .record_affiliate_conversion(
            &conversion,
            affiliate_attribution::AttributionModel::LastClick,
        )
        .await
        .unwrap();
    assert_eq!(recognized.status, affiliate_attribution::ReconciliationStatus::Verified);
    assert_eq!(recognized.net_commission_minor, 101);

    let after_recognition = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(after_recognition.cash_minor, 10_000);
    assert_eq!(after_recognition.revenue_minor, 5_101);
    assert_eq!(after_recognition.assets_minor, 10_101);

    store
        .record_affiliate_payout(
            &company_id,
            "payout-1",
            60,
            "USD",
            "2026-09-28T00:00:00Z",
        )
        .await
        .unwrap();
    store
        .record_affiliate_payout(
            &company_id,
            "payout-1",
            60,
            "USD",
            "2026-09-28T00:00:00Z",
        )
        .await
        .unwrap();
    store
        .record_affiliate_payout(
            &company_id,
            "payout-2",
            41,
            "USD",
            "2026-09-29T00:00:00Z",
        )
        .await
        .unwrap();

    let after_payout = store.load_snapshot(&company_id).await.unwrap().unwrap();
    assert_eq!(after_payout.cash_minor, 10_101);
    assert_eq!(after_payout.revenue_minor, 5_101);
    assert_eq!(after_payout.assets_minor, 10_101);

    assert!(store
        .record_affiliate_payout(
            &company_id,
            "payout-3",
            1,
            "USD",
            "2026-09-30T00:00:00Z",
        )
        .await
        .is_err());
}

#[tokio::test]
async fn cross_company_employee_and_business_unit_ids_cannot_overwrite() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_a = uuid::Uuid::new_v4().to_string();
    let company_b = uuid::Uuid::new_v4().to_string();
    store.ensure_company(&company_a, "A", "USD").await.unwrap();
    store.ensure_company(&company_b, "B", "USD").await.unwrap();

    let employee_id = uuid::Uuid::new_v4().to_string();
    let employee_a = company_organization::Employee {
        id: employee_id.clone(),
        name: "A Employee".into(),
        role: "editor".into(),
        monthly_cost_minor: 100,
        currency: "USD".into(),
        status: company_organization::EmployeeStatus::Active,
    };
    let employee_b = company_organization::Employee {
        id: employee_id,
        name: "B Employee".into(),
        role: "editor".into(),
        monthly_cost_minor: 200,
        currency: "USD".into(),
        status: company_organization::EmployeeStatus::Active,
    };

    store.upsert_employee(&company_a, &employee_a).await.unwrap();
    assert!(store.upsert_employee(&company_b, &employee_b).await.is_err());

    let unit_id = uuid::Uuid::new_v4().to_string();
    let unit_a = company_organization::BusinessUnit {
        id: unit_id.clone(),
        name: "A Unit".into(),
        currency: "USD".into(),
        cash_minor: 1_000,
        revenue_minor: 100,
        variable_cost_minor: 20,
        fixed_cost_minor: 10,
        budget_minor: 50,
        lifecycle: company_organization::BusinessUnitLifecycle::Testing,
    };
    let unit_b = company_organization::BusinessUnit {
        id: unit_id,
        name: "B Unit".into(),
        currency: "USD".into(),
        cash_minor: 2_000,
        revenue_minor: 200,
        variable_cost_minor: 30,
        fixed_cost_minor: 20,
        budget_minor: 60,
        lifecycle: company_organization::BusinessUnitLifecycle::Testing,
    };

    store.upsert_business_unit(&company_a, &unit_a).await.unwrap();
    assert!(store.upsert_business_unit(&company_b, &unit_b).await.is_err());
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

#[tokio::test]
async fn agent_memory_round_trip_is_bounded_and_persistent() {
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
            &serde_json::json!({"finding":"coupon quality matters"}),
            9_000,
            80,
        )
        .await
        .unwrap();

    let memory = store
        .load_agent_memory(&company_id, AgentRole::Analyst, 20)
        .await
        .unwrap();
    assert_eq!(memory.len(), 1);
    assert_eq!(memory[0].key, "lesson");
    assert_eq!(memory[0].confidence_bps, 9_000);
    assert_eq!(memory[0].importance, 80);
}

#[tokio::test]
async fn durable_agent_rate_limit_blocks_second_call_in_same_window() {
    let Some(store) = connect_store().await else {
        return;
    };

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Rate Test", "USD")
        .await
        .unwrap();

    let first = store
        .claim_agent_run_slots(&company_id, &[AgentRole::Analyst], 3_600, 1)
        .await
        .unwrap();
    let second = store
        .claim_agent_run_slots(&company_id, &[AgentRole::Analyst], 3_600, 1)
        .await
        .unwrap();

    assert_eq!(first, vec![AgentRole::Analyst]);
    assert!(second.is_empty());
}

#[tokio::test]
async fn scheduler_lease_can_be_recovered_by_another_store() {
    let Some(store) = connect_store().await else {
        return;
    };
    let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
        return;
    };
    let store_two = CompanyStore::connect(&database_url).await.unwrap();
    store_two.migrate().await.unwrap();

    let company_id = uuid::Uuid::new_v4().to_string();
    store
        .ensure_company(&company_id, "Scheduler Test", "USD")
        .await
        .unwrap();
    store
        .ensure_recurring_job(&company_id, "recovery-test", 15)
        .await
        .unwrap();

    let first = store
        .claim_due_job(&company_id, "recovery-test")
        .await
        .unwrap();
    assert!(first.is_some());

    let second = store_two
        .claim_due_job(&company_id, "recovery-test")
        .await
        .unwrap();
    assert!(second.is_none());

    {
        let client = store_two.client.lock().await;
        client
            .execute(
                "UPDATE scheduled_jobs
                    SET locked_until = now() - interval '1 second',
                        next_run_at = now() - interval '1 second'
                  WHERE company_id = $1 AND job_type = 'recovery-test'",
                &[&uuid::Uuid::parse_str(&company_id).unwrap()],
            )
            .await
            .unwrap();
    }

    let recovered = store_two
        .claim_due_job(&company_id, "recovery-test")
        .await
        .unwrap();
    assert!(recovered.is_some());
}

#[tokio::test]
async fn commercial_customer_links_are_company_scoped() {
    let Some(store) = connect_store().await else { return; };

    let company_a = uuid::Uuid::new_v4().to_string();
    let company_b = uuid::Uuid::new_v4().to_string();
    store.ensure_company(&company_a, "Isolation A", "USD").await.unwrap();
    store.ensure_company(&company_b, "Isolation B", "USD").await.unwrap();

    let customer_id = uuid::Uuid::new_v4();
    store.create_customer(
        &company_a, customer_id, "Customer A", Some("customer-a@example.test"),
        None, "LEAD", None, "isolation-customer-a",
    ).await.unwrap();

    let company_b_uuid = uuid::Uuid::parse_str(&company_b).unwrap();
    let proposal = commercial_sales::ServiceProposal {
        id: uuid::Uuid::new_v4(), company_id: company_b_uuid, customer_id,
        title: "Cross-company proposal".into(), currency: "USD".into(), total_minor: 10_000,
        status: commercial_sales::ProposalStatus::Draft, valid_until_epoch: 1_800_000_000,
        idempotency_key: "isolation-proposal".into(),
    };
    assert!(store.create_service_proposal(&proposal).await.is_err());

    let sponsorship = commercial_sales::Sponsorship {
        id: uuid::Uuid::new_v4(), company_id: company_b_uuid, customer_id,
        title: "Cross-company sponsorship".into(), currency: "USD".into(),
        committed_minor: 10_000, delivered_minor: 0, status: "PROSPECT".into(),
    };
    assert!(store.create_sponsorship(&sponsorship).await.is_err());

    let invoice = commercial_sales::Invoice {
        id: uuid::Uuid::new_v4(), company_id: company_b_uuid, customer_id,
        currency: "USD".into(), subtotal_minor: 10_000, paid_minor: 0,
        status: commercial_sales::InvoiceStatus::Draft, due_epoch: 1_800_000_000,
        idempotency_key: "isolation-invoice".into(),
    };
    let lines = vec![commercial_sales::InvoiceLine {
        description: "Cross-company line".into(), quantity: 1, unit_price_minor: 10_000,
    }];
    assert!(store.create_invoice(&invoice, &lines).await.is_err());

    assert!(store.create_customer_success_task(
        &company_b, uuid::Uuid::new_v4(), &customer_id.to_string(), "ONBOARDING",
        1_800_000_000, None, Some("cross-company customer"), "isolation-customer-task",
    ).await.is_err());
}

#[tokio::test]
async fn database_rejects_cross_company_invoice_customer_reference() {
    let Some(store) = connect_store().await else { return; };

    let company_a = uuid::Uuid::new_v4();
    let company_b = uuid::Uuid::new_v4();
    store.ensure_company(&company_a.to_string(), "FK Isolation A", "USD").await.unwrap();
    store.ensure_company(&company_b.to_string(), "FK Isolation B", "USD").await.unwrap();

    let customer_id = uuid::Uuid::new_v4();
    store.create_customer(
        &company_a.to_string(), customer_id, "Customer A", None, None, "LEAD", None,
        "fk-isolation-customer",
    ).await.unwrap();

    let client = store.client.lock().await;
    let result = client.execute(
        "INSERT INTO invoices
         (id,company_id,customer_id,currency,subtotal_minor,paid_minor,status,due_epoch,idempotency_key)
         VALUES ($1,$2,$3,'USD',100,0,'DRAFT',$4,$5)",
        &[&uuid::Uuid::new_v4(), &company_b, &customer_id, &1_800_000_000_i64, &"fk-cross-company-invoice"],
    ).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn typed_company_event_is_idempotent_in_outbox() {
    let Some(store) = connect_store().await else { return; };
    let company_id = uuid::Uuid::new_v4();
    store.ensure_company(&company_id.to_string(), "Event Contract", "USD").await.unwrap();

    let event = company_domain::CompanyEventEnvelope::new(
        company_id,
        company_domain::CompanyEventType::OrderCreated,
        "order",
        Some(uuid::Uuid::new_v4()),
        1_800_000_000,
        uuid::Uuid::new_v4(),
        None,
        "order-created-event-1",
        serde_json::json!({"order_minor": 2500}),
    ).unwrap();

    assert!(store.enqueue_company_event(&event).await.unwrap());
    assert!(!store.enqueue_company_event(&event).await.unwrap());

    let client = store.client.lock().await;
    let row = client.query_one(
        "SELECT event_type, schema_version, payload->>'event_type' FROM outbox_events WHERE company_id=$1 AND idempotency_key=$2",
        &[&company_id.to_string(), &event.idempotency_key],
    ).await.unwrap();
    assert_eq!(row.get::<_, String>(0), "ORDER_CREATED");
    assert_eq!(row.get::<_, i32>(1), 1);
    assert_eq!(row.get::<_, String>(2), "ORDER_CREATED");
}

}
