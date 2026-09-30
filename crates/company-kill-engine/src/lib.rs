#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KillDecision {
    Continue,
    Review,
    Kill,
}

impl KillDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Continue => "CONTINUE",
            Self::Review => "REVIEW",
            Self::Kill => "KILL",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KillReason {
    Healthy,
    BudgetExhausted,
    DurationExpired,
    LossLimitBreached,
    KillMetricBreached,
    EvidenceInsufficient,
    EvidenceStale,
    EvidenceFutureDated,
}

impl KillReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::BudgetExhausted => "BUDGET_EXHAUSTED",
            Self::DurationExpired => "DURATION_EXPIRED",
            Self::LossLimitBreached => "LOSS_LIMIT_BREACHED",
            Self::KillMetricBreached => "KILL_METRIC_BREACHED",
            Self::EvidenceInsufficient => "EVIDENCE_INSUFFICIENT",
            Self::EvidenceStale => "EVIDENCE_STALE",
            Self::EvidenceFutureDated => "EVIDENCE_FUTURE_DATED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct KillPolicy {
    pub max_budget_minor: i128,
    pub max_duration_seconds: u64,
    pub max_loss_minor: Option<i128>,
    pub kill_metric_bps: Option<i64>,
    pub min_evidence_count: u32,
    pub min_confidence_bps: u32,
    pub max_evidence_age_seconds: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct KillEvidence {
    pub spend_minor: i128,
    pub elapsed_seconds: u64,
    pub contribution_margin_minor: Option<i128>,
    pub lift_bps: Option<i64>,
    pub evidence_count: u32,
    pub confidence_bps: u32,
    pub observed_at_epoch: Option<i64>,
    pub as_of_epoch: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct KillAssessment {
    pub decision: KillDecision,
    pub reason: KillReason,
}

pub fn validate_policy(policy: &KillPolicy) -> Result<(), String> {
    if policy.max_budget_minor <= 0 || policy.max_duration_seconds == 0 {
        return Err("kill policy budget and duration must be positive".into());
    }
    if policy.min_confidence_bps > 10_000 {
        return Err("kill policy confidence must be <= 10000 bps".into());
    }
    if let Some(limit) = policy.max_loss_minor {
        if limit < 0 {
            return Err("kill loss limit cannot be negative".into());
        }
    }
    if let Some(metric) = policy.kill_metric_bps {
        if !(0..=10_000).contains(&metric) {
            return Err("kill metric must be between 0 and 10000 bps".into());
        }
    }
    if let Some(max_age) = policy.max_evidence_age_seconds {
        if !(60..=31_536_000).contains(&max_age) {
            return Err("kill evidence age must be between 60 seconds and 365 days".into());
        }
    }
    Ok(())
}

pub fn assess(policy: &KillPolicy, evidence: &KillEvidence) -> Result<KillAssessment, String> {
    validate_policy(policy)?;
    if evidence.spend_minor < 0
        || evidence
            .contribution_margin_minor
            .is_some_and(|value| value == i128::MIN)
        || evidence.confidence_bps > 10_000
    {
        return Err("kill evidence contains invalid bounds".into());
    }
    if evidence.as_of_epoch <= 0 {
        return Err("kill evidence as_of_epoch must be positive".into());
    }

    if evidence.spend_minor >= policy.max_budget_minor {
        return Ok(KillAssessment {
            decision: KillDecision::Kill,
            reason: KillReason::BudgetExhausted,
        });
    }
    if evidence.elapsed_seconds >= policy.max_duration_seconds {
        return Ok(KillAssessment {
            decision: KillDecision::Kill,
            reason: KillReason::DurationExpired,
        });
    }

    if let Some(observed_at) = evidence.observed_at_epoch {
        if observed_at > evidence.as_of_epoch {
            return Ok(KillAssessment {
                decision: KillDecision::Review,
                reason: KillReason::EvidenceFutureDated,
            });
        }
        if let Some(max_age) = policy.max_evidence_age_seconds {
            if evidence.as_of_epoch.saturating_sub(observed_at) > max_age {
                return Ok(KillAssessment {
                    decision: KillDecision::Review,
                    reason: KillReason::EvidenceStale,
                });
            }
        }
    } else if policy.max_evidence_age_seconds.is_some() {
        return Ok(KillAssessment {
            decision: KillDecision::Review,
            reason: KillReason::EvidenceStale,
        });
    }

    if evidence.evidence_count < policy.min_evidence_count
        || evidence.confidence_bps < policy.min_confidence_bps
    {
        return Ok(KillAssessment {
            decision: KillDecision::Review,
            reason: KillReason::EvidenceInsufficient,
        });
    }

    if let (Some(limit), Some(margin)) =
        (policy.max_loss_minor, evidence.contribution_margin_minor)
    {
        if margin < 0 && margin.unsigned_abs() > limit as u128 {
            return Ok(KillAssessment {
                decision: KillDecision::Kill,
                reason: KillReason::LossLimitBreached,
            });
        }
    }

    if let (Some(threshold), Some(lift)) = (policy.kill_metric_bps, evidence.lift_bps) {
        if lift <= -threshold {
            return Ok(KillAssessment {
                decision: KillDecision::Kill,
                reason: KillReason::KillMetricBreached,
            });
        }
    }

    Ok(KillAssessment {
        decision: KillDecision::Continue,
        reason: KillReason::Healthy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> KillPolicy {
        KillPolicy {
            max_budget_minor: 1_000,
            max_duration_seconds: 86_400,
            max_loss_minor: Some(500),
            kill_metric_bps: Some(300),
            min_evidence_count: 2,
            min_confidence_bps: 8_000,
            max_evidence_age_seconds: Some(86_400),
        }
    }

    fn evidence() -> KillEvidence {
        KillEvidence {
            spend_minor: 100,
            elapsed_seconds: 1_000,
            contribution_margin_minor: Some(100),
            lift_bps: Some(100),
            evidence_count: 3,
            confidence_bps: 9_000,
            observed_at_epoch: Some(1_800_000_000),
            as_of_epoch: 1_800_000_100,
        }
    }

    #[test]
    fn continues_with_fresh_sufficient_positive_evidence() {
        let result = assess(&policy(), &evidence()).unwrap();
        assert_eq!(result.decision, KillDecision::Continue);
        assert_eq!(result.reason, KillReason::Healthy);
    }

    #[test]
    fn kills_on_budget_duration_loss_or_kill_metric() {
        let mut value = evidence();
        value.spend_minor = 1_000;
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::BudgetExhausted);

        let mut value = evidence();
        value.elapsed_seconds = 86_400;
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::DurationExpired);

        let mut value = evidence();
        value.contribution_margin_minor = Some(-501);
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::LossLimitBreached);

        let mut value = evidence();
        value.lift_bps = Some(-300);
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::KillMetricBreached);
    }

    #[test]
    fn stale_future_and_weak_evidence_require_review() {
        let mut value = evidence();
        value.observed_at_epoch = Some(1_799_000_000);
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::EvidenceStale);

        let mut value = evidence();
        value.observed_at_epoch = Some(1_800_000_101);
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::EvidenceFutureDated);

        let mut value = evidence();
        value.evidence_count = 1;
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::EvidenceInsufficient);
    }

    #[test]
    fn strict_missing_timestamp_requires_review() {
        let mut value = evidence();
        value.observed_at_epoch = None;
        assert_eq!(assess(&policy(), &value).unwrap().reason, KillReason::EvidenceStale);
    }

    #[test]
    fn invalid_policy_and_evidence_are_rejected() {
        let mut p = policy();
        p.max_budget_minor = 0;
        assert!(assess(&p, &evidence()).is_err());

        let mut value = evidence();
        value.confidence_bps = 10_001;
        assert!(assess(&policy(), &value).is_err());
    }
}
