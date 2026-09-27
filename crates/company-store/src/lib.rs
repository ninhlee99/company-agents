#![forbid(unsafe_code)]

use agent_runtime::types::AgentRole;
use agent_runtime::{AgentRunResult, CompanySnapshot};
use company_execution::{
    execute_approved_results, proposal_idempotency_key, ExecutionPolicy, ExecutionReceipt,
};
use economic_core::{validate_balanced_transaction, LedgerEntry, LedgerTransaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls, Transaction};
use uuid::Uuid;

pub struct CompanyStore {
    client: Mutex<Client>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedCycle {
    pub snapshot: CompanySnapshot,
    #[serde(default)]
    pub results: Vec<AgentRunResult>,
    pub receipts: Vec<ExecutionReceipt>,
}

impl CompanyStore {
    pub async fn connect(database_url: &str) -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("postgres connection error: {error}");
            }
        });
        Ok(Self {
            client: Mutex::new(client),
        })
    }

    pub async fn migrate(&self) -> Result<(), tokio_postgres::Error> {
        let client = self.client.lock().await;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/001_economic_kernel.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/002_company_execution.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/003_agent_memory_and_rate_limits.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/004_affiliate_attribution.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/005_media_jobs.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/006_company_portfolio.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/007_affiliate_currency.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/008_affiliate_payouts.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/009_outbox_dispatch.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/010_lease_tokens.sql"
            ))
            .await
    }

    pub async fn ensure_company(
        &self,
        company_id: &str,
        name: &str,
        currency: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        economic_core::Money::new(0, currency).map_err(|error| error.to_string())?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let existing = tx
            .query_opt(
                "SELECT base_currency FROM companies WHERE id = $1 FOR UPDATE",
                &[&id],
            )
            .await?;

        match existing {
            Some(row) => {
                let existing_currency: String = row.get(0);
                if !existing_currency.eq_ignore_ascii_case(currency) {
                    return Err(format!(
                        "company currency mismatch: database={existing_currency}, requested={currency}"
                    )
                    .into());
                }
                tx.execute(
                    "UPDATE companies SET name = $2 WHERE id = $1",
                    &[&id, &name],
                )
                .await?;
            }
            None => {
                tx.execute(
                    "INSERT INTO companies (id, name, status, base_currency)
                     VALUES ($1, $2, 'ACTIVE', $3)",
                    &[&id, &name, &currency],
                )
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load_snapshot(
        &self,
        company_id: &str,
    ) -> Result<Option<CompanySnapshot>, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_opt(
                "SELECT state FROM company_state_snapshots WHERE company_id = $1",
                &[&id],
            )
            .await?;

        match row {
            Some(row) => {
                let state: serde_json::Value = row.get(0);
                let snapshot: CompanySnapshot = serde_json::from_value(state)?;
                snapshot
                    .validate()
                    .map_err(|error| format!("persisted company snapshot is invalid: {error}"))?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    pub async fn save_snapshot(
        &self,
        snapshot: &CompanySnapshot,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        snapshot.validate().map_err(|error| error.to_string())?;
        let id = Uuid::parse_str(&snapshot.company_id)?;
        let state = serde_json::to_value(snapshot)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        tx.execute(
            "INSERT INTO company_state_snapshots (company_id, state)
             VALUES ($1, $2)
             ON CONFLICT (company_id) DO UPDATE
             SET state = EXCLUDED.state, updated_at = now()",
            &[&id, &state],
        )
        .await?;
        update_company_status(&tx, id, snapshot).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn load_portfolio(
        &self,
        company_id: &str,
    ) -> Result<Option<business_economics::CompanyPortfolio>, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_opt(
                "SELECT state FROM company_portfolio_snapshots WHERE company_id = $1",
                &[&id],
            )
            .await?;
        match row {
            Some(row) => {
                let state: serde_json::Value = row.get(0);
                let portfolio: business_economics::CompanyPortfolio =
                    serde_json::from_value(state)?;
                portfolio.validate().map_err(|error| error.to_string())?;
                Ok(Some(portfolio))
            }
            None => Ok(None),
        }
    }

    pub async fn save_portfolio(
        &self,
        company_id: &str,
        portfolio: &business_economics::CompanyPortfolio,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        portfolio.validate().map_err(|error| error.to_string())?;
        let id = Uuid::parse_str(company_id)?;
        let state = serde_json::to_value(portfolio)?;
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO company_portfolio_snapshots
                 (company_id, schema_version, state)
                 VALUES ($1, 1, $2)
                 ON CONFLICT (company_id) DO UPDATE
                 SET schema_version = 1, state = EXCLUDED.state, updated_at = now()",
                &[&id, &state],
            )
            .await?;
        Ok(())
    }

    pub async fn persist_cycle(
        &self,
        snapshot: &CompanySnapshot,
        results: &[AgentRunResult],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.persist_and_execute_cycle(snapshot, results)
            .await
            .map(|_| ())
    }

    pub async fn persist_and_execute_cycle(
        &self,
        proposed_snapshot: &CompanySnapshot,
        results: &[AgentRunResult],
    ) -> Result<PersistedCycle, Box<dyn std::error::Error + Send + Sync>> {
        let cycle_id = cycle_digest(proposed_snapshot, results)?;
        self.persist_and_execute_cycle_with_id(proposed_snapshot, results, &cycle_id)
            .await
    }

    pub async fn persist_and_execute_cycle_with_id(
        &self,
        proposed_snapshot: &CompanySnapshot,
        results: &[AgentRunResult],
        cycle_id: &str,
    ) -> Result<PersistedCycle, Box<dyn std::error::Error + Send + Sync>> {
        if cycle_id.trim().is_empty() {
            return Err("cycle id is required".into());
        }
        proposed_snapshot
            .validate()
            .map_err(|error| error.to_string())?;

        let company_id = Uuid::parse_str(&proposed_snapshot.company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let current_snapshot = match tx
            .query_opt(
                "SELECT state
                   FROM company_state_snapshots
                  WHERE company_id = $1
                  FOR UPDATE",
                &[&company_id],
            )
            .await?
        {
            Some(row) => serde_json::from_value::<CompanySnapshot>(row.get(0))?,
            None => proposed_snapshot.clone(),
        };

        current_snapshot
            .validate()
            .map_err(|error| format!("authoritative snapshot is invalid: {error}"))?;

        if current_snapshot.company_id != proposed_snapshot.company_id {
            return Err("authoritative snapshot belongs to another company".into());
        }

        let governor = agent_runtime::governor::Governor;
        let authoritative_results = results
            .iter()
            .map(|result| {
                let proposal = result
                    .governance
                    .as_ref()
                    .map(|governed| governed.proposal.clone())
                    .unwrap_or_else(|| result.proposal.clone());
                let governance = governor.evaluate(proposal.clone(), &current_snapshot);
                AgentRunResult {
                    agent: result.agent,
                    proposal,
                    governance: Some(governance),
                }
            })
            .collect::<Vec<_>>();

        let cycle_key = format!("cycle:{cycle_id}");
        if let Some(row) = tx
            .query_opt(
                "SELECT status, response_json
                   FROM idempotency_keys
                  WHERE company_id = $1 AND key = $2
                  FOR UPDATE",
                &[&company_id, &cycle_key],
            )
            .await?
        {
            let status: String = row.get(0);
            if status == "SUCCEEDED" {
                let response = row
                    .get::<_, Option<serde_json::Value>>(1)
                    .ok_or("idempotent cycle is missing response_json")?;
                let persisted = serde_json::from_value::<PersistedCycle>(response)?;
                tx.rollback().await?;
                return Ok(persisted);
            }
            if status == "PROCESSING" {
                return Err("cycle is already being processed".into());
            }
            return Err("cycle is in a terminal failed state".into());
        }

        tx.execute(
            "INSERT INTO idempotency_keys
             (company_id, key, command_type, status)
             VALUES ($1, $2, 'agent_cycle', 'PROCESSING')",
            &[&company_id, &cycle_key],
        )
        .await?;

        let batch =
            execute_approved_results(current_snapshot, &authoritative_results, execution_policy())?;

        for result in &authoritative_results {
            let proposal = &result.proposal;
            let proposal_digest = proposal_idempotency_key(proposal);
            let proposal_key = format!("{cycle_key}:{proposal_digest}");
            let payload = serde_json::to_value(result)?;

            tx.execute(
                "INSERT INTO agent_runs
                 (company_id, agent_name, payload, idempotency_key)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (company_id, idempotency_key) DO NOTHING",
                &[&company_id, &result.agent.as_str(), &payload, &proposal_key],
            )
            .await?;

            let receipt = batch
                .receipts
                .iter()
                .find(|item| item.idempotency_key == proposal_digest)
                .cloned();
            let decision = result
                .governance
                .as_ref()
                .map(|item| format!("{:?}", item.decision))
                .unwrap_or_else(|| "MISSING".into());
            let reason = result
                .governance
                .as_ref()
                .map(|item| item.reason.clone())
                .unwrap_or_else(|| "missing Governor result".into());
            let execution = receipt.as_ref().map(serde_json::to_value).transpose()?;
            let executed = receipt.as_ref().is_some_and(|item| {
                matches!(
                    item.status,
                    company_execution::ExecutionStatus::Executed
                        | company_execution::ExecutionStatus::Noop
                )
            });

            tx.execute(
                "INSERT INTO agent_memory
                 (company_id, agent_name, memory_key, value,
                  confidence_bps, importance)
                 VALUES ($1, $2, 'last_decision', $3, $4, 60)
                 ON CONFLICT (company_id, agent_name, memory_key)
                 DO UPDATE SET value = EXCLUDED.value,
                               confidence_bps = EXCLUDED.confidence_bps,
                               importance = EXCLUDED.importance,
                               expires_at = NULL",
                &[
                    &company_id,
                    &result.agent.as_str(),
                    &serde_json::json!({
                        "action": format!("{:?}", proposal.action),
                        "decision": decision,
                        "reason": reason,
                        "cost_minor": proposal.cost_minor,
                        "expected_revenue_minor": proposal.expected_revenue_minor,
                    }),
                    &(proposal.confidence_bps as i32),
                ],
            )
            .await?;

            tx.execute(
                "INSERT INTO decision_journal
                 (company_id, idempotency_key, agent_name, action,
                  governor_decision, reason, proposal, execution, executed)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (company_id, idempotency_key) DO NOTHING",
                &[
                    &company_id,
                    &proposal_key,
                    &result.agent.as_str(),
                    &format!("{:?}", proposal.action),
                    &decision,
                    &reason,
                    &payload,
                    &execution,
                    &executed,
                ],
            )
            .await?;

            tx.execute(
                "INSERT INTO audit_log
                 (company_id, actor_type, actor_id, action,
                  resource_type, resource_id, decision, metadata)
                 VALUES ($1, 'AGENT', $2, $3, 'COMPANY_STATE',
                         $4, $5, $6)",
                &[
                    &company_id,
                    &result.agent.as_str(),
                    &format!("{:?}", proposal.action),
                    &cycle_key,
                    &decision,
                    &serde_json::json!({
                        "reason": reason,
                        "execution": execution,
                    }),
                ],
            )
            .await?;

            tx.execute(
                "INSERT INTO outbox_events
                 (company_id, event_type, aggregate_id, idempotency_key, payload)
                 VALUES ($1, 'AGENT_DECISION_RECORDED', $2, $3, $4)
                 ON CONFLICT (company_id, idempotency_key) DO NOTHING",
                &[
                    &company_id,
                    &proposed_snapshot.company_id,
                    &format!("outbox:{proposal_key}"),
                    &serde_json::json!({
                        "cycle_id": cycle_id,
                        "agent": result.agent.as_str(),
                        "execution": execution,
                    }),
                ],
            )
            .await?;
        }

        let persisted = PersistedCycle {
            snapshot: batch.snapshot.clone(),
            results: authoritative_results,
            receipts: batch.receipts,
        };

        let currency = persisted_cycle_currency(&tx, company_id).await?;
        persist_execution_ledgers(
            &tx,
            company_id,
            &cycle_key,
            &currency,
            &persisted.receipts,
        )
        .await?;
        let response = serde_json::to_value(&persisted)?;

        tx.execute(
            "INSERT INTO company_state_snapshots (company_id, state)
             VALUES ($1, $2)
             ON CONFLICT (company_id) DO UPDATE
             SET state = EXCLUDED.state, updated_at = now()",
            &[&company_id, &response["snapshot"]],
        )
        .await?;
        update_company_status(&tx, company_id, &persisted.snapshot).await?;

        tx.execute(
            "UPDATE idempotency_keys
                SET status = 'SUCCEEDED', response_json = $3
              WHERE company_id = $1 AND key = $2",
            &[&company_id, &cycle_key, &response],
        )
        .await?;

        tx.execute(
            "INSERT INTO company_cycle_health
             (company_id, last_started_at, last_succeeded_at, last_error,
              consecutive_failures, updated_at)
             VALUES ($1, now(), now(), NULL, 0, now())
             ON CONFLICT (company_id)
             DO UPDATE SET last_started_at = now(), last_succeeded_at = now(),
                           last_error = NULL, consecutive_failures = 0,
                           updated_at = now()",
            &[&company_id],
        )
        .await?;

        tx.commit().await?;
        Ok(persisted)
    }

    pub async fn append_ledger_transaction(
        &self,
        company_id: &str,
        transaction: &LedgerTransaction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        validate_balanced_transaction(transaction).map_err(|error| error.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let transaction_uuid = Uuid::parse_str(&transaction.id)?;
        let idempotency_key = format!("ledger:{}", transaction.id);

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        if let Some(row) = tx
            .query_opt(
                "SELECT status
                   FROM idempotency_keys
                  WHERE company_id = $1 AND key = $2
                  FOR UPDATE",
                &[&company_uuid, &idempotency_key],
            )
            .await?
        {
            let status: String = row.get(0);
            tx.rollback().await?;
            return match status.as_str() {
                "SUCCEEDED" => Ok(()),
                "PROCESSING" => Err("ledger transaction is already processing".into()),
                "FAILED" => Err("ledger transaction previously failed".into()),
                _ => Err("ledger transaction has unknown idempotency state".into()),
            };
        }

        tx.execute(
            "INSERT INTO idempotency_keys
             (company_id, key, command_type, status)
             VALUES ($1, $2, 'ledger_transaction', 'PROCESSING')",
            &[&company_uuid, &idempotency_key],
        )
        .await?;

        tx.execute(
            "INSERT INTO ledger_transactions
             (id, company_id, description, idempotency_key)
             VALUES ($1, $2, $3, $4)",
            &[
                &transaction_uuid,
                &company_uuid,
                &transaction.description,
                &idempotency_key,
            ],
        )
        .await?;

        for entry in &transaction.entries {
            let account_uuid = Uuid::parse_str(&entry.account_id)?;
            let debit = entry.debit_minor.to_string();
            let credit = entry.credit_minor.to_string();
            tx.execute(
                "INSERT INTO ledger_entries
                 (transaction_id, account_id, debit_minor, credit_minor, currency)
                 VALUES ($1, $2, $3::numeric, $4::numeric, $5)",
                &[
                    &transaction_uuid,
                    &account_uuid,
                    &debit,
                    &credit,
                    &entry.currency,
                ],
            )
            .await?;
        }

        tx.execute(
            "INSERT INTO audit_log
             (company_id, actor_type, actor_id, action,
              resource_type, resource_id, decision, metadata)
             VALUES ($1, 'SYSTEM', 'economic-kernel',
                     'LEDGER_TRANSACTION_COMMITTED', 'LEDGER_TRANSACTION',
                     $2, 'APPROVE', $3)",
            &[
                &company_uuid,
                &transaction.id,
                &serde_json::json!({
                    "entries": transaction.entries.len(),
                    "description": transaction.description,
                }),
            ],
        )
        .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id, event_type, aggregate_id, idempotency_key, payload)
             VALUES ($1, 'LEDGER_TRANSACTION_COMMITTED', $2, $3, $4)
             ON CONFLICT (company_id, idempotency_key) DO NOTHING",
            &[
                &company_uuid,
                &transaction.id,
                &format!("outbox:{idempotency_key}"),
                &serde_json::json!({
                    "transaction_id": transaction.id,
                    "description": transaction.description,
                }),
            ],
        )
        .await?;

        tx.execute(
            "UPDATE idempotency_keys
                SET status = 'SUCCEEDED',
                    response_json = jsonb_build_object(
                        'transaction_id', $3)
              WHERE company_id = $1 AND key = $2",
            &[&company_uuid, &idempotency_key, &transaction.id],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn ensure_recurring_job(
        &self,
        company_id: &str,
        job_type: &str,
        interval_seconds: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if interval_seconds < 15 {
            return Err("scheduler interval must be >= 15 seconds".into());
        }
        let id = Uuid::parse_str(company_id)?;
        let job_id = Uuid::new_v4();
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO scheduled_jobs
                 (id, company_id, job_type, interval_seconds,
                  next_run_at, status)
                 VALUES ($1, $2, $3, $4, now(), 'ACTIVE')
                 ON CONFLICT (company_id, job_type) DO NOTHING",
                &[&job_id, &id, &job_type, &interval_seconds],
            )
            .await?;
        Ok(())
    }

    pub async fn claim_due_job(
        &self,
        company_id: &str,
        job_type: &str,
    ) -> Result<Option<(Uuid, Uuid)>, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx
            .query_opt(
                "SELECT id, run_token
                   FROM scheduled_jobs
                  WHERE company_id = $1
                    AND job_type = $2
                    AND status = 'ACTIVE'
                    AND next_run_at <= now()
                    AND (locked_until IS NULL OR locked_until <= now())
                  FOR UPDATE SKIP LOCKED",
                &[&id, &job_type],
            )
            .await?;
        let Some(row) = row else {
            tx.rollback().await?;
            return Ok(None);
        };
        let job_id: Uuid = row.get(0);
        let run_token: Uuid = row.get(1);
        let lease_token = Uuid::new_v4();
        tx.execute(
            "UPDATE scheduled_jobs
                SET locked_until = now() + interval '5 minutes',
                    lease_token = $2,
                    updated_at = now()
              WHERE id = $1",
            &[&job_id, &lease_token],
        )
        .await?;
        tx.commit().await?;
        Ok(Some((job_id, run_token, lease_token)))
    }

    pub async fn complete_job(
        &self,
        job_id: Uuid,
        run_token: Uuid,
        lease_token: Uuid,
        next_run_token: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.lock().await;
        let updated = client
            .execute(
                "UPDATE scheduled_jobs
                    SET next_run_at = now() +
                        make_interval(secs => interval_seconds),
                        locked_until = NULL,
                        lease_token = NULL,
                        run_token = $4,
                        updated_at = now()
                  WHERE id = $1
                    AND run_token = $2
                    AND lease_token = $3
                    AND locked_until > now()",
                &[&job_id, &run_token, &lease_token, &next_run_token],
            )
            .await?;
        if updated != 1 {
            return Err("scheduler completion rejected: stale or invalid lease".into());
        }
        Ok(())
    }

    pub async fn release_job_after_failure(
        &self,
        job_id: Uuid,
        run_token: Uuid,
        lease_token: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.lock().await;
        let updated = client
            .execute(
                "UPDATE scheduled_jobs
                    SET next_run_at = now() + interval '30 seconds',
                        locked_until = NULL,
                        lease_token = NULL,
                        updated_at = now()
                  WHERE id = $1
                    AND run_token = $2
                    AND lease_token = $3",
                &[&job_id, &run_token, &lease_token],
            )
            .await?;
        if updated != 1 {
            return Err("scheduler failure release rejected: stale or invalid lease".into());
        }
        Ok(())
    }

    pub async fn load_agent_memory(
        &self,
        company_id: &str,
        agent: AgentRole,
        limit: i64,
    ) -> Result<Vec<agent_runtime::types::AgentMemory>, Box<dyn std::error::Error + Send + Sync>>
    {
        if !(1..=100).contains(&limit) {
            return Err("memory limit must be between 1 and 100".into());
        }
        let id = Uuid::parse_str(company_id)?;
        let name = agent.as_str();
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT memory_key, value, confidence_bps, importance,
                        updated_at::text, expires_at::text
                   FROM agent_memory
                  WHERE company_id = $1
                    AND agent_name = $2
                    AND (expires_at IS NULL OR expires_at > now())
                  ORDER BY importance DESC, updated_at DESC
                  LIMIT $3",
                &[&id, &name, &limit],
            )
            .await?;

        rows.into_iter()
            .map(|row| {
                let confidence: i32 = row.get(2);
                let importance: i16 = row.get(3);
                Ok(agent_runtime::types::AgentMemory {
                    key: row.get(0),
                    value: row.get(1),
                    confidence_bps: confidence.clamp(0, 10_000) as u16,
                    importance: importance.clamp(0, 100) as u8,
                    updated_at: row.get(4),
                    expires_at: row.get(5),
                })
            })
            .collect()
    }

    pub async fn load_all_agent_memory(
        &self,
        company_id: &str,
        limit_per_agent: i64,
    ) -> Result<
        std::collections::HashMap<AgentRole, Vec<agent_runtime::types::AgentMemory>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        if !(1..=100).contains(&limit_per_agent) {
            return Err("memory limit must be between 1 and 100".into());
        }
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT agent_name, memory_key, value, confidence_bps, importance,
                        updated_at::text, expires_at::text
                   FROM (
                        SELECT agent_name, memory_key, value, confidence_bps, importance,
                               updated_at, expires_at,
                               row_number() OVER (
                                   PARTITION BY agent_name
                                   ORDER BY importance DESC, updated_at DESC
                               ) AS row_number
                          FROM agent_memory
                         WHERE company_id = $1
                           AND (expires_at IS NULL OR expires_at > now())
                   ) ranked
                  WHERE row_number <= $2
                  ORDER BY agent_name, importance DESC, updated_at DESC",
                &[&id, &limit_per_agent],
            )
            .await?;

        let mut result = std::collections::HashMap::new();
        for row in rows {
            let Some(agent) = parse_agent_role(row.get(0)) else {
                continue;
            };
            let confidence: i32 = row.get(3);
            let importance: i16 = row.get(4);
            result
                .entry(agent)
                .or_insert_with(Vec::new)
                .push(agent_runtime::types::AgentMemory {
                    key: row.get(1),
                    value: row.get(2),
                    confidence_bps: confidence.clamp(0, 10_000) as u16,
                    importance: importance.clamp(0, 100) as u8,
                    updated_at: row.get(5),
                    expires_at: row.get(6),
                });
        }
        Ok(result)
    }

    pub async fn upsert_agent_memory(
        &self,
        company_id: &str,
        agent: AgentRole,
        key: &str,
        value: &serde_json::Value,
        confidence_bps: u16,
        importance: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if key.trim().is_empty() || key.len() > 128 {
            return Err("memory key must be 1..=128 bytes".into());
        }
        if confidence_bps > 10_000 {
            return Err("memory confidence must be <= 10000".into());
        }
        if importance > 100 {
            return Err("memory importance must be <= 100".into());
        }
        let encoded = serde_json::to_vec(value)?;
        if encoded.len() > 16 * 1024 {
            return Err("memory value exceeds 16 KiB safety limit".into());
        }

        let company_uuid = Uuid::parse_str(company_id)?;
        let agent_name = agent.as_str();
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO agent_memory
                 (company_id, agent_name, memory_key, value,
                  confidence_bps, importance)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (company_id, agent_name, memory_key)
                 DO UPDATE SET value = EXCLUDED.value,
                               confidence_bps = EXCLUDED.confidence_bps,
                               importance = EXCLUDED.importance,
                               expires_at = NULL",
                &[
                    &company_uuid,
                    &agent_name,
                    &key,
                    &value,
                    &(confidence_bps as i32),
                    &(importance as i16),
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn claim_agent_run_slots(
        &self,
        company_id: &str,
        agents: &[AgentRole],
        window_seconds: i64,
        max_calls: i32,
    ) -> Result<Vec<AgentRole>, Box<dyn std::error::Error + Send + Sync>> {
        if !(15..=86_400).contains(&window_seconds) {
            return Err("agent rate window must be 15..=86400 seconds".into());
        }
        if !(1..=100).contains(&max_calls) {
            return Err("agent max calls must be 1..=100".into());
        }

        let company_uuid = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let mut allowed = Vec::new();

        for agent in agents.iter().copied() {
            if agent == AgentRole::Governor {
                continue;
            }
            let name = agent.as_str();
            let row = tx
                .query_opt(
                    "INSERT INTO agent_rate_windows
                     (company_id, agent_name, window_started_at, call_count, max_calls)
                     VALUES ($1, $2, now(), 1, $4)
                     ON CONFLICT (company_id, agent_name)
                     DO UPDATE SET
                       window_started_at = CASE
                         WHEN agent_rate_windows.window_started_at <=
                              now() - make_interval(secs => $3)
                         THEN now()
                         ELSE agent_rate_windows.window_started_at
                       END,
                       call_count = CASE
                         WHEN agent_rate_windows.window_started_at <=
                              now() - make_interval(secs => $3)
                         THEN 1
                         ELSE agent_rate_windows.call_count + 1
                       END,
                       max_calls = EXCLUDED.max_calls,
                       updated_at = now()
                     WHERE agent_rate_windows.window_started_at <=
                               now() - make_interval(secs => $3)
                        OR agent_rate_windows.call_count < $4
                     RETURNING agent_name",
                    &[&company_uuid, &name, &window_seconds, &max_calls],
                )
                .await?;
            if row.is_some() {
                allowed.push(agent);
            }
        }

        tx.commit().await?;
        Ok(allowed)
    }

    pub async fn record_affiliate_click(
        &self,
        event: &affiliate_attribution::ClickEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        affiliate_attribution::validate_click_event(event)
            .map_err(|error| error.to_string())?;
        let company_id = Uuid::parse_str(&event.company_id)?;
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO affiliate_clicks
                 (company_id, click_id, product_id, advertiser_id, content_id,
                  occurred_at, source)
                 VALUES ($1,$2,$3,$4,$5,$6,$7)
                 ON CONFLICT (company_id, click_id) DO NOTHING",
                &[
                    &company_id,
                    &event.click_id,
                    &event.product_id,
                    &event.advertiser_id,
                    &event.content_id,
                    &event.occurred_at,
                    &event.source,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn record_affiliate_conversion(
        &self,
        event: &affiliate_attribution::ConversionEvent,
        model: affiliate_attribution::AttributionModel,
    ) -> Result<affiliate_attribution::ReconciledConversion, Box<dyn std::error::Error + Send + Sync>>
    {
        affiliate_attribution::validate_conversion_event(event)
            .map_err(|error| error.to_string())?;
        let company_id = Uuid::parse_str(&event.company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        if let Some(row) = tx
            .query_opt(
                "SELECT conversion_id, order_value_minor::text, commission_minor::text,
                        refunded_minor::text, cancelled, click_id, product_id,
                        advertiser_id, occurred_at, currency, source
                   FROM affiliate_conversions
                  WHERE company_id = $1 AND idempotency_key = $2",
                &[
                    &company_id,
                    &affiliate_attribution::conversion_idempotency_key(event),
                ],
            )
            .await?
        {
            let conversion_id: String = row.get(0);
            let result = affiliate_attribution::ReconciledConversion {
                conversion_id,
                attributed: tx
                    .query(
                        "SELECT click_id, product_id, content_id,
                                attributed_order_value_minor::text,
                                attributed_commission_minor::text,
                                confidence_bps
                           FROM affiliate_attributions
                          WHERE company_id = $1 AND conversion_id = $2
                          ORDER BY click_id",
                        &[&company_id, &row.get::<_, String>(0)],
                    )
                    .await?
                    .into_iter()
                    .map(
                        |r| -> Result<
                            affiliate_attribution::Attribution,
                            Box<dyn std::error::Error + Send + Sync>,
                        > {
                            Ok(affiliate_attribution::Attribution {
                                click_id: r.get(0),
                                product_id: r.get(1),
                                content_id: r.get(2),
                                attributed_order_value_minor: parse_i128_numeric(
                                    &r.get::<_, String>(3),
                                )?,
                                attributed_commission_minor: parse_i128_numeric(
                                    &r.get::<_, String>(4),
                                )?,
                                confidence_bps: (r.get::<_, i32>(5)).clamp(0, 10_000) as u32,
                            })
                        },
                    )
                    .collect::<Result<Vec<_>, _>>()?,
                net_commission_minor: parse_i128_numeric(&row.get::<_, String>(2))?,
                reconciliation_variance_minor: 0,
                status: affiliate_attribution::ReconciliationStatus::Verified,
                idempotency_key: affiliate_attribution::conversion_idempotency_key(event),
            };
            tx.rollback().await?;
            return Ok(result);
        }

        let rows = tx
            .query(
                "SELECT click_id, company_id, product_id, advertiser_id, content_id,
                        occurred_at, source
                   FROM affiliate_clicks
                  WHERE company_id = $1 AND product_id = $2 AND advertiser_id = $3
                  ORDER BY occurred_at ASC, click_id ASC",
                &[&company_id, &event.product_id, &event.advertiser_id],
            )
            .await?;
        let clicks = rows
            .into_iter()
            .map(|row| affiliate_attribution::ClickEvent {
                click_id: row.get(0),
                company_id: row.get::<_, Uuid>(1).to_string(),
                product_id: row.get(2),
                advertiser_id: row.get(3),
                content_id: row.get(4),
                occurred_at: row.get(5),
                source: row.get(6),
            })
            .collect::<Vec<_>>();

        let company_currency = company_base_currency(&tx, company_id).await?;
        if company_currency != event.currency {
            return Err(format!(
                "affiliate conversion currency {} does not match company currency {}",
                event.currency, company_currency
            )
            .into());
        }

        let reconciled = affiliate_attribution::attribute_conversion(event, &clicks, model)
            .map_err(|error| error.to_string())?;

        let order_value = event.order_value_minor.to_string();
        let commission = event.commission_minor.to_string();
        let refunded = event.refunded_minor.to_string();
        let idempotency_key = reconciled.idempotency_key.clone();

        tx.execute(
            "INSERT INTO affiliate_conversions
             (company_id, conversion_id, click_id, order_id, product_id,
              advertiser_id, occurred_at, currency, order_value_minor, commission_minor,
              refunded_minor, cancelled, source, idempotency_key)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10::numeric,$11::numeric,
                     $12,$13,$14)",
            &[
                &company_id,
                &event.conversion_id,
                &event.click_id,
                &event.order_id,
                &event.product_id,
                &event.advertiser_id,
                &event.occurred_at,
                &event.currency,
                &order_value,
                &commission,
                &refunded,
                &event.cancelled,
                &event.source,
                &idempotency_key,
            ],
        )
        .await?;

        for attribution in &reconciled.attributed {
            let value = attribution.attributed_order_value_minor.to_string();
            let commission = attribution.attributed_commission_minor.to_string();
            tx.execute(
                "INSERT INTO affiliate_attributions
                 (company_id, conversion_id, click_id, product_id, content_id,
                  attributed_order_value_minor, attributed_commission_minor, confidence_bps)
                 VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8)
                 ON CONFLICT (company_id, conversion_id, click_id) DO NOTHING",
                &[
                    &company_id,
                    &event.conversion_id,
                    &attribution.click_id,
                    &attribution.product_id,
                    &attribution.content_id,
                    &value,
                    &commission,
                    &(attribution.confidence_bps as i32),
                ],
            )
            .await?;
        }

        if matches!(
            reconciled.status,
            affiliate_attribution::ReconciliationStatus::Verified
        ) && reconciled.net_commission_minor > 0
        {
            let currency = company_base_currency(&tx, company_id).await?;
            persist_affiliate_revenue_earned(
                &tx,
                company_id,
                &event.conversion_id,
                reconciled.net_commission_minor,
                &currency,
            )
            .await?;
            increment_company_revenue(
                &tx,
                company_id,
                reconciled.net_commission_minor,
            )
            .await?;
        }

        tx.execute(
            "INSERT INTO outbox_events
             (company_id, event_type, aggregate_id, idempotency_key, payload)
             VALUES ($1,'AFFILIATE_CONVERSION_RECONCILED',$2,$3,$4)
             ON CONFLICT (company_id, idempotency_key) DO NOTHING",
            &[
                &company_id,
                &event.conversion_id,
                &format!("outbox:{idempotency_key}"),
                &serde_json::to_value(&reconciled)?,
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(reconciled)
    }

    pub async fn record_affiliate_payout(
        &self,
        event: &affiliate_attribution::AffiliatePayoutEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        affiliate_attribution::validate_payout_event(event)
            .map_err(|error| error.to_string())?;
        let company_id = Uuid::parse_str(&event.company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let company_currency = company_base_currency(&tx, company_id).await?;
        if company_currency != event.currency {
            return Err(format!(
                "affiliate payout currency {} does not match company currency {}",
                event.currency, company_currency
            )
            .into());
        }

        let idempotency_key = affiliate_attribution::payout_idempotency_key(event);
        if tx
            .query_opt(
                "SELECT payout_id
                   FROM affiliate_payouts
                  WHERE company_id = $1 AND idempotency_key = $2
                  FOR UPDATE",
                &[&company_id, &idempotency_key],
            )
            .await?
            .is_some()
        {
            tx.rollback().await?;
            return Ok(());
        }

        let receivable_balance: String = tx
            .query_one(
                "SELECT COALESCE(
                    SUM(le.debit_minor - le.credit_minor)::numeric, 0
                )::text
                   FROM ledger_entries le
                   JOIN ledger_accounts la ON la.id = le.account_id
                  WHERE la.company_id = $1
                    AND la.code = 'AFFILIATE_RECEIVABLE'
                    AND la.currency = $2",
                &[&company_id, &event.currency],
            )
            .await?
            .get(0);
        let receivable = parse_i128_numeric(&receivable_balance)?;
        if event.amount_minor > receivable {
            return Err(format!(
                "affiliate payout {} exceeds outstanding receivable {}",
                event.amount_minor, receivable
            )
            .into());
        }

        tx.execute(
            "INSERT INTO affiliate_payouts
             (company_id, payout_id, occurred_at, amount_minor,
              currency, source, idempotency_key)
             VALUES ($1,$2,$3,$4::numeric,$5,$6,$7)",
            &[
                &company_id,
                &event.payout_id,
                &event.occurred_at,
                &event.amount_minor.to_string(),
                &event.currency,
                &event.source,
                &idempotency_key,
            ],
        )
        .await?;

        if event.amount_minor > 0 {
            let receivable_account = ensure_system_account(
                &tx,
                company_id,
                "AFFILIATE_RECEIVABLE",
                "Affiliate Receivable",
                "ASSET",
                &event.currency,
            )
            .await?;
            let cash_account = ensure_system_account(
                &tx,
                company_id,
                "CASH",
                "Cash",
                "ASSET",
                &event.currency,
            )
            .await?;
            let transaction_id = Uuid::new_v4();
            let amount = event.amount_minor.to_string();
            tx.execute(
                "INSERT INTO ledger_transactions
                 (id, company_id, description, idempotency_key)
                 VALUES ($1,$2,$3,$4)",
                &[
                    &transaction_id,
                    &company_id,
                    &"Affiliate payout received",
                    &format!("ledger:{idempotency_key}"),
                ],
            )
            .await?;
            tx.execute(
                "INSERT INTO ledger_entries
                 (transaction_id, account_id, debit_minor, credit_minor, currency)
                 VALUES ($1,$2,$3::numeric,0,$4),($1,$5,0,$3::numeric,$4)",
                &[
                    &transaction_id,
                    &cash_account,
                    &amount,
                    &event.currency,
                    &receivable_account,
                ],
            )
            .await?;
            increment_company_cash(&tx, company_id, event.amount_minor).await?;
        }

        tx.execute(
            "INSERT INTO outbox_events
             (company_id, event_type, aggregate_id, idempotency_key, payload)
             VALUES ($1,'AFFILIATE_PAYOUT_RECEIVED',$2,$3,$4)
             ON CONFLICT (company_id, idempotency_key) DO NOTHING",
            &[
                &company_id,
                &event.payout_id,
                &format!("outbox:{idempotency_key}"),
                &serde_json::json!({
                    "payout_id": event.payout_id,
                    "amount_minor": event.amount_minor,
                    "currency": event.currency,
                }),
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn content_affiliate_performance(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) {
            return Err("performance limit must be between 1 and 200".into());
        }
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT jsonb_build_object(
                    'content_id', content_id,
                    'orders_minor', SUM(attributed_order_value_minor)::text,
                    'commission_minor', SUM(attributed_commission_minor)::text,
                    'attributions', COUNT(*),
                    'avg_confidence_bps', ROUND(AVG(confidence_bps))::int)
                   FROM affiliate_attributions
                  WHERE company_id = $1
                  GROUP BY content_id
                  ORDER BY SUM(attributed_commission_minor) DESC
                  LIMIT $2",
                &[&id, &limit],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| row.get::<_, serde_json::Value>(0))
            .collect())
    }

    pub async fn enqueue_media_job(
        &self,
        company_id: &str,
        job: &media_pipeline::MediaJob,
        idempotency_key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        media_pipeline::validate_job(job).map_err(|error| error.to_string())?;
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 256 {
            return Err("media idempotency key is invalid".into());
        }
        let company_id = Uuid::parse_str(company_id)?;
        let format = match job.format {
            media_pipeline::MediaFormat::Mp4H264 => "Mp4H264",
            media_pipeline::MediaFormat::WebMvp9 => "WebMvp9",
        };
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO media_jobs
                 (id, company_id, input_path, output_path, format,
                  width, height, fps, max_duration_seconds,
                  normalize_audio, status, idempotency_key)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'QUEUED',$11)
                 ON CONFLICT (company_id, idempotency_key) DO NOTHING",
                &[
                    &Uuid::parse_str(&job.id)?,
                    &company_id,
                    &job.input_path,
                    &job.output_path,
                    &format,
                    &(job.width as i32),
                    &(job.height as i32),
                    &(job.fps as i32),
                    &(job.max_duration_seconds as i32),
                    &job.normalize_audio,
                    &idempotency_key,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn claim_media_job(
        &self,
        company_id: &str,
    ) -> Result<Option<media_pipeline::MediaJob>, Box<dyn std::error::Error + Send + Sync>> {
        let company_id = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx
            .query_opt(
                "SELECT id, input_path, output_path, format, width, height,
                        fps, max_duration_seconds, normalize_audio
                   FROM media_jobs
                  WHERE company_id = $1
                    AND (
                      status = 'QUEUED'
                      OR (status = 'RUNNING' AND locked_until <= now())
                    )
                  ORDER BY created_at ASC
                  FOR UPDATE SKIP LOCKED
                  LIMIT 1",
                &[&company_id],
            )
            .await?;
        let Some(row) = row else {
            tx.rollback().await?;
            return Ok(None);
        };

        let id: Uuid = row.get(0);
        let format: String = row.get(3);
        let job = media_pipeline::MediaJob {
            id: id.to_string(),
            input_path: row.get(1),
            output_path: row.get(2),
            format: match format.as_str() {
                "Mp4H264" => media_pipeline::MediaFormat::Mp4H264,
                "WebMvp9" => media_pipeline::MediaFormat::WebMvp9,
                other => {
                    tx.rollback().await?;
                    return Err(format!("unknown media format {other}").into());
                }
            },
            width: u32::try_from(row.get::<_, i32>(4))
                .map_err(|_| "invalid media width in persistent job")?,
            height: u32::try_from(row.get::<_, i32>(5))
                .map_err(|_| "invalid media height in persistent job")?,
            fps: u32::try_from(row.get::<_, i32>(6))
                .map_err(|_| "invalid media fps in persistent job")?,
            max_duration_seconds: u32::try_from(row.get::<_, i32>(7))
                .map_err(|_| "invalid media duration in persistent job")?,
            normalize_audio: row.get(8),
        };
        media_pipeline::validate_job(&job).map_err(|error| error.to_string())?;

        let lease_token = Uuid::new_v4();
        tx.execute(
            "UPDATE media_jobs
                SET status='RUNNING', attempts=attempts+1,
                    locked_until=now()+interval '15 minutes',
                    lease_token=$2,
                    updated_at=now()
              WHERE id=$1 AND (status='QUEUED' OR (status='RUNNING' AND locked_until <= now()))
            ",
            &[&id, &lease_token],
        )
        .await?;
        tx.commit().await?;
        Ok(Some((job, lease_token)))
    }

    pub async fn finish_media_job(
        &self,
        job_id: &str,
        lease_token: Uuid,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !matches!(status, "SUCCEEDED" | "FAILED" | "QA_FAILED") {
            return Err("invalid media completion status".into());
        }
        let id = Uuid::parse_str(job_id)?;
        let bounded_error = error.map(|value| value.chars().take(4096).collect::<String>());
        let client = self.client.lock().await;
        let updated = client
            .execute(
                "UPDATE media_jobs
                    SET status = CASE
                          WHEN $2 = 'FAILED' AND attempts < 3 THEN 'QUEUED'
                          ELSE $2
                        END,
                        locked_until = NULL,
                        lease_token = NULL,
                        last_error = $3,
                        updated_at = now()
                  WHERE id = $1
                    AND status = 'RUNNING'
                    AND lease_token = $4
                    AND locked_until > now()",
                &[&id, &status, &bounded_error, &lease_token],
            )
            .await?;
        if updated != 1 {
            return Err("media job completion rejected: job is not RUNNING".into());
        }
        Ok(())
    }

    pub async fn record_cycle_failure(
        &self,
        company_id: &str,
        error: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let bounded = error.chars().take(4096).collect::<String>();
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO company_cycle_health
                 (company_id, last_failed_at, last_error,
                  consecutive_failures, updated_at)
                 VALUES ($1, now(), $2, 1, now())
                 ON CONFLICT (company_id)
                 DO UPDATE SET last_failed_at = now(), last_error = $2,
                               consecutive_failures =
                                 company_cycle_health.consecutive_failures + 1,
                               updated_at = now()",
                &[&id, &bounded],
            )
            .await?;
        Ok(())
    }

    pub async fn claim_outbox_events(
        &self,
        company_id: &str,
        limit: i64,
        lease_seconds: i64,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) || !(5..=900).contains(&lease_seconds) {
            return Err("outbox claim bounds are invalid".into());
        }
        let id = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let rows = tx
            .query(
                "WITH candidates AS (
                    SELECT id
                      FROM outbox_events
                     WHERE company_id = $1
                       AND published_at IS NULL
                       AND (locked_until IS NULL OR locked_until <= now())
                     ORDER BY created_at ASC
                     FOR UPDATE SKIP LOCKED
                     LIMIT $2
                )
                UPDATE outbox_events event
                   SET locked_until = now() + make_interval(secs => $3),
                       lease_token = gen_random_uuid(),
                       attempt_count = event.attempt_count + 1
                  FROM candidates
                 WHERE event.id = candidates.id
             RETURNING event.id, event.event_type, event.aggregate_id,
                       event.idempotency_key, event.schema_version,
                       event.payload, event.attempt_count, event.lease_token",
                &[&id, &limit, &lease_seconds],
            )
            .await?;

        let events = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<_, i64>(0),
                    "event_type": row.get::<_, String>(1),
                    "aggregate_id": row.get::<_, Option<String>>(2),
                    "idempotency_key": row.get::<_, String>(3),
                    "schema_version": row.get::<_, i32>(4),
                    "payload": row.get::<_, serde_json::Value>(5),
                    "attempt_count": row.get::<_, i32>(6),
                })
            })
            .collect::<Vec<_>>();
        tx.commit().await?;
        Ok(events)
    }

    pub async fn mark_outbox_published(
        &self,
        company_id: &str,
        event_id: i64,
        lease_token: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let updated = client
            .execute(
                "UPDATE outbox_events
                    SET published_at = now(), locked_until = NULL,
                        lease_token = NULL, last_error = NULL
                  WHERE company_id = $1
                    AND id = $2
                    AND published_at IS NULL
                    AND lease_token = $3
                    AND locked_until > now()",
                &[&id, &event_id, &lease_token],
            )
            .await?;
        if updated != 1 {
            return Err("outbox publish acknowledgement rejected".into());
        }
        Ok(())
    }

    pub async fn mark_outbox_failed(
        &self,
        company_id: &str,
        event_id: i64,
        lease_token: Uuid,
        error: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let bounded = error.chars().take(4096).collect::<String>();
        let client = self.client.lock().await;
        client
            .execute(
                "UPDATE outbox_events
                    SET locked_until = now() +
                          CASE
                            WHEN attempt_count >= 8 THEN interval '1 hour'
                            ELSE make_interval(secs => LEAST(900, GREATEST(5, attempt_count * 15)))
                          END,
                        last_error = $4
                  WHERE company_id = $1
                    AND id = $2
                    AND published_at IS NULL
                    AND lease_token = $3",
                &[&id, &event_id, &lease_token, &bounded],
            )
            .await?;
        Ok(())
    }

    pub async fn recent_journal(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) {
            return Err("journal limit must be between 1 and 200".into());
        }
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT jsonb_build_object(
                    'id', id,
                    'idempotency_key', idempotency_key,
                    'agent_name', agent_name,
                    'action', action,
                    'governor_decision', governor_decision,
                    'reason', reason,
                    'execution', execution,
                    'executed', executed,
                    'created_at', created_at)
                 FROM decision_journal
                 WHERE company_id = $1
                 ORDER BY id DESC
                 LIMIT $2",
                &[&id, &limit],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| row.get::<_, serde_json::Value>(0))
            .collect())
    }
}

async fn company_base_currency(
    tx: &Transaction<'_>,
    company_id: Uuid,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let row = tx
        .query_one(
            "SELECT base_currency FROM companies WHERE id = $1",
            &[&company_id],
        )
        .await?;
    Ok(row.get(0))
}

async fn persist_affiliate_revenue_earned(
    tx: &Transaction<'_>,
    company_id: Uuid,
    conversion_id: &str,
    amount_minor: i128,
    currency: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if amount_minor <= 0 {
        return Ok(());
    }
    let receivable = ensure_system_account(
        tx,
        company_id,
        "AFFILIATE_RECEIVABLE",
        "Affiliate Receivable",
        "ASSET",
        currency,
    )
    .await?;
    let revenue = ensure_system_account(
        tx,
        company_id,
        "AFFILIATE_REVENUE",
        "Affiliate Revenue",
        "REVENUE",
        currency,
    )
    .await?;
    let ledger_key = format!("affiliate-revenue:{conversion_id}");
    let transaction_id = Uuid::new_v4();
    let exists = tx
        .query_opt(
            "SELECT id FROM ledger_transactions WHERE company_id = $1 AND idempotency_key = $2",
            &[&company_id, &ledger_key],
        )
        .await?
        .is_some();
    if exists {
        return Ok(());
    }

    tx.execute(
        "INSERT INTO ledger_transactions
         (id, company_id, description, idempotency_key)
         VALUES ($1,$2,$3,$4)",
        &[
            &transaction_id,
            &company_id,
            &"Affiliate commission earned",
            &ledger_key,
        ],
    )
    .await?;

    let amount = amount_minor.to_string();
    tx.execute(
        "INSERT INTO ledger_entries
         (transaction_id, account_id, debit_minor, credit_minor, currency)
         VALUES ($1,$2,$3::numeric,0,$4),($1,$5,0,$3::numeric,$4)",
        &[&transaction_id, &receivable, &amount, &currency, &revenue],
    )
    .await?;

    tx.execute(
        "INSERT INTO outbox_events
         (company_id, event_type, aggregate_id, idempotency_key, payload)
         VALUES ($1,'AFFILIATE_REVENUE_EARNED',$2,$3,$4)
         ON CONFLICT (company_id, idempotency_key) DO NOTHING",
        &[
            &company_id,
            &conversion_id,
            &format!("outbox:{ledger_key}"),
            &serde_json::json!({
                "conversion_id": conversion_id,
                "amount_minor": amount_minor,
                "currency": currency,
            }),
        ],
    )
    .await?;
    Ok(())
}

fn parse_agent_role(value: &str) -> Option<AgentRole> {
    AgentRole::ALL
        .into_iter()
        .find(|role| role.as_str().eq_ignore_ascii_case(value))
}

async fn update_company_snapshot(
    tx: &Transaction<'_>,
    company_id: Uuid,
    mutate: impl FnOnce(&mut CompanySnapshot) -> Result<(), String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let row = tx
        .query_opt(
            "SELECT state FROM company_state_snapshots WHERE company_id = $1 FOR UPDATE",
            &[&company_id],
        )
        .await?
        .ok_or("company snapshot is required for economic mutation")?;
    let state: serde_json::Value = row.get(0);
    let mut snapshot: CompanySnapshot = serde_json::from_value(state)?;
    mutate(&mut snapshot).map_err(|error| error.to_string())?;
    snapshot.validate().map_err(|error| error.to_string())?;
    let state = serde_json::to_value(&snapshot)?;
    tx.execute(
        "UPDATE company_state_snapshots
            SET state = $2, updated_at = now()
          WHERE company_id = $1",
        &[&company_id, &state],
    )
    .await?;
    update_company_status(tx, company_id, &snapshot).await?;
    Ok(())
}

async fn increment_company_revenue(
    tx: &Transaction<'_>,
    company_id: Uuid,
    amount_minor: i128,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    update_company_snapshot(tx, company_id, |snapshot| {
        snapshot.revenue_minor = snapshot
            .revenue_minor
            .checked_add(amount_minor)
            .ok_or_else(|| "revenue arithmetic overflow".to_string())?;
        snapshot.assets_minor = snapshot
            .assets_minor
            .checked_add(amount_minor)
            .ok_or_else(|| "asset arithmetic overflow".to_string())?;

        Ok(())
    })
    .await
}

async fn increment_company_cash(
    tx: &Transaction<'_>,
    company_id: Uuid,
    amount_minor: i128,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    update_company_snapshot(tx, company_id, |snapshot| {
        snapshot.cash_minor = snapshot
            .cash_minor
            .checked_add(amount_minor)
            .ok_or_else(|| "cash arithmetic overflow".to_string())?;
        Ok(())
    })
    .await
}

async fn persisted_cycle_currency(
    tx: &Transaction<'_>,
    company_id: Uuid,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let row = tx
        .query_one(
            "SELECT base_currency FROM companies WHERE id = $1",
            &[&company_id],
        )
        .await?;
    let currency: String = row.get(0);
    if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err("company base currency is invalid".into());
    }
    Ok(currency)
}

async fn persist_execution_ledgers(
    tx: &Transaction<'_>,
    company_id: Uuid,
    cycle_key: &str,
    currency: &str,
    receipts: &[ExecutionReceipt],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let spend = receipts
        .iter()
        .filter(|receipt| {
            matches!(receipt.status, company_execution::ExecutionStatus::Executed)
                && receipt.action == agent_runtime::types::ActionKind::CreateExperiment
        })
        .try_fold(0_i128, |total, receipt| {
            total.checked_add(receipt.cost_minor).ok_or("execution spend overflow")
        })?;

    if spend == 0 {
        return Ok(());
    }

    let cash_account = ensure_system_account(
        tx,
        company_id,
        "CASH",
        "Cash",
        "ASSET",
        currency,
    )
    .await?;
    let expense_account = ensure_system_account(
        tx,
        company_id,
        "EXPERIMENT_EXPENSE",
        "Experiment Expense",
        "EXPENSE",
        currency,
    )
    .await?;

    for receipt in receipts
        .iter()
        .filter(|item| {
            matches!(item.status, company_execution::ExecutionStatus::Executed)
                && item.action == agent_runtime::types::ActionKind::CreateExperiment
        })
        .filter(|item| item.cost_minor > 0)
    {
        let proposal_key = &receipt.idempotency_key;
        let ledger_key = format!("ledger:{cycle_key}:{proposal_key}");
        let transaction_id = Uuid::new_v4();
        let amount = receipt.cost_minor;
        let transaction = LedgerTransaction {
            id: transaction_id.to_string(),
            description: format!("Agent {:?} execution spend", receipt.action),
            entries: vec![
                LedgerEntry {
                    account_id: expense_account.to_string(),
                    debit_minor: amount,
                    credit_minor: 0,
                    currency: currency.to_owned(),
                },
                LedgerEntry {
                    account_id: cash_account.to_string(),
                    debit_minor: 0,
                    credit_minor: amount,
                    currency: currency.to_owned(),
                },
            ],
        };
        validate_balanced_transaction(&transaction).map_err(|error| error.to_string())?;

        tx.execute(
            "INSERT INTO ledger_transactions
             (id, company_id, description, idempotency_key)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (company_id, idempotency_key) DO NOTHING",
            &[
                &transaction_id,
                &company_id,
                &transaction.description,
                &ledger_key,
            ],
        )
        .await?;

        tx.execute(
            "INSERT INTO ledger_entries
             (transaction_id, account_id, debit_minor, credit_minor, currency)
             VALUES ($1,$2,$3::numeric,0,$4),($1,$5,0,$3::numeric,$4)",
            &[
                &transaction_id,
                &expense_account,
                &amount.to_string(),
                &currency,
                &cash_account,
            ],
        )
        .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id, event_type, aggregate_id, idempotency_key, payload)
             VALUES ($1,'LEDGER_TRANSACTION_COMMITTED',$2,$3,$4)
             ON CONFLICT (company_id, idempotency_key) DO NOTHING",
            &[
                &company_id,
                &transaction_id.to_string(),
                &format!("outbox:{ledger_key}"),
                &serde_json::json!({
                    "transaction_id": transaction_id,
                    "agent": receipt.agent.as_str(),
                    "action": format!("{:?}", receipt.action),
                    "amount_minor": amount,
                    "currency": currency,
                }),
            ],
        )
        .await?;
    }
    Ok(())
}

async fn ensure_system_account(
    tx: &Transaction<'_>,
    company_id: Uuid,
    code: &str,
    name: &str,
    account_type: &str,
    currency: &str,
) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
    let id = Uuid::new_v4();
    tx.execute(
        "INSERT INTO ledger_accounts
         (id, company_id, code, name, account_type, currency)
         VALUES ($1,$2,$3,$4,$5,$6)
         ON CONFLICT (company_id, code) DO NOTHING",
        &[&id, &company_id, &code, &name, &account_type, &currency],
    )
    .await?;
    let row = tx
        .query_one(
            "SELECT id, currency FROM ledger_accounts WHERE company_id = $1 AND code = $2",
            &[&company_id, &code],
        )
        .await?;
    let account_id: Uuid = row.get(0);
    let account_currency: String = row.get(1);
    if !account_currency.eq_ignore_ascii_case(currency) {
        return Err(format!("system ledger account {code} uses wrong currency").into());
    }
    Ok(account_id)
}

fn execution_policy() -> ExecutionPolicy {
    let max_spend = std::env::var("MAX_EXECUTION_SPEND_MINOR")
        .ok()
        .and_then(|value| value.parse::<i128>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(1_000);
    ExecutionPolicy {
        max_spend_per_cycle_minor: max_spend,
    }
}

fn cycle_digest(
    snapshot: &CompanySnapshot,
    results: &[AgentRunResult],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut digest = Sha256::new();
    digest.update(serde_json::to_vec(snapshot)?);
    digest.update([0]);
    digest.update(serde_json::to_vec(results)?);
    Ok(format!("{:x}", digest.finalize()))
}

async fn update_company_status(
    tx: &Transaction<'_>,
    company_id: Uuid,
    snapshot: &CompanySnapshot,
) -> Result<(), tokio_postgres::Error> {
    let status = match snapshot.status {
        economic_core::CompanyStatus::Active => "ACTIVE",
        economic_core::CompanyStatus::Growth => "GROWTH",
        economic_core::CompanyStatus::Warning => "WARNING",
        economic_core::CompanyStatus::CostControl => "COST_CONTROL",
        economic_core::CompanyStatus::Distress => "DISTRESS",
        economic_core::CompanyStatus::Emergency => "EMERGENCY",
        economic_core::CompanyStatus::Liquidation => "LIQUIDATION",
        economic_core::CompanyStatus::Bankrupt => "BANKRUPT",
    };
    tx.execute(
        "UPDATE companies SET status = $2 WHERE id = $1",
        &[&company_id, &status],
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests;

#[async_trait::async_trait]
impl agent_runtime::agent::AgentStateProvider for CompanyStore {
    async fn load_memory(
        &self,
        company_id: &str,
        agent: agent_runtime::types::AgentRole,
    ) -> Result<serde_json::Value, String> {
        let records = self
            .load_agent_memory(company_id, agent, 50)
            .await
            .map_err(|error| error.to_string())?;
        Ok(serde_json::json!({
            "items": records,
        }))
    }

    async fn admit_model_call(
        &self,
        company_id: &str,
        agent: agent_runtime::types::AgentRole,
    ) -> Result<(), String> {
        let window_seconds = std::env::var("AGENT_RATE_WINDOW_SECONDS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| (15..=86_400).contains(v))
            .unwrap_or(60);
        let max_calls = std::env::var("AGENT_MAX_MODEL_CALLS_PER_WINDOW")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|v| (1..=100).contains(v))
            .unwrap_or(60);
        let allowed = self
            .claim_agent_run_slots(company_id, &[agent], window_seconds, max_calls)
            .await
            .map_err(|error| error.to_string())?;
        if allowed.contains(&agent) {
            Ok(())
        } else {
            Err(format!(
                "durable model-call rate limit exceeded for {} ({max_calls} calls/{window_seconds}s)",
                agent.as_str()
            ))
        }
    }
}

fn parse_i128_numeric(value: &str) -> Result<i128, Box<dyn std::error::Error + Send + Sync>> {
    value
        .parse::<i128>()
        .map_err(|error| format!("numeric value out of i128 range: {error}").into())
}
