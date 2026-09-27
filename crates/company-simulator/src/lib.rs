#![forbid(unsafe_code)]

use agent_runtime::{model::MockModel, types::CompanySnapshot, AgentRuntime};
use company_execution::{execute_approved_results, ExecutionPolicy, ExecutionStatus};
use business_economics::{
    BusinessUnit, BusinessUnitKind, BusinessUnitStatus, CompanyPortfolio, PayrollObligation,
};
use economic_core::{CompanyState, CompanyStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub initial_cash_minor: i128,
    pub currency: String,
    pub days: u32,
    pub seed: u64,
    pub reserve_ratio_bps: u32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            initial_cash_minor: 100_000,
            currency: "USD".into(),
            days: 180,
            seed: 42,
            reserve_ratio_bps: 2_500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub days_simulated: u32,
    pub ending_cash_minor: i128,
    pub revenue_minor: i128,
    pub expenses_minor: i128,
    pub liabilities_minor: i128,
    pub free_cash_flow_minor: i128,
    pub minimum_cash_minor: i128,
    pub bankruptcy_day: Option<u32>,
    pub decision_cycles: u64,
    pub violations: Vec<String>,
    pub survived: bool,
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn pct(&mut self, bps: u32) -> bool {
        self.next() % 10_000 < bps as u64
    }
}

pub async fn run(config: SimConfig) -> SimulationResult {
    let runtime = AgentRuntime::new_with_concurrency(Box::new(MockModel), 2);
    let mut rng = Rng::new(config.seed);

    let mut cash = config.initial_cash_minor;
    let mut revenue = 0_i128;
    let mut expenses = 0_i128;
    let mut liabilities = 0_i128;
    let mut minimum_cash = cash;

    let mut portfolio = CompanyPortfolio {
        units: vec![
            BusinessUnit {
                id: "creator-media".into(),
                name: "Creator Media".into(),
                kind: BusinessUnitKind::Creator,
                status: BusinessUnitStatus::Growing,
                cash_minor: cash.saturating_mul(70) / 100,
                revenue_minor: 0,
                direct_cost_minor: 0,
                fixed_cost_minor: 180,
                payroll: vec![PayrollObligation {
                    employee_id: "editor-1".into(),
                    amount_minor: 70,
                    due_day: 1,
                    recurrence_days: Some(1),
                    priority: 100,
                }],
                contracts: vec![],
            },
            BusinessUnit {
                id: "affiliate-commerce".into(),
                name: "Affiliate Commerce".into(),
                kind: BusinessUnitKind::AffiliateChannel,
                status: BusinessUnitStatus::Growing,
                cash_minor: cash.saturating_sub(cash.saturating_mul(70) / 100),
                revenue_minor: 0,
                direct_cost_minor: 0,
                fixed_cost_minor: 120,
                payroll: vec![PayrollObligation {
                    employee_id: "ops-1".into(),
                    amount_minor: 50,
                    due_day: 1,
                    recurrence_days: Some(1),
                    priority: 80,
                }],
                contracts: vec![],
            },
        ],
    };

    if portfolio.validate().is_err() {
        return SimulationResult {
            days_simulated: 0,
            ending_cash_minor: cash,
            revenue_minor: 0,
            expenses_minor: 0,
            liabilities_minor: 0,
            free_cash_flow_minor: 0,
            minimum_cash_minor: cash,
            bankruptcy_day: None,
            decision_cycles: 0,
            violations: vec!["initial business portfolio is invalid".into()],
            survived: false,
        };
    }
    let mut bankruptcy_day = None;
    let mut violations = Vec::new();
    let mut decision_cycles = 0_u64;

    let mut audience = 10_000_u64;
    let mut conversion_bps = 220_u32;
    let mut content_efficiency_bps = 12_000_u32;
    let mut backlog = 5_u32;
    let capacity = 10_u32;

    for day in 1..=config.days {
        if cash <= 0 {
            bankruptcy_day = Some(day);
            break;
        }

        let content_units = capacity.min(12) as i128;
        let content_cost = content_units * 35;

        let content_revenue = ((audience as i128)
            .saturating_mul(conversion_bps as i128)
            .saturating_mul(content_efficiency_bps as i128))
            / 100_000_000;

        let sponsor_revenue = if rng.pct(150) { 250_i128 } else { 0 };
        let affiliate_revenue = content_revenue / 2;
        let day_revenue = content_revenue + affiliate_revenue + sponsor_revenue;

        let portfolio_day = portfolio
            .settle_day(
                day as u64,
                &[content_revenue.max(0), affiliate_revenue.saturating_add(sponsor_revenue).max(0)],
                &[content_cost.max(0), 0],
            )
            .unwrap_or_else(|error| {
                violations.push(format!("day {day}: portfolio economics error: {error}"));
                business_economics::PortfolioDayResult {
                    day: day as u64,
                    revenue_minor: 0,
                    costs_minor: 0,
                    cash_minor: portfolio.total_cash_minor().unwrap_or(0),
                    unpaid_priority_minor: 0,
                    distressed_units: portfolio.units.len(),
                }
            });

        revenue = revenue.saturating_add(portfolio_day.revenue_minor.max(0));
        expenses = expenses.saturating_add(portfolio_day.costs_minor.max(0));
        liabilities = liabilities.saturating_add(portfolio_day.unpaid_priority_minor.max(0));
        cash = portfolio.total_cash_minor().unwrap_or(portfolio_day.cash_minor);
        minimum_cash = minimum_cash.min(cash);

        if cash <= 0 {
            bankruptcy_day = Some(day);
            break;
        }

        audience = audience.saturating_add((audience / 10_000) * 8);
        if rng.pct(800) {
            conversion_bps = conversion_bps.saturating_add(2).min(800);
        }

        if backlog > capacity {
            backlog = backlog.saturating_sub(1);
        } else {
            backlog = backlog.saturating_add(1).min(100);
        }

        let daily_burn = (expenses / day as i128).max(1);
        let mut economic = CompanyState {
            company_id: "simulation".into(),
            cash_minor: cash,
            revenue_minor: revenue,
            expenses_minor: expenses,
            liabilities_minor: liabilities,
            assets_minor: cash,
            runway_days: 0,
            status: CompanyStatus::Active,
        }
        .refresh_status_from_runway(daily_burn);

        if portfolio_day.distressed_units > 0 || portfolio_day.unpaid_priority_minor > 0 {
            economic.status = CompanyStatus::Distress;
        }

        let experiment_budget = cash
            .saturating_mul((10_000_u32 - config.reserve_ratio_bps.min(9_000)) as i128)
            / 10_000;
        let snapshot = CompanySnapshot {
            company_id: "simulation".into(),
            cash_minor: cash,
            revenue_minor: revenue,
            expenses_minor: expenses,
            liabilities_minor: liabilities,
            assets_minor: cash,
            runway_days: economic.runway_days,
            status: economic.status,
            budget_remaining_minor: experiment_budget.max(0),
            experiment_budget_minor: experiment_budget.max(0).min(1_000),
            content_cost_minor: content_cost.max(0),
            content_revenue_minor: content_revenue.max(0),
            backlog,
            capacity,
            conversion_bps,
            audience_growth_bps: 80,
            hiring_need: if backlog > capacity * 2 && cash > 2_000 { 1 } else { 0 },
        };

        let results = runtime.run_all(snapshot.clone()).await;
        decision_cycles += 1;

        let execution = execute_approved_results(
            snapshot.clone(),
            &results,
            ExecutionPolicy {
                max_spend_per_cycle_minor: 1_000,
            },
        );
        match execution {
            Ok(batch) => {
                for receipt in &batch.receipts {
                    if receipt.status == ExecutionStatus::Rejected {
                        violations.push(format!(
                            "day {day}: execution rejected for {:?}: {}",
                            receipt.action, receipt.reason
                        ));
                    }
                    if receipt.status == ExecutionStatus::Executed
                        && receipt.action == agent_runtime::types::ActionKind::CreateExperiment
                    {
                        content_efficiency_bps =
                            content_efficiency_bps.saturating_add(50).min(20_000);
                    }
                }
                let cash_delta = batch.snapshot.cash_minor.saturating_sub(snapshot.cash_minor);
                if let Err(error) = portfolio.apply_external_cash_delta(cash_delta) {
                    violations.push(format!(
                        "day {day}: portfolio cash synchronization error: {error}"
                    ));
                }
                cash = portfolio.total_cash_minor().unwrap_or(batch.snapshot.cash_minor);
                expenses = batch.snapshot.expenses_minor;
                liabilities = batch.snapshot.liabilities_minor;
                backlog = batch.snapshot.backlog;
            }
            Err(error) => violations.push(format!("day {day}: execution engine error: {error}")),
        }

        minimum_cash = minimum_cash.min(cash);
        if cash <= 0 {
            bankruptcy_day = Some(day);
            break;
        }
    }

    SimulationResult {
        days_simulated: if bankruptcy_day.is_some() {
            bankruptcy_day.unwrap_or(0)
        } else {
            config.days
        },
        ending_cash_minor: cash,
        revenue_minor: revenue,
        expenses_minor: expenses,
        liabilities_minor: liabilities,
        free_cash_flow_minor: revenue.saturating_sub(expenses),
        minimum_cash_minor: minimum_cash,
        bankruptcy_day,
        decision_cycles,
        violations,
        survived: bankruptcy_day.is_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn simulation_is_deterministic_for_same_seed() {
        let config = SimConfig {
            days: 60,
            ..SimConfig::default()
        };
        let a = run(config.clone()).await;
        let b = run(config).await;
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }

    #[tokio::test]
    async fn simulation_runs_without_paid_model_api() {
        let result = run(SimConfig {
            days: 30,
            ..SimConfig::default()
        })
        .await;
        assert_eq!(result.decision_cycles, 30);
        assert!(result.violations.is_empty(), "{:?}", result.violations);
    }

    #[tokio::test]
    async fn severe_cash_shock_does_not_create_negative_spend() {
        let result = run(SimConfig {
            initial_cash_minor: 1_000,
            days: 30,
            seed: 99,
            ..SimConfig::default()
        })
        .await;
        assert!(result.ending_cash_minor >= 0 || result.bankruptcy_day.is_some());
        assert!(result.violations.is_empty(), "{:?}", result.violations);
    }
}
