# Production Readiness

## Definition of done
A vertical is production-ready only when:
- accounting is deterministic
- all material actions are authorized
- adapters are idempotent
- secrets are isolated
- audit trail is complete
- backups and restore are tested
- monitoring and alerts exist
- agent evaluations pass
- chaos tests pass
- human escalation paths exist
- legal/platform requirements are reviewed.

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
- agent authorization denials
- incident count.

## Rollout
1. Simulator only.
2. Shadow mode.
3. Human-approved real actions.
4. Limited autonomous actions with strict budgets.
5. Gradual increase based on evidence.

Never jump directly from prototype to unrestricted real-money autonomy.
