#![forbid(unsafe_code)]

use agent_runtime::{model::MockModel, types::CompanySnapshot, AgentRuntime};
use company_execution::{execute_approved_results, ExecutionPolicy, ExecutionStatus};
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
    pub free_cash_flow_minor: i128,
    pub minimum_cash_minor: i128,
    pub minimum_runway_days: i64,
    pub peak_liabilities_minor: i128,
    pub payroll_accrued_minor: i128,
    pub payroll_paid_minor: i128,
    pub business_unit_count: usize,
    pub bankruptcy_day: Option<u32>,
    pub decision_cycles: u64,
    pub violations: Vec<String>,
    pub survived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub runs: u32,
    pub seed: u64,
    pub seed_stride: u64,
    pub simulation: SimConfig,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            runs: 30,
            seed: 42,
            seed_stride: 1,
            simulation: SimConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StatisticSummary {
    pub samples: usize,
    pub mean: f64,
    pub stddev: f64,
    pub ci95_low: f64,
    pub ci95_high: f64,
    pub p05: f64,
    pub median: f64,
    pub p95: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationEvaluation {
    pub runs: u32,
    pub seed: u64,
    pub seed_stride: u64,
    pub simulation_days: u32,
    pub ending_cash_minor: StatisticSummary,
    pub revenue_minor: StatisticSummary,
    pub expenses_minor: StatisticSummary,
    pub free_cash_flow_minor: StatisticSummary,
    pub minimum_cash_minor: StatisticSummary,
    pub minimum_runway_days: StatisticSummary,
    pub survival_rate_bps: i64,
    pub bankruptcy_rate_bps: i64,
    pub total_violations: usize,
    pub truth_boundary: &'static str,
}

fn t_critical_95(df: usize) -> f64 {
    match df {
        1 => 12.706,
        2 => 4.303,
        3 => 3.182,
        4 => 2.776,
        5 => 2.571,
        6 => 2.447,
        7 => 2.365,
        8 => 2.306,
        9 => 2.262,
        10 => 2.228,
        11 => 2.201,
        12 => 2.179,
        13 => 2.160,
        14 => 2.145,
        15 => 2.131,
        16 => 2.120,
        17 => 2.110,
        18 => 2.101,
        19 => 2.093,
        20 => 2.086,
        21 => 2.080,
        22 => 2.074,
        23 => 2.069,
        24 => 2.064,
        25 => 2.060,
        26 => 2.056,
        27 => 2.052,
        28 => 2.048,
        29 => 2.045,
        _ => 1.960,
    }
}

fn quantile(sorted: &[f64], probability: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let position = probability.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] + (sorted[upper] - sorted[lower]) * weight
    }
}

fn summarize(values: &[f64]) -> StatisticSummary {
    if values.is_empty() {
        return StatisticSummary {
            samples: 0,
            mean: 0.0,
            stddev: 0.0,
            ci95_low: 0.0,
            ci95_high: 0.0,
            p05: 0.0,
            median: 0.0,
            p95: 0.0,
        };
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = if values.len() > 1 {
        values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (values.len() - 1) as f64
    } else {
        0.0
    };
    let stddev = variance.sqrt();
    let standard_error = if values.len() > 1 {
        stddev / (values.len() as f64).sqrt()
    } else {
        0.0
    };
    let margin = t_critical_95(values.len().saturating_sub(1)) * standard_error;
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);

    StatisticSummary {
        samples: values.len(),
        mean,
        stddev,
        ci95_low: mean - margin,
        ci95_high: mean + margin,
        p05: quantile(&sorted, 0.05),
        median: quantile(&sorted, 0.50),
        p95: quantile(&sorted, 0.95),
    }
}

pub async fn evaluate(config: EvaluationConfig) -> Result<SimulationEvaluation, String> {
    if !(1..=500).contains(&config.runs) {
        return Err("evaluation runs must be between 1 and 500".into());
    }
    if config.seed_stride == 0 {
        return Err("evaluation seed_stride must be >= 1".into());
    }

    let mut ending_cash = Vec::with_capacity(config.runs as usize);
    let mut revenue = Vec::with_capacity(config.runs as usize);
    let mut expenses = Vec::with_capacity(config.runs as usize);
    let mut free_cash_flow = Vec::with_capacity(config.runs as usize);
    let mut minimum_cash = Vec::with_capacity(config.runs as usize);
    let mut minimum_runway = Vec::with_capacity(config.runs as usize);
    let mut survived = 0_u32;
    let mut total_violations = 0_usize;

    for index in 0..config.runs {
        let offset = (index as u64)
            .checked_mul(config.seed_stride)
            .ok_or("evaluation seed overflow")?;
        let seed = config.seed.checked_add(offset).ok_or("evaluation seed overflow")?;
        let mut simulation = config.simulation.clone();
        simulation.seed = seed;

        let result = run(simulation).await;
        ending_cash.push(result.ending_cash_minor as f64);
        revenue.push(result.revenue_minor as f64);
        expenses.push(result.expenses_minor as f64);
        free_cash_flow.push(result.free_cash_flow_minor as f64);
        minimum_cash.push(result.minimum_cash_minor as f64);
        minimum_runway.push(result.minimum_runway_days as f64);
        survived += u32::from(result.survived);
        total_violations += result.violations.len();
    }

    let runs = config.runs as i64;
    let survival_rate_bps = ((survived as i64) * 10_000) / runs;
    let bankruptcy_rate_bps = 10_000 - survival_rate_bps;

    Ok(SimulationEvaluation {
        runs: config.runs,
        seed: config.seed,
        seed_stride: config.seed_stride,
        simulation_days: config.simulation.days,
        ending_cash_minor: summarize(&ending_cash),
        revenue_minor: summarize(&revenue),
        expenses_minor: summarize(&expenses),
        free_cash_flow_minor: summarize(&free_cash_flow),
        minimum_cash_minor: summarize(&minimum_cash),
        minimum_runway_days: summarize(&minimum_runway),
        survival_rate_bps,
        bankruptcy_rate_bps,
        total_violations,
        truth_boundary: "Confidence intervals describe the simulated multi-seed distribution of this model; they are not real-world confidence intervals or revenue forecasts.",
    })
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
    let mut minimum_cash = cash;
    let mut bankruptcy_day = None;
    let mut violations = Vec::new();
    let mut decision_cycles = 0_u64;
    let employees = vec![
        company_organization::Employee {
            id: "sim-ceo".into(),
            name: "CEO".into(),
            role: "executive".into(),
            monthly_cost_minor: 300,
            currency: config.currency.clone(),
            status: company_organization::EmployeeStatus::Active,
        },
        company_organization::Employee {
            id: "sim-operator".into(),
            name: "Operator".into(),
            role: "operations".into(),
            monthly_cost_minor: 240,
            currency: config.currency.clone(),
            status: company_organization::EmployeeStatus::Active,
        },
        company_organization::Employee {
            id: "sim-editor".into(),
            name: "Editor".into(),
            role: "content".into(),
            monthly_cost_minor: 210,
            currency: config.currency.clone(),
            status: company_organization::EmployeeStatus::Active,
        },
    ];
    let business_units = vec![
        ("owned-media", 4_000_u32, 2_500_u32, 2_000_u32),
        ("affiliate", 2_500_u32, 1_400_u32, 1_000_u32),
        ("services", 1_500_u32, 900_u32, 700_u32),
    ];
    let mut liabilities = 0_i128;
    let mut peak_liabilities = 0_i128;
    let mut payroll_accrued = 0_i128;
    let mut payroll_paid = 0_i128;
    let mut minimum_runway_days = i64::MAX;

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

        let daily_fixed = 300_i128;
        let content_units = capacity.min(12) as i128;
        let content_cost = content_units * 35;

        let content_revenue = ((audience as i128)
            .saturating_mul(conversion_bps as i128)
            .saturating_mul(content_efficiency_bps as i128))
            / 100_000_000;

        let sponsor_revenue = if rng.pct(150) { 250_i128 } else { 0 };
        let affiliate_revenue = content_revenue / 2;

        let units = vec![
            company_organization::BusinessUnit {
                id: business_units[0].into(),
                name: "Owned Media".into(),
                currency: config.currency.clone(),
                cash_minor: cash.max(0),
                revenue_minor: content_revenue.max(0),
                variable_cost_minor: content_cost * 4 / 10,
                fixed_cost_minor: daily_fixed / 2,
                budget_minor: 0,
                lifecycle: company_organization::BusinessUnitLifecycle::Growing,
            },
            company_organization::BusinessUnit {
                id: business_units[1].into(),
                name: "Affiliate Commerce".into(),
                currency: config.currency.clone(),
                cash_minor: cash.max(0),
                revenue_minor: affiliate_revenue.max(0),
                variable_cost_minor: affiliate_revenue.max(0) / 8,
                fixed_cost_minor: daily_fixed / 4,
                budget_minor: 0,
                lifecycle: company_organization::BusinessUnitLifecycle::Growing,
            },
            company_organization::BusinessUnit {
                id: business_units[2].into(),
                name: "Services".into(),
                currency: config.currency.clone(),
                cash_minor: cash.max(0),
                revenue_minor: sponsor_revenue.max(0),
                variable_cost_minor: sponsor_revenue.max(0) * 3 / 10,
                fixed_cost_minor: daily_fixed / 4,
                budget_minor: 0,
                lifecycle: if sponsor_revenue > 0 {
                    company_organization::BusinessUnitLifecycle::Growing
                } else {
                    company_organization::BusinessUnitLifecycle::Testing
                },
            },
        ];
        let portfolio = company_organization::summarize_portfolio(&units);
        let (day_revenue, day_business_cost) = match portfolio {
            Ok(metrics) => (metrics.revenue_minor, metrics.total_cost_minor),
            Err(error) => {
                violations.push(format!("day {day}: business-unit economics error: {error}"));
                (0, 0)
            }
        };
        let daily_payroll = employees
            .iter()
            .map(|employee| employee.monthly_cost_minor / 30)
            .sum::<i128>();
        liabilities = liabilities.saturating_add(daily_payroll.max(0));
        payroll_accrued = payroll_accrued.saturating_add(daily_payroll.max(0));
        peak_liabilities = peak_liabilities.max(liabilities);

        let day_expense = day_business_cost + content_cost * 6 / 10 + daily_payroll;

        let payroll_pay_threshold =
            cash.saturating_sub(config.initial_cash_minor.saturating_mul(config.reserve_ratio_bps.min(9_000) as i128) / 10_000);
        if payroll_pay_threshold >= liabilities && liabilities > 0 && day % 7 == 0 {
            let payment = liabilities.min(payroll_pay_threshold);
            liabilities = liabilities.saturating_sub(payment);
            payroll_paid = payroll_paid.saturating_add(payment);
            cash = cash.saturating_sub(payment);
        }

        revenue = revenue.saturating_add(day_revenue.max(0));
        expenses = expenses.saturating_add(day_expense.max(0));
        cash = cash.saturating_add(day_revenue).saturating_sub(day_expense);
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
        let economic = CompanyState {
            company_id: "simulation".into(),
            cash_minor: cash,
            revenue_minor: revenue,
            expenses_minor: expenses,
            liabilities_minor: liabilities.max(0),
            assets_minor: cash,
            runway_days: 0,
            status: CompanyStatus::Active,
        }
        .refresh_status_from_runway(daily_burn);

        let experiment_budget = cash
            .saturating_mul((10_000_u32 - config.reserve_ratio_bps.min(9_000)) as i128)
            / 10_000;
        let snapshot = CompanySnapshot {
            company_id: "simulation".into(),
            cash_minor: cash,
            revenue_minor: revenue,
            expenses_minor: expenses,
            liabilities_minor: liabilities.max(0),
            assets_minor: cash.saturating_add(liabilities.max(0)),
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
            hiring_need: if backlog > capacity * 2 { 1 } else { 0 },
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
                cash = batch.snapshot.cash_minor;
                expenses = batch.snapshot.expenses_minor;
                backlog = batch.snapshot.backlog;
            }
            Err(error) => violations.push(format!("day {day}: execution engine error: {error}")),
        }

        minimum_cash = minimum_cash.min(cash);
        minimum_runway_days = minimum_runway_days.min(economic.runway_days);
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
        free_cash_flow_minor: revenue.saturating_sub(expenses),
        minimum_cash_minor: minimum_cash,
        minimum_runway_days: if minimum_runway_days == i64::MAX { 0 } else { minimum_runway_days },
        peak_liabilities_minor: peak_liabilities,
        payroll_accrued_minor: payroll_accrued,
        payroll_paid_minor: payroll_paid,
        business_unit_count: business_units.len(),
        bankruptcy_day,
        decision_cycles,
        violations,
        survived: bankruptcy_day.is_none(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn summary_reports_mean_spread_and_confidence_interval() {
        let stats = summarize(&[1.0, 2.0, 3.0]);
        assert_eq!(stats.samples, 3);
        assert_eq!(stats.mean, 2.0);
        assert!(stats.stddev > 0.0);
        assert!(stats.ci95_low < stats.mean);
        assert!(stats.ci95_high > stats.mean);
        assert_eq!(stats.median, 2.0);
    }

    #[tokio::test]
    async fn multi_seed_evaluation_is_deterministic() {
        let config = EvaluationConfig {
            runs: 4,
            seed: 7,
            seed_stride: 3,
            simulation: SimConfig {
                days: 20,
                ..SimConfig::default()
            },
        };
        let a = evaluate(config.clone()).await.unwrap();
        let b = evaluate(config).await.unwrap();
        assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
        assert_eq!(a.runs, 4);
        assert_eq!(a.ending_cash_minor.samples, 4);
        assert_eq!(
            a.survival_rate_bps + a.bankruptcy_rate_bps,
            10_000
        );
    }

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
        assert_eq!(result.business_unit_count, 3);
        assert!(result.payroll_accrued_minor > 0);
        assert!(result.peak_liabilities_minor > 0);
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
        assert!(result.payroll_accrued_minor > 0);
        assert!(result.minimum_runway_days >= 0);
        assert!(result.violations.is_empty(), "{:?}", result.violations);
    }
}
