# Content Factory

## Pipeline
1. Research Agent identifies audience problems, trends and commercial opportunities.
2. Strategy Agent selects content hypotheses.
3. Script Agent creates structured scripts.
4. Creative Agent produces assets.
5. Media Worker renders/encodes.
6. QA Agent checks policy, claims, quality and required disclosures.
7. Publisher adapter publishes only within approved permissions.
8. Analytics Agent ingests verified metrics.
9. Attribution Agent connects content to revenue.
10. Experiment Agent updates the portfolio.

## Content object
A content item contains:
- creator/channel
- hypothesis
- audience
- format
- script/assets
- product/offer
- disclosure requirements
- expected cost
- success metric
- maximum loss
- publish status
- performance metrics
- attributed revenue
- decision outcome.

## Experiment rule
A content experiment must have a time/cost bound and a decision rule.
Outcomes: SCALE, ITERATE, PAUSE, KILL.

## Quality gates
- factual claims checked
- copyright/licensing checked
- commercial disclosure checked
- platform-specific constraints checked
- unsafe content rejected
- tracking identifiers validated
- final media checksum stored.

## Implemented control-plane foundation

The repository now includes a durable company-content contract and PostgreSQL content_items ledger. Each content item records the hypothesis, audience, format, product/offer references, disclosure requirement, expected cost, maximum loss, success metric/threshold, and a structured creative variant (hook, first frame, pacing, scene count, product placement, CTA, comment trigger, music and visual style).

The Company OS exposes authenticated GET/POST /api/content/items endpoints for creating and reviewing these plans. Validation is deterministic and fail-closed. Content creation does not imply rendering, publishing, platform approval, audience reach, or revenue; those remain separate governed workflows requiring verified evidence.

The next integration layer is to connect content plans to media rendering, publishing intents, verified analytics, attribution, experiment observations and learning entries without allowing unverified metrics to become business outcomes.


## Performance evidence loop

- Content cannot be measured while still in draft/approved/rendered state; a published state transition requires an external evidence reference.
- `POST /api/content/status` records governed lifecycle transitions and persists the evidence reference for published/measured states.
- `POST /api/content/observations` accepts only company-scoped, idempotent observations with source, evidence hash, timestamp, sample count and funnel metrics.
- Observation decisions are deterministic: exceeding the configured loss limit yields KILL; meeting the success threshold yields SCALE; otherwise the content is ITERATE.
- Duplicate observation keys return the previously persisted observation rather than creating a second outcome.
- These endpoints record evidence supplied by a verified analytics/publishing boundary; they do not fabricate platform metrics or prove that a TikTok API integration is active.
