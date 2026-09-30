# Production Readiness

This is the release gate for Veridara AI's transition from an AI-native operating platform into a production operating company.

## Definition of done

A vertical is production-ready only when:
- accounting is deterministic and reconciled
- all material actions are authorized
- adapters are idempotent and retry-safe
- secrets are isolated
- audit evidence is complete
- backups and restore are tested
- monitoring and alerts exist
- agent evaluations pass regression thresholds
- chaos/provider-failure tests pass
- human escalation and rollback paths exist
- legal/platform requirements are reviewed
- real revenue and cash outcomes are evidenced rather than simulated

## Evidence gates

### Engineering
- workspace tests, formatting and Clippy pass
- migration and secret-hygiene checks pass
- frontend typecheck/build passes where applicable
- recovery drills pass

### External side effects
- approval evidence is durable
- provider credentials are environment-managed
- provider timeouts/retries are bounded
- idempotency keys prevent duplicate effects
- provider evidence, not HTTP success alone, determines terminal state

### Economic reality
- revenue is tied to a real customer/order/conversion
- delivery evidence exists
- receivables and payments reconcile
- contribution margin and cash runway are observable
- simulated activity is never counted as verified revenue

### Autonomy expansion
- benchmark matrix and regression thresholds exist
- model routing is validated in shadow mode
- portfolio allocation has hard capital ceilings
- incident escalation and rollback are tested
- reliability and business-impact history justify any higher limit

## Observability

Track:
- queue latency
- task success/failure
- model latency/cost
- tool errors
- publication success
- content processing time
- revenue ingestion delay
- cash/runway
- authorization denials
- incident count
- provider reconciliation lag
- verified revenue and contribution margin

## Rollout

1. Simulator only.
2. Shadow mode.
3. Human-approved real actions.
4. Limited autonomous actions with strict budgets.
5. Gradual increase based on evidence.

Never jump directly from prototype to unrestricted real-money autonomy.

## Explicit non-claims

The repository must not claim guaranteed or enormous profits, fully autonomous hiring/payroll, unrestricted banking/payment access, or complete external-company operation until those outcomes are demonstrated by real deployment evidence.

The release gate is evidence-based: code existing is necessary, but not sufficient.
