#![forbid(unsafe_code)]

use agent_runtime::{types::AgentRole, AgentRunResult, CompanySnapshot};
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
pub struct RevenuePeriodMetrics {
    pub month_to_date_minor: i128,
    pub last_30_days_minor: i128,
    pub lifetime_minor: i128,
    pub forecast_month_minor: i128,
    pub run_rate_month_minor: i128,
    pub forecast_confidence_bps: u32,
    pub revenue_transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExperimentRecord {
    pub id: Uuid,
    pub company_id: Uuid,
    pub spec: company_experiments::ExperimentSpec,
    pub status: company_experiments::ExperimentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentRecord {
    pub item: company_content::ContentItem,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentObservationRecord {
    pub id: Uuid,
    pub observation: company_content::ContentObservation,
    pub decision: company_content::ContentDecision,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrowthTrendRecord {
    pub id: Uuid,
    pub signal: company_growth::TrendSignal,
    pub score_bps: u32,
    pub decision: company_growth::TrendDecision,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomySimulationRecord {
    pub id: Uuid,
    pub company_id: Uuid,
    pub idempotency_key: String,
    pub proposal: agent_runtime::types::Proposal,
    pub assessment: company_autonomy::AutonomyAssessment,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentOutcomeEvidenceRecord {
    pub id: Uuid,
    pub company_id: Uuid,
    pub decision_journal_id: i64,
    pub agent_name: String,
    pub action: String,
    pub evidence_ref: String,
    pub observed_revenue_delta_minor: i128,
    pub observed_contribution_margin_delta_minor: i128,
    pub observed_at_epoch: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CeoCommandCenterRecord {
    pub input: company_command_center::CommandCenterInput,
    pub summary: company_command_center::CommandCenterSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrowthOpportunityRecord {
    pub opportunity: company_growth::Opportunity,
    pub status: company_growth::OpportunityStatus,
    pub content_item_id: Option<Uuid>,
    pub content_created_at_epoch: Option<i64>,
    pub ttfc_seconds: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyControlRecord {
    pub controls: company_safety_controls::SafetyControls,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevenueGraphSummary {
    pub edge_count: i64,
    pub value_backed_edge_count: i64,
    pub latest_observed_at_epoch: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TikTokConnectionRecord {
    pub company_id: Uuid,
    pub open_id: String,
    pub scopes: String,
    pub token_type: String,
    pub access_token_expires_at_epoch: i64,
    pub refresh_token_expires_at_epoch: i64,
    pub status: String,
    pub last_error: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct TikTokTokenMaterial {
    pub company_id: Uuid,
    pub open_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at_epoch: i64,
    pub refresh_token_expires_at_epoch: i64,
    pub scopes: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalAllocationRecord {
    pub plan: company_capital::CapitalAllocationPlan,
    pub policy: company_capital::CapitalPolicy,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AffiliateReconciliationMetrics {
    pub reported_commission_mtd_minor: i128,
    pub attributed_commission_mtd_minor: i128,
    pub recorded_payout_mtd_minor: i128,
    pub variance_mtd_minor: i128,
    pub reported_attributed_variance_mtd_minor: i128,
    pub attributed_paid_variance_mtd_minor: i128,
    pub reported_paid_variance_mtd_minor: i128,
    pub conversion_count_mtd: i64,
    pub verified_conversion_count_mtd: i64,
    pub partial_or_rejected_count_mtd: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributionMarginMetrics {
    pub month_to_date_revenue_minor: i128,
    pub month_to_date_variable_cost_minor: i128,
    pub month_to_date_contribution_margin_minor: Option<i128>,
    pub platform_fees_minor: i128,
    pub affiliate_commission_minor: i128,
    pub refunds_cancellations_minor: i128,
    pub production_ai_cost_minor: i128,
    pub ad_spend_minor: i128,
    pub operating_cost_minor: i128,
    pub cash_minor: i128,
    pub unclassified_expense_minor: i128,
    pub unclassified_expense_entry_count: i64,
    pub variable_cost_transaction_count: i64,
}

#[derive(Debug, Clone)]
pub struct OutboxEvent {
    pub id: i64,
    pub company_id: String,
    pub event_type: String,
    pub aggregate_id: Option<String>,
    pub schema_version: i32,
    pub payload: serde_json::Value,
    pub attempts: i32,
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
                "../../../infra/db/migrations/002_company_control.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/003_ledger_completeness.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/004_affiliate_searches.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/005_durable_scheduler.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/006_company_operations.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/007_affiliate_attribution.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/008_commercial_and_payroll.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/009_outbox_leases.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/010_media_jobs.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/011_llm_web_relay.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/012_runtime_controls.sql"
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
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/009_llm_web_relay.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/010_llm_relay_request_fingerprint.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/011_affiliate_provider_verification.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/012_publish_intents.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/013_commercial_sales.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/014_customer_crm.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/015_outbound_messages.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/015_tiktok_webhook_receipts.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/017_payment_reconciliation_evidence.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/018_outbound_messages.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/019_control_plane_rbac_audit.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/020_customer_success_tasks.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/021_procurement_vendor_lifecycle.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/022_fpa_forecasts_cashflow.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/022_tiktok_live.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/023_ledger_cost_class.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/024_growth_experiments.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/025_legal_compliance.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/026_recurring_revenue.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/027_payment_execution.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/028_fpa_variance_alerts.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/029_customer_support.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/030_content_factory.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/031_content_performance.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/032_content_status_evidence.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/033_growth_loop.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/034_live_attention.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/035_policy_intelligence.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/036_agent_outcome_evaluation.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/037_autonomy_simulations.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/038_capital_allocation.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/039_autonomy_controls.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/040_tiktok_oauth.sql"
            ))
            .await?;
        client
            .batch_execute(include_str!(
                "../../../infra/db/migrations/041_revenue_intelligence_graph.sql"
            ))
            .await
    }

    pub async fn record_revenue_graph_edge(
        &self,
        edge: &company_revenue_graph::RevenueGraphEdge,
    ) -> Result<company_revenue_graph::RevenueGraphEdge, Box<dyn std::error::Error + Send + Sync>> {
        company_revenue_graph::validate_edge(edge).map_err(|error| error.to_string())?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let stored = record_revenue_graph_edge_tx(&tx, edge).await?;
        tx.commit().await?;
        Ok(stored)
    }

    pub async fn revenue_graph_lineage(
        &self,
        company_id: &str,
        root_type: company_revenue_graph::RevenueNodeType,
        root_ref: &str,
        max_depth: i32,
        limit: i64,
    ) -> Result<Vec<(i32, company_revenue_graph::RevenueGraphEdge)>, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        if root_ref.trim().is_empty() || root_ref.len() > 512 {
            return Err("revenue graph root_ref is invalid".into());
        }
        if !(0..=12).contains(&max_depth) || !(1..=500).contains(&limit) {
            return Err("revenue graph depth/limit is outside safe bounds".into());
        }
        let client = self.client.lock().await;
        let rows = client.query(
            "WITH RECURSIVE walk AS (
                SELECT e.id,e.company_id,e.edge_key,e.from_type,e.from_ref,e.relation,
                       e.to_type,e.to_ref,e.value_minor::text,e.currency,e.confidence_bps,
                       e.evidence_ref,e.source,e.observed_at_epoch,e.created_at::text,
                       0::int AS depth,
                       ARRAY[(e.from_type || ':' || e.from_ref),(e.to_type || ':' || e.to_ref)] AS visited
                  FROM revenue_graph_edges e
                 WHERE e.company_id=$1
                   AND e.from_type=$2
                   AND e.from_ref=$3
                UNION ALL
                SELECT e.id,e.company_id,e.edge_key,e.from_type,e.from_ref,e.relation,
                       e.to_type,e.to_ref,e.value_minor::text,e.currency,e.confidence_bps,
                       e.evidence_ref,e.source,e.observed_at_epoch,e.created_at::text,
                       w.depth + 1,
                       w.visited || (e.to_type || ':' || e.to_ref)
                  FROM walk w
                  JOIN revenue_graph_edges e
                    ON e.company_id=w.company_id
                   AND e.from_type=w.to_type
                   AND e.from_ref=w.to_ref
                 WHERE w.depth < $4
                   AND NOT ((e.to_type || ':' || e.to_ref) = ANY(w.visited))
            )
            SELECT id,company_id,edge_key,from_type,from_ref,relation,to_type,to_ref,
                   value_minor,currency,confidence_bps,evidence_ref,source,observed_at_epoch,
                   created_at,depth
              FROM walk
             ORDER BY depth ASC,observed_at_epoch DESC,created_at DESC
             LIMIT $5",
            &[
                &company,
                &company_revenue_graph::RevenueNodeType::as_str(root_type),
                &root_ref.trim(),
                &max_depth,
                &limit,
            ],
        ).await?;
        rows.into_iter()
            .map(|row| {
                Ok((
                    row.get::<_, i32>(15),
                    revenue_graph_edge_from_row(row)?,
                ))
            })
            .collect()
    }

    pub async fn revenue_graph_summary(
        &self,
        company_id: &str,
    ) -> Result<RevenueGraphSummary, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client.query_one(
            "SELECT COUNT(*)::bigint,
                    COUNT(*) FILTER (WHERE value_minor IS NOT NULL)::bigint,
                    MAX(observed_at_epoch)
               FROM revenue_graph_edges
              WHERE company_id=$1",
            &[&company],
        ).await?;
        Ok(RevenueGraphSummary {
            edge_count: row.get(0),
            value_backed_edge_count: row.get(1),
            latest_observed_at_epoch: row.get(2),
        })
    }

    pub async fn record_revenue_graph_edge(
        &self,
        edge: &company_revenue_graph::RevenueGraphEdge,
    ) -> Result<company_revenue_graph::RevenueGraphEdge, Box<dyn std::error::Error + Send + Sync>> {
        company_revenue_graph::validate_edge(edge).map_err(|error| error.to_string())?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let stored = record_revenue_graph_edge_tx(&tx, edge).await?;
        tx.commit().await?;
        Ok(stored)
    }

    pub async fn revenue_graph_lineage(
        &self,
        company_id: &str,
        root_type: company_revenue_graph::RevenueNodeType,
        root_ref: &str,
        max_depth: i32,
        limit: i64,
    ) -> Result<Vec<(i32, company_revenue_graph::RevenueGraphEdge)>, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        if root_ref.trim().is_empty() || root_ref.len() > 512 {
            return Err("revenue graph root_ref is invalid".into());
        }
        if !(0..=12).contains(&max_depth) || !(1..=500).contains(&limit) {
            return Err("revenue graph depth/limit is outside safe bounds".into());
        }
        let root_type = root_type.as_str();
        let client = self.client.lock().await;
        let rows = client.query(
            "WITH RECURSIVE walk AS (
                SELECT e.id,e.company_id,e.edge_key,e.from_type,e.from_ref,e.relation,
                       e.to_type,e.to_ref,e.value_minor::text,e.currency,e.confidence_bps,
                       e.evidence_ref,e.source,e.observed_at_epoch,e.created_at::text,
                       0::int AS depth,
                       ARRAY[(e.from_type || ':' || e.from_ref),(e.to_type || ':' || e.to_ref)] AS visited
                  FROM revenue_graph_edges e
                 WHERE e.company_id=$1
                   AND e.from_type=$2
                   AND e.from_ref=$3
                UNION ALL
                SELECT e.id,e.company_id,e.edge_key,e.from_type,e.from_ref,e.relation,
                       e.to_type,e.to_ref,e.value_minor::text,e.currency,e.confidence_bps,
                       e.evidence_ref,e.source,e.observed_at_epoch,e.created_at::text,
                       w.depth + 1,
                       w.visited || (e.to_type || ':' || e.to_ref)
                  FROM walk w
                  JOIN revenue_graph_edges e
                    ON e.company_id=w.company_id
                   AND e.from_type=w.to_type
                   AND e.from_ref=w.to_ref
                 WHERE w.depth < $4
                   AND NOT ((e.to_type || ':' || e.to_ref) = ANY(w.visited))
            )
            SELECT id,company_id,edge_key,from_type,from_ref,relation,to_type,to_ref,
                   value_minor,currency,confidence_bps,evidence_ref,source,observed_at_epoch,
                   created_at,depth
              FROM walk
             ORDER BY depth ASC,observed_at_epoch DESC,created_at DESC
             LIMIT $5",
            &[&company, &root_type, &root_ref.trim(), &max_depth, &limit],
        ).await?;
        rows.into_iter()
            .map(|row| {
                Ok((
                    row.get::<_, i32>(15),
                    revenue_graph_edge_from_row(row)?,
                ))
            })
            .collect()
    }

    pub async fn revenue_graph_summary(
        &self,
        company_id: &str,
    ) -> Result<RevenueGraphSummary, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client.query_one(
            "SELECT COUNT(*)::bigint,
                    COUNT(*) FILTER (WHERE value_minor IS NOT NULL)::bigint,
                    MAX(observed_at_epoch)
               FROM revenue_graph_edges
              WHERE company_id=$1",
            &[&company],
        ).await?;
        Ok(RevenueGraphSummary {
            edge_count: row.get(0),
            value_backed_edge_count: row.get(1),
            latest_observed_at_epoch: row.get(2),
        })
    }

    pub async fn record_policy_snapshot(
        &self,
        snapshot: &company_compliance::PolicySnapshot,
    ) -> Result<company_compliance::PolicySnapshot, Box<dyn std::error::Error + Send + Sync>> {
        snapshot.validate().map_err(|error| error.to_string())?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let inserted = tx.query_opt(
            "INSERT INTO policy_snapshots
             (id,company_id,policy_key,platform,jurisdiction,version,source_reference,
              evidence_hash,observed_at_epoch,effective_at_epoch,active,rules_json)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
             ON CONFLICT(company_id,policy_key,version) DO NOTHING
             RETURNING id",
            &[
                &snapshot.id,
                &snapshot.company_id,
                &snapshot.policy_key,
                &snapshot.platform,
                &snapshot.jurisdiction,
                &snapshot.version,
                &snapshot.source_reference,
                &snapshot.evidence_hash,
                &snapshot.observed_at_epoch,
                &snapshot.effective_at_epoch,
                &snapshot.active,
                &serde_json::to_value(&snapshot.rules)?,
            ],
        ).await?;

        let row = tx
            .query_one(
                "SELECT id,company_id,policy_key,platform,jurisdiction,version,
                        source_reference,evidence_hash,observed_at_epoch,effective_at_epoch,
                        active,rules_json
                   FROM policy_snapshots
                  WHERE company_id=$1 AND policy_key=$2 AND version=$3
                  FOR UPDATE",
                &[&snapshot.company_id, &snapshot.policy_key, &snapshot.version],
            )
            .await?;
        let stored = policy_snapshot_from_row(row)?;
        if stored != *snapshot {
            return Err("policy snapshot version already exists with different evidence".into());
        }

        if snapshot.active {
            let existing_effective = tx
                .query_opt(
                    "SELECT effective_at_epoch
                       FROM policy_snapshots
                      WHERE company_id=$1 AND policy_key=$2 AND platform=$4 AND jurisdiction=$5 AND active=true AND id<>$3
                      ORDER BY effective_at_epoch DESC
                      LIMIT 1",
                    &[&snapshot.company_id, &snapshot.policy_key, &snapshot.id, &snapshot.platform, &snapshot.jurisdiction],
                )
                .await?
                .map(|row| row.get::<_, i64>(0));

            if existing_effective.is_some_and(|value| value > snapshot.effective_at_epoch) {
                return Err("cannot activate a policy snapshot older than the active policy".into());
            }

            tx.execute(
                "UPDATE policy_snapshots
                    SET active=false
                  WHERE company_id=$1 AND policy_key=$2 AND platform=$4 AND jurisdiction=$5 AND id<>$3",
                &[
                    &snapshot.company_id,
                    &snapshot.policy_key,
                    &snapshot.id,
                    &snapshot.platform,
                    &snapshot.jurisdiction,
                ],
            ).await?;
            if inserted.is_some() {
                tx.execute(
                    "INSERT INTO outbox_events
                     (company_id,event_type,aggregate_id,idempotency_key,payload)
                     VALUES ($1,'POLICY_SNAPSHOT_ACTIVATED',$2,$3,$4)
                     ON CONFLICT(company_id,idempotency_key) DO NOTHING",
                    &[
                        &snapshot.company_id,
                        &snapshot.id.to_string(),
                        &format!("outbox:policy-activated:{}:{}", snapshot.policy_key, snapshot.version),
                        &serde_json::json!({
                            "policy_key": snapshot.policy_key,
                            "platform": snapshot.platform,
                            "jurisdiction": snapshot.jurisdiction,
                            "version": snapshot.version,
                            "effective_at_epoch": snapshot.effective_at_epoch,
                            "evidence_hash": snapshot.evidence_hash,
                        }),
                    ],
                ).await?;
            }
        }

        tx.commit().await?;
        Ok(stored)
    }

    pub async fn record_compliance_check(
        &self,
        input: &company_compliance::ComplianceInput,
    ) -> Result<company_compliance::ComplianceCheck, Box<dyn std::error::Error + Send + Sync>> {
        input.validate().map_err(|error| error.to_string())?;
        let client = self.client.lock().await;
        let now_epoch: i64 = client
            .query_one("SELECT EXTRACT(EPOCH FROM now())::bigint", &[])
            .await?
            .get(0);

        let snapshot = client
            .query_opt(
                "SELECT id,company_id,policy_key,platform,jurisdiction,version,
                        source_reference,evidence_hash,observed_at_epoch,effective_at_epoch,
                        active,rules_json
                   FROM policy_snapshots
                  WHERE company_id=$1 AND policy_key=$2 AND platform=$3
                    AND jurisdiction=$4 AND (policy_key || ':' || version)=$5",
                &[
                    &input.company_id,
                    &input.policy_key,
                    &input.platform,
                    &input.jurisdiction,
                    &input.policy_snapshot_key,
                ],
            )
            .await?
            .map(policy_snapshot_from_row)
            .transpose()?;

        let check = company_compliance::evaluate(snapshot.as_ref(), input, now_epoch)
            .map_err(|error| error.to_string())?;

        let input_hash = {
            let encoded = serde_json::to_vec(input)?;
            let digest = Sha256::digest(encoded);
            format!("sha256:{}", digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>())
        };

        let inserted = client
            .query_opt(
                "INSERT INTO compliance_checks
                 (id,company_id,policy_snapshot_id,surface,policy_key,policy_snapshot_key,
                  input_hash,decision,reason,evidence_ref,requires_human,checked_at_epoch)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
                 ON CONFLICT(company_id,policy_key,policy_snapshot_key,input_hash) DO NOTHING
                 RETURNING id",
                &[
                    &check.id,
                    &check.company_id,
                    &check.policy_snapshot_id,
                    &compliance_surface_name(input.surface),
                    &input.policy_key,
                    &input.policy_snapshot_key,
                    &input_hash,
                    &compliance_decision_name(check.decision),
                    &compliance_reason_name(check.reason),
                    &input.evidence_ref,
                    &check.requires_human,
                    &check.checked_at_epoch,
                ],
            )
            .await?;

        if inserted.is_none() {
            return existing_compliance_check(&client, input, &input_hash)
            .await;
        }

        Ok(check)
    }

    pub async fn compliance_status(
        &self,
        company_id: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let latest = client
            .query_opt(
                "SELECT policy_key,platform,jurisdiction,version,source_reference,
                        evidence_hash,effective_at_epoch,active
                   FROM policy_snapshots
                  WHERE company_id=$1
                  ORDER BY effective_at_epoch DESC,created_at DESC
                  LIMIT 1",
                &[&company],
            )
            .await?;
        let counts = client
            .query_one(
                "SELECT
                    COUNT(*) FILTER (WHERE decision='ALLOWED')::bigint,
                    COUNT(*) FILTER (WHERE decision='REVIEW')::bigint,
                    COUNT(*) FILTER (WHERE decision='BLOCKED')::bigint,
                    COUNT(*) FILTER (WHERE decision='UNKNOWN')::bigint
                 FROM compliance_checks
                WHERE company_id=$1 AND created_at >= now() - interval '24 hours'",
                &[&company],
            )
            .await?;
        Ok(serde_json::json!({
            "latest_policy": latest.map(|row| serde_json::json!({
                "policy_key": row.get::<_,String>(0),
                "platform": row.get::<_,String>(1),
                "jurisdiction": row.get::<_,String>(2),
                "version": row.get::<_,String>(3),
                "source_reference": row.get::<_,String>(4),
                "evidence_hash": row.get::<_,String>(5),
                "effective_at_epoch": row.get::<_,i64>(6),
                "active": row.get::<_,bool>(7)
            })),
            "checks_last_24h": {
                "allowed": counts.get::<_,i64>(0),
                "review": counts.get::<_,i64>(1),
                "blocked": counts.get::<_,i64>(2),
                "unknown": counts.get::<_,i64>(3)
            }
        }))
    }

    pub async fn check_tiktok_compliance_for_publish(
        &self,
        company_id: &str,
        intent_id: &str,
        policy_snapshot_key: &str,
        evidence_ref: &str,
        disclosure_present: bool,
        claim_evidence_present: bool,
        product_eligibility_verified: bool,
        rights_evidence_present: bool,
    ) -> Result<company_compliance::ComplianceCheck, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let intent = Uuid::parse_str(intent_id)?;
        let policy_key = std::env::var("TIKTOK_POLICY_KEY")
            .unwrap_or_else(|_| "TIKTOK_SHOP_VN".into());
        let jurisdiction =
            std::env::var("COMPLIANCE_JURISDICTION").unwrap_or_else(|_| "VN".into());
        let client = self.client.lock().await;
        let row = client
            .query_one(
                "SELECT title,caption
                   FROM publish_intents
                  WHERE company_id=$1 AND id=$2",
                &[&company, &intent],
            )
            .await?;
        let title: String = row.get(0);
        let caption: String = row.get(1);
        let input = company_compliance::ComplianceInput {
            company_id: company,
            surface: company_compliance::ComplianceSurface::Content,
            platform: "TIKTOK_SHOP".into(),
            jurisdiction,
            policy_key,
            policy_snapshot_key: policy_snapshot_key.trim().into(),
            evidence_ref: evidence_ref.trim().into(),
            text: format!("{title}\n{caption}"),
            product_category: None,
            disclosure_present,
            claim_evidence_present,
            product_eligibility_verified,
            simulcast: false,
            fake_engagement_detected: false,
            rights_evidence_present,
        };
        drop(client);
        self.record_compliance_check(&input).await
    }

    pub async fn check_tiktok_compliance_for_live(
        &self,
        company_id: &str,
        title: &str,
        policy_snapshot_key: &str,
        evidence_ref: &str,
        disclosure_present: bool,
        claim_evidence_present: bool,
        product_eligibility_verified: bool,
        rights_evidence_present: bool,
        simulcast: bool,
    ) -> Result<company_compliance::ComplianceCheck, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let policy_key = std::env::var("TIKTOK_POLICY_KEY")
            .unwrap_or_else(|_| "TIKTOK_SHOP_VN".into());
        let jurisdiction =
            std::env::var("COMPLIANCE_JURISDICTION").unwrap_or_else(|_| "VN".into());
        let input = company_compliance::ComplianceInput {
            company_id: company,
            surface: company_compliance::ComplianceSurface::Live,
            platform: "TIKTOK_SHOP".into(),
            jurisdiction,
            policy_key,
            policy_snapshot_key: policy_snapshot_key.trim().into(),
            evidence_ref: evidence_ref.trim().into(),
            text: title.trim().into(),
            product_category: None,
            disclosure_present,
            claim_evidence_present,
            product_eligibility_verified,
            simulcast,
            fake_engagement_detected: false,
            rights_evidence_present,
        };
        self.record_compliance_check(&input).await
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

    pub async fn revenue_period_metrics(
        &self,
        company_id: &str,
    ) -> Result<RevenuePeriodMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_one(
                "SELECT
                    COALESCE(SUM(CASE WHEN t.created_at >= date_trunc('month', now())
                                      THEN e.credit_minor - e.debit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN t.created_at >= now() - interval '30 days'
                                      THEN e.credit_minor - e.debit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(e.credit_minor - e.debit_minor), 0)::text,
                    COUNT(DISTINCT t.id)
                 FROM ledger_transactions t
                 JOIN ledger_entries e ON e.transaction_id = t.id
                 JOIN ledger_accounts a ON a.id = e.account_id
                WHERE t.company_id = $1
                  AND a.company_id = $1
                  AND a.account_type = 'REVENUE'",
                &[&id],
            )
            .await?;

        let month_to_date_minor = parse_i128_numeric(&row.get::<_, String>(0))?;
        let last_30_days_minor = parse_i128_numeric(&row.get::<_, String>(1))?;
        let lifetime_minor = parse_i128_numeric(&row.get::<_, String>(2))?;
        let now = time::OffsetDateTime::now_utc();
        let days_in_month = now.date().month().length(now.year()) as i128;
        let elapsed_days = now.day() as i128;
        let (forecast_month_minor, run_rate_month_minor, forecast_confidence_bps) =
            revenue_period_projection(month_to_date_minor, last_30_days_minor, elapsed_days, days_in_month);

        Ok(RevenuePeriodMetrics {
            month_to_date_minor,
            last_30_days_minor,
            lifetime_minor,
            forecast_month_minor,
            run_rate_month_minor,
            forecast_confidence_bps,
            revenue_transaction_count: row.get(3),
        })
    }

    fn revenue_period_projection(
        month_to_date_minor: i128,
        last_30_days_minor: i128,
        elapsed_days: i128,
        days_in_month: i128,
    ) -> (i128, i128, u32) {
        if days_in_month <= 0 || elapsed_days <= 0 {
            return (month_to_date_minor, last_30_days_minor, 0);
        }
        let elapsed_days = elapsed_days.min(days_in_month);
        let forecast = month_to_date_minor
            .saturating_mul(days_in_month)
            .checked_div(elapsed_days)
            .unwrap_or(month_to_date_minor);
        let run_rate = last_30_days_minor
            .saturating_mul(days_in_month)
            .checked_div(30)
            .unwrap_or(last_30_days_minor);
        let coverage_bps = ((elapsed_days * 10_000) / days_in_month).min(10_000) as u32;
        (forecast, run_rate, coverage_bps)
    }

    pub async fn create_experiment(
        &self,
        company_id: &str,
        id: Uuid,
        spec: &company_experiments::ExperimentSpec,
    ) -> Result<ExperimentRecord, Box<dyn std::error::Error + Send + Sync>> {
        company_experiments::validate_spec(spec).map_err(|error| error.to_string())?;
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO growth_experiments
             (id,company_id,hypothesis,control_variant,treatment_variant,max_budget_minor,min_observations,duration_seconds,success_metric_bps,kill_metric_bps,status)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'PROPOSED')",
            &[&id,&company,&spec.hypothesis,&spec.control,&spec.treatment,&spec.max_budget_minor.to_string(),
              &(spec.min_observations as i64),&(spec.duration_seconds as i64),&spec.success_metric_bps,&spec.kill_metric_bps],
        ).await?;
        Ok(ExperimentRecord { id, company_id: company, spec: spec.clone(), status: company_experiments::ExperimentStatus::Proposed })
    }

    pub async fn create_content_item(
        &self,
        item: &company_content::ContentItem,
    ) -> Result<ContentRecord, Box<dyn std::error::Error + Send + Sync>> {
        company_content::validate_item(item).map_err(|error| error.to_string())?;
        let client = self.client.lock().await;
        let brief = &item.brief;
        let variant = &item.variant;
        let format = content_format_name(brief.format);
        let metric = success_metric_name(brief.success_metric);
        let status = content_status_name(item.status);
        let decision = item.decision.map(content_decision_name);
        let row = client.query_one(
            "INSERT INTO content_items
             (id,company_id,hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
              expected_cost_minor,max_loss_minor,max_duration_seconds,success_metric,success_threshold_bps,
              variant_key,hook,first_frame,emotion,pacing,scene_count,text_density,voice_speed,
              product_placement,cta,comment_trigger,music_style,visual_style,status,decision)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28)
             RETURNING created_at::text",
            &[
                &item.id, &item.company_id, &brief.hypothesis, &brief.audience, &format,
                &brief.product_ref, &brief.offer_ref, &brief.disclosure_required,
                &brief.expected_cost_minor.to_string(), &brief.max_loss_minor.to_string(),
                &(brief.max_duration_seconds as i64), &metric, &(brief.success_threshold_bps as i32),
                &variant.variant_key, &variant.hook, &variant.first_frame, &variant.emotion,
                &variant.pacing, &(variant.scene_count as i32), &variant.text_density,
                &variant.voice_speed, &variant.product_placement, &variant.cta,
                &variant.comment_trigger, &variant.music_style, &variant.visual_style,
                &status, &decision,
            ],
        ).await?;
        Ok(ContentRecord { item: item.clone(), created_at: row.get(0) })
    }

    pub async fn list_content_items(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<ContentRecord>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=500).contains(&limit) {
            return Err("content limit must be between 1 and 500".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT id,company_id,hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
                    expected_cost_minor::text,max_loss_minor::text,max_duration_seconds,success_metric,
                    success_threshold_bps,variant_key,hook,first_frame,emotion,pacing,scene_count,
                    text_density,voice_speed,product_placement,cta,comment_trigger,music_style,
                    visual_style,status,decision,created_at::text
               FROM content_items
              WHERE company_id=$1
              ORDER BY created_at DESC
              LIMIT $2",
            &[&company, &limit],
        ).await?;
        rows.into_iter().map(content_record_from_row).collect()
    }

    pub async fn transition_content_status(
        &self,
        company_id: &str,
        content_id: Uuid,
        next: company_content::ContentStatus,
        evidence_ref: Option<&str>,
    ) -> Result<ContentRecord, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client.query_opt(
            "SELECT id,company_id,hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
                    expected_cost_minor::text,max_loss_minor::text,max_duration_seconds,success_metric,
                    success_threshold_bps,variant_key,hook,first_frame,emotion,pacing,scene_count,
                    text_density,voice_speed,product_placement,cta,comment_trigger,music_style,
                    visual_style,status,decision,created_at::text
               FROM content_items
              WHERE company_id=$1 AND id=$2",
            &[&company, &content_id],
        ).await?.ok_or("content item not found")?;
        let current = parse_content_status(row.get::<_, String>(26))?;
        company_content::validate_status_transition(current, next, evidence_ref)
            .map_err(|error| error.to_string())?;
        let next_name = content_status_name(next);
        client.execute(
            "UPDATE content_items
                SET status=$3, status_evidence_ref=$4
              WHERE company_id=$1 AND id=$2",
            &[&company, &content_id, &next_name, &evidence_ref],
        ).await?;
        let refreshed = client.query_one(
            "SELECT id,company_id,hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
                    expected_cost_minor::text,max_loss_minor::text,max_duration_seconds,success_metric,
                    success_threshold_bps,variant_key,hook,first_frame,emotion,pacing,scene_count,
                    text_density,voice_speed,product_placement,cta,comment_trigger,music_style,
                    visual_style,status,decision,created_at::text
               FROM content_items WHERE company_id=$1 AND id=$2",
            &[&company, &content_id],
        ).await?;
        content_record_from_row(refreshed)
    }

    pub async fn record_content_observation(
        &self,
        observation: &company_content::ContentObservation,
    ) -> Result<ContentObservationRecord, Box<dyn std::error::Error + Send + Sync>> {
        company_content::validate_observation(observation).map_err(|error| error.to_string())?;
        let client = self.client.lock().await;
        let row = client.query_opt(
            "SELECT hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
                    expected_cost_minor::text,max_loss_minor::text,max_duration_seconds,
                    success_metric,success_threshold_bps,variant_key,hook,first_frame,emotion,
                    pacing,scene_count,text_density,voice_speed,product_placement,cta,
                    comment_trigger,music_style,visual_style,status,decision
               FROM content_items
              WHERE company_id=$1 AND id=$2",
            &[&observation.company_id, &observation.content_id],
        ).await?.ok_or("content item not found")?;
        let status: String = row.get(26);
        if !matches!(status.as_str(), "PUBLISHED" | "MEASURED") {
            return Err("content must be published before performance can be recorded".into());
        }
        let item = content_record_from_row(row)?.item;
        let decision = company_content::decide_from_observation(&item.brief, observation)
            .map_err(|error| error.to_string())?;

        let id = Uuid::new_v4();
        let inserted = client.query_opt(
            "INSERT INTO content_observations
             (id,company_id,content_id,observation_key,source,evidence_hash,observed_at_epoch,
              sample_count,spend_minor,metric_bps,views,clicks,conversions,commission_minor,
              contribution_margin_minor,decision)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
             ON CONFLICT(company_id,observation_key) DO NOTHING
             RETURNING id,created_at::text",
            &[
                &id, &observation.company_id, &observation.content_id, &observation.observation_key,
                &observation.source, &observation.evidence_hash, &observation.observed_at_epoch,
                &(observation.sample_count as i64), &observation.spend_minor.to_string(),
                &(observation.metric_bps as i32), &(observation.views as i64), &(observation.clicks as i64),
                &(observation.conversions as i64), &observation.commission_minor.to_string(),
                &observation.contribution_margin_minor.to_string(), &content_decision_name(decision),
            ],
        ).await?;

        if let Some(inserted) = inserted {
            if matches!(decision, company_content::ContentDecision::Scale | company_content::ContentDecision::Kill) {
                let next_status = if decision == company_content::ContentDecision::Scale {
                    "MEASURED"
                } else {
                    "KILLED"
                };
                client.execute(
                    "UPDATE content_items SET status=$3,decision=$4
                     WHERE company_id=$1 AND id=$2",
                    &[&observation.company_id,&observation.content_id,&next_status,&content_decision_name(decision)],
                ).await?;
            }
            return Ok(ContentObservationRecord {
                id: inserted.get(0),
                observation: observation.clone(),
                decision,
                created_at: inserted.get(1),
            });
        }

        content_observation_by_key(&client, &observation.company_id, &observation.observation_key).await
    }

    pub async fn record_experiment_observation(
        &self,
        company_id: &str,
        experiment_id: Uuid,
        observation: &company_experiments::ExperimentObservation,
        observation_key: &str,
    ) -> Result<company_experiments::ExperimentDecision, Box<dyn std::error::Error + Send + Sync>> {
        if observation_key.trim().is_empty() { return Err("experiment observation key is required".into()); }
        if observation.spend_minor < 0 { return Err("experiment spend cannot be negative".into()); }
        let company = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let existing = tx.query_opt(
            "SELECT decision FROM growth_experiment_observations
              WHERE company_id=$1 AND experiment_id=$2 AND observation_key=$3",
            &[&company, &experiment_id, &observation_key],
        ).await?;
        if let Some(existing) = existing {
            let decision: String = existing.get(0);
            return match decision.as_str() {
                "CONTINUE" => Ok(company_experiments::ExperimentDecision::Continue),
                "SUCCEED" => Ok(company_experiments::ExperimentDecision::Succeed),
                "KILL" => Ok(company_experiments::ExperimentDecision::Kill),
                "EXPIRE" => Ok(company_experiments::ExperimentDecision::Expire),
                _ => Err("invalid persisted experiment decision".into()),
            };
        }

        let row = tx.query_opt(
            "SELECT hypothesis,control_variant,treatment_variant,max_budget_minor::text,min_observations,duration_seconds,success_metric_bps,kill_metric_bps,status
               FROM growth_experiments WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company,&experiment_id],
        ).await?.ok_or("experiment not found")?;
        let current_status: String = row.get(8);
        if matches!(current_status.as_str(), "SUCCEEDED" | "KILLED" | "EXPIRED" | "FAILED") {
            return Err("terminal experiment cannot accept new observations".into());
        }
        let spec = company_experiments::ExperimentSpec {
            hypothesis: row.get(0), control: row.get(1), treatment: row.get(2),
            max_budget_minor: row.get::<_,String>(3).parse()?,
            min_observations: row.get::<_,i64>(4) as u64,
            duration_seconds: row.get::<_,i64>(5) as u64,
            success_metric_bps: row.get(6), kill_metric_bps: row.get(7),
        };
        let decision = company_experiments::decide(&spec, observation).map_err(|error| error.to_string())?;
        let status = match decision {
            company_experiments::ExperimentDecision::Continue => "RUNNING",
            company_experiments::ExperimentDecision::Succeed => "SUCCEEDED",
            company_experiments::ExperimentDecision::Kill => "KILLED",
            company_experiments::ExperimentDecision::Expire => "EXPIRED",
        };
        tx.execute(
            "INSERT INTO growth_experiment_observations
             (company_id,experiment_id,control_observations,treatment_observations,control_metric_bps,treatment_metric_bps,spend_minor,elapsed_seconds,decision,observation_key)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
            &[&company,&experiment_id,&(observation.control_observations as i64),&(observation.treatment_observations as i64),
              &observation.control_metric_bps,&observation.treatment_metric_bps,&observation.spend_minor.to_string(),
              &(observation.elapsed_seconds as i64),&format!("{:?}",decision).to_uppercase(),&observation_key],
        ).await?;
        tx.execute(
            "UPDATE growth_experiments SET status=$3,
                started_at=COALESCE(started_at,CASE WHEN $3='RUNNING' THEN now() ELSE started_at END),
                completed_at=CASE WHEN $3 IN ('SUCCEEDED','KILLED','EXPIRED') THEN now() ELSE completed_at END
              WHERE company_id=$1 AND id=$2",
            &[&company,&experiment_id,&status],
        ).await?;

        let decision_ref = format!(
            "experiment:{}:decision:{}:{}",
            experiment_id,
            format!("{:?}", decision).to_ascii_uppercase(),
            observation_key
        );
        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company,
                company_revenue_graph::RevenueNodeType::Experiment,
                &experiment_id.to_string(),
                "RESULTS_IN",
                company_revenue_graph::RevenueNodeType::Decision,
                &decision_ref,
                None,
                None,
                10_000,
                &format!("experiment:{}:{}", experiment_id, observation_key),
                "experiment-engine",
                time::OffsetDateTime::now_utc().unix_timestamp(),
            ),
        )
        .await?;

        if !matches!(decision, company_experiments::ExperimentDecision::Continue) {
            let learning = experiment_learning_entry(
                company,
                experiment_id,
                &spec,
                observation,
                decision,
                observation_key,
            );
            company_learning::validate_evidence(&learning)
                .map_err(|error| error.to_string())?;

            let inserted = tx.query_opt(
                "INSERT INTO learning_entries
                 (id,company_id,entry_key,source_type,source_id,kind,severity,hypothesis,context,
                  expected_outcome,actual_outcome,impact_minor,confidence_bps,root_cause,
                  corrective_action,reusable_rule,decision)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
                 ON CONFLICT(company_id,entry_key) DO NOTHING
                 RETURNING id",
                &[
                    &Uuid::new_v4(),
                    &company,
                    &learning.entry_key,
                    &learning.source_type,
                    &learning.source_id,
                    &learning_kind_name(learning.kind),
                    &failure_severity_name(learning.severity),
                    &learning.hypothesis,
                    &learning.context,
                    &learning.expected_outcome,
                    &learning.actual_outcome,
                    &learning.impact_minor.to_string(),
                    &learning.confidence_bps,
                    &learning.root_cause,
                    &learning.corrective_action,
                    &learning.reusable_rule,
                    &learning_decision_name(learning.decision),
                ],
            )
            .await?;
            let learning_id = match inserted {
                Some(row) => row.get(0),
                None => tx
                    .query_one(
                        "SELECT id FROM learning_entries WHERE company_id=$1 AND entry_key=$2",
                        &[&company, &learning.entry_key],
                    )
                    .await?
                    .get(0),
            };

            let outbox_key = format!("outbox:learning:{}", learning.entry_key);
            let payload = serde_json::json!({
                "entry_key": &learning.entry_key,
                "source_type": &learning.source_type,
                "source_id": &learning.source_id,
                "kind": learning.kind,
                "decision": learning.decision,
                "confidence_bps": learning.confidence_bps
            });
            tx.execute(
                "INSERT INTO outbox_events
                 (company_id,event_type,aggregate_id,idempotency_key,payload)
                 VALUES ($1,'LEARNING_ENTRY_RECORDED',$2,$3,$4)
                 ON CONFLICT(company_id,idempotency_key) DO NOTHING",
                &[
                    &company,
                    &learning_id,
                    &outbox_key,
                    &payload,
                ],
            )
            .await?;
        }

        tx.commit().await?;
        Ok(decision)
    }

    pub async fn affiliate_reconciliation_metrics(
        &self,
        company_id: &str,
    ) -> Result<AffiliateReconciliationMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_one(
                "WITH conversions AS (
                    SELECT conversion_id, commission_minor::text AS commission, reconciliation_status
                      FROM affiliate_conversions
                     WHERE company_id = $1
                       AND created_at >= date_trunc('month', now())
                 ),
                 attributed AS (
                    SELECT a.conversion_id, COALESCE(SUM(a.attributed_commission_minor),0)::text AS commission
                      FROM affiliate_attributions a
                      JOIN conversions c ON c.conversion_id = a.conversion_id
                     WHERE a.company_id = $1
                     GROUP BY a.conversion_id
                 ),
                 totals AS (
                    SELECT
                      COALESCE((SELECT SUM(commission::numeric) FROM conversions),0)::text AS reported,
                      COALESCE((SELECT SUM(commission::numeric) FROM attributed),0)::text AS attributed,
                      COALESCE((SELECT SUM(amount_minor) FROM affiliate_payouts
                                WHERE company_id=$1 AND occurred_at >= date_trunc('month', now())),0)::text AS paid,
                      (SELECT COUNT(*) FROM conversions) AS conversion_count,
                      (SELECT COUNT(*) FROM conversions WHERE reconciliation_status='VERIFIED') AS verified_count,
                      (SELECT COUNT(*) FROM conversions WHERE reconciliation_status IN ('PARTIAL','REJECTED')) AS partial_or_rejected_count
                 )
                 SELECT reported, attributed, paid, conversion_count, verified_count, partial_or_rejected_count
                   FROM totals",
                &[&id],
            )
            .await?;

        let reported = parse_i128_numeric(&row.get::<_, String>(0))?;
        let attributed = parse_i128_numeric(&row.get::<_, String>(1))?;
        let paid = parse_i128_numeric(&row.get::<_, String>(2))?;
        let reported_attributed = reported.checked_sub(attributed).ok_or("affiliate reconciliation overflow")?;
        let attributed_paid = attributed.checked_sub(paid).ok_or("affiliate paid reconciliation overflow")?;
        let reported_paid = reported.checked_sub(paid).ok_or("affiliate paid reconciliation overflow")?;
        Ok(AffiliateReconciliationMetrics {
            reported_commission_mtd_minor: reported,
            attributed_commission_mtd_minor: attributed,
            recorded_payout_mtd_minor: paid,
            variance_mtd_minor: reported_attributed,
            reported_attributed_variance_mtd_minor: reported_attributed,
            attributed_paid_variance_mtd_minor: attributed_paid,
            reported_paid_variance_mtd_minor: reported_paid,
            conversion_count_mtd: row.get(3),
            verified_conversion_count_mtd: row.get(4),
            partial_or_rejected_count_mtd: row.get(5),
        })
    }

    pub async fn assess_autonomy_for_company(
        &self,
        company_id: &str,
        proposal: &agent_runtime::types::Proposal,
        policy: company_autonomy::AutonomyPolicy,
        emergency_stop: bool,
        twin_config: &company_autonomy::DigitalTwinConfig,
    ) -> Result<AutonomySimulationRecord, Box<dyn std::error::Error + Send + Sync>> {
        proposal.validate().map_err(|error| error.to_string())?;
        twin_config.validate().map_err(|error| error.to_string())?;
        let company = Uuid::parse_str(company_id)?;
        let snapshot = self
            .load_snapshot(company_id)
            .await?
            .ok_or("authoritative company snapshot is unavailable")?;
        if snapshot.company_id != company_id {
            return Err("authoritative snapshot belongs to a different company".into());
        }
        let persistent_stop = self.autonomy_controls(company_id).await?;
        let emergency_stop = emergency_stop || persistent_stop.controls.emergency_stop.enabled;

        let simulation =
            company_autonomy::simulate_proposal(&snapshot, proposal, twin_config)?;
        let input = company_autonomy::AutonomyGateInput {
            emergency_stop,
            company_status: snapshot.status,
            action: proposal.action,
            cost_minor: proposal.cost_minor,
            risk: proposal.risk,
            confidence_bps: proposal.confidence_bps,
            evidence_count: proposal.evidence.len().min(u8::MAX as usize) as u8,
            reversible: proposal.reversible,
            external_side_effect: proposal.action.inherently_material(),
            policy,
            simulation: Some(simulation.clone()),
        };
        let assessment = company_autonomy::assess(&input)?;

        let key_payload = serde_json::json!({
            "company_id": company_id,
            "proposal": proposal,
            "policy": policy,
            "emergency_stop": emergency_stop,
            "twin_config": twin_config,
            "snapshot": {
                "cash_minor": snapshot.cash_minor,
                "revenue_minor": snapshot.revenue_minor,
                "expenses_minor": snapshot.expenses_minor,
                "liabilities_minor": snapshot.liabilities_minor,
                "assets_minor": snapshot.assets_minor,
                "runway_days": snapshot.runway_days,
                "status": snapshot.status,
                "budget_remaining_minor": snapshot.budget_remaining_minor,
                "experiment_budget_minor": snapshot.experiment_budget_minor,
                "backlog": snapshot.backlog,
                "capacity": snapshot.capacity
            }
        });
        let payload_bytes = serde_json::to_vec(&key_payload)?;
        let digest = Sha256::digest(payload_bytes);
        let idempotency_key = format!("autonomy:{}", digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        let assessment_json = serde_json::to_value(&assessment)?;

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        if let Some(row) = tx
            .query_opt(
                "SELECT id,proposal,decision,ceiling,required_level,reason,assessment_json,created_at::text
                   FROM autonomy_simulations
                  WHERE company_id=$1 AND idempotency_key=$2",
                &[&company, &idempotency_key],
            )
            .await?
        {
            let existing = autonomy_simulation_from_row(
                row,
                company,
                idempotency_key.clone(),
            )?;
            tx.rollback().await?;
            return Ok(existing);
        }

        let id = Uuid::new_v4();
        let row = tx
            .query_one(
                "INSERT INTO autonomy_simulations
                 (id,company_id,idempotency_key,proposal,decision,ceiling,required_level,reason,assessment_json)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                 RETURNING created_at::text",
                &[
                    &id,
                    &company,
                    &idempotency_key,
                    &serde_json::to_value(proposal)?,
                    &assessment.decision.as_str(),
                    &assessment.ceiling.as_str(),
                    &assessment.required_level.as_str(),
                    &assessment.reason,
                    &assessment_json,
                ],
            )
            .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'AUTONOMY_ASSESSMENT_RECORDED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &id,
                &format!("outbox:{idempotency_key}"),
                &serde_json::json!({
                    "simulation_id": id,
                    "decision": assessment.decision.as_str(),
                    "ceiling": assessment.ceiling.as_str(),
                    "required_level": assessment.required_level.as_str(),
                }),
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(AutonomySimulationRecord {
            id,
            company_id: company,
            idempotency_key,
            proposal: proposal.clone(),
            assessment,
            created_at: row.get(0),
        })
    }

    pub async fn record_agent_outcome_evidence(
        &self,
        company_id: &str,
        decision_journal_id: i64,
        evidence_ref: &str,
        observed_revenue_delta_minor: i128,
        observed_contribution_margin_delta_minor: i128,
        observed_at_epoch: i64,
    ) -> Result<AgentOutcomeEvidenceRecord, Box<dyn std::error::Error + Send + Sync>> {
        if decision_journal_id <= 0
            || evidence_ref.trim().is_empty()
            || evidence_ref.len() > 1024
            || observed_at_epoch <= 0
        {
            return Err("agent outcome evidence identity is invalid".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let evidence_ref = evidence_ref.trim();
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let journal = tx
            .query_opt(
                "SELECT agent_name, action, governor_decision, execution, EXTRACT(EPOCH FROM created_at)::bigint
                   FROM decision_journal
                  WHERE company_id=$1 AND id=$2
                  FOR UPDATE",
                &[&company, &decision_journal_id],
            )
            .await?
            .ok_or("decision journal record not found for company")?;

        let agent_name: String = journal.get(0);
        let action: String = journal.get(1);
        let governor_decision: String = journal.get(2);
        let execution: Option<serde_json::Value> = journal.get(3);
        let decision_created_at_epoch: i64 = journal.get(4);

        if governor_decision != "Approve"
            && governor_decision != "APPROVE"
        {
            return Err("outcome evidence requires a Governor-approved decision".into());
        }
        if execution
            .as_ref()
            .and_then(|value| value.get("status"))
            .and_then(|value| value.as_str())
            != Some("Executed")
        {
            return Err("outcome evidence requires an executed decision".into());
        }
        if observed_at_epoch < decision_created_at_epoch {
            return Err("outcome evidence cannot predate the decision".into());
        }

        if let Some(row) = tx
            .query_opt(
                "SELECT id,agent_name,action,evidence_ref,
                        observed_revenue_delta_minor::text,
                        observed_contribution_margin_delta_minor::text,
                        observed_at_epoch,created_at::text
                   FROM agent_outcome_evidence oe
                  JOIN decision_journal dj
                    ON dj.company_id=oe.company_id
                   AND dj.id=oe.decision_journal_id
                  WHERE oe.company_id=$1 AND oe.decision_journal_id=$2
                  FOR UPDATE",
                &[&company, &decision_journal_id],
            )
            .await?
        {
            let existing = agent_outcome_evidence_from_row(row, company, decision_journal_id)?;
            if existing.evidence_ref != evidence_ref
                || existing.observed_revenue_delta_minor != observed_revenue_delta_minor
                || existing.observed_contribution_margin_delta_minor
                    != observed_contribution_margin_delta_minor
                || existing.observed_at_epoch != observed_at_epoch
            {
                return Err("agent outcome evidence is immutable and already recorded with different values".into());
            }
            tx.rollback().await?;
            return Ok(existing);
        }

        let id = Uuid::new_v4();
        let row = tx
            .query_one(
                "INSERT INTO agent_outcome_evidence
                 (id,company_id,decision_journal_id,evidence_ref,
                  observed_revenue_delta_minor,observed_contribution_margin_delta_minor,
                  observed_at_epoch)
                 VALUES ($1,$2,$3,$4,$5::numeric,$6::numeric,$7)
                 RETURNING created_at::text",
                &[
                    &id,
                    &company,
                    &decision_journal_id,
                    &evidence_ref,
                    &observed_revenue_delta_minor.to_string(),
                    &observed_contribution_margin_delta_minor.to_string(),
                    &observed_at_epoch,
                ],
            )
            .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'AGENT_OUTCOME_EVIDENCE_RECORDED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &decision_journal_id.to_string(),
                &format!("outbox:agent-outcome:{decision_journal_id}"),
                &serde_json::json!({
                    "decision_journal_id": decision_journal_id,
                    "agent": agent_name,
                    "action": action,
                    "evidence_ref": evidence_ref,
                    "observed_revenue_delta_minor": observed_revenue_delta_minor,
                    "observed_contribution_margin_delta_minor": observed_contribution_margin_delta_minor,
                    "observed_at_epoch": observed_at_epoch,
                }),
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(AgentOutcomeEvidenceRecord {
            id,
            company_id: company,
            decision_journal_id,
            agent_name,
            action,
            evidence_ref: evidence_ref.trim().to_owned(),
            observed_revenue_delta_minor,
            observed_contribution_margin_delta_minor,
            observed_at_epoch,
            created_at: row.get(0),
        })
    }

    pub async fn agent_outcome_evaluations(
        &self,
        company_id: &str,
        days: i64,
    ) -> Result<Vec<company_agent_evaluation::AgentEvaluation>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=365).contains(&days) {
            return Err("agent evaluation window must be between 1 and 365 days".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client
            .query(
                "SELECT
                    dj.agent_name,
                    COUNT(*)::bigint AS proposal_count,
                    COUNT(*) FILTER (WHERE dj.governor_decision IN ('Approve','APPROVE'))::bigint,
                    COUNT(*) FILTER (WHERE dj.governor_decision IN ('Reject','REJECT'))::bigint,
                    COUNT(*) FILTER (WHERE dj.governor_decision IN ('RequestRevision','REQUESTREVISION','REQUEST_REVISION'))::bigint,
                    COUNT(*) FILTER (WHERE dj.governor_decision IN ('Escalate','ESCALATE'))::bigint,
                    COUNT(*) FILTER (WHERE dj.execution->>'status'='Executed')::bigint,
                    COUNT(*) FILTER (WHERE dj.execution->>'status'='Deferred')::bigint,
                    COALESCE(
                        SUM(
                            CASE
                                WHEN dj.execution->>'status'='Executed'
                                THEN (dj.execution->>'cost_minor')::numeric
                                ELSE 0
                            END
                        ),
                        0
                    )::text,
                    COALESCE(
                        SUM(
                            CASE
                                WHEN dj.execution->>'status'='Executed'
                                THEN (dj.proposal->>'expected_revenue_minor')::numeric
                                ELSE 0
                            END
                        ),
                        0
                    )::text,
                    COUNT(oe.id)::bigint,
                    COALESCE(SUM(oe.observed_revenue_delta_minor),0)::text,
                    COALESCE(SUM(oe.observed_contribution_margin_delta_minor),0)::text
                 FROM decision_journal dj
            LEFT JOIN agent_outcome_evidence oe
                   ON oe.company_id=dj.company_id
                  AND oe.decision_journal_id=dj.id
                WHERE dj.company_id=$1
                  AND dj.created_at >= now() - ($2::double precision * interval '1 day')
                GROUP BY dj.agent_name
                ORDER BY dj.agent_name ASC",
                &[&company, &days],
            )
            .await?;

        rows.into_iter()
            .map(agent_evaluation_from_row)
            .collect()
    }

    pub async fn ceo_command_center(
        &self,
        company_id: &str,
        revenue_target_minor: i128,
    ) -> Result<CeoCommandCenterRecord, Box<dyn std::error::Error + Send + Sync>> {
        if revenue_target_minor <= 0 {
            return Err("CEO command center revenue target must be positive".into());
        }

        let company = Uuid::parse_str(company_id)?;
        let snapshot = self
            .load_snapshot(company_id)
            .await?
            .ok_or("authoritative company snapshot is unavailable")?;

        let revenue = self.revenue_period_metrics(company_id).await?;
        let margin = self.contribution_margin_metrics(company_id).await?;
        let affiliate = self.affiliate_reconciliation_metrics(company_id).await?;

        let client = self.client.lock().await;

        let affiliate_order_row = client
            .query_one(
                "SELECT
                    COUNT(DISTINCT order_id) FILTER (WHERE cancelled=false),
                    COALESCE(
                        SUM(
                            CASE
                                WHEN cancelled=false
                                THEN GREATEST(order_value_minor - refunded_minor, 0)
                                ELSE 0
                            END
                        ),
                        0
                    )::text
                 FROM affiliate_conversions
                WHERE company_id=$1
                  AND created_at >= date_trunc('month', now())",
                &[&company],
            )
            .await?;
        let affiliate_orders_mtd: i64 = affiliate_order_row.get(0);
        let affiliate_net_order_value_mtd_minor =
            parse_i128_numeric(&affiliate_order_row.get::<_, String>(1))?;

        let content_row = client
            .query_one(
                "WITH latest AS (
                    SELECT DISTINCT ON (content_id)
                        content_id,
                        views,
                        clicks,
                        conversions,
                        spend_minor::text AS spend_minor,
                        commission_minor::text AS commission_minor,
                        contribution_margin_minor::text AS contribution_margin_minor
                      FROM content_observations
                     WHERE company_id=$1
                       AND observed_at_epoch >= EXTRACT(EPOCH FROM (now() - interval '7 days'))::bigint
                     ORDER BY content_id, observed_at_epoch DESC, created_at DESC
                 )
                 SELECT
                    COALESCE(SUM(views),0)::bigint,
                    COALESCE(SUM(clicks),0)::bigint,
                    COALESCE(SUM(conversions),0)::bigint,
                    COALESCE(SUM(spend_minor::numeric),0)::text,
                    COALESCE(SUM(commission_minor::numeric),0)::text,
                    COALESCE(SUM(contribution_margin_minor::numeric),0)::text,
                    COUNT(*)::bigint
                   FROM latest",
                &[&company],
            )
            .await?;
        let views_7d: i64 = content_row.get(0);
        let clicks_7d: i64 = content_row.get(1);
        let conversions_7d: i64 = content_row.get(2);
        let spend_7d_minor = parse_i128_numeric(&content_row.get::<_, String>(3))?;
        let commission_7d_minor = parse_i128_numeric(&content_row.get::<_, String>(4))?;
        let contribution_margin_7d_minor =
            parse_i128_numeric(&content_row.get::<_, String>(5))?;
        let content_count_7d: i64 = content_row.get(6);

        let content = company_command_center::ContentFunnel {
            views_7d,
            clicks_7d,
            conversions_7d,
            spend_7d_minor,
            commission_7d_minor,
            contribution_margin_7d_minor,
            content_count_7d,
            ctr_bps: metric_bps(clicks_7d, views_7d)?,
            cvr_bps: metric_bps(conversions_7d, clicks_7d)?,
            commission_rpm_minor: scaled_minor(commission_7d_minor, views_7d, 1_000)?,
        };

        let live_row = client
            .query_one(
                "SELECT
                    COUNT(DISTINCT s.id)::bigint,
                    COALESCE(
                        SUM(
                            CASE WHEN e.kind='GIFT' THEN e.gift_quantity ELSE 0 END
                        ),
                        0
                    )::numeric::text,
                    COALESCE(
                        SUM(
                            CASE WHEN e.kind='GIFT' THEN e.gift_value_minor ELSE 0 END
                        ),
                        0
                    )::numeric::text
                   FROM tiktok_live_sessions s
              LEFT JOIN tiktok_live_events e
                     ON e.company_id=s.company_id
                    AND e.session_id=s.id
                  WHERE s.company_id=$1
                    AND s.started_at_epoch >= EXTRACT(EPOCH FROM (now() - interval '30 days'))::bigint",
                &[&company],
            )
            .await?;
        let live = company_command_center::LivePulse {
            sessions_30d: live_row.get(0),
            gift_count_30d: parse_i128_numeric(&live_row.get::<_, String>(1))?,
            gift_value_30d_minor: parse_i128_numeric(&live_row.get::<_, String>(2))?,
        };

        let daily_rows = client
            .query(
                "WITH days AS (
                    SELECT generate_series(
                        date_trunc('day', now()) - interval '6 days',
                        date_trunc('day', now()),
                        interval '1 day'
                    ) AS day
                ),
                revenue AS (
                    SELECT date_trunc('day', t.created_at) AS day,
                           COALESCE(SUM(e.credit_minor - e.debit_minor),0)::text AS revenue_minor
                      FROM ledger_transactions t
                      JOIN ledger_entries e ON e.transaction_id=t.id
                      JOIN ledger_accounts a ON a.id=e.account_id
                     WHERE t.company_id=$1
                       AND a.company_id=$1
                       AND a.account_type='REVENUE'
                       AND t.created_at >= now() - interval '7 days'
                     GROUP BY 1
                )
                SELECT to_char(days.day,'YYYY-MM-DD'),
                       COALESCE(revenue.revenue_minor,'0')
                  FROM days
                  LEFT JOIN revenue ON revenue.day=days.day
                 ORDER BY days.day ASC",
                &[&company],
            )
            .await?;
        let mut daily_revenue = Vec::with_capacity(daily_rows.len());
        for row in daily_rows {
            daily_revenue.push(company_command_center::DailyRevenuePoint {
                day: row.get(0),
                revenue_minor: parse_i128_numeric(&row.get::<_, String>(1))?,
            });
        }
        drop(client);

        let compliance = self.compliance_status(company_id).await?;
        let latest_policy = compliance.get("latest_policy");
        let compliance_counts = compliance
            .get("checks_last_24h")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let compliance_pulse = company_command_center::CompliancePulse {
            policy_ready: latest_policy
                .and_then(|policy| policy.get("active"))
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            allowed_24h: compliance_counts
                .get("allowed")
                .and_then(|value| value.as_i64())
                .unwrap_or(0),
            review_24h: compliance_counts
                .get("review")
                .and_then(|value| value.as_i64())
                .unwrap_or(0),
            blocked_24h: compliance_counts
                .get("blocked")
                .and_then(|value| value.as_i64())
                .unwrap_or(0),
            unknown_24h: compliance_counts
                .get("unknown")
                .and_then(|value| value.as_i64())
                .unwrap_or(0),
        };

        let growth_records = self.list_growth_opportunities(company_id, 5).await?;
        let growth_opportunities = growth_records
            .into_iter()
            .map(|record| {
                let status = match record.status {
                    company_growth::OpportunityStatus::Ready => "READY",
                    company_growth::OpportunityStatus::ContentCreated => "CONTENT_CREATED",
                };
                company_command_center::GrowthOpportunityDigest {
                    title: record.opportunity.title,
                    score_bps: record.opportunity.score_bps,
                    confidence_bps: record.opportunity.confidence_bps,
                    status: status.into(),
                    ttfc_seconds: record.ttfc_seconds,
                }
            })
            .collect::<Vec<_>>();

        let input = company_command_center::CommandCenterInput {
            cash_minor: snapshot.cash_minor,
            revenue_mtd_minor: revenue.month_to_date_minor,
            revenue_last_30d_minor: revenue.last_30_days_minor,
            revenue_lifetime_minor: revenue.lifetime_minor,
            revenue_target_minor,
            revenue_transaction_count: revenue.revenue_transaction_count,
            contribution_margin_mtd_minor: margin.month_to_date_contribution_margin_minor,
            unclassified_expense_entry_count: margin.unclassified_expense_entry_count,
            affiliate_reported_commission_mtd_minor: affiliate.reported_commission_mtd_minor,
            affiliate_attributed_commission_mtd_minor: affiliate.attributed_commission_mtd_minor,
            affiliate_payout_mtd_minor: affiliate.recorded_payout_mtd_minor,
            affiliate_variance_mtd_minor: affiliate.variance_mtd_minor,
            affiliate_orders_mtd,
            affiliate_net_order_value_mtd_minor,
            runway_days: snapshot.runway_days,
            active_employee_count: {
                let employees = self.list_employees(company_id).await?;
                employees
                    .iter()
                    .filter(|employee| {
                        matches!(
                            employee.status,
                            company_organization::EmployeeStatus::Active
                        )
                    })
                    .count() as i64
            },
            payroll_due_count: self.payroll_due(company_id, 500).await?.len() as i64,
            content,
            live,
            compliance: compliance_pulse,
            growth_opportunities,
            daily_revenue,
        };

        let summary =
            company_command_center::summarize(&input).map_err(|error| error.to_string())?;
        Ok(CeoCommandCenterRecord { input, summary })
    }

    pub async fn contribution_margin_metrics(
        &self,
        company_id: &str,
    ) -> Result<ContributionMarginMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_one(
                "SELECT
                    COALESCE(SUM(CASE WHEN a.account_type = 'REVENUE'
                                       THEN e.credit_minor - e.debit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE'
                                       AND a.cost_class = 'VARIABLE'
                                       THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE'
                                       AND a.cost_class = 'UNCLASSIFIED'
                                       THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COUNT(*) FILTER (WHERE a.account_type = 'EXPENSE'
                                      AND a.cost_class = 'UNCLASSIFIED'),
                    COUNT(DISTINCT CASE WHEN a.account_type = 'EXPENSE'
                                          AND a.cost_class = 'VARIABLE'
                                        THEN t.id END),
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE' AND (a.code IN ('PLATFORM_FEE_EXPENSE','PLATFORM_FEES') OR lower(a.name) LIKE '%platform fee%' OR lower(a.name) LIKE '%processing fee%') THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE' AND (a.code IN ('AFFILIATE_COMMISSION_EXPENSE','AFFILIATE_COMMISSION') OR lower(a.name) LIKE '%affiliate commission%') THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE' AND (a.code IN ('REFUND_EXPENSE','REFUNDS_CANCELLATIONS') OR lower(a.name) LIKE '%refund%' OR lower(a.name) LIKE '%cancellation%') THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE' AND (a.code IN ('PRODUCTION_AI_EXPENSE','AI_PRODUCTION') OR lower(a.name) LIKE '%production%' OR lower(a.name) LIKE '%ai cost%') THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE' AND (a.code IN ('AD_SPEND_EXPENSE','AD_SPEND') OR lower(a.name) LIKE '%ad spend%' OR lower(a.name) LIKE '%advertising%') THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COALESCE(SUM(CASE WHEN a.account_type = 'EXPENSE' AND a.cost_class = 'FIXED' THEN e.debit_minor - e.credit_minor ELSE 0 END), 0)::text,
                    COALESCE((
                        SELECT SUM(ce.debit_minor - ce.credit_minor)
                          FROM ledger_entries ce
                          JOIN ledger_accounts ca ON ca.id = ce.account_id
                         WHERE ca.company_id = $1
                           AND ca.code = 'CASH'
                    ), 0)::text
                 FROM ledger_transactions t
                 JOIN ledger_entries e ON e.transaction_id = t.id
                 JOIN ledger_accounts a ON a.id = e.account_id
                WHERE t.company_id = $1
                  AND a.company_id = $1
                  AND t.created_at >= date_trunc('month', now())",
                &[&id],
            )
            .await?;

        let revenue = parse_i128_numeric(&row.get::<_, String>(0))?;
        let variable_cost = parse_i128_numeric(&row.get::<_, String>(1))?;
        let unclassified = parse_i128_numeric(&row.get::<_, String>(2))?;
        let unclassified_entries: i64 = row.get(3);

        Ok(ContributionMarginMetrics {
            month_to_date_revenue_minor: revenue,
            month_to_date_variable_cost_minor: variable_cost,
            month_to_date_contribution_margin_minor: if unclassified_entries == 0 {
                Some(revenue.checked_sub(variable_cost).ok_or("contribution margin overflow")?)
            } else {
                None
            },
            platform_fees_minor: parse_i128_numeric(&row.get::<_, String>(5))?,
            affiliate_commission_minor: parse_i128_numeric(&row.get::<_, String>(6))?,
            refunds_cancellations_minor: parse_i128_numeric(&row.get::<_, String>(7))?,
            production_ai_cost_minor: parse_i128_numeric(&row.get::<_, String>(8))?,
            ad_spend_minor: parse_i128_numeric(&row.get::<_, String>(9))?,
            operating_cost_minor: parse_i128_numeric(&row.get::<_, String>(10))?,
            cash_minor: parse_i128_numeric(&row.get::<_, String>(11))?,
            unclassified_expense_minor: unclassified,
            unclassified_expense_entry_count: unclassified_entries,
            variable_cost_transaction_count: row.get(4),
        })
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

        let safety_controls = {
            tx.execute(
                "INSERT INTO autonomy_control_state
                 (company_id, emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
                  emergency_stop_changed_at_epoch, content_publish_daily, ads_spend_daily_minor,
                  live_minutes_daily, outbound_messages_daily, autonomous_capital_daily_minor,
                  updated_at_epoch)
                 VALUES ($1,false,NULL,'system-default',EXTRACT(EPOCH FROM now())::bigint,10,0,60,100,0,EXTRACT(EPOCH FROM now())::bigint)
                 ON CONFLICT(company_id) DO NOTHING",
                &[&company_id],
            )
            .await?;
            load_safety_controls_for_tx(&tx, company_id).await?
        };

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

        let batch = if safety_controls.emergency_stop.enabled {
            company_execution::ExecutionBatch {
                snapshot: current_snapshot.clone(),
                receipts: authoritative_results
                    .iter()
                    .map(|result| company_execution::ExecutionReceipt {
                        idempotency_key: proposal_idempotency_key(&result.proposal),
                        agent: result.agent,
                        action: result.proposal.action,
                        status: company_execution::ExecutionStatus::Rejected,
                        cost_minor: result.proposal.cost_minor,
                        reason: "persistent emergency stop blocks autonomous cycle side effects".into(),
                    })
                    .collect(),
                total_spend_minor: 0,
            }
        } else {
            let proposed_batch =
                execute_approved_results(current_snapshot.clone(), &authoritative_results, execution_policy())?;

            if proposed_batch.total_spend_minor > 0 {
                let now_epoch: i64 = tx
                    .query_one("SELECT EXTRACT(EPOCH FROM now())::bigint", &[])
                    .await?
                    .get(0);
                let budget = Self::consume_autonomy_budget_tx(
                    &tx,
                    company_id,
                    company_safety_controls::BudgetKind::AutonomousCapital,
                    proposed_batch.total_spend_minor,
                    &format!("autonomy-cycle:{cycle_key}"),
                    now_epoch,
                )
                .await?;

                if budget.allowed {
                    proposed_batch
                } else {
                    company_execution::ExecutionBatch {
                        snapshot: current_snapshot.clone(),
                        receipts: authoritative_results
                            .iter()
                            .map(|result| company_execution::ExecutionReceipt {
                                idempotency_key: proposal_idempotency_key(&result.proposal),
                                agent: result.agent,
                                action: result.proposal.action,
                                status: company_execution::ExecutionStatus::Deferred,
                                cost_minor: result.proposal.cost_minor,
                                reason: format!(
                                    "autonomous capital budget blocked cycle execution: {}",
                                    budget.reason
                                ),
                            })
                            .collect(),
                        total_spend_minor: 0,
                    }
                }
            } else {
                proposed_batch
            }
        };

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
                "last_decision",
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

    pub async fn create_publish_intent(
        &self,
        intent: &publishing_contract::PublishIntent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        intent.validate().map_err(|error| error.to_string())?;
        let company_uuid = Uuid::parse_str(&intent.company_id)?;
        let intent_uuid = Uuid::parse_str(&intent.id)?;
        let idempotency_key = publishing_contract::new_idempotency_key(intent);

        let scheduled_at = intent
            .scheduled_at
            .as_deref()
            .map(|value| {
                time::OffsetDateTime::parse(
                    value,
                    &time::format_description::well_known::Rfc3339,
                )
                .map(|parsed| parsed.format(&time::format_description::well_known::Rfc3339))
            })
            .transpose()??;

        let client = self.client.lock().await;
        let existing = client
            .query_opt(
                "SELECT id, content_id, platform, media_uri, title, caption,
                        content_hash, created_by
                   FROM publish_intents
                  WHERE company_id=$1 AND idempotency_key=$2",
                &[&company_uuid, &idempotency_key],
            )
            .await?;
        if let Some(row) = existing {
            let existing_id: Uuid = row.get(0);
            let existing_content: String = row.get(1);
            let existing_platform: String = row.get(2);
            let existing_uri: String = row.get(3);
            let existing_title: String = row.get(4);
            let existing_caption: String = row.get(5);
            let existing_hash: String = row.get(6);

            if existing_id != intent_uuid
                || existing_content != intent.content_id
                || existing_platform != intent.platform.as_str()
                || existing_uri != intent.media_uri
                || existing_title != intent.title
                || existing_caption != intent.caption
                || existing_hash != intent.content_hash
            {
                return Err("publish idempotency key conflicts with a different intent".into());
            }
            return Ok(());
        }

        client
            .execute(
                "INSERT INTO publish_intents
                 (id, company_id, content_id, platform, media_uri, title, caption,
                  scheduled_at, content_hash, created_by, idempotency_key, status)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'DRAFT')",
                &[
                    &intent_uuid,
                    &company_uuid,
                    &intent.content_id,
                    &intent.platform.as_str(),
                    &intent.media_uri,
                    &intent.title,
                    &intent.caption,
                    &scheduled_at,
                    &intent.content_hash,
                    &intent.created_by.as_str(),
                    &idempotency_key,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn approve_publish_intent(
        &self,
        company_id: &str,
        intent_id: &str,
        approved_by: &str,
        ttl_seconds: i64,
    ) -> Result<publishing_contract::PublishApproval, Box<dyn std::error::Error + Send + Sync>> {
        if approved_by.trim().is_empty() || approved_by.len() > 256 {
            return Err("publish approver is invalid".into());
        }
        if !(30..=86_400).contains(&ttl_seconds) {
            return Err("publish approval TTL must be between 30 seconds and 24 hours".into());
        }

        let company_uuid = Uuid::parse_str(company_id)?;
        let intent_uuid = Uuid::parse_str(intent_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let status: String = tx
            .query_opt(
                "SELECT status FROM publish_intents
                  WHERE company_id=$1 AND id=$2
                  FOR UPDATE",
                &[&company_uuid, &intent_uuid],
            )
            .await?
            .ok_or("publish intent not found")?
            .get(0);

        if status != publishing_contract::PublishIntentStatus::Draft.as_str() {
            return Err(format!("publish intent cannot be approved from {status} state").into());
        }

        let (approval_token, token_hash) = publishing_contract::generate_approval_token();
        let expires = time::OffsetDateTime::now_utc()
            + time::Duration::seconds(ttl_seconds);
        let expires_at = expires.format(&time::format_description::well_known::Rfc3339)?;

        tx.execute(
            "UPDATE publish_intents
                SET status='APPROVED',
                    approval_token_hash=$3,
                    approval_expires_at=$4,
                    approved_by=$5
              WHERE company_id=$1 AND id=$2",
            &[
                &company_uuid,
                &intent_uuid,
                &token_hash,
                &expires_at,
                &approved_by,
            ],
        )
        .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'PUBLISH_INTENT_APPROVED',$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[
                &company_uuid,
                &intent_id,
                &format!("outbox:publish-approved:{intent_id}:{expires_at}"),
                &serde_json::json!({
                    "intent_id": intent_id,
                    "approved_by": approved_by,
                    "expires_at": expires_at,
                }),
            ],
        )
        .await?;

        tx.commit().await?;

        Ok(publishing_contract::PublishApproval {
            intent_id: intent_id.to_owned(),
            approval_token,
            expires_at,
        })
    }

    pub async fn claim_publish_intent(
        &self,
        company_id: &str,
        intent_id: &str,
        approval_token: &str,
        lease_seconds: i64,
    ) -> Result<Option<publishing_contract::PublishJob>, Box<dyn std::error::Error + Send + Sync>> {
        if approval_token.trim().is_empty() || !(15..=300).contains(&lease_seconds) {
            return Err("publish claim token or lease is invalid".into());
        }

        let company_uuid = Uuid::parse_str(company_id)?;
        let intent_uuid = Uuid::parse_str(intent_id)?;
        let expected_hash = publishing_contract::hash_secret(approval_token);

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx
            .query_opt(
                "SELECT content_id, platform, media_uri, title, caption,
                        scheduled_at, content_hash, created_by, idempotency_key,
                        status, approval_token_hash, approval_expires_at, locked_until
                   FROM publish_intents
                  WHERE company_id=$1 AND id=$2
                  FOR UPDATE",
                &[&company_uuid, &intent_uuid],
            )
            .await?
            .ok_or("publish intent not found")?;

        let safety_controls = load_safety_controls_for_tx(&tx, company_uuid).await?;
        if safety_controls.emergency_stop.enabled {
            return Err("persistent emergency stop blocks publish claims".into());
        }

        let status: String = row.get(9);
        if status != publishing_contract::PublishIntentStatus::Approved.as_str() {
            return Ok(None);
        }
        let token_hash: Option<String> = row.get(10);
        if token_hash.as_deref() != Some(expected_hash.as_str()) {
            return Err("publish approval token is invalid".into());
        }

        let approval_expires_at: Option<time::OffsetDateTime> = row.get(11);
        if approval_expires_at.is_some_and(|expires| expires <= time::OffsetDateTime::now_utc()) {
            tx.execute(
                "UPDATE publish_intents
                    SET status='FAILED',
                        error_message='approval token expired'
                  WHERE company_id=$1 AND id=$2",
                &[&company_uuid, &intent_uuid],
            )
            .await?;
            tx.commit().await?;
            return Err("publish approval token expired".into());
        }

        let scheduled_at: Option<time::OffsetDateTime> = row.get(5);
        if scheduled_at.is_some_and(|scheduled| scheduled > time::OffsetDateTime::now_utc()) {
            return Ok(None);
        }

        let execution_token = Uuid::new_v4();
        let row = tx
            .query_one(
                "UPDATE publish_intents
                    SET status='RUNNING',
                        approval_token_hash=NULL,
                        execution_token=$3,
                        locked_until=now() +
                            ($4::double precision * interval '1 second'),
                        attempts=attempts+1
                  WHERE company_id=$1 AND id=$2
                    AND status='APPROVED'
                  RETURNING content_id, platform, media_uri, title, caption,
                            scheduled_at, content_hash, created_by, idempotency_key",
                &[
                    &company_uuid,
                    &intent_uuid,
                    &execution_token,
                    &lease_seconds,
                ],
            )
            .await?;

        let platform = publishing_contract::PublishPlatform::parse(&row.get::<_, String>(1))
            .ok_or("stored publish platform is invalid")?;
        let created_by = AgentRole::ALL
            .iter()
            .copied()
            .find(|role| role.as_str() == row.get::<_, String>(7))
            .ok_or("stored publish creator is invalid")?;
        let intent = publishing_contract::PublishIntent {
            id: intent_id.to_owned(),
            company_id: company_id.to_owned(),
            content_id: row.get(0),
            platform,
            media_uri: row.get(2),
            title: row.get(3),
            caption: row.get(4),
            scheduled_at: row
                .get::<_, Option<time::OffsetDateTime>>(5)
                .map(|value| value.format(&time::format_description::well_known::Rfc3339))
                .transpose()?,
            content_hash: row.get(6),
            created_by,
            idempotency_key: row.get(8),
        };

        tx.commit().await?;

        Ok(Some(publishing_contract::PublishJob {
            intent,
            execution_token: execution_token.to_string(),
        }))
    }

    pub async fn record_publish_started(
        &self,
        company_id: &str,
        intent_id: &str,
        execution_token: &str,
        publish_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if publish_id.trim().is_empty() || publish_id.len() > 512 {
            return Err("TikTok publish id is invalid".into());
        }
        let company_uuid = Uuid::parse_str(company_id)?;
        let intent_uuid = Uuid::parse_str(intent_id)?;
        let execution_uuid = Uuid::parse_str(execution_token)?;
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "UPDATE publish_intents
                    SET external_reference=$4
                  WHERE company_id=$1 AND id=$2
                    AND status='RUNNING' AND execution_token=$3",
                &[&company_uuid, &intent_uuid, &execution_uuid, &publish_id],
            )
            .await?;
        if changed == 0 {
            return Err("publish execution token is invalid or lease is no longer owned".into());
        }
        Ok(())
    }

    pub async fn reconcile_tiktok_webhook(
        &self,
        company_id: &str,
        event_key: &str,
        event_name: &str,
        publish_id: Option<&str>,
        payload_hash: &str,
        success: Option<bool>,
        failure_reason: Option<&str>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        if event_key.trim().is_empty() || event_key.len() > 512 {
            return Err("TikTok webhook event key is invalid".into());
        }
        let client = self.client.lock().await;
        let mut tx = client.transaction().await?;
        let inserted = tx
            .execute(
                "INSERT INTO tiktok_webhook_receipts
                 (id, company_id, event_key, event_name, publish_id, payload_hash)
                 VALUES ($1,$2,$3,$4,$5,$6)
                 ON CONFLICT (company_id,event_key) DO NOTHING",
                &[
                    &Uuid::new_v4(),
                    &company_uuid,
                    &event_key,
                    &event_name,
                    &publish_id,
                    &payload_hash,
                ],
            )
            .await?;
        if inserted == 0 {
            tx.rollback().await?;
            return Ok(false);
        }

        if let (Some(publish_id), Some(success)) = (publish_id, success) {
            let final_status = if success { "SUCCEEDED" } else { "FAILED" };
            let error_message = failure_reason.filter(|v| !v.is_empty());
            let changed = tx
                .execute(
                    "UPDATE publish_intents
                        SET status=$4,
                            error_message=$5,
                            execution_token=NULL,
                            locked_until=NULL
                      WHERE company_id=$1 AND external_reference=$2 AND status='RUNNING'",
                    &[&company_uuid, &publish_id, &final_status, &error_message],
                )
                .await?;
            if changed > 0 {
                tx.execute(
                    "INSERT INTO outbox_events
                     (company_id,event_type,aggregate_id,idempotency_key,payload)
                     VALUES ($1,'PUBLISH_INTENT_COMPLETED',$2,$3,$4)
                     ON CONFLICT (company_id,idempotency_key) DO NOTHING",
                    &[
                        &company_uuid,
                        &publish_id,
                        &format!("outbox:tiktok-webhook:{event_key}"),
                        &serde_json::json!({
                            "event": event_name,
                            "publish_id": publish_id,
                            "success": success,
                            "error_message": error_message,
                        }),
                    ],
                )
                .await?;
            }
        }

        tx.commit().await?;
        Ok(true)
    }

    pub async fn complete_publish_intent(
        &self,
        company_id: &str,
        intent_id: &str,
        execution_token: &str,
        success: bool,
        external_reference: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let intent_uuid = Uuid::parse_str(intent_id)?;
        let execution_uuid = Uuid::parse_str(execution_token)?;

        if let Some(reference) = external_reference {
            if reference.len() > 512 {
                return Err("external publish reference is too long".into());
            }
        }
        if let Some(error) = error_message {
            if error.len() > 4096 {
                return Err("publish error message is too long".into());
            }
        }

        let final_status = if success { "SUCCEEDED" } else { "FAILED" };
        let client = self.client.lock().await;
        let mut tx = client.transaction().await?;
        let changed = tx
            .execute(
                "UPDATE publish_intents
                    SET status=$4,
                        external_reference=$5,
                        error_message=$6,
                        execution_token=NULL,
                        locked_until=NULL
                  WHERE company_id=$1 AND id=$2
                    AND status='RUNNING' AND execution_token=$3",
                &[
                    &company_uuid,
                    &intent_uuid,
                    &execution_uuid,
                    &final_status,
                    &external_reference,
                    &error_message,
                ],
            )
            .await?;
        if changed == 0 {
            tx.rollback().await?;
            return Err("publish execution token is invalid or lease is no longer owned".into());
        }

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'PUBLISH_INTENT_COMPLETED',$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[
                &company_uuid,
                &intent_id,
                &format!("outbox:publish-completed:{intent_id}:{final_status}:{external_reference.unwrap_or("")}"),
                &serde_json::json!({
                    "intent_id": intent_id,
                    "status": final_status,
                    "external_reference": external_reference,
                    "error": error_message,
                }),
            ],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn revoke_publish_intent(
        &self,
        company_id: &str,
        intent_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let intent_uuid = Uuid::parse_str(intent_id)?;
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "UPDATE publish_intents
                    SET status='REVOKED',
                        approval_token_hash=NULL,
                        execution_token=NULL,
                        locked_until=NULL
                  WHERE company_id=$1 AND id=$2
                    AND status IN ('DRAFT','APPROVED')",
                &[&company_uuid, &intent_uuid],
            )
            .await?;
        if changed == 0 {
            return Err("publish intent cannot be revoked from its current state".into());
        }
        Ok(())
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

    pub async fn autonomy_controls(
        &self,
        company_id: &str,
    ) -> Result<AutonomyControlRecord, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let now_epoch: i64 = tx
            .query_one("SELECT EXTRACT(EPOCH FROM now())::bigint", &[])
            .await?
            .get(0);
        tx.execute(
            "INSERT INTO autonomy_control_state
             (company_id, emergency_stop_enabled, emergency_stop_reason,
              emergency_stop_actor, emergency_stop_changed_at_epoch,
              content_publish_daily, ads_spend_daily_minor, live_minutes_daily,
              outbound_messages_daily, autonomous_capital_daily_minor, updated_at_epoch)
             VALUES ($1,false,NULL,'system-default',$2,10,0,60,100,0,$2)
             ON CONFLICT(company_id) DO NOTHING",
            &[&company, &now_epoch],
        )
        .await?;
        let row = tx
            .query_one(
                "SELECT emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
                        emergency_stop_changed_at_epoch, content_publish_daily::text,
                        ads_spend_daily_minor::text, live_minutes_daily::text,
                        outbound_messages_daily::text, autonomous_capital_daily_minor::text,
                        updated_at_epoch, updated_at::text
                   FROM autonomy_control_state
                  WHERE company_id=$1",
                &[&company],
            )
            .await?;
        let controls = safety_controls_from_row(&row, company)?;
        tx.commit().await?;
        Ok(AutonomyControlRecord {
            controls,
            updated_at: row.get(11),
        })
    }

    pub async fn set_autonomy_controls(
        &self,
        company_id: &str,
        emergency_stop_enabled: bool,
        emergency_stop_reason: Option<&str>,
        actor: &str,
        budgets: &company_safety_controls::AutonomyBudgets,
    ) -> Result<AutonomyControlRecord, Box<dyn std::error::Error + Send + Sync>> {
        if actor.trim().is_empty() || actor.len() > 256 {
            return Err("autonomy control actor is invalid".into());
        }
        budgets.validate().map_err(|error| error.to_string())?;
        if emergency_stop_enabled
            && emergency_stop_reason.map(str::trim).filter(|v| !v.is_empty()).is_none()
        {
            return Err("enabled emergency stop requires a reason".into());
        }
        if let Some(reason) = emergency_stop_reason {
            if reason.trim().is_empty() || reason.len() > 1_024 {
                return Err("autonomy control reason is invalid".into());
            }
        }
        let emergency_stop_reason = if emergency_stop_enabled {
            emergency_stop_reason.map(str::trim)
        } else {
            None
        };

        let company = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let now_epoch: i64 = tx
            .query_one("SELECT EXTRACT(EPOCH FROM now())::bigint", &[])
            .await?
            .get(0);

        let previous = tx
            .query_opt(
                "SELECT emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
                        content_publish_daily::text, ads_spend_daily_minor::text,
                        live_minutes_daily::text, outbound_messages_daily::text,
                        autonomous_capital_daily_minor::text
                   FROM autonomy_control_state
                  WHERE company_id=$1
                  FOR UPDATE",
                &[&company],
            )
            .await?;

        let previous_json = previous
            .as_ref()
            .map(|row| {
                serde_json::json!({
                    "emergency_stop_enabled": row.get::<_, bool>(0),
                    "emergency_stop_reason": row.get::<_, Option<String>>(1),
                    "emergency_stop_actor": row.get::<_, String>(2),
                    "budgets": {
                        "content_publish_daily": row.get::<_, String>(3),
                        "ads_spend_daily_minor": row.get::<_, String>(4),
                        "live_minutes_daily": row.get::<_, String>(5),
                        "outbound_messages_daily": row.get::<_, String>(6),
                        "autonomous_capital_daily_minor": row.get::<_, String>(7)
                    }
                })
            })
            .unwrap_or(serde_json::Value::Null);

        tx.execute(
            "INSERT INTO autonomy_control_state
             (company_id, emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
              emergency_stop_changed_at_epoch, content_publish_daily, ads_spend_daily_minor,
              live_minutes_daily, outbound_messages_daily, autonomous_capital_daily_minor,
              updated_at_epoch)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8::numeric,$9::numeric,$10::numeric,$5)
             ON CONFLICT(company_id) DO UPDATE
             SET emergency_stop_enabled=EXCLUDED.emergency_stop_enabled,
                 emergency_stop_reason=EXCLUDED.emergency_stop_reason,
                 emergency_stop_actor=EXCLUDED.emergency_stop_actor,
                 emergency_stop_changed_at_epoch=CASE
                    WHEN autonomy_control_state.emergency_stop_enabled IS DISTINCT FROM EXCLUDED.emergency_stop_enabled
                      OR autonomy_control_state.emergency_stop_reason IS DISTINCT FROM EXCLUDED.emergency_stop_reason
                      OR autonomy_control_state.emergency_stop_actor IS DISTINCT FROM EXCLUDED.emergency_stop_actor
                    THEN EXCLUDED.emergency_stop_changed_at_epoch
                    ELSE autonomy_control_state.emergency_stop_changed_at_epoch
                 END,
                 content_publish_daily=EXCLUDED.content_publish_daily,
                 ads_spend_daily_minor=EXCLUDED.ads_spend_daily_minor,
                 live_minutes_daily=EXCLUDED.live_minutes_daily,
                 outbound_messages_daily=EXCLUDED.outbound_messages_daily,
                 autonomous_capital_daily_minor=EXCLUDED.autonomous_capital_daily_minor,
                 updated_at_epoch=EXCLUDED.updated_at_epoch",
            &[
                &company,
                &emergency_stop_enabled,
                &emergency_stop_reason,
                &actor.trim(),
                &now_epoch,
                &budgets.content_publish_daily.to_string(),
                &budgets.ads_spend_daily_minor.to_string(),
                &budgets.live_minutes_daily.to_string(),
                &budgets.outbound_messages_daily.to_string(),
                &budgets.autonomous_capital_daily_minor.to_string(),
            ],
        )
        .await?;

        {
            tx.execute(
                "INSERT INTO outbox_events
                 (company_id,event_type,aggregate_id,idempotency_key,payload)
                 VALUES ($1,'AUTONOMY_CONTROLS_CHANGED',$2,$3,$4)
                 ON CONFLICT(company_id,idempotency_key) DO NOTHING",
                &[
                    &company,
                    &company,
                    &format!("outbox:autonomy-controls:{company}:{now_epoch}"),
                    &serde_json::json!({
                        "emergency_stop_enabled": emergency_stop_enabled,
                        "actor": actor.trim(),
                        "reason": emergency_stop_reason,
                        "budgets": budgets
                    }),
                ],
            )
            .await?;
        }

        tx.execute(
            "INSERT INTO audit_log
             (company_id, actor_type, actor_id, action, resource_type, resource_id, decision, metadata)
             VALUES ($1,'CONTROL_PLANE',$2,'AUTONOMY_CONTROLS_CHANGED','AUTONOMY_CONTROL',$3,$4,$5)",
            &[
                &company,
                &actor.trim(),
                &company.to_string(),
                &if emergency_stop_enabled { "EMERGENCY_STOP_ON" } else { "EMERGENCY_STOP_OFF" },
                &serde_json::json!({
                    "previous": previous_json,
                    "current": {
                        "emergency_stop_enabled": emergency_stop_enabled,
                        "emergency_stop_reason": emergency_stop_reason,
                        "actor": actor.trim(),
                        "budgets": budgets
                    }
                }),
            ],
        )
        .await?;

        let row = tx
            .query_one(
                "SELECT emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
                        emergency_stop_changed_at_epoch, content_publish_daily::text,
                        ads_spend_daily_minor::text, live_minutes_daily::text,
                        outbound_messages_daily::text, autonomous_capital_daily_minor::text,
                        updated_at_epoch, updated_at::text
                   FROM autonomy_control_state
                  WHERE company_id=$1",
                &[&company],
            )
            .await?;
        let controls = safety_controls_from_row(&row, company)?;
        let created_at: String = row.get(11);
        tx.commit().await?;
        Ok(AutonomyControlRecord { controls, updated_at: created_at })
    }

    pub async fn autonomy_budget_statuses(
        &self,
        company_id: &str,
        now_epoch: i64,
    ) -> Result<Vec<company_safety_controls::BudgetStatus>, Box<dyn std::error::Error + Send + Sync>> {
        if now_epoch <= 0 {
            return Err("autonomy budget time must be positive".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let period = company_safety_controls::period_start_epoch(now_epoch)
            .map_err(|error| error.to_string())?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        tx.execute(
            "INSERT INTO autonomy_control_state
             (company_id, emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
              emergency_stop_changed_at_epoch, content_publish_daily, ads_spend_daily_minor,
              live_minutes_daily, outbound_messages_daily, autonomous_capital_daily_minor,
              updated_at_epoch)
             VALUES ($1,false,NULL,'system-default',$2,10,0,60,100,0,$2)
             ON CONFLICT(company_id) DO NOTHING",
            &[&company, &now_epoch],
        )
        .await?;
        let controls = load_safety_controls_for_tx(&tx, company).await?;
        let rows = tx
            .query(
                "SELECT budget_kind, used::text
                   FROM autonomy_budget_usage
                  WHERE company_id=$1 AND period_start_epoch=$2",
                &[&company, &period],
            )
            .await?;
        let mut used_by_kind = std::collections::HashMap::new();
        for row in rows {
            let kind = company_safety_controls::BudgetKind::parse(
                row.get::<_, String>(0).as_str(),
            )
            .ok_or("invalid stored autonomy budget kind")?;
            used_by_kind.insert(kind, parse_i128_numeric(&row.get::<_, String>(1))?);
        }
        tx.rollback().await?;
        Ok(company_safety_controls::BudgetKind::ALL
            .into_iter()
            .map(|kind| {
                let daily_limit = controls.budgets.limit(kind);
                let used = *used_by_kind.get(&kind).unwrap_or(&0);
                let remaining = if controls.emergency_stop.enabled {
                    0
                } else {
                    daily_limit.saturating_sub(used).max(0)
                };
                company_safety_controls::BudgetStatus {
                    kind,
                    period_start_epoch: period,
                    daily_limit,
                    used,
                    remaining,
                }
            })
            .collect())
    }

    pub async fn autonomy_budget_remaining(
        &self,
        company_id: &str,
        kind: company_safety_controls::BudgetKind,
        now_epoch: i64,
    ) -> Result<i128, Box<dyn std::error::Error + Send + Sync>> {
        if now_epoch <= 0 {
            return Err("autonomy budget time must be positive".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let period = company_safety_controls::period_start_epoch(now_epoch)
            .map_err(|error| error.to_string())?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        tx.execute(
            "INSERT INTO autonomy_control_state
             (company_id, emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
              emergency_stop_changed_at_epoch, content_publish_daily, ads_spend_daily_minor,
              live_minutes_daily, outbound_messages_daily, autonomous_capital_daily_minor,
              updated_at_epoch)
             VALUES ($1,false,NULL,'system-default',$2,10,0,60,100,0,$2)
             ON CONFLICT(company_id) DO NOTHING",
            &[&company, &now_epoch],
        )
        .await?;
        let controls = load_safety_controls_for_tx(&tx, company).await?;
        let used = tx
            .query_opt(
                "SELECT used::text
                   FROM autonomy_budget_usage
                  WHERE company_id=$1 AND budget_kind=$2 AND period_start_epoch=$3",
                &[&company, &kind.as_str(), &period],
            )
            .await?
            .map(|row| parse_i128_numeric(&row.get::<_, String>(0)))
            .transpose()?
            .unwrap_or(0);
        tx.rollback().await?;
        if controls.emergency_stop.enabled {
            return Ok(0);
        }
        Ok(controls.budgets.limit(kind).saturating_sub(used).max(0))
    }

    async fn consume_autonomy_budget_tx(
        tx: &tokio_postgres::Transaction<'_>,
        company: Uuid,
        kind: company_safety_controls::BudgetKind,
        amount: i128,
        idempotency_key: &str,
        now_epoch: i64,
    ) -> Result<company_safety_controls::BudgetDecision, Box<dyn std::error::Error + Send + Sync>> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 256 {
            return Err("autonomy budget idempotency key is invalid".into());
        }
        if amount <= 0 {
            return Err("autonomy budget amount must be positive".into());
        }
        let period = company_safety_controls::period_start_epoch(now_epoch)
            .map_err(|error| error.to_string())?;
        tx.execute(
            "INSERT INTO autonomy_control_state
             (company_id, emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
              emergency_stop_changed_at_epoch, content_publish_daily, ads_spend_daily_minor,
              live_minutes_daily, outbound_messages_daily, autonomous_capital_daily_minor,
              updated_at_epoch)
             VALUES ($1,false,NULL,'system-default',$2,10,0,60,100,0,$2)
             ON CONFLICT(company_id) DO NOTHING",
            &[&company, &now_epoch],
        )
        .await?;

        if let Some(row) = tx
            .query_opt(
                "SELECT period_start_epoch, amount::text
                   FROM autonomy_budget_consumptions
                  WHERE company_id=$1 AND budget_kind=$2 AND idempotency_key=$3",
                &[&company, &kind.as_str(), &idempotency_key],
            )
            .await?
        {
            let replay_period: i64 = row.get(0);
            let requested = parse_i128_numeric(&row.get::<_, String>(1))?;
            let used_after = parse_i128_numeric(
                &tx.query_one(
                    "SELECT used::text
                       FROM autonomy_budget_usage
                      WHERE company_id=$1 AND budget_kind=$2 AND period_start_epoch=$3",
                    &[&company, &kind.as_str(), &replay_period],
                )
                .await?
                .get::<_, String>(0),
            )?;
            let controls = load_safety_controls_for_tx(tx, company).await?;
            let daily_limit = controls.budgets.limit(kind);
            let used_before = used_after
                .checked_sub(requested)
                .ok_or("autonomy budget replay accounting underflow")?;
            return Ok(company_safety_controls::BudgetDecision {
                kind,
                period_start_epoch: replay_period,
                daily_limit,
                used_before,
                requested,
                remaining_after: daily_limit.saturating_sub(used_after).max(0),
                allowed: true,
                reason: "idempotent replay: consumption already recorded".into(),
            });
        }

        let controls = load_safety_controls_for_tx(tx, company).await?;
        tx.execute(
            "INSERT INTO autonomy_budget_usage
             (company_id,budget_kind,period_start_epoch,used)
             VALUES ($1,$2,$3,0)
             ON CONFLICT(company_id,budget_kind,period_start_epoch) DO NOTHING",
            &[&company, &kind.as_str(), &period],
        )
        .await?;
        let used = parse_i128_numeric(
            &tx.query_one(
                "SELECT used::text
                   FROM autonomy_budget_usage
                  WHERE company_id=$1 AND budget_kind=$2 AND period_start_epoch=$3
                  FOR UPDATE",
                &[&company, &kind.as_str(), &period],
            )
            .await?
            .get::<_, String>(0),
        )?;

        let decision = company_safety_controls::decide_budget(
            &controls,
            kind,
            now_epoch,
            used,
            amount,
        )
        .map_err(|error| error.to_string())?;
        if !decision.allowed {
            return Ok(decision);
        }

        let next_used = used
            .checked_add(amount)
            .ok_or("autonomy budget usage overflow")?;
        tx.execute(
            "UPDATE autonomy_budget_usage
                SET used=$4::numeric, updated_at=now()
              WHERE company_id=$1 AND budget_kind=$2 AND period_start_epoch=$3",
            &[&company, &kind.as_str(), &period, &next_used.to_string()],
        )
        .await?;
        tx.execute(
            "INSERT INTO autonomy_budget_consumptions
             (company_id,budget_kind,period_start_epoch,amount,idempotency_key)
             VALUES ($1,$2,$3,$4::numeric,$5)",
            &[&company, &kind.as_str(), &period, &amount.to_string(), &idempotency_key],
        )
        .await?;
        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'AUTONOMY_BUDGET_CONSUMED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &company,
                &format!("outbox:autonomy-budget:{kind:?}:{company}:{idempotency_key}"),
                &serde_json::json!({
                    "kind": kind.as_str(),
                    "amount": amount,
                    "period_start_epoch": period,
                    "used_before": used,
                    "used_after": next_used
                }),
            ],
        )
        .await?;

        Ok(decision)
    }

    pub async fn consume_autonomy_budget(
        &self,
        company_id: &str,
        kind: company_safety_controls::BudgetKind,
        amount: i128,
        idempotency_key: &str,
        now_epoch: i64,
    ) -> Result<company_safety_controls::BudgetDecision, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let decision = Self::consume_autonomy_budget_tx(
            &tx,
            company,
            kind,
            amount,
            idempotency_key,
            now_epoch,
        )
        .await?;
        tx.commit().await?;
        Ok(decision)
    }

    pub async fn create_tiktok_oauth_state(
        &self,
        company_id: &str,
        state_hash: &str,
        redirect_uri: &str,
        scopes: &str,
        expires_at_epoch: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        if state_hash.trim().is_empty() || state_hash.len() > 256 {
            return Err("TikTok OAuth state hash is invalid".into());
        }
        if redirect_uri.trim().is_empty() || redirect_uri.len() > 2048 {
            return Err("TikTok OAuth redirect URI is invalid".into());
        }
        if scopes.trim().is_empty() || scopes.len() > 4096 {
            return Err("TikTok OAuth scopes are invalid".into());
        }
        if expires_at_epoch <= 0 {
            return Err("TikTok OAuth state expiry is invalid".into());
        }
        let client = self.client.lock().await;
        client
            .execute(
                "DELETE FROM tiktok_oauth_states
                  WHERE company_id=$1 AND expires_at_epoch < $2",
                &[&company, &expires_at_epoch],
            )
            .await?;
        client
            .execute(
                "INSERT INTO tiktok_oauth_states
                 (id,company_id,state_hash,redirect_uri,scopes,expires_at_epoch)
                 VALUES ($1,$2,$3,$4,$5,$6)
                 ON CONFLICT(company_id,state_hash) DO UPDATE
                 SET redirect_uri=EXCLUDED.redirect_uri,
                     scopes=EXCLUDED.scopes,
                     expires_at_epoch=EXCLUDED.expires_at_epoch",
                &[
                    &Uuid::new_v4(),
                    &company,
                    &state_hash.trim(),
                    &redirect_uri.trim(),
                    &scopes.trim(),
                    &expires_at_epoch,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn consume_tiktok_oauth_state(
        &self,
        company_id: &str,
        state_hash: &str,
        now_epoch: i64,
    ) -> Result<Option<(String, String)>, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        if state_hash.trim().is_empty() || now_epoch <= 0 {
            return Err("TikTok OAuth state identity/time is invalid".into());
        }
        let client = self.client.lock().await;
        let row = client
            .query_opt(
                "DELETE FROM tiktok_oauth_states
                  WHERE company_id=$1
                    AND state_hash=$2
                    AND expires_at_epoch >= $3
                  RETURNING redirect_uri,scopes",
                &[&company, &state_hash.trim(), &now_epoch],
            )
            .await?;
        Ok(row.map(|value| (value.get(0), value.get(1))))
    }

    pub async fn save_tiktok_token_set(
        &self,
        company_id: &str,
        token: &company_tiktok_auth::TokenSet,
        cipher: &company_tiktok_auth::TokenCipher,
    ) -> Result<TikTokConnectionRecord, Box<dyn std::error::Error + Send + Sync>> {
        token.validate().map_err(|error| error.to_string())?;
        let company = Uuid::parse_str(company_id)?;
        let now_epoch: i64 = time::OffsetDateTime::now_utc().unix_timestamp();
        let access_expires_at_epoch = token.access_expires_at(now_epoch).map_err(|error| error.to_string())?;
        let refresh_expires_at_epoch = token.refresh_expires_at(now_epoch).map_err(|error| error.to_string())?;
        let encrypted_access_token = cipher.encrypt(company, &token.access_token).map_err(|error| error.to_string())?;
        let encrypted_refresh_token = cipher.encrypt(company, &token.refresh_token).map_err(|error| error.to_string())?;

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        tx.execute(
            "INSERT INTO tiktok_oauth_connections
             (company_id,open_id,encrypted_access_token,encrypted_refresh_token,
              access_token_expires_at_epoch,refresh_token_expires_at_epoch,
              scopes,token_type,status,last_error,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'ACTIVE',NULL,now())
             ON CONFLICT(company_id) DO UPDATE
             SET open_id=EXCLUDED.open_id,
                 encrypted_access_token=EXCLUDED.encrypted_access_token,
                 encrypted_refresh_token=EXCLUDED.encrypted_refresh_token,
                 access_token_expires_at_epoch=EXCLUDED.access_token_expires_at_epoch,
                 refresh_token_expires_at_epoch=EXCLUDED.refresh_token_expires_at_epoch,
                 scopes=EXCLUDED.scopes,
                 token_type=EXCLUDED.token_type,
                 status='ACTIVE',
                 last_error=NULL,
                 updated_at=now()",
            &[
                &company,
                &token.open_id,
                &encrypted_access_token,
                &encrypted_refresh_token,
                &access_expires_at_epoch,
                &refresh_expires_at_epoch,
                &token.scope,
                &token.token_type,
            ],
        )
        .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'TIKTOK_OAUTH_CONNECTED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &token.open_id,
                &format!("outbox:tiktok-oauth:connected:{}:{}", company, access_expires_at_epoch),
                &serde_json::json!({
                    "open_id": token.open_id,
                    "scopes": token.scope,
                    "access_token_expires_at_epoch": access_expires_at_epoch,
                    "refresh_token_expires_at_epoch": refresh_expires_at_epoch
                }),
            ],
        )
        .await?;

        tx.execute(
            "INSERT INTO audit_log
             (company_id,actor_type,actor_id,action,resource_type,resource_id,decision,metadata)
             VALUES ($1,'SYSTEM','tiktok-oauth','TIKTOK_OAUTH_CONNECTED','TIKTOK_CONNECTION',$2,'ACTIVE',$3)",
            &[
                &company,
                &token.open_id,
                &serde_json::json!({
                    "scopes": token.scope,
                    "access_token_expires_at_epoch": access_expires_at_epoch,
                    "refresh_token_expires_at_epoch": refresh_expires_at_epoch
                }),
            ],
        )
        .await?;

        let row = tx
            .query_one(
                "SELECT open_id,scopes,token_type,access_token_expires_at_epoch,
                        refresh_token_expires_at_epoch,status,last_error,updated_at::text
                   FROM tiktok_oauth_connections
                  WHERE company_id=$1",
                &[&company],
            )
            .await?;
        let record = tiktok_connection_from_row(company, &row)?;
        tx.commit().await?;
        Ok(record)
    }

    pub async fn tiktok_oauth_status(
        &self,
        company_id: &str,
    ) -> Result<Option<TikTokConnectionRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_opt(
                "SELECT open_id,scopes,token_type,access_token_expires_at_epoch,
                        refresh_token_expires_at_epoch,status,last_error,updated_at::text
                   FROM tiktok_oauth_connections
                  WHERE company_id=$1",
                &[&company],
            )
            .await?;
        row.map(|value| tiktok_connection_from_row(company, &value)).transpose()
    }

    pub async fn tiktok_oauth_token_material(
        &self,
        company_id: &str,
        cipher: &company_tiktok_auth::TokenCipher,
    ) -> Result<Option<TikTokTokenMaterial>, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_opt(
                "SELECT open_id,encrypted_access_token,encrypted_refresh_token,
                        access_token_expires_at_epoch,refresh_token_expires_at_epoch,
                        scopes,token_type,status
                   FROM tiktok_oauth_connections
                  WHERE company_id=$1",
                &[&company],
            )
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let status: String = row.get(7);
        if status != "ACTIVE" {
            return Ok(None);
        }
        let access_encrypted: String = row.get(1);
        let refresh_encrypted: String = row.get(2);
        Ok(Some(TikTokTokenMaterial {
            company_id: company,
            open_id: row.get(0),
            access_token: cipher.decrypt(company, &access_encrypted).map_err(|error| error.to_string())?,
            refresh_token: cipher.decrypt(company, &refresh_encrypted).map_err(|error| error.to_string())?,
            access_token_expires_at_epoch: row.get(3),
            refresh_token_expires_at_epoch: row.get(4),
            scopes: row.get(5),
            token_type: row.get(6),
        }))
    }

    pub async fn mark_tiktok_reauth_required(
        &self,
        company_id: &str,
        error: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let error = error.trim();
        if error.is_empty() || error.len() > 2048 {
            return Err("TikTok reauth error is invalid".into());
        }
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "UPDATE tiktok_oauth_connections
                    SET status='REAUTH_REQUIRED',last_error=$2,updated_at=now()
                  WHERE company_id=$1",
                &[&company, &error],
            )
            .await?;
        if changed == 1 {
            client.execute(
                "INSERT INTO audit_log
                 (company_id,actor_type,actor_id,action,resource_type,resource_id,decision,metadata)
                 VALUES ($1,'SYSTEM','tiktok-oauth','TIKTOK_OAUTH_REAUTH_REQUIRED','TIKTOK_CONNECTION',$2,'REAUTH_REQUIRED',$3)",
                &[
                    &company,
                    &company.to_string(),
                    &serde_json::json!({"error": error}),
                ],
            )
            .await?;
        }
        Ok(())
    }

    pub async fn mark_tiktok_revoked(
        &self,
        company_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "UPDATE tiktok_oauth_connections
                    SET status='REVOKED',last_error=NULL,updated_at=now()
                  WHERE company_id=$1",
                &[&company],
            )
            .await?;
        if changed == 1 {
            client
                .execute(
                    "INSERT INTO audit_log
                     (company_id,actor_type,actor_id,action,resource_type,resource_id,decision,metadata)
                     VALUES ($1,'SYSTEM','tiktok-oauth','TIKTOK_OAUTH_REVOKED','TIKTOK_CONNECTION',$2,'REVOKED','{}'::jsonb)",
                    &[&company, &company.to_string()],
                )
                .await?;
            client
                .execute(
                    "INSERT INTO outbox_events
                     (company_id,event_type,aggregate_id,idempotency_key,payload)
                     VALUES ($1,'TIKTOK_OAUTH_REVOKED',$2,$3,$4)
                     ON CONFLICT(company_id,idempotency_key) DO NOTHING",
                    &[
                        &company,
                        &company.to_string(),
                        &format!("outbox:tiktok-oauth:revoked:{}:{}", company, time::OffsetDateTime::now_utc().unix_timestamp()),
                        &serde_json::json!({"company_id": company}),
                    ],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn create_capital_allocation_plan(
        &self,
        company_id: &str,
        plan_key: &str,
        policy: &company_capital::CapitalPolicy,
        candidates: &[company_capital::CapitalCandidate],
    ) -> Result<CapitalAllocationRecord, Box<dyn std::error::Error + Send + Sync>> {
        if plan_key.trim().is_empty() || plan_key.len() > 256 {
            return Err("capital allocation plan key is invalid".into());
        }
        company_capital::validate_policy(policy).map_err(|error| error.to_string())?;
        let company = Uuid::parse_str(company_id)?;
        let mut seen_candidate_ids = std::collections::HashSet::new();
        for candidate in candidates {
            company_capital::validate_candidate(candidate).map_err(|error| error.to_string())?;
            if !seen_candidate_ids.insert(candidate.candidate_id) {
                return Err("duplicate capital candidate id".into());
            }
        }
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let authoritative = tx
            .query_opt(
                "SELECT state
                   FROM company_state_snapshots
                  WHERE company_id=$1
                  FOR SHARE",
                &[&company],
            )
            .await?
            .ok_or("authoritative company snapshot is unavailable")?
            .get::<_, serde_json::Value>(0);
        let authoritative: CompanySnapshot = serde_json::from_value(authoritative)?;
        if policy.company_status != authoritative.status
            || policy.cash_available_minor != authoritative.cash_minor.max(0)
            || policy.runway_days != authoritative.runway_days.max(0)
        {
            return Err("capital policy does not match the authoritative company snapshot".into());
        }

        for candidate in candidates {
            let unit_id = Uuid::parse_str(&candidate.unit_id)
                .map_err(|_| "capital candidate unit_id must be a business unit UUID".to_string())?;
            let owns_unit = tx
                .query_opt(
                    "SELECT 1
                       FROM business_units
                      WHERE company_id=$1
                        AND id=$2
                        AND lifecycle IN ('TESTING','GROWING','STABLE')
                      FOR SHARE",
                    &[&company, &unit_id],
                )
                .await?
                .is_some();
            if !owns_unit {
                return Err("capital candidate references a business unit outside the company".into());
            }
        }

        let mut fingerprint_payload = serde_json::Map::new();
        fingerprint_payload.insert("policy".into(), serde_json::to_value(policy)?);
        fingerprint_payload.insert("candidates".into(), serde_json::to_value(candidates)?);
        let fingerprint_bytes = serde_json::to_vec(&fingerprint_payload)?;
        let inputs_hash = format!(
            "sha256:{}",
            Sha256::digest(fingerprint_bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let plan_id = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("company-capital:{company}:{plan_key}").as_bytes(),
        );
        let plan = company_capital::plan_with_id(plan_id, policy, candidates)
            .map_err(|error| error.to_string())?;
        let policy_json = serde_json::to_value(policy)?;
        let plan_json = serde_json::to_value(&plan)?;

        if let Some(row) = tx
            .query_opt(
                "SELECT plan_json,policy_json,inputs_hash,created_at::text
                   FROM capital_allocation_plans
                  WHERE company_id=$1 AND plan_key=$2",
                &[&company, &plan_key],
            )
            .await?
        {
            let stored_plan: company_capital::CapitalAllocationPlan = serde_json::from_value(row.get(0))?;
            let stored_policy: company_capital::CapitalPolicy = serde_json::from_value(row.get(1))?;
            let stored_inputs_hash: String = row.get(2);
            let created_at: String = row.get(3);
            if stored_plan != plan || stored_policy != *policy || stored_inputs_hash != inputs_hash {
                return Err("capital allocation plan key already exists with different evidence".into());
            }
            tx.rollback().await?;
            return Ok(CapitalAllocationRecord {
                plan: stored_plan,
                policy: stored_policy,
                created_at,
            });
        }

        let plan_row = tx.query_one(
            "INSERT INTO capital_allocation_plans
             (id,company_id,plan_key,inputs_hash,policy_json,total_capital_minor,planned_capital_minor,unallocated_minor)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8::numeric)
             RETURNING created_at::text",
            &[
                &plan.plan_id,
                &company,
                &plan_key,
                &inputs_hash,
                &policy_json,
                &plan.total_capital_minor.to_string(),
                &plan.planned_capital_minor.to_string(),
                &plan.unallocated_minor.to_string(),
            ],
        )
        .await?;

        for candidate in candidates {
            tx.execute(
                "INSERT INTO capital_allocation_candidates
                 (id,company_id,plan_id,candidate_key,candidate_json)
                 VALUES ($1,$2,$3,$4,$5)",
                &[
                    &candidate.candidate_id,
                    &company,
                    &plan.plan_id,
                    &candidate.candidate_id.to_string(),
                    &serde_json::to_value(candidate)?,
                ],
            )
            .await?;
        }

        let candidate_map = candidates
            .iter()
            .map(|candidate| (candidate.candidate_id, candidate))
            .collect::<std::collections::HashMap<_, _>>();

        for decision in &plan.decisions {
            let candidate = candidate_map
                .get(&decision.candidate_id)
                .ok_or("capital decision references unknown candidate")?;
            tx.execute(
                "INSERT INTO capital_allocation_decisions
                 (id,company_id,plan_id,candidate_id,decision,score_bps,allocation_minor,reason)
                 VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8)",
                &[
                    &Uuid::new_v4(),
                    &company,
                    &plan.plan_id,
                    &candidate.candidate_id,
                    &format!("{:?}", decision.status).to_uppercase(),
                    &(decision.score_bps as i32),
                    &decision.allocation_minor.to_string(),
                    &decision.reason,
                ],
            )
            .await?;
        }

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'CAPITAL_ALLOCATION_PLAN_CREATED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &plan.plan_id,
                &format!("outbox:capital-plan:{plan_key}"),
                &serde_json::json!({
                    "plan_id": plan.plan_id,
                    "plan_key": plan_key,
                    "total_capital_minor": plan.total_capital_minor,
                    "planned_capital_minor": plan.planned_capital_minor,
                    "unallocated_minor": plan.unallocated_minor
                }),
            ],
        )
        .await?;

        let created_at: String = plan_row.get(0);
        tx.commit().await?;
        Ok(CapitalAllocationRecord {
            plan,
            policy: *policy,
            created_at,
        })
    }

    pub async fn latest_capital_allocation_plan(
        &self,
        company_id: &str,
    ) -> Result<Option<CapitalAllocationRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_opt(
                "SELECT plan_json, policy_json, created_at::text
                   FROM capital_allocation_plans
                  WHERE company_id=$1
                  ORDER BY created_at DESC, id DESC
                  LIMIT 1",
                &[&company],
            )
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(CapitalAllocationRecord {
            plan: serde_json::from_value(row.get(0))?,
            policy: serde_json::from_value(row.get(1))?,
            created_at: row.get(2),
        }))
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
        let observed_at_epoch = parse_rfc3339_epoch(&event.occurred_at)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        tx.execute(
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

        let content_ref = event.content_id.trim();
        let product_ref = event.product_id.trim();
        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company_id,
                company_revenue_graph::RevenueNodeType::Content,
                content_ref,
                "PROMOTES",
                company_revenue_graph::RevenueNodeType::Product,
                product_ref,
                None,
                None,
                10_000,
                &format!("affiliate:click:{}", event.click_id),
                graph_source(&event.source)?,
                observed_at_epoch,
            ),
        )
        .await?;

        if !event.source.trim().is_empty() {
            record_revenue_graph_edge_tx(
                &tx,
                &new_graph_edge(
                    company_id,
                    company_revenue_graph::RevenueNodeType::Traffic,
                    graph_source(&event.source)?,
                    "DRIVES",
                    company_revenue_graph::RevenueNodeType::Content,
                    content_ref,
                    None,
                    None,
                    10_000,
                    &format!("affiliate:click:{}", event.click_id),
                    event.source.trim(),
                    observed_at_epoch,
                ),
            )
            .await?;
        }

        tx.commit().await?;
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

        let observed_at_epoch = parse_rfc3339_epoch(&event.occurred_at)?;
        let net_order_value = event
            .order_value_minor
            .checked_sub(event.refunded_minor)
            .ok_or("affiliate order value underflow")?;
        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company_id,
                company_revenue_graph::RevenueNodeType::Order,
                &event.order_id,
                "PURCHASES",
                company_revenue_graph::RevenueNodeType::Product,
                &event.product_id,
                Some(net_order_value),
                Some(&currency),
                if event.cancelled { 2_000 } else { 10_000 },
                &format!("affiliate:conversion:{}", event.conversion_id),
                graph_source(&event.source)?,
                observed_at_epoch,
            ),
        )
        .await?;

        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company_id,
                company_revenue_graph::RevenueNodeType::Order,
                &event.order_id,
                "REPORTS_COMMISSION",
                company_revenue_graph::RevenueNodeType::Commission,
                &event.conversion_id,
                Some(if event.cancelled { 0 } else { event.commission_minor }),
                Some(&currency),
                5_000,
                &format!("affiliate:conversion:{}", event.conversion_id),
                graph_source(&event.source)?,
                observed_at_epoch,
            ),
        )
        .await?;

        for attribution in &reconciled.attributed {
            record_revenue_graph_edge_tx(
                &tx,
                &new_graph_edge(
                    company_id,
                    company_revenue_graph::RevenueNodeType::Content,
                    &attribution.content_id,
                    "ATTRIBUTED_TO",
                    company_revenue_graph::RevenueNodeType::Order,
                    &event.order_id,
                    Some(attribution.attributed_order_value_minor),
                    Some(&currency),
                    attribution.confidence_bps,
                    &format!("affiliate:conversion:{}", event.conversion_id),
                    graph_source(&event.source)?,
                    observed_at_epoch,
                ),
            )
            .await?;
        }

        let target_recognized = 0_i128;

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

    pub async fn verify_affiliate_conversion(
        &self,
        company_id: &str,
        conversion_id: &str,
        status: affiliate_attribution::ProviderVerificationStatus,
        verified_commission_minor: i128,
        verified_at: Option<&str>,
        verification_source: &str,
    ) -> Result<
        affiliate_attribution::ReconciledConversion,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        if conversion_id.trim().is_empty() || verification_source.trim().is_empty() {
            return Err("affiliate provider verification requires identifiers".into());
        }
        if verified_commission_minor < 0 {
            return Err("verified affiliate commission cannot be negative".into());
        }
        if !status.authorizes_revenue() && verified_commission_minor != 0 {
            return Err(
                "non-approved provider verification cannot carry a positive verified commission"
                    .into(),
            );
        }
        if let Some(value) = verified_at {
            time::OffsetDateTime::parse(
                value,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|error| format!("invalid provider verification timestamp: {error}"))?;
        }

        let company_uuid = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let row = tx
            .query_opt(
                "SELECT commission_minor::text,
                        reconciliation_status,
                        reconciliation_variance_minor::text
                   FROM affiliate_conversions
                  WHERE company_id=$1 AND conversion_id=$2
                  FOR UPDATE",
                &[&company_uuid, &conversion_id],
            )
            .await?
            .ok_or("affiliate conversion not found")?;

        let reported_commission = parse_i128_numeric(&row.get::<_, String>(0))?;
        let reconciliation_status = parse_reconciliation_status(&row.get::<_, String>(1))?;
        let variance = parse_i128_numeric(&row.get::<_, String>(2))?;

        let attributed_total = parse_i128_numeric(
            &tx
                .query_one(
                    "SELECT COALESCE(SUM(attributed_commission_minor),0)::text
                       FROM affiliate_attributions
                      WHERE company_id=$1 AND conversion_id=$2",
                    &[&company_uuid, &conversion_id],
                )
                .await?
                .get::<_, String>(0),
        )?;

        let attribution_is_verified = attributed_total == reported_commission;

        tx.execute(
            "UPDATE affiliate_conversions
                SET provider_verification_status=$3,
                    provider_verified_commission_minor=$4::numeric,
                    provider_verified_at=$5,
                    provider_verification_source=$6
              WHERE company_id=$1 AND conversion_id=$2",
            &[
                &company_uuid,
                &conversion_id,
                &status.as_str(),
                &verified_commission_minor.to_string(),
                &verified_at,
                &verification_source,
            ],
        )
        .await?;

        let target_recognized = if attribution_is_verified && status.authorizes_revenue() {
            verified_commission_minor
        } else {
            0
        };

        let recognition_row = tx
            .query_opt(
                "SELECT recognized_minor::text, ledger_transaction_id
                   FROM affiliate_revenue_recognition
                  WHERE company_id=$1 AND conversion_id=$2
                  FOR UPDATE",
                &[&company_uuid, &conversion_id],
            )
            .await?;

        let prior_recognized = recognition_row
            .as_ref()
            .map(|item| parse_i128_numeric(&item.get::<_, String>(0)))
            .transpose()?
            .unwrap_or(0);

        let delta = target_recognized
            .checked_sub(prior_recognized)
            .ok_or("provider verification recognition delta overflow")?;

        let mut ledger_transaction_id = recognition_row
            .as_ref()
            .and_then(|item| item.get::<_, Option<Uuid>>(1));

        let currency: String = tx
            .query_one(
                "SELECT base_currency FROM companies WHERE id=$1",
                &[&company_uuid],
            )
            .await?
            .get(0);

        if verified_commission_minor > 0 && status.authorizes_revenue() {
            let observed_at_epoch = verified_at
                .map(parse_rfc3339_epoch)
                .transpose()?
                .unwrap_or_else(|| time::OffsetDateTime::now_utc().unix_timestamp());
            let order_id: String = tx
                .query_one(
                    "SELECT order_id FROM affiliate_conversions
                      WHERE company_id=$1 AND conversion_id=$2",
                    &[&company_uuid, &conversion_id],
                )
                .await?
                .get(0);
            record_revenue_graph_edge_tx(
                &tx,
                &new_graph_edge(
                    company_uuid,
                    company_revenue_graph::RevenueNodeType::Order,
                    &order_id,
                    "VERIFIED_COMMISSION",
                    company_revenue_graph::RevenueNodeType::Commission,
                    &conversion_id,
                    Some(verified_commission_minor),
                    Some(&currency),
                    10_000,
                    &format!("affiliate:provider-verification:{}:{}", conversion_id, status.as_str()),
                    graph_source(verification_source)?,
                    observed_at_epoch,
                ),
            )
            .await?;
        }

        if delta != 0 {
            let (_cash_account, receivable_account, revenue_account) =
                ensure_affiliate_accounts(&tx, company_uuid, &currency).await?;
            let transaction_id = Uuid::new_v4();
            ledger_transaction_id = Some(transaction_id);
            let amount = delta.unsigned_abs().to_string();

            tx.execute(
                "INSERT INTO ledger_transactions
                 (id, company_id, description, idempotency_key)
                 VALUES ($1,$2,'affiliate provider verification adjustment',$3)",
                &[
                    &transaction_id,
                    &company_uuid,
                    &format!(
                        "affiliate:provider-verify:{conversion_id}:{status}:{delta}"
                    ),
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

            update_snapshot_financials(&tx, company_uuid, |snapshot| {
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
                        .ok_or("affiliate revenue reversal overflow".to_string())?;
                    snapshot.revenue_minor = snapshot
                        .revenue_minor
                        .checked_sub(decrease)
                        .ok_or("affiliate revenue reversal underflow".to_string())?;
                    snapshot.assets_minor = snapshot
                        .assets_minor
                        .checked_sub(decrease)
                        .ok_or("affiliate receivable reversal underflow".to_string())?;
                }
                Ok(())
            })
            .await?;
        }

        let recognition_status = if attribution_is_verified && status.authorizes_revenue() {
            "RECOGNIZED"
        } else if status == affiliate_attribution::ProviderVerificationStatus::Rejected {
            "REJECTED"
        } else {
            "PENDING"
        };

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
                &company_uuid,
                &conversion_id,
                &currency,
                &target_recognized.to_string(),
                &recognition_status,
                &ledger_transaction_id,
            ],
        )
        .await?;

        if target_recognized > 0 {
            let observed_at_epoch = verified_at
                .map(parse_rfc3339_epoch)
                .transpose()?
                .unwrap_or_else(|| time::OffsetDateTime::now_utc().unix_timestamp());
            record_revenue_graph_edge_tx(
                &tx,
                &new_graph_edge(
                    company_uuid,
                    company_revenue_graph::RevenueNodeType::Commission,
                    &conversion_id,
                    "RECOGNIZED_INTO",
                    company_revenue_graph::RevenueNodeType::Commission,
                    &format!("affiliate:recognized:{}", company_uuid),
                    Some(target_recognized),
                    Some(&currency),
                    10_000,
                    &format!("affiliate:provider-verification:{}:{}", conversion_id, status.as_str()),
                    graph_source(verification_source)?,
                    observed_at_epoch,
                ),
            )
            .await?;
        }

        let provider_event_key = format!(
            "outbox:affiliate:provider-verified:{conversion_id}:{}:{}:{}",
            status.as_str(),
            verified_commission_minor,
            verified_at.unwrap_or("")
        );
        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'AFFILIATE_PROVIDER_VERIFIED',$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[
                &company_uuid,
                &conversion_id,
                &provider_event_key,
                &serde_json::json!({
                    "conversion_id": conversion_id,
                    "status": status.as_str(),
                    "verified_commission_minor": verified_commission_minor,
                    "verification_source": verification_source,
                    "verified_at": verified_at,
                    "attribution_verified": attribution_is_verified,
                    "reported_commission_minor": reported_commission,
                }),
            ],
        )
        .await?;

        let attributed = load_affiliate_attributions(&tx, company_uuid, conversion_id).await?;
        tx.commit().await?;

        Ok(affiliate_attribution::ReconciledConversion {
            conversion_id: conversion_id.to_owned(),
            attributed,
            net_commission_minor: target_recognized,
            reconciliation_variance_minor: variance,
            status: reconciliation_status,
            idempotency_key: provider_event_key,
        })
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

        update_snapshot_financials(&tx, company_uuid, |snapshot| {
            snapshot.cash_minor = snapshot
                .cash_minor
                .checked_add(amount_minor)
                .ok_or("affiliate payout cash overflow".to_string())?;
            Ok(())
        })
        .await?;

        let occurred_text = occurred.format(&time::format_description::well_known::Rfc3339)?;
        tx.execute(
            "INSERT INTO affiliate_payouts
             (company_id,payout_id,currency,amount_minor,occurred_at,ledger_transaction_id)
             VALUES ($1,$2,$3,$4::numeric,$5,$6)",
            &[&company_uuid,&payout_id,&currency,&amount,&occurred_text,&payout_uuid],
        )
        .await?;

        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company_uuid,
                company_revenue_graph::RevenueNodeType::Commission,
                &format!("affiliate:recognized:{}", company_uuid),
                "SETTLES_TO_CASH",
                company_revenue_graph::RevenueNodeType::Cash,
                &format!("company:{}:cash", company_uuid),
                Some(amount_minor),
                Some(&company_currency),
                10_000,
                &format!("affiliate:payout:{}", payout_id),
                "affiliate-payout-ledger",
                occurred.unix_timestamp(),
            ),
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

    pub async fn claim_outbox_events(
        &self,
        company_id: &str,
        lease_owner: &str,
        lease_seconds: i64,
        max_attempts: i32,
        limit: i64,
    ) -> Result<Vec<OutboxEvent>, Box<dyn std::error::Error + Send + Sync>> {
        if lease_owner.trim().is_empty() || !(5..=300).contains(&lease_seconds)
            || !(1..=100).contains(&max_attempts) || !(1..=100).contains(&limit)
        {
            return Err("invalid outbox lease parameters".into());
        }
        let company_uuid = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let rows = tx
            .query(
                "SELECT id, company_id, event_type, aggregate_id, schema_version,
                        payload, attempts
                   FROM outbox_events
                  WHERE company_id=$1
                    AND published_at IS NULL
                    AND attempts < $2
                    AND (lease_until IS NULL OR lease_until <= now())
                  ORDER BY id ASC
                  FOR UPDATE SKIP LOCKED
                  LIMIT $3",
                &[&company_uuid, &max_attempts, &limit],
            )
            .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get(0);
            let company: Uuid = row.get(1);
            let attempts: i32 = row.get(6);
            tx.execute(
                "UPDATE outbox_events
                    SET attempts=attempts+1,
                        lease_owner=$2,
                        lease_until=now()+($3::double precision * interval '1 second'),
                        last_error=NULL
                  WHERE id=$1
                    AND published_at IS NULL",
                &[&id, &lease_owner, &lease_seconds],
            )
            .await?;

            events.push(OutboxEvent {
                id,
                company_id: company.to_string(),
                event_type: row.get(2),
                aggregate_id: row.get(3),
                schema_version: row.get(4),
                payload: row.get(5),
                attempts: attempts + 1,
            });
        }
        tx.commit().await?;
        Ok(events)
    }

    pub async fn mark_outbox_published(
        &self,
        event_id: i64,
        lease_owner: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if lease_owner.trim().is_empty() {
            return Err("outbox lease owner is required".into());
        }
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "UPDATE outbox_events
                    SET published_at=now(), lease_owner=NULL, lease_until=NULL, last_error=NULL
                  WHERE id=$1 AND published_at IS NULL AND lease_owner=$2",
                &[&event_id, &lease_owner],
            )
            .await?;
        if changed != 1 {
            return Err("outbox event lease is no longer owned".into());
        }
        Ok(())
    }

    pub async fn release_outbox_event(
        &self,
        event_id: i64,
        lease_owner: &str,
        error: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if lease_owner.trim().is_empty() {
            return Err("outbox lease owner is required".into());
        }
        let bounded = error.chars().take(4096).collect::<String>();
        let client = self.client.lock().await;
        let changed = client
            .execute(
                "UPDATE outbox_events
                    SET lease_owner=NULL, lease_until=NULL, last_error=$3
                  WHERE id=$1 AND published_at IS NULL AND lease_owner=$2",
                &[&event_id, &lease_owner, &bounded],
            )
            .await?;
        if changed != 1 {
            return Err("outbox event lease is no longer owned".into());
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

    pub async fn ready(
        &self,
        company_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.query_one("SELECT 1", &[]).await?;
        client
            .query_opt(
                "SELECT 1
                   FROM companies
                  WHERE id=$1",
                &[&id],
            )
            .await?
            .ok_or("company is not registered")?;
        client
            .query_opt(
                "SELECT 1
                   FROM company_state_snapshots
                  WHERE company_id=$1",
                &[&id],
            )
            .await?
            .ok_or("company state snapshot is unavailable")?;
        Ok(())
    }

    pub async fn create_tiktok_live_session(
        &self,
        session: &tiktok_live_engine::LiveSession,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        session.validate().map_err(|error| error.to_string())?;
        let company_id = session.company_id;
        let mode = format!("{:?}", session.mode).to_ascii_uppercase();
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO tiktok_live_sessions
                 (id, company_id, room_id, title, mode, started_at_epoch,
                  approved_for_external_publish)
                 VALUES ($1,$2,$3,$4,$5,$6,$7)
                 ON CONFLICT (id) DO NOTHING",
                &[
                    &session.id,
                    &company_id,
                    &session.room_id,
                    &session.title,
                    &mode,
                    &session.started_at_epoch,
                    &session.approved_for_external_publish,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn tiktok_live_mode(
        &self,
        company_id: &str,
        session_id: &str,
    ) -> Result<tiktok_live_engine::LiveMode, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let session_uuid = Uuid::parse_str(session_id)?;
        let client = self.client.lock().await;
        let mode: String = client
            .query_one(
                "SELECT mode FROM tiktok_live_sessions WHERE id=$1 AND company_id=$2",
                &[&session_uuid, &company_uuid],
            )
            .await?
            .get(0);
        match mode.as_str() {
            "SOLO" => Ok(tiktok_live_engine::LiveMode::Solo),
            "COHOST" | "CO_HOST" => Ok(tiktok_live_engine::LiveMode::CoHost),
            "PK" => Ok(tiktok_live_engine::LiveMode::Pk),
            "GAME" => Ok(tiktok_live_engine::LiveMode::Game),
            "STORY" => Ok(tiktok_live_engine::LiveMode::Story),
            "MUSIC" => Ok(tiktok_live_engine::LiveMode::Music),
            "SHOPPING" | "SHOP" => Ok(tiktok_live_engine::LiveMode::Shopping),
            other => Err(format!("unknown LIVE mode: {other}").into()),
        }
    }

    pub async fn record_tiktok_live_event(
        &self,
        company_id: &str,
        session_id: &str,
        event: &tiktok_live_engine::LiveEvent,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        event.validate().map_err(|error| error.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let session_uuid = Uuid::parse_str(session_id)?;
        if event.gift_quantity > i64::MAX as u64 || event.pk_score.is_some_and(|value| value > i64::MAX as u64) {
            return Err("LIVE event numeric fields exceed database range".into());
        }
        let kind = format!("{:?}", event.kind).to_ascii_uppercase();
        let gift_value = event.gift_value_minor.to_string();
        let client = self.client.lock().await;
        let session_exists = client
            .query_opt(
                "SELECT 1 FROM tiktok_live_sessions WHERE id=$1 AND company_id=$2",
                &[&session_uuid, &company_uuid],
            )
            .await?
            .is_some();
        if !session_exists {
            return Err("LIVE session is not owned by company".into());
        }
        let changed = client
            .execute(
                "INSERT INTO tiktok_live_events
                 (company_id, session_id, event_id, room_id, kind, user_id,
                  display_name, event_text, gift_id, gift_name, gift_quantity,
                  gift_value_minor, currency, pk_score, viewer_value_bps, occurred_at_epoch)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12::numeric,$13,$14,$15,$16)
                 ON CONFLICT (company_id, event_id) DO NOTHING",
                &[
                    &company_uuid,
                    &session_uuid,
                    &event.event_id,
                    &event.room_id,
                    &kind,
                    &event.user_id,
                    &event.display_name,
                    &event.text,
                    &event.gift_id,
                    &event.gift_name,
                    &(event.gift_quantity as i64),
                    &gift_value,
                    &event.currency,
                    &(event.pk_score.map(|value| value as i64)),
                    &event.viewer_value_bps.map(|value| value as i32),
                    &event.occurred_at_epoch,
                ],
            )
            .await?;
        Ok(changed == 1)
    }

    pub async fn record_tiktok_live_attention(
        &self,
        company_id: &str,
        session_id: &str,
        event: &tiktok_live_engine::LiveEvent,
    ) -> Result<company_live_attention::AttentionDecision, Box<dyn std::error::Error + Send + Sync>> {
        event.validate().map_err(|error| error.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let session_uuid = Uuid::parse_str(session_id)?;

        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let mode_value: String = tx
            .query_one(
                "SELECT mode
                   FROM tiktok_live_sessions
                  WHERE company_id=$1 AND id=$2
                  FOR UPDATE",
                &[&company_uuid, &session_uuid],
            )
            .await?
            .get(0);
        let mode = match mode_value.as_str() {
            "SOLO" => tiktok_live_engine::LiveMode::Solo,
            "COHOST" | "CO_HOST" => tiktok_live_engine::LiveMode::CoHost,
            "PK" => tiktok_live_engine::LiveMode::Pk,
            "GAME" => tiktok_live_engine::LiveMode::Game,
            "STORY" => tiktok_live_engine::LiveMode::Story,
            "MUSIC" => tiktok_live_engine::LiveMode::Music,
            "SHOPPING" | "SHOP" => tiktok_live_engine::LiveMode::Shopping,
            other => return Err(format!("unknown LIVE mode: {other}").into()),
        };

        tx.query_one(
            "SELECT 1 FROM tiktok_live_events
              WHERE company_id=$1 AND session_id=$2 AND event_id=$3",
            &[&company_uuid, &session_uuid, &event.event_id],
        ).await?;

        if let Some(row) = tx.query_opt(
            "SELECT id,company_id,session_id,event_id,action,reason,priority,decided_at_epoch,requires_human
               FROM live_attention_decisions
              WHERE company_id=$1 AND session_id=$2 AND event_id=$3",
            &[&company_uuid, &session_uuid, &event.event_id],
        ).await? {
            let decision = attention_decision_from_row(row)?;
            tx.commit().await?;
            return Ok(decision);
        }

        let policy = company_live_attention::AttentionPolicy::default();
        let now_epoch: i64 = tx
            .query_one(
                "SELECT EXTRACT(EPOCH FROM now())::bigint",
                &[],
            )
            .await?
            .get(0);
        let window_start = now_epoch.saturating_sub(policy.response_window_seconds);
        let context_row = tx
            .query_one(
                "SELECT
                    COALESCE(MAX(decided_at_epoch) FILTER (WHERE action IN ('RESPOND','ESCALATE')), 0),
                    COUNT(*) FILTER (WHERE action IN ('RESPOND','ESCALATE') AND decided_at_epoch >= $3)
                 FROM live_attention_decisions
                WHERE company_id=$1 AND session_id=$2",
                &[&company_uuid, &session_uuid, &window_start],
            )
            .await?;
        let last_value: i64 = context_row.get(0);
        let responses: i64 = context_row.get(1);
        let context = company_live_attention::AttentionContext {
            last_response_at_epoch: if last_value > 0 { Some(last_value) } else { None },
            window_started_at_epoch: Some(window_start),
            responses_in_window: responses.clamp(0, u32::MAX as i64) as u32,
        };

        let decision = company_live_attention::decide_attention(
            company_uuid,
            session_uuid,
            mode,
            event,
            &context,
            now_epoch,
            &policy,
        )
        .map_err(|error| error.to_string())?;

        tx.execute(
            "INSERT INTO live_attention_decisions
             (id,company_id,session_id,event_id,action,reason,priority,decided_at_epoch,requires_human,viewer_value_bps)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
             ON CONFLICT(company_id,session_id,event_id) DO NOTHING",
            &[
                &decision.decision_id,
                &company_uuid,
                &session_uuid,
                &decision.event_id,
                &attention_action_name(decision.action),
                &attention_reason_name(decision.reason),
                &(decision.priority as i16),
                &decision.decided_at_epoch,
                &decision.requires_human,
                &event.viewer_value_bps.map(|value| value as i32),
            ],
        ).await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'LIVE_ATTENTION_DECIDED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[
                &company_uuid,
                &session_uuid.to_string(),
                &format!("outbox:live-attention:{}:{}", session_uuid, decision.event_id),
                &serde_json::to_value(&decision)?,
            ],
        ).await?;

        let persisted = tx.query_one(
            "SELECT id,company_id,session_id,event_id,action,reason,priority,decided_at_epoch,requires_human
               FROM live_attention_decisions
              WHERE company_id=$1 AND session_id=$2 AND event_id=$3",
            &[&company_uuid, &session_uuid, &event.event_id],
        ).await?;
        let decision = attention_decision_from_row(persisted)?;
        tx.commit().await?;
        Ok(decision)
    }

    pub async fn tiktok_live_summary(
        &self,
        company_id: &str,
        session_id: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let session_uuid = Uuid::parse_str(session_id)?;
        let client = self.client.lock().await;
        let row = client
            .query_one(
                "SELECT
                    COUNT(*)::bigint,
                    COUNT(*) FILTER (WHERE kind='GIFT')::bigint,
                    COALESCE(SUM(gift_quantity) FILTER (WHERE kind='GIFT'),0)::text,
                    COALESCE(SUM(gift_value_minor) FILTER (WHERE kind='GIFT'),0)::text,
                    COUNT(*) FILTER (WHERE kind='COMMENT')::bigint,
                    COUNT(*) FILTER (WHERE kind='FOLLOW')::bigint,
                    COUNT(*) FILTER (WHERE kind='SHARE')::bigint,
                    COUNT(*) FILTER (WHERE kind='LIKE')::bigint
                 FROM tiktok_live_events
                 WHERE company_id=$1 AND session_id=$2",
                &[&company_uuid, &session_uuid],
            )
            .await?;
        let attention = client
            .query_one(
                "SELECT
                    COUNT(*) FILTER (WHERE action='RESPOND')::bigint,
                    COUNT(*) FILTER (WHERE action='DEFER')::bigint,
                    COUNT(*) FILTER (WHERE action='IGNORE')::bigint,
                    COUNT(*) FILTER (WHERE action='ESCALATE')::bigint,
                    COALESCE(MAX(decided_at_epoch),0)::bigint
                 FROM live_attention_decisions
                WHERE company_id=$1 AND session_id=$2",
                &[&company_uuid, &session_uuid],
            )
            .await?;

        Ok(serde_json::json!({
            "session_id": session_id,
            "events": row.get::<_, i64>(0),
            "gift_events": row.get::<_, i64>(1),
            "gift_count": row.get::<_, String>(2),
            "gift_value_minor": row.get::<_, String>(3),
            "comments": row.get::<_, i64>(4),
            "follows": row.get::<_, i64>(5),
            "shares": row.get::<_, i64>(6),
            "likes": row.get::<_, i64>(7),
            "attention": {
                "responded": attention.get::<_, i64>(0),
                "deferred": attention.get::<_, i64>(1),
                "ignored": attention.get::<_, i64>(2),
                "escalated": attention.get::<_, i64>(3),
                "last_decided_at_epoch": attention.get::<_, i64>(4),
            }
        }))
    }

    pub async fn reconcile_tiktok_live_gifts(
        &self,
        company_id: &str,
        session_id: &str,
        statement: &tiktok_live_engine::ProviderGiftStatement,
    ) -> Result<tiktok_live_engine::GiftReconciliation, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let session_uuid = Uuid::parse_str(session_id)?;
        let client = self.client.lock().await;
        let session_exists = client
            .query_opt(
                "SELECT 1 FROM tiktok_live_sessions WHERE id=$1 AND company_id=$2",
                &[&session_uuid, &company_uuid],
            )
            .await?
            .is_some();
        if !session_exists {
            return Err("LIVE session is not owned by company".into());
        }
        let row = client
            .query_one(
                "SELECT
                    COALESCE(SUM(gift_quantity) FILTER (WHERE kind='GIFT'),0)::text,
                    COALESCE(SUM(gift_value_minor) FILTER (WHERE kind='GIFT'),0)::text
                 FROM tiktok_live_events
                 WHERE company_id=$1 AND session_id=$2",
                &[&company_uuid, &session_uuid],
            )
            .await?;
        let gift_count_value = parse_i128_numeric(&row.get::<_, String>(0))?;
        if gift_count_value < 0 {
            return Err("stored LIVE gift count cannot be negative".into());
        }
        let gift_count = gift_count_value as u64;
        let gift_value_minor = parse_i128_numeric(&row.get::<_, String>(1))?;
        if gift_value_minor < 0 {
            return Err("stored LIVE gift value cannot be negative".into());
        }
        let ledger = tiktok_live_engine::LiveLedger {
            processed_event_ids: std::collections::HashSet::new(),
            gift_count,
            gift_value_minor: gift_value_minor as u128,
            comments: 0,
            follows: 0,
            shares: 0,
            likes: 0,
        };
        let reconciliation = tiktok_live_engine::reconcile_gifts(&ledger, statement)?;
        tx_store_live_gift_statement(
            &client,
            company_uuid,
            session_uuid,
            &reconciliation,
            &statement.currency,
        )
        .await?;
        Ok(reconciliation)
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

async fn tx_store_live_gift_statement(
    client: &Client,
    company_id: Uuid,
    session_id: Uuid,
    reconciliation: &tiktok_live_engine::GiftReconciliation,
    currency: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client
        .execute(
            "INSERT INTO tiktok_live_gift_statements
             (id, company_id, session_id, statement_id, gift_count,
              gross_value_minor, currency, matched, count_delta, value_delta_minor)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7,$8,$9,$10::numeric)
             ON CONFLICT (company_id, statement_id) DO UPDATE
             SET gift_count=EXCLUDED.gift_count,
                 gross_value_minor=EXCLUDED.gross_value_minor,
                 currency=EXCLUDED.currency,
                 matched=EXCLUDED.matched,
                 count_delta=EXCLUDED.count_delta,
                 value_delta_minor=EXCLUDED.value_delta_minor",
            &[
                &Uuid::new_v4(),
                &company_id,
                &session_id,
                &reconciliation.statement_id,
                &reconciliation.provider_gift_count.to_string(),
                &reconciliation.provider_value_minor.to_string(),
                &currency,
                &reconciliation.matched,
                &reconciliation.count_delta.to_string(),
                &reconciliation.value_delta_minor.to_string(),
            ],
        )
        .await?;
    Ok(())
}

fn attention_action_name(
    value: company_live_attention::AttentionAction,
) -> &'static str {
    match value {
        company_live_attention::AttentionAction::Respond => "RESPOND",
        company_live_attention::AttentionAction::Defer => "DEFER",
        company_live_attention::AttentionAction::Ignore => "IGNORE",
        company_live_attention::AttentionAction::Escalate => "ESCALATE",
    }
}

fn attention_reason_name(
    value: company_live_attention::AttentionReason,
) -> &'static str {
    match value {
        company_live_attention::AttentionReason::PurchaseIntent => "PURCHASE_INTENT",
        company_live_attention::AttentionReason::Objection => "OBJECTION",
        company_live_attention::AttentionReason::Gift => "GIFT",
        company_live_attention::AttentionReason::PkMoment => "PK_MOMENT",
        company_live_attention::AttentionReason::HighEngagement => "HIGH_ENGAGEMENT",
        company_live_attention::AttentionReason::HighValueViewer => "HIGH_VALUE_VIEWER",
        company_live_attention::AttentionReason::SafetyEscalation => "SAFETY_ESCALATION",
        company_live_attention::AttentionReason::Cooldown => "COOLDOWN",
        company_live_attention::AttentionReason::RateLimited => "RATE_LIMITED",
        company_live_attention::AttentionReason::LowSignal => "LOW_SIGNAL",
    }
}

fn attention_decision_from_row(
    row: tokio_postgres::Row,
) -> Result<company_live_attention::AttentionDecision, Box<dyn std::error::Error + Send + Sync>> {
    let action = match row.get::<_, String>(4).as_str() {
        "RESPOND" => company_live_attention::AttentionAction::Respond,
        "DEFER" => company_live_attention::AttentionAction::Defer,
        "IGNORE" => company_live_attention::AttentionAction::Ignore,
        "ESCALATE" => company_live_attention::AttentionAction::Escalate,
        other => return Err(format!("invalid stored attention action: {other}").into()),
    };
    let reason = match row.get::<_, String>(5).as_str() {
        "PURCHASE_INTENT" => company_live_attention::AttentionReason::PurchaseIntent,
        "OBJECTION" => company_live_attention::AttentionReason::Objection,
        "GIFT" => company_live_attention::AttentionReason::Gift,
        "PK_MOMENT" => company_live_attention::AttentionReason::PkMoment,
        "HIGH_ENGAGEMENT" => company_live_attention::AttentionReason::HighEngagement,
        "HIGH_VALUE_VIEWER" => company_live_attention::AttentionReason::HighValueViewer,
        "SAFETY_ESCALATION" => company_live_attention::AttentionReason::SafetyEscalation,
        "COOLDOWN" => company_live_attention::AttentionReason::Cooldown,
        "RATE_LIMITED" => company_live_attention::AttentionReason::RateLimited,
        "LOW_SIGNAL" => company_live_attention::AttentionReason::LowSignal,
        other => return Err(format!("invalid stored attention reason: {other}").into()),
    };
    Ok(company_live_attention::AttentionDecision {
        decision_id: row.get(0),
        company_id: row.get(1),
        session_id: row.get(2),
        event_id: row.get(3),
        action,
        reason,
        priority: row.get::<_, i16>(6).clamp(0, 100) as u8,
        decided_at_epoch: row.get(7),
        requires_human: row.get(8),
    })
}

fn policy_snapshot_from_row(
    row: tokio_postgres::Row,
) -> Result<company_compliance::PolicySnapshot, Box<dyn std::error::Error + Send + Sync>> {
    Ok(company_compliance::PolicySnapshot {
        id: row.get(0),
        company_id: row.get(1),
        policy_key: row.get(2),
        platform: row.get(3),
        jurisdiction: row.get(4),
        version: row.get(5),
        source_reference: row.get(6),
        evidence_hash: row.get(7),
        observed_at_epoch: row.get(8),
        effective_at_epoch: row.get(9),
        active: row.get(10),
        rules: serde_json::from_value(row.get(11))?,
    })
}

fn compliance_surface_name(value: company_compliance::ComplianceSurface) -> &'static str {
    match value {
        company_compliance::ComplianceSurface::Content => "CONTENT",
        company_compliance::ComplianceSurface::Affiliate => "AFFILIATE",
        company_compliance::ComplianceSurface::Live => "LIVE",
        company_compliance::ComplianceSurface::Advertising => "ADVERTISING",
        company_compliance::ComplianceSurface::Copyright => "COPYRIGHT",
        company_compliance::ComplianceSurface::ProductEligibility => "PRODUCT_ELIGIBILITY",
        company_compliance::ComplianceSurface::Claims => "CLAIMS",
    }
}

fn compliance_decision_name(value: company_compliance::ComplianceDecision) -> &'static str {
    match value {
        company_compliance::ComplianceDecision::Allowed => "ALLOWED",
        company_compliance::ComplianceDecision::Review => "REVIEW",
        company_compliance::ComplianceDecision::Blocked => "BLOCKED",
        company_compliance::ComplianceDecision::Unknown => "UNKNOWN",
    }
}

fn compliance_reason_name(value: company_compliance::ComplianceReason) -> &'static str {
    match value {
        company_compliance::ComplianceReason::PolicyUnavailable => "POLICY_UNAVAILABLE",
        company_compliance::ComplianceReason::MissingPolicyEvidence => "MISSING_POLICY_EVIDENCE",
        company_compliance::ComplianceReason::MissingDisclosure => "MISSING_DISCLOSURE",
        company_compliance::ComplianceReason::ProhibitedProduct => "PROHIBITED_PRODUCT",
        company_compliance::ComplianceReason::UnsupportedProduct => "UNSUPPORTED_PRODUCT",
        company_compliance::ComplianceReason::UnverifiedClaim => "UNVERIFIED_CLAIM",
        company_compliance::ComplianceReason::FakeEngagement => "FAKE_ENGAGEMENT",
        company_compliance::ComplianceReason::Simulcast => "SIMULCAST",
        company_compliance::ComplianceReason::MissingRightsEvidence => "MISSING_RIGHTS_EVIDENCE",
        company_compliance::ComplianceReason::HumanReviewRequired => "HUMAN_REVIEW_REQUIRED",
        company_compliance::ComplianceReason::AllowedByPolicy => "ALLOWED_BY_POLICY",
    }
}

fn compliance_check_from_row(
    row: tokio_postgres::Row,
    input: &company_compliance::ComplianceInput,
) -> Result<company_compliance::ComplianceCheck, Box<dyn std::error::Error + Send + Sync>> {
    let decision = match row.get::<_, String>(3).as_str() {
        "ALLOWED" => company_compliance::ComplianceDecision::Allowed,
        "REVIEW" => company_compliance::ComplianceDecision::Review,
        "BLOCKED" => company_compliance::ComplianceDecision::Blocked,
        "UNKNOWN" => company_compliance::ComplianceDecision::Unknown,
        other => return Err(format!("invalid stored compliance decision: {other}").into()),
    };
    let reason = match row.get::<_, String>(4).as_str() {
        "POLICY_UNAVAILABLE" => company_compliance::ComplianceReason::PolicyUnavailable,
        "MISSING_POLICY_EVIDENCE" => company_compliance::ComplianceReason::MissingPolicyEvidence,
        "MISSING_DISCLOSURE" => company_compliance::ComplianceReason::MissingDisclosure,
        "PROHIBITED_PRODUCT" => company_compliance::ComplianceReason::ProhibitedProduct,
        "UNSUPPORTED_PRODUCT" => company_compliance::ComplianceReason::UnsupportedProduct,
        "UNVERIFIED_CLAIM" => company_compliance::ComplianceReason::UnverifiedClaim,
        "FAKE_ENGAGEMENT" => company_compliance::ComplianceReason::FakeEngagement,
        "SIMULCAST" => company_compliance::ComplianceReason::Simulcast,
        "MISSING_RIGHTS_EVIDENCE" => company_compliance::ComplianceReason::MissingRightsEvidence,
        "HUMAN_REVIEW_REQUIRED" => company_compliance::ComplianceReason::HumanReviewRequired,
        "ALLOWED_BY_POLICY" => company_compliance::ComplianceReason::AllowedByPolicy,
        other => return Err(format!("invalid stored compliance reason: {other}").into()),
    };
    Ok(company_compliance::ComplianceCheck {
        id: row.get(0),
        company_id: row.get(1),
        policy_snapshot_id: row.get(2),
        input: input.clone(),
        decision,
        reason,
        requires_human: row.get(5),
        checked_at_epoch: row.get(6),
    })
}

async fn existing_compliance_check(
    client: &tokio_postgres::Client,
    input: &company_compliance::ComplianceInput,
    input_hash: &str,
) -> Result<company_compliance::ComplianceCheck, Box<dyn std::error::Error + Send + Sync>> {
    let row = client.query_one(
        "SELECT id,company_id,policy_snapshot_id,decision,reason,requires_human,checked_at_epoch
           FROM compliance_checks
          WHERE company_id=$1 AND policy_key=$2 AND policy_snapshot_key=$3 AND input_hash=$4",
        &[&input.company_id, &input.policy_key, &input.policy_snapshot_key, &input_hash],
    ).await?;
    compliance_check_from_row(row, input)
}

fn autonomy_simulation_from_row(
    row: tokio_postgres::Row,
    company_id: Uuid,
    idempotency_key: String,
) -> Result<AutonomySimulationRecord, Box<dyn std::error::Error + Send + Sync>> {
    let proposal = serde_json::from_value(row.get(1))?;
    let assessment: company_autonomy::AutonomyAssessment =
        serde_json::from_value(row.get::<_, serde_json::Value>(6))?;
    if row.get::<_, String>(2) != assessment.decision.as_str()
        || row.get::<_, String>(3) != assessment.ceiling.as_str()
        || row.get::<_, String>(4) != assessment.required_level.as_str()
        || row.get::<_, String>(5) != assessment.reason
    {
        return Err("persisted autonomy assessment summary does not match its JSON payload".into());
    }
    Ok(AutonomySimulationRecord {
        id: row.get(0),
        company_id,
        idempotency_key,
        proposal,
        assessment,
        created_at: row.get(7),
    })
}

fn agent_outcome_evidence_from_row(
    row: tokio_postgres::Row,
    company_id: Uuid,
    decision_journal_id: i64,
) -> Result<AgentOutcomeEvidenceRecord, Box<dyn std::error::Error + Send + Sync>> {
    Ok(AgentOutcomeEvidenceRecord {
        id: row.get(0),
        company_id,
        decision_journal_id,
        agent_name: row.get(1),
        action: row.get(2),
        evidence_ref: row.get(3),
        observed_revenue_delta_minor: parse_i128_numeric(&row.get::<_, String>(4))?,
        observed_contribution_margin_delta_minor: parse_i128_numeric(
            &row.get::<_, String>(5),
        )?,
        observed_at_epoch: row.get(6),
        created_at: row.get(7),
    })
}

fn agent_evaluation_from_row(
    row: tokio_postgres::Row,
) -> Result<company_agent_evaluation::AgentEvaluation, Box<dyn std::error::Error + Send + Sync>> {
    let input = company_agent_evaluation::AgentEvaluationInput {
        agent_name: row.get(0),
        proposal_count: row.get(1),
        approved_count: row.get(2),
        rejected_count: row.get(3),
        revision_count: row.get(4),
        escalated_count: row.get(5),
        executed_count: row.get(6),
        deferred_count: row.get(7),
        observed_spend_minor: parse_i128_numeric(&row.get::<_, String>(8))?,
        projected_revenue_minor: parse_i128_numeric(&row.get::<_, String>(9))?,
        outcome_evidence_count: row.get(10),
        observed_revenue_delta_minor: parse_i128_numeric(&row.get::<_, String>(11))?,
        observed_contribution_margin_delta_minor: parse_i128_numeric(
            &row.get::<_, String>(12),
        )?,
    };
    company_agent_evaluation::evaluate(&input)
        .map_err(|error| error.into())
}

fn metric_bps(numerator: i64, denominator: i64) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    if numerator < 0 || denominator < 0 {
        return Err("command center rate inputs cannot be negative".into());
    }
    if denominator == 0 {
        return Ok(0);
    }
    let value = (i128::from(numerator))
        .checked_mul(10_000)
        .and_then(|value| value.checked_div(i128::from(denominator)))
        .ok_or("command center rate overflow")?;
    Ok(value.clamp(0, 10_000) as u32)
}

fn scaled_minor(
    numerator: i128,
    denominator: i64,
    scale: i128,
) -> Result<i128, Box<dyn std::error::Error + Send + Sync>> {
    if numerator < 0 || denominator < 0 || scale < 0 {
        return Err("command center scaled metric inputs cannot be negative".into());
    }
    if denominator == 0 {
        return Ok(0);
    }
    numerator
        .checked_mul(scale)
        .and_then(|value| value.checked_div(i128::from(denominator)))
        .ok_or_else(|| "command center scaled metric overflow".into())
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



#[async_trait::async_trait]
impl agent_runtime::agent::AgentStateProvider for CompanyStore {
    async fn load_memory(
        &self,
        company_id: &str,
        agent: agent_runtime::types::AgentRole,
    ) -> Result<serde_json::Value, String> {
        let limit = std::env::var("AGENT_MEMORY_LIMIT")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| (1..=100).contains(v))
            .unwrap_or(20);

        self.load_agent_memory(company_id, agent, limit)
            .await
            .map(|items| serde_json::json!({ "items": items }))
            .map_err(|error| error.to_string())
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
            .unwrap_or(300);
        let max_calls = std::env::var("AGENT_MAX_CALLS_PER_WINDOW")
            .or_else(|_| std::env::var("AGENT_MAX_MODEL_CALLS_PER_WINDOW"))
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|v| (1..=100).contains(v))
            .unwrap_or(1);

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
    pub async fn create_service_proposal(&self, p: &commercial_sales::ServiceProposal) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if p.title.trim().is_empty() || p.idempotency_key.trim().is_empty() || p.total_minor < 0 || p.currency.len() != 3 { return Err("invalid service proposal".into()); }
        let mut c = self.client.lock().await;
        c.execute("INSERT INTO service_proposals (id,company_id,customer_id,title,currency,total_minor,status,valid_until_epoch,idempotency_key) VALUES ($1,$2,$3,$4,$5,$6::numeric,$7,$8,$9) ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[&p.id,&p.company_id,&p.customer_id,&p.title,&p.currency,&p.total_minor.to_string(),&format!("{:?}",p.status).to_uppercase(),&p.valid_until_epoch,&p.idempotency_key]).await?; Ok(())
    }

    pub async fn create_sponsorship(&self, s: &commercial_sales::Sponsorship) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if s.title.trim().is_empty() || s.committed_minor < 0 || s.delivered_minor < 0 || s.delivered_minor > s.committed_minor || s.currency.len() != 3 { return Err("invalid sponsorship".into()); }
        let mut c = self.client.lock().await;
        c.execute("INSERT INTO sponsorships (id,company_id,customer_id,title,currency,committed_minor,delivered_minor,status) VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8)",
            &[&s.id,&s.company_id,&s.customer_id,&s.title,&s.currency,&s.committed_minor.to_string(),&s.delivered_minor.to_string(),&s.status]).await?; Ok(())
    }

    pub async fn create_invoice(&self, invoice: &commercial_sales::Invoice, lines: &[commercial_sales::InvoiceLine]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if invoice.idempotency_key.trim().is_empty() || invoice.currency.len() != 3 || lines.is_empty() { return Err("invalid invoice".into()); }
        let total = commercial_sales::invoice_total(lines)?;
        if total <= 0 || total != invoice.subtotal_minor || invoice.paid_minor != 0 { return Err("invoice total or initial payment is invalid".into()); }
        let mut c = self.client.lock().await; let tx = c.transaction().await?;
        if tx.query_opt("SELECT id FROM invoices WHERE company_id=$1 AND idempotency_key=$2",&[&invoice.company_id,&invoice.idempotency_key]).await?.is_some() { tx.rollback().await?; return Ok(()); }
        tx.execute("INSERT INTO invoices (id,company_id,customer_id,currency,subtotal_minor,paid_minor,status,due_epoch,idempotency_key) VALUES ($1,$2,$3,$4,$5::numeric,0,$6,$7,$8)",
            &[&invoice.id,&invoice.company_id,&invoice.customer_id,&invoice.currency,&total.to_string(),&format!("{:?}",invoice.status).to_uppercase(),&invoice.due_epoch,&invoice.idempotency_key]).await?;
        for line in lines {
            if line.description.trim().is_empty() || line.quantity == 0 || line.unit_price_minor < 0 { return Err("invalid invoice line".into()); }
            tx.execute("INSERT INTO invoice_lines (invoice_id,description,quantity,unit_price_minor) VALUES ($1,$2,$3,$4::numeric)",
                &[&invoice.id,&line.description,&(line.quantity as i32),&line.unit_price_minor.to_string()]).await?;
        }
        tx.commit().await?; Ok(())
    }

    pub async fn record_payment_reconciliation_evidence(
        &self,
        company_id: &str,
        invoice_id: &str,
        provider: &str,
        provider_event_id: &str,
        external_ref: Option<&str>,
        amount_minor: i128,
        currency: &str,
        observed_at_epoch: i64,
        evidence_hash: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if amount_minor <= 0 || currency.len() != 3 || provider.trim().is_empty() || provider_event_id.trim().is_empty() {
            return Err("invalid payment reconciliation evidence".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let invoice = Uuid::parse_str(invoice_id)?;
        let mut c = self.client.lock().await;
        let tx = c.transaction().await?;
        if let Some(row) = tx.query_opt(
            "SELECT status FROM payment_reconciliation_evidence WHERE company_id=$1 AND provider=$2 AND provider_event_id=$3",
            &[&company,&provider,&provider_event_id]
        ).await? {
            let status: String = row.get(0);
            tx.rollback().await?;
            return Ok(status);
        }
        let inv = tx.query_opt(
            "SELECT currency, subtotal_minor::text, paid_minor::text FROM invoices WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company,&invoice]
        ).await?.ok_or("invoice not found")?;
        let invoice_currency: String = inv.get(0);
        if invoice_currency != currency { return Err("payment currency does not match invoice".into()); }
        let total = parse_i128_numeric(&inv.get::<_,String>(1))?;
        let paid = parse_i128_numeric(&inv.get::<_,String>(2))?;
        let remaining = total.checked_sub(paid).ok_or("invoice remaining overflow")?;
        let status = if amount_minor <= remaining { "OBSERVED" } else { "REJECTED" };
        let reason = if status == "REJECTED" { Some("observed payment exceeds invoice remaining balance") } else { None };
        tx.execute(
            "INSERT INTO payment_reconciliation_evidence
             (id,company_id,invoice_id,provider,provider_event_id,external_ref,observed_amount_minor,currency,observed_at_epoch,evidence_hash,status,reason)
             VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8,$9,$10,$11,$12)",
            &[&Uuid::new_v4(),&company,&invoice,&provider,&provider_event_id,&external_ref,&amount_minor.to_string(),&currency,&observed_at_epoch,&evidence_hash,&status,&reason]
        ).await?;
        if status == "OBSERVED" {
            tx.execute(
                "INSERT INTO outbox_events
                 (company_id,event_type,aggregate_id,idempotency_key,payload)
                 VALUES ($1,'PAYMENT_RECONCILIATION_OBSERVED',$2,$3,$4)
                 ON CONFLICT(company_id,idempotency_key) DO NOTHING",
                &[&company,&invoice,&format!("payment-reconcile:{provider}:{provider_event_id}"),
                  &serde_json::json!({"invoice_id":invoice_id,"provider":provider,"provider_event_id":provider_event_id,"amount_minor":amount_minor,"currency":currency})]
            ).await?;
        }
        tx.commit().await?;
        Ok(status.to_owned())
    }

    pub async fn record_invoice_payment(&self, company_id: &str, invoice_id: &str, payment_id: &str, amount_minor: i128, occurred_at_epoch: i64, external_ref: Option<&str>) -> Result<commercial_sales::InvoiceStatus, Box<dyn std::error::Error + Send + Sync>> {
        if amount_minor <= 0 { return Err("invoice payment must be positive".into()); }
        let company = Uuid::parse_str(company_id)?;
        let invoice = Uuid::parse_str(invoice_id)?;
        let payment = Uuid::parse_str(payment_id)?;
        let mut c = self.client.lock().await;
        let tx = c.transaction().await?;
        let row = tx.query_one(
            "SELECT subtotal_minor::text, paid_minor::text, status, currency
               FROM invoices
              WHERE company_id=$1 AND id=$2
              FOR UPDATE",
            &[&company, &invoice],
        ).await?;
        let total = parse_i128_numeric(&row.get::<_,String>(0))?;
        let paid = parse_i128_numeric(&row.get::<_,String>(1))?;
        let status: String = row.get(2);
        let currency: String = row.get(3);
        if status == "VOID" || status == "DRAFT" { return Err("invoice is not payable".into()); }
        let remaining = total.checked_sub(paid).ok_or("invoice remaining overflow")?;
        if amount_minor > remaining { return Err("invoice payment exceeds remaining balance".into()); }

        if tx.execute(
            "INSERT INTO invoice_payments
                (id,invoice_id,amount_minor,external_ref,occurred_at_epoch)
             VALUES ($1,$2,$3::numeric,$4,$5)
             ON CONFLICT (id) DO NOTHING",
            &[&payment,&invoice,&amount_minor.to_string(),&external_ref,&occurred_at_epoch],
        ).await? == 0 {
            let current = match status.as_str() {
                "PAID" => commercial_sales::InvoiceStatus::Paid,
                "PARTIALLY_PAID" => commercial_sales::InvoiceStatus::PartiallyPaid,
                "ISSUED" => commercial_sales::InvoiceStatus::Issued,
                _ => commercial_sales::InvoiceStatus::Draft,
            };
            tx.rollback().await?;
            return Ok(current);
        }

        let next = paid.checked_add(amount_minor).ok_or("invoice paid overflow")?;
        let next_status = commercial_sales::transition_invoice(commercial_sales::InvoiceStatus::Issued,next,total)?;

        let cash = ensure_ledger_account(&tx, company, "CASH", "Cash", "ASSET", &currency).await?;
        let receivable = ensure_ledger_account(&tx, company, "ACCOUNTS_RECEIVABLE", "Accounts Receivable", "ASSET", &currency).await?;
        let transaction_id = Uuid::new_v4();
        let idempotency_key = format!("invoice-payment:{payment}");
        tx.execute(
            "INSERT INTO ledger_transactions (id,company_id,description,idempotency_key)
             VALUES ($1,$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[&transaction_id,&company,&format!("Invoice payment {invoice}"),&idempotency_key],
        ).await?;
        insert_ledger_entry(&tx, transaction_id, cash, &amount_minor.to_string(), "0", &currency).await?;
        insert_ledger_entry(&tx, transaction_id, receivable, "0", &amount_minor.to_string(), &currency).await?;

        tx.execute(
            "UPDATE invoices
                SET paid_minor=$3::numeric,status=$4,updated_at=now()
              WHERE company_id=$1 AND id=$2",
            &[&company,&invoice,&next.to_string(),&format!("{:?}",next_status).to_uppercase()],
        ).await?;
        tx.commit().await?;
        Ok(next_status)
    }

    pub async fn issue_invoice(&self, company_id: &str, invoice_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let invoice = Uuid::parse_str(invoice_id)?;
        let mut c = self.client.lock().await;
        let tx = c.transaction().await?;
        let row = tx.query_opt(
            "SELECT subtotal_minor::text, currency, status
               FROM invoices
              WHERE company_id=$1 AND id=$2
              FOR UPDATE",
            &[&company,&invoice],
        ).await?;
        let Some(row) = row else {
            return Err("invoice not found".into());
        };
        let total = parse_i128_numeric(&row.get::<_,String>(0))?;
        let currency: String = row.get(1);
        let status: String = row.get(2);
        if status == "ISSUED" {
            tx.rollback().await?;
            return Ok(());
        }
        if status != "DRAFT" {
            return Err("invoice cannot be issued from its current state".into());
        }
        if total <= 0 {
            return Err("invoice total must be positive before issue".into());
        }

        let receivable = ensure_ledger_account(&tx, company, "ACCOUNTS_RECEIVABLE", "Accounts Receivable", "ASSET", &currency).await?;
        let revenue = ensure_ledger_account(&tx, company, "INVOICE_REVENUE", "Invoice Revenue", "REVENUE", &currency).await?;
        let transaction_id = Uuid::new_v4();
        let idempotency_key = format!("invoice-issued:{invoice}");
        tx.execute(
            "INSERT INTO ledger_transactions (id,company_id,description,idempotency_key)
             VALUES ($1,$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[&transaction_id,&company,&format!("Invoice issued {invoice}"),&idempotency_key],
        ).await?;
        insert_ledger_entry(&tx, transaction_id, receivable, &total.to_string(), "0", &currency).await?;
        insert_ledger_entry(&tx, transaction_id, revenue, "0", &total.to_string(), &currency).await?;
        tx.execute(
            "UPDATE invoices SET status='ISSUED', updated_at=now()
              WHERE company_id=$1 AND id=$2",
            &[&company,&invoice],
        ).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn create_customer(
        &self,
        company_id: &str,
        customer_id: Uuid,
        name: &str,
        email: Option<&str>,
        external_ref: Option<&str>,
        status: &str,
        notes: Option<&str>,
        idempotency_key: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        if name.trim().is_empty() || name.len() > 200 || idempotency_key.trim().is_empty() || idempotency_key.len() > 256 {
            return Err("invalid customer identity".into());
        }
        if !matches!(status, "LEAD" | "ACTIVE" | "INACTIVE" | "CHURNED") {
            return Err("invalid customer status".into());
        }
        if let Some(value) = email {
            if value.len() > 320 || !value.contains('@') {
                return Err("invalid customer email".into());
            }
        }
        let mut client = self.client.lock().await;
        let row = client
            .query_opt(
                "INSERT INTO customers
                    (id, company_id, name, email, external_ref, status, notes, idempotency_key)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                 ON CONFLICT (company_id, idempotency_key) WHERE idempotency_key IS NOT NULL
                 DO NOTHING
                 RETURNING id, name, email, external_ref, status, notes, lifetime_revenue_minor::text, created_at, updated_at",
                &[&customer_id, &company, &name, &email, &external_ref, &status, &notes, &idempotency_key],
            )
            .await?;
        let row = match row {
            Some(row) => row,
            None => client
                .query_one(
                    "SELECT id, name, email, external_ref, status, notes,
                            lifetime_revenue_minor::text, created_at, updated_at
                       FROM customers
                      WHERE company_id=$1 AND idempotency_key=$2",
                    &[&company, &idempotency_key],
                )
                .await?,
        };
        Ok(serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "name": row.get::<_, String>(1),
            "email": row.get::<_, Option<String>>(2),
            "external_ref": row.get::<_, Option<String>>(3),
            "status": row.get::<_, String>(4),
            "notes": row.get::<_, Option<String>>(5),
            "lifetime_revenue_minor": row.get::<_, String>(6),
            "created_at": row.get::<_, time::OffsetDateTime>(7).to_string(),
            "updated_at": row.get::<_, time::OffsetDateTime>(8).to_string()
        }))
    }

    pub async fn list_customers(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) {
            return Err("customer limit must be between 1 and 200".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT id, name, email, external_ref, status, notes,
                    lifetime_revenue_minor::text, created_at, updated_at
               FROM customers
              WHERE company_id=$1
              ORDER BY created_at DESC, id DESC
              LIMIT $2",
            &[&company, &limit],
        ).await?;
        Ok(rows.into_iter().map(|row| serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "name": row.get::<_, String>(1),
            "email": row.get::<_, Option<String>>(2),
            "external_ref": row.get::<_, Option<String>>(3),
            "status": row.get::<_, String>(4),
            "notes": row.get::<_, Option<String>>(5),
            "lifetime_revenue_minor": row.get::<_, String>(6),
            "created_at": row.get::<_, time::OffsetDateTime>(7).to_string(),
            "updated_at": row.get::<_, time::OffsetDateTime>(8).to_string()
        })).collect())
    }

    pub async fn list_commercial_pipeline(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) {
            return Err("commercial pipeline limit must be between 1 and 200".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let proposals = client.query(
            "SELECT id, customer_id, title, currency, total_minor::text, status,
                    valid_until_epoch, created_at, updated_at
               FROM service_proposals
              WHERE company_id=$1
              ORDER BY created_at DESC, id DESC
              LIMIT $2",
            &[&company, &limit],
        ).await?;
        let sponsorships = client.query(
            "SELECT id, customer_id, title, currency, committed_minor::text,
                    delivered_minor::text, status, created_at, updated_at
               FROM sponsorships
              WHERE company_id=$1
              ORDER BY created_at DESC, id DESC
              LIMIT $2",
            &[&company, &limit],
        ).await?;
        let invoices = client.query(
            "SELECT id, customer_id, currency, subtotal_minor::text,
                    paid_minor::text, status, due_epoch, created_at, updated_at
               FROM invoices
              WHERE company_id=$1
              ORDER BY created_at DESC, id DESC
              LIMIT $2",
            &[&company, &limit],
        ).await?;

        Ok(serde_json::json!({
            "proposals": proposals.into_iter().map(|row| serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "customer_id": row.get::<_, Uuid>(1),
                "title": row.get::<_, String>(2),
                "currency": row.get::<_, String>(3),
                "total_minor": row.get::<_, String>(4),
                "status": row.get::<_, String>(5),
                "valid_until_epoch": row.get::<_, i64>(6),
                "created_at": row.get::<_, time::OffsetDateTime>(7).to_string(),
                "updated_at": row.get::<_, time::OffsetDateTime>(8).to_string()
            })).collect::<Vec<_>>(),
            "sponsorships": sponsorships.into_iter().map(|row| serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "customer_id": row.get::<_, Uuid>(1),
                "title": row.get::<_, String>(2),
                "currency": row.get::<_, String>(3),
                "committed_minor": row.get::<_, String>(4),
                "delivered_minor": row.get::<_, String>(5),
                "status": row.get::<_, String>(6),
                "created_at": row.get::<_, time::OffsetDateTime>(7).to_string(),
                "updated_at": row.get::<_, time::OffsetDateTime>(8).to_string()
            })).collect::<Vec<_>>(),
            "invoices": invoices.into_iter().map(|row| serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "customer_id": row.get::<_, Uuid>(1),
                "currency": row.get::<_, String>(2),
                "subtotal_minor": row.get::<_, String>(3),
                "paid_minor": row.get::<_, String>(4),
                "status": row.get::<_, String>(5),
                "due_epoch": row.get::<_, i64>(6),
                "created_at": row.get::<_, time::OffsetDateTime>(7).to_string(),
                "updated_at": row.get::<_, time::OffsetDateTime>(8).to_string()
            })).collect::<Vec<_>>()
        }))
    }

    pub async fn transition_service_proposal(
        &self,
        company_id: &str,
        proposal_id: &str,
        next: commercial_sales::ProposalStatus,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let proposal = Uuid::parse_str(proposal_id)?;
        let next_status = format!("{:?}", next).to_uppercase();
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx.query_opt(
            "SELECT status FROM service_proposals
              WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company, &proposal],
        ).await?.ok_or("service proposal not found")?;
        let current = match row.get::<_, String>(0).as_str() {
            "DRAFT" => commercial_sales::ProposalStatus::Draft,
            "SENT" => commercial_sales::ProposalStatus::Sent,
            "ACCEPTED" => commercial_sales::ProposalStatus::Accepted,
            "REJECTED" => commercial_sales::ProposalStatus::Rejected,
            "EXPIRED" => commercial_sales::ProposalStatus::Expired,
            _ => return Err("invalid stored proposal status".into()),
        };
        let resolved = commercial_sales::transition_proposal(current, next)?;
        let resolved_status = format!("{:?}", resolved).to_uppercase();
        tx.execute(
            "UPDATE service_proposals SET status=$3, updated_at=now()
              WHERE company_id=$1 AND id=$2",
            &[&company, &proposal, &resolved_status],
        ).await?;
        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'SERVICE_PROPOSAL_STATUS_CHANGED',$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &proposal,
                &format!("outbox:proposal-status:{}:{}", proposal, resolved_status),
                &serde_json::json!({
                    "proposal_id": proposal,
                    "from": format!("{:?}", current),
                    "to": format!("{:?}", resolved)
                })
            ],
        ).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn transition_sponsorship(
        &self,
        company_id: &str,
        sponsorship_id: &str,
        next: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let sponsorship = Uuid::parse_str(sponsorship_id)?;
        let next = next.trim().to_uppercase();
        if !matches!(next.as_str(), "PROSPECT" | "CONTRACTED" | "DELIVERING" | "COMPLETED" | "CANCELLED") {
            return Err("invalid sponsorship status".into());
        }
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx.query_opt(
            "SELECT status FROM sponsorships
              WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company, &sponsorship],
        ).await?.ok_or("sponsorship not found")?;
        let current: String = row.get(0);
        let resolved = commercial_sales::transition_sponsorship(&current, &next)?;
        tx.execute(
            "UPDATE sponsorships SET status=$3, updated_at=now()
              WHERE company_id=$1 AND id=$2",
            &[&company, &sponsorship, &resolved],
        ).await?;
        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'SPONSORSHIP_STATUS_CHANGED',$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &sponsorship,
                &format!("outbox:sponsorship-status:{}:{}", sponsorship, resolved),
                &serde_json::json!({"sponsorship_id": sponsorship, "from": current, "to": resolved})
            ],
        ).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn record_sponsorship_delivery(
        &self,
        company_id: &str,
        sponsorship_id: &str,
        delivered_minor: i128,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if delivered_minor <= 0 {
            return Err("sponsorship delivery must be positive".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let sponsorship = Uuid::parse_str(sponsorship_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx.query_opt(
            "SELECT committed_minor::text, delivered_minor::text, status
               FROM sponsorships
              WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company, &sponsorship],
        ).await?.ok_or("sponsorship not found")?;
        let committed = parse_i128_numeric(&row.get::<_, String>(0))?;
        let delivered = parse_i128_numeric(&row.get::<_, String>(1))?;
        let status: String = row.get(2);
        if !matches!(status.as_str(), "CONTRACTED" | "DELIVERING") {
            return Err("sponsorship is not in a deliverable state".into());
        }
        let next_delivered = delivered.checked_add(delivered_minor).ok_or("sponsorship delivery overflow")?;
        if next_delivered > committed {
            return Err("sponsorship delivery exceeds committed value".into());
        }
        let next_status = if next_delivered == committed { "COMPLETED" } else { "DELIVERING" };
        tx.execute(
            "UPDATE sponsorships SET delivered_minor=$3::numeric,status=$4,updated_at=now()
              WHERE company_id=$1 AND id=$2",
            &[&company, &sponsorship, &next_delivered.to_string(), &next_status],
        ).await?;
        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'SPONSORSHIP_DELIVERY_RECORDED',$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &sponsorship,
                &format!("outbox:sponsorship-delivery:{}:{}", sponsorship, next_delivered),
                &serde_json::json!({
                    "sponsorship_id": sponsorship,
                    "delivered_minor": delivered_minor,
                    "total_delivered_minor": next_delivered,
                    "status": next_status
                })
            ],
        ).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn record_growth_trend(
        &self,
        signal: &company_growth::TrendSignal,
    ) -> Result<(GrowthTrendRecord, Option<GrowthOpportunityRecord>), Box<dyn std::error::Error + Send + Sync>> {
        company_growth::validate_trend(signal).map_err(|error| error.to_string())?;
        let evaluation = company_growth::evaluate_trend(signal).map_err(|error| error.to_string())?;
        let company = signal.company_id;
        let trend_id = Uuid::new_v4();
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let inserted_trend = tx.query_opt(
            "INSERT INTO growth_trends
             (id,company_id,trend_key,topic,source,evidence_ref,observed_at_epoch,
              velocity_bps,audience_fit_bps,product_fit_bps,contentability_bps,competition_bps,
              confidence_bps,product_ref,offer_ref,content_format,max_budget_minor,max_loss_minor,
              max_duration_seconds,success_metric,success_threshold_bps,policy_evidence_ref,score_bps,decision)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24)
             ON CONFLICT(company_id,trend_key) DO NOTHING
             RETURNING id",
            &[
                &trend_id, &company, &signal.trend_key, &signal.topic, &signal.source,
                &signal.evidence_ref, &signal.observed_at_epoch, &(signal.velocity_bps as i32),
                &(signal.audience_fit_bps as i32), &(signal.product_fit_bps as i32),
                &(signal.contentability_bps as i32), &(signal.competition_bps as i32),
                &(signal.confidence_bps as i32), &signal.product_ref, &signal.offer_ref,
                &content_format_name(signal.content_format), &signal.max_budget_minor.to_string(),
                &signal.max_loss_minor.to_string(), &(signal.max_duration_seconds as i64),
                &success_metric_name(signal.success_metric), &(signal.success_threshold_bps as i32),
                &signal.policy_evidence_ref, &(evaluation.score_bps as i32),
                &growth_trend_decision_name(evaluation.decision),
            ],
        ).await?;

        let trend = load_growth_trend(&tx, &company, &signal.trend_key)
            .await?
            .ok_or("persisted growth trend not found")?;

        if inserted_trend.is_some() {
            tx.execute(
                "INSERT INTO outbox_events
                 (company_id,event_type,aggregate_id,idempotency_key,payload)
                 VALUES ($1,'TREND_DETECTED',$2,$3,$4)
                 ON CONFLICT(company_id,idempotency_key) DO NOTHING",
                &[
                    &company,
                    &trend.id,
                    &format!("outbox:growth-trend:{}", trend.id),
                    &serde_json::json!({
                        "trend_id": trend.id,
                        "trend_key": trend.signal.trend_key,
                        "score_bps": trend.score_bps,
                        "decision": growth_trend_decision_name(trend.decision)
                    }),
                ],
            ).await?;
        }

        let opportunity = if trend.decision == company_growth::TrendDecision::Pursue {
            if let Some(existing) = load_growth_opportunity_by_trend(&tx, &company, trend.id).await? {
                Some(existing)
            } else {
                let opportunity = company_growth::opportunity_from_trend(trend.id, &trend.signal)?
                    .ok_or("pursue trend must create an opportunity")?;
                let plan_json = serde_json::to_value(&opportunity.plan)?;
                let inserted_opportunity = tx.query_opt(
                    "INSERT INTO growth_opportunities
                     (id,company_id,trend_id,opportunity_key,title,score_bps,confidence_bps,
                      policy_evidence_ref,plan_json,status)
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'READY')
                     ON CONFLICT(company_id,opportunity_key) DO NOTHING
                     RETURNING id",
                    &[
                        &opportunity.id, &company, &trend.id, &opportunity.opportunity_key,
                        &opportunity.title, &(opportunity.score_bps as i32), &(opportunity.confidence_bps as i32),
                        &opportunity.policy_evidence_ref, &plan_json,
                    ],
                ).await?;
                if inserted_opportunity.is_some() {
                    tx.execute(
                        "INSERT INTO outbox_events
                         (company_id,event_type,aggregate_id,idempotency_key,payload)
                         VALUES ($1,'OPPORTUNITY_CREATED',$2,$3,$4)
                         ON CONFLICT(company_id,idempotency_key) DO NOTHING",
                        &[
                            &company,
                            &opportunity.id,
                            &format!("outbox:growth-opportunity:{}", opportunity.id),
                            &serde_json::json!({
                                "opportunity_id": opportunity.id,
                                "trend_id": trend.id,
                                "score_bps": opportunity.score_bps,
                                "confidence_bps": opportunity.confidence_bps
                            }),
                        ],
                    ).await?;
                }
                load_growth_opportunity_by_trend(&tx, &company, trend.id)
                    .await?
                    .ok_or("growth opportunity persistence failed")?
            }
        } else {
            None
        };

        tx.commit().await?;
        Ok((trend, opportunity))
    }

    pub async fn list_growth_trends(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<GrowthTrendRecord>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) {
            return Err("growth trend limit must be between 1 and 200".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT id,company_id,trend_key,topic,source,evidence_ref,observed_at_epoch,
                    velocity_bps,audience_fit_bps,product_fit_bps,contentability_bps,competition_bps,
                    confidence_bps,product_ref,offer_ref,content_format,max_budget_minor,max_loss_minor,
                    max_duration_seconds,success_metric,success_threshold_bps,policy_evidence_ref,
                    score_bps,decision,created_at::text
               FROM growth_trends
              WHERE company_id=$1
              ORDER BY observed_at_epoch DESC,created_at DESC
              LIMIT $2",
            &[&company, &limit],
        ).await?;
        rows.into_iter().map(growth_trend_from_row).collect()
    }

    pub async fn list_growth_opportunities(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<GrowthOpportunityRecord>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) {
            return Err("growth opportunity limit must be between 1 and 200".into());
        }
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT o.id,o.company_id,o.trend_id,o.opportunity_key,o.title,o.score_bps,o.confidence_bps,
                    o.policy_evidence_ref,o.plan_json,o.status,o.content_item_id,o.content_created_at_epoch,
                    CASE WHEN o.content_created_at_epoch IS NULL THEN NULL
                         WHEN o.content_created_at_epoch >= t.observed_at_epoch
                           THEN o.content_created_at_epoch - t.observed_at_epoch
                         ELSE NULL END AS ttfc_seconds,
                    o.created_at::text
               FROM growth_opportunities o
               JOIN growth_trends t ON t.id=o.trend_id AND t.company_id=o.company_id
              WHERE o.company_id=$1
              ORDER BY o.score_bps DESC,o.created_at DESC
              LIMIT $2",
            &[&company, &limit],
        ).await?;
        rows.into_iter().map(growth_opportunity_from_row).collect()
    }

    pub async fn create_content_from_growth_opportunity(
        &self,
        company_id: &str,
        opportunity_id: Uuid,
    ) -> Result<ContentRecord, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx.query_one(
            "SELECT o.id,o.company_id,o.trend_id,o.opportunity_key,o.title,o.score_bps,o.confidence_bps,
                    o.policy_evidence_ref,o.plan_json,o.status,o.content_item_id,o.content_created_at_epoch,
                    CASE WHEN o.content_created_at_epoch IS NULL THEN NULL
                         WHEN o.content_created_at_epoch >= t.observed_at_epoch
                           THEN o.content_created_at_epoch - t.observed_at_epoch
                         ELSE NULL END AS ttfc_seconds,
                    o.created_at::text
               FROM growth_opportunities o
              JOIN growth_trends t ON t.id=o.trend_id AND t.company_id=o.company_id
              WHERE o.company_id=$1 AND o.id=$2
              FOR UPDATE",
            &[&company, &opportunity_id],
        ).await?;

        if let Some(content_id) = row.get::<_, Option<Uuid>>(10) {
            let content_row = tx.query_one(
                "SELECT id,company_id,hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
                        expected_cost_minor::text,max_loss_minor::text,max_duration_seconds,success_metric,
                        success_threshold_bps,variant_key,hook,first_frame,emotion,pacing,scene_count,
                        text_density,voice_speed,product_placement,cta,comment_trigger,music_style,
                        visual_style,status,decision,created_at::text
                   FROM content_items
                  WHERE company_id=$1 AND id=$2",
                &[&company, &content_id],
            ).await?;
            tx.commit().await?;
            return content_record_from_row(content_row);
        }

        let opportunity = growth_opportunity_from_row(row)?.opportunity;
        let item = company_content::ContentItem {
            id: Uuid::new_v4(),
            company_id: company,
            brief: opportunity.plan.brief,
            variant: opportunity.plan.variant,
            status: company_content::ContentStatus::Draft,
            decision: None,
        };
        company_content::validate_item(&item).map_err(|error| error.to_string())?;
        let brief = &item.brief;
        let variant = &item.variant;
        tx.execute(
            "INSERT INTO content_items
             (id,company_id,hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
              expected_cost_minor,max_loss_minor,max_duration_seconds,success_metric,success_threshold_bps,
              variant_key,hook,first_frame,emotion,pacing,scene_count,text_density,voice_speed,
              product_placement,cta,comment_trigger,music_style,visual_style,status,decision)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28)",
            &[
                &item.id, &company, &brief.hypothesis, &brief.audience,
                &content_format_name(brief.format), &brief.product_ref, &brief.offer_ref,
                &brief.disclosure_required, &brief.expected_cost_minor.to_string(),
                &brief.max_loss_minor.to_string(), &(brief.max_duration_seconds as i64),
                &success_metric_name(brief.success_metric), &(brief.success_threshold_bps as i32),
                &variant.variant_key, &variant.hook, &variant.first_frame, &variant.emotion,
                &variant.pacing, &(variant.scene_count as i32), &variant.text_density,
                &variant.voice_speed, &variant.product_placement, &variant.cta,
                &variant.comment_trigger, &variant.music_style, &variant.visual_style,
                &"DRAFT", &Option::<String>::None,
            ],
        ).await?;
        tx.execute(
            "UPDATE growth_opportunities
                SET status='CONTENT_CREATED', content_item_id=$3, content_created_at_epoch=EXTRACT(EPOCH FROM now())::bigint
              WHERE company_id=$1 AND id=$2",
            &[&company, &opportunity_id, &item.id],
        ).await?;
        let growth_observed_at_epoch = time::OffsetDateTime::now_utc().unix_timestamp();
        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company,
                company_revenue_graph::RevenueNodeType::Trend,
                &format!("growth-trend:{}", opportunity.trend_id),
                "GENERATES_CONTENT",
                company_revenue_graph::RevenueNodeType::Content,
                &item.id.to_string(),
                None,
                None,
                opportunity.confidence_bps,
                &format!("growth-opportunity:{}", opportunity.id),
                "growth-loop",
                growth_observed_at_epoch,
            ),
        )
        .await?;

        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company,
                company_revenue_graph::RevenueNodeType::Content,
                &item.id.to_string(),
                "USES_HOOK",
                company_revenue_graph::RevenueNodeType::Hook,
                &hashed_graph_ref("hook", &item.variant.hook),
                None,
                None,
                10_000,
                &format!("content:{}", item.id),
                "content-factory",
                growth_observed_at_epoch,
            ),
        )
        .await?;

        record_revenue_graph_edge_tx(
            &tx,
            &new_graph_edge(
                company,
                company_revenue_graph::RevenueNodeType::Content,
                &item.id.to_string(),
                "TARGETS_AUDIENCE",
                company_revenue_graph::RevenueNodeType::Audience,
                &hashed_graph_ref("audience", &item.brief.audience),
                None,
                None,
                10_000,
                &format!("content:{}", item.id),
                "content-factory",
                growth_observed_at_epoch,
            ),
        )
        .await?;

        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'CONTENT_CREATED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[
                &company,
                &item.id,
                &format!("outbox:growth-content:{}", item.id),
                &serde_json::json!({
                    "content_id": item.id,
                    "opportunity_id": opportunity_id,
                    "trend_id": opportunity.trend_id
                }),
            ],
        ).await?;
        let content_row = tx.query_one(
            "SELECT id,company_id,hypothesis,audience,format,product_ref,offer_ref,disclosure_required,
                    expected_cost_minor::text,max_loss_minor::text,max_duration_seconds,success_metric,
                    success_threshold_bps,variant_key,hook,first_frame,emotion,pacing,scene_count,
                    text_density,voice_speed,product_placement,cta,comment_trigger,music_style,
                    visual_style,status,decision,created_at::text
               FROM content_items
              WHERE company_id=$1 AND id=$2",
            &[&company, &item.id],
        ).await?;
        tx.commit().await?;
        content_record_from_row(content_row)
    }

    pub async fn create_customer_success_task(
        &self, company_id:&str, task_id:Uuid, customer_id:&str, task_type:&str,
        due_at_epoch:i64, owner:Option<&str>, notes:Option<&str>, idempotency_key:&str
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let company=Uuid::parse_str(company_id)?; let customer=Uuid::parse_str(customer_id)?;
        if !matches!(task_type,"ONBOARDING"|"HEALTH_REVIEW"|"RENEWAL"|"EXPANSION"|"RISK_REVIEW")
            || idempotency_key.trim().is_empty() { return Err("invalid customer success task".into()); }
        let client=self.client.lock().await;
        let row=client.query_opt(
            "INSERT INTO customer_success_tasks
             (id,company_id,customer_id,task_type,due_at_epoch,owner,status,notes,idempotency_key)
             VALUES ($1,$2,$3,$4,$5,$6,'OPEN',$7,$8)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING
             RETURNING id,status,due_at_epoch",
            &[&task_id,&company,&customer,&task_type,&due_at_epoch,&owner,&notes,&idempotency_key]
        ).await?;
        let row=match row { Some(r)=>r, None=>client.query_one(
            "SELECT id,status,due_at_epoch FROM customer_success_tasks WHERE company_id=$1 AND idempotency_key=$2",
            &[&company,&idempotency_key]).await? };
        Ok(serde_json::json!({"id":row.get::<_,Uuid>(0),"status":row.get::<_,String>(1),"due_at_epoch":row.get::<_,i64>(2)}))
    }

    pub async fn list_due_customer_success_tasks(
        &self, company_id:&str, now_epoch:i64, limit:i64
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let company=Uuid::parse_str(company_id)?; let client=self.client.lock().await;
        let rows=client.query(
            "SELECT id,customer_id,task_type,due_at_epoch,owner,status,notes,outcome
             FROM customer_success_tasks
             WHERE company_id=$1 AND status IN ('OPEN','IN_PROGRESS') AND due_at_epoch <= $2
             ORDER BY due_at_epoch ASC LIMIT $3",
            &[&company,&now_epoch,&limit.max(1).min(500)]
        ).await?;
        Ok(rows.into_iter().map(|r| serde_json::json!({
            "id":r.get::<_,Uuid>(0),"customer_id":r.get::<_,Uuid>(1),"task_type":r.get::<_,String>(2),
            "due_at_epoch":r.get::<_,i64>(3),"owner":r.get::<_,Option<String>>(4),
            "status":r.get::<_,String>(5),"notes":r.get::<_,Option<String>>(6),"outcome":r.get::<_,Option<String>>(7)
        })).collect())
    }

    pub async fn complete_customer_success_task(
        &self, company_id:&str, task_id:&str, outcome:&str
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company=Uuid::parse_str(company_id)?; let task=Uuid::parse_str(task_id)?;
        if outcome.trim().is_empty() { return Err("task outcome is required".into()); }
        let client=self.client.lock().await;
        let changed=client.execute(
            "UPDATE customer_success_tasks SET status='COMPLETED',outcome=$3,updated_at=now()
             WHERE company_id=$1 AND id=$2 AND status IN ('OPEN','IN_PROGRESS')",
            &[&company,&task,&outcome]
        ).await?;
        if changed!=1 { return Err("customer success task not found or already terminal".into()); }
        Ok(())
    }

    pub async fn create_vendor(
        &self,
        company_id: &str,
        vendor_id: Uuid,
        legal_name: &str,
        contact_email: Option<&str>,
        currency: &str,
        tax_ref: Option<&str>,
        idempotency_key: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let currency = currency.trim().to_uppercase();
        if legal_name.trim().is_empty() || legal_name.len() > 200
            || !matches!(currency.len(), 3)
            || idempotency_key.trim().is_empty() || idempotency_key.len() > 256 {
            return Err("invalid vendor".into());
        }
        if let Some(email) = contact_email {
            if email.len() > 320 || !email.contains('@') { return Err("invalid vendor email".into()); }
        }
        let client = self.client.lock().await;
        let row = client.query_opt(
            "INSERT INTO vendors
             (id,company_id,legal_name,contact_email,currency,tax_ref,status,idempotency_key)
             VALUES ($1,$2,$3,$4,$5,$6,'PROSPECT',$7)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING
             RETURNING id,legal_name,contact_email,currency,tax_ref,status,created_at",
            &[&vendor_id,&company,&legal_name,&contact_email,&currency,&tax_ref,&idempotency_key],
        ).await?;
        let row = match row {
            Some(row) => row,
            None => client.query_one(
                "SELECT id,legal_name,contact_email,currency,tax_ref,status,created_at
                 FROM vendors WHERE company_id=$1 AND idempotency_key=$2",
                &[&company,&idempotency_key],
            ).await?,
        };
        Ok(serde_json::json!({
            "id": row.get::<_,Uuid>(0),
            "legal_name": row.get::<_,String>(1),
            "contact_email": row.get::<_,Option<String>>(2),
            "currency": row.get::<_,String>(3),
            "tax_ref": row.get::<_,Option<String>>(4),
            "status": row.get::<_,String>(5),
            "created_at": row.get::<_,time::OffsetDateTime>(6).to_string()
        }))
    }

    pub async fn list_vendors(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        if !(1..=200).contains(&limit) { return Err("vendor limit must be between 1 and 200".into()); }
        let company = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT id,legal_name,contact_email,currency,tax_ref,status,created_at,updated_at
             FROM vendors WHERE company_id=$1 ORDER BY created_at DESC,id DESC LIMIT $2",
            &[&company,&limit],
        ).await?;
        Ok(rows.into_iter().map(|r| serde_json::json!({
            "id":r.get::<_,Uuid>(0),"legal_name":r.get::<_,String>(1),
            "contact_email":r.get::<_,Option<String>>(2),"currency":r.get::<_,String>(3),
            "tax_ref":r.get::<_,Option<String>>(4),"status":r.get::<_,String>(5),
            "created_at":r.get::<_,time::OffsetDateTime>(6).to_string(),
            "updated_at":r.get::<_,time::OffsetDateTime>(7).to_string()
        })).collect())
    }

    pub async fn create_purchase_request(
        &self,
        company_id: &str,
        request_id: Uuid,
        vendor_id: Uuid,
        title: &str,
        currency: &str,
        amount_minor: i128,
        requester: &str,
        idempotency_key: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let currency = currency.trim().to_uppercase();
        if title.trim().is_empty() || requester.trim().is_empty() || amount_minor <= 0
            || currency.len() != 3 || idempotency_key.trim().is_empty() {
            return Err("invalid purchase request".into());
        }
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let vendor = tx.query_opt(
            "SELECT currency,status FROM vendors WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company,&vendor_id],
        ).await?.ok_or("vendor not found")?;
        let vendor_currency: String = vendor.get(0);
        let vendor_status: String = vendor.get(1);
        if vendor_currency != currency { return Err("purchase currency does not match vendor".into()); }
        if !matches!(vendor_status.as_str(),"PROSPECT"|"ACTIVE") { return Err("vendor is not purchasable".into()); }
        let row = tx.query_opt(
            "INSERT INTO purchase_requests
             (id,company_id,vendor_id,title,currency,amount_minor,requester,status,idempotency_key)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7,'PENDING_APPROVAL',$8)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING
             RETURNING id,status",
            &[&request_id,&company,&vendor_id,&title,&currency,&amount_minor.to_string(),&requester,&idempotency_key],
        ).await?;
        let row = match row {
            Some(row) => row,
            None => tx.query_one(
                "SELECT id,status FROM purchase_requests WHERE company_id=$1 AND idempotency_key=$2",
                &[&company,&idempotency_key],
            ).await?,
        };
        tx.commit().await?;
        Ok(serde_json::json!({"id":row.get::<_,Uuid>(0),"status":row.get::<_,String>(1)}))
    }

    pub async fn approve_purchase_request(
        &self,
        company_id: &str,
        request_id: &str,
        approved_by: &str,
        approval_reference: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let request = Uuid::parse_str(request_id)?;
        if approved_by.trim().is_empty() || approval_reference.trim().is_empty() {
            return Err("approval identity and reference are required".into());
        }
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx.query_opt(
            "SELECT status FROM purchase_requests WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company,&request],
        ).await?.ok_or("purchase request not found")?;
        let status: String = row.get(0);
        if status != "PENDING_APPROVAL" { return Err("purchase request is not pending approval".into()); }
        tx.execute(
            "UPDATE purchase_requests SET status='APPROVED',approved_by=$3,approval_reference=$4,updated_at=now()
             WHERE company_id=$1 AND id=$2",
            &[&company,&request,&approved_by,&approval_reference],
        ).await?;
        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'PURCHASE_REQUEST_APPROVED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[&company,&request,&format!("outbox:purchase-approved:{}",request),
              &serde_json::json!({"purchase_request_id":request,"approved_by":approved_by,"approval_reference":approval_reference})],
        ).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn record_vendor_delivery(
        &self,
        company_id: &str,
        delivery_id: Uuid,
        purchase_request_id: &str,
        external_ref: Option<&str>,
        received_at_epoch: i64,
        evidence_hash: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company = Uuid::parse_str(company_id)?;
        let request = Uuid::parse_str(purchase_request_id)?;
        if received_at_epoch <= 0 || evidence_hash.trim().is_empty() { return Err("delivery evidence is incomplete".into()); }
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;
        let row = tx.query_opt(
            "SELECT status FROM purchase_requests WHERE company_id=$1 AND id=$2 FOR UPDATE",
            &[&company,&request],
        ).await?.ok_or("purchase request not found")?;
        let status: String = row.get(0);
        if !matches!(status.as_str(),"APPROVED"|"ORDERED") { return Err("purchase request is not receivable".into()); }
        tx.execute(
            "INSERT INTO vendor_deliveries
             (id,company_id,purchase_request_id,external_ref,received_at_epoch,evidence_hash,status)
             VALUES ($1,$2,$3,$4,$5,$6,'ACCEPTED')",
            &[&delivery_id,&company,&request,&external_ref,&received_at_epoch,&evidence_hash],
        ).await?;
        tx.execute(
            "UPDATE purchase_requests SET status='RECEIVED',updated_at=now()
             WHERE company_id=$1 AND id=$2",
            &[&company,&request],
        ).await?;
        tx.execute(
            "INSERT INTO outbox_events
             (company_id,event_type,aggregate_id,idempotency_key,payload)
             VALUES ($1,'VENDOR_DELIVERY_RECORDED',$2,$3,$4)
             ON CONFLICT(company_id,idempotency_key) DO NOTHING",
            &[&company,&request,&format!("outbox:vendor-delivery:{}:{}",request,evidence_hash),
              &serde_json::json!({"purchase_request_id":request,"delivery_id":delivery_id,"evidence_hash":evidence_hash})],
        ).await?;
        tx.commit().await?;
        Ok(())
    }

}

async fn load_growth_trend(
    tx: &tokio_postgres::Transaction<'_>,
    company_id: &Uuid,
    trend_key: &str,
) -> Result<Option<GrowthTrendRecord>, Box<dyn std::error::Error + Send + Sync>> {
    let row = tx.query_opt(
        "SELECT id,company_id,trend_key,topic,source,evidence_ref,observed_at_epoch,
                velocity_bps,audience_fit_bps,product_fit_bps,contentability_bps,competition_bps,
                confidence_bps,product_ref,offer_ref,content_format,max_budget_minor::text,max_loss_minor::text,
                max_duration_seconds,success_metric,success_threshold_bps,policy_evidence_ref,
                score_bps,decision,created_at::text
           FROM growth_trends
          WHERE company_id=$1 AND trend_key=$2",
        &[company_id, &trend_key],
    ).await?;
    row.map(growth_trend_from_row).transpose()
}

async fn load_growth_opportunity_by_trend(
    tx: &tokio_postgres::Transaction<'_>,
    company_id: &Uuid,
    trend_id: Uuid,
) -> Result<Option<GrowthOpportunityRecord>, Box<dyn std::error::Error + Send + Sync>> {
    let row = tx.query_opt(
        "SELECT o.id,o.company_id,o.trend_id,o.opportunity_key,o.title,o.score_bps,o.confidence_bps,
                o.policy_evidence_ref,o.plan_json,o.status,o.content_item_id,o.content_created_at_epoch,
                CASE WHEN o.content_created_at_epoch IS NULL THEN NULL
                     ELSE GREATEST(o.content_created_at_epoch - t.observed_at_epoch, 0) END AS ttfc_seconds,
                o.created_at::text
           FROM growth_opportunities o
          JOIN growth_trends t ON t.id=o.trend_id AND t.company_id=o.company_id
          WHERE o.company_id=$1 AND o.trend_id=$2",
        &[company_id, &trend_id],
    ).await?;
    row.map(growth_opportunity_from_row).transpose()
}

fn growth_trend_from_row(
    row: tokio_postgres::Row,
) -> Result<GrowthTrendRecord, Box<dyn std::error::Error + Send + Sync>> {
    let decision = match row.get::<_, String>(23).as_str() {
        "PURSUE" => company_growth::TrendDecision::Pursue,
        "MONITOR" => company_growth::TrendDecision::Monitor,
        "REJECT" => company_growth::TrendDecision::Reject,
        other => return Err(format!("invalid growth trend decision: {other}").into()),
    };
    let signal = company_growth::TrendSignal {
        company_id: row.get(1),
        trend_key: row.get(2),
        topic: row.get(3),
        source: row.get(4),
        evidence_ref: row.get(5),
        observed_at_epoch: row.get(6),
        velocity_bps: row.get::<_, i32>(7) as u32,
        audience_fit_bps: row.get::<_, i32>(8) as u32,
        product_fit_bps: row.get::<_, i32>(9) as u32,
        contentability_bps: row.get::<_, i32>(10) as u32,
        competition_bps: row.get::<_, i32>(11) as u32,
        confidence_bps: row.get::<_, i32>(12) as u32,
        product_ref: row.get(13),
        offer_ref: row.get(14),
        content_format: parse_content_format(row.get::<_, String>(15))?,
        max_budget_minor: row.get::<_, String>(16).parse()?,
        max_loss_minor: row.get::<_, String>(17).parse()?,
        max_duration_seconds: row.get::<_, i64>(18) as u32,
        success_metric: parse_success_metric(row.get::<_, String>(19))?,
        success_threshold_bps: row.get::<_, i32>(20) as u32,
        policy_evidence_ref: row.get(21),
    };
    company_growth::validate_trend(&signal).map_err(|error| error.to_string())?;
    Ok(GrowthTrendRecord {
        id: row.get(0),
        signal,
        score_bps: row.get::<_, i32>(22) as u32,
        decision,
        created_at: row.get(24),
    })
}

async fn load_safety_controls_for_tx(
    tx: &tokio_postgres::Transaction<'_>,
    company: Uuid,
) -> Result<company_safety_controls::SafetyControls, Box<dyn std::error::Error + Send + Sync>> {
    let row = tx
        .query_one(
            "SELECT emergency_stop_enabled, emergency_stop_reason, emergency_stop_actor,
                    emergency_stop_changed_at_epoch, content_publish_daily::text,
                    ads_spend_daily_minor::text, live_minutes_daily::text,
                    outbound_messages_daily::text, autonomous_capital_daily_minor::text,
                    updated_at_epoch, updated_at::text
               FROM autonomy_control_state
              WHERE company_id=$1
              FOR SHARE",
            &[&company],
        )
        .await?;
    safety_controls_from_row(&row, company)
}

fn safety_controls_from_row(
    row: &tokio_postgres::Row,
    company: Uuid,
) -> Result<company_safety_controls::SafetyControls, Box<dyn std::error::Error + Send + Sync>> {
    let controls = company_safety_controls::SafetyControls {
        company_id: company,
        emergency_stop: company_safety_controls::EmergencyStop {
            enabled: row.get(0),
            reason: row.get(1),
            actor: row.get(2),
            changed_at_epoch: row.get(3),
        },
        budgets: company_safety_controls::AutonomyBudgets {
            content_publish_daily: parse_i128_numeric(&row.get::<_, String>(4))?,
            ads_spend_daily_minor: parse_i128_numeric(&row.get::<_, String>(5))?,
            live_minutes_daily: parse_i128_numeric(&row.get::<_, String>(6))?,
            outbound_messages_daily: parse_i128_numeric(&row.get::<_, String>(7))?,
            autonomous_capital_daily_minor: parse_i128_numeric(&row.get::<_, String>(8))?,
        },
        updated_at_epoch: row.get(9),
    };
    controls
        .validate()
        .map_err(|error| error.to_string())?;
    Ok(controls)
}

fn tiktok_connection_from_row(
    company_id: Uuid,
    row: &tokio_postgres::Row,
) -> Result<TikTokConnectionRecord, Box<dyn std::error::Error + Send + Sync>> {
    let access_expires = row
        .get::<_, Option<i64>>(3)
        .ok_or("TikTok access token expiry is unavailable")?;
    let refresh_expires = row
        .get::<_, Option<i64>>(4)
        .ok_or("TikTok refresh token expiry is unavailable")?;
    let status: String = row.get(5);
    if !matches!(status.as_str(), "ACTIVE" | "REVOKED" | "REAUTH_REQUIRED") {
        return Err("invalid stored TikTok OAuth status".into());
    }
    Ok(TikTokConnectionRecord {
        company_id,
        open_id: row.get(0),
        scopes: row.get(1),
        token_type: row.get(2),
        access_token_expires_at_epoch: access_expires,
        refresh_token_expires_at_epoch: refresh_expires,
        status,
        last_error: row.get(6),
        updated_at: row.get(7),
    })
}

fn growth_opportunity_from_row(
    row: tokio_postgres::Row,
) -> Result<GrowthOpportunityRecord, Box<dyn std::error::Error + Send + Sync>> {
    let id: Uuid = row.get(0);
    let company_id: Uuid = row.get(1);
    let trend_id: Uuid = row.get(2);
    let plan: company_growth::ContentPlan = serde_json::from_value(row.get(8))?;
    let opportunity = company_growth::Opportunity {
        id,
        company_id,
        trend_id,
        opportunity_key: row.get(3),
        title: row.get(4),
        score_bps: row.get::<_, i32>(5) as u32,
        confidence_bps: row.get::<_, i32>(6) as u32,
        policy_evidence_ref: row.get(7),
        plan,
    };
    let status = match row.get::<_, String>(9).as_str() {
        "READY" => company_growth::OpportunityStatus::Ready,
        "CONTENT_CREATED" => company_growth::OpportunityStatus::ContentCreated,
        other => return Err(format!("invalid growth opportunity status: {other}").into()),
    };
    Ok(GrowthOpportunityRecord {
        opportunity,
        status,
        content_item_id: row.get(10),
        content_created_at_epoch: row.get(11),
        ttfc_seconds: row.get(12),
        created_at: row.get(13),
    })
}

async fn content_observation_by_key(
    client: &tokio_postgres::Client,
    company_id: &Uuid,
    observation_key: &str,
) -> Result<ContentObservationRecord, Box<dyn std::error::Error + Send + Sync>> {
    let row = client.query_one(
        "SELECT id,content_id,source,evidence_hash,observed_at_epoch,sample_count,
                spend_minor::text,metric_bps,views,clicks,conversions,commission_minor::text,
                contribution_margin_minor::text,decision,created_at::text
           FROM content_observations
          WHERE company_id=$1 AND observation_key=$2",
        &[company_id, &observation_key],
    ).await?;
    let decision = parse_content_decision(Some(row.get::<_, String>(13)))?
        .ok_or("content observation decision is missing")?;
    Ok(ContentObservationRecord {
        id: row.get(0),
        observation: company_content::ContentObservation {
            observation_key: observation_key.to_string(),
            content_id: row.get(1),
            company_id: *company_id,
            source: row.get(2),
            evidence_hash: row.get(3),
            observed_at_epoch: row.get(4),
            sample_count: row.get::<_, i64>(5) as u64,
            spend_minor: row.get::<_, String>(6).parse()?,
            metric_bps: row.get::<_, i32>(7) as u32,
            views: row.get::<_, i64>(8) as u64,
            clicks: row.get::<_, i64>(9) as u64,
            conversions: row.get::<_, i64>(10) as u64,
            commission_minor: row.get::<_, String>(11).parse()?,
            contribution_margin_minor: row.get::<_, String>(12).parse()?,
        },
        decision,
        created_at: row.get(14),
    })
}

fn experiment_learning_entry(
    experiment_id: Uuid,
    spec: &company_experiments::ExperimentSpec,
    observation: &company_experiments::ExperimentObservation,
    decision: company_experiments::ExperimentDecision,
    observation_key: &str,
) -> company_learning::LearningEntry {
    let min_observations = observation
        .control_observations
        .min(observation.treatment_observations);
    let required_observations = spec.min_observations.max(1);
    let confidence_bps =
        ((min_observations.min(required_observations) as u128 * 10_000)
            / required_observations as u128) as i64;

    let (kind, severity, learning_decision, root_cause, corrective_action, reusable_rule) =
        match decision {
            company_experiments::ExperimentDecision::Succeed => (
                company_learning::LearningKind::Success,
                company_learning::FailureSeverity::None,
                company_learning::LearningDecision::Reuse,
                "Recorded treatment lift met the configured success threshold.",
                "Carry the treatment forward only with new verified outcome evidence.",
                "A treatment that clears the configured success threshold is eligible for follow-up validation.",
            ),
            company_experiments::ExperimentDecision::Kill => {
                let cause = if observation.spend_minor >= spec.max_budget_minor {
                    "Recorded experiment spend reached the configured maximum budget."
                } else {
                    "Recorded treatment lift reached the configured kill boundary."
                };
                (
                    company_learning::LearningKind::Failure,
                    company_learning::FailureSeverity::Medium,
                    company_learning::LearningDecision::Stop,
                    cause,
                    "Stop the treatment under the current hypothesis and revise before retesting.",
                    "Do not reuse a killed treatment under the same evidence conditions.",
                )
            }
            company_experiments::ExperimentDecision::Expire => (
                company_learning::LearningKind::Learning,
                company_learning::FailureSeverity::None,
                company_learning::LearningDecision::Retest,
                "Recorded elapsed time reached the configured duration before a terminal success or kill boundary.",
                "Retest only with a new evidence plan or revised duration.",
                "Do not treat an expired experiment as validated; retest with new evidence.",
            ),
            company_experiments::ExperimentDecision::Continue => (
                company_learning::LearningKind::Learning,
                company_learning::FailureSeverity::None,
                company_learning::LearningDecision::Adjust,
                "The recorded observation did not reach a terminal threshold.",
                "Continue collecting evidence before changing the treatment.",
                "Do not treat an in-flight experiment as validated learning.",
            ),
        };

    company_learning::LearningEntry {
        entry_key: format!("experiment:{}:learning:{}", experiment_id, observation_key),
        source_type: "EXPERIMENT_DECISION".into(),
        source_id: experiment_id.to_string(),
        kind,
        severity,
        hypothesis: spec.hypothesis.clone(),
        context: format!(
            "control={} treatment={} success_threshold_bps={} kill_threshold_bps={}",
            spec.control, spec.treatment, spec.success_metric_bps, spec.kill_metric_bps
        ),
        expected_outcome: format!(
            "Treatment lift reaches at least {} bps above control.",
            spec.success_metric_bps
        ),
        actual_outcome: format!(
            "decision={:?}; control_observations={}; treatment_observations={}; control_metric_bps={}; treatment_metric_bps={}; spend_minor={}; elapsed_seconds={}; observation_key={}",
            decision,
            observation.control_observations,
            observation.treatment_observations,
            observation.control_metric_bps,
            observation.treatment_metric_bps,
            observation.spend_minor,
            observation.elapsed_seconds,
            observation_key
        ),
        impact_minor: -observation.spend_minor,
        confidence_bps,
        root_cause: root_cause.into(),
        corrective_action: corrective_action.into(),
        reusable_rule: reusable_rule.into(),
        decision: learning_decision,
    }
}

fn learning_kind_name(value: company_learning::LearningKind) -> &'static str {
    match value {
        company_learning::LearningKind::Learning => "LEARNING",
        company_learning::LearningKind::Failure => "FAILURE",
        company_learning::LearningKind::NearMiss => "NEAR_MISS",
        company_learning::LearningKind::Success => "SUCCESS",
    }
}

fn failure_severity_name(value: company_learning::FailureSeverity) -> &'static str {
    match value {
        company_learning::FailureSeverity::None => "NONE",
        company_learning::FailureSeverity::Low => "LOW",
        company_learning::FailureSeverity::Medium => "MEDIUM",
        company_learning::FailureSeverity::High => "HIGH",
        company_learning::FailureSeverity::Critical => "CRITICAL",
    }
}

fn learning_decision_name(value: company_learning::LearningDecision) -> &'static str {
    match value {
        company_learning::LearningDecision::Reuse => "REUSE",
        company_learning::LearningDecision::Adjust => "ADJUST",
        company_learning::LearningDecision::Retest => "RETEST",
        company_learning::LearningDecision::Stop => "STOP",
        company_learning::LearningDecision::Escalate => "ESCALATE",
    }
}

fn content_format_name(value: company_content::ContentFormat) -> &'static str {
    match value {
        company_content::ContentFormat::ShortVideo => "SHORT_VIDEO",
        company_content::ContentFormat::LiveSegment => "LIVE_SEGMENT",
        company_content::ContentFormat::Story => "STORY",
        company_content::ContentFormat::Carousel => "CAROUSEL",
    }
}

fn success_metric_name(value: company_content::SuccessMetric) -> &'static str {
    match value {
        company_content::SuccessMetric::Views => "VIEWS",
        company_content::SuccessMetric::ClickThroughRate => "CLICK_THROUGH_RATE",
        company_content::SuccessMetric::ConversionRate => "CONVERSION_RATE",
        company_content::SuccessMetric::Commission => "COMMISSION",
        company_content::SuccessMetric::ContributionMargin => "CONTRIBUTION_MARGIN",
    }
}

fn content_status_name(value: company_content::ContentStatus) -> &'static str {
    match value {
        company_content::ContentStatus::Draft => "DRAFT",
        company_content::ContentStatus::Approved => "APPROVED",
        company_content::ContentStatus::Rendered => "RENDERED",
        company_content::ContentStatus::Published => "PUBLISHED",
        company_content::ContentStatus::Measured => "MEASURED",
        company_content::ContentStatus::Paused => "PAUSED",
        company_content::ContentStatus::Killed => "KILLED",
    }
}

fn growth_trend_decision_name(value: company_growth::TrendDecision) -> &'static str {
    match value {
        company_growth::TrendDecision::Pursue => "PURSUE",
        company_growth::TrendDecision::Monitor => "MONITOR",
        company_growth::TrendDecision::Reject => "REJECT",
    }
}

fn content_decision_name(value: company_content::ContentDecision) -> &'static str {
    match value {
        company_content::ContentDecision::Scale => "SCALE",
        company_content::ContentDecision::Iterate => "ITERATE",
        company_content::ContentDecision::Pause => "PAUSE",
        company_content::ContentDecision::Kill => "KILL",
    }
}

fn parse_content_format(value: &str) -> Result<company_content::ContentFormat, std::io::Error> {
    match value {
        "SHORT_VIDEO" => Ok(company_content::ContentFormat::ShortVideo),
        "LIVE_SEGMENT" => Ok(company_content::ContentFormat::LiveSegment),
        "STORY" => Ok(company_content::ContentFormat::Story),
        "CAROUSEL" => Ok(company_content::ContentFormat::Carousel),
        _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid content format: {value}"))),
    }
}

fn parse_success_metric(value: &str) -> Result<company_content::SuccessMetric, std::io::Error> {
    match value {
        "VIEWS" => Ok(company_content::SuccessMetric::Views),
        "CLICK_THROUGH_RATE" => Ok(company_content::SuccessMetric::ClickThroughRate),
        "CONVERSION_RATE" => Ok(company_content::SuccessMetric::ConversionRate),
        "COMMISSION" => Ok(company_content::SuccessMetric::Commission),
        "CONTRIBUTION_MARGIN" => Ok(company_content::SuccessMetric::ContributionMargin),
        _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid success metric: {value}"))),
    }
}

fn parse_content_status(value: &str) -> Result<company_content::ContentStatus, std::io::Error> {
    match value {
        "DRAFT" => Ok(company_content::ContentStatus::Draft),
        "APPROVED" => Ok(company_content::ContentStatus::Approved),
        "RENDERED" => Ok(company_content::ContentStatus::Rendered),
        "PUBLISHED" => Ok(company_content::ContentStatus::Published),
        "MEASURED" => Ok(company_content::ContentStatus::Measured),
        "PAUSED" => Ok(company_content::ContentStatus::Paused),
        "KILLED" => Ok(company_content::ContentStatus::Killed),
        _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid content status: {value}"))),
    }
}

fn parse_content_decision(value: Option<String>) -> Result<Option<company_content::ContentDecision>, std::io::Error> {
    match value.as_deref() {
        None => Ok(None),
        Some("SCALE") => Ok(Some(company_content::ContentDecision::Scale)),
        Some("ITERATE") => Ok(Some(company_content::ContentDecision::Iterate)),
        Some("PAUSE") => Ok(Some(company_content::ContentDecision::Pause)),
        Some("KILL") => Ok(Some(company_content::ContentDecision::Kill)),
        Some(other) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid content decision: {other}"))),
    }
}

fn parse_rfc3339_epoch(value: &str) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    Ok(time::OffsetDateTime::parse(
        value,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|error| format!("invalid RFC3339 timestamp: {error}"))?
    .unix_timestamp())
}

fn hashed_graph_ref(prefix: &str, value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!(
        "{}:sha256:{}",
        prefix,
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn graph_source(value: &str) -> Result<&str, Box<dyn std::error::Error + Send + Sync>> {
    let value = value.trim();
    if value.is_empty() {
        return Ok("affiliate");
    }
    if value.len() > 256 {
        return Err("revenue graph source exceeds 256 bytes".into());
    }
    Ok(value)
}

fn new_graph_edge(
    company_id: Uuid,
    from_type: company_revenue_graph::RevenueNodeType,
    from_ref: &str,
    relation: &str,
    to_type: company_revenue_graph::RevenueNodeType,
    to_ref: &str,
    value_minor: Option<i128>,
    currency: Option<&str>,
    confidence_bps: u32,
    evidence_ref: &str,
    source: &str,
    observed_at_epoch: i64,
) -> company_revenue_graph::RevenueGraphEdge {
    let edge_key = company_revenue_graph::build_edge_key(
        from_type,
        from_ref,
        relation,
        to_type,
        to_ref,
    );
    company_revenue_graph::RevenueGraphEdge {
        id: company_revenue_graph::RevenueGraphEdge::deterministic_id(company_id, &edge_key),
        company_id,
        edge_key,
        from_type,
        from_ref: from_ref.trim().to_owned(),
        relation: relation.trim().to_owned(),
        to_type,
        to_ref: to_ref.trim().to_owned(),
        value_minor,
        currency: currency.map(|value| value.trim().to_ascii_uppercase()),
        confidence_bps,
        evidence_ref: evidence_ref.trim().to_owned(),
        source: source.trim().to_owned(),
        observed_at_epoch,
    }
}

async fn record_revenue_graph_edge_tx(
    tx: &Transaction<'_>,
    edge: &company_revenue_graph::RevenueGraphEdge,
) -> Result<company_revenue_graph::RevenueGraphEdge, Box<dyn std::error::Error + Send + Sync>> {
    company_revenue_graph::validate_edge(edge).map_err(|error| error.to_string())?;

    if let Some(row) = tx
        .query_opt(
            "SELECT id,company_id,edge_key,from_type,from_ref,relation,to_type,to_ref,
                    value_minor::text,currency,confidence_bps,evidence_ref,source,
                    observed_at_epoch,created_at::text
               FROM revenue_graph_edges
              WHERE company_id=$1 AND edge_key=$2
              FOR UPDATE",
            &[&edge.company_id, &edge.edge_key],
        )
        .await?
    {
        let stored = revenue_graph_edge_from_row(row)?;
        if stored != *edge {
            return Err("revenue graph edge key already exists with different evidence".into());
        }
        return Ok(stored);
    }

    let id = company_revenue_graph::RevenueGraphEdge::deterministic_id(
        edge.company_id,
        &edge.edge_key,
    );
    if id != edge.id {
        return Err("revenue graph edge id must be deterministic from company and edge key".into());
    }

    tx.execute(
        "INSERT INTO revenue_graph_edges
         (id,company_id,edge_key,from_type,from_ref,relation,to_type,to_ref,
          value_minor,currency,confidence_bps,evidence_ref,source,observed_at_epoch)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10,$11,$12,$13,$14)",
        &[
            &edge.id,
            &edge.company_id,
            &edge.edge_key,
            &edge.from_type.as_str(),
            &edge.from_ref,
            &edge.relation,
            &edge.to_type.as_str(),
            &edge.to_ref,
            &edge.value_minor.map(|value| value.to_string()),
            &edge.currency,
            &(edge.confidence_bps as i32),
            &edge.evidence_ref,
            &edge.source,
            &edge.observed_at_epoch,
        ],
    )
    .await?;

    tx.execute(
        "INSERT INTO outbox_events
         (company_id,event_type,aggregate_id,idempotency_key,payload)
         VALUES ($1,'REVENUE_GRAPH_EDGE_RECORDED',$2,$3,$4)
         ON CONFLICT(company_id,idempotency_key) DO NOTHING",
        &[
            &edge.company_id,
            &edge.id,
            &format!("outbox:revenue-graph:{}", edge.edge_key),
            &serde_json::to_value(edge)?,
        ],
    )
    .await?;

    Ok(edge.clone())
}

fn revenue_graph_edge_from_row(
    row: tokio_postgres::Row,
) -> Result<company_revenue_graph::RevenueGraphEdge, Box<dyn std::error::Error + Send + Sync>> {
    let from_type = company_revenue_graph::RevenueNodeType::parse(row.get::<_, String>(3))
        .ok_or("unknown revenue graph from_type")?;
    let to_type = company_revenue_graph::RevenueNodeType::parse(row.get::<_, String>(6))
        .ok_or("unknown revenue graph to_type")?;
    let value_minor = row
        .get::<_, Option<String>>(8)
        .map(|value| parse_i128_numeric(&value))
        .transpose()?;
    Ok(company_revenue_graph::RevenueGraphEdge {
        id: row.get(0),
        company_id: row.get(1),
        edge_key: row.get(2),
        from_type,
        from_ref: row.get(4),
        relation: row.get(5),
        to_type,
        to_ref: row.get(7),
        value_minor,
        currency: row.get(9),
        confidence_bps: row.get::<_, i32>(10) as u32,
        evidence_ref: row.get(11),
        source: row.get(12),
        observed_at_epoch: row.get(13),
    })
}

fn content_record_from_row(
    row: tokio_postgres::Row,
) -> Result<ContentRecord, Box<dyn std::error::Error + Send + Sync>> {
    let item = company_content::ContentItem {
        id: row.get(0),
        company_id: row.get(1),
        brief: company_content::ContentBrief {
            hypothesis: row.get(2),
            audience: row.get(3),
            format: parse_content_format(row.get::<_, String>(4))?,
            product_ref: row.get(5),
            offer_ref: row.get(6),
            disclosure_required: row.get(7),
            expected_cost_minor: row.get::<_, String>(8).parse()?,
            max_loss_minor: row.get::<_, String>(9).parse()?,
            max_duration_seconds: row.get::<_, i64>(10) as u32,
            success_metric: parse_success_metric(row.get::<_, String>(11))?,
            success_threshold_bps: row.get::<_, i32>(12) as u32,
        },
        variant: company_content::CreativeVariant {
            variant_key: row.get(13),
            hook: row.get(14),
            first_frame: row.get(15),
            emotion: row.get(16),
            pacing: row.get(17),
            scene_count: row.get::<_, i32>(18) as u8,
            text_density: row.get(19),
            voice_speed: row.get(20),
            product_placement: row.get(21),
            cta: row.get(22),
            comment_trigger: row.get(23),
            music_style: row.get(24),
            visual_style: row.get(25),
        },
        status: parse_content_status(row.get::<_, String>(26))?,
        decision: parse_content_decision(row.get::<_, Option<String>>(27))?,
    };
    company_content::validate_item(&item).map_err(|error| error.to_string())?;
    Ok(ContentRecord { item, created_at: row.get(28) })
}
#[cfg(test)]
mod experiment_learning_tests {
    use super::*;

    fn spec() -> company_experiments::ExperimentSpec {
        company_experiments::ExperimentSpec {
            hypothesis: "short hook improves conversion".into(),
            control: "baseline".into(),
            treatment: "short-hook".into(),
            max_budget_minor: 1_000,
            min_observations: 100,
            duration_seconds: 86_400,
            success_metric_bps: 500,
            kill_metric_bps: 300,
        }
    }

    fn observation() -> company_experiments::ExperimentObservation {
        company_experiments::ExperimentObservation {
            control_observations: 100,
            treatment_observations: 120,
            control_metric_bps: 500,
            treatment_metric_bps: 1_000,
            spend_minor: 250,
            elapsed_seconds: 3_600,
        }
    }

    #[test]
    fn terminal_experiment_decision_becomes_evidence_backed_learning() {
        let entry = experiment_learning_entry(
            Uuid::from_u128(1),
            &spec(),
            &observation(),
            company_experiments::ExperimentDecision::Succeed,
            "obs-1",
        );

        assert_eq!(entry.entry_key, "experiment:00000000-0000-0000-0000-000000000001:learning:obs-1");
        assert_eq!(entry.kind, company_learning::LearningKind::Success);
        assert_eq!(entry.decision, company_learning::LearningDecision::Reuse);
        assert_eq!(entry.impact_minor, -250);
        assert_eq!(entry.confidence_bps, 10_000);
        assert!(company_learning::validate_evidence(&entry).is_ok());
    }

    #[test]
    fn learning_confidence_reflects_observation_coverage() {
        let mut value = observation();
        value.control_observations = 50;
        value.treatment_observations = 100;

        let entry = experiment_learning_entry(
            Uuid::from_u128(1),
            &spec(),
            &value,
            company_experiments::ExperimentDecision::Succeed,
            "obs-2",
        );

        assert_eq!(entry.confidence_bps, 5_000);
    }
}

#[cfg(test)]
mod revenue_period_tests {
    use super::CompanyStore;

    #[test]
    fn forecast_uses_only_mtd_and_calendar_coverage() {
        assert_eq!(CompanyStore::revenue_period_projection(10_000, 99_000, 10, 30), (30_000, 99_000, 3333));
        assert_eq!(CompanyStore::revenue_period_projection(10_000, 99_000, 30, 30), (10_000, 99_000, 10_000));
    }

    #[test]
    fn invalid_period_inputs_fail_closed_to_observed_values() {
        assert_eq!(CompanyStore::revenue_period_projection(10_000, 99_000, 0, 30), (10_000, 99_000, 0));
        assert_eq!(CompanyStore::revenue_period_projection(10_000, 99_000, 31, 30), (10_000, 99_000, 10_000));
    }
}
