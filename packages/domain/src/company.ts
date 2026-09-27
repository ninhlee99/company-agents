export type CompanyStatus = "ACTIVE" | "GROWTH" | "WARNING" | "COST_CONTROL" | "DISTRESS" | "EMERGENCY" | "LIQUIDATION" | "BANKRUPT";

export type CompanyState = {
  companyId: string;
  cashMinor: bigint;
  revenueMinor: bigint;
  expensesMinor: bigint;
  liabilitiesMinor: bigint;
  assetsMinor: bigint;
  runwayDays: number;
  status: CompanyStatus;
};

export type Budget = {
  id: string;
  companyId: string;
  limitMinor: bigint;
  spentMinor: bigint;
  currency: string;
  active: boolean;
};

const blockedStatuses = new Set<CompanyStatus>(["LIQUIDATION", "BANKRUPT"]);

export function freeCashFlow(state: CompanyState): bigint {
  return state.revenueMinor - state.expensesMinor;
}

export function updateDistressStatus(state: CompanyState, daysOfRunway: number): CompanyState {
  let status: CompanyStatus = "ACTIVE";
  if (daysOfRunway <= 0 || state.cashMinor < 0n) status = "EMERGENCY";
  else if (daysOfRunway <= 7) status = "DISTRESS";
  else if (daysOfRunway <= 21) status = "COST_CONTROL";
  else if (daysOfRunway <= 45) status = "WARNING";
  else if (freeCashFlow(state) > 0n) status = "GROWTH";
  return { ...state, runwayDays: Math.max(0, Math.floor(daysOfRunway)), status };
}

export function canSpend(state: CompanyState, budget: Budget, amountMinor: bigint): boolean {
  if (amountMinor <= 0n) return false;
  if (blockedStatuses.has(state.status)) return false;
  if (!budget.active || budget.companyId !== state.companyId) return false;
  return budget.spentMinor + amountMinor <= budget.limitMinor;
}

export function spendFromBudget(state: CompanyState, budget: Budget, amountMinor: bigint): { state: CompanyState; budget: Budget } {
  if (!canSpend(state, budget, amountMinor)) throw new Error("Spend rejected by company/budget policy");
  return {
    state: { ...state, cashMinor: state.cashMinor - amountMinor, expensesMinor: state.expensesMinor + amountMinor },
    budget: { ...budget, spentMinor: budget.spentMinor + amountMinor },
  };
}

export function markBankrupt(state: CompanyState): CompanyState {
  if (state.cashMinor > 0n && state.liabilitiesMinor <= state.assetsMinor) {
    throw new Error("Bankruptcy requires insolvency or an explicit liquidation decision");
  }
  return { ...state, status: "BANKRUPT" };
}
