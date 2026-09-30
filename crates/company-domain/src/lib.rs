#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub type Minor = i128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompanyEventType {
    TrendDetected,
    ProductFound,
    ContentCreated,
    ContentPublished,
    ViewerSpike,
    CtrChanged,
    OrderCreated,
    OrderCancelled,
    CommissionVerified,
    GiftReceived,
    PolicyChanged,
    OutOfStock,
    RefundSpike,
    ExperimentCompleted,
    AffiliatePayoutSettled,
    AgentDecisionRecorded,
    AffiliateConversionReconciled,
    PublishIntentCompleted,
    PublishIntentApproved,
    LedgerTransactionCommitted,
    AutonomyAssessmentRecorded,
    AgentOutcomeEvidenceRecorded,
}

impl CompanyEventType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrendDetected => "TREND_DETECTED",
            Self::ProductFound => "PRODUCT_FOUND",
            Self::ContentCreated => "CONTENT_CREATED",
            Self::ContentPublished => "CONTENT_PUBLISHED",
            Self::ViewerSpike => "VIEWER_SPIKE",
            Self::CtrChanged => "CTR_CHANGED",
            Self::OrderCreated => "ORDER_CREATED",
            Self::OrderCancelled => "ORDER_CANCELLED",
            Self::CommissionVerified => "COMMISSION_VERIFIED",
            Self::GiftReceived => "GIFT_RECEIVED",
            Self::PolicyChanged => "POLICY_CHANGED",
            Self::OutOfStock => "OUT_OF_STOCK",
            Self::RefundSpike => "REFUND_SPIKE",
            Self::ExperimentCompleted => "EXPERIMENT_COMPLETED",
            Self::AffiliatePayoutSettled => "AFFILIATE_PAYOUT_SETTLED",
            Self::AgentDecisionRecorded => "AGENT_DECISION_RECORDED",
            Self::AffiliateConversionReconciled => "AFFILIATE_CONVERSION_RECONCILED",
            Self::PublishIntentCompleted => "PUBLISH_INTENT_COMPLETED",
            Self::PublishIntentApproved => "PUBLISH_INTENT_APPROVED",
            Self::LedgerTransactionCommitted => "LEDGER_TRANSACTION_COMMITTED",
            Self::AutonomyAssessmentRecorded => "AUTONOMY_ASSESSMENT_RECORDED",
            Self::AgentOutcomeEvidenceRecorded => "AGENT_OUTCOME_EVIDENCE_RECORDED",
            Self::LearningEntryRecorded => "LEARNING_ENTRY_RECORDED",
            Self::AgentOutcomeEvidenceRecorded => "AGENT_OUTCOME_EVIDENCE_RECORDED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyEventEnvelope {
    pub event_id: Uuid,
    pub company_id: Uuid,
    pub event_type: CompanyEventType,
    pub schema_version: i32,
    pub aggregate_type: String,
    pub aggregate_id: Option<Uuid>,
    pub occurred_at_epoch: i64,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub idempotency_key: String,
    pub payload: serde_json::Value,
}

impl CompanyEventEnvelope {
    pub fn new(
        company_id: Uuid,
        event_type: CompanyEventType,
        aggregate_type: impl Into<String>,
        aggregate_id: Option<Uuid>,
        occurred_at_epoch: i64,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        idempotency_key: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<Self, DomainError> {
        let event = Self {
            event_id: Uuid::new_v4(),
            company_id,
            event_type,
            schema_version: 1,
            aggregate_type: aggregate_type.into(),
            aggregate_id,
            occurred_at_epoch,
            correlation_id,
            causation_id,
            idempotency_key: idempotency_key.into(),
            payload,
        };
        event.validate()?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.schema_version < 1 {
            return Err(DomainError::Invariant("event schema version must be >= 1"));
        }
        validate_text(&self.aggregate_type, "event aggregate type")?;
        validate_text(&self.idempotency_key, "event idempotency key")?;
        if self.idempotency_key.len() > 512 {
            return Err(DomainError::Invariant("event idempotency key must be <= 512 bytes"));
        }
        if self.occurred_at_epoch < 0 {
            return Err(DomainError::Invariant("event occurred_at must be >= 0"));
        }
        if self.payload.is_null() {
            return Err(DomainError::Invariant("event payload cannot be null"));
        }
        Ok(())
    }

    pub fn event_type_name(&self) -> &'static str {
        self.event_type.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatorStatus {
    Testing,
    Growing,
    Stable,
    Distress,
    Paused,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStatus {
    Draft,
    Active,
    Completed,
    Paused,
    Cancelled,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmploymentStatus {
    Proposed,
    Active,
    Leave,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractStatus {
    Draft,
    Active,
    Expired,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Backlog,
    Ready,
    Running,
    Blocked,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus {
    Draft,
    Qa,
    Approved,
    Scheduled,
    Published,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityStatus {
    Active,
    Paused,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessUnit {
    pub id: Uuid,
    pub name: String,
    pub currency: String,
    pub status: EntityStatus,
    pub cash_minor: Minor,
    pub revenue_minor: Minor,
    pub expenses_minor: Minor,
}

impl BusinessUnit {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.name, "business unit name")?;
        validate_currency(&self.currency)?;
        validate_nonnegative(self.cash_minor, "business unit cash")?;
        validate_nonnegative(self.revenue_minor, "business unit revenue")?;
        validate_nonnegative(self.expenses_minor, "business unit expenses")?;
        Ok(())
    }

    pub fn contribution_margin_minor(&self) -> Minor {
        self.revenue_minor.saturating_sub(self.expenses_minor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub external_ref: Option<String>,
    pub status: EntityStatus,
    pub lifetime_revenue_minor: Minor,
}

impl Customer {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.name, "customer name")?;
        validate_nonnegative(self.lifetime_revenue_minor, "customer lifetime revenue")?;
        if self
            .external_ref
            .as_ref()
            .is_some_and(|v| v.trim().is_empty())
        {
            return Err(DomainError::InvalidText("customer external ref"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub currency: String,
    pub price_minor: Minor,
    pub active: bool,
}

impl Product {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_text(&self.name, "product name")?;
        validate_text(&self.category, "product category")?;
        validate_currency(&self.currency)?;
        validate_nonnegative(self.price_minor, "product price")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayrollLine {
    pub employee_id: Uuid,
    pub gross_minor: Minor,
    pub employer_cost_minor: Minor,
    pub withholding_minor: Minor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayrollRun {
    pub id: Uuid,
    pub period_start_epoch: i64,
    pub period_end_epoch: i64,
    pub lines: Vec<PayrollLine>,
}

impl PayrollRun {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.period_end_epoch < self.period_start_epoch {
            return Err(DomainError::Invariant("payroll period end precedes start"));
        }
        if self.lines.is_empty() {
            return Err(DomainError::Invariant(
                "payroll must contain at least one employee",
            ));
        }
        for line in &self.lines {
            if line.gross_minor < 0 || line.employer_cost_minor < 0 || line.withholding_minor < 0 {
                return Err(DomainError::Invariant("payroll amounts cannot be negative"));
            }
            if line.withholding_minor > line.gross_minor {
                return Err(DomainError::Invariant("withholding exceeds gross payroll"));
            }
        }
        Ok(())
    }

    pub fn gross_minor(&self) -> Minor {
        self.lines
            .iter()
            .fold(0, |a, l| a.saturating_add(l.gross_minor))
    }

    pub fn employer_cost_minor(&self) -> Minor {
        self.lines
            .iter()
            .fold(0, |a, l| a.saturating_add(l.employer_cost_minor))
    }

    pub fn cash_due_minor(&self) -> Minor {
        self.lines.iter().fold(0, |a, l| {
            a.saturating_add(l.gross_minor.saturating_sub(l.withholding_minor))
        })
    }
}

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
        self.attributed_revenue_minor
            .saturating_add(self.affiliate_commission_minor)
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
                return Err(DomainError::Invariant(
                    "contract end precedes contract start",
                ));
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
    if value.trim().is_empty() {
        Err(DomainError::InvalidText(field))
    } else {
        Ok(())
    }
}

fn validate_nonnegative(value: Minor, field: &'static str) -> Result<(), DomainError> {
    if value < 0 {
        Err(DomainError::Negative(field))
    } else {
        Ok(())
    }
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
    #[test]
    fn company_event_contract_has_stable_names_and_validation() {
        assert_eq!(CompanyEventType::CtrChanged.as_str(), "CTR_CHANGED");
        assert_eq!(CompanyEventType::OrderCancelled.as_str(), "ORDER_CANCELLED");
        let company_id = Uuid::new_v4();
        let event = CompanyEventEnvelope::new(
            company_id,
            CompanyEventType::OrderCreated,
            "order",
            Some(Uuid::new_v4()),
            1_800_000_000,
            Uuid::new_v4(),
            None,
            "order-created-1",
            serde_json::json!({"order_minor": 2500}),
        )
        .unwrap();
        assert_eq!(event.company_id, company_id);
        assert_eq!(event.schema_version, 1);
        assert_eq!(event.event_type_name(), "ORDER_CREATED");
        assert!(event.validate().is_ok());
    }

    #[test]
    fn event_serialization_uses_canonical_names() {
        let value = serde_json::to_value(CompanyEventType::ProductFound).unwrap();
        assert_eq!(value, serde_json::json!("PRODUCT_FOUND"));
    }

    #[test]
    fn company_event_rejects_invalid_metadata() {
        let event = CompanyEventEnvelope {
            event_id: Uuid::new_v4(),
            company_id: Uuid::new_v4(),
            event_type: CompanyEventType::GiftReceived,
            schema_version: 0,
            aggregate_type: "".into(),
            aggregate_id: None,
            occurred_at_epoch: -1,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            idempotency_key: "".into(),
            payload: serde_json::Value::Null,
        };
        assert!(event.validate().is_err());
    }

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
