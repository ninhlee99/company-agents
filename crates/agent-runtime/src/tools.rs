use crate::types::AgentRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    ReadCompany,
    ReadMetrics,
    ResearchOpportunity,
    CreateExperiment,
    AllocateExperimentBudget,
    ReduceBudget,
    RebalanceOperations,
    ProposeHire,
    ProduceReport,
    PublishContent,
    SendExternalMessage,
    InitiatePayment,
    EscalateIncident,
}

impl Tool {
    pub const ALL: [Self; 12] = [
        Self::ReadCompany,
        Self::ReadMetrics,
        Self::ResearchOpportunity,
        Self::CreateExperiment,
        Self::AllocateExperimentBudget,
        Self::ReduceBudget,
        Self::RebalanceOperations,
        Self::ProposeHire,
        Self::ProduceReport,
        Self::PublishContent,
        Self::SendExternalMessage,
        Self::InitiatePayment,
        Self::EscalateIncident,
    ];
}

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn for_action(action: crate::types::ActionKind) -> Option<Tool> {
        match action {
            crate::types::ActionKind::CreateExperiment => Some(Tool::CreateExperiment),
            crate::types::ActionKind::AllocateExperimentBudget => Some(Tool::AllocateExperimentBudget),
            crate::types::ActionKind::ReduceBudget => Some(Tool::ReduceBudget),
            crate::types::ActionKind::RebalanceOperations => Some(Tool::RebalanceOperations),
            crate::types::ActionKind::ResearchOpportunity => Some(Tool::ResearchOpportunity),
            crate::types::ActionKind::ProposeHire => Some(Tool::ProposeHire),
            crate::types::ActionKind::ProduceReport => Some(Tool::ProduceReport),
            crate::types::ActionKind::PublishContent => Some(Tool::PublishContent),
            crate::types::ActionKind::EscalateIncident => Some(Tool::EscalateIncident),
            crate::types::ActionKind::None => None,
        }
    }

    pub fn allowed(role: AgentRole, tool: Tool) -> bool {
        match role {
            AgentRole::Governor => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::EscalateIncident),
            AgentRole::CEO => matches!(
                tool,
                Tool::ReadCompany
                    | Tool::ReadMetrics
                    | Tool::ResearchOpportunity
                    | Tool::AllocateExperimentBudget
                    | Tool::ReduceBudget
                    | Tool::ProduceReport
                    | Tool::EscalateIncident
            ),
            AgentRole::CFO => matches!(
                tool,
                Tool::ReadCompany | Tool::ReadMetrics | Tool::ReduceBudget | Tool::ProduceReport
                    | Tool::EscalateIncident
            ),
            AgentRole::COO => matches!(
                tool,
                Tool::ReadCompany
                    | Tool::ReadMetrics
                    | Tool::RebalanceOperations
                    | Tool::ProduceReport
                    | Tool::EscalateIncident
            ),
            AgentRole::Growth => matches!(
                tool,
                Tool::ReadCompany
                    | Tool::ReadMetrics
                    | Tool::ResearchOpportunity
                    | Tool::CreateExperiment
                    | Tool::ProduceReport
            ),
            AgentRole::Content => matches!(
                tool,
                Tool::ReadCompany
                    | Tool::ReadMetrics
                    | Tool::ResearchOpportunity
                    | Tool::CreateExperiment
                    | Tool::PublishContent
                    | Tool::ProduceReport
            ),
            AgentRole::Recruiter => matches!(
                tool,
                Tool::ReadCompany | Tool::ReadMetrics | Tool::ProposeHire | Tool::ProduceReport
            ),
            AgentRole::Analyst => matches!(
                tool,
                Tool::ReadCompany | Tool::ReadMetrics | Tool::ProduceReport
            ),
            AgentRole::Experiment => matches!(
                tool,
                Tool::ReadCompany
                    | Tool::ReadMetrics
                    | Tool::ResearchOpportunity
                    | Tool::CreateExperiment
                    | Tool::ProduceReport
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_tools_are_denied_by_default() {
        for role in AgentRole::ALL {
            assert!(!ToolRegistry::allowed(role, Tool::InitiatePayment));
            assert!(!ToolRegistry::allowed(role, Tool::SendExternalMessage));
        }
    }

    #[test]
    fn capabilities_follow_role_boundaries() {
        assert!(ToolRegistry::allowed(AgentRole::Analyst, Tool::ReadMetrics));
        assert!(ToolRegistry::allowed(
            AgentRole::Analyst,
            Tool::ProduceReport
        ));
        assert!(!ToolRegistry::allowed(
            AgentRole::Analyst,
            Tool::ProposeHire
        ));
        assert!(ToolRegistry::allowed(
            AgentRole::Recruiter,
            Tool::ProposeHire
        ));
        assert!(!ToolRegistry::allowed(
            AgentRole::Recruiter,
            Tool::PublishContent
        ));
        assert!(ToolRegistry::allowed(
            AgentRole::Content,
            Tool::PublishContent
        ));
        assert!(!ToolRegistry::allowed(
            AgentRole::CFO,
            Tool::CreateExperiment
        ));
    }
}
