#![forbid(unsafe_code)]

use company_content::{ContentBrief, ContentFormat, CreativeVariant, SuccessMetric};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_TEXT: usize = 2_000;
const MAX_KEY: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
