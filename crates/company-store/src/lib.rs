#![forbid(unsafe_code)]

use agent_runtime::types::AgentRole;
use agent_runtime::{AgentRunResult, CompanySnapshot};
use company_execution::{
    execute_approved_results, proposal_idempotency_key, ExecutionPolicy, ExecutionReceipt,
};
use economic_core::{validate_balanced_transaction, CompanyStatus, LedgerTransaction};
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
                "../../../infra/db/migrations/006_affiliate_reconciliation_state.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/007_organization_payroll.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/008_affiliate_revenue_accounting.sql"
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
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO companies (id, name, status, base_currency)
                 VALUES ($1, $2, 'ACTIVE', $3)
                 ON CONFLICT (id) DO UPDATE
                 SET name = EXCLUDED.name",
                &[&id, &name, &currency],
            )
            .await?;
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
                Ok(Some(serde_json::from_value(state)?))
            }
            None => Ok(None),
        }
    }

    pub async fn save_snapshot(
        &self,
        snapshot: &CompanySnapshot,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            upsert_agent_memory_tx(
                &tx,
                company_id,
                result.agent,
                "last_cycle_decision",
                &serde_json::json!({
                    "cycle_id": cycle_id,
                    "action": format!("{:?}", proposal.action),
                    "decision": decision,
                    "reason": reason,
                    "cost_minor": proposal.cost_minor,
                    "expected_revenue_minor": proposal.expected_revenue_minor,
                    "confidence_bps": proposal.confidence_bps,
                    "executed": executed,
                }),
                proposal.confidence_bps,
                70,
            )
            .await?;

        }

        let persisted = PersistedCycle {
            snapshot: batch.snapshot.clone(),
            results: authoritative_results,
            receipts: batch.receipts,
        };
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

    pub async fn upsert_employee(
        &self,
        company_id: &str,
        employee: &company_organization::Employee,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        company_organization::validate_employee(employee)
            .map_err(|error| error.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        if employee.currency != self.company_currency(&company_uuid).await? {
            return Err("employee currency must match company currency".into());
        }
        let employee_uuid = Uuid::parse_str(&employee.id)?;
        let status = match employee.status {
            company_organization::EmployeeStatus::Proposed => "PROPOSED",
            company_organization::EmployeeStatus::Active => "ACTIVE",
            company_organization::EmployeeStatus::Suspended => "SUSPENDED",
            company_organization::EmployeeStatus::Terminated => "TERMINATED",
        };
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "INSERT INTO employees
                 (id, company_id, name, role, monthly_cost_minor, currency, status)
                 VALUES ($1,$2,$3,$4,$5::numeric,$6,$7)
                 ON CONFLICT (id) DO UPDATE
                 SET name = EXCLUDED.name,
                     role = EXCLUDED.role,
                     monthly_cost_minor = EXCLUDED.monthly_cost_minor,
                     currency = EXCLUDED.currency,
                     status = EXCLUDED.status,
                     updated_at = now()
                 WHERE employees.company_id = EXCLUDED.company_id",
                &[
                    &employee_uuid,
                    &company_uuid,
                    &employee.name,
                    &employee.role,
                    &employee.monthly_cost_minor.to_string(),
                    &employee.currency,
                    &status,
                ],
            )
            .await?;
        if changed == 0 {
            return Err("employee id already belongs to another company".into());
        }
        Ok(())
    }

    pub async fn list_employees(
        &self,
        company_id: &str,
    ) -> Result<Vec<company_organization::Employee>, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT id, name, role, monthly_cost_minor::text, currency, status
                   FROM employees
                  WHERE company_id = $1
                  ORDER BY created_at ASC, id ASC",
                &[&company_uuid],
            )
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(company_organization::Employee {
                    id: row.get::<_, Uuid>(0).to_string(),
                    name: row.get(1),
                    role: row.get(2),
                    monthly_cost_minor: parse_i128_numeric(&row.get::<_, String>(3))?,
                    currency: row.get(4),
                    status: parse_employee_status(&row.get::<_, String>(5))?,
                })
            })
            .collect()
    }

    pub async fn accrue_payroll(
        &self,
        company_id: &str,
        period: &str,
        due_at: &str,
    ) -> Result<i128, Box<dyn std::error::Error + Send + Sync>> {
        if period.trim().is_empty() || period.len() > 32 {
            return Err("payroll period is invalid".into());
        }
        let due = time::OffsetDateTime::parse(
            due_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|error| format!("invalid payroll due_at: {error}"))?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let currency = self.company_currency(&company_uuid).await?;
        let accrual_key = format!("payroll:accrual:{company_id}:{period}");

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        if tx
            .query_opt(
                "SELECT status FROM idempotency_keys
                  WHERE company_id = $1 AND key = $2
                  FOR UPDATE",
                &[&company_uuid, &accrual_key],
            )
            .await?
            .is_some()
        {
            tx.rollback().await?;
            return Ok(0);
        }

        let employees = tx
            .query(
                "SELECT id, monthly_cost_minor::text, currency
                   FROM employees
                  WHERE company_id = $1 AND status = 'ACTIVE'
                  ORDER BY id ASC",
                &[&company_uuid],
            )
            .await?;
        if employees.is_empty() {
            tx.rollback().await?;
            return Ok(0);
        }

        tx.execute(
            "INSERT INTO idempotency_keys
             (company_id, key, command_type, status)
             VALUES ($1,$2,'payroll_accrual','PROCESSING')",
            &[&company_uuid, &accrual_key],
        )
        .await?;

        let mut total = 0_i128;
        for row in employees {
            let employee_id: Uuid = row.get(0);
            let amount = parse_i128_numeric(&row.get::<_, String>(1))?;
            let employee_currency: String = row.get(2);
            if employee_currency != currency {
                return Err("active employee currency differs from company currency".into());
            }
            if amount < 0 {
                return Err("employee monthly cost cannot be negative".into());
            }
            let obligation_id = Uuid::new_v4();
            let due_text = due.format(&time::format_description::well_known::Rfc3339)?;
            let row = tx
                .query_opt(
                    "INSERT INTO payroll_obligations
                     (id, company_id, employee_id, period, gross_minor, currency, due_at)
                     VALUES ($1,$2,$3,$4,$5::numeric,$6,$7)
                     ON CONFLICT (company_id, employee_id, period) DO NOTHING
                     RETURNING gross_minor::text",
                    &[
                        &obligation_id,
                        &company_uuid,
                        &employee_id,
                        &period,
                        &amount.to_string(),
                        &currency,
                        &due_text,
                    ],
                )
                .await?;
            if let Some(inserted) = row {
                total = total
                    .checked_add(parse_i128_numeric(&inserted.get::<_, String>(0))?)
                    .ok_or("payroll accrual overflow")?;
            }
        }

        if total > 0 {
            let (_cash_account, liability_account, expense_account) =
                ensure_payroll_accounts(&tx, company_uuid, &currency).await?;
            let transaction_id = Uuid::new_v4();
            let debit = total.to_string();
            let credit = total.to_string();
            tx.execute(
                "INSERT INTO ledger_transactions
                 (id, company_id, description, idempotency_key)
                 VALUES ($1,$2,'monthly payroll accrual',$3)",
                &[&transaction_id, &company_uuid, &accrual_key],
            )
            .await?;
            insert_ledger_entry(
                &tx,
                transaction_id,
                expense_account,
                &debit,
                "0",
                &currency,
            )
            .await?;
            insert_ledger_entry(
                &tx,
                transaction_id,
                liability_account,
                "0",
                &credit,
                &currency,
            )
            .await?;

            update_snapshot_financials(&tx, company_uuid, |snapshot| {
                snapshot.expenses_minor = snapshot
                    .expenses_minor
                    .checked_add(total)
                    .ok_or("payroll expense overflow".to_string())?;
                snapshot.liabilities_minor = snapshot
                    .liabilities_minor
                    .checked_add(total)
                    .ok_or("payroll liability overflow".to_string())?;
                Ok(())
            })
            .await?;
        }

        tx.execute(
            "UPDATE idempotency_keys
                SET status='SUCCEEDED',
                    response_json=jsonb_build_object('accrued_minor',$3::numeric)
              WHERE company_id=$1 AND key=$2",
            &[&company_uuid, &accrual_key, &total.to_string()],
        )
        .await?;

        tx.commit().await?;
        Ok(total)
    }

    pub async fn pay_payroll(
        &self,
        company_id: &str,
        obligation_id: &str,
        amount_minor: i128,
    ) -> Result<i128, Box<dyn std::error::Error + Send + Sync>> {
        if amount_minor <= 0 {
            return Err("payroll payment must be positive".into());
        }
        let company_uuid = Uuid::parse_str(company_id)?;
        let obligation_uuid = Uuid::parse_str(obligation_id)?;
        let payment_key = format!("payroll:payment:{obligation_id}:{amount_minor}");

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        if tx
            .query_opt(
                "SELECT status FROM idempotency_keys
                  WHERE company_id=$1 AND key=$2
                  FOR UPDATE",
                &[&company_uuid, &payment_key],
            )
            .await?
            .is_some()
        {
            tx.rollback().await?;
            return Ok(0);
        }

        let row = tx
            .query_opt(
                "SELECT gross_minor::text, paid_minor::text, currency
                   FROM payroll_obligations
                  WHERE company_id=$1 AND id=$2
                  FOR UPDATE",
                &[&company_uuid, &obligation_uuid],
            )
            .await?
            .ok_or("payroll obligation not found")?;
        let gross = parse_i128_numeric(&row.get::<_, String>(0))?;
        let paid = parse_i128_numeric(&row.get::<_, String>(1))?;
        let currency: String = row.get(2);
        let remaining = gross.checked_sub(paid).ok_or("payroll remaining overflow")?;
        if amount_minor > remaining {
            return Err("payroll payment exceeds remaining obligation".into());
        }

        let snapshot = authoritative_snapshot(&tx, company_uuid).await?;
        if snapshot.status == CompanyStatus::Bankrupt {
            return Err("bankrupt company cannot initiate payroll payment".into());
        }
        if amount_minor > snapshot.cash_minor {
            return Err("insufficient cash for payroll payment".into());
        }

        tx.execute(
            "INSERT INTO idempotency_keys
             (company_id,key,command_type,status)
             VALUES ($1,$2,'payroll_payment','PROCESSING')",
            &[&company_uuid, &payment_key],
        )
        .await?;

        let (cash_account, liability_account, _expense_account) =
            ensure_payroll_accounts(&tx, company_uuid, &currency).await?;
        let transaction_id = Uuid::new_v4();
        let amount = amount_minor.to_string();
        tx.execute(
            "INSERT INTO ledger_transactions
             (id, company_id, description, idempotency_key)
             VALUES ($1,$2,'payroll payment',$3)",
            &[&transaction_id, &company_uuid, &payment_key],
        )
        .await?;
        insert_ledger_entry(
            &tx,
            transaction_id,
            liability_account,
            &amount,
            "0",
            &currency,
        )
        .await?;
        insert_ledger_entry(
            &tx,
            transaction_id,
            cash_account,
            "0",
            &amount,
            &currency,
        )
        .await?;

        let next_paid = paid
            .checked_add(amount_minor)
            .ok_or("payroll paid amount overflow")?;
        tx.execute(
            "UPDATE payroll_obligations
                SET paid_minor=$3::numeric, updated_at=now()
              WHERE company_id=$1 AND id=$2",
            &[&company_uuid, &obligation_uuid, &next_paid.to_string()],
        )
        .await?;

        update_snapshot_financials(&tx, company_uuid, |snapshot| {
            snapshot.cash_minor = snapshot
                .cash_minor
                .checked_sub(amount_minor)
                .ok_or("cash underflow".to_string())?;
            snapshot.assets_minor = snapshot
                .assets_minor
                .checked_sub(amount_minor)
                .ok_or("asset underflow".to_string())?;
            snapshot.liabilities_minor = snapshot
                .liabilities_minor
                .checked_sub(amount_minor)
                .ok_or("liability underflow".to_string())?;
            if snapshot.cash_minor == 0 {
                snapshot.status = CompanyStatus::Emergency;
            }
            Ok(())
        })
        .await?;

        tx.execute(
            "UPDATE idempotency_keys
                SET status='SUCCEEDED',
                    response_json=jsonb_build_object('paid_minor',$3::numeric)
              WHERE company_id=$1 AND key=$2",
            &[&company_uuid, &payment_key, &amount_minor.to_string()],
        )
        .await?;

        tx.commit().await?;
        Ok(amount_minor)
    }

    pub async fn payroll_due(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<company_organization::PayrollObligation>, Box<dyn std::error::Error + Send + Sync>>
    {
        if !(1..=500).contains(&limit) {
            return Err("payroll limit must be between 1 and 500".into());
        }
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT id, employee_id, period, gross_minor::text, currency,
                        due_at::text, paid_minor::text
                   FROM payroll_obligations
                  WHERE company_id=$1
                    AND paid_minor < gross_minor
                    AND due_at <= now()
                  ORDER BY due_at ASC, id ASC
                  LIMIT $2",
                &[&company_uuid, &limit],
            )
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(company_organization::PayrollObligation {
                    id: row.get::<_, Uuid>(0).to_string(),
                    employee_id: row.get::<_, Uuid>(1).to_string(),
                    period: row.get(2),
                    gross_minor: parse_i128_numeric(&row.get::<_, String>(3))?,
                    currency: row.get(4),
                    due_at: row.get(5),
                    paid_minor: parse_i128_numeric(&row.get::<_, String>(6))?,
                })
            })
            .collect()
    }

    async fn company_currency(
        &self,
        company_id: &Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.lock().await;
        let row = client
            .query_opt(
                "SELECT TRIM(base_currency)::text FROM companies WHERE id=$1",
                &[company_id],
            )
            .await?
            .ok_or("company not found")?;
        Ok(row.get(0))
    }

    pub async fn upsert_business_unit(
        &self,
        company_id: &str,
        unit: &company_organization::BusinessUnit,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if unit.id.trim().is_empty() || unit.name.trim().is_empty() {
            return Err("business unit id and name are required".into());
        }
        let company_uuid = Uuid::parse_str(company_id)?;
        if unit.currency != self.company_currency(&company_uuid).await? {
            return Err("business unit currency must match company currency".into());
        }
        if unit.cash_minor < 0
            || unit.revenue_minor < 0
            || unit.variable_cost_minor < 0
            || unit.fixed_cost_minor < 0
            || unit.budget_minor < 0
        {
            return Err("business unit economics cannot be negative".into());
        }
        let unit_uuid = Uuid::parse_str(&unit.id)?;
        let lifecycle = match unit.lifecycle {
            company_organization::BusinessUnitLifecycle::Testing => "TESTING",
            company_organization::BusinessUnitLifecycle::Growing => "GROWING",
            company_organization::BusinessUnitLifecycle::Stable => "STABLE",
            company_organization::BusinessUnitLifecycle::Distress => "DISTRESS",
            company_organization::BusinessUnitLifecycle::Paused => "PAUSED",
            company_organization::BusinessUnitLifecycle::Closed => "CLOSED",
        };
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "INSERT INTO business_units
                 (id, company_id, name, currency, cash_minor, revenue_minor,
                  variable_cost_minor, fixed_cost_minor, budget_minor, lifecycle)
                 VALUES ($1,$2,$3,$4,$5::numeric,$6::numeric,$7::numeric,
                         $8::numeric,$9::numeric,$10)
                 ON CONFLICT (id) DO UPDATE
                 SET name=EXCLUDED.name,
                     currency=EXCLUDED.currency,
                     cash_minor=EXCLUDED.cash_minor,
                     revenue_minor=EXCLUDED.revenue_minor,
                     variable_cost_minor=EXCLUDED.variable_cost_minor,
                     fixed_cost_minor=EXCLUDED.fixed_cost_minor,
                     budget_minor=EXCLUDED.budget_minor,
                     lifecycle=EXCLUDED.lifecycle,
                     updated_at=now()
                 WHERE business_units.company_id = EXCLUDED.company_id",
                &[
                    &unit_uuid,
                    &company_uuid,
                    &unit.name,
                    &unit.currency,
                    &unit.cash_minor.to_string(),
                    &unit.revenue_minor.to_string(),
                    &unit.variable_cost_minor.to_string(),
                    &unit.fixed_cost_minor.to_string(),
                    &unit.budget_minor.to_string(),
                    &lifecycle,
                ],
            )
            .await?;
        if changed == 0 {
            return Err("business unit id already belongs to another company".into());
        }
        Ok(())
    }

    pub async fn list_business_units(
        &self,
        company_id: &str,
    ) -> Result<Vec<company_organization::BusinessUnit>, Box<dyn std::error::Error + Send + Sync>>
    {
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT id, name, currency, cash_minor::text, revenue_minor::text,
                        variable_cost_minor::text, fixed_cost_minor::text,
                        budget_minor::text, lifecycle
                   FROM business_units
                  WHERE company_id=$1
                  ORDER BY name ASC, id ASC",
                &[&company_uuid],
            )
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(company_organization::BusinessUnit {
                    id: row.get::<_, Uuid>(0).to_string(),
                    name: row.get(1),
                    currency: row.get(2),
                    cash_minor: parse_i128_numeric(&row.get::<_, String>(3))?,
                    revenue_minor: parse_i128_numeric(&row.get::<_, String>(4))?,
                    variable_cost_minor: parse_i128_numeric(&row.get::<_, String>(5))?,
                    fixed_cost_minor: parse_i128_numeric(&row.get::<_, String>(6))?,
                    budget_minor: parse_i128_numeric(&row.get::<_, String>(7))?,
                    lifecycle: parse_business_unit_lifecycle(&row.get::<_, String>(8))?,
                })
            })
            .collect()
    }

    pub async fn portfolio_metrics(
        &self,
        company_id: &str,
    ) -> Result<company_organization::PortfolioMetrics, Box<dyn std::error::Error + Send + Sync>>
    {
        let units = self.list_business_units(company_id).await?;
        company_organization::summarize_portfolio(&units)
            .map_err(|error| error.to_string().into())
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
        tx.execute(
            "UPDATE scheduled_jobs
                SET locked_until = now() + interval '5 minutes',
                    updated_at = now()
              WHERE id = $1",
            &[&job_id],
        )
        .await?;
        tx.commit().await?;
        Ok(Some((job_id, run_token)))
    }

    pub async fn complete_job(
        &self,
        job_id: Uuid,
        next_run_token: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.lock().await;
        client
            .execute(
                "UPDATE scheduled_jobs
                    SET next_run_at = now() +
                        make_interval(secs => interval_seconds),
                        locked_until = NULL,
                        run_token = $2,
                        updated_at = now()
                  WHERE id = $1",
                &[&job_id, &next_run_token],
            )
            .await?;
        Ok(())
    }

    pub async fn release_job_after_failure(
        &self,
        job_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.lock().await;
        client
            .execute(
                "UPDATE scheduled_jobs
                    SET next_run_at = now() + interval '30 seconds',
                        locked_until = NULL,
                        updated_at = now()
                  WHERE id = $1",
                &[&job_id],
            )
            .await?;
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
        let mut result = std::collections::HashMap::new();
        for agent in AgentRole::ALL {
            if agent == AgentRole::Governor {
                continue;
            }
            let memory = self
                .load_agent_memory(company_id, agent, limit_per_agent)
                .await?;
            result.insert(agent, memory);
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
        if agent == AgentRole::Governor {
            return Err("Governor cannot persist operational agent memory".into());
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
        if event.company_id.trim().is_empty()
            || event.click_id.trim().is_empty()
            || event.product_id.trim().is_empty()
            || event.advertiser_id.trim().is_empty()
            || event.content_id.trim().is_empty()
            || event.occurred_at.trim().is_empty()
        {
            return Err("affiliate click has incomplete identifiers".into());
        }
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
    ) -> Result<
        affiliate_attribution::ReconciledConversion,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let company_id = Uuid::parse_str(&event.company_id)?;
        let currency = self.company_currency(&company_id).await?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let idempotency_key = affiliate_attribution::conversion_idempotency_key(event);

        if let Some(row) = tx
            .query_opt(
                "SELECT conversion_id, commission_minor::text,
                        reconciliation_status,
                        reconciliation_variance_minor::text
                   FROM affiliate_conversions
                  WHERE company_id=$1 AND idempotency_key=$2",
                &[&company_id, &idempotency_key],
            )
            .await?
        {
            let conversion_id: String = row.get(0);
            let status = parse_reconciliation_status(&row.get::<_, String>(2))?;
            let variance = parse_i128_numeric(&row.get::<_, String>(3))?;
            let attributed = load_affiliate_attributions(
                &tx,
                company_id,
                &conversion_id,
            )
            .await?;
            let net_commission = parse_i128_numeric(&row.get::<_, String>(1))?;
            tx.rollback().await?;
            return Ok(affiliate_attribution::ReconciledConversion {
                conversion_id,
                attributed,
                net_commission_minor: net_commission,
                reconciliation_variance_minor: variance,
                status,
                idempotency_key,
            });
        }

        let rows = tx
            .query(
                "SELECT click_id, company_id, product_id, advertiser_id, content_id,
                        occurred_at, source
                   FROM affiliate_clicks
                  WHERE company_id=$1 AND product_id=$2 AND advertiser_id=$3
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

        let reconciled = affiliate_attribution::attribute_conversion(event, &clicks, model)
            .map_err(|error| error.to_string())?;

        let order_value = event.order_value_minor.to_string();
        let commission = event.commission_minor.to_string();
        let refunded = event.refunded_minor.to_string();

        tx.execute(
            "INSERT INTO affiliate_conversions
             (company_id, conversion_id, click_id, order_id, product_id,
              advertiser_id, occurred_at, order_value_minor, commission_minor,
              refunded_minor, cancelled, source, idempotency_key,
              reconciliation_status, reconciliation_variance_minor)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8::numeric,$9::numeric,$10::numeric,
                     $11,$12,$13,$14,$15::numeric)",
            &[
                &company_id,
                &event.conversion_id,
                &event.click_id,
                &event.order_id,
                &event.product_id,
                &event.advertiser_id,
                &event.occurred_at,
                &order_value,
                &commission,
                &refunded,
                &event.cancelled,
                &event.source,
                &idempotency_key,
                &format!("{:?}", reconciled.status).to_ascii_uppercase(),
                &reconciled.reconciliation_variance_minor.to_string(),
            ],
        )
        .await?;

        for attribution in &reconciled.attributed {
            let value = attribution.attributed_order_value_minor.to_string();
            let attributed_commission = attribution.attributed_commission_minor.to_string();
            tx.execute(
                "INSERT INTO affiliate_attributions
                 (company_id, conversion_id, click_id, product_id, content_id,
                  attributed_order_value_minor, attributed_commission_minor,
                  confidence_bps)
                 VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8)
                 ON CONFLICT (company_id, conversion_id, click_id) DO NOTHING",
                &[
                    &company_id,
                    &event.conversion_id,
                    &attribution.click_id,
                    &attribution.product_id,
                    &attribution.content_id,
                    &value,
                    &attributed_commission,
                    &(attribution.confidence_bps as i32),
                ],
            )
            .await?;
        }

        let target_recognized = if matches!(
            reconciled.status,
            affiliate_attribution::ReconciliationStatus::Verified
        ) {
            reconciled.net_commission_minor
        } else {
            0
        };

        let prior = tx
            .query_opt(
                "SELECT recognized_minor::text, ledger_transaction_id
                   FROM affiliate_revenue_recognition
                  WHERE company_id=$1 AND conversion_id=$2
                  FOR UPDATE",
                &[&company_id, &event.conversion_id],
            )
            .await?;

        let prior_recognized = prior
            .as_ref()
            .map(|row| parse_i128_numeric(&row.get::<_, String>(0)))
            .transpose()?
            .unwrap_or(0);

        let delta = target_recognized
            .checked_sub(prior_recognized)
            .ok_or("affiliate revenue recognition delta overflow")?;

        let mut ledger_transaction_id = prior
            .as_ref()
            .and_then(|row| row.get::<_, Option<Uuid>>(1));

        if delta != 0 {
            let (_cash_account, receivable_account, revenue_account) =
                ensure_affiliate_accounts(&tx, company_id, &currency).await?;

            let transaction_id = Uuid::new_v4();
            ledger_transaction_id = Some(transaction_id);
            let amount = delta.unsigned_abs().to_string();

            tx.execute(
                "INSERT INTO ledger_transactions
                 (id, company_id, description, idempotency_key)
                 VALUES ($1,$2,'affiliate revenue recognition',$3)",
                &[
                    &transaction_id,
                    &company_id,
                    &format!("affiliate:recognition:{}", event.conversion_id),
                ],
            )
            .await?;

            if delta > 0 {
                insert_ledger_entry(
                    &tx,
                    transaction_id,
                    receivable_account,
                    &amount,
                    "0",
                    &currency,
                )
                .await?;
                insert_ledger_entry(
                    &tx,
                    transaction_id,
                    revenue_account,
                    "0",
                    &amount,
                    &currency,
                )
                .await?;
            } else {
                insert_ledger_entry(
                    &tx,
                    transaction_id,
                    revenue_account,
                    &amount,
                    "0",
                    &currency,
                )
                .await?;
                insert_ledger_entry(
                    &tx,
                    transaction_id,
                    receivable_account,
                    "0",
                    &amount,
                    &currency,
                )
                .await?;
            }

            update_snapshot_financials(&tx, company_id, |snapshot| {
                if delta > 0 {
                    snapshot.revenue_minor = snapshot
                        .revenue_minor
                        .checked_add(delta)
                        .ok_or("affiliate revenue overflow".to_string())?;
                    snapshot.assets_minor = snapshot
                        .assets_minor
                        .checked_add(delta)
                        .ok_or("affiliate receivable asset overflow".to_string())?;
                } else {
                    let decrease = delta
                        .checked_neg()
                        .ok_or("affiliate revenue adjustment overflow".to_string())?;
                    snapshot.revenue_minor = snapshot
                        .revenue_minor
                        .checked_sub(decrease)
                        .ok_or("affiliate revenue underflow".to_string())?;
                    snapshot.assets_minor = snapshot
                        .assets_minor
                        .checked_sub(decrease)
                        .ok_or("affiliate receivable asset underflow".to_string())?;
                }
                Ok(())
            })
            .await?;
        }

        tx.execute(
            "INSERT INTO affiliate_revenue_recognition
             (company_id, conversion_id, currency, recognized_minor, status,
              ledger_transaction_id)
             VALUES ($1,$2,$3,$4::numeric,$5,$6)
             ON CONFLICT (company_id, conversion_id)
             DO UPDATE SET recognized_minor=EXCLUDED.recognized_minor,
                           currency=EXCLUDED.currency,
                           status=EXCLUDED.status,
                           ledger_transaction_id=EXCLUDED.ledger_transaction_id,
                           updated_at=now()",
            &[
                &company_id,
                &event.conversion_id,
                &currency,
                &target_recognized.to_string(),
                &if target_recognized > 0 { "RECOGNIZED" } else { "PENDING" },
                &ledger_transaction_id,
            ],
        )
        .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
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
        company_id: &str,
        payout_id: &str,
        amount_minor: i128,
        currency: &str,
        occurred_at: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if payout_id.trim().is_empty() || amount_minor <= 0 {
            return Err("affiliate payout requires a positive amount and payout id".into());
        }
        let occurred = time::OffsetDateTime::parse(
            occurred_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|error| format!("invalid affiliate payout timestamp: {error}"))?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let company_currency = self.company_currency(&company_uuid).await?;
        if currency != company_currency {
            return Err("affiliate payout currency must match company currency".into());
        }

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let exists = tx
            .query_opt(
                "SELECT 1 FROM affiliate_payouts
                  WHERE company_id=$1 AND payout_id=$2
                  FOR UPDATE",
                &[&company_uuid, &payout_id],
            )
            .await?;
        if exists.is_some() {
            tx.rollback().await?;
            return Ok(());
        }

        let recognized = parse_i128_numeric(
            &tx
                .query_one(
                    "SELECT COALESCE(SUM(recognized_minor),0)::text
                       FROM affiliate_revenue_recognition
                      WHERE company_id=$1 AND status='RECOGNIZED'",
                    &[&company_uuid],
                )
                .await?
                .get::<_, String>(0),
        )?;
        let paid = parse_i128_numeric(
            &tx
                .query_one(
                    "SELECT COALESCE(SUM(amount_minor),0)::text
                       FROM affiliate_payouts
                      WHERE company_id=$1",
                    &[&company_uuid],
                )
                .await?
                .get::<_, String>(0),
        )?;
        let available = recognized
            .checked_sub(paid)
            .ok_or("affiliate receivable balance overflow")?;
        if amount_minor > available {
            return Err("affiliate payout exceeds recognized receivable".into());
        }

        let (cash_account, receivable_account, _) =
            ensure_affiliate_accounts(&tx, company_uuid, &company_currency).await?;
        let payout_uuid = Uuid::new_v4();
        let amount = amount_minor.to_string();
        tx.execute(
            "INSERT INTO ledger_transactions
             (id, company_id, description, idempotency_key)
             VALUES ($1,$2,'affiliate payout settlement',$3)",
            &[&payout_uuid, &company_uuid, &format!("affiliate:payout:{payout_id}")],
        )
        .await?;
        insert_ledger_entry(
            &tx,
            payout_uuid,
            cash_account,
            &amount,
            "0",
            &company_currency,
        )
        .await?;
        insert_ledger_entry(
            &tx,
            payout_uuid,
            receivable_account,
            "0",
            &amount,
            &company_currency,
        )
        .await?;

        let occurred_text = occurred.format(&time::format_description::well_known::Rfc3339)?;
        tx.execute(
            "INSERT INTO affiliate_payouts
             (company_id,payout_id,currency,amount_minor,occurred_at,ledger_transaction_id)
             VALUES ($1,$2,$3,$4::numeric,$5,$6)",
            &[&company_uuid,&payout_id,&currency,&amount,&occurred_text,&payout_uuid],
        )
        .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'AFFILIATE_PAYOUT_SETTLED',$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[
                &company_uuid,
                &payout_id,
                &format!("outbox:affiliate:payout:{payout_id}"),
                &serde_json::json!({
                    "payout_id": payout_id,
                    "amount_minor": amount_minor,
                    "currency": currency,
                    "occurred_at": occurred_text,
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
            width: row.get::<_, i32>(4).max(16) as u32,
            height: row.get::<_, i32>(5).max(16) as u32,
            fps: row.get::<_, i32>(6).max(1) as u32,
            max_duration_seconds: row.get::<_, i32>(7).max(1) as u32,
            normalize_audio: row.get(8),
        };

        tx.execute(
            "UPDATE media_jobs
                SET status='RUNNING', attempts=attempts+1,
                    locked_until=now()+interval '15 minutes',
                    updated_at=now()
              WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        Ok(Some(job))
    }

    pub async fn finish_media_job(
        &self,
        job_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !matches!(status, "SUCCEEDED" | "FAILED" | "QA_FAILED") {
            return Err("invalid media completion status".into());
        }
        let id = Uuid::parse_str(job_id)?;
        let bounded_error = error.map(|value| value.chars().take(4096).collect::<String>());
        let client = self.client.lock().await;
        client
            .execute(
                "UPDATE media_jobs
                    SET status = CASE
                          WHEN $2 = 'FAILED' AND attempts < 3 THEN 'QUEUED'
                          ELSE $2
                        END,
                        locked_until = NULL,
                        last_error = $3,
                        updated_at = now()
                  WHERE id = $1",
                &[&id, &status, &bounded_error],
            )
            .await?;
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

fn parse_reconciliation_status(
    value: &str,
) -> Result<affiliate_attribution::ReconciliationStatus, Box<dyn std::error::Error + Send + Sync>> {
    match value.to_ascii_uppercase().as_str() {
        "VERIFIED" => Ok(affiliate_attribution::ReconciliationStatus::Verified),
        "PARTIAL" => Ok(affiliate_attribution::ReconciliationStatus::Partial),
        "REJECTED" => Ok(affiliate_attribution::ReconciliationStatus::Rejected),
        other => Err(format!("unknown affiliate reconciliation status: {other}").into()),
    }
}

fn parse_business_unit_lifecycle(
    value: &str,
) -> Result<company_organization::BusinessUnitLifecycle, Box<dyn std::error::Error + Send + Sync>> {
    match value {
        "TESTING" => Ok(company_organization::BusinessUnitLifecycle::Testing),
        "GROWING" => Ok(company_organization::BusinessUnitLifecycle::Growing),
        "STABLE" => Ok(company_organization::BusinessUnitLifecycle::Stable),
        "DISTRESS" => Ok(company_organization::BusinessUnitLifecycle::Distress),
        "PAUSED" => Ok(company_organization::BusinessUnitLifecycle::Paused),
        "CLOSED" => Ok(company_organization::BusinessUnitLifecycle::Closed),
        other => Err(format!("unknown business unit lifecycle: {other}").into()),
    }
}

fn parse_employee_status(
    value: &str,
) -> Result<company_organization::EmployeeStatus, Box<dyn std::error::Error + Send + Sync>> {
    match value {
        "PROPOSED" => Ok(company_organization::EmployeeStatus::Proposed),
        "ACTIVE" => Ok(company_organization::EmployeeStatus::Active),
        "SUSPENDED" => Ok(company_organization::EmployeeStatus::Suspended),
        "TERMINATED" => Ok(company_organization::EmployeeStatus::Terminated),
        other => Err(format!("unknown employee status: {other}").into()),
    }
}

async fn load_affiliate_attributions(
    tx: &Transaction<'_>,
    company_id: Uuid,
    conversion_id: &str,
) -> Result<Vec<affiliate_attribution::Attribution>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = tx
        .query(
            "SELECT click_id, product_id, content_id,
                    attributed_order_value_minor::text,
                    attributed_commission_minor::text,
                    confidence_bps
               FROM affiliate_attributions
              WHERE company_id=$1 AND conversion_id=$2
              ORDER BY click_id ASC",
            &[&company_id, &conversion_id],
        )
        .await?;
    rows.into_iter()
        .map(|row| {
            Ok(affiliate_attribution::Attribution {
                click_id: row.get(0),
                product_id: row.get(1),
                content_id: row.get(2),
                attributed_order_value_minor: parse_i128_numeric(
                    &row.get::<_, String>(3),
                )?,
                attributed_commission_minor: parse_i128_numeric(
                    &row.get::<_, String>(4),
                )?,
                confidence_bps: row.get::<_, i32>(5).clamp(0, 10_000) as u32,
            })
        })
        .collect()
}

async fn ensure_affiliate_accounts(
    tx: &Transaction<'_>,
    company_id: Uuid,
    currency: &str,
) -> Result<(Uuid, Uuid, Uuid), Box<dyn std::error::Error + Send + Sync>> {
    let cash = ensure_ledger_account(tx, company_id, "CASH", "Cash", "ASSET", currency).await?;
    let receivable = ensure_ledger_account(
        tx,
        company_id,
        "AFFILIATE_RECEIVABLE",
        "Affiliate Receivable",
        "ASSET",
        currency,
    )
    .await?;
    let revenue = ensure_ledger_account(
        tx,
        company_id,
        "AFFILIATE_REVENUE",
        "Affiliate Revenue",
        "REVENUE",
        currency,
    )
    .await?;
    Ok((cash, receivable, revenue))
}

async fn ensure_payroll_accounts(
    tx: &Transaction<'_>,
    company_id: Uuid,
    currency: &str,
) -> Result<(Uuid, Uuid, Uuid), Box<dyn std::error::Error + Send + Sync>> {
    let cash = ensure_ledger_account(tx, company_id, "CASH", "Cash", "ASSET", currency).await?;
    let payroll_liability =
        ensure_ledger_account(tx, company_id, "PAYROLL_LIABILITY", "Payroll Liability", "LIABILITY", currency).await?;
    let payroll_expense = ensure_ledger_account(
        tx,
        company_id,
        "PAYROLL_EXPENSE",
        "Payroll Expense",
        "EXPENSE",
        currency,
    )
    .await?;
    Ok((cash, payroll_liability, payroll_expense))
}

async fn ensure_ledger_account(
    tx: &Transaction<'_>,
    company_id: Uuid,
    code: &str,
    name: &str,
    account_type: &str,
    currency: &str,
) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
    let id = Uuid::new_v4();
    let row = tx
        .query_one(
            "INSERT INTO ledger_accounts
             (id, company_id, code, name, account_type, currency)
             VALUES ($1,$2,$3,$4,$5,$6)
             ON CONFLICT (company_id, code)
             DO UPDATE SET name=EXCLUDED.name
             RETURNING id",
            &[&id, &company_id, &code, &name, &account_type, &currency],
        )
        .await?;
    Ok(row.get(0))
}

async fn insert_ledger_entry(
    tx: &Transaction<'_>,
    transaction_id: Uuid,
    account_id: Uuid,
    debit: &str,
    credit: &str,
    currency: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tx.execute(
        "INSERT INTO ledger_entries
         (transaction_id, account_id, debit_minor, credit_minor, currency)
         VALUES ($1,$2,$3::numeric,$4::numeric,$5)",
        &[&transaction_id, &account_id, &debit, &credit, &currency],
    )
    .await?;
    Ok(())
}

async fn authoritative_snapshot(
    tx: &Transaction<'_>,
    company_id: Uuid,
) -> Result<CompanySnapshot, Box<dyn std::error::Error + Send + Sync>> {
    let row = tx
        .query_one(
            "SELECT state FROM company_state_snapshots WHERE company_id=$1 FOR UPDATE",
            &[&company_id],
        )
        .await?;
    Ok(serde_json::from_value(row.get(0))?)
}

async fn update_snapshot_financials<F>(
    tx: &Transaction<'_>,
    company_id: Uuid,
    update: F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce(&mut CompanySnapshot) -> Result<(), String>,
{
    let mut snapshot = authoritative_snapshot(tx, company_id).await?;
    update(&mut snapshot).map_err(|error| error.into())?;
    let state = serde_json::to_value(&snapshot)?;
    tx.execute(
        "UPDATE company_state_snapshots
            SET state=$2, updated_at=now()
          WHERE company_id=$1",
        &[&company_id, &state],
    )
    .await?;
    update_company_status(tx, company_id, &snapshot).await?;
    Ok(())
}

async fn upsert_agent_memory_tx(
    tx: &Transaction<'_>,
    company_id: Uuid,
    agent: AgentRole,
    key: &str,
    value: &serde_json::Value,
    confidence_bps: u16,
    importance: u8,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if agent == AgentRole::Governor {
        return Err("Governor cannot persist operational agent memory".into());
    }
    if key.is_empty() || key.len() > 128 || confidence_bps > 10_000 || importance > 100 {
        return Err("invalid agent memory metadata".into());
    }
    let encoded = serde_json::to_vec(value)?;
    if encoded.len() > 16 * 1024 {
        return Err("agent memory value exceeds 16 KiB".into());
    }
    tx.execute(
        "INSERT INTO agent_memory
         (company_id, agent_name, memory_key, value, confidence_bps, importance)
         VALUES ($1,$2,$3,$4,$5,$6)
         ON CONFLICT (company_id, agent_name, memory_key)
         DO UPDATE SET value=EXCLUDED.value,
                       confidence_bps=EXCLUDED.confidence_bps,
                       importance=EXCLUDED.importance,
                       expires_at=NULL",
        &[
            &company_id,
            &agent.as_str(),
            &key,
            &value,
            &(confidence_bps as i32),
            &(importance as i16),
        ],
    )
    .await?;
    Ok(())
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
