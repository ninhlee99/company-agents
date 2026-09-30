use economic_core::CompanyStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentRole {
    Governor,
    CEO,
    CFO,
    COO,
    ProductLead,
    Growth,
    Content,
    Recruiter,
    Analyst,
    Experiment,
    RiskOfficer,
    CustomerSuccess,
    TreasuryOfficer,
}

impl AgentRole {
    pub const ALL: [Self; 13] = [
        Self::Governor,
        Self::CEO,
        Self::CFO,
        Self::COO,
        Self::ProductLead,
        Self::Growth,
        Self::Content,
        Self::Recruiter,
        Self::Analyst,
        Self::Experiment,
        Self::RiskOfficer,
        Self::CustomerSuccess,
        Self::TreasuryOfficer,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Governor => "Governor",
            Self::CEO => "CEO",
            Self::CFO => "CFO",
            Self::COO => "COO",
            Self::ProductLead => "ProductLead",
            Self::Growth => "Growth",
            Self::Content => "Content",
            Self::Recruiter => "Recruiter",
            Self::Analyst => "Analyst",
            Self::Experiment => "Experiment",
            Self::RiskOfficer => "RiskOfficer",
            Self::CustomerSuccess => "CustomerSuccess",
            Self::TreasuryOfficer => "TreasuryOfficer",
        }
    }

    pub fn allowed_actions(self) -> &'static [ActionKind] {
        match self {
            Self::Governor => &[],
            Self::CEO => &[
                ActionKind::AllocateExperimentBudget,
                ActionKind::ReduceBudget,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::CFO => &[
                ActionKind::ReduceBudget,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::COO => &[
                ActionKind::RebalanceOperations,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::ProductLead => &[
                ActionKind::DevelopProduct,
                ActionKind::ResearchOpportunity,
                ActionKind::CreateExperiment,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::Growth => &[
                ActionKind::CreateExperiment,
                ActionKind::ResearchOpportunity,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::Content => &[
                ActionKind::CreateExperiment,
                ActionKind::ResearchOpportunity,
                ActionKind::PublishContent,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::Recruiter => &[
                ActionKind::ProposeHire,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::Analyst => &[ActionKind::ProduceReport, ActionKind::EscalateIncident],
            Self::Experiment => &[
                ActionKind::CreateExperiment,
                ActionKind::ResearchOpportunity,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::RiskOfficer => &[
                ActionKind::MitigateRisk,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::CustomerSuccess => &[
                ActionKind::ResolveSupportCase,
                ActionKind::OptimizeRetention,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
            Self::TreasuryOfficer => &[
                ActionKind::ReconcileTreasury,
                ActionKind::ReduceBudget,
                ActionKind::ProduceReport,
                ActionKind::EscalateIncident,
            ],
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
    MitigateRisk,
    DevelopProduct,
    ResolveSupportCase,
    OptimizeRetention,
    ReconcileTreasury,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentMemory {
    pub key: String,
    pub value: serde_json::Value,
    pub confidence_bps: u16,
    pub importance: u8,
    pub updated_at: String,
    pub expires_at: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedProposal {
    pub proposal: Proposal,
    pub decision: GovernorDecision,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRunResult {
    pub agent: AgentRole,
    pub proposal: Proposal,
    pub governance: Option<GovernedProposal>,
}

impl ActionKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "createexperiment" | "create_experiment" => Some(Self::CreateExperiment),
            "allocateexperimentbudget" | "allocate_experiment_budget" => {
                Some(Self::AllocateExperimentBudget)
            }
            "reducebudget" | "reduce_budget" => Some(Self::ReduceBudget),
            "rebalanceoperations" | "rebalance_operations" => Some(Self::RebalanceOperations),
            "researchopportunity" | "research_opportunity" => Some(Self::ResearchOpportunity),
            "publishcontent" | "publish_content" => Some(Self::PublishContent),
            "proposehire" | "propose_hire" => Some(Self::ProposeHire),
            "producereport" | "produce_report" => Some(Self::ProduceReport),
            "escalateincident" | "escalate_incident" => Some(Self::EscalateIncident),
            "mitigaterisk" | "mitigate_risk" => Some(Self::MitigateRisk),
            "developproduct" | "develop_product" => Some(Self::DevelopProduct),
            "resolvesupportcase" | "resolve_support_case" => Some(Self::ResolveSupportCase),
            "optimizeretention" | "optimize_retention" => Some(Self::OptimizeRetention),
            "reconciletreasury" | "reconcile_treasury" => Some(Self::ReconcileTreasury),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn max_cost_minor(self) -> i128 {
        match self {
            Self::CreateExperiment
            | Self::AllocateExperimentBudget
            | Self::ResearchOpportunity
            | Self::DevelopProduct => 500,
            Self::OptimizeRetention => 200,
            Self::ProposeHire => 1_000,
            Self::PublishContent
            | Self::ProduceReport
            | Self::ReduceBudget
            | Self::RebalanceOperations
            | Self::EscalateIncident
            | Self::MitigateRisk
            | Self::ResolveSupportCase
            | Self::ReconcileTreasury
            | Self::None => 0,
        }
    }

    pub fn minimum_risk(self) -> RiskTier {
        match self {
            Self::ProposeHire => RiskTier::High,
            Self::AllocateExperimentBudget
            | Self::CreateExperiment
            | Self::ResearchOpportunity
            | Self::PublishContent
            | Self::DevelopProduct => RiskTier::Medium,
            Self::ReduceBudget
            | Self::RebalanceOperations
            | Self::ProduceReport
            | Self::EscalateIncident
            | Self::MitigateRisk
            | Self::ResolveSupportCase
            | Self::OptimizeRetention
            | Self::ReconcileTreasury
            | Self::None => RiskTier::Low,
        }
    }

    pub fn inherently_material(self) -> bool {
        matches!(self, Self::ProposeHire | Self::PublishContent)
    }
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
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

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
        if self.cost_minor > self.action.max_cost_minor() {
            return Err("proposal cost exceeds action safety cap".into());
        }
        if self.confidence_bps > 10_000 {
            return Err("confidence must be <= 10000 basis points".into());
        }
        if self.requested_permission != Permission::Propose {
            return Err("agent proposals may request Propose permission only".into());
        }
        if !self.agent.may_propose(self.action) {
            return Err(format!(
                "agent {} is not allowed to propose {:?}",
                self.agent.as_str(),
                self.action
            ));
        }
        if self.risk.rank() < self.action.minimum_risk().rank() {
            return Err("proposal risk is below the action minimum".into());
        }
        Ok(())
    }
}
