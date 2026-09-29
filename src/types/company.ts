export type CompanyStatus = 
  | 'Active' 
  | 'Growth' 
  | 'Warning' 
  | 'CostControl' 
  | 'Distress' 
  | 'Emergency' 
  | 'Liquidation' 
  | 'Bankrupt';

export type AgentRole = 
  | 'Governor'
  | 'CEO'
  | 'CFO'
  | 'COO'
  | 'Growth'
  | 'Content'
  | 'Recruiter'
  | 'Analyst'
  | 'Experiment'
  | string;

export type ActionKind = 
  | 'AllocateExperimentBudget'
  | 'ReduceBudget'
  | 'RebalanceOperations'
  | 'ResearchOpportunity'
  | 'CreateExperiment'
  | 'ProposeHire'
  | 'ProduceReport'
  | 'EscalateIncident'
  | 'PublishContent'
  | 'LaunchCampaign'
  | 'ExecutePipeline';

export type RiskTier = 'Low' | 'Medium' | 'High' | 'Critical';
export type GovernorDecision = 'Approve' | 'Reject' | 'RequestRevision' | 'EscalateToHuman';

export interface Proposal {
  id: string;
  agent: AgentRole;
  objective: string;
  action: ActionKind;
  cost_minor: number;
  expected_revenue_minor: number;
  risk: RiskTier;
  confidence_bps: number;
  evidence: string[];
  rationale: string;
  reversible: boolean;
  timestamp: string;
}

export interface GovernedProposal {
  proposal: Proposal;
  decision: GovernorDecision;
  reason: string;
  evaluatedAt: string;
  executed: boolean;
}

export interface LedgerEntry {
  id: string;
  timestamp: string;
  description: string;
  debitAccount: string;
  creditAccount: string;
  amount_minor: number;
  cycle: number;
  proposalId?: string;
}

export interface ExecutionReceipt {
  id: string;
  proposalId: string;
  agent: AgentRole;
  action: ActionKind;
  status: 'Completed' | 'Failed' | 'PendingHumanReview';
  outcome: string;
  cost_minor: number;
  timestamp: string;
}

export interface CompanySnapshot {
  status: CompanyStatus;
  cash_minor: number;
  revenue_minor: number;
  expenses_minor: number;
  budget_remaining_minor: number;
  experiment_budget_minor: number;
  runway_days: number;
  backlog: number;
  capacity: number;
  conversion_bps: number;
  audience_growth_bps: number;
  content_revenue_minor: number;
  content_cost_minor: number;
  hiring_need: number;
  cycle_count: number;
  currency: string;
}

export interface CustomAgent {
  id: string;
  name: string;
  role: string;
  department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech';
  description: string;
  salary_minor: number;
  tasksCompleted: number;
  status: 'Active' | 'Paused';
  hiredAtCycle: number;
  skillLevel?: number; // e.g. 1, 2, 3
  taskMultiplier?: number; // e.g. 1.0, 1.25, 1.5
  trainedSkills?: string[];
  trainingCount?: number;
}

export interface SkillTrainingCourse {
  id: string;
  name: string;
  department: 'Leadership' | 'Growth' | 'Ops' | 'All';
  description: string;
  cost_minor: number;
  multiplierBoost: number; // e.g. 0.25 = +25%
  tasksBonus: number; // e.g. +10 tasks
  badge: string;
  levelRequired: number;
}

export interface WorkloadHeatmapCell {
  cycle: number;
  agentId: string;
  agentName: string;
  agentRole: string;
  department: 'Leadership' | 'Growth' | 'Ops';
  workloadPct: number; // 0 - 100%
  tasksProcessed: number;
  isBottleneck: boolean;
  bottleneckType?: 'Queue Overflow' | 'Token Exhaustion' | 'Manual Escalation' | 'Compliance Lock';
  notes?: string;
}

export interface CycleTrendPoint {
  cycle: string;
  cycleNum: number;
  cash: number;
  revenue: number;
  expenses: number;
  netCashFlow: number;
}

export interface DepartmentBudgetPoint {
  cycle: string;
  cycleNum: number;
  leadership: number;
  growth: number;
  ops: number;
  techAndMedia: number;
  total: number;
}

export interface AgentMessage {
  id: string;
  type: 'Chat' | 'Memo';
  fromAgent: string;
  fromRole: string;
  toAgent: string;
  toRole: string;
  subject?: string;
  content: string;
  actionItem?: string;
  tag: string;
  cycle: number;
  timestamp: string;
  priority?: 'High' | 'Normal' | 'Urgent';
}

export interface AgentTaskItem {
  id: string;
  cycle: number;
  title: string;
  action: string;
  outcome: string;
  cost_minor: number;
  status: 'Completed' | 'Pending' | 'Failed';
  timestamp: string;
}

export interface AutoAuditReport {
  id: string;
  cycleMilestone: number;
  timestamp: string;
  plannedRevenueMinor: number;
  actualRevenueMinor: number;
  varianceRevenueMinor: number;
  variancePercent: number;
  plannedExpensesMinor: number;
  actualExpensesMinor: number;
  cashReserveMinor: number;
  verdict: 'ExceededTarget' | 'OnTrack' | 'UnderTarget';
  summary: string;
  governorNote: string;
}

export interface MediaContentGenerated {
  title: string;
  hook: string;
  scriptOutline: {
    timestamp: string;
    visual: string;
    audio: string;
  }[];
  affiliateOffer: string;
  projectedEpc: string;
  callToAction: string;
  governorComplianceCheck: string;
}

export interface DebateTurn {
  agent: string;
  stance: string;
  argument: string;
  proposedAction?: string;
  finalRuling?: string;
  policyJustification?: string;
}

export interface SystemAlert {
  id: string;
  level: 'Critical' | 'Warning' | 'Info' | 'Resolved';
  type: 'Low Runway' | 'Budget Exhaustion' | 'High Expense Spike' | 'Constitutional Override' | 'Backlog Surge';
  title: string;
  description: string;
  discoveredBy: string; // 'Governor AI'
  cycle: number;
  timestamp: string;
  resolved: boolean;
  mitigationAction?: string;
}

export interface CompanyKPIs {
  revenueVelocity: {
    value: string;
    changeRate: number; // e.g. +14.2%
    periodLabel: string;
    trend: 'up' | 'down' | 'stable';
  };
  burnRateEfficiency: {
    value: string;
    ratio: number; // e.g. 0.65
    statusText: string;
    trend: 'up' | 'down' | 'stable';
  };
  roiPerCycle: {
    value: string;
    percentage: number; // e.g. 24.6%
    statusText: string;
    trend: 'up' | 'down' | 'stable';
  };
}
