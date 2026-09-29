#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_TEXT: usize = 2_000;
const MAX_HOOK: usize = 240;
const MAX_SCENE_COUNT: u8 = 60;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentFormat {
    ShortVideo,
    LiveSegment,
    Story,
    Carousel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuccessMetric {
    Views,
    ClickThroughRate,
    ConversionRate,
    Commission,
    ContributionMargin,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentStatus {
    Draft,
    Approved,
    Rendered,
    Published,
    Measured,
    Paused,
    Killed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentDecision {
    Scale,
    Iterate,
    Pause,
    Kill,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentBrief {
    pub hypothesis: String,
    pub audience: String,
    pub format: ContentFormat,
    pub product_ref: Option<String>,
    pub offer_ref: Option<String>,
    pub disclosure_required: bool,
    pub expected_cost_minor: i128,
    pub max_loss_minor: i128,
    pub max_duration_seconds: u32,
    pub success_metric: SuccessMetric,
    pub success_threshold_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreativeVariant {
    pub variant_key: String,
    pub hook: String,
    pub first_frame: String,
    pub emotion: String,
    pub pacing: String,
    pub scene_count: u8,
    pub text_density: String,
    pub voice_speed: String,
    pub product_placement: String,
    pub cta: String,
    pub comment_trigger: String,
    pub music_style: String,
    pub visual_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentItem {
    pub id: Uuid,
    pub company_id: Uuid,
    pub brief: ContentBrief,
    pub variant: CreativeVariant,
    pub status: ContentStatus,
    pub decision: Option<ContentDecision>,
}

pub fn validate_brief(brief: &ContentBrief) -> Result<(), String> {
    require_text("hypothesis", &brief.hypothesis, MAX_TEXT)?;
    require_text("audience", &brief.audience, MAX_TEXT)?;
    if let Some(value) = &brief.product_ref {
        require_text("product_ref", value, 256)?;
    }
    if let Some(value) = &brief.offer_ref {
        require_text("offer_ref", value, 256)?;
    }
    if brief.expected_cost_minor < 0 || brief.max_loss_minor < 0 {
        return Err("content cost and max loss cannot be negative".into());
    }
    if brief.max_loss_minor < brief.expected_cost_minor {
        return Err("max loss cannot be below expected cost".into());
    }
    if !(1..=86_400).contains(&brief.max_duration_seconds) {
        return Err("content duration is outside safe bounds".into());
    }
    if brief.success_threshold_bps > 10_000 {
        return Err("success threshold must be between 0 and 10000 bps".into());
    }
    Ok(())
}

pub fn validate_variant(variant: &CreativeVariant) -> Result<(), String> {
    require_text("variant_key", &variant.variant_key, 128)?;
    require_text("hook", &variant.hook, MAX_HOOK)?;
    require_text("first_frame", &variant.first_frame, MAX_TEXT)?;
    require_text("emotion", &variant.emotion, 256)?;
    require_text("pacing", &variant.pacing, 256)?;
    if !(1..=MAX_SCENE_COUNT).contains(&variant.scene_count) {
        return Err("scene_count is outside safe bounds".into());
    }
    require_text("text_density", &variant.text_density, 256)?;
    require_text("voice_speed", &variant.voice_speed, 256)?;
    require_text("product_placement", &variant.product_placement, 256)?;
    require_text("cta", &variant.cta, MAX_TEXT)?;
    require_text("comment_trigger", &variant.comment_trigger, MAX_TEXT)?;
    require_text("music_style", &variant.music_style, 256)?;
    require_text("visual_style", &variant.visual_style, 256)?;
    Ok(())
}

pub fn validate_item(item: &ContentItem) -> Result<(), String> {
    validate_brief(&item.brief)?;
    validate_variant(&item.variant)?;
    if item.company_id == Uuid::nil() || item.id == Uuid::nil() {
        return Err("content identifiers are required".into());
    }
    if matches!(item.status, ContentStatus::Published | ContentStatus::Measured)
        && item.decision.is_none()
    {
        return Err("published or measured content requires a decision".into());
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn brief() -> ContentBrief {
        ContentBrief {
            hypothesis: "A proof-first hook improves product clicks".into(),
            audience: "Vietnam mobile shoppers".into(),
            format: ContentFormat::ShortVideo,
            product_ref: Some("product-1".into()),
            offer_ref: None,
            disclosure_required: true,
            expected_cost_minor: 100,
            max_loss_minor: 500,
            max_duration_seconds: 60,
            success_metric: SuccessMetric::ClickThroughRate,
            success_threshold_bps: 500,
        }
    }

    fn variant() -> CreativeVariant {
        CreativeVariant {
            variant_key: "proof-hook-a".into(),
            hook: "Watch the result before you buy".into(),
            first_frame: "Finished result + product".into(),
            emotion: "curiosity".into(),
            pacing: "fast-first-3s".into(),
            scene_count: 6,
            text_density: "low".into(),
            voice_speed: "1.05x".into(),
            product_placement: "show at second 2".into(),
            cta: "Check the product card".into(),
            comment_trigger: "Ask for the use case".into(),
            music_style: "light beat".into(),
            visual_style: "clean handheld".into(),
        }
    }

    #[test]
    fn valid_item_passes() {
        let item = ContentItem {
            id: Uuid::new_v4(),
            company_id: Uuid::new_v4(),
            brief: brief(),
            variant: variant(),
            status: ContentStatus::Draft,
            decision: None,
        };
        assert!(validate_item(&item).is_ok());
    }

    #[test]
    fn loss_below_expected_cost_is_rejected() {
        let mut value = brief();
        value.max_loss_minor = 50;
        assert!(validate_brief(&value).is_err());
    }

    #[test]
    fn oversized_hook_is_rejected() {
        let mut value = variant();
        value.hook = "x".repeat(MAX_HOOK + 1);
        assert!(validate_variant(&value).is_err());
    }

    #[test]
    fn published_content_requires_decision() {
        let item = ContentItem {
            id: Uuid::new_v4(),
            company_id: Uuid::new_v4(),
            brief: brief(),
            variant: variant(),
            status: ContentStatus::Published,
            decision: None,
        };
        assert!(validate_item(&item).is_err());
    }

    #[test]
    fn json_roundtrip_is_stable() {
        let item = ContentItem {
            id: Uuid::new_v4(),
            company_id: Uuid::new_v4(),
            brief: brief(),
            variant: variant(),
            status: ContentStatus::Draft,
            decision: None,
        };
        let encoded = serde_json::to_string(&item).unwrap();
        let decoded: ContentItem = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, item);
    }
}
