# Security and Trust

## Principles
- least privilege
- explicit tool scopes
- secret isolation
- immutable audit trail
- idempotent financial commands
- human approval for high-impact actions
- provider adapters with revocable credentials

## Never
- put secrets in prompts
- allow agents to alter permissions
- let an LLM directly edit balances
- allow arbitrary shell/network access in production
- treat model output as verified financial fact

## Production controls
Use isolated service accounts, secret manager, encrypted storage, structured logs, rate limits, spend limits and emergency shutdown.
