#![forbid(unsafe_code)]

use agent_runtime::{
    model::MockModel,
    types::{CompanySnapshot},
    AgentRuntime, ExecutionEngine,
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
    pub free_cash_flow_minor: i128,
    pub minimum_cash_minor: i128,
    pub bankruptcy_day: Option<u32>,
    pub decision_cycles: u64,
    pub executed_actions: u64,
    pub violations: Vec<String>,
    pub survived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloSummary {
    pub trials: u32,
    pub survival_rate_bps: u32,
    pub bankruptcy_rate_bps: u32,
    pub average_ending_cash_minor: i128,
    pub worst_ending_cash_minor: i128,
    pub best_ending_cash_minor: i128,
    pub total_violations: u64,
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

    let mut cash = config.initial_cash_minor.max(0);
    let mut revenue = 0_i128;
    let mut expenses = 0_i128;
    let mut minimum_cash = cash;
    let mut bankruptcy_day = None;
    let mut violations = Vec::new();
    let mut decision_cycles = 0_u64;
    let mut executed_actions = 0_u64;

    let mut audience = 10_000_u64;
    let mut conversion_bps = 220_u32;
    let mut content_efficiency_bps = 12_000_u32;
    let mut backlog = 5_u32;
    let mut capacity = 10_u32;

    for day in 1..=config.days {
        if cash <= 0 {
            bankruptcy_day = Some(day);
            break;
        }

        let daily_fixed = 300_i128;
        let content_units = capacity.min(12) as i128;
        let content_cost = content_units.saturating_mul(35);

        let content_revenue = audience
            .saturating_mul(conversion_bps as u64)
            .saturating_mul(content_efficiency_bps as u64)
            / 100_000_000_u64 as i128;

        let sponsor_revenue = if rng.pct(150) { 250_i128 } else { 0 };
        let affiliate_revenue = content_revenue / 2;
        let day_revenue = content_revenue + affiliate_revenue + sponsor_revenue;
        let day_expense = daily_fixed + content_cost;

        revenue = revenue.saturating_add(day_revenue.max(0));
        expenses = expenses.saturating_add(day_expense.max(0));
        cash = cash.saturating_add(day_revenue).saturating_sub(day_expense);
        minimum_cash = minimum_cash.min(cash);

        if cash <= 0 {
            bankruptcy_day = Some(day);
            break;
        }

        audience = audience.saturating_add((audience / 10_000).saturating_mul(8));
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
            liabilities_minor: 0,
            assets_minor: cash,
            runway_days: 0,
            status: CompanyStatus::Active,
        }
        .refresh_status_from_runway(daily_burn);

        let experiment_budget = cash
            .saturating_mul((10_000_u32 - config.reserve_ratio_bps.min(9_000)) as i128)
            .saturating_div(10_000);

        let mut snapshot = CompanySnapshot {
            company_id: "simulation".into(),
            cash_minor: cash,
            revenue_minor: revenue,
            expenses_minor: expenses,
            liabilities_minor: 0,
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
            hiring_need: if backlog > capacity.saturating_mul(2) { 1 } else { 0 },
        };

        let mut results = runtime.run_all(snapshot.clone()).await;
        let outcomes = ExecutionEngine::default().execute_batch(&mut snapshot, &mut results);
        decision_cycles += 1;
        executed_actions += outcomes.iter().filter(|o| o.executed).count() as u64;

        for outcome in &outcomes {
            if outcome.executed && outcome.cost_minor > snapshot.cash_minor.saturating_add(outcome.cost_minor) {
                violations.push(format!("day {day}: impossible execution cost"));
            }
        }

        cash = snapshot.cash_minor;
        expenses = snapshot.expenses_minor;
        backlog = snapshot.backlog;
        minimum_cash = minimum_cash.min(cash);
    }

    let days_simulated = if let Some(day) = bankruptcy_day { day } else { config.days };
    SimulationResult {
        days_simulated,
        ending_cash_minor: cash,
        revenue_minor: revenue,
        expenses_minor: expenses,
        free_cash_flow_minor: revenue.saturating_sub(expenses),
        minimum_cash_minor: minimum_cash,
        bankruptcy_day,
        decision_cycles,
        executed_actions,
        violations,
        survived: bankruptcy_day.is_none(),
    }
}

pub async fn run_many(config: SimConfig, trials: u32) -> MonteCarloSummary {
    let trials = trials.clamp(1, 10_000);
    let mut survival = 0_u32;
    let mut ending_sum = 0_i128;
    let mut worst = i128::MAX;
    let mut best = i128::MIN;
    let mut total_violations = 0_u64;

    for index in 0..trials {
        let trial = run(SimConfig { seed: config.seed.wrapping_add(index as u64), ..config.clone() }).await;
        survival += u32::from(trial.survived);
        ending_sum = ending_sum.saturating_add(trial.ending_cash_minor);
        worst = worst.min(trial.ending_cash_minor);
        best = best.max(trial.ending_cash_minor);
        total_violations = total_violations.saturating_add(trial.violations.len() as u64);
    }

    MonteCarloSummary {
        trials,
        survival_rate_bps: ((survival as u64 * 10_000) / trials as u64) as u32,
        bankruptcy_rate_bps: (((trials - survival) as u64 * 10_000) / trials as u64) as u32,
        average_ending_cash_minor: ending_sum / trials as i128,
        worst_ending_cash_minor: worst,
        best_ending_cash_minor: best,
        total_violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn simulation_is_deterministic_for_same_seed() {
        let config = SimConfig { days: 60, ..SimConfig::default() };
        let a = run(config.clone()).await;
        let b = run(config).await;
        assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    }

    #[tokio::test]
    async fn simulation_runs_without_paid_model_api() {
        let result = run(SimConfig { days: 30, ..SimConfig::default() }).await;
        assert_eq!(result.decision_cycles, 30);
        assert!(result.violations.is_empty(), "{:?}", result.violations);
        assert!(result.executed_actions > 0);
    }

    #[tokio::test]
    async fn severe_cash_shock_does_not_create_negative_spend() {
        let result = run(SimConfig { initial_cash_minor: 1_000, days: 30, seed: 99, ..SimConfig::default() }).await;
        assert!(result.ending_cash_minor >= 0 || result.bankruptcy_day.is_some());
        assert!(result.violations.is_empty(), "{:?}", result.violations);
    }

    #[tokio::test]
    async fn monte_carlo_summary_is_bounded_and_deterministic() {
        let config = SimConfig { days: 90, ..SimConfig::default() };
        let a = run_many(config.clone(), 25).await;
        let b = run_many(config, 25).await;
        assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
        assert_eq!(a.trials, 25);
        assert!(a.survival_rate_bps <= 10_000);
        assert!(a.bankruptcy_rate_bps <= 10_000);
        assert_eq!(a.total_violations, 0);
    }
}
