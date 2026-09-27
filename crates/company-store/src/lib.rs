#![forbid(unsafe_code)]

use agent_runtime::{AgentRunResult, CompanySnapshot};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

pub struct CompanyStore {
    client: Client,
}

impl CompanyStore {
    pub async fn connect(database_url: &str) -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("postgres connection error: {error}");
            }
        });
        Ok(Self { client })
    }

    pub async fn migrate(&self) -> Result<(), tokio_postgres::Error> {
        self.client.batch_execute(include_str!("../../../infra/db/migrations/001_economic_kernel.sql")).await
    }

    pub async fn ensure_company(
        &self,
        company_id: &str,
        name: &str,
        currency: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::parse_str(company_id)?;
        self.client
            .execute(
                "INSERT INTO companies (id, name, status, base_currency) VALUES ($1, $2, 'ACTIVE', $3) ON CONFLICT (id) DO NOTHING",
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
        let row = self
            .client
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
        self.client
            .execute(
                "INSERT INTO company_state_snapshots (company_id, state) VALUES ($1, $2)
                 ON CONFLICT (company_id) DO UPDATE SET state = EXCLUDED.state, updated_at = now()",
                &[&id, &state],
            )
            .await?;
        Ok(())
    }

    pub async fn append_agent_runs(
        &self,
        snapshot: &CompanySnapshot,
        results: &[AgentRunResult],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company_id = Uuid::parse_str(&snapshot.company_id)?;
        for result in results {
            let payload = serde_json::to_value(result)?;
            self.client
                .execute(
                    "INSERT INTO agent_runs (company_id, agent_name, payload) VALUES ($1, $2, $3)",
                    &[&company_id, &result.agent.as_str(), &payload],
                )
                .await?;
        }
        Ok(())
    }
}
