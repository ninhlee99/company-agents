use crate::types::*;

pub struct Governor;

impl Governor {
    pub fn evaluate(&self, proposal: Proposal, company: &CompanySnapshot) -> GovernedProposal {
        if proposal.agent == AgentRole::Governor {
            return GovernedProposal {
                proposal,
                decision: GovernorDecision::Reject,
                reason: "governor cannot govern its own authority".into(),
            };
        }

        if matches!(company.status, economic_core::CompanyStatus::Bankrupt | economic_core::CompanyStatus::Liquidation)
            && proposal.action != ActionKind::ProduceReport
            && proposal.action != ActionKind::EscalateIncident
        {
            return GovernedProposal {
                proposal,
                decision: GovernorDecision::Reject,
                reason: "company state blocks discretionary execution".into(),
            };
        }

        if proposal.cost_minor < 0 || proposal.expected_revenue_minor < 0 {
            return GovernedProposal {
                proposal,
                decision: GovernorDecision::Reject,
                reason: "negative economic values are invalid".into(),
            };
        }

        if proposal.cost_minor > company.budget_remaining_minor {
            return GovernedProposal {
                proposal,
                decision: GovernorDecision::Reject,
                reason: "proposal exceeds remaining budget".into(),
            };
        }

        if proposal.confidence_bps < 3_000 {
            return GovernedProposal {
                proposal,
                decision: GovernorDecision::RequestRevision,
                reason: "confidence is below minimum proposal threshold".into(),
            };
        }

        let material = matches!(proposal.risk, RiskTier::High | RiskTier::Critical)
            || proposal.cost_minor > 1_000
            || !proposal.reversible;

        let decision = if material {
            GovernorDecision::Escalate
        } else {
            GovernorDecision::Approve
        };

        let reason = if material {
            "material or irreversible action requires explicit authorization".into()
        } else {
            "proposal satisfies bounded autonomy policy".into()
        };

        GovernedProposal { proposal, decision, reason }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> CompanySnapshot {
        CompanySnapshot {
            company_id: "c1".into(),
            cash_minor: 10_000,
            revenue_minor: 2_000,
            expenses_minor: 1_000,
            liabilities_minor: 0,
            assets_minor: 10_000,
            runway_days: 90,
            status: economic_core::CompanyStatus::Active,
            budget_remaining_minor: 2_000,
            experiment_budget_minor: 500,
            content_cost_minor: 100,
            content_revenue_minor: 200,
            backlog: 1,
            capacity: 2,
            conversion_bps: 300,
            audience_growth_bps: 100,
            hiring_need: 0,
        }
    }

    #[test]
    fn governor_approves_small_reversible_action() {
        let p = Proposal {
            agent: AgentRole::Experiment,
            objective: "test".into(),
            action: ActionKind::CreateExperiment,
            cost_minor: 100,
            expected_revenue_minor: 200,
            risk: RiskTier::Low,
            confidence_bps: 7000,
            evidence: vec!["bounded".into()],
            rationale: "small test".into(),
            reversible: true,
            requested_permission: Permission::Propose,
        };
        assert_eq!(Governor.evaluate(&Governor, p, &snapshot()).decision, GovernorDecision::Approve);
    }

    #[test]
    fn governor_escalates_irreversible_action() {
        let p = Proposal {
            agent: AgentRole::Recruiter,
            objective: "hire".into(),
            action: ActionKind::ProposeHire,
            cost_minor: 500,
            expected_revenue_minor: 1200,
            risk: RiskTier::High,
            confidence_bps: 7000,
            evidence: vec!["need".into()],
            rationale: "hire".into(),
            reversible: false,
            requested_permission: Permission::Propose,
        };
        assert_eq!(Governor.evaluate(&Governor, p, &snapshot()).decision, GovernorDecision::Escalate);
    }
}
