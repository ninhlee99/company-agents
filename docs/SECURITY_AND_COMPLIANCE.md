# Security and Compliance

## Security model
Agents are untrusted decision-makers, not administrators.

Controls:
- least privilege
- short-lived credentials
- secrets outside prompts
- network allowlists
- typed tools
- schema validation
- audit logging
- approval policies
- idempotency keys
- rate limits
- sandboxed media processing.

## Content and commercial compliance
The company must follow applicable platform rules and laws for advertising, affiliate disclosure, intellectual property, privacy and consumer protection.

## Human approval
Initially require human approval for:
- contracts
- material spending
- payroll changes
- hiring/firing
- account ownership changes
- high-risk publishing
- legal commitments
- irreversible external actions.

## Data
Collect only data needed for the business. Encrypt secrets and sensitive records. Define retention and deletion policies. Do not put credentials in agent memory.
