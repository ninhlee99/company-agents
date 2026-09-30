#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvaluationStatus {
    InsufficientEvidence,
    PartialEvidence,
    Evaluated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentEvaluationInput {
    pub agent_name: String,
    pub proposal_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
    pub revision_count: i64,
    pub escalated_count: i64,
    pub executed_count: i64,
    pub deferred_count: i64,
    pub observed_spend_minor: i128,
    pub projected_revenue_minor: i128,
    pub outcome_evidence_count: i64,
    pub observed_revenue_delta_minor: i128,
    pub observed_contribution_margin_delta_minor: i128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentEvaluation {
    pub agent_name: String,
    pub proposal_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
    pub revision_count: i64,
    pub escalated_count: i64,
    pub executed_count: i64,
    pub deferred_count: i64,
    pub observed_spend_minor: i128,
    pub projected_revenue_minor: i128,
    pub outcome_evidence_count: i64,
    pub observed_revenue_delta_minor: Option<i128>,
    pub observed_contribution_margin_delta_minor: Option<i128>,
    pub approval_rate_bps: u32,
    pub execution_realization_bps: u32,
    pub outcome_evidence_coverage_bps: u32,
    pub projected_return_bps: Option<i32>,
    pub observed_return_bps: Option<i32>,
    pub status: EvaluationStatus,
}

pub fn evaluate(input: &AgentEvaluationInput) -> Result<AgentEvaluation, String> {
    validate(input)?;

    let approval_rate_bps = ratio_bps(input.approved_count, input.proposal_count);
    let execution_realization_bps = ratio_bps(input.executed_count, input.approved_count);
    let outcome_evidence_coverage_bps =
        ratio_bps(input.outcome_evidence_count, input.executed_count);

    let projected_return_bps = signed_return_bps(
        input.projected_revenue_minor,
        input.observed_spend_minor,
    )?;
    let observed_return_bps = if input.outcome_evidence_count > 0 {
        signed_return_bps(
            input.observed_revenue_delta_minor,
            input.observed_spend_minor,
        )?
    } else {
        None
    };

    let status = if input.executed_count == 0 || input.outcome_evidence_count == 0 {
        EvaluationStatus::InsufficientEvidence
    } else if outcome_evidence_coverage_bps < 10_000 {
        EvaluationStatus::PartialEvidence
    } else {
        EvaluationStatus::Evaluated
    };

    Ok(AgentEvaluation {
        agent_name: input.agent_name.clone(),
        proposal_count: input.proposal_count,
        approved_count: input.approved_count,
        rejected_count: input.rejected_count,
        revision_count: input.revision_count,
        escalated_count: input.escalated_count,
        executed_count: input.executed_count,
        deferred_count: input.deferred_count,
        observed_spend_minor: input.observed_spend_minor,
        projected_revenue_minor: input.projected_revenue_minor,
        outcome_evidence_count: input.outcome_evidence_count,
        observed_revenue_delta_minor: (input.outcome_evidence_count > 0)
            .then_some(input.observed_revenue_delta_minor),
        observed_contribution_margin_delta_minor: (input.outcome_evidence_count > 0)
            .then_some(input.observed_contribution_margin_delta_minor),
        approval_rate_bps,
        execution_realization_bps,
        outcome_evidence_coverage_bps,
        projected_return_bps,
        observed_return_bps,
        status,
    })
}

fn validate(input: &AgentEvaluationInput) -> Result<(), String> {
    if input.agent_name.trim().is_empty() || input.agent_name.len() > 128 {
        return Err("agent_name is invalid".into());
    }
    for (name, value) in [
        ("proposal_count", input.proposal_count),
        ("approved_count", input.approved_count),
        ("rejected_count", input.rejected_count),
        ("revision_count", input.revision_count),
        ("escalated_count", input.escalated_count),
        ("executed_count", input.executed_count),
        ("deferred_count", input.deferred_count),
        ("outcome_evidence_count", input.outcome_evidence_count),
    ] {
        if value < 0 {
            return Err(format!("{name} cannot be negative"));
        }
    }
    if input.approved_count > input.proposal_count
        || input.executed_count > input.approved_count
        || input.outcome_evidence_count > input.executed_count
    {
        return Err("agent evaluation counters are not monotonic".into());
    }
    if input.observed_spend_minor < 0 || input.projected_revenue_minor < 0 {
        return Err("agent spend and projected revenue cannot be negative".into());
    }
    Ok(())
}

fn ratio_bps(numerator: i64, denominator: i64) -> u32 {
    if numerator <= 0 || denominator <= 0 {
        return 0;
    }
    numerator
        .saturating_mul(10_000)
        .checked_div(denominator)
        .unwrap_or(10_000)
        .min(10_000) as u32
}

fn signed_return_bps(numerator: i128, denominator: i128) -> Result<Option<i32>, String> {
    if denominator == 0 {
        return Ok(None);
    }
    if denominator < 0 {
        return Err("return denominator cannot be negative".into());
    }
    let value = numerator
        .checked_mul(10_000)
        .and_then(|value| value.checked_div(denominator))
        .ok_or_else(|| "agent return ratio overflow".to_string())?;
    Ok(Some(
        value
            .clamp(i128::from(i32::MIN), i128::from(i32::MAX))
            as i32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> AgentEvaluationInput {
        AgentEvaluationInput {
            agent_name: "Growth".into(),
            proposal_count: 10,
            approved_count: 8,
            rejected_count: 1,
            revision_count: 1,
            escalated_count: 0,
            executed_count: 6,
            deferred_count: 2,
            observed_spend_minor: 1_000,
            projected_revenue_minor: 2_000,
            outcome_evidence_count: 3,
            observed_revenue_delta_minor: 1_500,
            observed_contribution_margin_delta_minor: 700,
        }
    }

    #[test]
    fn metrics_are_deterministic() {
        let result = evaluate(&input()).unwrap();
        assert_eq!(result.approval_rate_bps, 8_000);
        assert_eq!(result.execution_realization_bps, 7_500);
        assert_eq!(result.outcome_evidence_coverage_bps, 5_000);
        assert_eq!(result.projected_return_bps, Some(20_000));
        assert_eq!(result.observed_return_bps, Some(15_000));
        assert_eq!(result.status, EvaluationStatus::PartialEvidence);
    }

    #[test]
    fn no_outcome_evidence_stays_insufficient() {
        let mut value = input();
        value.outcome_evidence_count = 0;
        let result = evaluate(&value).unwrap();
        assert_eq!(result.status, EvaluationStatus::InsufficientEvidence);
        assert_eq!(result.observed_return_bps, None);
    }

    #[test]
    fn full_outcome_evidence_is_evaluated() {
        let mut value = input();
        value.outcome_evidence_count = value.executed_count;
        let result = evaluate(&value).unwrap();
        assert_eq!(result.status, EvaluationStatus::Evaluated);
        assert_eq!(result.outcome_evidence_coverage_bps, 10_000);
    }

    #[test]
    fn invalid_counter_relationship_fails_closed() {
        let mut value = input();
        value.executed_count = value.approved_count + 1;
        assert!(evaluate(&value).is_err());
    }

    #[test]
    fn negative_observed_return_is_preserved() {
        let mut value = input();
        value.observed_revenue_delta_minor = -500;
        let result = evaluate(&value).unwrap();
        assert_eq!(result.observed_return_bps, Some(-5_000));
    }
}
