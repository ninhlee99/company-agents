# TikTok LIVE commerce operating design

Veridara's TikTok strategy is explicitly split into two revenue engines:

1. TikTok Shop affiliate commerce — select products using the existing affiliate intelligence/ranking pipeline, publish shoppable content, and reconcile commissions from provider evidence.
2. TikTok LIVE — run an AI-assisted live show with chat, story, games, music and product moments, while recording gift events and LIVE economics when an authorized event source is available.

## Implemented in the LIVE engine

- Deterministic LIVE session/event contracts.
- Idempotent gift/comment/follow/share/like ledger.
- Engagement policy for welcome, comments, gifts, milestones, PK commentary, games, stories, music and shopping moments.
- Explicit external-capability gate: the repository does not pretend that a generic TikTok developer token grants LIVE publishing, gift events or PK control.
- Game/story/music/product overlays can be driven by the same event engine.
- Gift earnings are treated as provider evidence; the engine never invents a payout amount.

## Production boundary

TikTok Shop Creator access in Vietnam currently has eligibility and identity/tax requirements. TikTok's developer platform also requires app configuration/review for API access. The exact LIVE publishing, gift-event and PK capabilities available to an account must be verified in the account/developer configuration before enabling external side effects.

The intended production path is:

authorization/capability -> event source -> live-engine -> AI response/TTS/scene compositor -> approved LIVE destination -> TikTok LIVE -> Shop orders + gifts -> provider evidence -> company ledger

The repository deliberately does not use unofficial scraping or bypass mechanisms for TikTok LIVE chat/gifts.

## LIVE content modes

- Solo: AI host + chat
- CoHost: AI host + co-host
- PK: internal PK commentary/score overlay; actual platform PK controls remain provider-gated
- Game: trivia, guessing, audience voting and timed rounds
- Story: episodic interactive storytelling
- Music: playlist/transition orchestration; only music with the necessary rights may be used
- Shopping: verified product moments and affiliate CTAs

## Gift policy

The host may thank viewers for gifts and acknowledge milestones, but must not claim that a gift guarantees money, prizes, refunds or special financial treatment. Provider settlement is reconciled from TikTok evidence, not inferred from chat events.

## Acceptance checklist

- [ ] TikTok account has LIVE + Shop Creator permissions.
- [ ] Identity/tax verification complete.
- [ ] Developer app approved for the exact capabilities being used.
- [ ] Public HTTPS webhook configured where supported.
- [ ] RTMP/LIVE destination or supported publishing mechanism verified for the account.
- [ ] TTS/voice and media rights configured.
- [ ] Affiliate products selected and disclosed.
- [ ] Gift events observed and reconciled against TikTok's own earnings view.
- [ ] End-to-end LIVE rehearsal passes with failure/reconnect tests.
