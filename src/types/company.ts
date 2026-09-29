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
