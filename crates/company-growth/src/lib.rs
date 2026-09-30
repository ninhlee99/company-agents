#![forbid(unsafe_code)]

use company_content::{ContentBrief, ContentFormat, CreativeVariant, SuccessMetric};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_TEXT: usize = 2_000;
const MAX_KEY: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompetitorObservation {
    pub company_id: Uuid,
    pub competitor_id: String,
    pub content_key: String,
    pub topic: String,
    pub source: String,
    pub evidence_ref: String,
    pub observed_at_epoch: i64,
    pub audience_signal_bps: u32,
    pub engagement_signal_bps: u32,
    pub offer_presence_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnedContentCoverage {
    pub company_id: Uuid,
    pub topic: String,
    pub coverage_bps: u32,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentWhitespaceGap {
    pub topic: String,
    pub competitor_demand_bps: u32,
    pub owned_coverage_bps: u32,
    pub whitespace_bps: u32,
    pub priority_bps: u32,
    pub evidence_refs: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdsEvidenceCandidate {
    pub company_id: Uuid,
    pub campaign_id: String,
    pub channel: String,
    pub audience_segment: String,
    pub control_conversion_bps: u32,
    pub treatment_conversion_bps: u32,
    pub incremental_revenue_minor: i128,
    pub platform_fees_minor: i128,
    pub refunds_cancellations_minor: i128,
    pub production_ai_cost_minor: i128,
    pub ad_spend_minor: i128,
    pub observed_at_epoch: i64,
    pub time_to_feedback_seconds: i64,
    pub confidence_bps: u32,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdsDecision {
    pub campaign_id: String,
    pub channel: String,
    pub incremental_contribution_margin_minor: i128,
    pub incremental_roas_bps: u32,
    pub lift_bps: i32,
    pub confidence_bps: u32,
    pub score_bps: u32,
    pub decision: String,
    pub evidence_ref: String,
    pub reason: String,
}

fn validate_non_negative_money(name: &str, value: i128) -> Result<(), String> {
    if value < 0 {
        return Err(format!("{name} must not be negative"));
    }
    Ok(())
}

pub fn evaluate_ads_campaigns(
    candidates: &[AdsEvidenceCandidate],
    as_of_epoch: i64,
    max_age_seconds: i64,
    max_results: usize,
) -> Result<Vec<AdsDecision>, String> {
    if as_of_epoch <= 0 {
        return Err("ads as_of_epoch must be positive".into());
    }
    if !(60..=31_536_000).contains(&max_age_seconds) {
        return Err("ads max_age_seconds is outside safe bounds".into());
    }
    if !(1..=500).contains(&max_results) {
        return Err("ads max_results must be between 1 and 500".into());
    }

    for candidate in candidates {
        if candidate.company_id == Uuid::nil()
            || candidate.observed_at_epoch <= 0
            || candidate.observed_at_epoch > as_of_epoch
            || candidate.time_to_feedback_seconds <= 0
        {
            return Err("ads evidence has invalid company/time scope".into());
        }
        require_text("ads.campaign_id", &candidate.campaign_id, MAX_KEY)?;
        require_text("ads.channel", &candidate.channel, MAX_TEXT)?;
        require_text("ads.audience_segment", &candidate.audience_segment, MAX_TEXT)?;
        require_text("ads.evidence_ref", &candidate.evidence_ref, 256)?;
        for (name, value) in [
            ("control_conversion_bps", candidate.control_conversion_bps),
            ("treatment_conversion_bps", candidate.treatment_conversion_bps),
            ("confidence_bps", candidate.confidence_bps),
        ] {
            bounded_signal(name, value)?;
        }
        for (name, value) in [
            ("platform_fees_minor", candidate.platform_fees_minor),
            ("refunds_cancellations_minor", candidate.refunds_cancellations_minor),
            ("production_ai_cost_minor", candidate.production_ai_cost_minor),
            ("ad_spend_minor", candidate.ad_spend_minor),
            ("incremental_revenue_minor", candidate.incremental_revenue_minor),
        ] {
            validate_non_negative_money(name, value)?;
        }
        if candidate.ad_spend_minor <= 0 {
            return Err("ads ad_spend_minor must be positive".into());
        }
    }

    let mut decisions = Vec::new();
    for candidate in candidates.iter().filter(|value| {
        as_of_epoch.saturating_sub(value.observed_at_epoch) <= max_age_seconds
    }) {
        let lift = i64::from(candidate.treatment_conversion_bps)
            - i64::from(candidate.control_conversion_bps);
        let margin = candidate
            .incremental_revenue_minor
            .checked_sub(candidate.platform_fees_minor)
            .and_then(|value| value.checked_sub(candidate.refunds_cancellations_minor))
            .and_then(|value| value.checked_sub(candidate.production_ai_cost_minor))
            .and_then(|value| value.checked_sub(candidate.ad_spend_minor))
            .ok_or("ads contribution margin overflow")?;

        let roas = ((candidate.incremental_revenue_minor as u128)
            .saturating_mul(10_000)
            / candidate.ad_spend_minor as u128)
            .min(u128::from(u32::MAX)) as u32;

        let speed_bps = ((86_400_i64.saturating_mul(10_000)
            / candidate.time_to_feedback_seconds.max(1) as i64)
            .min(10_000)) as u32;
        let margin_efficiency_bps = if margin > 0 {
            ((margin as u128)
                .saturating_mul(10_000)
                / candidate.ad_spend_minor as u128)
                .min(u128::from(u32::MAX)) as u32
        } else {
            0
        };

        let margin_score = if margin <= 0 {
            0
        } else {
            u32::try_from(
                (margin_efficiency_bps as u64 / 2).saturating_add(u64::from(candidate.confidence_bps) / 2),
            )
            .unwrap_or(u32::MAX)
            .min(10_000)
        };
        let lift_score = if lift <= 0 {
            0
        } else {
            u32::try_from(lift.min(10_000) as u64).unwrap_or(10_000)
        };
        let score = ((u64::from(margin_score) * 50
            + u64::from(lift_score) * 20
            + u64::from(candidate.confidence_bps) * 20
            + u64::from(speed_bps) * 10)
            / 100) as u32;

        let decision = if margin <= 0 {
            "HOLD_OR_STOP"
        } else if candidate.confidence_bps < 7_000 || lift <= 0 {
            "CONTROLLED_TEST"
        } else if score >= 7_500 {
            "SCALE_WITH_CAP"
        } else {
            "CONTROLLED_TEST"
        };

        decisions.push(AdsDecision {
            campaign_id: candidate.campaign_id.clone(),
            channel: candidate.channel.clone(),
            incremental_contribution_margin_minor: margin,
            incremental_roas_bps: roas,
            lift_bps: lift as i32,
            confidence_bps: candidate.confidence_bps,
            score_bps: score,
            decision: decision.into(),
            evidence_ref: candidate.evidence_ref.clone(),
            reason: format!(
                "Decision uses incremental contribution margin after fees/refunds/production/ad spend; ROAS is diagnostic only. Lift={} bps, margin efficiency={} bps, confidence={} bps, feedback={}s.",
                lift, margin_efficiency_bps, candidate.confidence_bps, candidate.time_to_feedback_seconds
            ),
        });
    }

    decisions.sort_by(|a, b| {
        b.score_bps
            .cmp(&a.score_bps)
            .then_with(|| a.campaign_id.cmp(&b.campaign_id))
    });
    decisions.truncate(max_results);
    Ok(decisions)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatorIntelligence {
    pub company_id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub specialty_categories: Vec<String>,
    pub audience_quality_bps: u32,
    pub engagement_bps: u32,
    pub click_through_bps: u32,
    pub conversion_bps: u32,
    pub observed_at_epoch: i64,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductMatchCandidate {
    pub company_id: Uuid,
    pub product_id: String,
    pub category: String,
    pub commission_rate_bps: Option<u32>,
    pub rating_bps: Option<u32>,
    pub refund_rate_bps: Option<u32>,
    pub delivery_reliability_bps: Option<u32>,
    pub in_stock: bool,
    pub observed_at_epoch: i64,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatorProductMatch {
    pub creator_id: Uuid,
    pub creator_name: String,
    pub product_id: String,
    pub creator_fit_bps: u32,
    pub product_economics_bps: u32,
    pub match_score_bps: u32,
    pub evidence_refs: Vec<String>,
    pub reason: String,
}

fn bounded_signal(name: &str, value: u32) -> Result<(), String> {
    if value > 10_000 {
        return Err(format!("{name} must be between 0 and 10000 bps"));
    }
    Ok(())
}

pub fn match_creators_to_products(
    creators: &[CreatorIntelligence],
    products: &[ProductMatchCandidate],
    as_of_epoch: i64,
    max_age_seconds: i64,
    max_results: usize,
) -> Result<Vec<CreatorProductMatch>, String> {
    if as_of_epoch <= 0 {
        return Err("creator-product as_of_epoch must be positive".into());
    }
    if !(60..=31_536_000).contains(&max_age_seconds) {
        return Err("creator-product max_age_seconds is outside safe bounds".into());
    }
    if !(1..=500).contains(&max_results) {
        return Err("creator-product max_results must be between 1 and 500".into());
    }
    for creator in creators {
        if creator.company_id == Uuid::nil() || creator.creator_id == Uuid::nil()
            || creator.observed_at_epoch <= 0 || creator.observed_at_epoch > as_of_epoch
        {
            return Err("creator intelligence has invalid company/time scope".into());
        }
        if creator.specialty_categories.is_empty() || creator.specialty_categories.len() > 32 {
            return Err("creator specialty_categories must contain 1..32 values".into());
        }
        for category in &creator.specialty_categories {
            require_text("creator.specialty_category", category, 128)?;
        }
        for (name, value) in [
            ("audience_quality_bps", creator.audience_quality_bps),
            ("engagement_bps", creator.engagement_bps),
            ("click_through_bps", creator.click_through_bps),
            ("conversion_bps", creator.conversion_bps),
        ] {
            bounded_signal(name, value)?;
        }
        require_text("creator.name", &creator.name, MAX_TEXT)?;
        require_text("creator.evidence_ref", &creator.evidence_ref, 256)?;
    }
    for product in products {
        if product.company_id == Uuid::nil()
            || product.observed_at_epoch <= 0
            || product.observed_at_epoch > as_of_epoch
        {
            return Err("product candidate has invalid company/time scope".into());
        }
        require_text("product.product_id", &product.product_id, MAX_KEY)?;
        require_text("product.category", &product.category, MAX_TEXT)?;
        require_text("product.evidence_ref", &product.evidence_ref, 256)?;
        for (name, value) in [
            ("commission_rate_bps", product.commission_rate_bps.unwrap_or(0)),
            ("rating_bps", product.rating_bps.unwrap_or(0)),
            ("refund_rate_bps", product.refund_rate_bps.unwrap_or(0)),
            ("delivery_reliability_bps", product.delivery_reliability_bps.unwrap_or(0)),
        ] {
            bounded_signal(name, value)?;
        }
    }

    let mut results = Vec::new();
    for creator in creators.iter().filter(|value| {
        as_of_epoch.saturating_sub(value.observed_at_epoch) <= max_age_seconds
    }) {
        for product in products.iter().filter(|value| {
            value.in_stock
                && as_of_epoch.saturating_sub(value.observed_at_epoch) <= max_age_seconds
                && value.company_id == creator.company_id
        }) {
            let product_category = product.category.trim().to_ascii_lowercase();
            let category_matches = creator
                .specialty_categories
                .iter()
                .any(|category| category.trim().eq_ignore_ascii_case(&product_category));
            if !category_matches {
                continue;
            }
            let creator_fit = ((10_000_u64 * 40
                + u64::from(creator.audience_quality_bps) * 20
                + u64::from(creator.engagement_bps) * 20
                + u64::from(creator.click_through_bps) * 10
                + u64::from(creator.conversion_bps) * 10)
                / 100) as u32;

            let inverse_refund = 10_000_u32.saturating_sub(product.refund_rate_bps.unwrap_or(0));
            let economics = ((u64::from(product.commission_rate_bps.unwrap_or(0)) * 35
                + u64::from(product.rating_bps.unwrap_or(0)) * 20
                + u64::from(inverse_refund) * 25
                + u64::from(product.delivery_reliability_bps.unwrap_or(0)) * 20)
                / 100) as u32;
            let match_score = ((u64::from(creator_fit) * 65 + u64::from(economics) * 35) / 100) as u32;

            let mut evidence_refs = vec![creator.evidence_ref.clone(), product.evidence_ref.clone()];
            evidence_refs.sort();
            evidence_refs.dedup();
            results.push(CreatorProductMatch {
                creator_id: creator.creator_id,
                creator_name: creator.name.clone(),
                product_id: product.product_id.clone(),
                creator_fit_bps: creator_fit,
                product_economics_bps: economics,
                match_score_bps: match_score,
                evidence_refs,
                reason: format!(
                    "Creator fit {} bps × 65% plus product economics {} bps × 35%; only fresh in-stock evidence is considered.",
                    creator_fit, economics
                ),
            });
        }
    }

    results.sort_by(|a, b| {
        b.match_score_bps
            .cmp(&a.match_score_bps)
            .then_with(|| a.creator_id.cmp(&b.creator_id))
            .then_with(|| a.product_id.cmp(&b.product_id))
    });
    results.truncate(max_results);
    Ok(results)
}

pub fn evaluate_content_whitespace(
    observations: &[CompetitorObservation],
    owned_coverage: &[OwnedContentCoverage],
    as_of_epoch: i64,
    max_age_seconds: i64,
) -> Result<Vec<ContentWhitespaceGap>, String> {
    if as_of_epoch <= 0 {
        return Err("whitespace as_of_epoch must be positive".into());
    }
    if !(60..=31_536_000).contains(&max_age_seconds) {
        return Err("whitespace max_age_seconds is outside safe bounds".into());
    }
    for observation in observations {
        if observation.company_id == Uuid::nil()
            || observation.observed_at_epoch <= 0
            || observation.observed_at_epoch > as_of_epoch
        {
            return Err("competitor observation has invalid company/time scope".into());
        }
        for (name, value) in [
            ("audience_signal_bps", observation.audience_signal_bps),
            ("engagement_signal_bps", observation.engagement_signal_bps),
            ("offer_presence_bps", observation.offer_presence_bps),
        ] {
            if value > 10_000 {
                return Err(format!("{name} must be between 0 and 10000 bps"));
            }
        }
        require_text("competitor_id", &observation.competitor_id, MAX_KEY)?;
        require_text("content_key", &observation.content_key, MAX_KEY)?;
        require_text("topic", &observation.topic, MAX_TEXT)?;
        require_text("source", &observation.source, 256)?;
        require_text("evidence_ref", &observation.evidence_ref, 256)?;
        if as_of_epoch.saturating_sub(observation.observed_at_epoch) > max_age_seconds {
            continue;
        }
    }
    for coverage in owned_coverage {
        if coverage.company_id == Uuid::nil() {
            return Err("owned coverage company_id is required".into());
        }
        require_text("coverage.topic", &coverage.topic, MAX_TEXT)?;
        require_text("coverage.evidence_ref", &coverage.evidence_ref, 256)?;
        if coverage.coverage_bps > 10_000 {
            return Err("coverage_bps must be between 0 and 10000 bps".into());
        }
    }

    let mut topics: std::collections::BTreeMap<String, (Vec<u32>, Vec<String>)> =
        std::collections::BTreeMap::new();
    for observation in observations {
        if as_of_epoch.saturating_sub(observation.observed_at_epoch) > max_age_seconds {
            continue;
        }
        let demand = (
            u64::from(observation.audience_signal_bps) * 50
                + u64::from(observation.engagement_signal_bps) * 30
                + u64::from(observation.offer_presence_bps) * 20
        ) / 100;
        let entry = topics
            .entry(observation.topic.trim().to_ascii_lowercase())
            .or_insert_with(|| (Vec::new(), Vec::new()));
        entry.0.push(demand as u32);
        entry.1.push(observation.evidence_ref.clone());
    }

    let mut gaps = Vec::new();
    for (topic, (signals, mut evidence_refs)) in topics {
        let competitor_demand_bps =
            (signals.iter().map(|value| u64::from(*value)).sum::<u64>() / signals.len() as u64)
                as u32;
        let owned_coverage_bps = owned_coverage
            .iter()
            .filter(|coverage| coverage.topic.trim().eq_ignore_ascii_case(&topic))
            .map(|coverage| coverage.coverage_bps)
            .max()
            .unwrap_or(0);
        let whitespace_bps = competitor_demand_bps.saturating_sub(owned_coverage_bps);
        let priority_bps =
            ((u64::from(competitor_demand_bps) * 60 + u64::from(whitespace_bps) * 40) / 100) as u32;
        if whitespace_bps == 0 {
            continue;
        }
        for coverage in owned_coverage
            .iter()
            .filter(|coverage| coverage.topic.trim().eq_ignore_ascii_case(&topic))
        {
            evidence_refs.push(coverage.evidence_ref.clone());
        }
        evidence_refs.sort();
        evidence_refs.dedup();
        gaps.push(ContentWhitespaceGap {
            topic: topic.clone(),
            competitor_demand_bps,
            owned_coverage_bps,
            whitespace_bps,
            priority_bps,
            evidence_refs,
            reason: format!(
                "Competitor evidence indicates demand at {} bps while owned-content coverage is {} bps.",
                competitor_demand_bps, owned_coverage_bps
            ),
        });
    }

    gaps.sort_by(|a, b| b.priority_bps.cmp(&a.priority_bps).then_with(|| a.topic.cmp(&b.topic)));
    Ok(gaps)
}

pub struct TrendSignal {
    pub company_id: Uuid,
    pub trend_key: String,
    pub topic: String,
    pub source: String,
    pub evidence_ref: String,
    pub observed_at_epoch: i64,
    pub velocity_bps: u32,
    pub audience_fit_bps: u32,
    pub product_fit_bps: u32,
    pub contentability_bps: u32,
    pub competition_bps: u32,
    pub confidence_bps: u32,
    pub product_ref: Option<String>,
    pub offer_ref: Option<String>,
    pub content_format: ContentFormat,
    pub max_budget_minor: i128,
    pub max_loss_minor: i128,
    pub max_duration_seconds: u32,
    pub success_metric: SuccessMetric,
    pub success_threshold_bps: u32,
    pub policy_evidence_ref: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendDecision {
    Pursue,
    Monitor,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrendEvaluation {
    pub score_bps: u32,
    pub decision: TrendDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentPlan {
    pub brief: ContentBrief,
    pub variant: CreativeVariant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Opportunity {
    pub id: Uuid,
    pub company_id: Uuid,
    pub trend_id: Uuid,
    pub opportunity_key: String,
    pub title: String,
    pub score_bps: u32,
    pub confidence_bps: u32,
    pub policy_evidence_ref: String,
    pub plan: ContentPlan,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpportunityStatus {
    Ready,
    ContentCreated,
}

pub fn validate_trend(signal: &TrendSignal) -> Result<(), String> {
    if signal.company_id == Uuid::nil() {
        return Err("trend company_id is required".into());
    }
    require_text("trend_key", &signal.trend_key, MAX_KEY)?;
    require_text("topic", &signal.topic, MAX_TEXT)?;
    require_text("source", &signal.source, 256)?;
    require_text("evidence_ref", &signal.evidence_ref, 256)?;
    require_text("policy_evidence_ref", &signal.policy_evidence_ref, 256)?;
    if signal.observed_at_epoch <= 0 {
        return Err("trend observed_at_epoch must be positive".into());
    }
    for (field, value) in [
        ("velocity_bps", signal.velocity_bps),
        ("audience_fit_bps", signal.audience_fit_bps),
        ("product_fit_bps", signal.product_fit_bps),
        ("contentability_bps", signal.contentability_bps),
        ("competition_bps", signal.competition_bps),
        ("confidence_bps", signal.confidence_bps),
        ("success_threshold_bps", signal.success_threshold_bps),
    ] {
        if value > 10_000 {
            return Err(format!("{field} must be between 0 and 10000 bps"));
        }
    }
    if signal.max_budget_minor < 0 || signal.max_loss_minor < 0 {
        return Err("growth budget and max loss cannot be negative".into());
    }
    if signal.max_loss_minor < signal.max_budget_minor {
        return Err("max loss cannot be below max budget".into());
    }
    if !(1..=86_400).contains(&signal.max_duration_seconds) {
        return Err("growth content duration is outside safe bounds".into());
    }
    if let Some(value) = &signal.product_ref {
        require_text("product_ref", value, 256)?;
    }
    if let Some(value) = &signal.offer_ref {
        require_text("offer_ref", value, 256)?;
    }
    Ok(())
}

pub fn evaluate_trend(signal: &TrendSignal) -> Result<TrendEvaluation, String> {
    validate_trend(signal)?;
    let inverse_competition = 10_000u32 - signal.competition_bps;
    let weighted = u64::from(signal.velocity_bps) * 25
        + u64::from(signal.audience_fit_bps) * 20
        + u64::from(signal.product_fit_bps) * 25
        + u64::from(signal.contentability_bps) * 20
        + u64::from(inverse_competition) * 10;
    let score_bps = (weighted / 100) as u32;
    let decision = if score_bps >= 6_500 && signal.confidence_bps >= 6_000 {
        TrendDecision::Pursue
    } else if score_bps >= 4_500 && signal.confidence_bps >= 4_000 {
        TrendDecision::Monitor
    } else {
        TrendDecision::Reject
    };
    Ok(TrendEvaluation { score_bps, decision })
}

pub fn build_content_plan(signal: &TrendSignal) -> Result<ContentPlan, String> {
    validate_trend(signal)?;
    let evaluation = evaluate_trend(signal)?;
    if evaluation.decision != TrendDecision::Pursue {
        return Err("only pursue opportunities can generate a content plan".into());
    }

    let hook = truncate(&format!(
        "See a verified use case for {} before you buy",
        signal.topic
    ), 240);

    let first_frame = "Use-case visual + source-backed context".to_string();
    let brief = ContentBrief {
        hypothesis: truncate(
            &format!(
                "The trend around {} creates qualified attention for the selected commerce use case.",
                signal.topic
            ),
            MAX_TEXT,
        ),
        audience: truncate(&format!("Trend-aligned audience for {}", signal.topic), MAX_TEXT),
        format: signal.content_format,
        product_ref: signal.product_ref.clone(),
        offer_ref: signal.offer_ref.clone(),
        disclosure_required: true,
        expected_cost_minor: signal.max_budget_minor.min(signal.max_loss_minor),
        max_loss_minor: signal.max_loss_minor,
        max_duration_seconds: signal.max_duration_seconds,
        success_metric: signal.success_metric,
        success_threshold_bps: signal.success_threshold_bps,
    };
    let variant = CreativeVariant {
        variant_key: truncate(&format!("{}:v1", signal.trend_key), 128),
        hook,
        first_frame,
        emotion: "curiosity".into(),
        pacing: "front-load-context".into(),
        scene_count: 6,
        text_density: "low".into(),
        voice_speed: "clear".into(),
        product_placement: "show only where evidence supports the use case".into(),
        cta: "Review the product details and offer evidence before purchase".into(),
        comment_trigger: "Ask for the use case or objection".into(),
        music_style: "platform-safe background".into(),
        visual_style: "proof-first handheld".into(),
    };
    company_content::validate_brief(&brief)?;
    company_content::validate_variant(&variant)?;
    Ok(ContentPlan { brief, variant })
}

pub fn opportunity_from_trend(
    trend_id: Uuid,
    signal: &TrendSignal,
) -> Result<Option<Opportunity>, String> {
    let evaluation = evaluate_trend(signal)?;
    if evaluation.decision != TrendDecision::Pursue {
        return Ok(None);
    }
    let plan = build_content_plan(signal)?;
    Ok(Some(Opportunity {
        id: Uuid::new_v4(),
        company_id: signal.company_id,
        trend_id,
        opportunity_key: format!("{}:opportunity", signal.trend_key),
        title: format!("Commerce opportunity: {}", signal.topic),
        score_bps: evaluation.score_bps,
        confidence_bps: signal.confidence_bps,
        policy_evidence_ref: signal.policy_evidence_ref.clone(),
        plan,
    }))
}

fn require_text(field: &str, value: &str, max: usize) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} is required"));
    }
    if trimmed.len() > max {
        return Err(format!("{field} exceeds {max} bytes"));
    }
    if trimmed.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err(format!("{field} contains control characters"));
    }
    Ok(())
}

fn truncate(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal() -> TrendSignal {
        TrendSignal {
            company_id: Uuid::new_v4(),
            trend_key: "trend-1".into(),
            topic: "desk organization".into(),
            source: "verified-trend-feed".into(),
            evidence_ref: "trend-evidence-1".into(),
            observed_at_epoch: 1_700_000_000,
            velocity_bps: 8_000,
            audience_fit_bps: 8_000,
            product_fit_bps: 8_500,
            contentability_bps: 8_000,
            competition_bps: 2_000,
            confidence_bps: 8_000,
            product_ref: Some("product-1".into()),
            offer_ref: None,
            content_format: ContentFormat::ShortVideo,
            max_budget_minor: 100,
            max_loss_minor: 500,
            max_duration_seconds: 60,
            success_metric: SuccessMetric::ClickThroughRate,
            success_threshold_bps: 500,
            policy_evidence_ref: "policy-2026-09".into(),
        }
    }

    #[test]
    fn ads_engine_prioritizes_incremental_margin_over_roas() {
        let company_id = Uuid::new_v4();
        let high_roas_low_margin = AdsEvidenceCandidate {
            company_id,
            campaign_id: "roas-high".into(),
            channel: "search".into(),
            audience_segment: "test".into(),
            control_conversion_bps: 300,
            treatment_conversion_bps: 320,
            incremental_revenue_minor: 20_000,
            platform_fees_minor: 1_000,
            refunds_cancellations_minor: 1_000,
            production_ai_cost_minor: 1_000,
            ad_spend_minor: 2_000,
            observed_at_epoch: 1_800_000_000,
            time_to_feedback_seconds: 86_400,
            confidence_bps: 8_500,
            evidence_ref: "e-roas-high".into(),
        };
        let lower_roas_better_margin = AdsEvidenceCandidate {
            campaign_id: "margin-better".into(),
            incremental_revenue_minor: 30_000,
            platform_fees_minor: 3_000,
            refunds_cancellations_minor: 1_000,
            production_ai_cost_minor: 1_000,
            ad_spend_minor: 5_000,
            treatment_conversion_bps: 600,
            control_conversion_bps: 500,
            ..high_roas_low_margin.clone()
        };
        let decisions = evaluate_ads_campaigns(
            &[high_roas_low_margin, lower_roas_better_margin],
            1_800_000_100,
            86_400,
            10,
        ).unwrap();
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].campaign_id, "margin-better");
        assert!(decisions.iter().all(|value| value.incremental_contribution_margin_minor > 0));
    }

    #[test]
    fn ads_engine_stops_when_incremental_margin_is_non_positive() {
        let company_id = Uuid::new_v4();
        let candidate = AdsEvidenceCandidate {
            company_id,
            campaign_id: "loss".into(),
            channel: "social".into(),
            audience_segment: "test".into(),
            control_conversion_bps: 500,
            treatment_conversion_bps: 400,
            incremental_revenue_minor: 10_000,
            platform_fees_minor: 2_000,
            refunds_cancellations_minor: 1_000,
            production_ai_cost_minor: 1_000,
            ad_spend_minor: 6_000,
            observed_at_epoch: 1_800_000_000,
            time_to_feedback_seconds: 86_400,
            confidence_bps: 9_000,
            evidence_ref: "e-loss".into(),
        };
        let decisions = evaluate_ads_campaigns(&[candidate], 1_800_000_100, 86_400, 10).unwrap();
        assert_eq!(decisions[0].decision, "HOLD_OR_STOP");
        assert!(decisions[0].incremental_contribution_margin_minor <= 0);
    }

    #[test]
    fn creator_product_matching_is_deterministic_and_economic() {
        let company_id = Uuid::new_v4();
        let creator = CreatorIntelligence {
            company_id,
            creator_id: Uuid::new_v4(),
            name: "Creator A".into(),
            specialty_categories: vec!["desk".into()],
            audience_quality_bps: 8_000,
            engagement_bps: 8_500,
            click_through_bps: 7_000,
            conversion_bps: 6_000,
            observed_at_epoch: 1_800_000_000,
            evidence_ref: "creator-evidence".into(),
        };
        let product_good = ProductMatchCandidate {
            company_id,
            product_id: "product-good".into(),
            category: "desk".into(),
            commission_rate_bps: Some(8_000),
            rating_bps: Some(9_000),
            refund_rate_bps: Some(500),
            delivery_reliability_bps: Some(9_000),
            in_stock: true,
            observed_at_epoch: 1_800_000_050,
            evidence_ref: "product-good-evidence".into(),
        };
        let product_bad = ProductMatchCandidate {
            company_id,
            product_id: "product-bad".into(),
            category: "desk".into(),
            commission_rate_bps: Some(8_000),
            rating_bps: Some(9_000),
            refund_rate_bps: Some(9_000),
            delivery_reliability_bps: Some(9_000),
            in_stock: true,
            observed_at_epoch: 1_800_000_050,
            evidence_ref: "product-bad-evidence".into(),
        };
        let matches = match_creators_to_products(
            &[creator.clone()],
            &[product_bad.clone(), product_good.clone()],
            1_800_000_100,
            86_400,
            10,
        )
        .unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].product_id, "product-good");
        let again = match_creators_to_products(
            &[creator],
            &[product_bad, product_good],
            1_800_000_100,
            86_400,
            10,
        )
        .unwrap();
        assert_eq!(matches, again);
    }

    #[test]
    fn creator_product_matching_requires_specialty_category_alignment() {
        let company_id = Uuid::new_v4();
        let creator = CreatorIntelligence {
            company_id,
            creator_id: Uuid::new_v4(),
            name: "Creator A".into(),
            specialty_categories: vec!["desk".into()],
            audience_quality_bps: 9_000,
            engagement_bps: 9_000,
            click_through_bps: 9_000,
            conversion_bps: 9_000,
            observed_at_epoch: 1_800_000_000,
            evidence_ref: "creator-evidence".into(),
        };
        let unrelated = ProductMatchCandidate {
            company_id,
            product_id: "unrelated".into(),
            category: "beauty".into(),
            commission_rate_bps: Some(10_000),
            rating_bps: Some(10_000),
            refund_rate_bps: Some(0),
            delivery_reliability_bps: Some(10_000),
            in_stock: true,
            observed_at_epoch: 1_800_000_050,
            evidence_ref: "unrelated-evidence".into(),
        };
        let matches = match_creators_to_products(&[creator], &[unrelated], 1_800_000_100, 86_400, 10).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn creator_product_matching_excludes_stale_or_out_of_stock_products() {
        let company_id = Uuid::new_v4();
        let creator = CreatorIntelligence {
            company_id,
            creator_id: Uuid::new_v4(),
            name: "Creator A".into(),
            specialty_categories: vec!["desk".into()],
            audience_quality_bps: 8_000,
            engagement_bps: 8_000,
            click_through_bps: 8_000,
            conversion_bps: 8_000,
            observed_at_epoch: 1_800_000_000,
            evidence_ref: "creator-evidence".into(),
        };
        let stale = ProductMatchCandidate {
            company_id,
            product_id: "stale".into(),
            category: "desk".into(),
            commission_rate_bps: Some(9_000),
            rating_bps: Some(9_000),
            refund_rate_bps: Some(0),
            delivery_reliability_bps: Some(9_000),
            in_stock: true,
            observed_at_epoch: 1_700_000_000,
            evidence_ref: "stale-evidence".into(),
        };
        let out_of_stock = ProductMatchCandidate {
            product_id: "oos".into(),
            in_stock: false,
            ..stale.clone()
        };
        let matches = match_creators_to_products(&[creator], &[stale, out_of_stock], 1_800_000_100, 86_400, 10).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn whitespace_is_deterministic_and_evidence_backed() {
        let company_id = Uuid::new_v4();
        let observations = vec![
            CompetitorObservation {
                company_id,
                competitor_id: "competitor-a".into(),
                content_key: "a-1".into(),
                topic: "standing desk".into(),
                source: "verified-competitor-feed".into(),
                evidence_ref: "comp-a-1".into(),
                observed_at_epoch: 1_800_000_000,
                audience_signal_bps: 9_000,
                engagement_signal_bps: 8_000,
                offer_presence_bps: 7_000,
            },
            CompetitorObservation {
                company_id,
                competitor_id: "competitor-b".into(),
                content_key: "b-1".into(),
                topic: "standing desk".into(),
                source: "verified-competitor-feed".into(),
                evidence_ref: "comp-b-1".into(),
                observed_at_epoch: 1_800_000_100,
                audience_signal_bps: 8_000,
                engagement_signal_bps: 7_000,
                offer_presence_bps: 9_000,
            },
        ];
        let owned = vec![OwnedContentCoverage {
            company_id,
            topic: "standing desk".into(),
            coverage_bps: 2_000,
            evidence_ref: "owned-coverage-1".into(),
        }];
        let gaps = evaluate_content_whitespace(&observations, &owned, 1_800_000_200, 86_400).unwrap();
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].whitespace_bps > 0);
        assert!(gaps[0].priority_bps >= gaps[0].whitespace_bps);
        assert_eq!(gaps[0].evidence_refs, vec!["comp-a-1", "comp-b-1", "owned-coverage-1"]);
    }

    #[test]
    fn owned_coverage_requires_company_identity() {
        let observation_company = Uuid::new_v4();
        let coverage = OwnedContentCoverage {
            company_id: Uuid::nil(),
            topic: "standing desk".into(),
            coverage_bps: 1_000,
            evidence_ref: "owned-1".into(),
        };
        let observation = CompetitorObservation {
            company_id: observation_company,
            competitor_id: "competitor-a".into(),
            content_key: "a-1".into(),
            topic: "standing desk".into(),
            source: "verified-competitor-feed".into(),
            evidence_ref: "comp-a-1".into(),
            observed_at_epoch: 1_800_000_000,
            audience_signal_bps: 9_000,
            engagement_signal_bps: 9_000,
            offer_presence_bps: 9_000,
        };
        assert!(evaluate_content_whitespace(
            &[observation],
            &[coverage],
            1_800_000_100,
            86_400
        )
        .is_err());
    }

    #[test]
    fn stale_competitor_evidence_is_excluded() {
        let company_id = Uuid::new_v4();
        let observation = CompetitorObservation {
            company_id,
            competitor_id: "competitor-a".into(),
            content_key: "a-1".into(),
            topic: "old topic".into(),
            source: "verified-competitor-feed".into(),
            evidence_ref: "comp-a-1".into(),
            observed_at_epoch: 1_000,
            audience_signal_bps: 9_000,
            engagement_signal_bps: 9_000,
            offer_presence_bps: 9_000,
        };
        let gaps = evaluate_content_whitespace(&[observation], &[], 10_000, 60).unwrap();
        assert!(gaps.is_empty());
    }

    #[test]
    fn scoring_is_deterministic() {
        let value = evaluate_trend(&signal()).unwrap();
        assert_eq!(value.score_bps, 8_125);
        assert_eq!(value.decision, TrendDecision::Pursue);
    }

    #[test]
    fn weak_confidence_never_auto_pursues() {
        let mut value = signal();
        value.confidence_bps = 5_999;
        assert_eq!(evaluate_trend(&value).unwrap().decision, TrendDecision::Monitor);
    }

    #[test]
    fn high_competition_reduces_score() {
        let mut value = signal();
        value.competition_bps = 10_000;
        assert!(evaluate_trend(&value).unwrap().score_bps < 7_500);
    }

    #[test]
    fn plan_requires_pursue() {
        let mut value = signal();
        value.product_fit_bps = 1_000;
        assert!(build_content_plan(&value).is_err());
    }

    #[test]
    fn monitor_does_not_create_opportunity() {
        let mut value = signal();
        value.confidence_bps = 4_000;
        assert!(opportunity_from_trend(Uuid::new_v4(), &value).unwrap().is_none());
    }

    #[test]
    fn generated_plan_always_requires_disclosure() {
        let plan = build_content_plan(&signal()).unwrap();
        assert!(plan.brief.disclosure_required);
    }

    #[test]
    fn cross_field_budget_is_rejected() {
        let mut value = signal();
        value.max_loss_minor = 99;
        assert!(validate_trend(&value).is_err());
    }
}
