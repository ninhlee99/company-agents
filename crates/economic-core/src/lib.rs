#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub amount_minor: i128,
    pub currency: String,
}

impl Money {
    pub fn new(amount_minor: i128, currency: impl Into<String>) -> Result<Self, &'static str> {
        let currency = currency.into();
        if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err("currency must be a 3-letter uppercase code");
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub account_id: String,
    pub debit_minor: i128,
    pub credit_minor: i128,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerTransaction {
    pub id: String,
    pub description: String,
    pub entries: Vec<LedgerEntry>,
}

pub fn validate_balanced_transaction(tx: &LedgerTransaction) -> Result<(), &'static str> {
    if tx.id.trim().is_empty() {
        return Err("transaction id is required");
    }
    if tx.entries.is_empty() {
        return Err("transaction requires entries");
    }
    let currency = tx.entries[0].currency.as_str();
    let mut debit = 0_i128;
    let mut credit = 0_i128;
    for entry in &tx.entries {
        if entry.account_id.trim().is_empty() {
            return Err("ledger account id is required");
        }
        if entry.currency != currency {
            return Err("all ledger entries must use the same currency");
        }
        if entry.debit_minor < 0 || entry.credit_minor < 0 {
            return Err("negative ledger entry");
        }
        if entry.debit_minor > 0 && entry.credit_minor > 0 {
            return Err("entry cannot contain both debit and credit");
        }
        if entry.debit_minor == 0 && entry.credit_minor == 0 {
            return Err("zero-value ledger entry");
        }
        debit = debit
            .checked_add(entry.debit_minor)
            .ok_or("debit overflow")?;
        credit = credit
            .checked_add(entry.credit_minor)
            .ok_or("credit overflow")?;
    }
    if debit != credit {
        return Err("unbalanced ledger transaction");
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanyStatus {
    Active,
    Growth,
    Warning,
    CostControl,
    Distress,
    Emergency,
    Liquidation,
    Bankrupt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyState {
    pub company_id: String,
    pub cash_minor: i128,
    pub revenue_minor: i128,
    pub expenses_minor: i128,
    pub liabilities_minor: i128,
    pub assets_minor: i128,
    pub runway_days: i64,
    pub status: CompanyStatus,
}

impl CompanyState {
    pub fn free_cash_flow(&self) -> i128 {
        self.revenue_minor.saturating_sub(self.expenses_minor)
    }
    pub fn can_spend(&self, budget: &Budget, amount_minor: i128) -> bool {
        amount_minor > 0
            && !matches!(
                self.status,
                CompanyStatus::Liquidation | CompanyStatus::Bankrupt
            )
            && budget.active
            && budget.company_id == self.company_id
            && budget
                .spent_minor
                .checked_add(amount_minor)
                .is_some_and(|next| next <= budget.limit_minor)
            && amount_minor <= self.cash_minor
    }

    pub fn runway_days_from_burn(&self, daily_burn_minor: i128) -> i64 {
        if daily_burn_minor <= 0 {
            return i64::MAX;
        }
        (self.cash_minor / daily_burn_minor).clamp(0, i64::MAX as i128) as i64
    }

    pub fn refresh_status_from_runway(&self, daily_burn_minor: i128) -> CompanyState {
        let runway = self.runway_days_from_burn(daily_burn_minor);
        let status = if self.cash_minor < 0 {
            CompanyStatus::Emergency
        } else if runway <= 0 {
            CompanyStatus::Emergency
        } else if runway <= 7 {
            CompanyStatus::Distress
        } else if runway <= 21 {
            CompanyStatus::CostControl
        } else if runway <= 45 {
            CompanyStatus::Warning
        } else if self.free_cash_flow() > 0 {
            CompanyStatus::Growth
        } else {
            CompanyStatus::Active
        };
        CompanyState {
            runway_days: runway,
            status,
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    pub id: String,
    pub company_id: String,
    pub limit_minor: i128,
    pub spent_minor: i128,
    pub currency: String,
    pub active: bool,
}

pub fn spend_from_budget(
    state: &CompanyState,
    budget: &Budget,
    amount_minor: i128,
) -> Result<(CompanyState, Budget), &'static str> {
    if !state.can_spend(budget, amount_minor) {
        return Err("spend rejected by company or budget policy");
    }
    let next_cash = state
        .cash_minor
        .checked_sub(amount_minor)
        .ok_or("cash arithmetic overflow")?;
    let next_expenses = state
        .expenses_minor
        .checked_add(amount_minor)
        .ok_or("expense arithmetic overflow")?;
    let next_spent = budget
        .spent_minor
        .checked_add(amount_minor)
        .ok_or("budget arithmetic overflow")?;

    let next_state = CompanyState {
        cash_minor: next_cash,
        expenses_minor: next_expenses,
        ..state.clone()
    };
    let next_budget = Budget {
        spent_minor: next_spent,
        ..budget.clone()
    };
    Ok((next_state, next_budget))
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortfolioAction {
    Reinvest,
    Hold,
    Reduce,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioUnitMetrics {
    pub unit_id: String,
    pub cash_minor: i128,
    pub revenue_minor: i128,
    pub expenses_minor: i128,
    pub runway_days: i64,
    pub evidence_confidence_bps: u16,
    pub evidence_count: u32,
    pub allocation_cap_minor: i128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioDecision {
    pub unit_id: String,
    pub action: PortfolioAction,
    pub amount_minor: i128,
    pub reason: String,
}

pub fn decide_portfolio_action(
    company_status: CompanyStatus,
    unit: &PortfolioUnitMetrics,
) -> Result<PortfolioDecision, &'static str> {
    if unit.unit_id.trim().is_empty() {
        return Err("portfolio unit id is required");
    }
    if unit.cash_minor < 0
        || unit.revenue_minor < 0
        || unit.expenses_minor < 0
        || unit.runway_days < 0
        || unit.evidence_confidence_bps > 10_000
        || unit.allocation_cap_minor < 0
    {
        return Err("portfolio metrics are invalid");
    }

    let margin = unit.revenue_minor.saturating_sub(unit.expenses_minor);

    if matches!(
        company_status,
        CompanyStatus::Emergency | CompanyStatus::Liquidation | CompanyStatus::Bankrupt
    ) {
        return Ok(PortfolioDecision {
            unit_id: unit.unit_id.clone(),
            action: if margin < 0 {
                PortfolioAction::Reduce
            } else {
                PortfolioAction::Hold
            },
            amount_minor: 0,
            reason: "company liquidity protection blocks discretionary reinvestment".into(),
        });
    }

    if margin < 0 {
        return Ok(PortfolioDecision {
            unit_id: unit.unit_id.clone(),
            action: if unit.runway_days <= 21 {
                PortfolioAction::Close
            } else {
                PortfolioAction::Reduce
            },
            amount_minor: 0,
            reason: "negative contribution margin requires capital preservation".into(),
        });
    }

    if !matches!(company_status, CompanyStatus::Active | CompanyStatus::Growth)
        || unit.runway_days < 45
        || unit.cash_minor == 0
        || unit.evidence_count < 3
        || unit.evidence_confidence_bps < 8_000
        || unit.allocation_cap_minor == 0
    {
        return Ok(PortfolioDecision {
            unit_id: unit.unit_id.clone(),
            action: PortfolioAction::Hold,
            amount_minor: 0,
            reason: "insufficient verified evidence for reinvestment".into(),
        });
    }

    let cash_cap = unit.cash_minor / 10;
    let margin_cap = margin / 4;
    let amount = cash_cap
        .min(margin_cap)
        .min(unit.allocation_cap_minor);

    if amount <= 0 {
        return Ok(PortfolioDecision {
            unit_id: unit.unit_id.clone(),
            action: PortfolioAction::Hold,
            amount_minor: 0,
            reason: "deterministic allocation caps produce no safe reinvestment amount".into(),
        });
    }

    Ok(PortfolioDecision {
        unit_id: unit.unit_id.clone(),
        action: PortfolioAction::Reinvest,
        amount_minor: amount,
        reason: "positive margin and verified evidence permit bounded reinvestment".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> CompanyState {
        CompanyState {
            company_id: "c1".into(),
            cash_minor: 1_000,
            revenue_minor: 0,
            expenses_minor: 0,
            liabilities_minor: 0,
            assets_minor: 1_000,
            runway_days: 30,
            status: CompanyStatus::Active,
        }
    }
    fn budget() -> Budget {
        Budget {
            id: "b1".into(),
            company_id: "c1".into(),
            limit_minor: 500,
            spent_minor: 0,
            currency: "USD".into(),
            active: true,
        }
    }
    #[test]
    fn portfolio_policy_reinvests_only_with_verified_evidence() {
        let unit = PortfolioUnitMetrics {
            unit_id: "u1".into(),
            cash_minor: 10_000,
            revenue_minor: 5_000,
            expenses_minor: 3_000,
            runway_days: 90,
            evidence_confidence_bps: 9_000,
            evidence_count: 5,
            allocation_cap_minor: 500,
        };
        let decision = decide_portfolio_action(CompanyStatus::Growth, &unit).unwrap();
        assert_eq!(decision.action, PortfolioAction::Reinvest);
        assert_eq!(decision.amount_minor, 500);
    }

    #[test]
    fn portfolio_policy_blocks_reinvestment_in_distress() {
        let unit = PortfolioUnitMetrics {
            unit_id: "u1".into(),
            cash_minor: 10_000,
            revenue_minor: 5_000,
            expenses_minor: 1_000,
            runway_days: 5,
            evidence_confidence_bps: 10_000,
            evidence_count: 20,
            allocation_cap_minor: 5_000,
        };
        let decision = decide_portfolio_action(CompanyStatus::Distress, &unit).unwrap();
        assert_eq!(decision.action, PortfolioAction::Hold);
        assert_eq!(decision.amount_minor, 0);
    }

    #[test]
    fn portfolio_policy_blocks_discretionary_spend_in_emergency() {
        let unit = PortfolioUnitMetrics {
            unit_id: "u1".into(),
            cash_minor: 10_000,
            revenue_minor: 5_000,
            expenses_minor: 1_000,
            runway_days: 5,
            evidence_confidence_bps: 10_000,
            evidence_count: 20,
            allocation_cap_minor: 5_000,
        };
        let decision = decide_portfolio_action(CompanyStatus::Emergency, &unit).unwrap();
        assert_eq!(decision.action, PortfolioAction::Hold);
        assert_eq!(decision.amount_minor, 0);
    }

    #[test]
    fn portfolio_policy_closes_loss_making_unit_when_runway_is_short() {
        let unit = PortfolioUnitMetrics {
            unit_id: "u1".into(),
            cash_minor: 100,
            revenue_minor: 100,
            expenses_minor: 200,
            runway_days: 10,
            evidence_confidence_bps: 9_000,
            evidence_count: 5,
            allocation_cap_minor: 1_000,
        };
        let decision = decide_portfolio_action(CompanyStatus::CostControl, &unit).unwrap();
        assert_eq!(decision.action, PortfolioAction::Close);
        assert_eq!(decision.amount_minor, 0);
    }

    #[test]
    fn portfolio_policy_holds_when_evidence_is_insufficient() {
        let unit = PortfolioUnitMetrics {
            unit_id: "u1".into(),
            cash_minor: 10_000,
            revenue_minor: 5_000,
            expenses_minor: 1_000,
            runway_days: 90,
            evidence_confidence_bps: 7_999,
            evidence_count: 2,
            allocation_cap_minor: 5_000,
        };
        let decision = decide_portfolio_action(CompanyStatus::Growth, &unit).unwrap();
        assert_eq!(decision.action, PortfolioAction::Hold);
        assert_eq!(decision.amount_minor, 0);
    }

    #[test]
    fn portfolio_policy_rejects_invalid_metrics() {
        let unit = PortfolioUnitMetrics {
            unit_id: "".into(),
            cash_minor: -1,
            revenue_minor: 0,
            expenses_minor: 0,
            runway_days: 0,
            evidence_confidence_bps: 10_001,
            evidence_count: 0,
            allocation_cap_minor: 0,
        };
        assert_eq!(
            decide_portfolio_action(CompanyStatus::Active, &unit),
            Err("portfolio unit id is required")
        );
    }

    #[test]
    fn ledger_must_balance() {
        let tx = LedgerTransaction {
            id: "x".into(),
            description: "bad".into(),
            entries: vec![LedgerEntry {
                account_id: "a".into(),
                debit_minor: 10,
                credit_minor: 0,
                currency: "USD".into(),
            }],
        };
        assert_eq!(
            validate_balanced_transaction(&tx),
            Err("unbalanced ledger transaction")
        );
    }
    #[test]
    fn ledger_rejects_mixed_currency() {
        let tx = LedgerTransaction {
            id: "x".into(),
            description: "bad".into(),
            entries: vec![
                LedgerEntry {
                    account_id: "a".into(),
                    debit_minor: 1,
                    credit_minor: 0,
                    currency: "USD".into(),
                },
                LedgerEntry {
                    account_id: "b".into(),
                    debit_minor: 0,
                    credit_minor: 1,
                    currency: "EUR".into(),
                },
            ],
        };
        assert_eq!(
            validate_balanced_transaction(&tx),
            Err("all ledger entries must use the same currency")
        );
    }
    #[test]
    fn budget_and_cash_limits_hold() {
        let s = state();
        let b = budget();
        assert!(s.can_spend(&b, 500));
        assert!(!s.can_spend(&b, 501));
        assert!(!CompanyState {
            cash_minor: 400,
            ..s.clone()
        }
        .can_spend(&b, 401));
    }
    #[test]
    fn bankrupt_and_liquidation_block_spend() {
        let b = budget();
        assert!(!CompanyState {
            status: CompanyStatus::Bankrupt,
            ..state()
        }
        .can_spend(&b, 1));
        assert!(!CompanyState {
            status: CompanyStatus::Liquidation,
            ..state()
        }
        .can_spend(&b, 1));
    }
    #[test]
    fn huge_money_is_exact() {
        let value = 10_i128.pow(30);
        let m = Money::new(value, "USD").unwrap();
        assert_eq!(m.amount_minor, value);
    }
    #[test]
    fn invalid_money_currency_is_rejected() {
        assert!(Money::new(1, "usd").is_err());
    }
    #[test]
    fn spend_updates_state_and_budget() {
        let (next_state, next_budget) = spend_from_budget(&state(), &budget(), 100).unwrap();
        assert_eq!(next_state.cash_minor, 900);
        assert_eq!(next_state.expenses_minor, 100);
        assert_eq!(next_budget.spent_minor, 100);
    }

    #[test]
    fn runway_and_status_refresh_are_deterministic() {
        let mut s = state();
        s.revenue_minor = 0;
        s.expenses_minor = 1_000;
        assert_eq!(s.runway_days_from_burn(100), 10);
        assert_eq!(
            s.refresh_status_from_runway(100).status,
            CompanyStatus::CostControl
        );
    }

    #[test]
    fn zero_burn_has_infinite_runway() {
        assert_eq!(state().runway_days_from_burn(0), i64::MAX);
    }

    #[test]
    fn expense_overflow_is_rejected() {
        let mut s = state();
        s.expenses_minor = i128::MAX;
        assert_eq!(
            spend_from_budget(&s, &budget(), 1),
            Err("expense arithmetic overflow")
        );
    }

    #[test]
    fn free_cash_flow_never_panics_on_extreme_values() {
        let s = CompanyState {
            revenue_minor: i128::MIN,
            expenses_minor: i128::MAX,
            ..state()
        };
        assert_eq!(s.free_cash_flow(), i128::MIN);
    }

    #[test]
    fn spend_overflow_is_rejected() {
        let mut b = budget();
        b.spent_minor = i128::MAX;
        b.limit_minor = i128::MAX;
        assert_eq!(
            spend_from_budget(&state(), &b, 1),
            Err("spend rejected by company or budget policy")
        );
    }
}
