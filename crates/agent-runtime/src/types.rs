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
