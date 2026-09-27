#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub type Minor = i128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatorStatus { Testing, Growing, Stable, Distress, Paused, Closed }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStatus { Draft, Active, Completed, Paused, Cancelled, Closed }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmploymentStatus { Proposed, Active, Leave, Terminated }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractStatus { Draft, Active, Expired, Terminated }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus { Backlog, Ready, Running, Blocked, Done, Cancelled }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus { Draft, Qa, Approved, Scheduled, Published, Archived }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatorUnit {
    pub id: Uuid,
    pub name: String,
    pub currency: String,
    pub status: CreatorStatus,
    pub cash_minor: Minor,
    pub revenue_minor: Minor,
    pub expenses_minor: Minor,
    pub audience: u64,
    pub content_count: u64,
}

impl CreatorUnit {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.name, "creator name")?;
        validate_currency(&self.currency)?;
        validate_nonnegative(self.cash_minor, "creator cash")?;
        validate_nonnegative(self.revenue_minor, "creator revenue")?;
        validate_nonnegative(self.expenses_minor, "creator expenses")?;
        Ok(())
    }

    pub fn contribution_margin_minor(&self) -> Minor {
        self.revenue_minor.saturating_sub(self.expenses_minor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentAsset {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub title: String,
    pub channel: String,
    pub status: ContentStatus,
    pub production_cost_minor: Minor,
    pub attributed_revenue_minor: Minor,
    pub affiliate_commission_minor: Minor,
    pub views: u64,
    pub clicks: u64,
    pub orders: u64,
    pub published_at_epoch: Option<i64>,
}

impl ContentAsset {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.title, "content title")?;
        validate_text(&self.channel, "content channel")?;
        for (name, value) in [
            ("production cost", self.production_cost_minor),
            ("attributed revenue", self.attributed_revenue_minor),
            ("affiliate commission", self.affiliate_commission_minor),
        ] {
            validate_nonnegative(value, name)?;
        }
        Ok(())
    }

    pub fn contribution_margin_minor(&self) -> Minor {
        self.attributed_revenue_minor.saturating_add(self.affiliate_commission_minor)
            .saturating_sub(self.production_cost_minor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Experiment {
    pub id: Uuid,
    pub name: String,
    pub hypothesis: String,
    pub status: LifecycleStatus,
    pub budget_minor: Minor,
    pub spent_minor: Minor,
    pub expected_revenue_minor: Minor,
}

impl Experiment {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.name, "experiment name")?;
        validate_text(&self.hypothesis, "experiment hypothesis")?;
        validate_nonnegative(self.budget_minor, "experiment budget")?;
        validate_nonnegative(self.spent_minor, "experiment spent")?;
        validate_nonnegative(self.expected_revenue_minor, "experiment expected revenue")?;
        if self.spent_minor > self.budget_minor {
            return Err(DomainError::Invariant("experiment spent exceeds budget"));
        }
        Ok(())
    }

    pub fn remaining_budget_minor(&self) -> Minor {
        self.budget_minor.saturating_sub(self.spent_minor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Employee {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub status: EmploymentStatus,
    pub monthly_cost_minor: Minor,
    pub start_epoch: Option<i64>,
}

impl Employee {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.name, "employee name")?;
        validate_text(&self.role, "employee role")?;
        validate_nonnegative(self.monthly_cost_minor, "employee monthly cost")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract {
    pub id: Uuid,
    pub counterparty: String,
    pub contract_type: String,
    pub status: ContractStatus,
    pub value_minor: Minor,
    pub start_epoch: i64,
    pub end_epoch: Option<i64>,
}

impl Contract {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.counterparty, "counterparty")?;
        validate_text(&self.contract_type, "contract type")?;
        validate_nonnegative(self.value_minor, "contract value")?;
        if let Some(end) = self.end_epoch {
            if end < self.start_epoch {
                return Err(DomainError::Invariant("contract end precedes contract start"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub priority: u8,
    pub owner_agent: Option<String>,
    pub creator_id: Option<Uuid>,
}

impl Task {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.title, "task title")?;
        if self.priority > 100 {
            return Err(DomainError::Invariant("task priority must be <= 100"));
        }
        if let Some(owner) = &self.owner_agent {
            validate_text(owner, "task owner")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    InvalidCurrency,
    InvalidText(&'static str),
    Negative(&'static str),
    Invariant(&'static str),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCurrency => write!(f, "currency must be a 3-letter uppercase code"),
            Self::InvalidText(field) => write!(f, "{field} is required"),
            Self::Negative(field) => write!(f, "{field} cannot be negative"),
            Self::Invariant(message) => write!(f, "{message}"),
        }
    }
}

fn validate_text(value: &str, field: &'static str) -> Result<(), DomainError> {
    if value.trim().is_empty() { Err(DomainError::InvalidText(field)) } else { Ok(()) }
}

fn validate_nonnegative(value: Minor, field: &'static str) -> Result<(), DomainError> {
    if value < 0 { Err(DomainError::Negative(field)) } else { Ok(()) }
}

fn validate_currency(currency: &str) -> Result<(), DomainError> {
    if currency.len() == 3 && currency.bytes().all(|b| b.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err(DomainError::InvalidCurrency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creator_margin_is_exact_and_saturating() {
        let creator = CreatorUnit {
            id: Uuid::new_v4(),
            name: "c".into(),
            currency: "USD".into(),
            status: CreatorStatus::Growing,
            cash_minor: 1,
            revenue_minor: 100,
            expenses_minor: 30,
            audience: 10,
            content_count: 1,
        };
        creator.validate().unwrap();
        assert_eq!(creator.contribution_margin_minor(), 70);
    }

    #[test]
    fn experiment_cannot_overspend() {
        let experiment = Experiment {
            id: Uuid::new_v4(),
            name: "x".into(),
            hypothesis: "h".into(),
            status: LifecycleStatus::Active,
            budget_minor: 100,
            spent_minor: 101,
            expected_revenue_minor: 1000,
        };
        assert!(experiment.validate().is_err());
    }

    #[test]
    fn invalid_currency_fails_closed() {
        let employee = Employee {
            id: Uuid::new_v4(),
            name: "e".into(),
            role: "r".into(),
            status: EmploymentStatus::Proposed,
            monthly_cost_minor: 1,
            start_epoch: None,
        };
        employee.validate().unwrap();
        let creator = CreatorUnit {
            id: Uuid::new_v4(),
            name: "c".into(),
            currency: "usd".into(),
            status: CreatorStatus::Testing,
            cash_minor: 0,
            revenue_minor: 0,
            expenses_minor: 0,
            audience: 0,
            content_count: 0,
        };
        assert!(creator.validate().is_err());
    }
}
