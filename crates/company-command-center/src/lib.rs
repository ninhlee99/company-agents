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
    pub gift_count_30d: i128,
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
    if input.revenue_target_minor <= 0 {
        return Err("revenue target must be positive".into());
    }
    if input.cash_minor < 0
        || input.revenue_mtd_minor < 0
        || input.revenue_last_30d_minor < 0
        || input.revenue_lifetime_minor < 0
        || input.revenue_transaction_count < 0
        || input.affiliate_orders_mtd < 0
        || input.active_employee_count < 0
        || input.payroll_due_count < 0
    {
        return Err("command center economic counts cannot be negative".into());
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

    if alerts.is_empty() {
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
