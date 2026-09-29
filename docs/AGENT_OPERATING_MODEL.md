# Agent Operating Model

## Agent contract
Every agent has:
- mission
- inputs
- outputs
- tools
- permissions
- budget
- rate limits
- memory policy
- escalation policy
- KPIs
- failure handling
- test suite.

## Command model
Observe -> reason -> produce Proposal -> Governor validates -> deterministic executor performs -> event emitted.

## Permission levels
READ, PROPOSE, EXECUTE_LIMITED, EXECUTE_MATERIAL, ADMIN.
No agent receives ADMIN for constitution, permissions, accounting source-of-truth or security policy.

## Agent lifecycle
SCHEDULED -> READY -> RUNNING -> WAITING -> SUCCEEDED / FAILED / ESCALATED.

## Cost controls
- daily LLM budget
- per-task token budget
- maximum tool calls
- concurrency limit
- timeout
- retry budget
- context window limit.

## Memory
Raw event -> structured fact -> decision record -> reusable lesson.
Sensitive data is minimized and access-controlled.

## Failure behavior
Transient failure: retry with bounded backoff.
Validation failure: reject and request revision.
Repeated failure: quarantine capability and escalate.
External outage: pause dependent workflows.
