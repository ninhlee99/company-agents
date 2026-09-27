use crate::{
    agent::{call_model, proposal_confidence, Agent, AgentContext, AgentError},
    model::Model,
    types::*,
};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

fn evidence(ctx: &AgentContext, item: &str) -> Vec<String> {
    vec![
        item.to_owned(),
        format!("status={:?}", ctx.company.status),
        format!("runway_days={}", ctx.company.runway_days),
    ]
}

fn base_proposal(
    agent: AgentRole,
    action: ActionKind,
    ctx: &AgentContext,
    objective: &str,
    cost: i128,
    expected: i128,
    risk: RiskTier,
    confidence: u16,
    rationale: &str,
    reversible: bool,
) -> Proposal {
    Proposal {
        agent,
        objective: objective.into(),
        action,
        cost_minor: cost,
        expected_revenue_minor: expected,
        risk,
        confidence_bps: confidence,
        evidence: evidence(ctx, rationale),
        rationale: rationale.into(),
        reversible,
        requested_permission: Permission::Propose,
    }
}

fn max_safe_cost(action: ActionKind) -> i128 {
    action.max_cost_minor()
}

fn floor_risk_for_action(action: ActionKind) -> RiskTier {
    action.minimum_risk()
}

fn attach_model_reasoning(mut proposal: Proposal, reasoning: &Value, ctx: &AgentContext) -> Proposal {
    let suggestion = serde_json::from_value::<ModelSuggestion>(reasoning.clone()).ok();

    if let Some(s) = suggestion {
        if let Some(action_name) = s.action.as_deref() {
            if let Some(action) = ActionKind::parse(action_name) {
                if proposal.agent.may_propose(action) {
                    proposal.action = action;
                }
            }
        }

        if let Some(objective) = s.objective.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()) {
            if objective.len() <= 2_000 {
                proposal.objective = objective;
            }
        }

        if let Some(rationale) = s.rationale.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()) {
            if rationale.len() <= 4_000 {
                proposal.rationale = rationale;
            }
        }

        if let Some(cost) = s.cost_minor.filter(|v| *v >= 0) {
            proposal.cost_minor = cost
                .min(max_safe_cost(proposal.action))
                .min(ctx.company.budget_remaining_minor.max(0))
                .min(ctx.company.cash_minor.max(0));
        }

        if let Some(expected) = s.expected_revenue_minor.filter(|v| *v >= 0) {
            proposal.expected_revenue_minor = expected.min(1_000_000_000);
        }

        if let Some(risk) = s.risk.as_deref().and_then(RiskTier::parse) {
            proposal.risk = floor_risk_for_action(proposal.action).max(risk);
        } else {
            proposal.risk = proposal.risk.max(floor_risk_for_action(proposal.action));
        }

        if let Some(confidence) = s.confidence {
            proposal.confidence_bps = proposal.confidence_bps.min(proposal_confidence(confidence));
        }

        if let Some(reversible) = s.reversible {
            proposal.reversible = proposal.reversible && reversible;
        }
    }

    proposal.evidence.push("llm_reasoning_is_untrusted_metadata".into());
    proposal
}
macro_rules! define_agent {
    ($name:ident, $role:expr, $permission:expr, $prompt:literal, $body:expr) => {
        pub struct $name;
        #[async_trait]
        impl Agent for $name {
            fn role(&self) -> AgentRole { $role }
            fn permission(&self) -> Permission { $permission }
            fn system_prompt(&self) -> &'static str { include_str!(concat!("../../../agents/", $prompt, "/agent.md")) }
            async fn propose(&self, ctx: &AgentContext, model: Arc<dyn Model>) -> Result<Proposal, AgentError> {
                let reasoning = call_model(self, ctx, model).await?;
                let proposal = $body(ctx);
                Ok(attach_model_reasoning(proposal, &reasoning, ctx))
            }
        }
    };
}

define_agent!(CeoAgent, AgentRole::CEO, Permission::Propose, "ceo", |ctx: &AgentContext| {
    if matches!(ctx.company.status, economic_core::CompanyStatus::Distress | economic_core::CompanyStatus::Emergency | economic_core::CompanyStatus::Liquidation | economic_core::CompanyStatus::Bankrupt) {
        base_proposal(AgentRole::CEO, ActionKind::ReduceBudget, ctx, "protect solvency", 0, 0, RiskTier::Low, proposal_confidence(0.95), "company is in financial distress; strategy must prioritize liquidity", true)
    } else {
        let spend = ctx.company.experiment_budget_minor.min(ctx.company.budget_remaining_minor).min(500);
        base_proposal(AgentRole::CEO, ActionKind::AllocateExperimentBudget, ctx, "fund the highest-value validated opportunity", spend, spend.saturating_mul(3), RiskTier::Medium, proposal_confidence(0.70), "allocate only a bounded reversible experiment budget", true)
    }
});

define_agent!(CfoAgent, AgentRole::CFO, Permission::Propose, "cfo", |ctx: &AgentContext| {
    if ctx.company.runway_days <= 21 || ctx.company.expenses_minor > ctx.company.revenue_minor {
        base_proposal(AgentRole::CFO, ActionKind::ReduceBudget, ctx, "extend runway", 0, 0, RiskTier::Low, proposal_confidence(0.92), "cash preservation threshold triggered", true)
    } else {
        base_proposal(AgentRole::CFO, ActionKind::ProduceReport, ctx, "verify cash flow and unit economics", 0, 0, RiskTier::Low, proposal_confidence(0.97), "financial reporting is required before material allocation", true)
    }
});

define_agent!(CooAgent, AgentRole::COO, Permission::Propose, "coo", |ctx: &AgentContext| {
    if ctx.company.backlog > ctx.company.capacity {
        base_proposal(AgentRole::COO, ActionKind::RebalanceOperations, ctx, "restore delivery capacity", 0, 0, RiskTier::Low, proposal_confidence(0.90), "backlog exceeds operating capacity", true)
    } else {
        base_proposal(AgentRole::COO, ActionKind::ProduceReport, ctx, "maintain operating cadence", 0, 0, RiskTier::Low, proposal_confidence(0.88), "capacity is currently sufficient", true)
    }
});

define_agent!(GrowthAgent, AgentRole::Growth, Permission::Propose, "growth", |ctx: &AgentContext| {
    let weak_conversion = ctx.company.conversion_bps < 150;
    let low_growth = ctx.company.audience_growth_bps < 0;
    let action = if weak_conversion || low_growth { ActionKind::ResearchOpportunity } else { ActionKind::CreateExperiment };
    base_proposal(AgentRole::Growth, action, ctx, "increase profitable demand", 100, 300, RiskTier::Medium, proposal_confidence(0.68), "connect audience growth to conversion and contribution economics", true)
});

define_agent!(ContentAgent, AgentRole::Content, Permission::Propose, "content", |ctx: &AgentContext| {
    if ctx.company.content_revenue_minor < ctx.company.content_cost_minor {
        base_proposal(AgentRole::Content, ActionKind::ResearchOpportunity, ctx, "redesign underperforming content formats", 0, 0, RiskTier::Low, proposal_confidence(0.91), "content unit economics are currently negative", true)
    } else {
        base_proposal(AgentRole::Content, ActionKind::CreateExperiment, ctx, "test a new content hypothesis", 100, 250, RiskTier::Low, proposal_confidence(0.73), "content experiment has bounded downside", true)
    }
});

define_agent!(RecruiterAgent, AgentRole::Recruiter, Permission::Propose, "recruiter", |ctx: &AgentContext| {
    if ctx.company.hiring_need > 0 && ctx.company.revenue_minor > ctx.company.expenses_minor && ctx.company.runway_days > 60 {
        base_proposal(AgentRole::Recruiter, ActionKind::ProposeHire, ctx, "increase productive capacity", 500, 1200, RiskTier::High, proposal_confidence(0.62), "hiring need exists with positive economic headroom", false)
    } else {
        base_proposal(AgentRole::Recruiter, ActionKind::ProduceReport, ctx, "defer hiring until economics justify it", 0, 0, RiskTier::Low, proposal_confidence(0.94), "reuse automation and existing capacity first", true)
    }
});

define_agent!(AnalystAgent, AgentRole::Analyst, Permission::Propose, "analyst", |ctx: &AgentContext| {
    base_proposal(AgentRole::Analyst, ActionKind::ProduceReport, ctx, "produce verified decision support", 0, 0, RiskTier::Low, proposal_confidence(0.98), "separate facts, estimates and predictions", true)
});

define_agent!(ExperimentAgent, AgentRole::Experiment, Permission::Propose, "experiment", |ctx: &AgentContext| {
    let amount = ctx.company.experiment_budget_minor.min(100);
    if amount > 0 {
        base_proposal(AgentRole::Experiment, ActionKind::CreateExperiment, ctx, "discover a profitable opportunity", amount, amount.saturating_mul(2), RiskTier::Medium, proposal_confidence(0.66), "small reversible experiment with explicit max loss", true)
    } else {
        base_proposal(AgentRole::Experiment, ActionKind::ProduceReport, ctx, "wait for experiment budget", 0, 0, RiskTier::Low, proposal_confidence(0.95), "no experiment budget is available", true)
    }
});

pub fn executive_agents() -> Vec<Arc<dyn Agent>> {
    vec![
        Arc::new(CeoAgent),
        Arc::new(CfoAgent),
        Arc::new(CooAgent),
        Arc::new(GrowthAgent),
        Arc::new(ContentAgent),
        Arc::new(RecruiterAgent),
        Arc::new(AnalystAgent),
        Arc::new(ExperimentAgent),
    ]
}
