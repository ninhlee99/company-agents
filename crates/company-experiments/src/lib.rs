#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExperimentStatus {
    Proposed,
    Running,
    Succeeded,
    Failed,
    Killed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExperimentSpec {
    pub hypothesis: String,
    pub control: String,
    pub treatment: String,
    pub max_budget_minor: i128,
    pub min_observations: u64,
    pub duration_seconds: u64,
    pub success_metric_bps: i64,
    pub kill_metric_bps: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExperimentObservation {
    pub control_observations: u64,
    pub treatment_observations: u64,
    pub control_metric_bps: i64,
    pub treatment_metric_bps: i64,
    pub spend_minor: i128,
    pub elapsed_seconds: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperimentDecision {
    Continue,
    Succeed,
    Kill,
    Expire,
}

pub fn validate_spec(spec: &ExperimentSpec) -> Result<(), String> {
    if spec.hypothesis.trim().is_empty() || spec.control.trim().is_empty() || spec.treatment.trim().is_empty() {
        return Err("experiment hypothesis, control and treatment are required".into());
    }
    if spec.max_budget_minor <= 0 {
        return Err("experiment budget must be positive".into());
    }
    if spec.min_observations == 0 || spec.duration_seconds == 0 {
        return Err("experiment observation and duration limits must be positive".into());
    }
    if !(0..=10_000).contains(&spec.success_metric_bps) || !(0..=10_000).contains(&spec.kill_metric_bps) {
        return Err("experiment thresholds must be between 0 and 10000 bps".into());
    }
    if spec.kill_metric_bps > spec.success_metric_bps {
        return Err("kill threshold cannot exceed success threshold".into());
    }
    Ok(())
}

pub fn decide(spec: &ExperimentSpec, observation: &ExperimentObservation) -> Result<ExperimentDecision, String> {
    validate_spec(spec)?;
    if observation.spend_minor < 0 {
        return Err("experiment spend cannot be negative".into());
    }
    if observation.spend_minor >= spec.max_budget_minor {
        return Ok(ExperimentDecision::Kill);
    }
    if observation.elapsed_seconds >= spec.duration_seconds {
        return Ok(ExperimentDecision::Expire);
    }
    let min_obs = observation.control_observations.min(observation.treatment_observations);
    if min_obs < spec.min_observations {
        return Ok(ExperimentDecision::Continue);
    }

    let lift_bps = observation.treatment_metric_bps - observation.control_metric_bps;
    if lift_bps >= spec.success_metric_bps {
        Ok(ExperimentDecision::Succeed)
    } else if lift_bps <= -spec.kill_metric_bps {
        Ok(ExperimentDecision::Kill)
    } else {
        Ok(ExperimentDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ExperimentSpec {
        ExperimentSpec {
            hypothesis: "short hook improves conversion".into(),
            control: "baseline".into(),
            treatment: "short-hook".into(),
            max_budget_minor: 1_000,
            min_observations: 100,
            duration_seconds: 86_400,
            success_metric_bps: 500,
            kill_metric_bps: 300,
        }
    }

    #[test]
    fn continues_until_minimum_observations() {
        let s = spec();
        let o = ExperimentObservation { control_observations: 10, treatment_observations: 100, control_metric_bps: 500, treatment_metric_bps: 900, spend_minor: 0, elapsed_seconds: 1 };
        assert_eq!(decide(&s, &o).unwrap(), ExperimentDecision::Continue);
    }

    #[test]
    fn succeeds_when_lift_hits_threshold() {
        let s = spec();
        let o = ExperimentObservation { control_observations: 100, treatment_observations: 100, control_metric_bps: 500, treatment_metric_bps: 1000, spend_minor: 0, elapsed_seconds: 1 };
        assert_eq!(decide(&s, &o).unwrap(), ExperimentDecision::Succeed);
    }

    #[test]
    fn kills_on_budget_or_bad_lift() {
        let s = spec();
        let budget = ExperimentObservation { control_observations: 100, treatment_observations: 100, control_metric_bps: 500, treatment_metric_bps: 900, spend_minor: 1000, elapsed_seconds: 1 };
        assert_eq!(decide(&s, &budget).unwrap(), ExperimentDecision::Kill);
        let bad = ExperimentObservation { control_observations: 100, treatment_observations: 100, control_metric_bps: 500, treatment_metric_bps: 100, spend_minor: 0, elapsed_seconds: 1 };
        assert_eq!(decide(&s, &bad).unwrap(), ExperimentDecision::Kill);
    }

    #[test]
    fn expires_after_duration() {
        let s = spec();
        let o = ExperimentObservation { control_observations: 100, treatment_observations: 100, control_metric_bps: 500, treatment_metric_bps: 600, spend_minor: 0, elapsed_seconds: 86_400 };
        assert_eq!(decide(&s, &o).unwrap(), ExperimentDecision::Expire);
    }

    #[test]
    fn rejects_invalid_spec() {
        let mut s = spec();
        s.max_budget_minor = 0;
        assert!(validate_spec(&s).is_err());
    }
}
