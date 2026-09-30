#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DAY_SECONDS: i64 = 86_400;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BudgetKind {
    ContentPublish,
    AdsSpend,
    LiveMinutes,
    OutboundMessages,
    AutonomousCapital,
}

impl BudgetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ContentPublish => "CONTENT_PUBLISH",
            Self::AdsSpend => "ADS_SPEND",
            Self::LiveMinutes => "LIVE_MINUTES",
            Self::OutboundMessages => "OUTBOUND_MESSAGES",
            Self::AutonomousCapital => "AUTONOMOUS_CAPITAL",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "CONTENT_PUBLISH" => Some(Self::ContentPublish),
            "ADS_SPEND" => Some(Self::AdsSpend),
            "LIVE_MINUTES" => Some(Self::LiveMinutes),
            "OUTBOUND_MESSAGES" => Some(Self::OutboundMessages),
            "AUTONOMOUS_CAPITAL" => Some(Self::AutonomousCapital),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyBudgets {
    pub content_publish_daily: i128,
    pub ads_spend_daily_minor: i128,
    pub live_minutes_daily: i128,
    pub outbound_messages_daily: i128,
    pub autonomous_capital_daily_minor: i128,
}

impl Default for AutonomyBudgets {
    fn default() -> Self {
        Self {
            content_publish_daily: 10,
            ads_spend_daily_minor: 0,
            live_minutes_daily: 60,
            outbound_messages_daily: 100,
            autonomous_capital_daily_minor: 0,
        }
    }
}

impl AutonomyBudgets {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("content_publish_daily", self.content_publish_daily),
            ("ads_spend_daily_minor", self.ads_spend_daily_minor),
            ("live_minutes_daily", self.live_minutes_daily),
            ("outbound_messages_daily", self.outbound_messages_daily),
            ("autonomous_capital_daily_minor", self.autonomous_capital_daily_minor),
        ] {
            if value < 0 {
                return Err(format!("{name} cannot be negative"));
            }
        }
        Ok(())
    }

    pub fn limit(self, kind: BudgetKind) -> i128 {
        match kind {
            BudgetKind::ContentPublish => self.content_publish_daily,
            BudgetKind::AdsSpend => self.ads_spend_daily_minor,
            BudgetKind::LiveMinutes => self.live_minutes_daily,
            BudgetKind::OutboundMessages => self.outbound_messages_daily,
            BudgetKind::AutonomousCapital => self.autonomous_capital_daily_minor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmergencyStop {
    pub enabled: bool,
    pub reason: Option<String>,
    pub actor: String,
    pub changed_at_epoch: i64,
}

impl Default for EmergencyStop {
    fn default() -> Self {
        Self {
            enabled: false,
            reason: None,
            actor: "system-default".into(),
            changed_at_epoch: 1,
        }
    }
}

impl EmergencyStop {
    pub fn validate(&self) -> Result<(), String> {
        if self.actor.trim().is_empty() || self.actor.len() > 256 {
            return Err("emergency stop actor is invalid".into());
        }
        if self.changed_at_epoch <= 0 {
            return Err("emergency stop changed_at_epoch must be positive".into());
        }
        if let Some(reason) = &self.reason {
            if reason.trim().is_empty() || reason.len() > 1_024 {
                return Err("emergency stop reason is invalid".into());
            }
        }
        if self.enabled && self.reason.is_none() {
            return Err("enabled emergency stop requires a reason".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafetyControls {
    pub company_id: Uuid,
    pub emergency_stop: EmergencyStop,
    pub budgets: AutonomyBudgets,
    pub updated_at_epoch: i64,
}

impl SafetyControls {
    pub fn validate(&self) -> Result<(), String> {
        if self.company_id == Uuid::nil() {
            return Err("company_id is required".into());
        }
        self.emergency_stop.validate()?;
        self.budgets.validate()?;
        if self.updated_at_epoch <= 0 {
            return Err("updated_at_epoch must be positive".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BudgetDecision {
    pub kind: BudgetKind,
    pub period_start_epoch: i64,
    pub daily_limit: i128,
    pub used_before: i128,
    pub requested: i128,
    pub remaining_after: i128,
    pub allowed: bool,
    pub reason: String,
}

pub fn period_start_epoch(now_epoch: i64) -> Result<i64, String> {
    if now_epoch <= 0 {
        return Err("budget time must be positive".into());
    }
    Ok(now_epoch - now_epoch.rem_euclid(DAY_SECONDS))
}

pub fn decide_budget(
    controls: &SafetyControls,
    kind: BudgetKind,
    now_epoch: i64,
    used_before: i128,
    requested: i128,
) -> Result<BudgetDecision, String> {
    controls.validate()?;
    if used_before < 0 || requested <= 0 {
        return Err("budget usage must be non-negative and requested amount positive".into());
    }

    let period_start_epoch = period_start_epoch(now_epoch)?;
    let daily_limit = controls.budgets.limit(kind);
    if controls.emergency_stop.enabled {
        return Ok(BudgetDecision {
            kind,
            period_start_epoch,
            daily_limit,
            used_before,
            requested,
            remaining_after: daily_limit.saturating_sub(used_before).max(0),
            allowed: false,
            reason: "emergency stop blocks autonomous side effects".into(),
        });
    }

    let projected = used_before
        .checked_add(requested)
        .ok_or_else(|| "budget usage overflow".to_string())?;
    if projected > daily_limit {
        return Ok(BudgetDecision {
            kind,
            period_start_epoch,
            daily_limit,
            used_before,
            requested,
            remaining_after: daily_limit.saturating_sub(used_before).max(0),
            allowed: false,
            reason: "daily autonomy budget would be exceeded".into(),
        });
    }

    Ok(BudgetDecision {
        kind,
        period_start_epoch,
        daily_limit,
        used_before,
        requested,
        remaining_after: daily_limit - projected,
        allowed: true,
        reason: "daily autonomy budget remains within the configured hard cap".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controls() -> SafetyControls {
        SafetyControls {
            company_id: Uuid::new_v4(),
            emergency_stop: EmergencyStop {
                enabled: false,
                reason: None,
                actor: "test".into(),
                changed_at_epoch: 1_700_000_000,
            },
            budgets: AutonomyBudgets {
                content_publish_daily: 10,
                ads_spend_daily_minor: 1_000,
                live_minutes_daily: 60,
                outbound_messages_daily: 100,
                autonomous_capital_daily_minor: 500,
            },
            updated_at_epoch: 1_700_000_000,
        }
    }

    #[test]
    fn budget_period_is_utc_day_start() {
        assert_eq!(period_start_epoch(86_401).unwrap(), 86_400);
    }

    #[test]
    fn budget_allows_under_cap() {
        let decision =
            decide_budget(&controls(), BudgetKind::ContentPublish, 1_700_000_000, 3, 4).unwrap();
        assert!(decision.allowed);
        assert_eq!(decision.remaining_after, 3);
    }

    #[test]
    fn budget_rejects_over_cap() {
        let decision =
            decide_budget(&controls(), BudgetKind::LiveMinutes, 1_700_000_000, 50, 11).unwrap();
        assert!(!decision.allowed);
        assert_eq!(decision.remaining_after, 10);
    }

    #[test]
    fn emergency_stop_blocks_without_mutating_accounting() {
        let mut state = controls();
        state.emergency_stop = EmergencyStop {
            enabled: true,
            reason: Some("incident".into()),
            actor: "operator".into(),
            changed_at_epoch: 1_700_000_001,
        };
        let decision =
            decide_budget(&state, BudgetKind::AutonomousCapital, 1_700_000_000, 0, 1).unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn zero_limit_is_a_hard_block() {
        let mut state = controls();
        state.budgets.autonomous_capital_daily_minor = 0;
        let decision =
            decide_budget(&state, BudgetKind::AutonomousCapital, 1_700_000_000, 0, 1).unwrap();
        assert!(!decision.allowed);
        assert_eq!(decision.remaining_after, 0);
    }
}
