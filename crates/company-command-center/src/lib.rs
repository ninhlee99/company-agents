#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyRevenuePoint {
    pub day: String,
    pub revenue_minor: i128,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RevenueTrend {
    Up,
    Flat,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentFunnel {
    pub views_7d: i64,
    pub clicks_7d: i64,
    pub conversions_7d: i64,
    pub spend_7d_minor: i128,
    pub commission_7d_minor: i128,
    pub contribution_margin_7d_minor: i128,
    pub content_count_7d: i64,
    pub ctr_bps: u32,
    pub cvr_bps: u32,
    pub commission_rpm_minor: i128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LivePulse {
    pub sessions_30d: i64,
    pub gift_count_30d: i64,
    pub gift_value_30d_minor: i128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompliancePulse {
    pub policy_ready: bool,
    pub allowed_24h: i64,
    pub review_24h: i64,
    pub blocked_24h: i64,
    pub unknown_24h: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrowthOpportunityDigest {
    pub title: String,
    pub score_bps: u32,
    pub confidence_bps: u32,
    pub status: String,
    pub ttfc_seconds: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertKind {
    NeedsAttention,
    Opportunity,
    Healthy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandCenterAlert {
    pub kind: AlertKind,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandCenterInput {
    pub cash_minor: i128,
    pub revenue_mtd_minor: i128,
    pub revenue_last_30d_minor: i128,
    pub revenue_lifetime_minor: i128,
    pub revenue_target_minor: i128,
    pub revenue_transaction_count: i64,
    pub contribution_margin_mtd_minor: Option<i128>,
    pub unclassified_expense_entry_count: i64,
    pub affiliate_reported_commission_mtd_minor: i128,
    pub affiliate_attributed_commission_mtd_minor: i128,
    pub affiliate_payout_mtd_minor: i128,
    pub affiliate_variance_mtd_minor: i128,
    pub affiliate_orders_mtd: i64,
    pub affiliate_net_order_value_mtd_minor: i128,
    pub runway_days: i64,
    pub active_employee_count: i64,
    pub payroll_due_count: i64,
    pub content: ContentFunnel,
    pub live: LivePulse,
    pub compliance: CompliancePulse,
    pub growth_opportunities: Vec<GrowthOpportunityDigest>,
    pub daily_revenue: Vec<DailyRevenuePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandCenterSummary {
    pub target_progress_bps: u32,
    pub revenue_trend: RevenueTrend,
    pub revenue_trend_delta_bps: i32,
    pub revenue_mtd_per_active_employee_minor: Option<i128>,
    pub commission_7d_per_content_minor: Option<i128>,
    pub alerts: Vec<CommandCenterAlert>,
}

pub fn summarize(input: &CommandCenterInput) -> Result<CommandCenterSummary, String> {
    validate_input(input)?;
    if input.revenue_target_minor <= 0 {
        return Err("revenue target must be positive".into());
    }
    if input.revenue_transaction_count < 0
        || input.affiliate_orders_mtd < 0
        || input.active_employee_count < 0
        || input.payroll_due_count < 0
    {
        return Err("command center counts cannot be negative".into());
    }
    if input.unclassified_expense_entry_count < 0 {
        return Err("unclassified expense count cannot be negative".into());
    }
    let target_progress_bps = ratio_bps(input.revenue_mtd_minor, input.revenue_target_minor);
    let (revenue_trend, revenue_trend_delta_bps) = revenue_trend(&input.daily_revenue);
    let revenue_mtd_per_active_employee_minor = input
        .active_employee_count
        .checked_sub(0)
        .filter(|count| *count > 0)
        .and_then(|count| input.revenue_mtd_minor.checked_div(i128::from(count)));
    let commission_7d_per_content_minor = input
        .content
        .content_count_7d
        .checked_sub(0)
        .filter(|count| *count > 0)
        .and_then(|count| {
            input.content.commission_7d_minor.checked_div(i128::from(count))
        });

    let mut alerts = Vec::new();
    if input.cash_minor <= 0 || input.runway_days < 15 {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::NeedsAttention,
            title: "Liquidity risk".into(),
            detail: format!(
                "Cash {} and runway {} days require immediate review.",
                input.cash_minor, input.runway_days
            ),
        });
    } else if input.runway_days < 30 {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::NeedsAttention,
            title: "Runway watch".into(),
            detail: format!("Runway is {} days; discretionary spend should stay conservative.", input.runway_days),
        });
    }

    if input.contribution_margin_mtd_minor.is_none() {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::NeedsAttention,
            title: "Margin evidence incomplete".into(),
            detail: format!(
                "{} MTD expense entries are still unclassified.",
                input.unclassified_expense_entry_count
            ),
        });
    } else if input.contribution_margin_mtd_minor.unwrap_or_default() < 0 {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::NeedsAttention,
            title: "Negative contribution margin".into(),
            detail: "MTD contribution margin is negative on classified expenses.".into(),
        });
    }

    if input.affiliate_variance_mtd_minor != 0 {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::NeedsAttention,
            title: "Affiliate reconciliation variance".into(),
            detail: format!(
                "Reported vs attributed commission variance is {} minor units.",
                input.affiliate_variance_mtd_minor
            ),
        });
    }

    if !input.compliance.policy_ready
        || input.compliance.review_24h > 0
        || input.compliance.blocked_24h > 0
        || input.compliance.unknown_24h > 0
    {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::NeedsAttention,
            title: "Policy evidence needs attention".into(),
            detail: format!(
                "Policy ready: {} · 24h checks: {} allowed / {} review / {} blocked / {} unknown.",
                input.compliance.policy_ready,
                input.compliance.allowed_24h,
                input.compliance.review_24h,
                input.compliance.blocked_24h,
                input.compliance.unknown_24h
            ),
        });
    }

    for opportunity in input.growth_opportunities.iter().take(3) {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::Opportunity,
            title: opportunity.title.clone(),
            detail: format!(
                "{} · {}% confidence · {} · TTFC {}",
                opportunity.status,
                opportunity.confidence_bps / 100,
                opportunity.score_bps / 100,
                opportunity
                    .ttfc_seconds
                    .map(|value| format!("{value}s"))
                    .unwrap_or_else(|| "pending".into())
            ),
        });
    }

    let has_attention = alerts
        .iter()
        .any(|alert| alert.kind == AlertKind::NeedsAttention);
    if !has_attention {
        alerts.push(CommandCenterAlert {
            kind: AlertKind::Healthy,
            title: "Core controls healthy".into(),
            detail: "No command-center exception was raised from the supplied evidence.".into(),
        });
    }

    Ok(CommandCenterSummary {
        target_progress_bps,
        revenue_trend,
        revenue_trend_delta_bps,
        revenue_mtd_per_active_employee_minor,
        commission_7d_per_content_minor,
        alerts,
    })
}

fn validate_input(input: &CommandCenterInput) -> Result<(), String> {
    for (name, value) in [
        ("content.views_7d", input.content.views_7d),
        ("content.clicks_7d", input.content.clicks_7d),
        ("content.conversions_7d", input.content.conversions_7d),
        ("content.content_count_7d", input.content.content_count_7d),
        ("live.sessions_30d", input.live.sessions_30d),
        ("live.gift_count_30d", input.live.gift_count_30d),
        ("affiliate orders", input.affiliate_orders_mtd),
    ] {
        if value < 0 {
            return Err(format!("{name} cannot be negative"));
        }
    }
    if input.content.clicks_7d > input.content.views_7d
        || input.content.conversions_7d > input.content.clicks_7d
    {
        return Err("content funnel counters must be monotonic".into());
    }
    for (name, value) in [
        ("content.spend_7d_minor", input.content.spend_7d_minor),
        ("content.commission_7d_minor", input.content.commission_7d_minor),
        (
            "live.gift_value_30d_minor",
            input.live.gift_value_30d_minor,
        ),
        (
            "affiliate_reported_commission_mtd_minor",
            input.affiliate_reported_commission_mtd_minor,
        ),
        (
            "affiliate_attributed_commission_mtd_minor",
            input.affiliate_attributed_commission_mtd_minor,
        ),
        ("affiliate_payout_mtd_minor", input.affiliate_payout_mtd_minor),
        ("affiliate_net_order_value_mtd_minor", input.affiliate_net_order_value_mtd_minor),
    ] {
        if value < 0 {
            return Err(format!("{name} cannot be negative"));
        }
    }
    let expected_affiliate_variance = input
        .affiliate_reported_commission_mtd_minor
        .checked_sub(input.affiliate_attributed_commission_mtd_minor)
        .ok_or_else(|| "affiliate commission variance overflow".to_string())?;
    if input.affiliate_variance_mtd_minor != expected_affiliate_variance {
        return Err("affiliate variance does not reconcile with reported and attributed commission".into());
    }
    for (name, value) in [
        ("compliance.allowed_24h", input.compliance.allowed_24h),
        ("compliance.review_24h", input.compliance.review_24h),
        ("compliance.blocked_24h", input.compliance.blocked_24h),
        ("compliance.unknown_24h", input.compliance.unknown_24h),
    ] {
        if value < 0 {
            return Err(format!("{name} cannot be negative"));
        }
    }
    for point in &input.daily_revenue {
        if point.day.trim().is_empty() {
            return Err("daily revenue points must have a day".into());
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CeoExamStatus {
    Pass,
    Review,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CeoExamDomain {
    Market,
    Product,
    Content,
    Live,
    Finance,
    Strategy,
    Ai,
    Operations,
}

impl CeoExamDomain {
    pub const ALL: [Self; 8] = [
        Self::Market,
        Self::Product,
        Self::Content,
        Self::Live,
        Self::Finance,
        Self::Strategy,
        Self::Ai,
        Self::Operations,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Market => "market",
            Self::Product => "product",
            Self::Content => "content",
            Self::Live => "live",
            Self::Finance => "finance",
            Self::Strategy => "strategy",
            Self::Ai => "ai",
            Self::Operations => "operations",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CeoExamDomainResult {
    pub domain: CeoExamDomain,
    pub status: CeoExamStatus,
    pub score_bps: Option<u32>,
    pub evidence_refs: Vec<String>,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CeoExamReport {
    pub company_id: uuid::Uuid,
    pub exam_key: String,
    pub period_start_epoch: i64,
    pub period_end_epoch: i64,
    pub overall_status: CeoExamStatus,
    pub overall_score_bps: Option<u32>,
    pub domains: Vec<CeoExamDomainResult>,
    pub critical_findings: Vec<String>,
    pub source: String,
}

pub fn run_ceo_exam(
    company_id: uuid::Uuid,
    exam_key: impl Into<String>,
    period_start_epoch: i64,
    period_end_epoch: i64,
    input: &CommandCenterInput,
    summary: &CommandCenterSummary,
    agent_evaluations: &[company_agent_evaluation::AgentEvaluation],
) -> Result<CeoExamReport, String> {
    if company_id == uuid::Uuid::nil() {
        return Err("CEO exam company_id is required".into());
    }
    if period_start_epoch <= 0 || period_end_epoch <= period_start_epoch {
        return Err("CEO exam period must be positive and ordered".into());
    }
    validate_input(input)?;
    if agent_evaluations.len() > 500 {
        return Err("CEO exam received too many agent evaluations".into());
    }

    let mut domains = Vec::with_capacity(CeoExamDomain::ALL.len());

    domains.push(exam_market(input));
    domains.push(exam_product(input));
    domains.push(exam_content(input));
    domains.push(exam_live(input));
    domains.push(exam_finance(input));
    domains.push(exam_strategy(input, summary));
    domains.push(exam_ai(agent_evaluations));
    domains.push(exam_operations(input));

    let overall_status = if domains.iter().any(|d| d.status == CeoExamStatus::Review) {
        CeoExamStatus::Review
    } else if domains.iter().any(|d| d.status == CeoExamStatus::Unknown) {
        CeoExamStatus::Unknown
    } else {
        CeoExamStatus::Pass
    };

    let scored = domains.iter().filter_map(|d| d.score_bps);
    let scores: Vec<u32> = scored.collect();
    let overall_score_bps = if scores.len() >= 4 {
        Some(
            (scores.iter().map(|value| u64::from(*value)).sum::<u64>() / scores.len() as u64)
                .min(10_000) as u32,
        )
    } else {
        None
    };

    let critical_findings = domains
        .iter()
        .filter(|d| d.status != CeoExamStatus::Pass)
        .flat_map(|d| d.findings.iter().map(|finding| format!("{}: {finding}", d.domain.as_str())))
        .take(24)
        .collect();

    Ok(CeoExamReport {
        company_id,
        exam_key: exam_key.into(),
        period_start_epoch,
        period_end_epoch,
        overall_status,
        overall_score_bps,
        domains,
        critical_findings,
        source: "authoritative_company_command_center".into(),
    })
}

fn exam_market(input: &CommandCenterInput) -> CeoExamDomainResult {
    let evidence = "command-center:growth-opportunities".into();
    let best_confidence = input
        .growth_opportunities
        .iter()
        .map(|item| item.confidence_bps)
        .max();
    match best_confidence {
        Some(confidence) if confidence >= 8_000 => CeoExamDomainResult {
            domain: CeoExamDomain::Market,
            status: CeoExamStatus::Pass,
            score_bps: Some(confidence),
            evidence_refs: vec![evidence],
            findings: vec!["At least one current growth opportunity has strong evidence confidence.".into()],
        },
        Some(confidence) => CeoExamDomainResult {
            domain: CeoExamDomain::Market,
            status: CeoExamStatus::Review,
            score_bps: Some(confidence),
            evidence_refs: vec![evidence],
            findings: vec!["Current growth opportunities are below the strong-confidence gate.".into()],
        },
        None => CeoExamDomainResult {
            domain: CeoExamDomain::Market,
            status: CeoExamStatus::Unknown,
            score_bps: None,
            evidence_refs: vec![evidence],
            findings: vec!["No current growth-opportunity evidence is available.".into()],
        },
    }
}

fn exam_product(input: &CommandCenterInput) -> CeoExamDomainResult {
    let refs = vec!["command-center:affiliate-orders".into()];
    if input.affiliate_orders_mtd > 0 && input.affiliate_net_order_value_mtd_minor > 0 {
        CeoExamDomainResult {
            domain: CeoExamDomain::Product,
            status: CeoExamStatus::Pass,
            score_bps: Some(10_000),
            evidence_refs: refs,
            findings: vec!["Observed affiliate orders and positive net order value are present for MTD.".into()],
        }
    } else if input.affiliate_orders_mtd > 0 {
        CeoExamDomainResult {
            domain: CeoExamDomain::Product,
            status: CeoExamStatus::Review,
            score_bps: Some(5_000),
            evidence_refs: refs,
            findings: vec!["Orders are present but observed net order value is not positive.".into()],
        }
    } else {
        CeoExamDomainResult {
            domain: CeoExamDomain::Product,
            status: CeoExamStatus::Unknown,
            score_bps: None,
            evidence_refs: refs,
            findings: vec!["No observed affiliate order evidence is available for the current MTD period.".into()],
        }
    }
}

fn exam_content(input: &CommandCenterInput) -> CeoExamDomainResult {
    let refs = vec!["command-center:content-funnel-7d".into()];
    if input.content.content_count_7d <= 0 {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Content,
            status: CeoExamStatus::Unknown,
            score_bps: None,
            evidence_refs: refs,
            findings: vec!["No content observation exists in the 7-day window.".into()],
        };
    }
    if input.content.contribution_margin_7d_minor < 0 {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Content,
            status: CeoExamStatus::Review,
            score_bps: Some((input.content.ctr_bps / 2 + input.content.cvr_bps / 2).min(10_000)),
            evidence_refs: refs,
            findings: vec!["Observed 7-day content contribution margin is negative.".into()],
        };
    }
    CeoExamDomainResult {
        domain: CeoExamDomain::Content,
        status: CeoExamStatus::Pass,
        score_bps: Some((input.content.ctr_bps / 2 + input.content.cvr_bps / 2).min(10_000)),
        evidence_refs: refs,
        findings: vec!["Content has current observations with non-negative 7-day contribution margin.".into()],
    }
}

fn exam_live(input: &CommandCenterInput) -> CeoExamDomainResult {
    let refs = vec!["command-center:live-30d".into()];
    if input.live.sessions_30d <= 0 {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Live,
            status: CeoExamStatus::Unknown,
            score_bps: None,
            evidence_refs: refs,
            findings: vec!["No LIVE session evidence exists in the 30-day window.".into()],
        };
    }
    CeoExamDomainResult {
        domain: CeoExamDomain::Live,
        status: CeoExamStatus::Pass,
        score_bps: Some(10_000),
        evidence_refs: refs,
        findings: vec![
            format!("{} LIVE sessions were observed in 30 days.", input.live.sessions_30d),
            "Gift value is treated as engagement evidence, not company revenue.".into(),
        ],
    }
}

fn exam_finance(input: &CommandCenterInput) -> CeoExamDomainResult {
    let refs = vec![
        "command-center:revenue-periods".into(),
        "command-center:contribution-margin".into(),
        "command-center:affiliate-reconciliation".into(),
    ];
    let mut findings = Vec::new();
    if input.cash_minor <= 0 {
        findings.push("Cash is non-positive.".into());
    }
    if input.runway_days < 15 {
        findings.push("Runway is below the emergency liquidity threshold.".into());
    }
    if input.contribution_margin_mtd_minor.is_none() {
        findings.push("Contribution-margin evidence is incomplete.".into());
    }
    if input.unclassified_expense_entry_count > 0 {
        findings.push("Unclassified expense entries remain.".into());
    }
    if input.affiliate_variance_mtd_minor != 0 {
        findings.push("Affiliate reported-attributed variance is non-zero.".into());
    }
    if !findings.is_empty() {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Finance,
            status: CeoExamStatus::Review,
            score_bps: None,
            evidence_refs: refs,
            findings,
        };
    }
    let liquidity_score = (input.runway_days.max(0).min(100) as u32 * 100).min(10_000);
    let margin_score = if input.contribution_margin_mtd_minor.unwrap_or(0) >= 0 {
        10_000
    } else {
        0
    };
    CeoExamDomainResult {
        domain: CeoExamDomain::Finance,
        status: CeoExamStatus::Pass,
        score_bps: Some(((liquidity_score as u64 + margin_score as u64) / 2) as u32),
        evidence_refs: refs,
        findings: vec!["Liquidity, margin evidence and reconciliation gates show no current exception.".into()],
    }
}

fn exam_strategy(
    input: &CommandCenterInput,
    summary: &CommandCenterSummary,
) -> CeoExamDomainResult {
    let refs = vec!["command-center:revenue-trend-and-target".into()];
    if input.daily_revenue.len() < 6 {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Strategy,
            status: CeoExamStatus::Unknown,
            score_bps: None,
            evidence_refs: refs,
            findings: vec!["Fewer than 6 daily revenue points are available; trend confidence is insufficient.".into()],
        };
    }
    if summary.revenue_trend == RevenueTrend::Down && summary.target_progress_bps < 5_000 {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Strategy,
            status: CeoExamStatus::Review,
            score_bps: Some(summary.target_progress_bps),
            evidence_refs: refs,
            findings: vec!["Revenue trend is down while MTD progress remains below half of the planning target.".into()],
        };
    }
    CeoExamDomainResult {
        domain: CeoExamDomain::Strategy,
        status: CeoExamStatus::Pass,
        score_bps: Some(summary.target_progress_bps),
        evidence_refs: refs,
        findings: vec!["Current revenue trend does not trigger the deterministic strategy review gate.".into()],
    }
}

fn exam_ai(evaluations: &[company_agent_evaluation::AgentEvaluation]) -> CeoExamDomainResult {
    let refs = vec!["agent-evaluation:outcome-evidence".into()];
    if evaluations.is_empty() {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Ai,
            status: CeoExamStatus::Unknown,
            score_bps: None,
            evidence_refs: refs,
            findings: vec!["No agent outcome evaluation evidence exists in the exam window.".into()],
        };
    }
    let coverage: u32 = (evaluations
        .iter()
        .map(|value| u64::from(value.outcome_evidence_coverage_bps))
        .sum::<u64>()
        / evaluations.len() as u64)
        .min(10_000) as u32;
    let has_insufficient = evaluations
        .iter()
        .any(|value| matches!(value.status, company_agent_evaluation::EvaluationStatus::InsufficientEvidence));
    let status = if has_insufficient {
        CeoExamStatus::Review
    } else if coverage >= 8_000 {
        CeoExamStatus::Pass
    } else {
        CeoExamStatus::Review
    };
    CeoExamDomainResult {
        domain: CeoExamDomain::Ai,
        status,
        score_bps: Some(coverage),
        evidence_refs: refs,
        findings: vec![format!(
            "Average outcome-evidence coverage across {} evaluated agents is {}%.",
            evaluations.len(),
            coverage / 100
        )],
    }
}

fn exam_operations(input: &CommandCenterInput) -> CeoExamDomainResult {
    let refs = vec![
        "command-center:workforce".into(),
        "command-center:payroll".into(),
    ];
    if input.active_employee_count <= 0 {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Operations,
            status: CeoExamStatus::Unknown,
            score_bps: None,
            evidence_refs: refs,
            findings: vec!["No active employee evidence is available.".into()],
        };
    }
    if input.payroll_due_count > 0 {
        return CeoExamDomainResult {
            domain: CeoExamDomain::Operations,
            status: CeoExamStatus::Review,
            score_bps: Some(5_000),
            evidence_refs: refs,
            findings: vec![format!("{} payroll obligations are due.", input.payroll_due_count)],
        };
    }
    CeoExamDomainResult {
        domain: CeoExamDomain::Operations,
        status: CeoExamStatus::Pass,
        score_bps: Some(10_000),
        evidence_refs: refs,
        findings: vec!["Active workforce evidence exists with no currently due payroll obligation.".into()],
    }
}

fn ratio_bps(numerator: i128, denominator: i128) -> u32 {
    if numerator <= 0 || denominator <= 0 {
        return 0;
    }
    numerator
        .checked_mul(10_000)
        .and_then(|value| value.checked_div(denominator))
        .map(|value| value.clamp(0, 10_000) as u32)
        .unwrap_or(10_000)
}

fn revenue_trend(points: &[DailyRevenuePoint]) -> (RevenueTrend, i32) {
    if points.len() < 6 {
        return (RevenueTrend::Flat, 0);
    }
    let split = points.len() - 3;
    let prior = points[..split].iter().rev().take(3).map(|p| p.revenue_minor).sum::<i128>();
    let recent = points[split..].iter().map(|p| p.revenue_minor).sum::<i128>();
    if prior <= 0 {
        return (if recent > 0 { RevenueTrend::Up } else { RevenueTrend::Flat }, 0);
    }
    let delta_bps = recent
        .checked_sub(prior)
        .and_then(|delta| delta.checked_mul(10_000))
        .and_then(|delta| delta.checked_div(prior))
        .unwrap_or(0)
        .clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32;
    let trend = if delta_bps >= 500 {
        RevenueTrend::Up
    } else if delta_bps <= -500 {
        RevenueTrend::Down
    } else {
        RevenueTrend::Flat
    };
    (trend, delta_bps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CommandCenterInput {
        CommandCenterInput {
            cash_minor: 100_000,
            revenue_mtd_minor: 40_000,
            revenue_last_30d_minor: 80_000,
            revenue_lifetime_minor: 200_000,
            revenue_target_minor: 100_000,
            revenue_transaction_count: 20,
            contribution_margin_mtd_minor: Some(15_000),
            unclassified_expense_entry_count: 0,
            affiliate_reported_commission_mtd_minor: 2_000,
            affiliate_attributed_commission_mtd_minor: 1_900,
            affiliate_payout_mtd_minor: 1_500,
            affiliate_variance_mtd_minor: 100,
            affiliate_orders_mtd: 8,
            affiliate_net_order_value_mtd_minor: 35_000,
            runway_days: 45,
            active_employee_count: 4,
            payroll_due_count: 0,
            content: ContentFunnel {
                views_7d: 10_000,
                clicks_7d: 800,
                conversions_7d: 40,
                spend_7d_minor: 2_000,
                commission_7d_minor: 4_000,
                contribution_margin_7d_minor: 2_000,
                content_count_7d: 5,
                ctr_bps: 800,
                cvr_bps: 500,
                commission_rpm_minor: 400,
            },
            live: LivePulse {
                sessions_30d: 3,
                gift_count_30d: 20,
                gift_value_30d_minor: 300,
            },
            compliance: CompliancePulse {
                policy_ready: true,
                allowed_24h: 10,
                review_24h: 0,
                blocked_24h: 0,
                unknown_24h: 0,
            },
            growth_opportunities: vec![],
            daily_revenue: (0..7)
                .map(|day| DailyRevenuePoint {
                    day: day.to_string(),
                    revenue_minor: 100 + i128::from(day) * 25,
                })
                .collect(),
        }
    }

    #[test]
    fn ceo_exam_marks_missing_evidence_unknown_instead_of_zero() {
        let value = input();
        let summary = summarize(&value).unwrap();
        let report = run_ceo_exam(
            uuid::Uuid::new_v4(),
            "test-exam",
            1_800_000_000 - 86_400,
            1_800_000_000,
            &value,
            &summary,
            &[],
        )
        .unwrap();
        assert_eq!(report.domains.len(), 8);
        assert_eq!(
            report.domains.iter().find(|d| d.domain == CeoExamDomain::Ai).unwrap().status,
            CeoExamStatus::Unknown
        );
        assert_eq!(report.domains.iter().filter(|d| d.status == CeoExamStatus::Unknown).count(), 1);
        assert_eq!(report.overall_status, CeoExamStatus::Unknown);
    }

    #[test]
    fn ceo_exam_flags_finance_and_compliance_exceptions() {
        let mut value = input();
        value.cash_minor = 0;
        value.contribution_margin_mtd_minor = Some(-1);
        value.affiliate_variance_mtd_minor = 100;
        value.compliance.policy_ready = false;
        let summary = summarize(&value).unwrap();
        let report = run_ceo_exam(
            uuid::Uuid::new_v4(),
            "risk-exam",
            1_800_000_000 - 86_400,
            1_800_000_000,
            &value,
            &summary,
            &[],
        )
        .unwrap();
        assert_eq!(
            report.domains.iter().find(|d| d.domain == CeoExamDomain::Finance).unwrap().status,
            CeoExamStatus::Review
        );
        assert!(report.critical_findings.len() >= 2);
    }

    #[test]
    fn ceo_exam_ai_domain_passes_with_full_outcome_evidence() {
        let value = input();
        let summary = summarize(&value).unwrap();
        let eval = company_agent_evaluation::evaluate(&company_agent_evaluation::AgentEvaluationInput {
            agent_name: "Growth".into(),
            proposal_count: 10,
            approved_count: 8,
            rejected_count: 1,
            revision_count: 1,
            escalated_count: 0,
            executed_count: 6,
            deferred_count: 2,
            observed_spend_minor: 1_000,
            projected_revenue_minor: 2_000,
            outcome_evidence_count: 6,
            observed_revenue_delta_minor: 1_500,
            observed_contribution_margin_delta_minor: 700,
        }).unwrap();
        let report = run_ceo_exam(
            uuid::Uuid::new_v4(),
            "ai-exam",
            1_800_000_000 - 86_400,
            1_800_000_000,
            &value,
            &summary,
            &[eval],
        )
        .unwrap();
        assert_eq!(
            report.domains.iter().find(|d| d.domain == CeoExamDomain::Ai).unwrap().status,
            CeoExamStatus::Pass
        );
    }

    #[test]
    fn target_progress_and_trend_are_deterministic() {
        let summary = summarize(&input()).unwrap();
        assert_eq!(summary.target_progress_bps, 4_000);
        assert_eq!(summary.revenue_trend, RevenueTrend::Up);
        assert!(summary.revenue_trend_delta_bps > 0);
        assert!(summary
            .alerts
            .iter()
            .any(|alert| alert.kind == AlertKind::NeedsAttention));
    }

    #[test]
    fn content_productivity_uses_same_seven_day_window() {
        let summary = summarize(&input()).unwrap();
        assert_eq!(summary.commission_7d_per_content_minor, Some(800));
    }

    #[test]
    fn healthy_state_can_coexist_with_growth_opportunity() {
        let mut value = input();
        value.affiliate_variance_mtd_minor = 0;
        value.growth_opportunities.push(GrowthOpportunityDigest {
            title: "Test opportunity".into(),
            score_bps: 7000,
            confidence_bps: 8000,
            status: "READY".into(),
            ttfc_seconds: None,
        });
        let summary = summarize(&value).unwrap();
        assert!(summary
            .alerts
            .iter()
            .any(|alert| alert.kind == AlertKind::Healthy));
        assert!(summary
            .alerts
            .iter()
            .any(|alert| alert.kind == AlertKind::Opportunity));
    }

    #[test]
    fn invalid_nested_funnel_fails_closed() {
        let mut value = input();
        value.content.conversions_7d = 900;
        assert!(summarize(&value).is_err());
    }

    #[test]
    fn affiliate_variance_must_reconcile() {
        let mut value = input();
        value.affiliate_variance_mtd_minor = 0;
        assert!(summarize(&value).is_err());
    }

    #[test]
    fn incomplete_margin_is_visible() {
        let mut value = input();
        value.contribution_margin_mtd_minor = None;
        value.unclassified_expense_entry_count = 2;
        let summary = summarize(&value).unwrap();
        assert!(summary
            .alerts
            .iter()
            .any(|alert| alert.title == "Margin evidence incomplete"));
    }

    #[test]
    fn invalid_target_fails_closed() {
        let mut value = input();
        value.revenue_target_minor = 0;
        assert!(summarize(&value).is_err());
    }

    #[test]
    fn signed_revenue_and_cash_are_preserved_for_truthful_alerting() {
        let mut value = input();
        value.cash_minor = -1;
        value.revenue_mtd_minor = -10;
        value.revenue_last_30d_minor = -5;
        value.revenue_lifetime_minor = -20;
        value.daily_revenue[0].revenue_minor = -30;
        value.affiliate_variance_mtd_minor = 0;
        let summary = summarize(&value).unwrap();
        assert!(summary
            .alerts
            .iter()
            .any(|alert| alert.title == "Liquidity risk"));
    }

    #[test]
    fn no_exceptions_creates_healthy_state() {
        let mut value = input();
        value.affiliate_variance_mtd_minor = 0;
        value.revenue_mtd_minor = 0;
        value.daily_revenue.clear();
        let summary = summarize(&value).unwrap();
        assert!(summary
            .alerts
            .iter()
            .any(|alert| alert.kind == AlertKind::Healthy));
    }
}
