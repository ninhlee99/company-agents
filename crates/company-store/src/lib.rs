#![forbid(unsafe_code)]

use agent_runtime::{
    ExecutionEngine, ExecutionOutcome, AgentRunResult, CompanySnapshot,
};
use affiliate_attribution::{attribute, AttributionModel, AttributionResult, ClickTouch, OrderEvent, OrderStatus};
use affiliate_intelligence::{AffiliateSearchResult, ProductSearchQuery};
use company_domain::{
    BusinessUnit, ContentAsset, Contract, CreatorUnit, Customer, Employee, Experiment, PayrollRun,
    Product, Task,
};
use economic_core::{validate_balanced_transaction, LedgerEntry, LedgerTransaction};
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
            .await?;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/004_affiliate_searches.sql"))
            .await?;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/005_durable_scheduler.sql"))
            .await?;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/006_company_operations.sql"))
            .await?;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/007_affiliate_attribution.sql"))
            .await?;
        client
            .batch_execute(include_str!("../../../infra/db/migrations/008_commercial_and_payroll.sql"))
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
        if idempotency_key.trim().is_empty() {
            return Err("ledger idempotency key is required".into());
        }
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

    pub async fn save_business_unit(
        &self,
        company_id: &str,
        unit: &BusinessUnit,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        unit.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO business_units
             (id,company_id,name,currency,status,cash_minor,revenue_minor,expenses_minor)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8::numeric)
             ON CONFLICT (id) DO UPDATE SET
               name=EXCLUDED.name,currency=EXCLUDED.currency,status=EXCLUDED.status,
               cash_minor=EXCLUDED.cash_minor,revenue_minor=EXCLUDED.revenue_minor,
               expenses_minor=EXCLUDED.expenses_minor,updated_at=now()
             WHERE business_units.company_id=EXCLUDED.company_id",
            &[
                &unit.id,&company_uuid,&unit.name,&unit.currency,&format!("{:?}",unit.status),
                &unit.cash_minor.to_string(),&unit.revenue_minor.to_string(),&unit.expenses_minor.to_string()
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_customer(
        &self,
        company_id: &str,
        customer: &Customer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        customer.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO customers
             (id,company_id,name,external_ref,status,lifetime_revenue_minor)
             VALUES ($1,$2,$3,$4,$5,$6::numeric)
             ON CONFLICT (id) DO UPDATE SET
               name=EXCLUDED.name,external_ref=EXCLUDED.external_ref,status=EXCLUDED.status,
               lifetime_revenue_minor=EXCLUDED.lifetime_revenue_minor,updated_at=now()
             WHERE customers.company_id=EXCLUDED.company_id",
            &[
                &customer.id,&company_uuid,&customer.name,&customer.external_ref,
                &format!("{:?}",customer.status),&customer.lifetime_revenue_minor.to_string()
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_product(
        &self,
        company_id: &str,
        product: &Product,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        product.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO products
             (id,company_id,name,category,currency,price_minor,active)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7)
             ON CONFLICT (id) DO UPDATE SET
               name=EXCLUDED.name,category=EXCLUDED.category,currency=EXCLUDED.currency,
               price_minor=EXCLUDED.price_minor,active=EXCLUDED.active,updated_at=now()
             WHERE products.company_id=EXCLUDED.company_id",
            &[
                &product.id,&company_uuid,&product.name,&product.category,&product.currency,
                &product.price_minor.to_string(),&product.active
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_payroll_run(
        &self,
        company_id: &str,
        payroll: &PayrollRun,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        payroll.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        tx.execute(
            "INSERT INTO payroll_runs
             (id,company_id,period_start_epoch,period_end_epoch,gross_minor,employer_cost_minor,cash_due_minor)
             VALUES ($1,$2,$3,$4,$5::numeric,$6::numeric,$7::numeric)
             ON CONFLICT (id) DO UPDATE SET
               period_start_epoch=EXCLUDED.period_start_epoch,period_end_epoch=EXCLUDED.period_end_epoch,
               gross_minor=EXCLUDED.gross_minor,employer_cost_minor=EXCLUDED.employer_cost_minor,
               cash_due_minor=EXCLUDED.cash_due_minor
             WHERE payroll_runs.company_id=EXCLUDED.company_id",
            &[
                &payroll.id,&company_uuid,&payroll.period_start_epoch,&payroll.period_end_epoch,
                &payroll.gross_minor().to_string(),&payroll.employer_cost_minor().to_string(),
                &payroll.cash_due_minor().to_string()
            ],
        ).await?;

        tx.execute("DELETE FROM payroll_lines WHERE payroll_run_id=$1", &[&payroll.id]).await?;
        for line in &payroll.lines {
            tx.execute(
                "INSERT INTO payroll_lines
                 (payroll_run_id,employee_id,gross_minor,employer_cost_minor,withholding_minor)
                 VALUES ($1,$2,$3::numeric,$4::numeric,$5::numeric)",
                &[
                    &payroll.id,&line.employee_id,&line.gross_minor.to_string(),
                    &line.employer_cost_minor.to_string(),&line.withholding_minor.to_string()
                ],
            ).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn save_creator(
        &self,
        company_id: &str,
        creator: &CreatorUnit,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        creator.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO creators
             (id,company_id,name,currency,status,cash_minor,revenue_minor,expenses_minor,audience,content_count)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8::numeric,$9,$10)
             ON CONFLICT (id) DO UPDATE SET
               name=EXCLUDED.name,currency=EXCLUDED.currency,status=EXCLUDED.status,
               cash_minor=EXCLUDED.cash_minor,revenue_minor=EXCLUDED.revenue_minor,
               expenses_minor=EXCLUDED.expenses_minor,audience=EXCLUDED.audience,
               content_count=EXCLUDED.content_count,updated_at=now()
             WHERE creators.company_id=EXCLUDED.company_id",
            &[
                &creator.id,&company_uuid,&creator.name,&creator.currency,
                &format!("{:?}",creator.status),
                &creator.cash_minor.to_string(),&creator.revenue_minor.to_string(),
                &creator.expenses_minor.to_string(),&(creator.audience as i64),&(creator.content_count as i64)
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_content_asset(
        &self,
        company_id: &str,
        content: &ContentAsset,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        content.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO content_assets
             (id,company_id,creator_id,title,channel,status,production_cost_minor,attributed_revenue_minor,
              affiliate_commission_minor,views,clicks,orders,published_at_epoch)
             VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8::numeric,$9::numeric,$10,$11,$12,$13)
             ON CONFLICT (id) DO UPDATE SET
               title=EXCLUDED.title,channel=EXCLUDED.channel,status=EXCLUDED.status,
               production_cost_minor=EXCLUDED.production_cost_minor,
               attributed_revenue_minor=EXCLUDED.attributed_revenue_minor,
               affiliate_commission_minor=EXCLUDED.affiliate_commission_minor,
               views=EXCLUDED.views,clicks=EXCLUDED.clicks,orders=EXCLUDED.orders,
               published_at_epoch=EXCLUDED.published_at_epoch,updated_at=now()
             WHERE content_assets.company_id=EXCLUDED.company_id",
            &[
                &content.id,&company_uuid,&content.creator_id,&content.title,&content.channel,
                &format!("{:?}",content.status),&content.production_cost_minor.to_string(),
                &content.attributed_revenue_minor.to_string(),&content.affiliate_commission_minor.to_string(),
                &(content.views as i64),&(content.clicks as i64),&(content.orders as i64),&content.published_at_epoch
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_experiment(
        &self,
        company_id: &str,
        experiment: &Experiment,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        experiment.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO experiments
             (id,company_id,name,hypothesis,status,budget_minor,spent_minor,expected_revenue_minor)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8::numeric)
             ON CONFLICT (id) DO UPDATE SET
               name=EXCLUDED.name,hypothesis=EXCLUDED.hypothesis,status=EXCLUDED.status,
               budget_minor=EXCLUDED.budget_minor,spent_minor=EXCLUDED.spent_minor,
               expected_revenue_minor=EXCLUDED.expected_revenue_minor,updated_at=now()
             WHERE experiments.company_id=EXCLUDED.company_id",
            &[
                &experiment.id,&company_uuid,&experiment.name,&experiment.hypothesis,
                &format!("{:?}",experiment.status),&experiment.budget_minor.to_string(),
                &experiment.spent_minor.to_string(),&experiment.expected_revenue_minor.to_string()
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_employee(
        &self,
        company_id: &str,
        employee: &Employee,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        employee.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO employees
             (id,company_id,name,role,status,monthly_cost_minor,start_epoch)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7)
             ON CONFLICT (id) DO UPDATE SET
               name=EXCLUDED.name,role=EXCLUDED.role,status=EXCLUDED.status,
               monthly_cost_minor=EXCLUDED.monthly_cost_minor,start_epoch=EXCLUDED.start_epoch,updated_at=now()
             WHERE employees.company_id=EXCLUDED.company_id",
            &[
                &employee.id,&company_uuid,&employee.name,&employee.role,
                &format!("{:?}",employee.status),&employee.monthly_cost_minor.to_string(),&employee.start_epoch
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_contract(
        &self,
        company_id: &str,
        contract: &Contract,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        contract.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO contracts
             (id,company_id,counterparty,contract_type,status,value_minor,start_epoch,end_epoch)
             VALUES ($1,$2,$3,$4,$5,$6::numeric,$7,$8)
             ON CONFLICT (id) DO UPDATE SET
               counterparty=EXCLUDED.counterparty,contract_type=EXCLUDED.contract_type,status=EXCLUDED.status,
               value_minor=EXCLUDED.value_minor,start_epoch=EXCLUDED.start_epoch,end_epoch=EXCLUDED.end_epoch,updated_at=now()
             WHERE contracts.company_id=EXCLUDED.company_id",
            &[
                &contract.id,&company_uuid,&contract.counterparty,&contract.contract_type,
                &format!("{:?}",contract.status),&contract.value_minor.to_string(),&contract.start_epoch,&contract.end_epoch
            ],
        ).await?;
        Ok(())
    }

    pub async fn save_task(
        &self,
        company_id: &str,
        task: &Task,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        task.validate().map_err(|e| e.to_string())?;
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO tasks (id,company_id,title,status,priority,owner_agent,creator_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7)
             ON CONFLICT (id) DO UPDATE SET
               title=EXCLUDED.title,status=EXCLUDED.status,priority=EXCLUDED.priority,
               owner_agent=EXCLUDED.owner_agent,creator_id=EXCLUDED.creator_id,updated_at=now()
             WHERE tasks.company_id=EXCLUDED.company_id",
            &[
                &task.id,&company_uuid,&task.title,&format!("{:?}",task.status),
                &(task.priority as i16),&task.owner_agent,&task.creator_id
            ],
        ).await?;
        Ok(())
    }

    pub async fn record_cash_revenue(
        &self,
        company_id: &str,
        amount_minor: i128,
        source: &str,
        idempotency_key: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        self.record_cash_flow(company_id, amount_minor, source, idempotency_key, true).await
    }

    pub async fn record_cash_expense(
        &self,
        company_id: &str,
        amount_minor: i128,
        source: &str,
        idempotency_key: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        self.record_cash_flow(company_id, amount_minor, source, idempotency_key, false).await
    }

    async fn record_cash_flow(
        &self,
        company_id: &str,
        amount_minor: i128,
        source: &str,
        idempotency_key: &str,
        revenue: bool,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        if amount_minor <= 0 {
            return Err("cash flow amount must be positive".into());
        }
        if source.trim().is_empty() || idempotency_key.trim().is_empty() {
            return Err("cash flow source and idempotency key are required".into());
        }

        let company_uuid = Uuid::parse_str(company_id)?;
        let mut client = self.client.lock().await;
        let tx = client.transaction().await?;

        let row = tx.query_one(
            "SELECT base_currency FROM companies WHERE id=$1 FOR UPDATE",
            &[&company_uuid],
        ).await?;
        let currency: String = row.get::<_, String>(0);

        let cash_id: Uuid = tx.query_one(
            "SELECT id FROM ledger_accounts WHERE company_id=$1 AND code='1000'",
            &[&company_uuid],
        ).await?.get(0);
        let other_code = if revenue { "4000" } else { "5000" };
        let other_id: Uuid = tx.query_one(
            "SELECT id FROM ledger_accounts WHERE company_id=$1 AND code=$2",
            &[&company_uuid, &other_code],
        ).await?.get(0);

        let transaction_id = Uuid::new_v4();
        let transaction = LedgerTransaction {
            id: transaction_id.to_string(),
            description: source.to_owned(),
            entries: if revenue {
                vec![
                    LedgerEntry {
                        account_id: cash_id.to_string(),
                        debit_minor: amount_minor,
                        credit_minor: 0,
                        currency: currency.clone(),
                    },
                    LedgerEntry {
                        account_id: other_id.to_string(),
                        debit_minor: 0,
                        credit_minor: amount_minor,
                        currency: currency.clone(),
                    },
                ]
            } else {
                vec![
                    LedgerEntry {
                        account_id: other_id.to_string(),
                        debit_minor: amount_minor,
                        credit_minor: 0,
                        currency: currency.clone(),
                    },
                    LedgerEntry {
                        account_id: cash_id.to_string(),
                        debit_minor: 0,
                        credit_minor: amount_minor,
                        currency: currency.clone(),
                    },
                ]
            },
        };
        validate_balanced_transaction(&transaction).map_err(|e| format!("cash-flow ledger validation failed: {e}"))?;

        let ledger_key = if revenue {
            format!("REVENUE:{idempotency_key}")
        } else {
            format!("EXPENSE:{idempotency_key}")
        };

        let inserted = tx.execute(
            "INSERT INTO ledger_transactions (id, company_id, description, idempotency_key)
             VALUES ($1,$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[&transaction_id, &company_uuid, &source, &ledger_key],
        ).await?;

        if inserted == 0 {
            let row = tx.query_one(
                "SELECT id FROM ledger_transactions WHERE company_id=$1 AND idempotency_key=$2",
                &[&company_uuid, &ledger_key],
            ).await?;
            tx.rollback().await?;
            return Ok(row.get(0));
        }

        for entry in &transaction.entries {
            let account_id = Uuid::parse_str(&entry.account_id)?;
            tx.execute(
                "INSERT INTO ledger_entries (transaction_id, account_id, debit_minor, credit_minor, currency)
                 VALUES ($1,$2,$3::numeric,$4::numeric,$5)",
                &[
                    &transaction_id,
                    &account_id,
                    &entry.debit_minor.to_string(),
                    &entry.credit_minor.to_string(),
                    &entry.currency,
                ],
            ).await?;
        }

        let row = tx.query_one(
            "SELECT state FROM company_state_snapshots WHERE company_id=$1 FOR UPDATE",
            &[&company_uuid],
        ).await?;
        let state: Value = row.get(0);
        let mut snapshot: CompanySnapshot = serde_json::from_value(state)?;
        ExecutionEngine::validate_snapshot(&snapshot)
            .map_err(|e| format!("snapshot validation failed: {e}"))?;

        if revenue {
            snapshot.cash_minor = snapshot.cash_minor.checked_add(amount_minor).ok_or("cash overflow")?;
            snapshot.revenue_minor = snapshot.revenue_minor.checked_add(amount_minor).ok_or("revenue overflow")?;
            snapshot.assets_minor = snapshot.assets_minor.checked_add(amount_minor).ok_or("assets overflow")?;
        } else {
            snapshot.cash_minor = snapshot.cash_minor.checked_sub(amount_minor).ok_or("cash underflow")?;
            snapshot.expenses_minor = snapshot.expenses_minor.checked_add(amount_minor).ok_or("expense overflow")?;
            snapshot.assets_minor = snapshot.assets_minor.checked_sub(amount_minor).ok_or("assets underflow")?;
        }

        let next_state = serde_json::to_value(&snapshot)?;
        tx.execute(
            "UPDATE company_state_snapshots
             SET state=$2, updated_at=now(), revision=revision+1
             WHERE company_id=$1",
            &[&company_uuid, &next_state],
        ).await?;

        tx.commit().await?;
        Ok(transaction_id)
    }

    pub async fn record_affiliate_click(
        &self,
        company_id: &str,
        click: &ClickTouch,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        if click.click_id.trim().is_empty() || click.product_id.trim().is_empty() {
            return Err("affiliate click id and product id are required".into());
        }
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO affiliate_clicks
             (company_id,click_id,content_id,creator_id,product_id,occurred_at_epoch)
             VALUES ($1,$2,$3,$4,$5,$6)
             ON CONFLICT (company_id,click_id) DO NOTHING",
            &[&company_uuid,&click.click_id,&click.content_id,&click.creator_id,&click.product_id,&click.occurred_at_epoch],
        ).await?;
        Ok(())
    }

    pub async fn record_affiliate_order(
        &self,
        company_id: &str,
        order: &OrderEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        if order.order_id.trim().is_empty() || order.product_id.trim().is_empty() {
            return Err("affiliate order id and product id are required".into());
        }
        if order.gross_sales_minor < 0 || order.commission_minor < 0 {
            return Err("affiliate order amounts cannot be negative".into());
        }
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO affiliate_order_events
             (company_id,order_id,click_id,product_id,gross_sales_minor,commission_minor,status,occurred_at_epoch)
             VALUES ($1,$2,$3,$4,$5::numeric,$6::numeric,$7,$8)
             ON CONFLICT (company_id,order_id,status) DO NOTHING",
            &[
                &company_uuid,
                &order.order_id,
                &order.click_id,
                &order.product_id,
                &order.gross_sales_minor.to_string(),
                &order.commission_minor.to_string(),
                &format!("{:?}", order.status),
                &order.occurred_at_epoch,
            ],
        ).await?;
        Ok(())
    }

    pub async fn rebuild_affiliate_attribution(
        &self,
        company_id: &str,
        model: AttributionModel,
        window_secs: i64,
        since_epoch: i64,
    ) -> Result<AttributionResult, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let since = since_epoch.saturating_sub(window_secs.max(1));
        let mut client = self.client.lock().await;
        let click_rows = client.query(
            "SELECT click_id,content_id,creator_id,product_id,occurred_at_epoch
             FROM affiliate_clicks
             WHERE company_id=$1 AND occurred_at_epoch >= $2
             ORDER BY occurred_at_epoch, click_id",
            &[&company_uuid,&since],
        ).await?;
        let order_rows = client.query(
            "SELECT order_id,click_id,product_id,gross_sales_minor::text,commission_minor::text,status,occurred_at_epoch
             FROM affiliate_order_events
             WHERE company_id=$1 AND occurred_at_epoch >= $2
             ORDER BY occurred_at_epoch, id",
            &[&company_uuid,&since],
        ).await?;

        let mut clicks = Vec::with_capacity(click_rows.len());
        for row in click_rows {
            clicks.push(ClickTouch {
                click_id: row.get(0),
                content_id: row.get(1),
                creator_id: row.get(2),
                product_id: row.get(3),
                occurred_at_epoch: row.get(4),
            });
        }

        let mut orders = Vec::with_capacity(order_rows.len());
        for row in order_rows {
            orders.push(OrderEvent {
                order_id: row.get(0),
                click_id: row.get(1),
                product_id: row.get(2),
                gross_sales_minor: row.get::<_, String>(3).parse()?,
                commission_minor: row.get::<_, String>(4).parse()?,
                status: parse_order_status(row.get::<_, String>(5).as_str())?,
                occurred_at_epoch: row.get(6),
            });
        }

        let result = attribute(&clicks, &orders, model, window_secs)
            .map_err(|e| e.to_string())?;

        let mut tx = client.transaction().await?;
        for order in &result.orders {
            tx.execute(
                "INSERT INTO affiliate_attributions
                 (company_id,order_id,content_id,creator_id,product_id,gross_sales_minor,commission_minor,
                  attribution_confidence_bps,attribution_reason)
                 VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8,$9)
                 ON CONFLICT (company_id,order_id) DO UPDATE SET
                   content_id=EXCLUDED.content_id,creator_id=EXCLUDED.creator_id,
                   product_id=EXCLUDED.product_id,gross_sales_minor=EXCLUDED.gross_sales_minor,
                   commission_minor=EXCLUDED.commission_minor,
                   attribution_confidence_bps=EXCLUDED.attribution_confidence_bps,
                   attribution_reason=EXCLUDED.attribution_reason",
                &[
                    &company_uuid,
                    &order.order_id,
                    &order.content_id,
                    &order.creator_id,
                    &order.product_id,
                    &order.gross_sales_minor.to_string(),
                    &order.commission_minor.to_string(),
                    &(order.attribution_confidence_bps as i32),
                    &order.attribution_reason,
                ],
            ).await?;
        }
        tx.commit().await?;
        Ok(result)
    }

    pub async fn record_affiliate_search(
        &self,
        company_id: &str,
        query: &ProductSearchQuery,
        result: &AffiliateSearchResult,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let search_id = Uuid::new_v4();
        let query_json = serde_json::to_value(query)?;
        let result_json = serde_json::to_value(result)?;
        let client = self.client.lock().await;
        client.execute(
            "INSERT INTO affiliate_searches (id, company_id, query_json, result_json)
             VALUES ($1,$2,$3,$4)",
            &[&search_id, &company_uuid, &query_json, &result_json],
        ).await?;
        Ok(search_id)
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
        if cycle_id.trim().is_empty() || cycle_id.len() > 200 {
            return Err("cycle id must be non-empty and <= 200 characters".into());
        }
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
                    let previous_snapshot = tx
            .query_opt(
                "SELECT state FROM company_state_snapshots WHERE company_id=$1 FOR UPDATE",
                &[&company_id],
            )
            .await?
            .map(|row| row.get::<_, Value>(0));

        if let Some(previous_state) = previous_snapshot {
            let previous: CompanySnapshot = serde_json::from_value(previous_state)?;
            Self::post_economic_deltas(&tx, &company_id, &cycle_uuid, &previous, snapshot).await?;
        }

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

        let previous_snapshot = tx
            .query_opt(
                "SELECT state FROM company_state_snapshots WHERE company_id=$1 FOR UPDATE",
                &[&company_id],
            )
            .await?
            .map(|row| row.get::<_, Value>(0));

        if let Some(previous_state) = previous_snapshot {
            let previous: CompanySnapshot = serde_json::from_value(previous_state)?;
            Self::post_economic_deltas(&tx, &company_id, &cycle_uuid, &previous, snapshot).await?;
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

    async fn post_economic_deltas(
        tx: &tokio_postgres::Transaction<'_>,
        company_id: &Uuid,
        cycle_id: &Uuid,
        previous: &CompanySnapshot,
        next: &CompanySnapshot,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let revenue_delta = next.revenue_minor
            .checked_sub(previous.revenue_minor)
            .ok_or("revenue delta overflow")?;
        let expense_delta = next.expenses_minor
            .checked_sub(previous.expenses_minor)
            .ok_or("expense delta overflow")?;
        let cash_delta = next.cash_minor
            .checked_sub(previous.cash_minor)
            .ok_or("cash delta overflow")?;

        if revenue_delta < 0 || expense_delta < 0 {
            return Err("revenue and expenses cannot decrease between committed states".into());
        }
        if cash_delta != revenue_delta.saturating_sub(expense_delta) {
            return Err("cash delta does not reconcile with revenue and expense deltas".into());
        }

        if revenue_delta > 0 {
            Self::post_ledger_tx(
                tx,
                company_id,
                cycle_id,
                "REVENUE",
                revenue_delta,
                true,
                "agent-cycle revenue delta",
            ).await?;
        }
        if expense_delta > 0 {
            Self::post_ledger_tx(
                tx,
                company_id,
                cycle_id,
                "EXPENSE",
                expense_delta,
                false,
                "agent-cycle expense delta",
            ).await?;
        }
        Ok(())
    }

    async fn post_ledger_tx(
        tx: &tokio_postgres::Transaction<'_>,
        company_id: &Uuid,
        cycle_id: &Uuid,
        suffix: &str,
        amount_minor: i128,
        revenue: bool,
        description: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if amount_minor <= 0 {
            return Ok(());
        }

        let currency: String = tx.query_one(
            "SELECT base_currency FROM companies WHERE id=$1",
            &[company_id],
        ).await?.get(0);

        let cash_id: Uuid = tx.query_one(
            "SELECT id FROM ledger_accounts WHERE company_id=$1 AND code='1000'",
            &[company_id],
        ).await?.get(0);
        let other_code = if revenue { "4000" } else { "5000" };
        let other_id: Uuid = tx.query_one(
            "SELECT id FROM ledger_accounts WHERE company_id=$1 AND code=$2",
            &[company_id, &other_code],
        ).await?.get(0);

        let transaction_id = Uuid::new_v4();
        let ledger_key = format!("AGENT_CYCLE:{cycle_id}:{suffix}");
        let (debit_account, credit_account) = if revenue { (cash_id, other_id) } else { (other_id, cash_id) };

        let transaction = LedgerTransaction {
            id: transaction_id.to_string(),
            description: description.into(),
            entries: vec![
                LedgerEntry {
                    account_id: debit_account.to_string(),
                    debit_minor: amount_minor,
                    credit_minor: 0,
                    currency: currency.clone(),
                },
                LedgerEntry {
                    account_id: credit_account.to_string(),
                    debit_minor: 0,
                    credit_minor: amount_minor,
                    currency: currency.clone(),
                },
            ],
        };
        validate_balanced_transaction(&transaction).map_err(|e| format!("cycle ledger validation failed: {e}"))?;

        tx.execute(
            "INSERT INTO ledger_transactions (id, company_id, description, idempotency_key)
             VALUES ($1,$2,$3,$4)
             ON CONFLICT (company_id,idempotency_key) DO NOTHING",
            &[&transaction_id, company_id, &transaction.description, &ledger_key],
        ).await?;

        for entry in &transaction.entries {
            let account_id = Uuid::parse_str(&entry.account_id)?;
            tx.execute(
                "INSERT INTO ledger_entries (transaction_id, account_id, debit_minor, credit_minor, currency)
                 VALUES ($1,$2,$3::numeric,$4::numeric,$5)",
                &[
                    &transaction_id,
                    &account_id,
                    &entry.debit_minor.to_string(),
                    &entry.credit_minor.to_string(),
                    &entry.currency,
                ],
            ).await?;
        }

        Ok(())
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

    pub async fn ensure_cycle_schedule(
        &self,
        company_id: &str,
        interval_seconds: i64,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let interval_seconds = interval_seconds.clamp(15, 86_400);
        let schedule_id = Uuid::new_v4();
        let client = self.client.lock().await;
        let row = client.query_one(
            "INSERT INTO company_schedules (id, company_id, job_type, interval_seconds, next_run_at)
             VALUES ($1,$2,'AGENT_CYCLE',$3,now())
             ON CONFLICT (company_id, job_type) DO UPDATE
             SET interval_seconds=EXCLUDED.interval_seconds,
                 enabled=true,
                 updated_at=now()
             RETURNING id",
            &[&schedule_id, &company_uuid, &interval_seconds],
        ).await?;
        Ok(row.get(0))
    }

    pub async fn claim_due_cycle(&self) -> Result<Option<Uuid>, tokio_postgres::Error> {
        let client = self.client.lock().await;
        let row = client.query_opt(
            "WITH due AS (
                SELECT id
                FROM company_schedules
                WHERE enabled=true
                  AND job_type='AGENT_CYCLE'
                  AND next_run_at <= now()
                ORDER BY next_run_at
                FOR UPDATE SKIP LOCKED
                LIMIT 1
             )
             UPDATE company_schedules s
                SET next_run_at = now() + (s.interval_seconds * interval '1 second'),
                    updated_at = now()
              FROM due
              WHERE s.id = due.id
              RETURNING s.company_id",
            &[],
        ).await?;
        Ok(row.map(|r| r.get(0)))
    }

    pub async fn retry_cycle_schedule(
        &self,
        company_id: &str,
        delay_seconds: i64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let delay = delay_seconds.clamp(5, 3_600);
        let client = self.client.lock().await;
        Ok(client.execute(
            "UPDATE company_schedules
             SET next_run_at=now() + ($2 * interval '1 second'), updated_at=now()
             WHERE company_id=$1 AND job_type='AGENT_CYCLE' AND enabled=true",
            &[&company_uuid, &delay],
        ).await? == 1)
    }

    pub async fn set_schedule_enabled(
        &self,
        company_id: &str,
        enabled: bool,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        Ok(client.execute(
            "UPDATE company_schedules SET enabled=$2, updated_at=now()
             WHERE company_id=$1 AND job_type='AGENT_CYCLE'",
            &[&company_uuid, &enabled],
        ).await? == 1)
    }

    pub async fn load_cycle_results(
        &self,
        cycle_id: &str,
    ) -> Result<Option<Vec<AgentRunResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let cycle_uuid = Uuid::parse_str(cycle_id)?;
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT proposal_json
             FROM decision_journal
             WHERE cycle_id=$1
             ORDER BY id",
            &[&cycle_uuid],
        ).await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let value: Value = row.get(0);
            results.push(serde_json::from_value(value)?);
        }
        Ok(Some(results))
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

fn parse_order_status(value: &str) -> Result<OrderStatus, Box<dyn std::error::Error + Send + Sync>> {
    match value {
        "Pending" => Ok(OrderStatus::Pending),
        "Confirmed" => Ok(OrderStatus::Confirmed),
        "Refunded" => Ok(OrderStatus::Refunded),
        _ => Err(format!("invalid affiliate order status: {value}").into()),
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
mod tests;    pub async fn claim_due_cycle_for(
        &self,
        company_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        let row = client.query_opt(
            "WITH due AS (
                SELECT id
                FROM company_schedules
                WHERE enabled=true
                  AND company_id=$1
                  AND job_type='AGENT_CYCLE'
                  AND next_run_at <= now()
                ORDER BY next_run_at
                FOR UPDATE SKIP LOCKED
                LIMIT 1
             )
             UPDATE company_schedules s
                SET next_run_at = now() + (s.interval_seconds * interval '1 second'),
                    updated_at = now()
              FROM due
              WHERE s.id = due.id
              RETURNING s.id",
            &[&company_uuid],
        ).await?;
        Ok(row.is_some())
    }

    pub async fn set_schedule_enabled(
        &self,
        company_id: &str,
        enabled: bool,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let company_uuid = Uuid::parse_str(company_id)?;
        let client = self.client.lock().await;
        Ok(client.execute(
            "UPDATE company_schedules SET enabled=$2, updated_at=now()
             WHERE company_id=$1 AND job_type='AGENT_CYCLE'",
            &[&company_uuid, &enabled],
        ).await? == 1)
    }

    pub async fn load_cycle_results(
        &self,
        cycle_id: &str,
    ) -> Result<Option<Vec<AgentRunResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let cycle_uuid = Uuid::parse_str(cycle_id)?;
        let client = self.client.lock().await;
        let rows = client.query(
            "SELECT proposal_json
             FROM decision_journal
             WHERE cycle_id=$1
             ORDER BY id",
            &[&cycle_uuid],
        ).await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let value: Value = row.get(0);
            results.push(serde_json::from_value(value)?);
        }
        Ok(Some(results))
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
