#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use tiktok_live_engine::{LiveEvent, LiveEventKind, LiveMode};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttentionAction {
    Respond,
    Defer,
    Ignore,
    Escalate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttentionReason {
    PurchaseIntent,
    Objection,
    Gift,
    PkMoment,
    HighEngagement,
    SafetyEscalation,
    Cooldown,
    RateLimited,
    LowSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttentionPolicy {
    pub response_cooldown_seconds: i64,
    pub max_responses_per_window: u32,
    pub response_window_seconds: i64,
    pub minimum_priority: u8,
    pub priority_bypass_cooldown: u8,
    pub gift_response_threshold_minor: u128,
}

impl Default for AttentionPolicy {
    fn default() -> Self {
        Self {
            response_cooldown_seconds: 8,
            max_responses_per_window: 10,
            response_window_seconds: 60,
            minimum_priority: 45,
            priority_bypass_cooldown: 90,
            gift_response_threshold_minor: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttentionContext {
    pub last_response_at_epoch: Option<i64>,
    pub window_started_at_epoch: Option<i64>,
    pub responses_in_window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttentionDecision {
    pub decision_id: Uuid,
    pub company_id: Uuid,
    pub session_id: Uuid,
    pub event_id: String,
    pub action: AttentionAction,
    pub reason: AttentionReason,
    pub priority: u8,
    pub decided_at_epoch: i64,
    pub requires_human: bool,
}

pub fn decide_attention(
    company_id: Uuid,
    session_id: Uuid,
    mode: LiveMode,
    event: &LiveEvent,
    context: &AttentionContext,
    now_epoch: i64,
    policy: &AttentionPolicy,
) -> Result<AttentionDecision, String> {
    if company_id == Uuid::nil() || session_id == Uuid::nil() {
        return Err("attention decision identifiers are required".into());
    }
    if now_epoch <= 0 {
        return Err("attention decision timestamp must be positive".into());
    }
    event.validate()?;
    if context.responses_in_window > policy.max_responses_per_window {
        return Err("attention response count exceeds policy range".into());
    }

    let (base_priority, reason) = classify_event(event, mode, policy);
    let reason = reason.unwrap_or(AttentionReason::LowSignal);
    let priority = base_priority;

    if reason == AttentionReason::SafetyEscalation {
        return Ok(decision(
            company_id,
            session_id,
            event,
            AttentionAction::Escalate,
            reason,
            priority,
            now_epoch,
            true,
        ));
    }

    if context
        .window_started_at_epoch
        .is_some_and(|started| now_epoch.saturating_sub(started) >= policy.response_window_seconds)
    {
        let context = AttentionContext {
            window_started_at_epoch: Some(now_epoch),
            ..*context
        };
        return decide_attention(company_id, session_id, mode, event, &context, now_epoch, policy);
    }

    if context.responses_in_window >= policy.max_responses_per_window {
        return Ok(decision(
            company_id,
            session_id,
            event,
            AttentionAction::Defer,
            AttentionReason::RateLimited,
            priority,
            now_epoch,
            false,
        ));
    }

    if context
        .last_response_at_epoch
        .is_some_and(|last| now_epoch.saturating_sub(last) < policy.response_cooldown_seconds)
        && priority < policy.priority_bypass_cooldown
    {
        return Ok(decision(
            company_id,
            session_id,
            event,
            AttentionAction::Defer,
            AttentionReason::Cooldown,
            priority,
            now_epoch,
            false,
        ));
    }

    if priority < policy.minimum_priority {
        return Ok(decision(
            company_id,
            session_id,
            event,
            AttentionAction::Ignore,
            AttentionReason::LowSignal,
            priority,
            now_epoch,
            false,
        ));
    }

    Ok(decision(
        company_id,
        session_id,
        event,
        AttentionAction::Respond,
        reason,
        priority,
        now_epoch,
        false,
    ))
}

fn classify_event(
    event: &LiveEvent,
    mode: LiveMode,
    policy: &AttentionPolicy,
) -> (u8, Option<AttentionReason>) {
    if matches!(event.kind, LiveEventKind::Comment | LiveEventKind::System)
        && contains_safety_signal(event.text.as_deref().unwrap_or_default())
    {
        return (100, Some(AttentionReason::SafetyEscalation));
    }

    match event.kind {
        LiveEventKind::Gift if event.gift_value_minor >= policy.gift_response_threshold_minor => {
            (95, Some(AttentionReason::Gift))
        }
        LiveEventKind::Comment => {
            let text = event.text.as_deref().unwrap_or_default();
            if contains_objection(text) {
                (88, Some(AttentionReason::Objection))
            } else if contains_purchase_intent(text) {
                (86, Some(AttentionReason::PurchaseIntent))
            } else if matches!(mode, LiveMode::Shopping) && is_question(text) {
                (74, Some(AttentionReason::PurchaseIntent))
            } else {
                (35, None)
            }
        }
        LiveEventKind::PkUpdate if matches!(mode, LiveMode::Pk | LiveMode::CoHost) => {
            (68, Some(AttentionReason::PkMoment))
        }
        LiveEventKind::Share => (52, Some(AttentionReason::HighEngagement)),
        LiveEventKind::Follow => (48, Some(AttentionReason::HighEngagement)),
        LiveEventKind::Join if matches!(mode, LiveMode::Shopping) => {
            (45, Some(AttentionReason::HighEngagement))
        }
        _ => (15, None),
    }
}

fn decision(
    company_id: Uuid,
    session_id: Uuid,
    event: &LiveEvent,
    action: AttentionAction,
    reason: AttentionReason,
    priority: u8,
    decided_at_epoch: i64,
    requires_human: bool,
) -> AttentionDecision {
    AttentionDecision {
        decision_id: Uuid::new_v4(),
        company_id,
        session_id,
        event_id: event.event_id.clone(),
        action,
        reason,
        priority,
        decided_at_epoch,
        requires_human,
    }
}

fn normalize(text: &str) -> String {
    text.trim().to_lowercase()
}

fn contains_purchase_intent(text: &str) -> bool {
    let text = normalize(text);
    [
        "mua",
        "đặt",
        "link",
        "giá",
        "bao nhiêu",
        "ship",
        "size",
        "màu",
        "còn hàng",
        "voucher",
        "coupon",
        "giỏ hàng",
        "sản phẩm",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn contains_objection(text: &str) -> bool {
    let text = normalize(text);
    [
        "đắt",
        "fake",
        "lừa",
        "thật không",
        "bảo hành",
        "đổi trả",
        "hoàn tiền",
        "khiếu nại",
        "không tốt",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn contains_safety_signal(text: &str) -> bool {
    let text = normalize(text);
    [
        "bản quyền",
        "vi phạm",
        "scam",
        "lừa đảo",
        "báo cáo",
        "report",
        "khiếu nại",
        "cảnh sát",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn is_question(text: &str) -> bool {
    let text = text.trim();
    text.ends_with('?') || text.ends_with('？')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: LiveEventKind, text: &str, gift_value_minor: u128) -> LiveEvent {
        LiveEvent {
            event_id: "event-1".into(),
            room_id: "room-1".into(),
            user_id: Some("viewer-1".into()),
            display_name: Some("Alice".into()),
            kind,
            text: Some(text.into()),
            gift_id: Some("gift-1".into()),
            gift_name: Some("Rose".into()),
            gift_quantity: 1,
            gift_value_minor,
            currency: "VND".into(),
            pk_score: Some(1),
            occurred_at_epoch: 1_750_000_000,
        }
    }

    fn context() -> AttentionContext {
        AttentionContext {
            last_response_at_epoch: None,
            window_started_at_epoch: Some(1_750_000_000),
            responses_in_window: 0,
        }
    }

    #[test]
    fn buyer_intent_is_prioritized() {
        let result = decide_attention(
            Uuid::new_v4(),
            Uuid::new_v4(),
            LiveMode::Shopping,
            &event(LiveEventKind::Comment, "Giá bao nhiêu và link ở đâu?", 0),
            &context(),
            1_750_000_010,
            &AttentionPolicy::default(),
        )
        .unwrap();
        assert_eq!(result.action, AttentionAction::Respond);
        assert_eq!(result.reason, AttentionReason::PurchaseIntent);
        assert!(result.priority >= 80);
    }

    #[test]
    fn objections_take_priority_over_generic_comments() {
        let result = decide_attention(
            Uuid::new_v4(),
            Uuid::new_v4(),
            LiveMode::Shopping,
            &event(LiveEventKind::Comment, "Có bảo hành không, có đổi trả không?", 0),
            &context(),
            1_750_000_010,
            &AttentionPolicy::default(),
        )
        .unwrap();
        assert_eq!(result.action, AttentionAction::Respond);
        assert_eq!(result.reason, AttentionReason::Objection);
    }

    #[test]
    fn cooldown_defers_noncritical_responses() {
        let context = AttentionContext {
            last_response_at_epoch: Some(1_750_000_010),
            window_started_at_epoch: Some(1_750_000_000),
            responses_in_window: 1,
        };
        let result = decide_attention(
            Uuid::new_v4(),
            Uuid::new_v4(),
            LiveMode::Shopping,
            &event(LiveEventKind::Comment, "Giá bao nhiêu?", 0),
            &context,
            1_750_000_014,
            &AttentionPolicy::default(),
        )
        .unwrap();
        assert_eq!(result.action, AttentionAction::Defer);
        assert_eq!(result.reason, AttentionReason::Cooldown);
    }

    #[test]
    fn gift_bypasses_normal_comment_priority_but_not_rate_limit() {
        let context = AttentionContext {
            responses_in_window: 10,
            ..context()
        };
        let result = decide_attention(
            Uuid::new_v4(),
            Uuid::new_v4(),
            LiveMode::Shopping,
            &event(LiveEventKind::Gift, "", 100),
            &context,
            1_750_000_010,
            &AttentionPolicy::default(),
        )
        .unwrap();
        assert_eq!(result.action, AttentionAction::Defer);
        assert_eq!(result.reason, AttentionReason::RateLimited);
    }

    #[test]
    fn safety_signal_escalates_to_human() {
        let result = decide_attention(
            Uuid::new_v4(),
            Uuid::new_v4(),
            LiveMode::Shopping,
            &event(LiveEventKind::Comment, "Báo cáo vì vi phạm", 0),
            &context(),
            1_750_000_010,
            &AttentionPolicy::default(),
        )
        .unwrap();
        assert_eq!(result.action, AttentionAction::Escalate);
        assert!(result.requires_human);
    }

    #[test]
    fn low_signal_is_ignored() {
        let result = decide_attention(
            Uuid::new_v4(),
            Uuid::new_v4(),
            LiveMode::Solo,
            &event(LiveEventKind::Like, "hello", 0),
            &context(),
            1_750_000_010,
            &AttentionPolicy::default(),
        )
        .unwrap();
        assert_eq!(result.action, AttentionAction::Ignore);
        assert_eq!(result.reason, AttentionReason::LowSignal);
    }
}
