use economic_core::CompanyStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentRole {
    Governor,
    CEO,
    CFO,
    COO,
    Growth,
    Content,
    Recruiter,
    Analyst,
    Experiment,
}

impl AgentRole {
    pub const ALL: [Self; 9] = [
        Self::Governor, Self::CEO, Self::CFO, Self::COO, Self::Growth,
        Self::Content, Self::Recruiter, Self::Analyst, Self::Experiment,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Governor => "Governor",
            Self::CEO => "CEO",
            Self::CFO => "CFO",
            Self::COO => "COO",
            Self::Growth => "Growth",
            Self::Content => "Content",
            Self::Recruiter => "Recruiter",
            Self::Analyst => "Analyst",
            Self::Experiment => "Experiment",
        }
    }

    pub fn allowed_actions(self) -> &'static [ActionKind] {
        match self {
            Self::Governor => &[],
            Self::CEO => &[ActionKind::AllocateExperimentBudget, ActionKind::ReduceBudget, ActionKind::ProduceReport],
            Self::CFO => &[ActionKind::ReduceBudget, ActionKind::ProduceReport],
            Self::COO => &[ActionKind::RebalanceOperations, ActionKind::ProduceReport],
            Self::Growth => &[ActionKind::CreateExperiment, ActionKind::ResearchOpportunity, ActionKind::ProduceReport],
            Self::Content => &[ActionKind::CreateExperiment, ActionKind::ResearchOpportunity, ActionKind::ProduceReport],
            Self::Recruiter => &[ActionKind::ProposeHire, ActionKind::ProduceReport],
            Self::Analyst => &[ActionKind::ProduceReport],
            Self::Experiment => &[ActionKind::CreateExperiment, ActionKind::ResearchOpportunity, ActionKind::ProduceReport],
        }
    }

    pub fn may_propose(self, action: ActionKind) -> bool {
        self.allowed_actions().contains(&action)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    Read,
    Propose,
    ExecuteLimited,
    ExecuteMaterial,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionKind {
    None,
    CreateExperiment,
    AllocateExperimentBudget,
    ReduceBudget,
    RebalanceOperations,
    ResearchOpportunity,
    PublishContent,
    ProposeHire,
    ProduceReport,
    EscalateIncident,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskTier {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernorDecision {
    Approve,
    Reject,
    RequestRevision,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanySnapshot {
    pub company_id: String,
    pub cash_minor: i128,
    pub revenue_minor: i128,
    pub expenses_minor: i128,
    pub liabilities_minor: i128,
    pub assets_minor: i128,
    pub runway_days: i64,
    pub status: CompanyStatus,
    pub budget_remaining_minor: i128,
    pub experiment_budget_minor: i128,
    pub content_cost_minor: i128,
    pub content_revenue_minor: i128,
    pub backlog: u32,
    pub capacity: u32,
    pub conversion_bps: u32,
    pub audience_growth_bps: i32,
    pub hiring_need: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSuggestion {
    pub action: Option<String>,
    pub objective: Option<String>,
    pub cost_minor: Option<i128>,
    pub expected_revenue_minor: Option<i128>,
    pub risk: Option<String>,
    pub confidence: Option<f64>,
    pub rationale: Option<String>,
    pub reversible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub agent: AgentRole,
    pub objective: String,
    pub action: ActionKind,
    pub cost_minor: i128,
    pub expected_revenue_minor: i128,
    pub risk: RiskTier,
    pub confidence_bps: u16,
    pub evidence: Vec<String>,
    pub rationale: String,
    pub reversible: bool,
    pub requested_permission: Permission,
}

impl Proposal {
    pub fn validate(&self) -> Result<(), String> {
        if self.objective.trim().is_empty() {
            return Err("objective is required".into());
        }
        if self.rationale.trim().is_empty() {
            return Err("rationale is required".into());
        }
        if self.evidence.is_empty() {
            return Err("at least one evidence item is required".into());
        }
        if self.action == ActionKind::None {
            return Err("action is required".into());
        }
        if self.cost_minor < 0 || self.expected_revenue_minor < 0 {
            return Err("economic values cannot be negative".into());
        }
        if self.confidence_bps > 10_000 {
            return Err("confidence must be <= 10000 basis points".into());
        }
        if self.requested_permission != Permission::Propose {
            return Err("agent proposals may request Propose permission only".into());
        }
        if !self.agent.may_propose(self.action) {
            return Err(format!("agent {} is not allowed to propose {:?}", self.agent.as_str(), self.action));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedProposal {
    pub proposal: Proposal,
    pub decision: GovernorDecision,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunResult {
    pub agent: AgentRole,
    pub proposal: Proposal,
    pub governance: Option<GovernedProposal>,
}


impl RiskTier {
    pub fn rank(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Critical => 3,
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self.rank() >= other.rank() { self } else { other }
    }
}

impl ActionKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "createexperiment" | "create_experiment" => Some(Self::CreateExperiment),
            "allocateexperimentbudget" | "allocate_experiment_budget" => Some(Self::AllocateExperimentBudget),
            "reducebudget" | "reduce_budget" => Some(Self::ReduceBudget),
            "rebalanceoperations" | "rebalance_operations" => Some(Self::RebalanceOperations),
            "researchopportunity" | "research_opportunity" => Some(Self::ResearchOpportunity),
            "publishcontent" | "publish_content" => Some(Self::PublishContent),
            "proposehire" | "propose_hire" => Some(Self::ProposeHire),
            "producereport" | "produce_report" => Some(Self::ProduceReport),
            "escalateincident" | "escalate_incident" => Some(Self::EscalateIncident),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

impl RiskTier {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
}
