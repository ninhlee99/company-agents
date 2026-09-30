# Data Model

Core aggregates:
Company, BusinessUnit, Agent, Employee, Customer, Product, Creator, ContentAsset, Experiment, Contract, Budget, Approval, Decision, Task, Event.

Financial:
LedgerAccount, LedgerTransaction, LedgerEntry, Invoice, Payment, Expense, Payroll, Asset, Liability.

Agent:
AgentDefinition, AgentRun, ToolCall, Memory, Skill, Policy, Permission, Evaluation.

Invariant examples:
- ledger entries balance to zero per transaction
- posted transactions are immutable
- cash equals the ledger-derived cash balance
- agent permissions cannot be self-modified
- approved spend cannot exceed available budget
- bankruptcy prevents discretionary execution
