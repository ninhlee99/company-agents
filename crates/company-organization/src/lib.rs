#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmployeeStatus {
    Proposed,
    Active,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Employee {
    pub id: String,
    pub name: String,
    pub role: String,
    pub monthly_cost_minor: i128,
    pub currency: String,
    pub status: EmployeeStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BusinessUnitLifecycle {
    Testing,
    Growing,
    Stable,
    Distress,
    Paused,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BusinessUnit {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub cash_minor: i128,
    pub revenue_minor: i128,
    pub variable_cost_minor: i128,
    pub fixed_cost_minor: i128,
    pub budget_minor: i128,
    pub lifecycle: BusinessUnitLifecycle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayrollObligation {
    pub id: String,
    pub employee_id: String,
    pub period: String,
    pub gross_minor: i128,
    pub currency: String,
    pub due_at: String,
    pub paid_minor: i128,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioMetrics {
    pub revenue_minor: i128,
    pub contribution_margin_minor: i128,
    pub variable_cost_minor: i128,
    pub fixed_cost_minor: i128,
    pub total_cost_minor: i128,
    pub burn_minor: i128,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepartmentType {
    Executive,
    ProductAndInnovation,
    GrowthAndMarketing,
    CreativeAndMedia,
    CommercialAndSales,
    RiskAndCompliance,
    CustomerSuccess,
    PeopleAndCulture,
    TreasuryAndFinance,
    OperationsAndTech,
}

impl DepartmentType {
    pub const ALL: [Self; 10] = [
        Self::Executive,
        Self::ProductAndInnovation,
        Self::GrowthAndMarketing,
        Self::CreativeAndMedia,
        Self::CommercialAndSales,
        Self::RiskAndCompliance,
        Self::CustomerSuccess,
        Self::PeopleAndCulture,
        Self::TreasuryAndFinance,
        Self::OperationsAndTech,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Executive => "EXEC",
            Self::ProductAndInnovation => "PROD",
            Self::GrowthAndMarketing => "GROWTH",
            Self::CreativeAndMedia => "CREATIVE",
            Self::CommercialAndSales => "COMMERCIAL",
            Self::RiskAndCompliance => "RISK",
            Self::CustomerSuccess => "CS",
            Self::PeopleAndCulture => "PEOPLE",
            Self::TreasuryAndFinance => "TREASURY",
            Self::OperationsAndTech => "OPS",
        }
    }

    pub fn default_lead_role(self) -> &'static str {
        match self {
            Self::Executive => "CEO",
            Self::ProductAndInnovation => "CPO",
            Self::GrowthAndMarketing => "Growth Lead",
            Self::CreativeAndMedia => "Creative Director",
            Self::CommercialAndSales => "Head of Sales",
            Self::RiskAndCompliance => "Chief Risk Officer",
            Self::CustomerSuccess => "Head of Customer Success",
            Self::PeopleAndCulture => "People & Culture Lead",
            Self::TreasuryAndFinance => "CFO",
            Self::OperationsAndTech => "COO",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub department_type: DepartmentType,
    pub lead_role: String,
    pub monthly_budget_minor: i128,
    pub currency: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyOrganizationStructure {
    pub company_id: String,
    pub company_name: String,
    pub departments: Vec<Department>,
    pub default_currency: String,
}

impl CompanyOrganizationStructure {
    pub fn standard_autonomous_template(company_id: &str, company_name: &str, currency: &str) -> Self {
        let departments = DepartmentType::ALL
            .iter()
            .map(|&dept_type| Department {
                id: format!("dept-{}", dept_type.code().to_lowercase()),
                name: format!("{:?} Department", dept_type),
                department_type: dept_type,
                lead_role: dept_type.default_lead_role().into(),
                monthly_budget_minor: 0,
                currency: currency.into(),
                active: true,
            })
            .collect();

        Self {
            company_id: company_id.into(),
            company_name: company_name.into(),
            departments,
            default_currency: currency.into(),
        }
    }

    pub fn validate(&self) -> Result<(), OrganizationError> {
        if self.company_id.trim().is_empty() || self.company_name.trim().is_empty() {
            return Err(OrganizationError::InvalidValue(
                "company_id and company_name are required".into(),
            ));
        }
        if self.default_currency.len() != 3 || !self.default_currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(OrganizationError::InvalidValue(
                "default currency must be uppercase 3-letter code".into(),
            ));
        }
        for dept in &self.departments {
            if dept.id.trim().is_empty() || dept.name.trim().is_empty() || dept.lead_role.trim().is_empty() {
                return Err(OrganizationError::InvalidValue(
                    "department id, name, and lead_role are required".into(),
                ));
            }
            if dept.monthly_budget_minor < 0 {
                return Err(OrganizationError::InvalidValue(
                    "department monthly budget cannot be negative".into(),
                ));
            }
            if dept.currency != self.default_currency {
                return Err(OrganizationError::InvalidValue(
                    "department currency must match organization currency".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn department_by_type(&self, dept_type: DepartmentType) -> Option<&Department> {
        self.departments.iter().find(|d| d.department_type == dept_type && d.active)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrganizationError {
    InvalidValue(String),
    Overflow,
}

impl std::fmt::Display for OrganizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidValue(v) => write!(f, "invalid organization value: {v}"),
            Self::Overflow => write!(f, "organization arithmetic overflow"),
        }
    }
}

impl std::error::Error for OrganizationError {}

pub fn contribution_margin(
    revenue_minor: i128,
    variable_cost_minor: i128,
) -> Result<i128, OrganizationError> {
    if revenue_minor < 0 || variable_cost_minor < 0 {
        return Err(OrganizationError::InvalidValue(
            "revenue and variable cost must be non-negative".into(),
        ));
    }
    revenue_minor
        .checked_sub(variable_cost_minor)
        .ok_or(OrganizationError::Overflow)
}

pub fn unit_contribution_margin(unit: &BusinessUnit) -> Result<i128, OrganizationError> {
    let contribution = contribution_margin(unit.revenue_minor, unit.variable_cost_minor)?;
    contribution
        .checked_sub(unit.fixed_cost_minor.max(0))
        .ok_or(OrganizationError::Overflow)
}

pub fn summarize_portfolio(units: &[BusinessUnit]) -> Result<PortfolioMetrics, OrganizationError> {
    let mut revenue = 0_i128;
    let mut contribution = 0_i128;
    let mut variable = 0_i128;
    let mut fixed = 0_i128;
    let portfolio_currency = units.first().map(|unit| unit.currency.clone());

    for unit in units {
        if unit.currency.len() != 3 || !unit.currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(OrganizationError::InvalidValue(
                "business unit currency must be uppercase 3-letter code".into(),
            ));
        }
        if portfolio_currency
            .as_deref()
            .is_some_and(|currency| currency != unit.currency)
        {
            return Err(OrganizationError::InvalidValue(
                "portfolio units must share one currency".into(),
            ));
        }
        if unit.cash_minor < 0
            || unit.revenue_minor < 0
            || unit.variable_cost_minor < 0
            || unit.fixed_cost_minor < 0
            || unit.budget_minor < 0
        {
            return Err(OrganizationError::InvalidValue(
                "business unit economics cannot be negative".into(),
            ));
        }
        revenue = revenue
            .checked_add(unit.revenue_minor)
            .ok_or(OrganizationError::Overflow)?;
        contribution = contribution
            .checked_add(unit_contribution_margin(unit)?)
            .ok_or(OrganizationError::Overflow)?;
        variable = variable
            .checked_add(unit.variable_cost_minor)
            .ok_or(OrganizationError::Overflow)?;
        fixed = fixed
            .checked_add(unit.fixed_cost_minor)
            .ok_or(OrganizationError::Overflow)?;
    }

    Ok(PortfolioMetrics {
        revenue_minor: revenue,
        contribution_margin_minor: contribution,
        variable_cost_minor: variable,
        fixed_cost_minor: fixed,
        total_cost_minor: variable
            .checked_add(fixed)
            .ok_or(OrganizationError::Overflow)?,
        burn_minor: contribution.saturating_neg().max(0),
    })
}

pub fn validate_employee(employee: &Employee) -> Result<(), OrganizationError> {
    if employee.id.trim().is_empty() || employee.name.trim().is_empty() || employee.role.trim().is_empty() {
        return Err(OrganizationError::InvalidValue(
            "employee id, name and role are required".into(),
        ));
    }
    if employee.monthly_cost_minor < 0 {
        return Err(OrganizationError::InvalidValue(
            "monthly employee cost cannot be negative".into(),
        ));
    }
    if employee.currency.len() != 3 || !employee.currency.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(OrganizationError::InvalidValue(
            "employee currency must be uppercase 3-letter code".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(
        id: &str,
        revenue: i128,
        variable: i128,
        fixed: i128,
    ) -> BusinessUnit {
        BusinessUnit {
            id: id.into(),
            name: id.into(),
            currency: "USD".into(),
            cash_minor: 1_000,
            revenue_minor: revenue,
            variable_cost_minor: variable,
            fixed_cost_minor: fixed,
            budget_minor: 500,
            lifecycle: BusinessUnitLifecycle::Growing,
        }
    }

    #[test]
    fn contribution_margin_is_exact() {
        assert_eq!(contribution_margin(1_000, 250).unwrap(), 750);
    }

    #[test]
    fn portfolio_metrics_sum_units() {
        let metrics = summarize_portfolio(&[
            unit("a", 1_000, 100, 50),
            unit("b", 2_000, 500, 100),
        ])
        .unwrap();
        assert_eq!(metrics.revenue_minor, 3_000);
        assert_eq!(metrics.contribution_margin_minor, 2_250);
        assert_eq!(metrics.variable_cost_minor, 600);
        assert_eq!(metrics.fixed_cost_minor, 150);
        assert_eq!(metrics.total_cost_minor, 750);
    }

    #[test]
    fn mixed_currency_portfolio_is_rejected() {
        let mut foreign = unit("foreign", 500, 100, 50);
        foreign.currency = "VND".into();
        assert!(matches!(
            summarize_portfolio(&[unit("usd", 1_000, 100, 50), foreign]),
            Err(OrganizationError::InvalidValue(message)) if message.contains("one currency")
        ));
    }

    #[test]
    fn invalid_employee_is_rejected() {
        let employee = Employee {
            id: "".into(),
            name: "x".into(),
            role: "creator".into(),
            monthly_cost_minor: 100,
            currency: "USD".into(),
            status: EmployeeStatus::Active,
        };
        assert!(validate_employee(&employee).is_err());
    }

    #[test]
    fn negative_economics_are_rejected() {
        assert!(contribution_margin(-1, 0).is_err());
        assert!(contribution_margin(1, -1).is_err());
    }

    #[test]
    fn standard_autonomous_template_is_valid() {
        let org = CompanyOrganizationStructure::standard_autonomous_template(
            "org-1",
            "Autonomous Agents Inc",
            "USD",
        );
        assert_eq!(org.departments.len(), 9);
        assert!(org.validate().is_ok());
        assert!(org.department_by_type(DepartmentType::ProductAndInnovation).is_some());
        assert!(org.department_by_type(DepartmentType::RiskAndCompliance).is_some());
        assert!(org.department_by_type(DepartmentType::CustomerSuccess).is_some());
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmploymentType {
    Official,
    Probation,
    Apprentice,
    PartTime,
    Contractor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepartmentLifecycle {
    Proposed,
    Active,
    Scaling,
    Paused,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepartmentCriticality {
    Core,
    Growth,
    Control,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttendanceStatus {
    Present,
    Remote,
    Late,
    Leave,
    Absent,
    CheckedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Team {
    pub id: String,
    pub department_id: String,
    pub parent_team_id: Option<String>,
    pub name: String,
    pub charter: String,
    pub owner_employee_id: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationEmployeeRecord {
    pub employee_id: String,
    pub company_id: String,
    pub department_id: String,
    pub team_id: Option<String>,
    pub manager_id: Option<String>,
    pub title: String,
    pub employment_type: EmploymentType,
    pub employment_level: String,
    pub joined_at_epoch: i64,
    pub status: EmployeeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobPositionRecord {
    pub id: String,
    pub company_id: String,
    pub department_id: String,
    pub team_id: Option<String>,
    pub code: String,
    pub title: String,
    pub level: String,
    pub employment_types: Vec<EmploymentType>,
    pub responsibilities: Vec<String>,
    pub monthly_cost_min_minor: i128,
    pub monthly_cost_max_minor: i128,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttendancePolicyRecord {
    pub id: String,
    pub company_id: String,
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub shift_start: String,
    pub shift_end: String,
    pub grace_minutes: i32,
    pub work_days: Vec<u8>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmploymentLifecycleEventType {
    Hired,
    ProbationStarted,
    ProbationPassed,
    ApprenticeshipStarted,
    Appointed,
    Promoted,
    Transferred,
    ManagerChanged,
    Suspended,
    Reinstated,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmploymentLifecycleEventRecord {
    pub id: String,
    pub company_id: String,
    pub employee_id: String,
    pub event_type: EmploymentLifecycleEventType,
    pub effective_at_epoch: i64,
    pub position_id: Option<String>,
    pub department_id: Option<String>,
    pub team_id: Option<String>,
    pub manager_id: Option<String>,
    pub notes: Option<String>,
    pub approval_reference: Option<String>,
    pub actor_id: String,
    pub created_at_epoch: i64,
}

pub struct OrganizationEmployeeView {
    pub employee_id: String,
    pub company_id: String,
    pub name: String,
    pub title: String,
    pub department_id: Option<String>,
    pub team_id: Option<String>,
    pub manager_id: Option<String>,
    pub position_id: Option<String>,
    pub employment_type: EmploymentType,
    pub employment_level: String,
    pub joined_at_epoch: Option<i64>,
    pub status: EmployeeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationEmployeeUpsert {
    pub employee: OrganizationEmployeeRecord,
    pub position_id: Option<String>,
    pub name: String,
    pub monthly_cost_minor: i128,
    pub currency: String,
}

pub struct DepartmentRecord {
    pub id: String,
    pub company_id: String,
    pub parent_department_id: Option<String>,
    pub code: String,
    pub name: String,
    pub charter: String,
    pub responsibilities: Vec<String>,
    pub kpis: Vec<String>,
    pub owner_employee_id: Option<String>,
    pub monthly_budget_minor: i128,
    pub currency: String,
    pub lifecycle: DepartmentLifecycle,
    pub criticality: DepartmentCriticality,
    pub formation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamRecord {
    pub id: String,
    pub company_id: String,
    pub department_id: String,
    pub parent_team_id: Option<String>,
    pub name: String,
    pub charter: String,
    pub owner_employee_id: Option<String>,
    pub active: bool,
}

pub struct AttendanceRecord {
    pub id: String,
    pub company_id: String,
    pub employee_id: String,
    pub work_date: String,
    pub status: AttendanceStatus,
    pub shift_start: String,
    pub shift_end: String,
    pub check_in_at_epoch: Option<i64>,
    pub check_out_at_epoch: Option<i64>,
    pub source: String,
    pub exception_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepartmentNeedSignal {
    pub department_type: DepartmentType,
    pub requested_code: Option<String>,
    pub requested_name: Option<String>,
    pub required_capabilities: Vec<String>,
    pub capacity_gap_pct: u8,
    pub sustained_cycles: u16,
    pub monthly_budget_ceiling_minor: i128,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DepartmentFormationDecision {
    NoChange,
    FormDepartment,
    ScaleExisting,
}

pub fn evaluate_department_formation(
    departments: &[Department],
    signal: &DepartmentNeedSignal,
) -> Result<DepartmentFormationDecision, OrganizationError> {
    if signal.capacity_gap_pct > 100 {
        return Err(OrganizationError::InvalidValue(
            "capacity gap percentage must be <= 100".into(),
        ));
    }
    if signal.sustained_cycles == 0 || signal.required_capabilities.is_empty() {
        return Ok(DepartmentFormationDecision::NoChange);
    }
    if signal.monthly_budget_ceiling_minor < 0 {
        return Err(OrganizationError::InvalidValue(
            "department budget ceiling cannot be negative".into(),
        ));
    }

    let active = departments.iter().any(|department| department.active);

    if signal.capacity_gap_pct < 25 || signal.sustained_cycles < 3 {
        return Ok(DepartmentFormationDecision::NoChange);
    }

    if active {
        Ok(DepartmentFormationDecision::ScaleExisting)
    } else {
        Ok(DepartmentFormationDecision::FormDepartment)
    }
}

pub fn validate_reporting_tree(
    employees: &[OrganizationEmployeeRecord],
) -> Result<(), OrganizationError> {
    use std::collections::{HashMap, HashSet};

    let company_id = employees.first().map(|employee| employee.company_id.as_str());
    let mut ids = HashSet::new();
    let mut managers = HashMap::new();
    for employee in employees {
        if employee.employee_id.trim().is_empty()
            || employee.company_id.trim().is_empty()
            || employee.department_id.trim().is_empty()
            || employee.title.trim().is_empty()
            || employee.employment_level.trim().is_empty()
        {
            return Err(OrganizationError::InvalidValue(
                "employee organization record is incomplete".into(),
            ));
        }
        if company_id != Some(employee.company_id.as_str()) {
            return Err(OrganizationError::InvalidValue(
                "reporting tree cannot mix employees from different companies".into(),
            ));
        }
        if !ids.insert(employee.employee_id.clone()) {
            return Err(OrganizationError::InvalidValue(
                "duplicate employee id in reporting tree".into(),
            ));
        }
        if employee.manager_id.as_deref() == Some(employee.employee_id.as_str()) {
            return Err(OrganizationError::InvalidValue(
                "employee cannot manage itself".into(),
            ));
        }
        managers.insert(employee.employee_id.clone(), employee.manager_id.clone());
    }

    for employee_id in managers.keys() {
        let mut cursor = Some(employee_id.clone());
        let mut seen = HashSet::new();
        while let Some(current) = cursor {
            if !seen.insert(current.clone()) {
                return Err(OrganizationError::InvalidValue(
                    "reporting tree contains a management cycle".into(),
                ));
            }
            cursor = managers.get(&current).cloned().flatten();
            if let Some(manager) = &cursor {
                if !managers.contains_key(manager) {
                    return Err(OrganizationError::InvalidValue(
                        "manager must belong to the same organization tree".into(),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod enterprise_organization_tests {
    use super::*;

    #[test]
    fn department_formation_requires_sustained_capacity_gap() {
        let signal = DepartmentNeedSignal {
            department_type: DepartmentType::CustomerSuccess,
            requested_code: Some("REVOPS".into()),
            requested_name: Some("Revenue Operations".into()),
            required_capabilities: vec!["customer success".into()],
            capacity_gap_pct: 35,
            sustained_cycles: 3,
            monthly_budget_ceiling_minor: 1_000_000,
        };
        assert_eq!(
            evaluate_department_formation(&[], &signal).unwrap(),
            DepartmentFormationDecision::FormDepartment
        );
    }

    #[test]
    fn custom_department_code_can_form_when_standard_department_exists() {
        let signal = DepartmentNeedSignal {
            department_type: DepartmentType::OperationsAndTech,
            requested_code: Some("REVOPS".into()),
            requested_name: Some("Revenue Operations".into()),
            required_capabilities: vec!["revenue operations".into()],
            capacity_gap_pct: 45,
            sustained_cycles: 4,
            monthly_budget_ceiling_minor: 2_000_000,
        };
        let operations = Department {
            id: "dept-ops".into(),
            name: "Operations & Technology".into(),
            department_type: DepartmentType::OperationsAndTech,
            lead_role: "COO".into(),
            monthly_budget_minor: 500_000,
            currency: "USD".into(),
            active: true,
        };
        assert_eq!(
            evaluate_department_formation(&[operations], &signal).unwrap(),
            DepartmentFormationDecision::FormDepartment
        );
    }

    #[test]
    fn existing_department_scales_instead_of_duplicate_forming() {
        let department = Department {
            id: "dept-cs".into(),
            name: "Customer Success".into(),
            department_type: DepartmentType::CustomerSuccess,
            lead_role: "Head of Customer Success".into(),
            monthly_budget_minor: 500_000,
            currency: "USD".into(),
            active: true,
        };
        let signal = DepartmentNeedSignal {
            department_type: DepartmentType::CustomerSuccess,
            requested_code: Some("REVOPS".into()),
            requested_name: Some("Revenue Operations".into()),
            required_capabilities: vec!["customer success".into()],
            capacity_gap_pct: 40,
            sustained_cycles: 4,
            monthly_budget_ceiling_minor: 900_000,
        };
        assert_eq!(
            evaluate_department_formation(&[department], &signal).unwrap(),
            DepartmentFormationDecision::ScaleExisting
        );
    }

    #[test]
    fn reporting_tree_rejects_cycles_and_unknown_managers() {
        let mut people = vec![
            OrganizationEmployeeRecord {
                employee_id: "a".into(),
                company_id: "c".into(),
                department_id: "d".into(),
                team_id: None,
                manager_id: Some("b".into()),
                title: "Lead".into(),
                employment_type: EmploymentType::Official,
                employment_level: "L5".into(),
                joined_at_epoch: 1,
                status: EmployeeStatus::Active,
            },
            OrganizationEmployeeRecord {
                employee_id: "b".into(),
                company_id: "c".into(),
                department_id: "d".into(),
                team_id: None,
                manager_id: Some("a".into()),
                title: "Lead".into(),
                employment_type: EmploymentType::Official,
                employment_level: "L5".into(),
                joined_at_epoch: 1,
                status: EmployeeStatus::Active,
            },
        ];
        assert!(validate_reporting_tree(&people).is_err());

        people[1].manager_id = Some("missing".into());
        assert!(validate_reporting_tree(&people).is_err());
    }

    #[test]
    fn reporting_tree_accepts_root_and_direct_report() {
        let people = vec![
            OrganizationEmployeeRecord {
                employee_id: "ceo".into(),
                company_id: "c".into(),
                department_id: "exec".into(),
                team_id: None,
                manager_id: None,
                title: "CEO".into(),
                employment_type: EmploymentType::Official,
                employment_level: "L9".into(),
                joined_at_epoch: 1,
                status: EmployeeStatus::Active,
            },
            OrganizationEmployeeRecord {
                employee_id: "hr".into(),
                company_id: "c".into(),
                department_id: "people".into(),
                team_id: None,
                manager_id: Some("ceo".into()),
                title: "People Director".into(),
                employment_type: EmploymentType::Official,
                employment_level: "L7".into(),
                joined_at_epoch: 2,
                status: EmployeeStatus::Active,
            },
        ];
        assert!(validate_reporting_tree(&people).is_ok());
    }
}
