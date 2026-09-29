# Agent Protocol

Agents communicate through typed commands and events rather than unstructured chat.

Example proposal:

```json
{
  "type": "CAPITAL_ALLOCATION_PROPOSAL",
  "from": "ceo",
  "payload": {
    "business_unit_id": "creator-042",
    "amount": 5000,
    "thesis": "Scale a format with positive contribution margin",
    "horizon_days": 30,
    "max_loss": 5000
  }
}
```

Governor/CFO may return APPROVE, REJECT, REQUEST_REVISION or ESCALATE.

All decisions are immutable audit records.
