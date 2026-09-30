#![forbid(unsafe_code)]

use economic_core::CompanyStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalCandidate {
    pub candidate_id: Uuid,
    pub unit_id: String,
    pub purpose: String,
    pub expected_contribution_minor: i128,
    pub downside_minor: i128,
    pub capital_required_minor: i128,
    pub feedback_seconds: u64,
    pub reversibility_bps: u32,
    pub strategic_value_bps: u32,
    pub confidence_bps: u32,
    pub evidence_count: u32,
    pub allocation_cap_minor: i128,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalPolicy {
    pub company_status: CompanyStatus,
    pub cash_available_minor: i128,
    pub reserve_cash_minor: i128,
    pub discretionary_budget_minor: i128,
    pub min_runway_days: i64,
    pub runway_days: i64,
    pub min_confidence_bps: u32,
    pub min_evidence_count: u32,
    pub max_feedback_seconds: u64,
    pub min_score_bps: u32,
    pub emergency_stop: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapitalDecisionStatus {
    Allocate,
    Hold,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalDecision {
    pub candidate_id: Uuid,
    pub status: CapitalDecisionStatus,
    pub score_bps: u32,
    pub allocation_minor: i128,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalAllocationPlan {
    pub plan_id: Uuid,
    pub total_capital_minor: i128,
    pub planned_capital_minor: i128,
    pub unallocated_minor: i128,
    pub decisions: Vec<CapitalDecision>,
}

pub fn validate_candidate(candidate: &CapitalCandidate) -> Result<(), String> {
    if candidate.unit_id.trim().is_empty() || candidate.purpose.trim().is_empty() {
        return Err("capital candidate identity and purpose are required".into());
    }
    if candidate.expected_contribution_minor < 0
        || candidate.downside_minor < 0
        || candidate.capital_required_minor <= 0
        || candidate.allocation_cap_minor < 0
    {
        return Err("capital candidate economics are invalid".into());
    }
    for (name, value) in [
        ("reversibility_bps", candidate.reversibility_bps),
        ("strategic_value_bps", candidate.strategic_value_bps),
        ("confidence_bps", candidate.confidence_bps),
    ] {
        if value > 10_000 {
            return Err(format!("{name} must be between 0 and 10000"));
        }
    }
    if candidate.evidence_count > 10_000 {
        return Err("capital candidate evidence count is out of range".into());
    }
    if candidate.feedback_seconds == 0 {
        return Err("feedback time must be positive".into());
    }
    Ok(())
}

pub fn validate_policy(policy: &CapitalPolicy) -> Result<(), String> {
    if policy.cash_available_minor < 0
        || policy.reserve_cash_minor < 0
        || policy.discretionary_budget_minor < 0
        || policy.reserve_cash_minor > policy.cash_available_minor
    {
        return Err("capital policy cash bounds are invalid".into());
    }
    if policy.min_runway_days < 0 || policy.runway_days < 0 {
        return Err("capital policy runway cannot be negative".into());
    }
    if policy.min_confidence_bps > 10_000
        || policy.min_evidence_count > 10_000
        || policy.min_score_bps > 10_000
        || policy.max_feedback_seconds == 0
    {
        return Err("capital policy bounds are invalid".into());
    }
    if policy.discretionary_budget_minor
        > policy.cash_available_minor.saturating_sub(policy.reserve_cash_minor)
    {
        return Err("discretionary budget exceeds cash available after reserve".into());
    }
    Ok(())
}

pub fn score_candidate(candidate: &CapitalCandidate, policy: &CapitalPolicy) -> Result<u32, String> {
    validate_candidate(candidate)?;
    validate_policy(policy)?;

    let return_bps = ratio_bps(candidate.expected_contribution_minor, candidate.capital_required_minor);
    let downside_safety_bps = 10_000u32.saturating_sub(
        ratio_bps(candidate.downside_minor, candidate.capital_required_minor),
    );
    let speed_bps = if candidate.feedback_seconds >= policy.max_feedback_seconds {
        0
    } else {
        10_000u32.saturating_sub(
            ((candidate.feedback_seconds as u128 * 10_000) / policy.max_feedback_seconds as u128)
                .min(10_000) as u32,
        )
    };

    let weighted = u64::from(return_bps) * 30
        + u64::from(downside_safety_bps) * 20
        + u64::from(speed_bps) * 15
        + u64::from(candidate.reversibility_bps) * 10
        + u64::from(candidate.strategic_value_bps) * 10
        + u64::from(candidate.confidence_bps) * 15;

    Ok((weighted / 100) as u32)
}

pub fn decide_candidate(
    candidate: &CapitalCandidate,
    policy: &CapitalPolicy,
) -> Result<CapitalDecision, String> {
    let score = score_candidate(candidate, policy)?;
    if policy.emergency_stop {
        return Ok(held(candidate.candidate_id, score, "emergency stop blocks discretionary capital"));
    }
    if !matches!(policy.company_status, CompanyStatus::Active | CompanyStatus::Growth)
        || policy.runway_days < policy.min_runway_days
    {
        return Ok(held(candidate.candidate_id, score, "company liquidity gate blocks discretionary capital"));
    }
    if candidate.expected_contribution_minor == 0 {
        return Ok(CapitalDecision {
            candidate_id: candidate.candidate_id,
            status: CapitalDecisionStatus::Reject,
            score_bps: score,
            allocation_minor: 0,
            reason: "candidate has no positive expected contribution".into(),
        });
    }
    if candidate.confidence_bps < policy.min_confidence_bps
        || candidate.evidence_count < policy.min_evidence_count
    {
        return Ok(held(candidate.candidate_id, score, "candidate evidence is below the allocation gate"));
    }
    if score < policy.min_score_bps {
        return Ok(CapitalDecision {
            candidate_id: candidate.candidate_id,
            status: CapitalDecisionStatus::Reject,
            score_bps: score,
            allocation_minor: 0,
            reason: "candidate score is below the allocation threshold".into(),
        });
    }
    Ok(CapitalDecision {
        candidate_id: candidate.candidate_id,
        status: CapitalDecisionStatus::Allocate,
        score_bps: score,
        allocation_minor: candidate.capital_required_minor.min(candidate.allocation_cap_minor.max(1)),
        reason: "candidate passes liquidity, evidence and deterministic return/risk gates".into(),
    })
}

pub fn plan(
    policy: &CapitalPolicy,
    candidates: &[CapitalCandidate],
) -> Result<CapitalAllocationPlan, String> {
    plan_with_id(Uuid::new_v4(), policy, candidates)
}

pub fn plan_with_id(
    plan_id: Uuid,
    policy: &CapitalPolicy,
    candidates: &[CapitalCandidate],
) -> Result<CapitalAllocationPlan, String> {
    validate_policy(policy)?;
    let mut candidates = candidates.to_vec();
    for candidate in &candidates {
        validate_candidate(candidate)?;
        if candidate.candidate_id == Uuid::nil() {
            return Err("candidate id cannot be nil".into());
        }
    }
    candidates.sort_by(|a, b| {
        score_candidate(b, policy)
            .unwrap_or(0)
            .cmp(&score_candidate(a, policy).unwrap_or(0))
            .then_with(|| a.candidate_id.cmp(&b.candidate_id))
    });

    let total_capital_minor = policy
        .cash_available_minor
        .saturating_sub(policy.reserve_cash_minor)
        .min(policy.discretionary_budget_minor);

    let mut remaining = total_capital_minor;
    let mut decisions = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let mut decision = decide_candidate(&candidate, policy)?;
        if decision.status == CapitalDecisionStatus::Allocate {
            if remaining <= 0 {
                decision.status = CapitalDecisionStatus::Hold;
                decision.allocation_minor = 0;
                decision.reason = "portfolio allocation budget is exhausted".into();
            } else {
                decision.allocation_minor = decision.allocation_minor.min(remaining);
                if decision.allocation_minor <= 0 {
                    decision.status = CapitalDecisionStatus::Hold;
                    decision.reason = "candidate has no allocatable amount after hard caps".into();
                } else {
                    remaining = remaining.saturating_sub(decision.allocation_minor);
                }
            }
        }
        decisions.push(decision);
    }

    let planned_capital_minor = total_capital_minor.saturating_sub(remaining);
    Ok(CapitalAllocationPlan {
        plan_id,
        total_capital_minor,
        planned_capital_minor,
        unallocated_minor: remaining,
        decisions,
    })
}

fn ratio_bps(numerator: i128, denominator: i128) -> u32 {
    if numerator <= 0 || denominator <= 0 {
        return 0;
    }
    ((numerator as u128)
        .saturating_mul(10_000)
        .checked_div(denominator as u128)
        .unwrap_or(0)
        .min(10_000)) as u32
}

fn held(candidate_id: Uuid, score_bps: u32, reason: &str) -> CapitalDecision {
    CapitalDecision {
        candidate_id,
        status: CapitalDecisionStatus::Hold,
        score_bps,
        allocation_minor: 0,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> CapitalPolicy {
        CapitalPolicy {
            company_status: CompanyStatus::Growth,
            cash_available_minor: 10_000,
            reserve_cash_minor: 3_000,
            discretionary_budget_minor: 5_000,
            min_runway_days: 45,
            runway_days: 90,
            min_confidence_bps: 8_000,
            min_evidence_count: 3,
            max_feedback_seconds: 604_800,
            min_score_bps: 6_500,
            emergency_stop: false,
        }
    }

    fn candidate(seed: u128, expected: i128, downside: i128) -> CapitalCandidate {
        CapitalCandidate {
            candidate_id: Uuid::from_u128(seed),
            unit_id: format!("unit-{seed}"),
            purpose: "validated growth test".into(),
            expected_contribution_minor: expected,
            downside_minor: downside,
            capital_required_minor: 1_000,
            feedback_seconds: 86_400,
            reversibility_bps: 9_000,
            strategic_value_bps: 7_000,
            confidence_bps: 9_000,
            evidence_count: 5,
            allocation_cap_minor: 800,
        }
    }

    #[test]
    fn high_confidence_positive_return_can_allocate() {
        let c = candidate(1, 3_000, 200);
        let decision = decide_candidate(&c, &policy()).unwrap();
        assert_eq!(decision.status, CapitalDecisionStatus::Allocate);
        assert_eq!(decision.allocation_minor, 800);
    }

    #[test]
    fn emergency_stop_holds_without_spend() {
        let mut p = policy();
        p.emergency_stop = true;
        let decision = decide_candidate(&candidate(1, 3_000, 200), &p).unwrap();
        assert_eq!(decision.status, CapitalDecisionStatus::Hold);
        assert_eq!(decision.allocation_minor, 0);
    }

    #[test]
    fn low_confidence_holds() {
        let mut c = candidate(1, 3_000, 200);
        c.confidence_bps = 7_999;
        let decision = decide_candidate(&c, &policy()).unwrap();
        assert_eq!(decision.status, CapitalDecisionStatus::Hold);
    }

    #[test]
    fn plan_respects_total_discretionary_budget() {
        let candidates = vec![
            candidate(1, 3_000, 200),
            candidate(2, 3_000, 200),
            candidate(3, 3_000, 200),
        ];
        let plan = plan(&policy(), &candidates).unwrap();
        assert_eq!(plan.total_capital_minor, 5_000);
        assert_eq!(plan.planned_capital_minor, 2_400);
        assert_eq!(plan.unallocated_minor, 2_600);
    }

    #[test]
    fn invalid_budget_above_cash_is_rejected() {
        let mut p = policy();
        p.discretionary_budget_minor = 8_000;
        assert!(validate_policy(&p).is_err());
    }
}
