#![forbid(unsafe_code)]

use agent_runtime::types::{
    ActionKind, AgentRole, CompanySnapshot, GovernedProposal, GovernorDecision, Proposal, RiskTier,
};
use company_execution::ExecutionStatus;
use economic_core::CompanyStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel {
    Observe,
    Recommend,
    Simulate,
    HumanApprove,
    LimitedAutonomy,
    StrategicAutonomy,
}

impl AutonomyLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "OBSERVE",
            Self::Recommend => "RECOMMEND",
            Self::Simulate => "SIMULATE",
            Self::HumanApprove => "HUMAN_APPROVE",
            Self::LimitedAutonomy => "LIMITED_AUTONOMY",
            Self::StrategicAutonomy => "STRATEGIC_AUTONOMY",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutonomyDecision {
    Observe,
    Recommend,
    SimulateOnly,
    NeedsApproval,
    ExecuteLimited,
    ExecuteStrategic,
    Blocked,
}

impl AutonomyDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "OBSERVE",
            Self::Recommend => "RECOMMEND",
            Self::SimulateOnly => "SIMULATE_ONLY",
            Self::NeedsApproval => "NEEDS_APPROVAL",
            Self::ExecuteLimited => "EXECUTE_LIMITED",
            Self::ExecuteStrategic => "EXECUTE_STRATEGIC",
            Self::Blocked => "BLOCKED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyPolicy {
    pub max_level: AutonomyLevel,
    pub min_confidence_bps: u16,
    pub min_evidence_count: u8,
    pub max_limited_cost_minor: i128,
    pub min_runway_days: i64,
    pub require_reversible_for_limited: bool,
    pub require_simulation_for_limited: bool,
    pub allow_strategic: bool,
}

impl Default for AutonomyPolicy {
    fn default() -> Self {
        Self {
            max_level: AutonomyLevel::Simulate,
            min_confidence_bps: 8_500,
            min_evidence_count: 2,
            max_limited_cost_minor: 250,
            min_runway_days: 30,
            require_reversible_for_limited: true,
            require_simulation_for_limited: true,
            allow_strategic: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DigitalTwinConfig {
    pub daily_burn_minor: i128,
    pub reserve_cash_minor: i128,
}

impl DigitalTwinConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.daily_burn_minor <= 0 {
            return Err("daily burn must be positive".into());
        }
        if self.reserve_cash_minor < 0 {
            return Err("reserve cash cannot be negative".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DigitalTwinResult {
    pub execution_status: ExecutionStatus,
    pub reason: String,
    pub before_cash_minor: i128,
    pub after_cash_minor: i128,
    pub cash_delta_minor: i128,
    pub expense_delta_minor: i128,
    pub revenue_delta_minor: i128,
    pub asset_delta_minor: i128,
    pub backlog_delta: i64,
    pub simulated_runway_days: i64,
    pub status_after: CompanyStatus,
    pub negative_cash: bool,
    pub below_reserve: bool,
    pub bankruptcy_risk: bool,
    pub policy_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyGateInput {
    pub emergency_stop: bool,
    pub company_status: CompanyStatus,
    pub action: ActionKind,
    pub cost_minor: i128,
    pub risk: RiskTier,
    pub confidence_bps: u16,
    pub evidence_count: u8,
    pub reversible: bool,
    pub external_side_effect: bool,
    pub policy: AutonomyPolicy,
    pub simulation: Option<DigitalTwinResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyAssessment {
    pub decision: AutonomyDecision,
    pub ceiling: AutonomyLevel,
    pub required_level: AutonomyLevel,
    pub reason: String,
    pub simulation: Option<DigitalTwinResult>,
}

pub fn simulate_proposal(
    snapshot: &CompanySnapshot,
    proposal: &Proposal,
    config: &DigitalTwinConfig,
) -> Result<DigitalTwinResult, String> {
    config.validate()?;
    proposal.validate().map_err(|error| error.to_string())?;

    let governed = GovernedProposal {
        proposal: proposal.clone(),
        decision: GovernorDecision::Approve,
        reason: "digital twin simulation only".into(),
    };
    let (after, receipt) =
        company_execution::apply_approved(snapshot, &governed).map_err(|error| error.to_string())?;

    let cash_delta_minor = after
        .cash_minor
        .checked_sub(snapshot.cash_minor)
        .ok_or_else(|| "cash delta overflow".to_string())?;
    let expense_delta_minor = after
        .expenses_minor
        .checked_sub(snapshot.expenses_minor)
        .ok_or_else(|| "expense delta overflow".to_string())?;
    let revenue_delta_minor = after
        .revenue_minor
        .checked_sub(snapshot.revenue_minor)
        .ok_or_else(|| "revenue delta overflow".to_string())?;
    let asset_delta_minor = after
        .assets_minor
        .checked_sub(snapshot.assets_minor)
        .ok_or_else(|| "asset delta overflow".to_string())?;
    let backlog_delta = i64::from(after.backlog) - i64::from(snapshot.backlog);

    let usable_cash = (after.cash_minor - config.reserve_cash_minor).max(0);
    let simulated_runway_days = usable_cash
        .checked_div(config.daily_burn_minor)
        .unwrap_or(0)
        .min(i64::MAX as i128) as i64;
    let negative_cash = after.cash_minor < 0;
    let below_reserve = after.cash_minor < config.reserve_cash_minor;
    let bankruptcy_risk = negative_cash
        || simulated_runway_days <= 0
        || matches!(
            after.status,
            CompanyStatus::Liquidation | CompanyStatus::Bankrupt
        );
    let policy_safe = !negative_cash && !bankruptcy_risk;

    Ok(DigitalTwinResult {
        execution_status: receipt.status,
        reason: receipt.reason,
        before_cash_minor: snapshot.cash_minor,
        after_cash_minor: after.cash_minor,
        cash_delta_minor,
        expense_delta_minor,
        revenue_delta_minor,
        asset_delta_minor,
        backlog_delta,
        simulated_runway_days,
        status_after: after.status,
        negative_cash,
        below_reserve,
        bankruptcy_risk,
        policy_safe,
    })
}

pub fn assess(
    input: &AutonomyGateInput,
) -> Result<AutonomyAssessment, String> {
    if input.cost_minor < 0 {
        return Err("autonomy cost cannot be negative".into());
    }
    if input.confidence_bps > 10_000 {
        return Err("autonomy confidence must be <= 10000 bps".into());
    }
    if input.evidence_count > 200 {
        return Err("autonomy evidence count is out of range".into());
    }
    if input.policy.max_limited_cost_minor < 0 || input.policy.min_runway_days < 0 {
        return Err("autonomy policy contains invalid negative limits".into());
    }

    let required_level = if input.external_side_effect || input.action.inherently_material() {
        AutonomyLevel::HumanApprove
    } else if input.risk.rank() >= RiskTier::Critical.rank() {
        AutonomyLevel::HumanApprove
    } else if input.company_status != CompanyStatus::Active
        && input.company_status != CompanyStatus::Growth
    {
        AutonomyLevel::Recommend
    } else if input.confidence_bps < input.policy.min_confidence_bps
        || input.evidence_count < input.policy.min_evidence_count
    {
        AutonomyLevel::Recommend
    } else {
        AutonomyLevel::LimitedAutonomy
    };

    if input.emergency_stop {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::Blocked,
            ceiling: input.policy.max_level,
            required_level,
            reason: "emergency stop blocks autonomous side effects; audit/simulation remain available".into(),
            simulation: input.simulation.clone(),
        });
    }

    if input.external_side_effect || input.action.inherently_material() || input.risk.rank() >= RiskTier::Critical.rank() {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::NeedsApproval,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::HumanApprove,
            reason: "material, external or critical-risk actions require explicit human approval".into(),
            simulation: input.simulation.clone(),
        });
    }

    if matches!(
        input.company_status,
        CompanyStatus::Distress
            | CompanyStatus::Emergency
            | CompanyStatus::Liquidation
            | CompanyStatus::Bankrupt
    ) {
        let decision = if input.policy.max_level >= AutonomyLevel::Simulate
            && input.simulation.is_some()
        {
            AutonomyDecision::SimulateOnly
        } else if input.policy.max_level == AutonomyLevel::Observe {
            AutonomyDecision::Observe
        } else {
            AutonomyDecision::Recommend
        };
        return Ok(AutonomyAssessment {
            decision,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::Recommend,
            reason: "financial distress caps autonomy at recommendation/simulation".into(),
            simulation: input.simulation.clone(),
        });
    }

    if input.confidence_bps < input.policy.min_confidence_bps
        || input.evidence_count < input.policy.min_evidence_count
    {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::Recommend,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::Recommend,
            reason: "confidence or evidence is below the limited-autonomy gate".into(),
            simulation: input.simulation.clone(),
        });
    }

    if input.policy.max_level < AutonomyLevel::Simulate {
        return Ok(AutonomyAssessment {
            decision: if input.policy.max_level == AutonomyLevel::Observe {
                AutonomyDecision::Observe
            } else {
                AutonomyDecision::Recommend
            },
            ceiling: input.policy.max_level,
            required_level,
            reason: "configured autonomy ceiling does not permit simulation or execution".into(),
            simulation: input.simulation.clone(),
        });
    }

    if input.policy.max_level == AutonomyLevel::Simulate {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::SimulateOnly,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::Simulate,
            reason: "configured ceiling is simulation-only".into(),
            simulation: input.simulation.clone(),
        });
    }

    if input.policy.max_level < AutonomyLevel::LimitedAutonomy {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::NeedsApproval,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::HumanApprove,
            reason: "configured ceiling does not permit autonomous execution".into(),
            simulation: input.simulation.clone(),
        });
    }

    let simulation = if input.policy.require_simulation_for_limited {
        input
            .simulation
            .clone()
            .ok_or_else(|| "limited autonomy requires a successful digital twin simulation".to_string())?
    } else {
        input
            .simulation
            .clone()
            .ok_or_else(|| "limited autonomy requires simulation evidence".to_string())?
    };

    if input.cost_minor > input.policy.max_limited_cost_minor {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::NeedsApproval,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::HumanApprove,
            reason: "cost exceeds limited-autonomy cap".into(),
            simulation: Some(simulation),
        });
    }

    if input.policy.require_reversible_for_limited && !input.reversible {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::NeedsApproval,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::HumanApprove,
            reason: "limited autonomy requires a reversible action".into(),
            simulation: Some(simulation),
        });
    }

    if !simulation.policy_safe
        || simulation.simulated_runway_days < input.policy.min_runway_days
        || simulation.negative_cash
        || simulation.bankruptcy_risk
    {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::NeedsApproval,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::HumanApprove,
            reason: "digital twin predicts an unsafe liquidity or bankruptcy condition".into(),
            simulation: Some(simulation),
        });
    }

    if input.policy.max_level >= AutonomyLevel::StrategicAutonomy && input.policy.allow_strategic {
        return Ok(AutonomyAssessment {
            decision: AutonomyDecision::ExecuteStrategic,
            ceiling: input.policy.max_level,
            required_level: AutonomyLevel::StrategicAutonomy,
            reason: "explicit strategic autonomy policy and simulation gates are satisfied".into(),
            simulation: Some(simulation),
        });
    }

    Ok(AutonomyAssessment {
        decision: AutonomyDecision::ExecuteLimited,
        ceiling: input.policy.max_level,
        required_level: AutonomyLevel::LimitedAutonomy,
        reason: "limited autonomy policy and digital twin gates are satisfied".into(),
        simulation: Some(simulation),
    })
}

pub fn policy_with(
    max_level: AutonomyLevel,
    min_confidence_bps: u16,
    min_evidence_count: u8,
    max_limited_cost_minor: i128,
    min_runway_days: i64,
    allow_strategic: bool,
) -> Result<AutonomyPolicy, String> {
    if min_confidence_bps > 10_000
        || max_limited_cost_minor < 0
        || min_runway_days < 0
    {
        return Err("invalid autonomy policy bounds".into());
    }
    Ok(AutonomyPolicy {
        max_level,
        min_confidence_bps,
        min_evidence_count,
        max_limited_cost_minor,
        min_runway_days,
        require_reversible_for_limited: true,
        require_simulation_for_limited: true,
        allow_strategic,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> CompanySnapshot {
        CompanySnapshot {
            company_id: "company-1".into(),
            cash_minor: 10_000,
            revenue_minor: 5_000,
            expenses_minor: 2_000,
            liabilities_minor: 0,
            assets_minor: 8_000,
            runway_days: 90,
            status: CompanyStatus::Growth,
            budget_remaining_minor: 1_000,
            experiment_budget_minor: 500,
            content_cost_minor: 100,
            content_revenue_minor: 200,
            backlog: 8,
            capacity: 10,
            conversion_bps: 300,
            audience_growth_bps: 100,
            hiring_need: 0,
        }
    }

    fn proposal() -> Proposal {
        Proposal {
            agent: AgentRole::Growth,
            objective: "run bounded experiment".into(),
            action: ActionKind::CreateExperiment,
            cost_minor: 100,
            expected_revenue_minor: 300,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence: vec!["trend".into(), "offer".into()],
            rationale: "bounded and reversible".into(),
            reversible: true,
            requested_permission: agent_runtime::types::Permission::Propose,
        }
    }

    fn simulation() -> DigitalTwinResult {
        DigitalTwinResult {
            execution_status: ExecutionStatus::Executed,
            reason: "ok".into(),
            before_cash_minor: 10_000,
            after_cash_minor: 9_900,
            cash_delta_minor: -100,
            expense_delta_minor: 100,
            revenue_delta_minor: 0,
            asset_delta_minor: -100,
            backlog_delta: 0,
            simulated_runway_days: 50,
            status_after: CompanyStatus::Growth,
            negative_cash: false,
            below_reserve: false,
            bankruptcy_risk: false,
            policy_safe: true,
        }
    }

    #[test]
    fn digital_twin_matches_bounded_execution_shape() {
        let result = simulate_proposal(
            &snapshot(),
            &proposal(),
            &DigitalTwinConfig {
                daily_burn_minor: 100,
                reserve_cash_minor: 5_000,
            },
        )
        .unwrap();
        assert_eq!(result.after_cash_minor, 9_900);
        assert_eq!(result.cash_delta_minor, -100);
        assert_eq!(result.execution_status, ExecutionStatus::Executed);
        assert_eq!(result.simulated_runway_days, 49);
        assert!(result.policy_safe);
    }

    #[test]
    fn observe_ceiling_never_upgrades_to_recommendation_execution() {
        let input = AutonomyGateInput {
            emergency_stop: false,
            company_status: CompanyStatus::Growth,
            action: ActionKind::CreateExperiment,
            cost_minor: 0,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence_count: 5,
            reversible: true,
            external_side_effect: false,
            policy: AutonomyPolicy {
                max_level: AutonomyLevel::Observe,
                ..AutonomyPolicy::default()
            },
            simulation: None,
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::Observe);
    }

    #[test]
    fn external_side_effects_require_human_approval() {
        let input = AutonomyGateInput {
            emergency_stop: false,
            company_status: CompanyStatus::Growth,
            action: ActionKind::PublishContent,
            cost_minor: 0,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence_count: 5,
            reversible: true,
            external_side_effect: true,
            policy: AutonomyPolicy {
                max_level: AutonomyLevel::StrategicAutonomy,
                allow_strategic: true,
                ..AutonomyPolicy::default()
            },
            simulation: Some(simulation()),
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::NeedsApproval);
    }

    #[test]
    fn emergency_stop_blocks() {
        let input = AutonomyGateInput {
            emergency_stop: true,
            company_status: CompanyStatus::Growth,
            action: ActionKind::CreateExperiment,
            cost_minor: 50,
            risk: RiskTier::Medium,
            confidence_bps: 10_000,
            evidence_count: 5,
            reversible: true,
            external_side_effect: false,
            policy: AutonomyPolicy::default(),
            simulation: Some(simulation()),
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::Blocked);
    }

    #[test]
    fn low_evidence_stays_recommend_only() {
        let input = AutonomyGateInput {
            emergency_stop: false,
            company_status: CompanyStatus::Growth,
            action: ActionKind::CreateExperiment,
            cost_minor: 50,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence_count: 1,
            reversible: true,
            external_side_effect: false,
            policy: AutonomyPolicy::default(),
            simulation: Some(simulation()),
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::Recommend);
    }

    #[test]
    fn default_ceiling_is_simulation_only() {
        let input = AutonomyGateInput {
            emergency_stop: false,
            company_status: CompanyStatus::Growth,
            action: ActionKind::CreateExperiment,
            cost_minor: 50,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence_count: 5,
            reversible: true,
            external_side_effect: false,
            policy: AutonomyPolicy::default(),
            simulation: Some(simulation()),
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::SimulateOnly);
    }

    #[test]
    fn human_approve_ceiling_never_executes_autonomously() {
        let input = AutonomyGateInput {
            emergency_stop: false,
            company_status: CompanyStatus::Growth,
            action: ActionKind::CreateExperiment,
            cost_minor: 50,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence_count: 5,
            reversible: true,
            external_side_effect: false,
            policy: AutonomyPolicy {
                max_level: AutonomyLevel::HumanApprove,
                ..AutonomyPolicy::default()
            },
            simulation: Some(simulation()),
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::NeedsApproval);
    }

    #[test]
    fn limited_autonomy_requires_strong_gates() {
        let input = AutonomyGateInput {
            emergency_stop: false,
            company_status: CompanyStatus::Growth,
            action: ActionKind::CreateExperiment,
            cost_minor: 50,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence_count: 5,
            reversible: true,
            external_side_effect: false,
            policy: AutonomyPolicy {
                max_level: AutonomyLevel::LimitedAutonomy,
                ..AutonomyPolicy::default()
            },
            simulation: Some(simulation()),
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::ExecuteLimited);
    }

    #[test]
    fn strategic_requires_explicit_enablement() {
        let input = AutonomyGateInput {
            emergency_stop: false,
            company_status: CompanyStatus::Growth,
            action: ActionKind::CreateExperiment,
            cost_minor: 50,
            risk: RiskTier::Low,
            confidence_bps: 9_000,
            evidence_count: 5,
            reversible: true,
            external_side_effect: false,
            policy: AutonomyPolicy {
                max_level: AutonomyLevel::StrategicAutonomy,
                allow_strategic: false,
                ..AutonomyPolicy::default()
            },
            simulation: Some(simulation()),
        };
        assert_eq!(assess(&input).unwrap().decision, AutonomyDecision::ExecuteLimited);
    }
}
