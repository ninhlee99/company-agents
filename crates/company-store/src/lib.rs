#![forbid(unsafe_code)]

use agent_runtime::{
    ExecutionEngine, ExecutionOutcome, AgentRunResult, CompanySnapshot,
};
use economic_core::{validate_balanced_transaction, LedgerTransaction};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

pub struct CompanyStore {
    client: Mutex<Client>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistCycleResult {
    Committed,
    AlreadyProcessed,
    InProgress,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutboxEvent {
    pub id: i64,
    pub company_id: String,
    pub event_type: String,
    pub aggregate_id: Option<String>,
    pub idempotency_key: String,
    pub schema_version: i32,
    pub payload: Value,
}

impl CompanyStore {
    pub async fn connect(database_url: &str) -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("postgres connection error: {error}");
            }
        });
        Ok(Self { client: Mutex::new(client) })
    }

    pub async fn migrate(&self) -> Result<(), tokio_postgres::Error> {
        let client = self.client.lock().await;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/001_economic_kernel.sql"))
            .await?;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/002_company_control.sql"))
            .await?;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/003_ledger_completeness.sql"))
            .await
    }

    pub async fn ensure_company(
        &self,
        company_id: &str,
        name: &str,
        currency: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err("currency must be a 3-letter uppercase code".into());
        }
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO companies (id, name, status, base_currency) VALUES ($1, $2, 'ACTIVE', $3)
                 ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, base_currency = EXCLUDED.base_currency",
                &[&id, &name, &currency],
            )
            .await?;

        let accounts = [
            ("1000", "Cash", "ASSET"),
            ("2000", "Accounts Payable", "LIABILITY"),
            ("3000", "Equity", "EQUITY"),
            ("4000", "Affiliate Revenue", "REVENUE"),
            ("5000", "Operating Expense", "EXPENSE"),
        ];
        for (code, account_name, account_type) in accounts {
            client.execute(
                "INSERT INTO ledger_accounts (id, company_id, code, name, account_type, currency)
                 VALUES ($1,$2,$3,$4,$5,$6)
                 ON CONFLICT (company_id,code) DO NOTHING",
                &[&Uuid::new_v4(), &id, &code, &account_name, &account_type, &currency],
            ).await?;
        }
        Ok(())
    }

    pub async fn post_ledger_transaction(
        &self,
        company_id: &str,
        transaction: &LedgerTransaction,
        idempotency_key: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        validate_balanced_transaction(transaction)
            .map_err(|e| format!("ledger validation failed: {e}"))?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let transaction_uuid = Uuid::parse_str(&transaction.id)?;

        let client = self.client.lock().await;
        let tx = client.transaction().await?;
        let inserted = tx.execute(
            "INSERT INTO ledger_transactions (id, company_id, description, idempotency_key)
             VALUES ($1,$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[&transaction_uuid, &company_uuid, &transaction.description, &idempotency_key],
        ).await?;

        if inserted == 0 {
            let row = tx.query_one(
                "SELECT id FROM ledger_transactions WHERE company_id=$1 AND idempotency_key=$2",
                &[&company_uuid, &idempotency_key],
            ).await?;
            tx.rollback().await?;
            return Ok(row.get(0));
        }

        for entry in &transaction.entries {
            let account_uuid = Uuid::parse_str(&entry.account_id)
                .map_err(|e| format!("invalid ledger account uuid: {e}"))?;
            tx.execute(
                "INSERT INTO ledger_entries (transaction_id, account_id, debit_minor, credit_minor, currency)
                 VALUES ($1,$2,$3::numeric,$4::numeric,$5)",
                &[
                    &transaction_uuid,
                    &account_uuid,
                    &entry.debit_minor.to_string(),
                    &entry.credit_minor.to_string(),
                    &entry.currency,
                ],
            ).await?;
        }

        tx.commit().await?;
        Ok(transaction_uuid)
    }

    pub async fn account_id_by_code(
        &self,
        company_id: &str,
        code: &str,
    ) -> Result<Option<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client.query_opt(
            "SELECT id FROM ledger_accounts WHERE company_id=$1 AND code=$2",
            &[&company_uuid, &code],
        ).await?;
        Ok(row.map(|r| r.get(0)))
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
                let state: Value = row.get(0);
                let snapshot: CompanySnapshot = serde_json::from_value(state)?;
                ExecutionEngine::validate_snapshot(&snapshot)
                    .map_err(|e| format!("persisted snapshot failed validation: {e}"))?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    pub async fn save_snapshot(
        &self,
        snapshot: &CompanySnapshot,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        ExecutionEngine::validate_snapshot(snapshot)
            .map_err(|e| format!("snapshot validation failed: {e}"))?;
        let id = Uuid::parse_str(&snapshot.company_id)?;
        let state = serde_json::to_value(snapshot)?;
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO company_state_snapshots (company_id, state) VALUES ($1, $2)
                 ON CONFLICT (company_id) DO UPDATE
                 SET state = EXCLUDED.state, updated_at = now(), revision = company_state_snapshots.revision + 1",
                &[&id, &state],
            )
            .await?;
        Ok(())
    }

    /// Transactionally records the decision cycle, updates the company snapshot,
    /// writes the decision journal and appends one durable outbox event.
    /// The idempotency key is the caller-supplied cycle UUID.
    pub async fn persist_decision_cycle(
        &self,
        snapshot: &CompanySnapshot,
        cycle_id: &str,
        results: &[AgentRunResult],
        outcomes: &[ExecutionOutcome],
    ) -> Result<PersistCycleResult, Box<dyn std::error::Error + Send + Sync>> {
        ExecutionEngine::validate_snapshot(snapshot)
            .map_err(|e| format!("snapshot validation failed: {e}"))?;
        if results.len() != outcomes.len() {
            return Err("results/outcomes length mismatch".into());
        }

        let company_id = Uuid::parse_str(&snapshot.company_id)?;
        let cycle_uuid = Uuid::parse_str(cycle_id)?;
        let state = serde_json::to_value(snapshot)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let inserted = tx.execute(
            "INSERT INTO idempotency_keys (company_id, key, command_type, status, response_json)
             VALUES ($1, $2, 'AGENT_CYCLE', 'PROCESSING', NULL)
             ON CONFLICT (company_id, key) DO NOTHING",
            &[&company_id, &cycle_id],
        ).await?;

        if inserted == 0 {
            let row = tx.query_one(
                "SELECT status FROM idempotency_keys WHERE company_id = $1 AND key = $2 FOR UPDATE",
                &[&company_id, &cycle_id],
            ).await?;
            let status: String = row.get(0);
            return match status.as_str() {
                "SUCCEEDED" => {
                    tx.rollback().await?;
                    Ok(PersistCycleResult::AlreadyProcessed)
                }
                "PROCESSING" => {
                    tx.rollback().await?;
                    Ok(PersistCycleResult::InProgress)
                }
                "FAILED" => {
                    tx.execute(
                        "UPDATE idempotency_keys SET status='PROCESSING', response_json=NULL WHERE company_id=$1 AND key=$2",
                        &[&company_id, &cycle_id],
                    ).await?;
                    Self::persist_cycle_rows(&tx, &company_id, &cycle_uuid, snapshot, &state, results, outcomes).await?;
                    tx.commit().await?;
                    Ok(PersistCycleResult::Committed)
                }
                _ => {
                    tx.rollback().await?;
                    Err(format!("unknown idempotency state: {status}").into())
                }
            };
        }

        Self::persist_cycle_rows(&tx, &company_id, &cycle_uuid, snapshot, &state, results, outcomes).await?;
        let response = json!({
            "cycle_id": cycle_id,
            "agents": results.len(),
            "executed": outcomes.iter().filter(|o| o.executed).count(),
            "state_changed": outcomes.iter().filter(|o| o.state_changed).count(),
        });

        tx.execute(
            "UPDATE idempotency_keys
             SET status='SUCCEEDED', response_json=$3, updated_at=now()
             WHERE company_id=$1 AND key=$2",
            &[&company_id, &cycle_id, &response],
        ).await?;
        tx.execute(
            "UPDATE cycle_runs SET status='SUCCEEDED', response_json=$2, completed_at=now()
             WHERE id=$1",
            &[&cycle_uuid, &response],
        ).await?;
        tx.commit().await?;
        Ok(PersistCycleResult::Committed)
    }

    async fn persist_cycle_rows(
        tx: &tokio_postgres::Transaction<'_>,
        company_id: &Uuid,
        cycle_id: &Uuid,
        snapshot: &CompanySnapshot,
        state: &Value,
        results: &[AgentRunResult],
        outcomes: &[ExecutionOutcome],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tx.execute(
            "INSERT INTO cycle_runs (id, company_id, status) VALUES ($1, $2, 'PROCESSING') ON CONFLICT (id) DO NOTHING",
            &[cycle_id, company_id],
        ).await?;

        for (result, outcome) in results.iter().zip(outcomes.iter()) {
            let payload = serde_json::to_value(result)?;
            let execution = serde_json::to_value(outcome)?;
            tx.execute(
                "INSERT INTO agent_runs (company_id, agent_name, payload) VALUES ($1, $2, $3)",
                &[company_id, &result.agent.as_str(), &payload],
            ).await?;
            let action = format!("{:?}", result.proposal.action);
            let decision = result.governance.as_ref()
                .map(|g| format!("{:?}", g.decision))
                .unwrap_or_else(|| "UNKNOWN".into());
            let execution_status = if outcome.executed {
                "EXECUTED"
            } else if outcome.decision == agent_runtime::GovernorDecision::Reject {
                "FAILED"
            } else {
                "SKIPPED"
            };
            tx.execute(
                "INSERT INTO decision_journal
                    (company_id, cycle_id, agent_name, action, governance_decision, execution_status, proposal_json, execution_json)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
                &[company_id, cycle_id, &result.agent.as_str(), &action, &decision, &execution_status, &payload, &execution],
            ).await?;
        }

        tx.execute(
            "INSERT INTO outbox_events
                (company_id, event_type, aggregate_id, idempotency_key, schema_version, payload)
             VALUES ($1,'AGENT_CYCLE_COMPLETED',$2,$3,1,$4)
             ON CONFLICT (company_id, idempotency_key) DO NOTHING",
            &[
                company_id,
                &snapshot.company_id,
                &format!("cycle:{cycle_id}"),
                &json!({"cycle_id": cycle_id, "results": results, "outcomes": outcomes}),
            ],
        ).await?;

        tx.execute(
            "INSERT INTO company_state_snapshots (company_id, state)
             VALUES ($1, $2)
             ON CONFLICT (company_id) DO UPDATE
             SET state = EXCLUDED.state,
                 updated_at = now(),
                 revision = company_state_snapshots.revision + 1",
            &[company_id, state],
        ).await?;

        tx.execute(
            "UPDATE companies SET status=$2 WHERE id=$1",
            &[company_id, &status_string(snapshot.status)],
        ).await?;

        Ok(())
    }

    /// Compatibility wrapper for existing callers/tests. New code should use
    /// persist_decision_cycle so actions and journal records are durable.
    pub async fn persist_cycle(
        &self,
        snapshot: &CompanySnapshot,
        results: &[AgentRunResult],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut clone_results = results.to_vec();
        let mut working = snapshot.clone();
        let outcomes = ExecutionEngine::default().execute_batch(&mut working, &mut clone_results);
        let cycle_id = Uuid::new_v4().to_string();
        let _ = self.persist_decision_cycle(&working, &cycle_id, &clone_results, &outcomes).await?;
        Ok(())
    }

    pub async fn recover_stale_cycles(&self, stale_after_secs: i64) -> Result<u64, tokio_postgres::Error> {
        let threshold = stale_after_secs.clamp(60, 86_400);
        let client = self.client.lock().await;
        let mut recovered = 0_u64;
        recovered += client.execute(
            "UPDATE cycle_runs SET status='FAILED', completed_at=now(),
                response_json=jsonb_build_object('error','stale cycle recovered')
             WHERE status='PROCESSING'
               AND created_at < now() - ($1 * interval '1 second')",
            &[&threshold],
        ).await?;
        recovered += client.execute(
            "UPDATE idempotency_keys SET status='FAILED',
                response_json=jsonb_build_object('error','stale cycle recovered'),
                updated_at=now()
             WHERE command_type='AGENT_CYCLE'
               AND status='PROCESSING'
               AND updated_at < now() - ($1 * interval '1 second')",
            &[&threshold],
        ).await?;
        Ok(recovered)
    }

    pub async fn pending_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxEvent>, tokio_postgres::Error> {
        let limit = limit.clamp(1, 1000);
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT id, company_id, event_type, aggregate_id, idempotency_key, schema_version, payload
             FROM outbox_events
             WHERE published_at IS NULL
             ORDER BY id
             LIMIT $1",
            &[&limit],
        ).await?;
        rows.into_iter().map(|row| {
            Ok(OutboxEvent {
                id: row.get(0),
                company_id: row.get::<_, Uuid>(1).to_string(),
                event_type: row.get(2),
                aggregate_id: row.get(3),
                idempotency_key: row.get(4),
                schema_version: row.get(5),
                payload: row.get(6),
            })
        }).collect()
    }

    pub async fn mark_outbox_published(&self, event_id: i64) -> Result<bool, tokio_postgres::Error> {
        let client = self.client.lock().await;
        Ok(client.execute(
            "UPDATE outbox_events SET published_at=now() WHERE id=$1 AND published_at IS NULL",
            &[&event_id],
        ).await? == 1)
    }
}

fn status_string(status: economic_core::CompanyStatus) -> String {
    match status {
        economic_core::CompanyStatus::Active => "ACTIVE",
        economic_core::CompanyStatus::Growth => "GROWTH",
        economic_core::CompanyStatus::Warning => "WARNING",
        economic_core::CompanyStatus::CostControl => "COST_CONTROL",
        economic_core::CompanyStatus::Distress => "DISTRESS",
        economic_core::CompanyStatus::Emergency => "EMERGENCY",
        economic_core::CompanyStatus::Liquidation => "LIQUIDATION",
        economic_core::CompanyStatus::Bankrupt => "BANKRUPT",
    }.into()
}

#[cfg(test)]
mod tests;
