#![forbid(unsafe_code)]

use agent_runtime::{
    types::{ActionKind, AgentRole, AgentRunResult, GovernedProposal, GovernorDecision, Proposal},
    Tool, ToolRegistry,
};
use economic_core::CompanyStatus;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Executed,
    Noop,
    Rejected,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionReceipt {
    pub idempotency_key: String,
    pub agent: AgentRole,
    pub action: ActionKind,
    pub status: ExecutionStatus,
    pub cost_minor: i128,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionBatch {
    pub snapshot: agent_runtime::CompanySnapshot,
    pub receipts: Vec<ExecutionReceipt>,
    pub total_spend_minor: i128,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPolicy {
    pub max_spend_per_cycle_minor: i128,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            max_spend_per_cycle_minor: 1_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidProposal(String),
    InvalidState(String),
    Overflow,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProposal(v) => write!(f, "invalid proposal: {v}"),
            Self::InvalidState(v) => write!(f, "invalid company state: {v}"),
            Self::Overflow => write!(f, "execution arithmetic overflow"),
        }
    }
}

impl std::error::Error for ExecutionError {}

pub fn execute_approved_results(
    start: agent_runtime::CompanySnapshot,
    results: &[AgentRunResult],
    policy: ExecutionPolicy,
) -> Result<ExecutionBatch, ExecutionError> {
    if policy.max_spend_per_cycle_minor < 0 {
        return Err(ExecutionError::InvalidState(
            "negative execution cycle cap".into(),
        ));
    }

    let mut ordered = results.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|r| role_order(r.agent));

    let mut snapshot = start;
    let mut total_spend = 0_i128;
    let mut receipts = Vec::with_capacity(ordered.len());

    for result in ordered {
        let Some(governed) = result.governance.as_ref() else {
            receipts.push(receipt_for(
                &result.proposal,
                ExecutionStatus::Rejected,
                "missing Governor decision; fail closed",
            ));
            continue;
        };

        if result.agent != governed.proposal.agent {
            receipts.push(ExecutionReceipt {
                idempotency_key: proposal_idempotency_key(&governed.proposal),
                agent: result.agent,
                action: governed.proposal.action,
                status: ExecutionStatus::Rejected,
                cost_minor: governed.proposal.cost_minor,
                reason:
                    "agent/result identity mismatch; execution boundary rejects ambiguous authority"
                        .into(),
            });
            continue;
        }

        let key = proposal_idempotency_key(&governed.proposal);
        if governed.decision != GovernorDecision::Approve {
            receipts.push(ExecutionReceipt {
                idempotency_key: key,
                agent: governed.proposal.agent,
                action: governed.proposal.action,
                status: ExecutionStatus::Deferred,
                cost_minor: governed.proposal.cost_minor,
                reason: format!("Governor decision is {:?}", governed.decision),
            });
            continue;
        }

        let remaining_cycle_cap = policy.max_spend_per_cycle_minor.saturating_sub(total_spend);
        if governed.proposal.cost_minor > remaining_cycle_cap {
            receipts.push(ExecutionReceipt {
                idempotency_key: key,
                agent: governed.proposal.agent,
                action: governed.proposal.action,
                status: ExecutionStatus::Deferred,
                cost_minor: governed.proposal.cost_minor,
                reason: "global execution spend cap reached; explicit revision required".into(),
            });
            continue;
        }

        let (next, receipt) = apply_approved(&snapshot, governed)?;
        if receipt.status == ExecutionStatus::Executed {
            total_spend = total_spend
                .checked_add(receipt.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
        }
        snapshot = next;
        receipts.push(receipt);
    }

    Ok(ExecutionBatch {
        snapshot,
        receipts,
        total_spend_minor: total_spend,
    })
}

pub fn apply_approved(
    snapshot: &agent_runtime::CompanySnapshot,
    governed: &GovernedProposal,
) -> Result<(agent_runtime::CompanySnapshot, ExecutionReceipt), ExecutionError> {
    let proposal = &governed.proposal;
    proposal
        .validate()
        .map_err(ExecutionError::InvalidProposal)?;
    if governed.decision != GovernorDecision::Approve {
        return Ok((
            snapshot.clone(),
            ExecutionReceipt {
                idempotency_key: proposal_idempotency_key(proposal),
                agent: proposal.agent,
                action: proposal.action,
                status: ExecutionStatus::Deferred,
                cost_minor: proposal.cost_minor,
                reason: format!("Governor decision is {:?}", governed.decision),
            },
        ));
    }
    if !ToolRegistry::allowed(proposal.agent, action_tool(proposal.action)) {
        return Ok((
            snapshot.clone(),
            receipt_for(
                proposal,
                ExecutionStatus::Rejected,
                "tool capability denied at execution boundary",
            ),
        ));
    }
    if proposal.cost_minor > 0
        && matches!(
            snapshot.status,
            CompanyStatus::Distress
                | CompanyStatus::Emergency
                | CompanyStatus::Liquidation
                | CompanyStatus::Bankrupt
        )
    {
        return Ok((
            snapshot.clone(),
            receipt_for(
                proposal,
                ExecutionStatus::Rejected,
                "financial distress blocks discretionary spend",
            ),
        ));
    }

    let mut next = snapshot.clone();
    match proposal.action {
        ActionKind::AllocateExperimentBudget => {
            if proposal.cost_minor == 0 {
                return Ok((
                    next,
                    receipt_for(
                        proposal,
                        ExecutionStatus::Noop,
                        "zero-value allocation is a no-op",
                    ),
                ));
            }
            if proposal.cost_minor > snapshot.budget_remaining_minor {
                return Ok((
                    next,
                    receipt_for(
                        proposal,
                        ExecutionStatus::Rejected,
                        "allocation exceeds remaining budget",
                    ),
                ));
            }
            next.budget_remaining_minor = snapshot
                .budget_remaining_minor
                .checked_sub(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            next.experiment_budget_minor = snapshot
                .experiment_budget_minor
                .checked_add(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            Ok((
                next,
                receipt_for(
                    proposal,
                    ExecutionStatus::Executed,
                    "experiment budget allocated within existing company budget",
                ),
            ))
        }
        ActionKind::CreateExperiment => {
            if proposal.cost_minor == 0 {
                return Ok((
                    next,
                    receipt_for(
                        proposal,
                        ExecutionStatus::Noop,
                        "zero-cost experiment is a no-op",
                    ),
                ));
            }
            if proposal.cost_minor > snapshot.experiment_budget_minor {
                return Ok((
                    next,
                    receipt_for(
                        proposal,
                        ExecutionStatus::Rejected,
                        "experiment cost exceeds experiment budget",
                    ),
                ));
            }
            next.cash_minor = snapshot
                .cash_minor
                .checked_sub(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            next.expenses_minor = snapshot
                .expenses_minor
                .checked_add(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            next.assets_minor = snapshot
                .assets_minor
                .checked_sub(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            next.experiment_budget_minor = snapshot
                .experiment_budget_minor
                .checked_sub(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            if next.cash_minor == 0 {
                next.status = CompanyStatus::Emergency;
            }
            Ok((
                next,
                receipt_for(
                    proposal,
                    ExecutionStatus::Executed,
                    "bounded experiment spend executed",
                ),
            ))
        }
        ActionKind::ReduceBudget => {
            let before = snapshot.experiment_budget_minor;
            next.experiment_budget_minor = snapshot
                .experiment_budget_minor
                .min(snapshot.budget_remaining_minor);
            let reason = if next.experiment_budget_minor < before {
                "experiment budget reduced to remaining company budget"
            } else {
                "budget already within guard; no state change"
            };
            let status = if next.experiment_budget_minor < before {
                ExecutionStatus::Executed
            } else {
                ExecutionStatus::Noop
            };
            Ok((next, receipt_for(proposal, status, reason)))
        }
        ActionKind::RebalanceOperations => {
            if snapshot.backlog == 0 || snapshot.backlog <= snapshot.capacity {
                return Ok((
                    next,
                    receipt_for(
                        proposal,
                        ExecutionStatus::Noop,
                        "operations are already within capacity",
                    ),
                ));
            }
            next.backlog = snapshot.backlog.saturating_sub(1);
            Ok((
                next,
                receipt_for(
                    proposal,
                    ExecutionStatus::Executed,
                    "one backlog unit rebalanced without capital spend",
                ),
            ))
        }
        ActionKind::ResearchOpportunity
        | ActionKind::ProduceReport
        | ActionKind::EscalateIncident
        | ActionKind::MitigateRisk
        | ActionKind::ResolveSupportCase
        | ActionKind::OptimizeRetention
        | ActionKind::ReconcileTreasury => Ok((
            next,
            receipt_for(
                proposal,
                ExecutionStatus::Noop,
                "informational/risk action recorded; no mutable economic state",
            ),
        )),
        ActionKind::DevelopProduct => {
            if proposal.cost_minor == 0 {
                return Ok((
                    next,
                    receipt_for(
                        proposal,
                        ExecutionStatus::Noop,
                        "zero-cost product research is a no-op",
                    ),
                ));
            }
            if proposal.cost_minor > snapshot.experiment_budget_minor {
                return Ok((
                    next,
                    receipt_for(
                        proposal,
                        ExecutionStatus::Rejected,
                        "product development cost exceeds experiment budget",
                    ),
                ));
            }
            next.cash_minor = snapshot
                .cash_minor
                .checked_sub(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            next.expenses_minor = snapshot
                .expenses_minor
                .checked_add(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            next.assets_minor = snapshot
                .assets_minor
                .checked_sub(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            next.experiment_budget_minor = snapshot
                .experiment_budget_minor
                .checked_sub(proposal.cost_minor)
                .ok_or(ExecutionError::Overflow)?;
            if next.cash_minor == 0 {
                next.status = CompanyStatus::Emergency;
            }
            Ok((
                next,
                receipt_for(
                    proposal,
                    ExecutionStatus::Executed,
                    "bounded product prototype spend executed",
                ),
            ))
        }
        ActionKind::PublishContent | ActionKind::ProposeHire => Ok((
            next,
            receipt_for(
                proposal,
                ExecutionStatus::Deferred,
                "material side effect requires explicit authorization",
            ),
        )),
        ActionKind::None => Err(ExecutionError::InvalidProposal(
            "None action cannot execute".into(),
        )),
    }
}

fn action_tool(action: ActionKind) -> Tool {
    match action {
        ActionKind::CreateExperiment | ActionKind::DevelopProduct => Tool::CreateExperiment,
        ActionKind::AllocateExperimentBudget => Tool::AllocateExperimentBudget,
        ActionKind::ReduceBudget => Tool::ReduceBudget,
        ActionKind::RebalanceOperations => Tool::RebalanceOperations,
        ActionKind::ResearchOpportunity | ActionKind::OptimizeRetention => Tool::ResearchOpportunity,
        ActionKind::PublishContent => Tool::PublishContent,
        ActionKind::ProposeHire => Tool::ProposeHire,
        ActionKind::ProduceReport
        | ActionKind::EscalateIncident
        | ActionKind::MitigateRisk
        | ActionKind::ResolveSupportCase
        | ActionKind::ReconcileTreasury => Tool::ProduceReport,
        ActionKind::None => Tool::ReadCompany,
    }
}

fn receipt_for(proposal: &Proposal, status: ExecutionStatus, reason: &str) -> ExecutionReceipt {
    ExecutionReceipt {
        idempotency_key: proposal_idempotency_key(proposal),
        agent: proposal.agent,
        action: proposal.action,
        status,
        cost_minor: proposal.cost_minor,
        reason: reason.into(),
    }
}

pub fn proposal_idempotency_key(proposal: &Proposal) -> String {
    let canonical = serde_json::json!({
        "agent": proposal.agent.as_str(),
        "action": format!("{:?}", proposal.action),
        "objective": proposal.objective,
        "cost_minor": proposal.cost_minor,
        "expected_revenue_minor": proposal.expected_revenue_minor,
        "risk": format!("{:?}", proposal.risk),
        "confidence_bps": proposal.confidence_bps,
        "evidence": proposal.evidence,
        "rationale": proposal.rationale,
        "reversible": proposal.reversible,
    });
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let hash = Sha256::digest(bytes);
    format!("proposal:{:x}", hash)
}

fn role_order(role: AgentRole) -> usize {
    AgentRole::ALL
        .iter()
        .position(|v| *v == role)
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::types::{
        ActionKind, AgentRole, CompanySnapshot, GovernedProposal, GovernorDecision, Permission,
        Proposal, RiskTier,
    };
    use economic_core::CompanyStatus;

    fn snapshot() -> CompanySnapshot {
        CompanySnapshot {
            company_id: "c1".into(),
            cash_minor: 10_000,
            revenue_minor: 20_000,
            expenses_minor: 10_000,
            liabilities_minor: 0,
            assets_minor: 10_000,
            runway_days: 90,
            status: CompanyStatus::Growth,
            budget_remaining_minor: 1_000,
            experiment_budget_minor: 500,
            content_cost_minor: 100,
            content_revenue_minor: 200,
            backlog: 5,
            capacity: 2,
            conversion_bps: 250,
            audience_growth_bps: 100,
            hiring_need: 0,
        }
    }

    fn governed(action: ActionKind, cost: i128) -> GovernedProposal {
        let risk = action.minimum_risk();
        let proposal = Proposal {
            agent: match action {
                ActionKind::AllocateExperimentBudget | ActionKind::ReduceBudget => AgentRole::CEO,
                ActionKind::RebalanceOperations => AgentRole::COO,
                _ => AgentRole::Experiment,
            },
            objective: "test".into(),
            action,
            cost_minor: cost,
            expected_revenue_minor: cost.saturating_mul(2),
            risk,
            confidence_bps: 8_000,
            evidence: vec!["test".into()],
            rationale: "bounded".into(),
            reversible: true,
            requested_permission: Permission::Propose,
        };
        GovernedProposal {
            proposal,
            decision: GovernorDecision::Approve,
            reason: "test".into(),
        }
    }

    #[test]
    fn approved_experiment_spend_changes_state() {
        let (next, receipt) =
            apply_approved(&snapshot(), &governed(ActionKind::CreateExperiment, 100)).unwrap();
        assert_eq!(receipt.status, ExecutionStatus::Executed);
        assert_eq!(next.cash_minor, 9_900);
        assert_eq!(next.experiment_budget_minor, 400);
    }

    #[test]
    fn approved_allocate_moves_budget_without_spending_cash() {
        let before = snapshot();
        let (next, receipt) = apply_approved(
            &before,
            &governed(ActionKind::AllocateExperimentBudget, 100),
        )
        .unwrap();
        assert_eq!(receipt.status, ExecutionStatus::Executed);
        assert_eq!(next.cash_minor, before.cash_minor);
        assert_eq!(next.experiment_budget_minor, 600);
        assert_eq!(next.budget_remaining_minor, 900);
    }

    #[test]
    fn global_spend_cap_limits_multi_agent_execution() {
        let results = vec![
            AgentRunResult {
                agent: AgentRole::Experiment,
                proposal: governed(ActionKind::CreateExperiment, 400).proposal.clone(),
                governance: Some(governed(ActionKind::CreateExperiment, 400)),
            },
            AgentRunResult {
                agent: AgentRole::Experiment,
                proposal: governed(ActionKind::CreateExperiment, 400).proposal.clone(),
                governance: Some(governed(ActionKind::CreateExperiment, 400)),
            },
        ];
        let batch = execute_approved_results(
            snapshot(),
            &results,
            ExecutionPolicy {
                max_spend_per_cycle_minor: 600,
            },
        )
        .unwrap();
        assert_eq!(batch.total_spend_minor, 400);
        assert_eq!(batch.snapshot.cash_minor, 9_600);
        assert!(batch
            .receipts
            .iter()
            .any(|r| r.status == ExecutionStatus::Deferred));
    }

    #[test]
    fn mismatched_agent_identity_is_rejected() {
        let mut result = AgentRunResult {
            agent: AgentRole::Growth,
            proposal: governed(ActionKind::CreateExperiment, 100).proposal.clone(),
            governance: Some(governed(ActionKind::CreateExperiment, 100)),
        };
        result.proposal.agent = AgentRole::Experiment;
        let batch =
            execute_approved_results(snapshot(), &[result], ExecutionPolicy::default()).unwrap();
        assert!(batch
            .receipts
            .iter()
            .any(|r| r.status == ExecutionStatus::Rejected));
        assert_eq!(batch.snapshot.cash_minor, 10_000);
    }

    #[test]
    fn material_actions_never_execute() {
        let p = Proposal {
            agent: AgentRole::Content,
            objective: "publish".into(),
            action: ActionKind::PublishContent,
            cost_minor: 0,
            expected_revenue_minor: 100,
            risk: RiskTier::Medium,
            confidence_bps: 9_000,
            evidence: vec!["approved".into()],
            rationale: "material".into(),
            reversible: true,
            requested_permission: Permission::Propose,
        };
        let g = GovernedProposal {
            proposal: p,
            decision: GovernorDecision::Approve,
            reason: "synthetic".into(),
        };
        let (_next, receipt) = apply_approved(&snapshot(), &g).unwrap();
        assert_eq!(receipt.status, ExecutionStatus::Deferred);
    }
}
