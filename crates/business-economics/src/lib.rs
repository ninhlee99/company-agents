#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BusinessUnitStatus {
    Testing,
    Growing,
    Stable,
    Distress,
    Paused,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BusinessUnitKind {
    Creator,
    AffiliateChannel,
    MarketingService,
    OwnedMedia,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayrollObligation {
    pub employee_id: String,
    pub amount_minor: i128,
    pub due_day: u64,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractObligation {
    pub contract_id: String,
    pub counterparty: String,
    pub amount_minor: i128,
    pub due_day: u64,
    pub cancellable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BusinessUnit {
    pub id: String,
    pub name: String,
    pub kind: BusinessUnitKind,
    pub status: BusinessUnitStatus,
    pub cash_minor: i128,
    pub revenue_minor: i128,
    pub direct_cost_minor: i128,
    pub fixed_cost_minor: i128,
    pub payroll: Vec<PayrollObligation>,
    pub contracts: Vec<ContractObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BusinessDayResult {
    pub day: u64,
    pub opening_cash_minor: i128,
    pub revenue_minor: i128,
    pub direct_cost_minor: i128,
    pub fixed_cost_minor: i128,
    pub payroll_paid_minor: i128,
    pub contracts_paid_minor: i128,
    pub closing_cash_minor: i128,
    pub free_cash_flow_minor: i128,
    pub unpaid_priority_minor: i128,
    pub status: BusinessUnitStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BusinessError {
    Invalid(String),
    Overflow,
}

impl std::fmt::Display for BusinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(value) => write!(f, "invalid business unit: {value}"),
            Self::Overflow => write!(f, "business economics arithmetic overflow"),
        }
    }
}

impl std::error::Error for BusinessError {}

impl BusinessUnit {
    pub fn validate(&self) -> Result<(), BusinessError> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err(BusinessError::Invalid("unit id and name are required".into()));
        }
        if self.cash_minor < 0
            || self.revenue_minor < 0
            || self.direct_cost_minor < 0
            || self.fixed_cost_minor < 0
        {
            return Err(BusinessError::Invalid("unit economics cannot be negative".into()));
        }
        for payroll in &self.payroll {
            if payroll.employee_id.trim().is_empty() || payroll.amount_minor < 0 {
                return Err(BusinessError::Invalid("payroll obligation is invalid".into()));
            }
        }
        for contract in &self.contracts {
            if contract.contract_id.trim().is_empty()
                || contract.counterparty.trim().is_empty()
                || contract.amount_minor < 0
            {
                return Err(BusinessError::Invalid("contract obligation is invalid".into()));
            }
        }
        Ok(())
    }

    pub fn monthly_runway_days(&self) -> u64 {
        let recurring = self
            .fixed_cost_minor
            .saturating_add(self.payroll.iter().map(|item| item.amount_minor).sum());
        if recurring == 0 {
            return u64::MAX;
        }
        self.cash_minor
            .saturating_mul(30)
            .checked_div(recurring)
            .unwrap_or(0)
            .max(0) as u64
    }

    pub fn settle_day(
        &mut self,
        day: u64,
        revenue_minor: i128,
        direct_cost_minor: i128,
    ) -> Result<BusinessDayResult, BusinessError> {
        self.validate()?;
        if revenue_minor < 0 || direct_cost_minor < 0 {
            return Err(BusinessError::Invalid("daily economics cannot be negative".into()));
        }

        let opening = self.cash_minor;
        let mut cash = opening;

        cash = cash
            .checked_add(revenue_minor)
            .ok_or(BusinessError::Overflow)?;
        cash = cash
            .checked_sub(direct_cost_minor)
            .ok_or(BusinessError::Overflow)?;
        cash = cash
            .checked_sub(self.fixed_cost_minor)
            .ok_or(BusinessError::Overflow)?;

        let mut payroll_paid = 0_i128;
        let mut unpaid_priority = 0_i128;
        for obligation in &self.payroll {
            if obligation.due_day > day {
                continue;
            }
            if obligation.amount_minor <= cash {
                cash = cash
                    .checked_sub(obligation.amount_minor)
                    .ok_or(BusinessError::Overflow)?;
                payroll_paid = payroll_paid
                    .checked_add(obligation.amount_minor)
                    .ok_or(BusinessError::Overflow)?;
            } else {
                unpaid_priority = unpaid_priority
                    .checked_add(obligation.amount_minor)
                    .ok_or(BusinessError::Overflow)?;
            }
        }

        let mut contracts_paid = 0_i128;
        for contract in &self.contracts {
            if contract.due_day > day {
                continue;
            }
            if contract.amount_minor <= cash {
                cash = cash
                    .checked_sub(contract.amount_minor)
                    .ok_or(BusinessError::Overflow)?;
                contracts_paid = contracts_paid
                    .checked_add(contract.amount_minor)
                    .ok_or(BusinessError::Overflow)?;
            } else {
                unpaid_priority = unpaid_priority
                    .checked_add(contract.amount_minor)
                    .ok_or(BusinessError::Overflow)?;
            }
        }

        self.cash_minor = cash;
        self.revenue_minor = self
            .revenue_minor
            .checked_add(revenue_minor)
            .ok_or(BusinessError::Overflow)?;
        self.direct_cost_minor = self
            .direct_cost_minor
            .checked_add(direct_cost_minor)
            .ok_or(BusinessError::Overflow)?;

        self.status = if cash <= 0 {
            BusinessUnitStatus::Distress
        } else if self.monthly_runway_days() <= 7 {
            BusinessUnitStatus::Distress
        } else if self.revenue_minor > self.direct_cost_minor.saturating_add(self.fixed_cost_minor) {
            BusinessUnitStatus::Growing
        } else {
            BusinessUnitStatus::Stable
        };

        let total_cost = direct_cost_minor
            .checked_add(self.fixed_cost_minor)
            .and_then(|v| v.checked_add(payroll_paid))
            .and_then(|v| v.checked_add(contracts_paid))
            .ok_or(BusinessError::Overflow)?;

        let fcf = revenue_minor
            .checked_sub(total_cost)
            .ok_or(BusinessError::Overflow)?;

        Ok(BusinessDayResult {
            day,
            opening_cash_minor: opening,
            revenue_minor,
            direct_cost_minor,
            fixed_cost_minor: self.fixed_cost_minor,
            payroll_paid_minor: payroll_paid,
            contracts_paid_minor: contracts_paid,
            closing_cash_minor: cash,
            free_cash_flow_minor: fcf,
            unpaid_priority_minor: unpaid_priority,
            status: self.status,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompanyPortfolio {
    pub units: Vec<BusinessUnit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioDayResult {
    pub day: u64,
    pub revenue_minor: i128,
    pub costs_minor: i128,
    pub cash_minor: i128,
    pub unpaid_priority_minor: i128,
    pub distressed_units: usize,
}

impl CompanyPortfolio {
    pub fn validate(&self) -> Result<(), BusinessError> {
        for unit in &self.units {
            unit.validate()?;
        }
        Ok(())
    }

    pub fn settle_day(
        &mut self,
        day: u64,
        revenues_minor: &[i128],
        direct_costs_minor: &[i128],
    ) -> Result<PortfolioDayResult, BusinessError> {
        if revenues_minor.len() != self.units.len() || direct_costs_minor.len() != self.units.len() {
            return Err(BusinessError::Invalid(
                "portfolio daily economics must match unit count".into(),
            ));
        }
        let mut revenue = 0_i128;
        let mut costs = 0_i128;
        let mut cash = 0_i128;
        let mut unpaid = 0_i128;
        let mut distressed = 0_usize;

        for (index, unit) in self.units.iter_mut().enumerate() {
            let result = unit.settle_day(day, revenues_minor[index], direct_costs_minor[index])?;
            revenue = revenue.checked_add(result.revenue_minor).ok_or(BusinessError::Overflow)?;
            costs = costs
                .checked_add(
                    result.direct_cost_minor
                        .saturating_add(result.fixed_cost_minor)
                        .saturating_add(result.payroll_paid_minor)
                        .saturating_add(result.contracts_paid_minor),
                )
                .ok_or(BusinessError::Overflow)?;
            cash = cash.checked_add(result.closing_cash_minor).ok_or(BusinessError::Overflow)?;
            unpaid = unpaid.checked_add(result.unpaid_priority_minor).ok_or(BusinessError::Overflow)?;
            if result.status == BusinessUnitStatus::Distress {
                distressed += 1;
            }
        }

        Ok(PortfolioDayResult {
            day,
            revenue_minor: revenue,
            costs_minor: costs,
            cash_minor: cash,
            unpaid_priority_minor: unpaid,
            distressed_units: distressed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit() -> BusinessUnit {
        BusinessUnit {
            id: "creator-1".into(),
            name: "Creator A".into(),
            kind: BusinessUnitKind::Creator,
            status: BusinessUnitStatus::Growing,
            cash_minor: 10_000,
            revenue_minor: 0,
            direct_cost_minor: 0,
            fixed_cost_minor: 100,
            payroll: vec![PayrollObligation {
                employee_id: "editor-1".into(),
                amount_minor: 200,
                due_day: 1,
                priority: 100,
            }],
            contracts: vec![],
        }
    }

    #[test]
    fn payroll_is_paid_before_nonpriority_contracts() {
        let mut value = unit();
        value.contracts.push(ContractObligation {
            contract_id: "vendor-1".into(),
            counterparty: "Vendor".into(),
            amount_minor: 9_900,
            due_day: 1,
            cancellable: true,
        });
        let result = value.settle_day(1, 0, 0).unwrap();
        assert_eq!(result.payroll_paid_minor, 200);
        assert_eq!(result.contracts_paid_minor, 0);
        assert!(result.unpaid_priority_minor > 0);
        assert_eq!(value.cash_minor, 9_700);
    }

    #[test]
    fn negative_daily_revenue_is_rejected() {
        let mut value = unit();
        assert!(value.settle_day(1, -1, 0).is_err());
    }

    #[test]
    fn portfolio_requires_aligned_inputs() {
        let mut portfolio = CompanyPortfolio { units: vec![unit()] };
        assert!(portfolio.settle_day(1, &[], &[]).is_err());
    }

    #[test]
    fn zero_recurring_cost_has_infinite_runway() {
        let mut value = unit();
        value.fixed_cost_minor = 0;
        value.payroll.clear();
        assert_eq!(value.monthly_runway_days(), u64::MAX);
    }
}
