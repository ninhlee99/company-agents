# Chaos & Worst-Case Testing

## Purpose
Test the company as if every dependency, data source and agent can fail at the worst possible time.

## Scenarios
### Finance
- duplicate payment event
- missing payment
- negative/incorrect amount
- currency mismatch
- budget exhausted
- payroll due with low cash
- insolvency
- database transaction rollback.

### Agents
- hallucinated revenue
- malicious tool result
- conflicting agent instructions
- infinite retry tendency
- prompt injection in content/customer data
- corrupted memory
- unavailable model
- model returning invalid JSON.

### Media
- corrupted video
- codec failure
- oversized file
- missing audio
- publication rejection
- rate limit
- duplicate publication
- platform outage.

### Business
- affiliate program terminates
- sponsor cancels
- customer does not pay
- traffic collapses
- creator account is suspended
- campaign becomes unprofitable
- sudden cost spike.

## Expected behavior
Detect -> contain -> preserve evidence -> stop unsafe actions -> recover or escalate -> record postmortem.

## Chaos score
Do not use a single vanity score. Release decisions are based on explicit pass/fail safety gates plus measured recovery time, data integrity and economic loss bounds.
