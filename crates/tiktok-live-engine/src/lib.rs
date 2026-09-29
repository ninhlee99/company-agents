#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

const MAX_TEXT_LEN: usize = 2_000;
const MAX_EVENT_ID_LEN: usize = 128;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LiveMode { Solo, CoHost, Pk, Game, Story, Music, Shopping }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveSession {
    pub id: Uuid, pub company_id: Uuid, pub room_id: Option<String>,
    pub title: String, pub mode: LiveMode, pub started_at_epoch: i64,
    pub approved_for_external_publish: bool,
}

impl LiveSession {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() || self.title.len() > 200 { return Err("live title is invalid".into()); }
        if let Some(room_id) = self.room_id.as_deref() {
            if room_id.trim().is_empty() || room_id.len() > 256 { return Err("room_id is invalid".into()); }
        }
        if self.started_at_epoch <= 0 { return Err("started_at_epoch must be positive".into()); }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LiveEventKind { Join, Comment, Follow, Share, Like, Gift, CoHost, PkUpdate, System }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveEvent {
    pub event_id: String, pub room_id: String, pub user_id: Option<String>, pub display_name: Option<String>,
    pub kind: LiveEventKind, pub text: Option<String>, pub gift_id: Option<String>, pub gift_name: Option<String>,
    pub gift_quantity: u64, pub gift_value_minor: u128, pub currency: String, pub pk_score: Option<u64>,
    #[serde(default)]
    pub viewer_value_bps: Option<u32>,
    pub occurred_at_epoch: i64,
}

impl LiveEvent {
    pub fn validate(&self) -> Result<(), String> {
        if self.event_id.trim().is_empty() || self.event_id.len() > MAX_EVENT_ID_LEN { return Err("event_id is invalid".into()); }
        if self.room_id.trim().is_empty() || self.room_id.len() > 256 { return Err("room_id is invalid".into()); }
        if self.gift_quantity == 0 && matches!(self.kind, LiveEventKind::Gift) { return Err("gift_quantity must be positive for gift events".into()); }
        if self.gift_value_minor > 0 && self.currency.trim().is_empty() { return Err("currency is required when gift_value_minor is non-zero".into()); }
        if let Some(text) = self.text.as_deref() { if text.len() > MAX_TEXT_LEN { return Err("event text is too long".into()); } }
        if self.occurred_at_epoch <= 0 { return Err("occurred_at_epoch must be positive".into()); }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResponseAction { WelcomeViewer, ReplyComment, ThankGift, CelebrateMilestone, StartStoryBeat, StartGameRound, TransitionMusic, ProductMoment, PkCommentary, SafetyEscalation, IdlePrompt }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveResponse { pub action: ResponseAction, pub text: String, pub priority: u8, pub requires_human: bool }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveLedger {
    pub processed_event_ids: HashSet<String>, pub gift_count: u64, pub gift_value_minor: u128,
    pub comments: u64, pub follows: u64, pub shares: u64, pub likes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderGiftStatement {
    pub statement_id: String,
    pub room_id: String,
    pub gift_count: u64,
    pub gross_value_minor: u128,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiftReconciliation {
    pub statement_id: String,
    pub recorded_gift_count: u64,
    pub provider_gift_count: u64,
    pub recorded_value_minor: u128,
    pub provider_value_minor: u128,
    pub count_delta: i128,
    pub value_delta_minor: i128,
    pub matched: bool,
}

pub fn reconcile_gifts(
    ledger: &LiveLedger,
    statement: &ProviderGiftStatement,
) -> Result<GiftReconciliation, String> {
    if statement.statement_id.trim().is_empty()
        || statement.room_id.trim().is_empty()
        || statement.currency.trim().is_empty()
    {
        return Err("provider gift statement identifiers/currency are required".into());
    }
    if ledger.gift_value_minor > i128::MAX as u128
        || statement.gross_value_minor > i128::MAX as u128
    {
        return Err("gift values exceed reconciliation range".into());
    }
    let count_delta = ledger.gift_count as i128 - statement.gift_count as i128;
    let value_delta_minor =
        ledger.gift_value_minor as i128 - statement.gross_value_minor as i128;
    Ok(GiftReconciliation {
        statement_id: statement.statement_id.clone(),
        recorded_gift_count: ledger.gift_count,
        provider_gift_count: statement.gift_count,
        recorded_value_minor: ledger.gift_value_minor,
        provider_value_minor: statement.gross_value_minor,
        count_delta,
        value_delta_minor,
        matched: count_delta == 0 && value_delta_minor == 0,
    })
}

impl LiveLedger {
    pub fn ingest(&mut self, event: &LiveEvent) -> Result<bool, String> {
        event.validate()?;
        if !self.processed_event_ids.insert(event.event_id.clone()) { return Ok(false); }
        match event.kind {
            LiveEventKind::Gift => {
                self.gift_count = self.gift_count.checked_add(event.gift_quantity).ok_or_else(|| "gift count overflow".to_string())?;
                self.gift_value_minor = self.gift_value_minor.checked_add(event.gift_value_minor).ok_or_else(|| "gift value overflow".to_string())?;
            }
            LiveEventKind::Comment => self.comments = self.comments.saturating_add(1),
            LiveEventKind::Follow => self.follows = self.follows.saturating_add(1),
            LiveEventKind::Share => self.shares = self.shares.saturating_add(1),
            LiveEventKind::Like => self.likes = self.likes.saturating_add(1),
            _ => {}
        }
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngagementPolicy {
    pub thank_gift_threshold_minor: u128, pub milestone_gift_count: u64, pub max_response_chars: usize,
    pub allow_product_moments: bool, pub allow_music_transitions: bool, pub allow_game_rounds: bool,
}

impl Default for EngagementPolicy {
    fn default() -> Self { Self { thank_gift_threshold_minor: 1, milestone_gift_count: 25, max_response_chars: 280, allow_product_moments: true, allow_music_transitions: true, allow_game_rounds: true } }
}

pub fn decide_response(mode: LiveMode, event: &LiveEvent, ledger: &LiveLedger, policy: &EngagementPolicy) -> LiveResponse {
    let display_name = event.display_name.as_deref().unwrap_or("bạn");
    let response = match event.kind {
        LiveEventKind::Gift if event.gift_value_minor >= policy.thank_gift_threshold_minor => {
            let gift = event.gift_name.as_deref().unwrap_or("món quà");
            if ledger.gift_count >= policy.milestone_gift_count {
                LiveResponse { action: ResponseAction::CelebrateMilestone, text: format!("Cảm ơn {display_name} vì {gift}! Cả phòng vừa chạm một cột mốc quà tặng 🎉"), priority: 10, requires_human: false }
            } else {
                LiveResponse { action: ResponseAction::ThankGift, text: format!("Cảm ơn {display_name} vì {gift}! Mình ghi nhận món quà của bạn 💛"), priority: 10, requires_human: false }
            }
        }
        LiveEventKind::Comment => {
            let text = event.text.as_deref().unwrap_or_default().trim();
            if text.is_empty() {
                LiveResponse { action: ResponseAction::IdlePrompt, text: "Mọi người muốn chơi game, nghe nhạc hay nghe một câu chuyện?".into(), priority: 2, requires_human: false }
            } else {
                LiveResponse { action: ResponseAction::ReplyComment, text: format!("{display_name}, mình thấy câu hỏi của bạn: {text}"), priority: 7, requires_human: false }
            }
        }
        LiveEventKind::Join => LiveResponse { action: ResponseAction::WelcomeViewer, text: format!("Chào mừng {display_name} vào phòng! Bạn muốn chọn game, chuyện hay nhạc?"), priority: 4, requires_human: false },
        LiveEventKind::Follow => LiveResponse { action: ResponseAction::WelcomeViewer, text: format!("Cảm ơn {display_name} đã follow! Chào mừng bạn đến với Veridara LIVE."), priority: 6, requires_human: false },
        LiveEventKind::Share => LiveResponse { action: ResponseAction::CelebrateMilestone, text: format!("Cảm ơn {display_name} đã chia sẻ LIVE! Mời mọi người cùng tham gia nhé."), priority: 5, requires_human: false },
        LiveEventKind::PkUpdate if matches!(mode, LiveMode::Pk | LiveMode::CoHost) => LiveResponse { action: ResponseAction::PkCommentary, text: format!("PK đang diễn ra — điểm hiện tại: {}. Cùng theo dõi diễn biến tiếp theo!", event.pk_score.unwrap_or(0)), priority: 8, requires_human: false },
        _ if matches!(mode, LiveMode::Game) && policy.allow_game_rounds => LiveResponse { action: ResponseAction::StartGameRound, text: "Vòng game mới bắt đầu: mọi người chọn A, B hay C trong phần chat!".into(), priority: 5, requires_human: false },
        _ if matches!(mode, LiveMode::Story) => LiveResponse { action: ResponseAction::StartStoryBeat, text: "Đến đoạn tiếp theo của câu chuyện rồi — mọi người đoán xem nhân vật sẽ làm gì?".into(), priority: 4, requires_human: false },
        _ if matches!(mode, LiveMode::Music) && policy.allow_music_transitions => LiveResponse { action: ResponseAction::TransitionMusic, text: "Chuyển sang tiết mục tiếp theo. Nếu muốn đổi mood, hãy chọn chill, vui hoặc sôi động.".into(), priority: 4, requires_human: false },
        _ if matches!(mode, LiveMode::Shopping) && policy.allow_product_moments => LiveResponse { action: ResponseAction::ProductMoment, text: "Mình sẽ giới thiệu nhanh sản phẩm đang được ghim và chỉ nêu thông tin đã xác minh.".into(), priority: 5, requires_human: false },
        _ => LiveResponse { action: ResponseAction::IdlePrompt, text: "Mình đang theo dõi phòng LIVE. Hãy gửi câu hỏi hoặc chọn một chủ đề để tiếp tục!".into(), priority: 2, requires_human: false },
    };
    clamp_response(response, policy.max_response_chars)
}

fn clamp_response(mut response: LiveResponse, max_chars: usize) -> LiveResponse {
    if response.text.chars().count() > max_chars { response.text = response.text.chars().take(max_chars).collect(); }
    response
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalLiveCapability { pub live_publish: bool, pub chat_ingest: bool, pub gift_events: bool, pub pk_control: bool, pub music_playback: bool, pub game_overlay: bool }

impl ExternalLiveCapability {
    pub fn production_default() -> Self { Self { live_publish: false, chat_ingest: false, gift_events: false, pk_control: false, music_playback: false, game_overlay: true } }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveLaunchGate { pub capability: ExternalLiveCapability, pub approved: bool, pub stream_destination: Option<String>, pub reason: String }

impl LiveLaunchGate {
    pub fn validate(&self) -> Result<(), String> {
        if self.approved && !self.capability.live_publish { return Err("live launch cannot be approved without a configured publishing capability".into()); }
        if self.approved {
            let destination = self.stream_destination.as_deref().ok_or_else(|| "approved live launch requires a stream destination".to_string())?;
            if !(destination.starts_with("rtmp://") || destination.starts_with("rtmps://")) { return Err("stream destination must be RTMP(S)".into()); }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(kind: LiveEventKind) -> LiveEvent { LiveEvent { event_id: Uuid::new_v4().to_string(), room_id: "room-1".into(), user_id: Some("user-1".into()), display_name: Some("Alice".into()), kind, text: Some("hello".into()), gift_id: Some("gift-1".into()), gift_name: Some("Rose".into()), gift_quantity: 1, gift_value_minor: 10, currency: "USD".into(), pk_score: Some(7), viewer_value_bps: None, occurred_at_epoch: 1_750_000_000 } }
    #[test] fn ledger_is_idempotent_for_replayed_events() { let mut ledger = LiveLedger::default(); let e = event(LiveEventKind::Gift); assert!(ledger.ingest(&e).unwrap()); assert!(!ledger.ingest(&e).unwrap()); assert_eq!(ledger.gift_count, 1); assert_eq!(ledger.gift_value_minor, 10); }
    #[test] fn gift_gets_priority_response_without_promising_money() { let mut ledger = LiveLedger::default(); let e = event(LiveEventKind::Gift); ledger.ingest(&e).unwrap(); let response = decide_response(LiveMode::Solo, &e, &ledger, &EngagementPolicy::default()); assert_eq!(response.action, ResponseAction::ThankGift); assert!(response.text.contains("Cảm ơn")); assert!(!response.text.to_ascii_lowercase().contains("thưởng tiền")); }
    #[test] fn mode_switches_enable_story_game_music_and_shopping() { let e = event(LiveEventKind::System); let policy = EngagementPolicy::default(); assert_eq!(decide_response(LiveMode::Story, &e, &LiveLedger::default(), &policy).action, ResponseAction::StartStoryBeat); assert_eq!(decide_response(LiveMode::Game, &e, &LiveLedger::default(), &policy).action, ResponseAction::StartGameRound); assert_eq!(decide_response(LiveMode::Music, &e, &LiveLedger::default(), &policy).action, ResponseAction::TransitionMusic); assert_eq!(decide_response(LiveMode::Shopping, &e, &LiveLedger::default(), &policy).action, ResponseAction::ProductMoment); }
    #[test] fn launch_gate_rejects_unavailable_external_publish() { let gate = LiveLaunchGate { capability: ExternalLiveCapability::production_default(), approved: true, stream_destination: Some("rtmps://example.invalid/live".into()), reason: "test".into() }; assert!(gate.validate().is_err()); }
    #[test]
    fn provider_statement_reconciles_recorded_gifts() {
        let mut ledger = LiveLedger::default();
        let e = event(LiveEventKind::Gift);
        ledger.ingest(&e).unwrap();
        let statement = ProviderGiftStatement {
            statement_id: "stmt-1".into(),
            room_id: "room-1".into(),
            gift_count: 1,
            gross_value_minor: 10,
            currency: "USD".into(),
        };
        let reconciliation = reconcile_gifts(&ledger, &statement).unwrap();
        assert!(reconciliation.matched);
        assert_eq!(reconciliation.value_delta_minor, 0);
    }

    #[test] fn invalid_events_fail_closed() { let mut e = event(LiveEventKind::Comment); e.event_id.clear(); assert!(LiveLedger::default().ingest(&e).is_err()); }
}
