#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LearningKind {
    Learning,
    Failure,
    NearMiss,
    Success,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LearningDecision {
    Reuse,
    Adjust,
    Retest,
    Stop,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningEntry {
    pub entry_key: String,
    pub source_type: String,
    pub source_id: String,
    pub kind: LearningKind,
    pub severity: FailureSeverity,
    pub hypothesis: String,
    pub context: String,
    pub expected_outcome: String,
    pub actual_outcome: String,
    pub impact_minor: i128,
    pub confidence_bps: i64,
    pub root_cause: String,
    pub corrective_action: String,
    pub reusable_rule: String,
    pub decision: LearningDecision,
}

pub fn validate(entry: &LearningEntry) -> Result<(), String> {
    for (name, value) in [
        ("entry key", &entry.entry_key),
        ("source type", &entry.source_type),
        ("source id", &entry.source_id),
        ("hypothesis", &entry.hypothesis),
        ("context", &entry.context),
        ("expected outcome", &entry.expected_outcome),
        ("actual outcome", &entry.actual_outcome),
        ("root cause", &entry.root_cause),
        ("corrective action", &entry.corrective_action),
        ("reusable rule", &entry.reusable_rule),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{name} is required"));
        }
        if value.len() > 4000 {
            return Err(format!("{name} exceeds 4000 bytes"));
        }
    }
    if entry.entry_key.len() > 256 || entry.source_type.len() > 64 || entry.source_id.len() > 256 {
        return Err("learning identifiers exceed allowed length".into());
    }
    if !(0..=10_000).contains(&entry.confidence_bps) {
        return Err("learning confidence must be between 0 and 10000 bps".into());
    }
    if matches!(entry.kind, LearningKind::Failure | LearningKind::NearMiss)
        && entry.severity == FailureSeverity::None
    {
        return Err("failure and near-miss entries require a non-NONE severity".into());
    }
    if !matches!(entry.kind, LearningKind::Failure | LearningKind::NearMiss)
        && entry.severity != FailureSeverity::None
    {
        return Err("non-failure entries must use NONE severity".into());
    }
    Ok(())
}

pub fn validate_evidence(entry: &LearningEntry) -> Result<(), String> {
    validate(entry)?;
    if entry.actual_outcome.trim().is_empty() {
        return Err("actual outcome evidence is required".into());
    }
    if entry.decision == LearningDecision::Reuse && entry.reusable_rule.trim().is_empty() {
        return Err("reuse decisions require a reusable rule".into());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningSynthesis {
    pub total_entries: usize,
    pub successes: usize,
    pub failures: usize,
    pub near_misses: usize,
    pub total_impact_minor: i128,
    pub reusable_rules: Vec<String>,
    pub high_confidence_rules_count: usize,
}

pub fn synthesize_learnings(entries: &[LearningEntry]) -> Result<LearningSynthesis, String> {
    let mut successes = 0;
    let mut failures = 0;
    let mut near_misses = 0;
    let mut total_impact = 0_i128;
    let mut reusable_rules = Vec::new();
    let mut high_confidence_rules_count = 0;

    for entry in entries {
        validate(entry)?;
        match entry.kind {
            LearningKind::Success => successes += 1,
            LearningKind::Failure => failures += 1,
            LearningKind::NearMiss => near_misses += 1,
            LearningKind::Learning => {}
        }
        total_impact = total_impact
            .checked_add(entry.impact_minor)
            .ok_or_else(|| "impact overflow during learning synthesis".to_string())?;

        if !entry.reusable_rule.trim().is_empty() {
            reusable_rules.push(entry.reusable_rule.clone());
            if entry.confidence_bps >= 8_000 {
                high_confidence_rules_count += 1;
            }
        }
    }

    reusable_rules.sort();
    reusable_rules.dedup();

    Ok(LearningSynthesis {
        total_entries: entries.len(),
        successes,
        failures,
        near_misses,
        total_impact_minor: total_impact,
        reusable_rules,
        high_confidence_rules_count,
    })
}

impl fmt::Display for LearningDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> LearningEntry {
        LearningEntry {
            entry_key: "exp-1:variant-a".into(),
            source_type: "EXPERIMENT".into(),
            source_id: "00000000-0000-0000-0000-000000000001".into(),
            kind: LearningKind::Failure,
            severity: FailureSeverity::Medium,
            hypothesis: "Shorter hook improves first-second retention".into(),
            context: "Affiliate video experiment".into(),
            expected_outcome: "Retention improves by at least 5%".into(),
            actual_outcome: "Retention fell 8% after 1000 views".into(),
            impact_minor: -120_000,
            confidence_bps: 8_500,
            root_cause: "Hook removed product context".into(),
            corrective_action: "Restore product context in first frame".into(),
            reusable_rule: "Do not remove product context from the first frame".into(),
            decision: LearningDecision::Adjust,
        }
    }

    #[test]
    fn accepts_evidence_backed_failure() {
        assert!(validate_evidence(&entry()).is_ok());
    }

    #[test]
    fn rejects_failure_without_severity() {
        let mut value = entry();
        value.severity = FailureSeverity::None;
        assert!(validate(&value).is_err());
    }

    #[test]
    fn rejects_out_of_range_confidence() {
        let mut value = entry();
        value.confidence_bps = 10_001;
        assert!(validate(&value).is_err());
    }

    #[test]
    fn rejects_empty_actual_outcome() {
        let mut value = entry();
        value.actual_outcome.clear();
        assert!(validate_evidence(&value).is_err());
    }

    #[test]
    fn round_trips_json() {
        let value = entry();
        let encoded = serde_json::to_string(&value).unwrap();
        let decoded: LearningEntry = serde_json::from_str(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn synthesize_learnings_aggregates_evidence_cleanly() {
        let mut success = entry();
        success.entry_key = "exp-2:variant-b".into();
        success.kind = LearningKind::Success;
        success.severity = FailureSeverity::None;
        success.impact_minor = 300_000;
        success.confidence_bps = 9_000;
        success.reusable_rule = "Use proof-first hook".into();

        let entries = vec![entry(), success];
        let synthesis = synthesize_learnings(&entries).unwrap();

        assert_eq!(synthesis.total_entries, 2);
        assert_eq!(synthesis.failures, 1);
        assert_eq!(synthesis.successes, 1);
        assert_eq!(synthesis.total_impact_minor, 180_000);
        assert_eq!(synthesis.high_confidence_rules_count, 2);
        assert_eq!(synthesis.reusable_rules.len(), 2);
    }
}
