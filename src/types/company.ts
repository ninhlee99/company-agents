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

export interface CandidateProfile {
  id: string;
  name: string;
  avatar: string;
  role: string;
  department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech';
  level: 'Senior' | 'Lead' | 'Principal Expert' | 'Director';
  yearsExperience: number;
  expectedSalaryMinor: number;
  skills: { name: string; score: number }[]; // score 1-100
  bio: string;
  portfolio: { title: string; metric: string; description: string }[];
  culturalFitScore: number; // 0-100
  recommendedBy: string; // 'Recruiter AI'
  status: 'Available' | 'Interviewing' | 'Hired' | 'Rejected';
  interviewNotes?: string;
  roiProjectionBps: number;
}

export interface OfficeActivityEvent {
  id: string;
  timestamp: string;
  agentId: string;
  agentName: string;
  agentRole: string;
  department: string;
  actionType: 'CreateContent' | 'ComposeMusic' | 'RenderVideo' | 'TakePhotos' | 'AuditLedger' | 'RecruitTalent' | 'ExecutePayout' | 'OptimizeTraffic';
  title: string;
  detail: string;
  impactMinor?: number; // revenue or cost delta
  badgeColor?: string;
}

export interface CopywritingSpec {
  headline: string;
  hook3s: string;
  retentionFormula: string;
  bodyPainPoints: string[];
  ctaText: string;
  targetAudience: string;
  complianceChecked: boolean;
}

export interface AudioTrackSpec {
  title: string;
  genre: 'Lo-Fi Chill' | 'Upbeat Commercial Pop' | 'Trap Tech Energy' | 'Ambient Focus' | 'Phonk Viral';
  bpm: number;
  mood: string;
  voiceoverTone: 'Confident & Crisp' | 'Friendly & Enthusiastic' | 'Deep & Authoritative';
  voiceSpeed: string;
  loudnessLufs: number; // e.g. -14 LUFS
}

export interface VisualShotSpec {
  shotIndex: number;
  framing: 'Macro Detail Close-Up' | '45-Degree Desk Top-Down' | 'POV Handheld Showcase' | 'Side Split Comparison';
  lighting: 'Studio Softbox Glow' | 'Cyberpunk Neon Accent' | 'Warm Natural Daylight';
  imagePrompt: string;
  durationSec: number;
  textOverlay: string;
}

export interface FullCreativeProduction {
  id: string;
  campaignTitle: string;
  niche: string;
  projectedRevenueMinor: number;
  costMinor: number;
  copywriting: CopywritingSpec;
  audioTrack: AudioTrackSpec;
  visualShots: VisualShotSpec[];
  renderSettings: {
    resolution: string;
    fps: number;
    codec: string;
    aspectRatio: string;
  };
  governorApproved: boolean;
  publishedChannels: string[];
  attributionEpc: string;
}

export interface CompanyPnL {
  totalRevenueMinor: number;
  grossMarginPercent: number;
  operatingExpensesMinor: number;
  netIncomeMinor: number;
  monthlyRunRateMinor: number;
  dividendsDeclaredMinor: number;
  retainedEarningsMinor: number;
}

export interface AutonomousSettings {
  isAutoPilotActive: boolean;
  intervalSeconds: number; // e.g. 5s, 10s, 30s
  autoHireWhenBacklogHigh: boolean;
  autoReinvestProfitPct: number; // e.g. 25%
  maxSpendPerAutoCycleMinor: number; // e.g. $500
  lastTickTimestamp?: string;
}

export interface ClientContractDeliverables {
  summary: string;
  scriptContent?: string;
  audioVoiceover?: string;
  visualPrompts?: string[];
  videoSpecs?: {
    resolution: string;
    fps: string;
    duration: string;
    aspectRatio: string;
  };
  researchInsights?: string[];
  deliveredAt: string;
  qualityScore?: number;
  downloadUrl?: string;
}

export interface ClientContract {
  id: string;
  contractNumber: string;
  clientName: string;
  clientEmail?: string;
  title: string;
  category: 'VideoMarketing' | 'Copywriting' | 'MediaDesign' | 'MarketIntelligence' | 'FullCampaign';
  requirements: string;
  budgetMinor: number; // in cents (e.g. 25000 = $250.00)
  createdAt: string;
  deadline: string;
  status: 'Received' | 'Scoping' | 'InProduction' | 'QualityReview' | 'Delivered' | 'Completed';
  currentStage: string;
  progressPercent: number;
  assignedAgents: {
    role: string;
    name: string;
    step: string;
    completed: boolean;
  }[];
  deliverables?: ClientContractDeliverables;
  invoice: {
    amountMinor: number;
    paidStatus: 'Paid' | 'Pending';
    paidAt?: string;
    transactionId?: string;
  };
  rating?: number;
  clientFeedback?: string;
}

export interface LiveStreamDonation {
  id: string;
  donor: string;
  avatar: string;
  amountMinor: number;
  giftName: string;
  giftIcon: string;
  message: string;
  timestamp: string;
}

export interface LiveStreamComment {
  id: string;
  userName: string;
  avatar: string;
  message: string;
  timestamp: string;
  aiHostReply?: string;
  isDonation?: boolean;
  donationAmountMinor?: number;
  giftIcon?: string;
}

export interface AIComputerUseState {
  isActive: boolean;
  gameTitle: string;
  apm: number; // Actions Per Minute
  reactionSpeedMs: number;
  currentKeyAction: string;
  visionFps: number;
  aiPlayerRank: string;
  gameplayLog: string;
}

export interface LivestreamSession {
  id: string;
  channelId: string;
  channelName: string;
  platform: 'TikTok Live' | 'YouTube Live' | 'Twitch' | 'Facebook Gaming';
  streamType: 'Gaming & Reaction' | 'Storytelling & Mystery' | 'Healing & Q&A' | 'Lofi Chill & Minigames';
  talentMode: 'ChitChat' | 'SingingCover' | 'AutonomousGaming';
  hostAgentName: string;
  hostAgentAvatar: string;
  digitalHumanModel: string; // Unreal Engine 5 Metahuman / Neural Gaussian Avatar
  personaStyle: string;
  virtualSet: string;
  aiDecisionRationale: string;
  title: string;
  currentGameOrTopic: string;
  streamStatus: 'Live' | 'Paused' | 'Ended';
  viewersCount: number;
  peakViewers: number;
  donationReceivedMinor: number;
  liveDurationSec: number;
  aiComputerUse: AIComputerUseState;
  currentSongPlaying?: {
    title: string;
    artist: string;
    vocalPitchQuality: string;
  };
  comments: LiveStreamComment[];
  recentDonations: LiveStreamDonation[];
  startedAt: string;
}

export interface SocialChannel {
  id: string;
  platform: 'TikTok' | 'YouTube' | 'Twitch' | 'Facebook Reels' | 'Instagram';
  name: string;
  handle: string;
  avatar: string;
  status: 'LiveNow' | 'Active' | 'Growing' | 'Listed' | 'Sold';
  category: 'Gaming & Reaction' | 'Storytelling & Mystery' | 'Lofi & Healing Talks' | 'AI Tech & Memes';
  followers: number;
  views30d: number;
  monthlyDonationMinor: number;
  estimatedValuationMinor: number; // Định giá bán kênh trên thị trường
  saleStatus: 'NotForSale' | 'AcceptingOffers' | 'Listed' | 'Sold';
  engagementRateBps: number;
  niche: string;
  activeStreamSession?: LivestreamSession;
  totalStreamsRun: number;
}



