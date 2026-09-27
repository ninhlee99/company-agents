export type CompanyStatus = "ACTIVE" | "GROWTH" | "WARNING" | "COST_CONTROL" | "DISTRESS" | "EMERGENCY" | "LIQUIDATION" | "BANKRUPT";
export type CompanyState={companyId:string;cashMinor:bigint;revenueMinor:bigint;expensesMinor:bigint;liabilitiesMinor:bigint;assetsMinor:bigint;runwayDays:number;status:CompanyStatus};
export type Budget={id:string;companyId:string;limitMinor:bigint;spentMinor:bigint;currency:string;active:boolean};
export const freeCashFlow=(s:CompanyState)=>s.revenueMinor-s.expensesMinor;
export function updateDistressStatus(s:CompanyState,days:number):CompanyState{let status:CompanyStatus="ACTIVE";if(days<=0||s.cashMinor<0n)status="EMERGENCY";else if(days<=7)status="DISTRESS";else if(days<=21)status="COST_CONTROL";else if(days<=45)status="WARNING";else if(freeCashFlow(s)>0n)status="GROWTH";return{...s,runwayDays:Math.max(0,Math.floor(days)),status};}
export function canSpend(s:CompanyState,b:Budget,a:bigint){return a>0n&&s.status!=="LIQUIDATION"&&s.status!=="BANKRUPT"&&b.active&&b.companyId===s.companyId&&b.spentMinor+a<=b.limitMinor&&a<=s.cashMinor;}
export function spendFromBudget(s:CompanyState,b:Budget,a:bigint){if(!canSpend(s,b,a))throw new Error("Spend rejected by company/budget policy");return{state:{...s,cashMinor:s.cashMinor-a,expensesMinor:s.expensesMinor+a},budget:{...b,spentMinor:b.spentMinor+a}};}
