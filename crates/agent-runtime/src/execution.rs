use crate::{
    governor::Governor,
    tools::ToolRegistry,
    types::{ActionKind, AgentRunResult, CompanySnapshot, GovernedProposal, GovernorDecision},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub agent: crate::types::AgentRole,
    pub action: ActionKind,
    pub decision: GovernorDecision,
    pub executed: bool,
    pub state_changed: bool,
    pub cost_minor: i128,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidSnapshot(String),
    Arithmetic(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSnapshot(e) => write!(f, "invalid company snapshot: {e}"),
            Self::Arithmetic(e) => write!(f, "arithmetic error: {e}"),
        }
    }
}

pub struct ExecutionEngine {
    governor: Governor,
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self { governor: Governor }
    }
}

impl ExecutionEngine {
    pub fn validate_snapshot(snapshot: &CompanySnapshot) -> Result<(), ExecutionError> {
        if snapshot.company_id.trim().is_empty() {
            return Err(ExecutionError::InvalidSnapshot("company id is required".into()));
        }
        for (name, value) in [
            ("cash_minor", snapshot.cash_minor),
            ("revenue_minor", snapshot.revenue_minor),
            ("expenses_minor", snapshot.expenses_minor),
            ("liabilities_minor", snapshot.liabilities_minor),
            ("assets_minor", snapshot.assets_minor),
            ("budget_remaining_minor", snapshot.budget_remaining_minor),
            ("experiment_budget_minor", snapshot.experiment_budget_minor),
            ("content_cost_minor", snapshot.content_cost_minor),
            ("content_revenue_minor", snapshot.content_revenue_minor),
        ] {
            if value < 0 {
                return Err(ExecutionError::InvalidSnapshot(format!("{name} cannot be negative")));
            }
        }
        if snapshot.runway_days < 0 {
            return Err(ExecutionError::InvalidSnapshot("runway_days cannot be negative".into()));
        }
        if snapshot.conversion_bps > 10_000 {
            return Err(ExecutionError::InvalidSnapshot("conversion_bps must be <= 10000".into()));
        }
        Ok(())
    }

    pub fn execute_batch(
        &self,
        snapshot: &mut CompanySnapshot,
        results: &mut [AgentRunResult],
    ) -> Vec<ExecutionOutcome> {
        let mut outcomes = Vec::with_capacity(results.len());

        if let Err(error) = Self::validate_snapshot(snapshot) {
            for result in results.iter() {
                outcomes.push(ExecutionOutcome {
                    agent: result.agent,
                    action: result.proposal.action,
                    decision: GovernorDecision::Reject,
                    executed: false,
                    state_changed: false,
                    cost_minor: 0,
                    reason: error.to_string(),
                });
            }
            return outcomes;
        }

        for result in results.iter_mut() {
            // Re-govern against the latest state. Parallel proposals must never
            // be allowed to oversubscribe the same cash/budget snapshot.
            let governed = self.governor.evaluate(result.proposal.clone(), snapshot);
            result.governance = Some(governed.clone());
            let outcome = self.execute_one(snapshot, &governed);
            outcomes.push(outcome);
        }
        outcomes
    }

    fn execute_one(
        &self,
        snapshot: &mut CompanySnapshot,
        governed: &GovernedProposal,
    ) -> ExecutionOutcome {
        let proposal = &governed.proposal;
        if let Some(tool) = ToolRegistry::for_action(proposal.action) {
            if !ToolRegistry::allowed(proposal.agent, tool) {
                return ExecutionOutcome {
                    agent: proposal.agent,
                    action: proposal.action,
                    decision: GovernorDecision::Reject,
                    executed: false,
                    state_changed: false,
                    cost_minor: 0,
                    reason: "least-privilege tool registry denied the action".into(),
                };
            }
        } else {
            return ExecutionOutcome {
                agent: proposal.agent,
                action: proposal.action,
                decision: GovernorDecision::Reject,
                executed: false,
                state_changed: false,
                cost_minor: 0,
                reason: "action has no executable tool binding".into(),
            };
        }

        if governed.decision != GovernorDecision::Approve {
            return ExecutionOutcome {
                agent: proposal.agent,
                action: proposal.action,
                decision: governed.decision,
                executed: false,
                state_changed: false,
                cost_minor: 0,
                reason: format!("not executed: {}", governed.reason),
            };
        }

        let mut state_changed = false;
        let mut cost_minor = 0_i128;
        let result = match proposal.action {
            ActionKind::AllocateExperimentBudget => {
                if proposal.cost_minor <= 0 {
                    Ok("no allocation requested")
                } else if proposal.cost_minor > snapshot.budget_remaining_minor {
                    Err("allocation exceeds remaining budget")
                } else {
                    snapshot.budget_remaining_minor -= proposal.cost_minor;
                    snapshot.experiment_budget_minor = match snapshot
                        .experiment_budget_minor
                        .checked_add(proposal.cost_minor)
                    {
                        Some(value) => value,
                        None => return arithmetic_failure(proposal, "experiment budget overflow"),
                    };
                    state_changed = true;
                    cost_minor = proposal.cost_minor;
                    Ok("experiment budget allocated")
                }
            }
            ActionKind::CreateExperiment => {
                if proposal.cost_minor <= 0 {
                    Ok("zero-cost experiment")
                } else if proposal.cost_minor > snapshot.experiment_budget_minor {
                    Err("experiment exceeds experiment budget")
                } else if proposal.cost_minor > snapshot.budget_remaining_minor {
                    Err("experiment exceeds remaining company budget")
                } else if proposal.cost_minor > snapshot.cash_minor {
                    Err("experiment exceeds available cash")
                } else {
                    snapshot.cash_minor -= proposal.cost_minor;
                    snapshot.expenses_minor = match snapshot.expenses_minor.checked_add(proposal.cost_minor) {
                        Some(value) => value,
                        None => return arithmetic_failure(proposal, "expense overflow"),
                    };
                    snapshot.experiment_budget_minor -= proposal.cost_minor;
                    snapshot.budget_remaining_minor -= proposal.cost_minor;
                    state_changed = true;
                    cost_minor = proposal.cost_minor;
                    Ok("experiment executed")
                }
            }
            ActionKind::RebalanceOperations => {
                let before = snapshot.backlog;
                snapshot.backlog = snapshot.backlog.saturating_sub(2);
                state_changed = snapshot.backlog != before;
                Ok("operations rebalanced")
            }
            ActionKind::ReduceBudget => {
                Ok("budget reduction recorded; no hidden spend")
            }
            ActionKind::ResearchOpportunity => {
                Ok("research decision recorded; no external side effect")
            }
            ActionKind::ProduceReport => {
                Ok("report decision recorded; no external side effect")
            }
            ActionKind::EscalateIncident => {
                Ok("incident escalation recorded; no external side effect")
            }
            ActionKind::PublishContent => {
                Ok("publish is material and should already have been escalated")
            }
            ActionKind::ProposeHire => {
                Ok("hire is material and should already have been escalated")
            }
            ActionKind::None => Err("no-op action is invalid for execution"),
        };

        match result {
            Ok(reason) => ExecutionOutcome {
                agent: proposal.agent,
                action: proposal.action,
                decision: governed.decision,
                executed: true,
                state_changed,
                cost_minor,
                reason: reason.into(),
            },
            Err(reason) => ExecutionOutcome {
                agent: proposal.agent,
                action: proposal.action,
                decision: GovernorDecision::Reject,
                executed: false,
                state_changed: false,
                cost_minor: 0,
                reason: reason.into(),
            },
        }
    }
}

fn arithmetic_failure(
    proposal: &crate::types::Proposal,
    reason: &str,
) -> ExecutionOutcome {
    ExecutionOutcome {
        agent: proposal.agent,
        action: proposal.action,
        decision: GovernorDecision::Reject,
        executed: false,
        state_changed: false,
        cost_minor: 0,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ActionKind, AgentRole, CompanySnapshot, GovernorDecision, Permission, Proposal, RiskTier};

    fn snapshot() -> CompanySnapshot {
        CompanySnapshot {
            company_id: "c1".into(),
            cash_minor: 1_000,
            revenue_minor: 2_000,
            expenses_minor: 500,
            liabilities_minor: 0,
            assets_minor: 1_000,
            runway_days: 30,
            status: economic_core::CompanyStatus::Active,
            budget_remaining_minor: 400,
            experiment_budget_minor: 200,
            content_cost_minor: 10,
            content_revenue_minor: 20,
            backlog: 8,
            capacity: 5,
            conversion_bps: 300,
            audience_growth_bps: 50,
            hiring_need: 0,
        }
    }

    fn result(agent: AgentRole, action: ActionKind, cost: i128) -> AgentRunResult {
        AgentRunResult {
            agent,
            proposal: Proposal {
                agent,
                objective: "bounded".into(),
                action,
                cost_minor: cost,
                expected_revenue_minor: 100,
                risk: action.minimum_risk(),
                confidence_bps: 9_000,
                evidence: vec!["test".into()],
                rationale: "test".into(),
                reversible: true,
                requested_permission: Permission::Propose,
            },
            governance: None,
        }
    }

    #[test]
    fn allocation_moves_budget_without_creating_revenue() {
        let mut s = snapshot();
        let mut results = vec![result(AgentRole::CEO, ActionKind::AllocateExperimentBudget, 100)];
        let outcomes = ExecutionEngine::default().execute_batch(&mut s, &mut results);
        assert_eq!(outcomes[0].decision, GovernorDecision::Approve);
        assert!(outcomes[0].executed);
        assert_eq!(s.budget_remaining_minor, 300);
        assert_eq!(s.experiment_budget_minor, 300);
        assert_eq!(s.cash_minor, 1_000);
    }

    #[test]
    fn sequential_regovern_prevents_experiment_budget_overspend() {
        let mut s = snapshot();
        s.experiment_budget_minor = 100;
        let mut results = vec![
            result(AgentRole::Growth, ActionKind::CreateExperiment, 80),
            result(AgentRole::Experiment, ActionKind::CreateExperiment, 80),
        ];
        let outcomes = ExecutionEngine::default().execute_batch(&mut s, &mut results);
        assert!(outcomes[0].executed);
        assert!(!outcomes[1].executed);
        assert_eq!(s.experiment_budget_minor, 20);
        assert_eq!(s.cash_minor, 920);
    }

    #[test]
    fn rejected_and_escalated_actions_have_no_side_effect() {
        let mut s = snapshot();
        let mut results = vec![
            result(AgentRole::Recruiter, ActionKind::ProposeHire, 500),
            result(AgentRole::Content, ActionKind::PublishContent, 0),
        ];
        let before = s.clone();
        let outcomes = ExecutionEngine::default().execute_batch(&mut s, &mut results);
        assert!(outcomes.iter().all(|o| !o.executed));
        assert_eq!(s, before);
    }

    #[test]
    fn invalid_snapshot_fails_closed() {
        let mut s = snapshot();
        s.cash_minor = -1;
        let mut results = vec![result(AgentRole::Experiment, ActionKind::CreateExperiment, 10)];
        let outcomes = ExecutionEngine::default().execute_batch(&mut s, &mut results);
        assert_eq!(outcomes[0].decision, GovernorDecision::Reject);
        assert!(!outcomes[0].executed);
    }
}
