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
    EscalateIncident,
    SendExternalMessage,
    InitiatePayment,
}

impl Tool {
    pub const ALL: [Self; 12] = [
        Self::ReadCompany, Self::ReadMetrics, Self::ResearchOpportunity,
        Self::CreateExperiment, Self::AllocateExperimentBudget, Self::ReduceBudget,
        Self::RebalanceOperations, Self::ProposeHire, Self::ProduceReport,
        Self::PublishContent, Self::EscalateIncident, Self::SendExternalMessage, Self::InitiatePayment,
    ];
}

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn for_action(action: crate::types::ActionKind) -> Option<Tool> {
        use crate::types::ActionKind;
        Some(match action {
            ActionKind::CreateExperiment => Tool::CreateExperiment,
            ActionKind::AllocateExperimentBudget => Tool::AllocateExperimentBudget,
            ActionKind::ReduceBudget => Tool::ReduceBudget,
            ActionKind::RebalanceOperations => Tool::RebalanceOperations,
            ActionKind::ResearchOpportunity => Tool::ResearchOpportunity,
            ActionKind::PublishContent => Tool::PublishContent,
            ActionKind::ProposeHire => Tool::ProposeHire,
            ActionKind::ProduceReport => Tool::ProduceReport,
            ActionKind::EscalateIncident => Tool::EscalateIncident,
            ActionKind::None => return None,
        })
    }

    pub fn allowed(role: AgentRole, tool: Tool) -> bool {
        match role {
            AgentRole::Governor => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::EscalateIncident),
            AgentRole::CEO => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::ResearchOpportunity | Tool::AllocateExperimentBudget | Tool::ReduceBudget | Tool::ProduceReport | Tool::EscalateIncident),
            AgentRole::CFO => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::ReduceBudget | Tool::ProduceReport | Tool::EscalateIncident),
            AgentRole::COO => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::RebalanceOperations | Tool::ProduceReport | Tool::EscalateIncident),
            AgentRole::Growth => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::ResearchOpportunity | Tool::CreateExperiment | Tool::ProduceReport | Tool::EscalateIncident),
            AgentRole::Content => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::ResearchOpportunity | Tool::CreateExperiment | Tool::PublishContent | Tool::ProduceReport | Tool::EscalateIncident),
            AgentRole::Recruiter => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::ProposeHire | Tool::ProduceReport | Tool::EscalateIncident),
            AgentRole::Analyst => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::ProduceReport | Tool::EscalateIncident),
            AgentRole::Experiment => matches!(tool, Tool::ReadCompany | Tool::ReadMetrics | Tool::ResearchOpportunity | Tool::CreateExperiment | Tool::ProduceReport | Tool::EscalateIncident),
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
    fn action_tool_bindings_exist_for_executable_actions() {
        for action in [
            ActionKind::CreateExperiment,
            ActionKind::AllocateExperimentBudget,
            ActionKind::ReduceBudget,
            ActionKind::RebalanceOperations,
            ActionKind::ResearchOpportunity,
            ActionKind::PublishContent,
            ActionKind::ProposeHire,
            ActionKind::ProduceReport,
            ActionKind::EscalateIncident,
        ] {
            assert!(ToolRegistry::for_action(action).is_some());
        }
        assert!(ToolRegistry::for_action(ActionKind::None).is_none());
    }

    #[test]
    fn capabilities_follow_role_boundaries() {
        assert!(ToolRegistry::allowed(AgentRole::Analyst, Tool::ReadMetrics));
        assert!(ToolRegistry::allowed(AgentRole::Analyst, Tool::ProduceReport));
        assert!(!ToolRegistry::allowed(AgentRole::Analyst, Tool::ProposeHire));
        assert!(ToolRegistry::allowed(AgentRole::Recruiter, Tool::ProposeHire));
        assert!(!ToolRegistry::allowed(AgentRole::Recruiter, Tool::PublishContent));
        assert!(ToolRegistry::allowed(AgentRole::Content, Tool::PublishContent));
        assert!(!ToolRegistry::allowed(AgentRole::CFO, Tool::CreateExperiment));
    }
}
