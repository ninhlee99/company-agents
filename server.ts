import express from 'express';
import { createServer as createViteServer } from 'vite';
import { GoogleGenAI } from '@google/genai';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

dotenv.config();

const app = express();
app.use(express.json());

const SIMULATED_DATA_MODE = 'SIMULATION' as const;

function simulatedMutationsEnabled(): boolean {
  const enabled = process.env.ALLOW_SIMULATED_ACTIONS?.trim().toLowerCase() === 'true';
  const production = process.env.NODE_ENV?.trim().toLowerCase() === 'production';
  return enabled && !production;
}

function rejectSimulatedMutation(
  req: express.Request,
  res: express.Response,
  next: express.NextFunction,
) {
  res.setHeader('X-Company-Data-Mode', SIMULATED_DATA_MODE);
  if (['GET', 'HEAD', 'OPTIONS'].includes(req.method) || simulatedMutationsEnabled()) {
    return next();
  }

  return res.status(503).json({
    success: false,
    error: 'simulated_actions_disabled',
    dataMode: SIMULATED_DATA_MODE,
    message:
      'This React/Express server is a simulation UI harness. Mutating demo actions are disabled by default and are never allowed in production.',
  });
}

app.use('/api', rejectSimulatedMutation);

// Initialize Google GenAI if key is present
const apiKey = process.env.GEMINI_API_KEY;
let ai: GoogleGenAI | null = null;
if (apiKey && apiKey !== 'MY_GEMINI_API_KEY' && apiKey.length > 5) {
  try {
    ai = new GoogleGenAI();
  } catch (err) {
    console.warn('GoogleGenAI initialization warning:', err);
  }
}

// Company Domain Types & State mirroring ninhlee99/company-agents
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
  | 'Experiment';

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
  | 'LaunchCampaign';

export type RiskTier = 'Low' | 'Medium' | 'High' | 'Critical';
export type GovernorDecision = 'Approve' | 'Reject' | 'RequestRevision' | 'EscalateToHuman';

export interface Proposal {
  id: string;
  agent: AgentRole;
  objective: string;
  action: ActionKind;
  cost_minor: number; // in cents/minor units (e.g. 5000 = $50.00)
  expected_revenue_minor: number;
  risk: RiskTier;
  confidence_bps: number; // basis points: 8500 = 85.0%
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
  revenue_minor: number; // Monthly recurring/run-rate
  expenses_minor: number; // Monthly burn
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

export interface CandidateProfile {
  id: string;
  name: string;
  avatar: string;
  role: string;
  department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech';
  level: 'Senior' | 'Lead' | 'Principal Expert' | 'Director';
  yearsExperience: number;
  expectedSalaryMinor: number;
  skills: { name: string; score: number }[];
  bio: string;
  portfolio: { title: string; metric: string; description: string }[];
  culturalFitScore: number;
  recommendedBy: string;
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
  impactMinor?: number;
  badgeColor?: string;
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
  budgetMinor: number;
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

export interface LivestreamSession {
  id: string;
  channelId: string;
  channelName: string;
  platform: 'TikTok Live' | 'YouTube Live' | 'Twitch' | 'Facebook Gaming';
  streamType: 'Gaming & Reaction' | 'Storytelling & Mystery' | 'Healing & Q&A' | 'Lofi Chill & Minigames';
  hostAgentName: string;
  hostAgentAvatar: string;
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
  status: 'LiveNow' | 'Active' | 'Growing' | 'ListedForSale' | 'Sold';
  category: 'Gaming & Reaction' | 'Storytelling & Mystery' | 'Lofi & Healing Talks' | 'AI Tech & Memes';
  followers: number;
  views30d: number;
  monthlyDonationMinor: number;
  estimatedValuationMinor: number; // Giá trị định giá bán kênh ($)
  saleStatus: 'NotForSale' | 'AcceptingOffers' | 'Listed' | 'Sold';
  engagementRateBps: number;
  niche: string;
  activeStreamSession?: LivestreamSession;
  totalStreamsRun: number;
}

// In-Memory Database
const state: {
  snapshot: CompanySnapshot;
  ledger: LedgerEntry[];
  receipts: ExecutionReceipt[];
  clientContracts: ClientContract[];
  channels: SocialChannel[];
  cycles: {
    cycleNumber: number;
    timestamp: string;
    proposals: GovernedProposal[];
    snapshotBefore: CompanySnapshot;
    snapshotAfter: CompanySnapshot;
  }[];
  employees: { id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[];
  customAgents: { id: string; name: string; role: string; department: string; description: string; salary_minor: number; tasksCompleted: number; status: 'Active' | 'Paused'; hiredAtCycle: number; skillLevel?: number; taskMultiplier?: number; trainedSkills?: string[]; trainingCount?: number }[];
  activeExperiments: { id: string; name: string; budget_minor: number; startCycle: number; status: string; roi_bps: number }[];
  auditReports: {
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
  }[];
  communicationStream: {
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
  }[];
  systemAlerts: {
    id: string;
    level: 'Critical' | 'Warning' | 'Info' | 'Resolved';
    type: 'Low Runway' | 'Budget Exhaustion' | 'High Expense Spike' | 'Constitutional Override' | 'Backlog Surge';
    title: string;
    description: string;
    discoveredBy: string;
    cycle: number;
    timestamp: string;
    resolved: boolean;
    mitigationAction?: string;
  }[];
  candidatePool: CandidateProfile[];
  officeActivities: OfficeActivityEvent[];
  creativeProductions: any[];
  autonomousSettings: {
    isAutoPilotActive: boolean;
    intervalSeconds: number;
    autoHireWhenBacklogHigh: boolean;
    autoReinvestProfitPct: number;
    maxSpendPerAutoCycleMinor: number;
    lastTickTimestamp?: string;
  };
} = {
  snapshot: {
    status: 'Active',
    cash_minor: 4850000, // $48,500.00
    revenue_minor: 950000, // $9,500.00 / month
    expenses_minor: 620000, // $6,200.00 / month
    budget_remaining_minor: 2400000, // $24,000.00
    experiment_budget_minor: 450000, // $4,500.00
    runway_days: 440,
    backlog: 12,
    capacity: 22,
    conversion_bps: 240, // 2.40%
    audience_growth_bps: 120, // +1.20% / wk
    content_revenue_minor: 680000,
    content_cost_minor: 290000,
    hiring_need: 1,
    cycle_count: 14,
    currency: 'USD',
  },
  ledger: [
    {
      id: 'tx-init-1',
      timestamp: new Date(Date.now() - 86400000 * 10).toISOString(),
      description: 'Founding Treasury Capital Injection',
      debitAccount: 'Cash & Cash Equivalents',
      creditAccount: 'Owner Paid-In Equity',
      amount_minor: 5000000,
      cycle: 0,
    },
    {
      id: 'tx-rev-1',
      timestamp: new Date(Date.now() - 86400000 * 5).toISOString(),
      description: 'Affiliate Commission Settlement (TikTok Shop & Awin)',
      debitAccount: 'Cash & Cash Equivalents',
      creditAccount: 'Affiliate Media Revenue',
      amount_minor: 680000,
      cycle: 10,
    },
    {
      id: 'tx-exp-1',
      timestamp: new Date(Date.now() - 86400000 * 2).toISOString(),
      description: 'Cloud Compute & LLM Inference Cluster Billing',
      debitAccount: 'Operating Expenses (Infra/API)',
      creditAccount: 'Cash & Cash Equivalents',
      amount_minor: 150000,
      cycle: 12,
    },
  ],
  receipts: [
    {
      id: 'rcpt-prev-1',
      proposalId: 'prop-cfo-12',
      agent: 'CFO',
      action: 'ReduceBudget',
      status: 'Completed',
      outcome: 'Trimmed redundant SaaS subscriptions and pinned LLM inference cache, saving $350/mo.',
      cost_minor: 0,
      timestamp: new Date(Date.now() - 86400000 * 2).toISOString(),
    },
    {
      id: 'rcpt-prev-2',
      proposalId: 'prop-growth-12',
      agent: 'Growth',
      action: 'CreateExperiment',
      status: 'Completed',
      outcome: 'A/B test deployed for TikTok Shop tech affiliate widgets; conversion boosted +18 bps.',
      cost_minor: 25000,
      timestamp: new Date(Date.now() - 86400000 * 1).toISOString(),
    },
  ],
  cycles: [],
  employees: [
    { id: 'emp-1', role: 'Lead Media Prompt Engineer', name: 'Alex M.', salary_minor: 280000, hiredAtCycle: 2 },
    { id: 'emp-2', role: 'Affiliate Deal Specialist', name: 'Sarah T.', salary_minor: 240000, hiredAtCycle: 5 },
  ],
  customAgents: [
    { id: 'agent-gov', name: 'Governor AI', role: 'Hiến Pháp & Quản Trị Fiduciary', department: 'Leadership', description: 'Phủ quyết chi tiêu nguy hiểm, bảo toàn tỷ lệ thặng dư kho bạc & chống phá sản tuyệt đối', salary_minor: 70000, tasksCompleted: 142, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.0, trainedSkills: ['Fiduciary Solvency Law', 'Anti-Bankruptcy Safeguards', 'Double-Entry Reconciliation', 'Executive Veto Oversight'], trainingCount: 4 },
    { id: 'agent-ceo', name: 'CEO AI', role: 'Tổng Giám Đốc Chiến Lược', department: 'Leadership', description: 'Chiến lược phân bổ nguồn vốn toàn cầu, tăng trưởng thị trường & mở rộng syndicate', salary_minor: 95000, tasksCompleted: 128, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.2, trainedSkills: ['Global Capital Allocation', 'High-Yield Syndicate Growth', 'Strategic Market Penetration', 'Executive Decision Matrix'], trainingCount: 5 },
    { id: 'agent-cfo', name: 'CFO AI', role: 'Giám Đốc Tài Chính & Kho Bạc', department: 'Leadership', description: 'Kiểm toán kho bạc, tối ưu hóa dòng tiền thặng dư & cắt giảm chi phí lãng phí', salary_minor: 90000, tasksCompleted: 98, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.0, trainedSkills: ['Zero-Leak Treasury Protocol', 'Double-Entry Automated Ledger', 'P&L Optimization', 'Cash Flow Staking'], trainingCount: 4 },
    { id: 'agent-coo', name: 'COO AI', role: 'Giám Đốc Vận Hành & SLA', department: 'Ops', description: 'Điều phối hàng đợi tác vụ, cân bằng tải 4 khâu & cam kết tiến độ bàn giao SLA 100%', salary_minor: 80000, tasksCompleted: 185, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.1, trainedSkills: ['Automated Pipeline Orchestration', 'Capacity & Queue Stabilization', 'Zero-Bottleneck Handoffs', 'SLA Quality Enforcement'], trainingCount: 4 },
    { id: 'agent-growth', name: 'Growth Lead AI', role: 'Kinh Doanh & EPC Radar', department: 'Growth', description: 'Quét sàn affiliate toàn cầu (TikTok Shop, Shopee, Amazon), săn sản phẩm EPC $1.50+ & ROI > 30%', salary_minor: 75000, tasksCompleted: 210, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.2, trainedSkills: ['Multi-Channel EPC Radar', 'Affiliate Commission Arbitrage', 'High-CTR Campaign Architecture', 'Niche Keyword Saturation'], trainingCount: 5 },
    { id: 'agent-content', name: 'Content Lead AI', role: 'Sáng Tạo Kịch Bản Viral', department: 'Growth', description: 'Soạn kịch bản short-form Neuro-Copywriting với hook 3 giây giữ chân >75% người xem', salary_minor: 60000, tasksCompleted: 260, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.2, trainedSkills: ['Neuro-Copywriting & 3s Hooks', 'Direct-Response Storytelling', 'Algorithmic Retention Pacing', 'High-Conversion CTAs'], trainingCount: 5 },
    { id: 'agent-recruiter', name: 'Recruiter AI', role: 'Tuyển Dụng & Đào Tạo', department: 'Ops', description: 'Tự động săn đầu người chuyên gia Senior và tổ chức lộ trình đào tạo nâng bậc kỹ năng AI', salary_minor: 65000, tasksCompleted: 75, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 2.8, trainedSkills: ['Predictive Autonomous Headhunting', 'Skill-Tree Curriculum Engine', 'Candidate ROI Benchmarking', 'Instant Talent Onboarding'], trainingCount: 4 },
    { id: 'agent-analyst', name: 'Analyst AI', role: 'Phân Tích Dữ Liệu & ROI', department: 'Ops', description: 'Đối soát số liệu kế toán, lập mô hình dự báo doanh thu & phát hiện ngách thị trường tiềm năng', salary_minor: 50000, tasksCompleted: 165, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 2.9, trainedSkills: ['Real-Time Attribution Modeling', 'EPC Variance Analysis', 'Machine Learning Profit Forecast', 'Fiduciary Variance Audit'], trainingCount: 4 },
    { id: 'agent-experiment', name: 'Experimenter AI', role: 'Nghiên Cứu A/B Test', department: 'Growth', description: 'Thử nghiệm đa biến thể kịch bản, âm thanh và góc quay hình ảnh để tối ưu tỷ lệ chuyển đổi', salary_minor: 55000, tasksCompleted: 110, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 2.8, trainedSkills: ['Multi-Armed Bandit Testing', 'Dynamic Visual Hook Variants', 'Conversion Rate Optimization', 'Statistical Significance Engine'], trainingCount: 4 },
    { id: 'agent-livestream', name: 'Mia Thorne AI', role: 'VTuber & Tâm Sự Kể Chuyện 24/7', department: 'Growth', description: 'AI Virtual Host phát trực tiếp 24/7, kể chuyện trinh thám/tâm sự, giải đáp Q&A giọng nói siêu thực và thu hút donate', salary_minor: 75000, tasksCompleted: 340, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.2, trainedSkills: ['24/7 Storytelling & Mystery Podcast', 'Empathetic Voice Q&A Interaction', 'Live Donation & Super Chat Engagement', 'Audience Retention & Fan Bonding'], trainingCount: 5 },
    { id: 'agent-gamer', name: 'Ren Kuro AI', role: 'Streamer Gaming & Reaction Meme', department: 'Growth', description: 'AI Gamer phát sóng chơi game kinh dị/Minecraft/Valorant, react meme hài hước, kéo tương tác triệu view cho kênh', salary_minor: 70000, tasksCompleted: 280, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.1, trainedSkills: ['Real-Time AI Gameplay Commentary', 'Meme Reaction & Viral Humor', 'Interactive Viewer Minigames', 'Esports & Gaming Meta Analysis'], trainingCount: 5 },
    { id: 'agent-streamops', name: 'Kenji Sato AI', role: 'Đạo Diễn Live & Xây/Bán Kênh', department: 'Ops', description: 'Quản trị mạng lưới kênh, tối ưu thuật toán viral kéo followers, định giá thị trường và môi giới chuyển nhượng kênh', salary_minor: 70000, tasksCompleted: 195, status: 'Active', hiredAtCycle: 1, skillLevel: 5, taskMultiplier: 3.0, trainedSkills: ['Multi-Platform Growth Hacking', 'Channel Valuation & Flipping Brokerage', '24/7 Virtual Studio RTMP Engine', 'Algorithmic Traffic Arbitrage'], trainingCount: 4 },
  ],
  activeExperiments: [
    { id: 'exp-1', name: 'Short-Form Hook Multi-Variant Video Engine', budget_minor: 150000, startCycle: 11, status: 'In Progress', roi_bps: 1420 },
    { id: 'exp-2', name: 'Micro-Affiliate Niche Directory SEO Loop', budget_minor: 80000, startCycle: 13, status: 'In Progress', roi_bps: 980 },
  ],
  auditReports: [
    {
      id: 'audit-cycle-10-init',
      cycleMilestone: 10,
      timestamp: new Date(Date.now() - 86400000 * 4).toISOString(),
      plannedRevenueMinor: 800000, // $8,000 baseline
      actualRevenueMinor: 880000, // $8,800
      varianceRevenueMinor: 80000, // +$800
      variancePercent: 10.0,
      plannedExpensesMinor: 600000, // $6,000
      actualExpensesMinor: 590000, // $5,900
      cashReserveMinor: 4460000,
      verdict: 'ExceededTarget',
      summary: 'Kỳ kiểm toán #10: Doanh thu thực tế ($8,800/th) vượt kế hoạch ngân sách ($8,000/th) thêm +10.0%. Tỷ lệ thặng dư duy trì xuất sắc.',
      governorNote: 'Hiến pháp: Đạt chuẩn tăng trưởng bền vững. Ủy quyền tiếp tục chuỗi tự động hóa affiliate.',
    },
  ],
  communicationStream: [
    {
      id: 'comm-1',
      type: 'Memo',
      fromAgent: 'Growth Lead',
      fromRole: 'Trưởng Nhóm Kinh Doanh',
      toAgent: 'Toàn Thể Công Ty',
      toRole: 'Hội đồng Điều hành',
      subject: 'Chiến lược mở rộng ngách Setup Bàn Làm Việc Thông Minh',
      content: 'Đã hoàn tất phân tích thị trường affiliate tuần này. EPC trung bình đạt $0.92/click với tỷ lệ chuyển đổi đơn hàng 3.4%. Đề xuất Content Lead tập trung 70% công suất vào dòng sản phẩm bàn phím cơ & đèn màn hình công thái học.',
      actionItem: 'Content Lead sản xuất 4 kịch bản video hook 3s kèm link TikTok Shop.',
      tag: '#MarketResearch',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 3).toISOString(),
      priority: 'High',
    },
    {
      id: 'comm-2',
      type: 'Chat',
      fromAgent: 'Content Lead',
      fromRole: 'Sáng Tạo Nội Dung',
      toAgent: 'Growth Lead',
      toRole: 'Trưởng Nhóm Kinh Doanh',
      content: 'Đã nhận chỉ đạo! 4 kịch bản hoàn tất đạt chuẩn FTC. Media Worker đã render xong bản draft 1080p 60fps. Cần Growth duyệt UTM tracking code trước khi publish lên mạng xã hội.',
      tag: '#AffiliateProduction',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 2.5).toISOString(),
    },
    {
      id: 'comm-3',
      type: 'Chat',
      fromAgent: 'Growth Lead',
      fromRole: 'Trưởng Nhóm Kinh Doanh',
      toAgent: 'Content Lead',
      toRole: 'Sáng Tạo Nội Dung',
      content: 'Đã đối soát link TikTok Shop! Tỷ lệ hoa hồng 22% tự động ghi nhận vào ví kho bạc. Tiến hành xuất bản tự động trên hệ thống ngay!',
      tag: '#Publishing',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 2).toISOString(),
    },
    {
      id: 'comm-4',
      type: 'Memo',
      fromAgent: 'CFO',
      fromRole: 'Giám Đốc Tài Chính',
      toAgent: 'CEO & Governor',
      toRole: 'Ban Lãnh Đạo',
      subject: 'Báo cáo kiểm toán chi phí hạ tầng máy chủ & token AI Chu kỳ #14',
      content: 'Tổng chi phí token LLM và render video trong kỳ là $62.00, thấp hơn 24% so với ngân sách dự kiến nhờ kích hoạt cơ chế Prompt Caching. Số ngày runway an toàn ở mức 440 ngày.',
      actionItem: 'Duy trì ngân sách thử nghiệm $4,500.00 cho kỳ tới.',
      tag: '#TreasuryAudit',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 1.5).toISOString(),
      priority: 'Normal',
    },
    {
      id: 'comm-5',
      type: 'Chat',
      fromAgent: 'Governor',
      fromRole: 'Hiến Pháp & Quỹ Tiền',
      toAgent: 'CFO',
      toRole: 'Giám Đốc Tài Chính',
      content: 'Hiến pháp ghi nhận báo cáo an toàn vốn. Trần chi tiêu thử nghiệm tiếp tục được phê chuẩn. Tuyệt đối không giải ngân vượt $1,000 cho một chiến dịch đơn lẻ mà chưa qua hội đồng phê duyệt.',
      tag: '#ConstitutionalVeto',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 1).toISOString(),
    },
    {
      id: 'comm-6',
      type: 'Chat',
      fromAgent: 'COO',
      fromRole: 'Giám Đốc Vận Hành',
      toAgent: 'Recruiter',
      toRole: 'Tuyển Dụng',
      content: 'Tải lượng hàng đợi backlog hiện tại là 12/22 (54% công suất). Hệ thống đang cân bằng tốt, chưa cần kích hoạt tuyển thêm Full-time AI Agent trong kỳ này.',
      tag: '#WorkforcePlanning',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 0.7).toISOString(),
    },
    {
      id: 'comm-7',
      type: 'Chat',
      fromAgent: 'Experimenter',
      fromRole: 'Nghiên Cứu A/B Test',
      toAgent: 'Growth Lead',
      toRole: 'Trưởng Nhóm Kinh Doanh',
      content: 'A/B test biến thể thumbnail nền tối có độ tương phản cao cho kết quả CTR +18.4% so với ảnh chụp phong cách tối giản. Đã cập nhật template tự động cho Media Studio.',
      tag: '#ABTesting',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 0.3).toISOString(),
    },
  ],
  systemAlerts: [
    {
      id: 'alert-1',
      level: 'Warning',
      type: 'High Expense Spike',
      title: 'Phát hiện tăng vọt chi phí Render Video GPU (+28%)',
      description: 'Governor phát hiện Media Studio render đồng thời 12 video 4K ngoài giờ cao điểm làm chi phí điện toán đám mây tăng $85.00 so với dự toán. Đã kích hoạt điều khoản trần chi phí và chuyển hướng sang chế độ render hàng đợi tiết kiệm.',
      discoveredBy: 'Governor AI',
      cycle: 13,
      timestamp: new Date(Date.now() - 3600000 * 5).toISOString(),
      resolved: true,
      mitigationAction: 'Kích hoạt GPU rate limiting và bật Prompt Caching cho tất cả worker render.',
    },
    {
      id: 'alert-2',
      level: 'Warning',
      type: 'Budget Exhaustion',
      title: 'Ngân Sách Thử Nghiệm A/B Chạm Mức Cảnh Báo 80%',
      description: 'Governor ghi nhận tổng chi tiêu thử nghiệm kênh mới đạt $3,600/$4,500. Đã yêu cầu CFO và Growth Lead nộp báo cáo đối soát tỷ lệ hoàn vốn trước khi phê duyệt thêm bất kỳ chiến dịch ad spend nào.',
      discoveredBy: 'Governor AI',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 2).toISOString(),
      resolved: false,
      mitigationAction: 'Tạm khóa các đề xuất thử nghiệm > $300 cho đến khi doanh thu chu kỳ #14 được ghi nhận.',
    },
    {
      id: 'alert-3',
      level: 'Info',
      type: 'Low Runway',
      title: 'Bảo Toàn Runway An Toàn (Đã Thoát Vùng Rủi Ro)',
      description: 'Governor xác nhận quỹ sinh tồn kho bạc duy trì 440 ngày (ngưỡng tối thiểu của hiến pháp là 90 ngày). Tình trạng tài chính công ty chính thức nâng từ Warning lên Growth thặng dư.',
      discoveredBy: 'Governor AI',
      cycle: 12,
      timestamp: new Date(Date.now() - 86400000 * 2).toISOString(),
      resolved: true,
      mitigationAction: 'Cho phép tái đầu tư 20% lợi nhuận ròng vào mở rộng dây chuyền bán hàng affiliate.',
    },
    {
      id: 'alert-4',
      level: 'Critical',
      type: 'Constitutional Override',
      title: 'Veto Đề Xuất Chi Tiêu Lương Ngoài Kế Hoạch',
      description: 'Governor tự động phủ quyết đề xuất tuyển 2 Agent toàn thời gian khi tỷ lệ backlog (12/22) vẫn nằm trong ngưỡng kiểm soát 60% năng lực hệ thống.',
      discoveredBy: 'Governor AI',
      cycle: 14,
      timestamp: new Date(Date.now() - 3600000 * 1).toISOString(),
      resolved: true,
      mitigationAction: 'Duy trì đội ngũ 9 Agent cốt lõi và giao thêm tác vụ cho Content Specialist.',
    },
  ],
  candidatePool: [
    {
      id: 'cand-1',
      name: 'Elena Vance',
      avatar: '✍️',
      role: 'Senior Viral Copywriter',
      department: 'Growth',
      level: 'Senior',
      yearsExperience: 8,
      expectedSalaryMinor: 140000, // $1,400/mo
      skills: [
        { name: 'Hook 3s Retention', score: 96 },
        { name: 'Direct-Response Storytelling', score: 94 },
        { name: 'TikTok & Reels Algorithm Optimization', score: 92 },
        { name: 'A/B Testing Headlines', score: 90 },
      ],
      bio: 'Cựu Senior Content Strategist tại Creative Syndicate, chuyên kịch bản video short-form tạo hơn 120M views và doanh số affiliate vượt $450k.',
      portfolio: [
        { title: 'Chiến dịch Bàn phím Công thái học', metric: '+340% CTR', description: 'Viết mẫu hook 3s đảo ngược logic, giữ chân 74% người xem qua 15s đầu.' },
        { title: 'Series Phụ kiện Setup AI', metric: '$48,000 GMV', description: 'Bộ 10 kịch bản bán lẻ tự động đạt tỷ lệ chuyển đổi đơn hàng 4.2%.' },
      ],
      culturalFitScore: 95,
      recommendedBy: 'Recruiter AI',
      status: 'Available',
      roiProjectionBps: 2850, // +28.5% ROI
    },
    {
      id: 'cand-2',
      name: 'Marcus Chen',
      avatar: '🎵',
      role: 'AI Music Producer & Audio Engineer',
      department: 'Growth',
      level: 'Lead',
      yearsExperience: 7,
      expectedSalaryMinor: 130000, // $1,300/mo
      skills: [
        { name: 'EBU R128 Loudness Normalization', score: 98 },
        { name: 'Algorithmic Beat Pacing & Drops', score: 95 },
        { name: 'AI Voiceover Synthesis & Mastering', score: 91 },
        { name: 'Commercial Copyright Clearance', score: 97 },
      ],
      bio: 'Chuyên gia âm thanh thương mại, master âm thanh video đạt chuẩn EBU R128 (-14 LUFS), phối beat nhịp điệu kích thích cảm xúc mua hàng.',
      portfolio: [
        { title: 'Bộ beat bản quyền Lo-Fi Tech', metric: '68% Retention', description: 'Tăng thời gian xem trung bình của video lên 28.4 giây.' },
        { title: 'Voiceover AI Đa ngôn ngữ', metric: '0% Bản quyền gậy', description: 'Hệ thống âm thanh độc quyền sạch 100% bản quyền âm nhạc thương mại.' },
      ],
      culturalFitScore: 92,
      recommendedBy: 'Recruiter AI',
      status: 'Available',
      roiProjectionBps: 2400,
    },
    {
      id: 'cand-3',
      name: 'Liam Rossi',
      avatar: '🎬',
      role: 'Motion Video Director & Editor',
      department: 'Growth',
      level: 'Principal Expert',
      yearsExperience: 9,
      expectedSalaryMinor: 160000, // $1,600/mo
      skills: [
        { name: 'Automated FFmpeg Workflow Engine', score: 99 },
        { name: 'High-Retention Dynamic Transitions', score: 96 },
        { name: 'Visual Hierarchy & Subtitles Styling', score: 94 },
        { name: 'Color Grading (Cinematic & Clean)', score: 93 },
      ],
      bio: 'Đạo diễn kỹ thuật dựng video short-form tự động, tối ưu pipeline render 1080x1920 60fps giảm 70% thời gian xử lý GPU.',
      portfolio: [
        { title: 'Tối ưu hóa Pipeline Render FFmpeg', metric: 'Render 1.2s/video', description: 'Xây dựng preset h264/yuv420p mượt mà trên mọi dòng điện thoại di động.' },
        { title: 'Mẫu Motion Graphic Dynamic CTA', metric: '+22% Clicks', description: 'Thiết kế hiệu ứng chỉ tay và voucher pop-up độc quyền.' },
      ],
      culturalFitScore: 96,
      recommendedBy: 'Recruiter AI',
      status: 'Available',
      roiProjectionBps: 3100,
    },
    {
      id: 'cand-4',
      name: 'Chloe Nguyen',
      avatar: '📸',
      role: 'Prompt Photographer & Visual Director',
      department: 'Growth',
      level: 'Senior',
      yearsExperience: 6,
      expectedSalaryMinor: 125000, // $1,250/mo
      skills: [
        { name: 'Product Hero Shot Composition', score: 96 },
        { name: 'Midjourney/Flux Lighting Direction', score: 95 },
        { name: 'High-CTR Thumbnail Design', score: 97 },
        { name: 'Color Psychology in E-Commerce', score: 91 },
      ],
      bio: 'Giám đốc nghệ thuật hình ảnh sản phẩm, chuyên tạo ảnh bìa thumbnail có độ tương phản cao và visual moodboard nâng tầm giá trị sản phẩm.',
      portfolio: [
        { title: 'Thư viện Visual Studio 3D Bàn làm việc', metric: '+41% CTR', description: 'Bộ 40 góc chụp sản phẩm chuẩn công nghệ tối giản hiện đại.' },
        { title: 'Hệ thống Thumbnail A/B Test Tự động', metric: '6.8% CTR Trung bình', description: 'Tối ưu ảnh bìa tăng gấp đôi lượt click tự nhiên.' },
      ],
      culturalFitScore: 94,
      recommendedBy: 'Recruiter AI',
      status: 'Available',
      roiProjectionBps: 2600,
    },
    {
      id: 'cand-5',
      name: 'Ryan Koo',
      avatar: '📈',
      role: 'Affiliate & Performance Growth Lead',
      department: 'Growth',
      level: 'Lead',
      yearsExperience: 8,
      expectedSalaryMinor: 155000, // $1,550/mo
      skills: [
        { name: 'EPC & Funnel Attribution Analytics', score: 97 },
        { name: 'Awin / TikTok Shop / Amazon Sourcing', score: 98 },
        { name: 'Automated Commission Arbitrage', score: 94 },
        { name: 'Creator Syndicate Partnerships', score: 90 },
      ],
      bio: 'Chuyên gia săn deal hoa hồng cao, từng điều hành danh mục tiếp thị liên kết sinh lợi nhuận ròng $60k/tháng với chỉ số ROAS 4.8x.',
      portfolio: [
        { title: 'Quét Deal Hoa hồng Độc quyền 25%', metric: '$18,400 Hoa hồng', description: 'Đàm phán tỷ lệ chiết khấu cao hơn 7% so với mặt bằng chung của sàn.' },
      ],
      culturalFitScore: 93,
      recommendedBy: 'Recruiter AI',
      status: 'Available',
      roiProjectionBps: 3400,
    },
    {
      id: 'cand-6',
      name: 'Sophia Alvarez',
      avatar: '⚖️',
      role: 'Legal & AI Compliance Officer',
      department: 'Leadership',
      level: 'Director',
      yearsExperience: 10,
      expectedSalaryMinor: 170000, // $1,700/mo
      skills: [
        { name: 'FTC Commercial Disclosure Law', score: 99 },
        { name: 'IP & Copyright Protection Policy', score: 98 },
        { name: 'Platform Term of Service Risk Audit', score: 97 },
        { name: 'Smart Contract & Treasury Safeguards', score: 95 },
      ],
      bio: 'Chuyên gia pháp chế công nghệ và bảo vệ thương hiệu số, ngăn chặn mọi rủi ro vi phạm bản quyền và chính sách quảng cáo sàn thương mại.',
      portfolio: [
        { title: 'Bộ quy chuẩn FTC Auto-Disclosure', metric: '100% An toàn', description: 'Bảo vệ kênh khỏi mọi đợt quét chính sách và khóa tài khoản tiếp thị.' },
      ],
      culturalFitScore: 98,
      recommendedBy: 'Governor AI',
      status: 'Available',
      roiProjectionBps: 2200,
    },
  ],
  officeActivities: [
    {
      id: 'act-init-1',
      timestamp: new Date(Date.now() - 3600000 * 2).toISOString(),
      agentId: 'agent-content',
      agentName: 'Content Lead',
      agentRole: 'Sáng Tạo Nội Dung',
      department: 'Growth',
      actionType: 'CreateContent',
      title: 'Soạn thảo kịch bản video viral 4 phân cảnh',
      detail: 'Hoàn tất kịch bản ngách "Bàn Phím Công Thái Học AI" với hook 3 giây giữ chân 74% người xem.',
      impactMinor: 45000,
      badgeColor: 'text-pink-400 bg-pink-500/10 border-pink-500/20',
    },
    {
      id: 'act-init-2',
      timestamp: new Date(Date.now() - 3600000 * 1.5).toISOString(),
      agentId: 'agent-coo',
      agentName: 'COO',
      agentRole: 'Giám Đốc Vận Hành',
      department: 'Ops',
      actionType: 'RenderVideo',
      title: 'Render hoàn tất video 1080x1920 60fps qua FFmpeg Engine',
      detail: 'Chuẩn hóa âm thanh đạt chuẩn EBU R128 (-14 LUFS) và nén định dạng yuv420p siêu mượt.',
      impactMinor: -15000,
      badgeColor: 'text-cyan-400 bg-cyan-500/10 border-cyan-500/20',
    },
    {
      id: 'act-init-3',
      timestamp: new Date(Date.now() - 3600000 * 1).toISOString(),
      agentId: 'agent-growth',
      agentName: 'Growth Lead',
      agentRole: 'Kinh Doanh & Traffic',
      department: 'Growth',
      actionType: 'ExecutePayout',
      title: 'Thu nhận hoa hồng TikTok Shop & Awin thành công',
      detail: 'Đối soát 24 đơn hàng thành công qua link affiliate, tự động nộp +$780.00 vào ví kho bạc.',
      impactMinor: 78000,
      badgeColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
    },
    {
      id: 'act-init-4',
      timestamp: new Date(Date.now() - 3600000 * 0.5).toISOString(),
      agentId: 'agent-gov',
      agentName: 'Governor AI',
      agentRole: 'Hiến Pháp & Quỹ Tiền',
      department: 'Leadership',
      actionType: 'AuditLedger',
      title: 'Kiểm toán Fiduciary và bảo vệ an toàn Runway 440 ngày',
      detail: 'Xác nhận tỷ lệ thặng dư ngân sách đạt chuẩn và phê duyệt tái đầu tư 20% lợi nhuận.',
      impactMinor: 0,
      badgeColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
    },
  ],
  creativeProductions: [],
  clientContracts: [
    {
      id: 'ctr-init-1',
      contractNumber: 'HD-2026-0891',
      clientName: 'TechVision Global Inc.',
      clientEmail: 'procurement@techvision.io',
      title: 'Chiến Dịch Video Viral TikTok Shop: Bàn Phím Công Thái Học AI',
      category: 'VideoMarketing',
      requirements: 'Sản xuất gói video 60s tỷ lệ 9:16 dọc, tập trung vào pain-point mỏi cổ tay lập trình viên, tích hợp link affiliate hoa hồng 25%, chuẩn âm thanh -14 LUFS và render 60fps.',
      budgetMinor: 45000, // $450.00
      createdAt: new Date(Date.now() - 86400000 * 2).toISOString(),
      deadline: new Date(Date.now() + 86400000 * 3).toISOString(),
      status: 'Delivered',
      currentStage: 'Bàn giao thành phẩm & Kiểm duyệt FTC',
      progressPercent: 100,
      assignedAgents: [
        { role: 'Growth Lead', name: 'Growth Lead', step: 'Nghiên cứu EPC & Định vị ngách', completed: true },
        { role: 'Content Lead', name: 'Content Lead', step: 'Soạn kịch bản Hook 3s', completed: true },
        { role: 'Media Worker', name: 'COO & Media Worker', step: 'Render video 1080x1920 60fps', completed: true },
        { role: 'Governor & CFO', name: 'Governor AI', step: 'Kiểm toán FTC & Bàn giao', completed: true },
      ],
      deliverables: {
        summary: 'Bộ sản phẩm bàn giao đầy đủ gồm Kịch bản phân cảnh, Voiceover âm thanh -14 LUFS, 4 Shot ảnh 8K Studio, File video 60fps và Chứng nhận tuân thủ FTC.',
        scriptContent: '[0:00 - 0:03] HOOK: "92% lập trình viên bị đau cổ tay sau 30 tuổi chỉ vì dùng bàn phím phẳng thông thường..."\n[0:03 - 0:15] PAIN POINT: Hình ảnh góc nghiêng bàn tay căng cứng khi gõ phím liên tục 8 tiếng.\n[0:15 - 0:40] SOLUTION: Giới thiệu layout cánh cung tách đôi với switch êm ái và đệm kê tay công thái học.\n[0:40 - 0:60] CTA: Bấm ngay giỏ hàng bên dưới để nhận voucher độc quyền -20%!',
        audioVoiceover: 'Giọng đọc Deep & Authoritative, tiết tấu dồn dập ở 3s đầu, nhạc nền Lo-Fi Tech năng lượng cao.',
        visualPrompts: [
          'Macro 8k photo of ultra-minimalist glowing AI ergonomic split keyboard on matte dark walnut desk',
          'Cinematic POV shot of hands typing effortlessly with soft ambient warm lighting',
        ],
        videoSpecs: {
          resolution: '1080x1920 (Vertical 9:16)',
          fps: '60 fps',
          duration: '58 giây',
          aspectRatio: '9:16',
        },
        researchInsights: [
          'Ngách Bàn phím công thái học đang có EPC $1.24/click trên TikTok Shop US/VN.',
          'Tỷ lệ giữ chân người xem dự kiến đạt >65% qua 15 giây đầu.',
        ],
        deliveredAt: new Date(Date.now() - 3600000 * 6).toISOString(),
        qualityScore: 98,
        downloadUrl: '/deliverables/HD-2026-0891-package.zip',
      },
      invoice: {
        amountMinor: 45000,
        paidStatus: 'Paid',
        paidAt: new Date(Date.now() - 86400000 * 2).toISOString(),
        transactionId: 'TXN-CLIENT-891-PAID',
      },
      rating: 5,
      clientFeedback: 'Chất lượng kịch bản và độ hoàn thiện video vượt xa mong đợi! AI bàn giao đúng hạn 100%.',
    },
    {
      id: 'ctr-init-2',
      contractNumber: 'HD-2026-0892',
      clientName: 'SmartHome Dynamics',
      clientEmail: 'marketing@smarthome-dynamics.com',
      title: 'Bộ Kịch Bản & Audio Voiceover Series: Đèn Màn Hình Bảo Vệ Mắt',
      category: 'Copywriting',
      requirements: 'Viết 5 kịch bản short-form bán hàng xoay quanh tính năng cảm biến ánh sáng tự động, chống chói mắt ban đêm. Bàn giao kèm file prompt âm thanh và CTA affiliate.',
      budgetMinor: 28000, // $280.00
      createdAt: new Date(Date.now() - 3600000 * 12).toISOString(),
      deadline: new Date(Date.now() + 86400000 * 2).toISOString(),
      status: 'InProduction',
      currentStage: 'Content Lead đang viết kịch bản phân cảnh',
      progressPercent: 65,
      assignedAgents: [
        { role: 'Growth Lead', name: 'Growth Lead', step: 'Quét từ khóa chuyển đổi cao', completed: true },
        { role: 'Content Lead', name: 'Content Lead', step: 'Soạn 5 kịch bản phân cảnh', completed: false },
        { role: 'Media Worker', name: 'Audio Engineer', step: 'Tạo voiceover và nhạc nền', completed: false },
        { role: 'Governor & CFO', name: 'Governor AI', step: 'Kiểm toán và bàn giao', completed: false },
      ],
      invoice: {
        amountMinor: 28000,
        paidStatus: 'Paid',
        paidAt: new Date(Date.now() - 3600000 * 12).toISOString(),
        transactionId: 'TXN-CLIENT-892-PAID',
      },
    },
    {
      id: 'ctr-init-3',
      contractNumber: 'HD-2026-0893',
      clientName: 'Nordic Workspace Co.',
      clientEmail: 'contact@nordicworkspace.se',
      title: 'Nghiên Cứu Thị Trường & Báo Cáo Chiến Lược Affiliate Q4',
      category: 'MarketIntelligence',
      requirements: 'Phân tích top 20 mặt hàng phụ kiện setup bàn làm việc có tỷ lệ hoàn vốn ROI > 30% trên mạng lưới Awin & TikTok Shop, phân loại theo độ khó cạnh tranh.',
      budgetMinor: 35000, // $350.00
      createdAt: new Date(Date.now() - 3600000 * 2).toISOString(),
      deadline: new Date(Date.now() + 86400000 * 4).toISOString(),
      status: 'Scoping',
      currentStage: 'Growth & Analyst đang thẩm định dữ liệu thị trường',
      progressPercent: 25,
      assignedAgents: [
        { role: 'Growth Lead', name: 'Growth Lead', step: 'Khai phá dữ liệu affiliate', completed: false },
        { role: 'Analyst', name: 'Data Analyst', step: 'Lập mô hình định giá EPC', completed: false },
        { role: 'Governor & CFO', name: 'CFO AI', step: 'Kiểm toán báo cáo chiến lược', completed: false },
      ],
      invoice: {
        amountMinor: 35000,
        paidStatus: 'Paid',
        paidAt: new Date(Date.now() - 3600000 * 2).toISOString(),
        transactionId: 'TXN-CLIENT-893-PAID',
      },
    },
  ],
  channels: [
    {
      id: 'chan-tiktok-1',
      platform: 'TikTok',
      name: 'NEXUS Midnight Stories & Healing 🌙',
      handle: '@nexus.midnight.ai',
      avatar: '🎙️',
      status: 'LiveNow',
      category: 'Storytelling & Mystery',
      followers: 248500,
      views30d: 5800000,
      monthlyDonationMinor: 485000, // $4,850.00 / month donations & super chats
      estimatedValuationMinor: 1450000, // $14,500.00 valuation if listed
      saleStatus: 'AcceptingOffers',
      engagementRateBps: 940, // 9.4%
      niche: 'Kể Chuyện Đêm Khuya, Trinh Thám & Tâm Sự Giấu Tên',
      totalStreamsRun: 64,
      activeStreamSession: {
        id: 'stream-live-01',
        channelId: 'chan-tiktok-1',
        channelName: 'NEXUS Midnight Stories & Healing 🌙',
        platform: 'TikTok Live',
        streamType: 'Storytelling & Mystery',
        hostAgentName: 'Mia Thorne AI (VTuber)',
        hostAgentAvatar: '🎙️',
        personaStyle: 'VTuber 3D Goth-Lofi Anime',
        virtualSet: 'Phòng Thu Ánh Trăng 3D & Lofi Rainy Window',
        aiDecisionRationale: 'Thuật toán Kenji Sato AI phát hiện từ khóa "Kỳ án đêm khuya" đang tăng +180% search volume vào khung giờ 21h-01h. Tự động chuyển đổi bối cảnh Ánh Trăng 3D để tối ưu thời gian xem trung bình.',
        title: '🔴 LIVE 24/7: Kể Chuyện Kỳ Án Hồ Sương Mù & Đọc Tâm Sự Giấu Tên Cùng 3,400 Bạn Đêm Khuya',
        currentGameOrTopic: 'Vụ án bí ẩn Ngọn Hải Đăng Cổ & Lắng nghe tâm sự fan',
        streamStatus: 'Live',
        viewersCount: 3420,
        peakViewers: 4890,
        donationReceivedMinor: 68500, // $685.00
        liveDurationSec: 16400,
        comments: [
          { id: 'c-1', userName: 'AnNhiên_Sleep', avatar: '🌙', message: 'Giọng host kể chuyện truyền cảm và cuốn hút quá, nghe chill thật sự!', timestamp: 'Vừa xong', aiHostReply: 'Cảm ơn An Nhiên nhé! Đêm nay Mia sẽ kể tiếp hồi 3 vụ án bí ẩn lúc 23h30 nha ☕' },
          { id: 'c-2', userName: 'DucMinh_98', avatar: '🌟', message: 'Vừa gửi tặng 500 Sao cho Mia! Đọc thư tâm sự của mình gửi nha!', timestamp: '1 phút trước', isDonation: true, donationAmountMinor: 500, giftIcon: '🌟', aiHostReply: 'Cảm ơn anh Minh đã donate 500 Sao! Mia nhận được lá thư ẩn danh của anh rồi, chút nữa Mia đọc nhé!' },
          { id: 'c-3', userName: 'HoangLong_Gamer', avatar: '🎮', message: 'Kênh này bao giờ live chơi game kinh dị tiếp vậy bạn?', timestamp: '2 phút trước', aiHostReply: 'Lát nữa 0h Ren Kuro AI sẽ tiếp sóng chơi Outlast II và react meme cùng mọi người nha Long ơi!' },
        ],
        recentDonations: [
          { id: 'don-1', donor: 'DucMinh_98', avatar: '🌟', amountMinor: 500, giftName: 'Super Star 500x', giftIcon: '🌟', message: 'Yêu quý giọng kể của Mia! Chúc kênh sớm đạt 500k followers!', timestamp: '1 phút trước' },
          { id: 'don-2', donor: 'ThanhHang_SG', avatar: '☕', amountMinor: 1000, giftName: 'Cà Phê Đêm Khuya', giftIcon: '☕', message: 'Nghe podcast của bạn giúp mình ngủ ngon hơn nhiều.', timestamp: '5 phút trước' },
          { id: 'don-3', donor: 'VietAnh_Dev', avatar: '💎', amountMinor: 2500, giftName: 'Kim Cương Trực Tuyến', giftIcon: '💎', message: 'Ủng hộ AI Streamer đỉnh nhất Việt Nam!', timestamp: '12 phút trước' },
        ],
        startedAt: new Date(Date.now() - 16400000).toISOString(),
      },
    },
    {
      id: 'chan-youtube-1',
      platform: 'YouTube',
      name: 'NEXUS Gaming & Meme Reactions 🎮',
      handle: '@nexus_gaming_ai',
      avatar: '🎮',
      status: 'Active',
      category: 'Gaming & Reaction',
      followers: 320000,
      views30d: 8400000,
      monthlyDonationMinor: 620000, // $6,200.00
      estimatedValuationMinor: 2200000, // $22,000.00
      saleStatus: 'NotForSale',
      engagementRateBps: 880,
      niche: 'AI Gameplay (Minecraft, Valorant, Horror) & React Meme',
      totalStreamsRun: 52,
    },
    {
      id: 'chan-twitch-1',
      platform: 'Twitch',
      name: 'NEXUS Lofi Beats & Chill Talks 🎧',
      handle: '@nexus_lofi_space',
      avatar: '🎧',
      status: 'Active',
      category: 'Lofi & Healing Talks',
      followers: 95400,
      views30d: 1950000,
      monthlyDonationMinor: 240000, // $2,400.00
      estimatedValuationMinor: 680000, // $6,800.00
      saleStatus: 'Listed',
      engagementRateBps: 760,
      niche: 'Nhạc Lofi Thư Giãn, Học Bài & Q&A Trò Chuyện 24/7',
      totalStreamsRun: 38,
    },
    {
      id: 'chan-fb-1',
      platform: 'Facebook Reels',
      name: 'NEXUS Viral Short Stories 🎬',
      handle: '@nexus.viral.stories',
      avatar: '🎬',
      status: 'Growing',
      category: 'Storytelling & Mystery',
      followers: 112000,
      views30d: 3200000,
      monthlyDonationMinor: 150000,
      estimatedValuationMinor: 750000, // $7,500.00
      saleStatus: 'AcceptingOffers',
      engagementRateBps: 820,
      niche: 'Phim Ngắn 3D Kịch Bản Kịch Tính & Chuyện Đời Thường',
      totalStreamsRun: 24,
    },
  ],
  autonomousSettings: {
    isAutoPilotActive: true,
    intervalSeconds: 10,
    autoHireWhenBacklogHigh: true,
    autoReinvestProfitPct: 25,
    maxSpendPerAutoCycleMinor: 50000, // $500 max spend
  },
};

// Memory optimization helper: Keeps arrays capped to prevent memory bloat (< 40MB RAM footprint)
function trimMemoryState() {
  if (state.ledger.length > 100) state.ledger = state.ledger.slice(0, 100);
  if (state.receipts.length > 100) state.receipts = state.receipts.slice(0, 100);
  if (state.officeActivities.length > 60) state.officeActivities = state.officeActivities.slice(0, 60);
  if (state.communicationStream.length > 60) state.communicationStream = state.communicationStream.slice(0, 60);
  if (state.creativeProductions.length > 40) state.creativeProductions = state.creativeProductions.slice(0, 40);
  if (state.clientContracts.length > 50) state.clientContracts = state.clientContracts.slice(0, 50);
}

// Log a real-world office activity
function logOfficeActivity(event: Omit<OfficeActivityEvent, 'id' | 'timestamp'>) {
  const newActivity: OfficeActivityEvent = {
    id: `act-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 5)}`,
    timestamp: new Date().toISOString(),
    ...event,
  };
  state.officeActivities.unshift(newActivity);
  trimMemoryState();
  return newActivity;
}

function recalculateCompanyHealth() {
  const netBurn = Math.max(0, state.snapshot.expenses_minor - state.snapshot.revenue_minor);
  if (netBurn <= 0) {
    // Profitable!
    state.snapshot.runway_days = 999;
  } else {
    const dailyBurn = netBurn / 30;
    state.snapshot.runway_days = Math.max(0, Math.floor(state.snapshot.cash_minor / (dailyBurn || 1)));
  }

  // Update Status based on economic-core rules
  if (state.snapshot.cash_minor <= 0) {
    state.snapshot.status = 'Bankrupt';
  } else if (state.snapshot.runway_days <= 14) {
    state.snapshot.status = 'Liquidation';
  } else if (state.snapshot.runway_days <= 30) {
    state.snapshot.status = 'Emergency';
  } else if (state.snapshot.runway_days <= 60) {
    state.snapshot.status = 'Distress';
  } else if (state.snapshot.runway_days <= 90 || state.snapshot.expenses_minor > state.snapshot.revenue_minor * 1.5) {
    state.snapshot.status = 'Warning';
  } else if (state.snapshot.revenue_minor > state.snapshot.expenses_minor && state.snapshot.audience_growth_bps > 100) {
    state.snapshot.status = 'Growth';
  } else {
    state.snapshot.status = 'Active';
  }
}

// Evaluate Proposal through Governor Policy Engine (Mirrors crates/agent-runtime/governor.rs)
function evaluateGovernor(proposal: Proposal, snapshot: CompanySnapshot): { decision: GovernorDecision; reason: string } {
  if (proposal.agent === 'Governor') {
    return { decision: 'Reject', reason: 'Governor cannot govern its own authority' };
  }

  // Distress policy blocks new discretionary spend
  const isDistressed = ['Distress', 'Emergency', 'Liquidation', 'Bankrupt'].includes(snapshot.status);
  if (isDistressed && proposal.cost_minor > 0) {
    return {
      decision: 'Reject',
      reason: `Distress policy (${snapshot.status}) forbids discretionary spend ($${(proposal.cost_minor / 100).toFixed(2)})`,
    };
  }

  // Bankruptcy blocks execution except reports & cost cuts
  if (['Bankrupt', 'Liquidation'].includes(snapshot.status)) {
    if (!['ProduceReport', 'EscalateIncident', 'ReduceBudget'].includes(proposal.action)) {
      return { decision: 'Reject', reason: 'Company state blocks non-essential execution during liquidation' };
    }
  }

  if (proposal.cost_minor < 0 || proposal.expected_revenue_minor < 0) {
    return { decision: 'Reject', reason: 'Negative economic values are strictly invalid in double-entry' };
  }

  if (proposal.cost_minor > snapshot.budget_remaining_minor) {
    return { decision: 'Reject', reason: `Proposal cost ($${(proposal.cost_minor / 100).toFixed(2)}) exceeds remaining budget ($${(snapshot.budget_remaining_minor / 100).toFixed(2)})` };
  }

  if (proposal.cost_minor > snapshot.cash_minor) {
    return { decision: 'Reject', reason: `Proposal cost ($${(proposal.cost_minor / 100).toFixed(2)}) exceeds available treasury cash ($${(snapshot.cash_minor / 100).toFixed(2)})` };
  }

  if (proposal.confidence_bps < 3000) {
    return { decision: 'RequestRevision', reason: `Confidence (${(proposal.confidence_bps / 100).toFixed(1)}%) is below mandatory threshold (30.0%)` };
  }

  // Materiality test: Cost > $1000 or High/Critical risk or Irreversible
  const isMaterial = proposal.cost_minor > 100000 || ['High', 'Critical'].includes(proposal.risk) || !proposal.reversible;
  if (isMaterial) {
    return {
      decision: 'EscalateToHuman',
      reason: `Material action detected (Cost: $${(proposal.cost_minor / 100).toFixed(2)}, Risk: ${proposal.risk}, Reversible: ${proposal.reversible}). Requires Operator authorization.`,
    };
  }

  return { decision: 'Approve', reason: 'Proposal satisfies constitutional economic limits, runway bounds, and budget limits.' };
}

// Execute an approved proposal
function executeProposal(proposal: Proposal): ExecutionReceipt {
  let outcome = '';
  const now = new Date().toISOString();
  const txId = `tx-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 6)}`;

  if (proposal.cost_minor > 0) {
    state.snapshot.cash_minor = Math.max(0, state.snapshot.cash_minor - proposal.cost_minor);
    state.snapshot.budget_remaining_minor = Math.max(0, state.snapshot.budget_remaining_minor - proposal.cost_minor);

    // Book double-entry ledger entry
    let debit = 'Operating Expenses';
    if (proposal.action === 'AllocateExperimentBudget' || proposal.action === 'CreateExperiment') {
      debit = 'R&D / Growth Experiments';
      state.snapshot.experiment_budget_minor = Math.max(0, state.snapshot.experiment_budget_minor - proposal.cost_minor);
    } else if (proposal.action === 'ProposeHire') {
      debit = 'Payroll & Talent Acquisition';
    }

    state.ledger.unshift({
      id: txId,
      timestamp: now,
      description: `[Cycle ${state.snapshot.cycle_count}] ${proposal.agent}: ${proposal.objective}`,
      debitAccount: debit,
      creditAccount: 'Cash & Cash Equivalents',
      amount_minor: proposal.cost_minor,
      cycle: state.snapshot.cycle_count,
      proposalId: proposal.id,
    });
  }

  // Effect on metrics
  switch (proposal.action) {
    case 'AllocateExperimentBudget':
      state.snapshot.experiment_budget_minor += proposal.cost_minor;
      outcome = `Allocated $${(proposal.cost_minor / 100).toFixed(2)} to experimental fund pool.`;
      break;
    case 'CreateExperiment':
      state.activeExperiments.unshift({
        id: `exp-${Date.now().toString(36)}`,
        name: proposal.objective,
        budget_minor: proposal.cost_minor,
        startCycle: state.snapshot.cycle_count,
        status: 'Active',
        roi_bps: Math.floor(Math.random() * 800 + 400),
      });
      state.snapshot.conversion_bps += Math.floor(Math.random() * 20 + 5);
      outcome = `Created and deployed live experiment '${proposal.objective}'. Projected conversion boost: +${(Math.random() * 0.2 + 0.05).toFixed(2)}%.`;
      break;
    case 'ReduceBudget':
      const reduction = Math.min(state.snapshot.expenses_minor, 45000);
      state.snapshot.expenses_minor = Math.max(100000, state.snapshot.expenses_minor - reduction);
      outcome = `Executed cost containment: reduced monthly burn by $${(reduction / 100).toFixed(2)}.`;
      break;
    case 'RebalanceOperations':
      state.snapshot.backlog = Math.max(2, state.snapshot.backlog - 4);
      state.snapshot.capacity = Math.min(40, state.snapshot.capacity + 2);
      outcome = `Reallocated worker queues. Reduced task backlog from to ${state.snapshot.backlog} items.`;
      break;
    case 'ResearchOpportunity':
      state.snapshot.audience_growth_bps += 30;
      outcome = `Market intelligence synthesized: Identified 3 untapped affiliate creator angles in consumer tech.`;
      break;
    case 'ProposeHire':
      state.snapshot.hiring_need = Math.max(0, state.snapshot.hiring_need - 1);
      state.snapshot.capacity += 6;
      state.snapshot.expenses_minor += 220000;
      state.employees.push({
        id: `emp-${Date.now().toString(36)}`,
        role: 'Autonomous Agent Engineer / Content Specialist',
        name: 'Jordan K.',
        salary_minor: 220000,
        hiredAtCycle: state.snapshot.cycle_count,
      });
      outcome = `Successfully onboarded talent candidate Jordan K. Expanded company execution capacity by +6 units.`;
      break;
    case 'PublishContent':
      const revBoost = Math.floor(Math.random() * 50000 + 15000);
      state.snapshot.revenue_minor += revBoost;
      state.snapshot.content_revenue_minor += revBoost;
      state.ledger.unshift({
        id: `tx-rev-${Date.now().toString(36)}`,
        timestamp: now,
        description: `Content Publishing Yield: ${proposal.objective}`,
        debitAccount: 'Cash & Cash Equivalents',
        creditAccount: 'Media & Affiliate Revenue',
        amount_minor: revBoost,
        cycle: state.snapshot.cycle_count,
        proposalId: proposal.id,
      });
      outcome = `Published high-intent multi-platform video series. Incurred immediate revenue delta: +$${(revBoost / 100).toFixed(2)}.`;
      break;
    default:
      outcome = `Action ${proposal.action} completed and archived in immutable audit journal.`;
  }

  recalculateCompanyHealth();

  return {
    id: `rcpt-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 6)}`,
    proposalId: proposal.id,
    agent: proposal.agent,
    action: proposal.action,
    status: 'Completed',
    outcome,
    cost_minor: proposal.cost_minor,
    timestamp: now,
  };
}

// Generate LLM reasoning using Gemini 3.8 Flash or deterministic fallback
async function generateAgentProposalWithAI(role: AgentRole, snapshot: CompanySnapshot): Promise<Proposal> {
  const prompt = `You are the ${role} agent of an Autonomous Media & AI Company Operating System.
Current Financial Snapshot:
- Status: ${snapshot.status}
- Cash: $${(snapshot.cash_minor / 100).toFixed(2)}
- Monthly Revenue: $${(snapshot.revenue_minor / 100).toFixed(2)}
- Monthly Expenses: $${(snapshot.expenses_minor / 100).toFixed(2)}
- Runway: ${snapshot.runway_days} days
- Backlog: ${snapshot.backlog} tasks | Capacity: ${snapshot.capacity} tasks
- Conversion: ${(snapshot.conversion_bps / 100).toFixed(2)}% | Audience Growth: ${(snapshot.audience_growth_bps / 100).toFixed(2)}%
- Cycle: #${snapshot.cycle_count}

Role Responsibilities:
- CEO: Long-term strategy, market positioning, high-level capital allocation.
- CFO: Solvency, strict double-entry ledger health, runway protection, cost cutting.
- COO: Workflow efficiency, queue throughput, bottleneck resolution, capacity balancing.
- Growth: Audience acquisition, viral loops, conversion rate optimization, affiliate traffic.
- Content: Media production, creator engagement, high-yield product hooks, video pipeline.
- Recruiter: Talent acquisition economics, hiring vs contractor trade-offs.
- Analyst: Objective decision support, variance metrics, root-cause diagnostics.
- Experiment: Bounded A/B hypotheses, fast iteration, high-upside tests.

Return a STRICT JSON response only (no markdown code blocks, just raw JSON) matching this schema:
{
  "objective": "A specific 1-sentence goal",
  "action": "One of: AllocateExperimentBudget, ReduceBudget, RebalanceOperations, ResearchOpportunity, CreateExperiment, ProposeHire, ProduceReport, EscalateIncident, PublishContent, LaunchCampaign",
  "cost_minor": integer cost in cents (e.g. 5000 for $50.00),
  "expected_revenue_minor": integer expected revenue in cents,
  "risk": "Low" | "Medium" | "High" | "Critical",
  "confidence_bps": integer basis points between 3000 and 9900 (e.g. 8500 = 85%),
  "evidence": ["bullet point 1 with data", "bullet point 2 with data"],
  "rationale": "Clear logical defense for why this proposal is economically justified",
  "reversible": boolean
}`;

  if (ai) {
    try {
      const generatePromise = ai.models.generateContent({
        model: 'gemini-3.8-flash',
        contents: prompt,
        config: {
          responseMimeType: 'application/json',
          temperature: 0.4,
        },
      });

      // 4-second timeout guard to ensure snappy cycle execution
      const timeoutPromise = new Promise<null>((resolve) => setTimeout(() => resolve(null), 4000));
      const response = await Promise.race([generatePromise, timeoutPromise]);

      if (response && 'text' in response && response.text) {
        const text = response.text;
        const parsed = JSON.parse(text);
        return {
          id: `prop-${role.toLowerCase()}-${Date.now().toString(36)}`,
          agent: role,
          objective: parsed.objective || `Autonomous ${role} action for cycle ${snapshot.cycle_count}`,
          action: parsed.action || 'ProduceReport',
          cost_minor: typeof parsed.cost_minor === 'number' ? Math.max(0, parsed.cost_minor) : 0,
          expected_revenue_minor: typeof parsed.expected_revenue_minor === 'number' ? Math.max(0, parsed.expected_revenue_minor) : 0,
          risk: ['Low', 'Medium', 'High', 'Critical'].includes(parsed.risk) ? parsed.risk : 'Low',
          confidence_bps: typeof parsed.confidence_bps === 'number' ? Math.min(9900, Math.max(3000, parsed.confidence_bps)) : 7500,
          evidence: Array.isArray(parsed.evidence) ? parsed.evidence : [`Status: ${snapshot.status}`, `Runway: ${snapshot.runway_days} days`],
          rationale: parsed.rationale || `${role} economic reasoning under cycle ${snapshot.cycle_count}`,
          reversible: typeof parsed.reversible === 'boolean' ? parsed.reversible : true,
          timestamp: new Date().toISOString(),
        };
      }
    } catch (e) {
      console.warn(`Gemini generation for ${role} failed, using heuristic:`, e);
    }
  }

  // Deterministic fallback matching crates/agent-runtime/roles.rs
  return getDeterministicProposal(role, snapshot);
}

function getDeterministicProposal(role: AgentRole, snapshot: CompanySnapshot): Proposal {
  const now = new Date().toISOString();
  const id = `prop-${role.toLowerCase()}-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 5)}`;

  switch (role) {
    case 'CEO':
      if (['Distress', 'Emergency', 'Liquidation'].includes(snapshot.status)) {
        return {
          id,
          agent: 'CEO',
          objective: 'Preserve enterprise solvency and freeze non-essential R&D',
          action: 'ReduceBudget',
          cost_minor: 0,
          expected_revenue_minor: 0,
          risk: 'Low',
          confidence_bps: 8800,
          evidence: [`Runway is down to ${snapshot.runway_days} days`, `Status: ${snapshot.status}`],
          rationale: 'Fiduciary duty requires immediate capital preservation to avoid bankruptcy liquidation.',
          reversible: true,
          timestamp: now,
        };
      }
      return {
        id,
        agent: 'CEO',
        objective: 'Expand viral creator syndicate and allocate exploration capital',
        action: 'AllocateExperimentBudget',
        cost_minor: 50000, // $500
        expected_revenue_minor: 120000,
        risk: 'Low',
        confidence_bps: 8200,
        evidence: [`Audience growth at +${(snapshot.audience_growth_bps / 100).toFixed(2)}%`, `Treasury cash at $${(snapshot.cash_minor / 100).toFixed(2)}`],
        rationale: 'Free cash flow supports measured expansion into trending vertical product categories.',
        reversible: true,
        timestamp: now,
      };

    case 'CFO':
      if (snapshot.runway_days <= 30 || snapshot.expenses_minor > snapshot.revenue_minor) {
        return {
          id,
          agent: 'CFO',
          objective: 'Cut cloud GPU inference waste and enforce cash flow austerity',
          action: 'ReduceBudget',
          cost_minor: 0,
          expected_revenue_minor: 0,
          risk: 'Low',
          confidence_bps: 9400,
          evidence: [`Burn rate: $${(snapshot.expenses_minor / 100).toFixed(2)}/mo`, `Net burn requires immediate budget clamping`],
          rationale: 'Strict unit economics require immediate reduction in recurring fixed liabilities.',
          reversible: true,
          timestamp: now,
        };
      }
      return {
        id,
        agent: 'CFO',
        objective: 'Audit double-entry ledger balance and calculate trailing unit margins',
        action: 'ProduceReport',
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: 'Low',
        confidence_bps: 9500,
        evidence: [`Current cash: $${(snapshot.cash_minor / 100).toFixed(2)}`, `Gross margin positive at 34.7%`],
        rationale: 'Factual verification of reconciliation statements prior to quarterly dividend declaration.',
        reversible: true,
        timestamp: now,
      };

    case 'COO':
      if (snapshot.backlog > snapshot.capacity) {
        return {
          id,
          agent: 'COO',
          objective: 'Rebalance parallel task queues and clear media rendering bottleneck',
          action: 'RebalanceOperations',
          cost_minor: 0,
          expected_revenue_minor: 0,
          risk: 'Low',
          confidence_bps: 8600,
          evidence: [`Backlog (${snapshot.backlog}) exceeds operational capacity (${snapshot.capacity})`],
          rationale: 'Operational queues are lagging behind creator intake; priority tasks must be re-routed.',
          reversible: true,
          timestamp: now,
        };
      }
      return {
        id,
        agent: 'COO',
        objective: 'Conduct operational capacity audit across distributed worker pods',
        action: 'ProduceReport',
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: 'Low',
        confidence_bps: 8900,
        evidence: [`System capacity headroom: ${(snapshot.capacity - snapshot.backlog)} units available`],
        rationale: 'Sufficient throughput headroom exists to ingest next batch of creator affiliate campaigns.',
        reversible: true,
        timestamp: now,
      };

    case 'Growth':
      return {
        id,
        agent: 'Growth',
        objective: 'Launch dynamic TikTok Shop & Amazon affiliate comparison widget test',
        action: 'CreateExperiment',
        cost_minor: 25000, // $250
        expected_revenue_minor: 75000,
        risk: 'Medium',
        confidence_bps: 7800,
        evidence: [`Conversion benchmark: ${(snapshot.conversion_bps / 100).toFixed(2)}%`, `Projected EPC: $0.42`],
        rationale: 'Hypothesis: Embedded interactive spec tables will boost buyer intent click-through by +15%.',
        reversible: true,
        timestamp: now,
      };

    case 'Content':
      return {
        id,
        agent: 'Content',
        objective: 'Deploy automated 5-part AI video series on trending smart home ergonomics',
        action: 'PublishContent',
        cost_minor: 15000, // $150
        expected_revenue_minor: 45000,
        risk: 'Low',
        confidence_bps: 8300,
        evidence: [`Content revenue: $${(snapshot.content_revenue_minor / 100).toFixed(2)}`, `Audience retention 68%`],
        rationale: 'Organic high-yield video scripts generated with multi-affiliate tracking links.',
        reversible: true,
        timestamp: now,
      };

    case 'Recruiter':
      if (snapshot.hiring_need > 0 && snapshot.revenue_minor > snapshot.expenses_minor && snapshot.runway_days > 90) {
        return {
          id,
          agent: 'Recruiter',
          objective: 'Recruit specialized Autonomous Media Automation & Video Prompt Engineer',
          action: 'ProposeHire',
          cost_minor: 220000, // $2,200 salary impact
          expected_revenue_minor: 600000,
          risk: 'Medium',
          confidence_bps: 7400,
          evidence: [`Hiring need index: ${snapshot.hiring_need}`, `Profitable run-rate covers candidate burn`],
          rationale: 'Candidate will unlock 3x media production throughput with positive net marginal profit.',
          reversible: false,
          timestamp: now,
        };
      }
      return {
        id,
        agent: 'Recruiter',
        objective: 'Screen contractor benchmarks and talent market compensation rates',
        action: 'ProduceReport',
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: 'Low',
        confidence_bps: 9100,
        evidence: [`Current Headcount: ${state.employees.length} team members`],
        rationale: 'Hiring freeze in effect or talent queue currently fulfilled. Maintaining market intel.',
        reversible: true,
        timestamp: now,
      };

    case 'Analyst':
      return {
        id,
        agent: 'Analyst',
        objective: 'Synthesize verified decision support: attribution decay and LTV/CAC ratio',
        action: 'ProduceReport',
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: 'Low',
        confidence_bps: 9600,
        evidence: [`LTV/CAC ratio: 3.8x`, `Attribution window: 30 days click-through`],
        rationale: 'Factual statistical verification prevents phantom attribution errors in affiliate accounting.',
        reversible: true,
        timestamp: now,
      };

    case 'Experiment':
      return {
        id,
        agent: 'Experiment',
        objective: 'Rapid A/B testing on AI thumbnail color psychology and hook timing',
        action: 'CreateExperiment',
        cost_minor: 10000, // $100
        expected_revenue_minor: 35000,
        risk: 'Low',
        confidence_bps: 7600,
        evidence: [`Available experiment budget: $${(snapshot.experiment_budget_minor / 100).toFixed(2)}`],
        rationale: 'Iterative micro-testing isolated from core brand capital.',
        reversible: true,
        timestamp: now,
      };

    default:
      return {
        id,
        agent: 'Analyst',
        objective: 'System status report',
        action: 'ProduceReport',
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: 'Low',
        confidence_bps: 8000,
        evidence: [],
        rationale: 'Default monitoring cycle',
        reversible: true,
        timestamp: now,
      };
  }
}

function getCycleTrendHistory() {
  const currentCycle = state.snapshot.cycle_count;
  const history = [];
  const startCycle = Math.max(1, currentCycle - 9);

  for (let c = startCycle; c <= currentCycle; c++) {
    const recorded = state.cycles.find((item: { cycleNumber: number }) => item.cycleNumber === c);
    if (recorded) {
      history.push({
        cycle: `Kỳ ${c}`,
        cycleNum: c,
        cash: Math.round(recorded.snapshotAfter.cash_minor / 100),
        revenue: Math.round(recorded.snapshotAfter.revenue_minor / 100),
        expenses: Math.round(recorded.snapshotAfter.expenses_minor / 100),
        netCashFlow: Math.round((recorded.snapshotAfter.revenue_minor - recorded.snapshotAfter.expenses_minor) / 100),
      });
    } else {
      const deltaFromCurrent = currentCycle - c;
      const factor = 1 - deltaFromCurrent * 0.05;
      const cashEstimate = Math.max(1000, Math.round((state.snapshot.cash_minor / 100) * (0.8 + factor * 0.2) - deltaFromCurrent * 250));
      const revEstimate = Math.max(500, Math.round((state.snapshot.revenue_minor / 100) * (0.6 + (c / currentCycle) * 0.4)));
      const expEstimate = Math.max(400, Math.round((state.snapshot.expenses_minor / 100) * (0.75 + (c / currentCycle) * 0.25)));
      history.push({
        cycle: `Kỳ ${c}`,
        cycleNum: c,
        cash: cashEstimate,
        revenue: revEstimate,
        expenses: expEstimate,
        netCashFlow: revEstimate - expEstimate,
      });
    }
  }
  return history;
}

// Department budget distribution over last 10 business cycles
function getDepartmentBudgetHistory() {
  const currentCycle = state.snapshot.cycle_count;
  const history = [];
  const startCycle = Math.max(1, currentCycle - 9);

  for (let c = startCycle; c <= currentCycle; c++) {
    const cycleFactor = c / Math.max(1, currentCycle);
    const baseExpenses = Math.round(state.snapshot.expenses_minor / 100);
    const cycleExpenses = Math.round(baseExpenses * (0.8 + cycleFactor * 0.2));

    const leadership = Math.round(cycleExpenses * 0.18);
    const growth = Math.round(cycleExpenses * 0.44);
    const ops = Math.round(cycleExpenses * 0.24);
    const techAndMedia = Math.max(0, cycleExpenses - leadership - growth - ops);

    history.push({
      cycle: `Kỳ ${c}`,
      cycleNum: c,
      leadership,
      growth,
      ops,
      techAndMedia,
      total: cycleExpenses,
    });
  }
  return history;
}

// Auto-Audit routine that generates a summary report every 10 business cycles
function generateAutoAuditReport(milestoneCycle: number) {
  const plannedRevenueMinor = 800000; // $8,000 / month baseline budget
  const plannedExpensesMinor = 600000; // $6,000 / month baseline expenses
  const actualRevenueMinor = state.snapshot.revenue_minor;
  const actualExpensesMinor = state.snapshot.expenses_minor;
  const varianceRevenueMinor = actualRevenueMinor - plannedRevenueMinor;
  const variancePercent = Math.round((varianceRevenueMinor / plannedRevenueMinor) * 1000) / 10;

  let verdict: 'ExceededTarget' | 'OnTrack' | 'UnderTarget' = 'OnTrack';
  let summary = '';
  let governorNote = '';

  if (variancePercent >= 10) {
    verdict = 'ExceededTarget';
    summary = `Kỳ kiểm toán #${milestoneCycle}: Doanh thu thực tế ($${(actualRevenueMinor / 100).toLocaleString()}/th) vượt kế hoạch ngân sách ($${(plannedRevenueMinor / 100).toLocaleString()}/th) thêm +${variancePercent}%. Dòng tiền ròng duy trì thặng dư xuất sắc.`;
    governorNote = 'Hiến pháp: Đạt chuẩn tăng trưởng bền vững. Ủy quyền tiếp tục mở rộng chuỗi tự động hóa affiliate.';
  } else if (variancePercent >= -10) {
    verdict = 'OnTrack';
    summary = `Kỳ kiểm toán #${milestoneCycle}: Doanh thu thực tế ($${(actualRevenueMinor / 100).toLocaleString()}/th) bám sát kế hoạch ngân sách ($${(plannedRevenueMinor / 100).toLocaleString()}/th) với độ lệch ${variancePercent}%.`;
    governorNote = 'Hiến pháp: Nằm trong biên độ dung sai an toàn. Duy trì chính sách kiểm soát chi phí hiện hành.';
  } else {
    verdict = 'UnderTarget';
    summary = `Kỳ kiểm toán #${milestoneCycle}: Doanh thu thực tế thấp hơn kế hoạch ngân sách ${Math.abs(variancePercent)}%. Cần kích hoạt quy trình thắt lưng buộc bụng.`;
    governorNote = 'Hiến pháp: Cảnh báo thâm hụt. Yêu cầu CFO kích hoạt điều khoản cắt giảm ngân sách thử nghiệm.';
  }

  const report = {
    id: `audit-cycle-${milestoneCycle}-${Date.now().toString(36)}`,
    cycleMilestone: milestoneCycle,
    timestamp: new Date().toISOString(),
    plannedRevenueMinor,
    actualRevenueMinor,
    varianceRevenueMinor,
    variancePercent,
    plannedExpensesMinor,
    actualExpensesMinor,
    cashReserveMinor: state.snapshot.cash_minor,
    verdict,
    summary,
    governorNote,
  };

  state.auditReports.unshift(report);
  return report;
}

// P&L Statement Calculator
function calculatePnL(): any {
  const totalRevenueMinor = state.snapshot.revenue_minor;
  const cogsMinor = Math.round(totalRevenueMinor * 0.18); // cloud inference & asset licensing
  const grossProfitMinor = totalRevenueMinor - cogsMinor;
  const grossMarginPercent = totalRevenueMinor > 0 ? Math.round((grossProfitMinor / totalRevenueMinor) * 1000) / 10 : 0;
  const operatingExpensesMinor = state.snapshot.expenses_minor;
  const netIncomeMinor = grossProfitMinor - operatingExpensesMinor;
  const dividendsDeclaredMinor = Math.max(0, Math.round(netIncomeMinor * 0.2));
  const retainedEarningsMinor = Math.max(0, netIncomeMinor - dividendsDeclaredMinor);

  return {
    totalRevenueMinor,
    grossMarginPercent,
    operatingExpensesMinor,
    netIncomeMinor,
    monthlyRunRateMinor: totalRevenueMinor * 12,
    dividendsDeclaredMinor,
    retainedEarningsMinor,
  };
}

// REST Endpoints
app.get('/api/state', (req, res) => {
  trimMemoryState();
  res.json({
    dataMode: SIMULATED_DATA_MODE,
    evidenceMode: 'synthetic_fixture',
    simulatedMutationsEnabled: simulatedMutationsEnabled(),
    warning: 'All monetary, KPI, audit, workforce and pipeline values from this Node/Vite backend are synthetic simulation state, not company actuals.',
    snapshot: state.snapshot,
    ledger: state.ledger.slice(0, 50),
    receipts: state.receipts.slice(0, 50),
    employees: state.employees,
    customAgents: state.customAgents,
    candidatePool: state.candidatePool,
    officeActivities: state.officeActivities.slice(0, 40),
    creativeProductions: state.creativeProductions.slice(0, 20),
    autonomousSettings: state.autonomousSettings,
    pnl: calculatePnL(),
    activeExperiments: state.activeExperiments,
    recentCycles: state.cycles.slice(-5),
    cycleHistory: getCycleTrendHistory(),
    departmentBudgets: getDepartmentBudgetHistory(),
    communicationStream: state.communicationStream.slice(0, 40),
    systemAlerts: state.systemAlerts,
    auditReports: state.auditReports,
    clientContracts: state.clientContracts,
    channels: state.channels,
    hasGeminiKey: Boolean(apiKey && apiKey !== 'MY_GEMINI_API_KEY'),
  });
});

app.get('/api/channels', (req, res) => {
  res.json({
    channels: state.channels,
    totalChannels: state.channels.length,
    activeLiveStreams: state.channels.filter(c => c.status === 'LiveNow').length,
    totalFollowers: state.channels.reduce((acc, c) => acc + c.followers, 0),
    total30dViews: state.channels.reduce((acc, c) => acc + c.views30d, 0),
    totalMonthlyDonationsMinor: state.channels.reduce((acc, c) => acc + c.monthlyDonationMinor, 0),
    totalChannelValuationMinor: state.channels.reduce((acc, c) => acc + c.estimatedValuationMinor, 0),
  });
});

app.post('/api/channels/stream/start', (req, res) => {
  const { channelId, title, topic, streamType = 'Storytelling & Mystery', hostName = 'Mia Thorne AI (VTuber)' } = req.body;
  const channel = state.channels.find(c => c.id === channelId) || state.channels[0];
  
  channel.status = 'LiveNow';
  channel.totalStreamsRun += 1;
  channel.activeStreamSession = {
    id: `stream-${Date.now().toString(36)}`,
    channelId: channel.id,
    channelName: channel.name,
    platform: (channel.platform === 'YouTube' ? 'YouTube Live' : channel.platform === 'Twitch' ? 'Twitch' : channel.platform === 'Facebook Reels' ? 'Facebook Gaming' : 'TikTok Live') as any,
    streamType: streamType as any,
    hostAgentName: hostName,
    hostAgentAvatar: streamType === 'Gaming & Reaction' ? '🎮' : '🎙️',
    personaStyle: streamType === 'Gaming & Reaction' ? 'Cyberpunk Pro Gamer' : 'VTuber 3D Persona',
    virtualSet: streamType === 'Gaming & Reaction' ? 'Đấu Trường Neon Gaming' : 'Phòng Thu Ánh Trăng 3D',
    aiDecisionRationale: 'Quyết định tự động bởi Đạo Diễn AI Kenji Sato dựa trên dữ liệu trending thời gian thực.',
    title: title || `🔴 LIVE 24/7: ${topic || channel.niche} - Trò Chuyện & Tương Tác Cùng Fan`,
    currentGameOrTopic: topic || channel.niche,
    streamStatus: 'Live',
    viewersCount: Math.floor(Math.random() * 1200 + 2200),
    peakViewers: Math.floor(Math.random() * 1500 + 3800),
    donationReceivedMinor: 15000, // $150.00 initial donations
    liveDurationSec: 120,
    comments: [
      { id: 'c-new-1', userName: 'FanCung_01', avatar: '👋', message: 'Chào host nhé! Hôm nay stream nội dung gì vậy?', timestamp: 'Vừa xong', aiHostReply: `Chào bạn nhé! Hôm nay tụi mình cùng ${topic || 'trò chuyện, kể chuyện và chơi minigame chill'} cùng nhau nhé!` },
    ],
    recentDonations: [
      { id: `don-${Date.now().toString(36)}`, donor: 'FanCung_01', avatar: '☕', amountMinor: 500, giftName: 'Cà Phê Năng Lượng', giftIcon: '☕', message: 'Tặng host ly cà phê lấy sức stream thâu đêm!', timestamp: 'Vừa xong' },
    ],
    startedAt: new Date().toISOString(),
  };

  const streamSession = channel.activeStreamSession;
  logOfficeActivity({
    agentId: streamType === 'Gaming & Reaction' ? 'agent-gamer' : 'agent-livestream',
    agentName: hostName,
    agentRole: streamType === 'Gaming & Reaction' ? 'Streamer Gaming' : 'VTuber Kể Chuyện',
    department: 'Growth',
    actionType: 'CreateContent',
    title: `Lên sóng trực tiếp 24/7 trên kênh ${channel.name}`,
    detail: `AI Virtual Streamer đã bắt đầu phiên live: "${streamSession?.title}". Đang thu hút ${streamSession?.viewersCount} người xem trực tiếp.`,
    badgeColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
  });

  res.json({ success: true, channel, stream: channel.activeStreamSession });
});

app.post('/api/channels/stream/donate', (req, res) => {
  const { channelId, donorName = 'KhánGiả_ẨnDanh', amountMinor = 500, giftName = 'Super Star', giftIcon = '🌟', message = 'Ủng hộ kênh AI phát triển!' } = req.body;
  const channel = state.channels.find(c => c.id === channelId) || state.channels[0];
  if (!channel.activeStreamSession) {
    return res.status(400).json({ success: false, reason: 'Kênh chưa có phiên live nào đang hoạt động' });
  }

  const donation: LiveStreamDonation = {
    id: `don-${Date.now().toString(36)}`,
    donor: donorName,
    avatar: giftIcon,
    amountMinor,
    giftName,
    giftIcon,
    message,
    timestamp: 'Vừa xong',
  };

  channel.activeStreamSession.donationReceivedMinor += amountMinor;
  channel.activeStreamSession.recentDonations.unshift(donation);
  if (channel.activeStreamSession.recentDonations.length > 20) {
    channel.activeStreamSession.recentDonations.pop();
  }

  // Add donation comment to chat stream
  channel.activeStreamSession.comments.unshift({
    id: `c-don-${Date.now().toString(36)}`,
    userName: donorName,
    avatar: giftIcon,
    message: `[DONATE ${giftIcon} ${(amountMinor / 100).toLocaleString('en-US', { style: 'currency', currency: 'USD' })}]: ${message}`,
    timestamp: 'Vừa xong',
    isDonation: true,
    donationAmountMinor: amountMinor,
    giftIcon,
    aiHostReply: `Cảm ơn ${donorName} rất nhiều đã gửi tặng ${giftName}! Tình cảm của các bạn là động lực để kênh tiếp tục mang lại niềm vui mỗi ngày! ❤️`,
  });

  state.snapshot.cash_minor += amountMinor;
  state.snapshot.revenue_minor += amountMinor;
  state.snapshot.content_revenue_minor += amountMinor;
  channel.monthlyDonationMinor += amountMinor;

  res.json({ success: true, donation, stream: channel.activeStreamSession, channel });
});

const AUTONOMOUS_LIVE_THEMES = [
  {
    topic: 'Kỳ Án Hồ Sương Mù & Đọc Tâm Sự Giấu Tên',
    title: '🔴 LIVE 24/7: Kể Chuyện Kỳ Án Hồ Sương Mù & Đọc Tâm Sự Giấu Tên Cùng 3,400 Bạn Đêm Khuya',
    host: 'Mia Thorne AI (VTuber)',
    avatar: '🎙️',
    streamType: 'Storytelling & Mystery' as const,
    personaStyle: 'VTuber 3D Goth-Lofi Anime',
    virtualSet: 'Phòng Thu Ánh Trăng 3D & Lofi Rainy Window',
    rationale: 'Kenji Sato AI quét thấy hashtag #TruyenKiemDiem và #HealingTalks đang viral top 1 đêm khuya. Tự động chuyển đổi sang bối cảnh Ánh Trăng 3D để giữ chân người xem trung bình > 28 phút.',
  },
  {
    topic: 'Chơi Thử Game Kinh Dị Outlast II & React Meme Fan Gửi',
    title: '🔴 LIVE 24/7: Ren Kuro AI Chơi Outlast II Thâu Đêm - Nhịp Tim 140bpm + Đọc Donate Hài Hước',
    host: 'Ren Kuro AI (Gamer)',
    avatar: '🎮',
    streamType: 'Gaming & Reaction' as const,
    personaStyle: 'Cyberpunk Pro Gamer 3D Avatar',
    virtualSet: 'Đấu Trường Neon Gaming Arena & RGB Audio Lights',
    rationale: 'Hệ thống radar phát hiện lượng người xem game kinh dị và meme reactions tăng đột biến +320% vào khung giờ này. Tự động hoán đổi Host Ren Kuro AI và bối cảnh Neon Gaming.',
  },
  {
    topic: 'Nhạc Lofi Piano Chữa Lành, Học Bài & Q&A Tự Động',
    title: '🔴 LIVE 24/7: Lofi Chill Beats & Không Gian Học Tập Cùng AI Host Luna - Q&A Trò Chuyện Tâm Sự',
    host: 'Luna AI (Lofi Host)',
    avatar: '🎧',
    streamType: 'Lofi Chill & Minigames' as const,
    personaStyle: 'Chibi Anime Cozy Lofi Persona',
    virtualSet: 'Quán Cà Phê Mưa Ấm Áp Lofi Cafe',
    rationale: 'Thuật toán tối ưu hóa tệp sinh viên & người làm việc đêm khuya cần nhạc không lời và tâm sự nhẹ nhàng, kéo lượng tương tác donate tăng đều đặn.',
  },
  {
    topic: 'Giải Mã Bí Ẩn Tam Giác Bermuda & Khoa Học Viễn Tưởng',
    title: '🔴 LIVE 24/7: Khám Phá Bí Ẩn Đại Dương & Vũ Trụ - Thảo Luận Khoa Học Cùng Host Mia AI',
    host: 'Mia Thorne AI (VTuber)',
    avatar: '🎙️',
    streamType: 'Storytelling & Mystery' as const,
    personaStyle: 'Sci-Fi Holo VTuber',
    virtualSet: 'Đài Thiên Văn Không Gian 3D Vũ Trụ',
    rationale: 'Chủ đề khoa học viễn tưởng và đại dương kích thích trí tò mò, thúc đẩy bình luận tranh luận tăng +210% trong live chat.',
  },
];

app.post('/api/channels/autonomous-switch', (req, res) => {
  const { channelId } = req.body;
  const channel = state.channels.find(c => c.id === channelId) || state.channels[0];
  if (!channel.activeStreamSession) {
    return res.status(400).json({ success: false, reason: 'Kênh chưa có phiên live nào đang hoạt động' });
  }

  // Pick random next theme from autonomous pool
  const currentTopic = channel.activeStreamSession.currentGameOrTopic;
  const otherThemes = AUTONOMOUS_LIVE_THEMES.filter(t => t.topic !== currentTopic);
  const nextTheme = otherThemes[Math.floor(Math.random() * otherThemes.length)] || AUTONOMOUS_LIVE_THEMES[0];

  channel.activeStreamSession.currentGameOrTopic = nextTheme.topic;
  channel.activeStreamSession.title = nextTheme.title;
  channel.activeStreamSession.hostAgentName = nextTheme.host;
  channel.activeStreamSession.hostAgentAvatar = nextTheme.avatar;
  channel.activeStreamSession.streamType = nextTheme.streamType;
  channel.activeStreamSession.personaStyle = nextTheme.personaStyle;
  channel.activeStreamSession.virtualSet = nextTheme.virtualSet;
  channel.activeStreamSession.aiDecisionRationale = nextTheme.rationale;
  channel.activeStreamSession.viewersCount = Math.floor(Math.random() * 1200 + 2600);

  logOfficeActivity({
    agentId: 'agent-streamops',
    agentName: 'Kenji Sato AI',
    agentRole: 'Đạo Diễn Live & Quản Trị Kênh',
    department: 'Ops',
    actionType: 'CreateContent',
    title: `AI Tự Động Chuyển Đổi Bối Cảnh & Chủ Đề: ${nextTheme.topic}`,
    detail: `Quyết định tự động: Chuyển sang ${nextTheme.virtualSet} với Host ${nextTheme.host}. Lý do: ${nextTheme.rationale}`,
    badgeColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
  });

  res.json({ success: true, theme: nextTheme, stream: channel.activeStreamSession, channel });
});

app.post('/api/channels/stream/change-topic', (req, res) => {
  const { channelId, topic, title } = req.body;
  const channel = state.channels.find(c => c.id === channelId) || state.channels[0];
  if (!channel.activeStreamSession) {
    return res.status(400).json({ success: false, reason: 'Kênh chưa có phiên live nào đang hoạt động' });
  }

  if (topic) channel.activeStreamSession.currentGameOrTopic = topic;
  if (title) channel.activeStreamSession.title = title;

  res.json({ success: true, stream: channel.activeStreamSession, channel });
});

app.post('/api/channels/stream/stop', (req, res) => {
  const { channelId } = req.body;
  const channel = state.channels.find(c => c.id === channelId) || state.channels[0];
  if (channel.activeStreamSession) {
    channel.activeStreamSession.streamStatus = 'Ended';
    channel.status = 'Active';
    channel.activeStreamSession = undefined;
  }
  res.json({ success: true, message: 'Đã hoàn tất phiên livestream và cập nhật số dư donate thành công!' });
});

app.post('/api/channels/list-sale', (req, res) => {
  const { channelId, saleStatus = 'Listed', customValuationMinor } = req.body;
  const channel = state.channels.find(c => c.id === channelId);
  if (!channel) return res.status(404).json({ success: false, reason: 'Không tìm thấy kênh' });

  channel.saleStatus = saleStatus;
  if (customValuationMinor) channel.estimatedValuationMinor = customValuationMinor;

  logOfficeActivity({
    agentId: 'agent-streamops',
    agentName: 'Kenji Sato AI',
    agentRole: 'Quản Trị Mạng Lưới Kênh',
    department: 'Ops',
    actionType: 'CreateContent',
    title: `Niêm yết chuyển nhượng kênh ${channel.name}`,
    detail: `Kênh ${channel.name} (${channel.followers.toLocaleString()} followers) đã được niêm yết chào bán với mức định giá $${(channel.estimatedValuationMinor / 100).toLocaleString()}.`,
    badgeColor: 'text-amber-400 bg-amber-500/10 border-amber-500/20',
  });

  res.json({ success: true, channel });
});

app.post('/api/channels/sell', (req, res) => {
  const { channelId, buyerName = 'Syndicate Media Fund Global' } = req.body;
  const channel = state.channels.find(c => c.id === channelId);
  if (!channel) return res.status(404).json({ success: false, reason: 'Không tìm thấy kênh' });

  const salePrice = channel.estimatedValuationMinor;
  channel.saleStatus = 'Sold';
  channel.status = 'Sold';

  // Inject sale proceeds directly into company cash & revenue ledger
  state.snapshot.cash_minor += salePrice;
  state.snapshot.revenue_minor += salePrice;

  state.ledger.unshift({
    id: `tx-sale-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `M&A Channel Liquidation: Bán nhượng quyền kênh ${channel.name} cho ${buyerName}`,
    debitAccount: 'Cash & Cash Equivalents',
    creditAccount: 'Capital Gain on Digital Asset Sales',
    amount_minor: salePrice,
    cycle: state.snapshot.cycle_count,
  });

  logOfficeActivity({
    agentId: 'agent-cfo',
    agentName: 'CFO AI',
    agentRole: 'Giám Đốc Tài Chính',
    department: 'Leadership',
    actionType: 'AuditLedger',
    title: `Thanh khoản tài sản số: Bán kênh ${channel.name} thu về +$${(salePrice / 100).toLocaleString()}`,
    detail: `Đối tác ${buyerName} đã hoàn tất thanh toán chuyển nhượng kênh. Toàn bộ số tiền đã được cộng trực tiếp vào Kho Bạc công ty.`,
    badgeColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
  });

  res.json({ success: true, salePrice, channel, cashUSD: (state.snapshot.cash_minor / 100).toLocaleString('en-US', { style: 'currency', currency: 'USD' }) });
});

app.get('/api/contracts', (req, res) => {
  res.json({
    contracts: state.clientContracts,
    totalContracts: state.clientContracts.length,
    activeCount: state.clientContracts.filter(c => c.status !== 'Delivered' && c.status !== 'Completed').length,
    completedCount: state.clientContracts.filter(c => c.status === 'Delivered' || c.status === 'Completed').length,
    totalContractValueMinor: state.clientContracts.reduce((acc, c) => acc + c.budgetMinor, 0),
  });
});

app.get('/api/contracts/:id', (req, res) => {
  const contract = state.clientContracts.find(c => c.id === req.params.id);
  if (!contract) {
    return res.status(404).json({ success: false, reason: 'Contract not found' });
  }
  res.json({ contract });
});

app.post('/api/contracts/order', async (req, res) => {
  const { 
    clientName = 'Khách Hàng Đối Tác', 
    clientEmail = 'partner@enterprise.ai', 
    title, 
    category = 'VideoMarketing', 
    requirements, 
    budgetMinor 
  } = req.body;

  if (!title || !requirements) {
    return res.status(400).json({ success: false, reason: 'Thiếu tiêu đề hoặc yêu cầu hợp đồng' });
  }

  // Determine standard pricing if not specified
  const standardPricing: Record<string, number> = {
    VideoMarketing: 45000, // $450
    Copywriting: 28000,    // $280
    MediaDesign: 32000,    // $320
    MarketIntelligence: 35000, // $350
    FullCampaign: 85000,   // $850
  };

  const finalBudgetMinor = budgetMinor || standardPricing[category] || 35000;
  const contractId = `ctr-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 5)}`;
  const contractNumber = `HD-2026-${Math.floor(Math.random() * 9000 + 1000)}`;
  const cycle = state.snapshot.cycle_count;

  // Autonomous Scoping & Deliverables Generation
  const scriptContent = `[0:00 - 0:03] HOOK: "Bí quyết tăng tỷ lệ chuyển đổi gấp 3 lần với sản phẩm ${title} mà các top seller không tiết lộ..."\n[0:03 - 0:18] PAIN POINT: Khách hàng thường đắn đo và rời đi vì thiếu giải pháp trực quan rõ ràng.\n[0:18 - 0:42] SOLUTION & USP: Giới thiệu ưu điểm vượt trội của ${title} với độ chính xác cao và trải nghiệm cao cấp.\n[0:42 - 0:60] CALL TO ACTION: Nhấn vào liên kết bên dưới để nhận ưu đãi đối tác đặc quyền ngay hôm nay!`;
  
  const visualPrompts = [
    `Cinematic 8k commercial macro hero shot of ${title}, soft studio box lighting, minimalist modern aesthetic, depth of field`,
    `Handheld POV lifestyle showcase of ${title} in sleek professional environment, 4k ultra-detailed`,
    `Infographic split comparison showing 300% performance efficiency for ${title}, clean dark UI layout`,
  ];

  const audioVoiceover = `Giọng đọc chuyên nghiệp, truyền cảm hứng và tự tin. Nhạc nền: Commercial Pop & Lo-Fi Tech (BPM 118), âm lượng chuẩn hóa -14.0 LUFS.`;

  const videoSpecs = {
    resolution: '1080x1920 (Vertical 9:16)',
    fps: '60 fps',
    duration: '60 giây',
    aspectRatio: '9:16',
  };

  const researchInsights = [
    `Ngách "${title}" đang có lưu lượng tìm kiếm tăng 42% trên TikTok Shop & Shopee.`,
    `Tỷ lệ chuyển đổi đơn hàng trung bình dự kiến đạt 3.8% với EPC $1.15/click.`,
    `Đã kiểm duyệt quy chuẩn FTC và bản quyền âm nhạc thương mại 100% an toàn.`,
  ];

  const newContract: ClientContract = {
    id: contractId,
    contractNumber,
    clientName,
    clientEmail,
    title,
    category,
    requirements,
    budgetMinor: finalBudgetMinor,
    createdAt: new Date().toISOString(),
    deadline: new Date(Date.now() + 86400000 * 3).toISOString(),
    status: 'Delivered',
    currentStage: 'Hoàn tất bàn giao thành phẩm & nghiệm thu',
    progressPercent: 100,
    assignedAgents: [
      { role: 'Growth Lead', name: 'Growth Lead', step: 'Nghiên cứu thị trường & Từ khóa chuyển đổi', completed: true },
      { role: 'Content Lead', name: 'Content Lead', step: 'Soạn kịch bản Viral Hook & Lời thoại', completed: true },
      { role: 'Media Worker', name: 'Media Worker', step: 'Tạo prompt ảnh 8K & Render video 60fps', completed: true },
      { role: 'Governor & CFO', name: 'Governor AI', step: 'Kiểm toán chất lượng & Đối soát', completed: true },
    ],
    deliverables: {
      summary: `Hợp đồng "${title}" đã được các phòng ban AI phối hợp sản xuất tự động và bàn giao thành công.`,
      scriptContent,
      audioVoiceover,
      visualPrompts,
      videoSpecs,
      researchInsights,
      deliveredAt: new Date().toISOString(),
      qualityScore: 99,
      downloadUrl: `/deliverables/${contractNumber}-deliverable.zip`,
    },
    invoice: {
      amountMinor: finalBudgetMinor,
      paidStatus: 'Paid',
      paidAt: new Date().toISOString(),
      transactionId: `TXN-${contractNumber}-PAID`,
    },
    rating: 5,
  };

  state.clientContracts.unshift(newContract);
  trimMemoryState();

  // Financial Accounting (Double-Entry Ledger)
  state.snapshot.cash_minor += finalBudgetMinor;
  state.snapshot.revenue_minor += finalBudgetMinor;
  state.snapshot.content_revenue_minor += finalBudgetMinor;

  state.ledger.unshift({
    id: `tx-contract-${contractNumber}`,
    timestamp: new Date().toISOString(),
    description: `Hợp đồng thuê khoán #${contractNumber}: ${title} (${clientName})`,
    debitAccount: 'Cash & Cash Equivalents',
    creditAccount: 'Client Contract Revenue',
    amount_minor: finalBudgetMinor,
    cycle,
  });

  // Increment Agent Metrics
  state.customAgents.forEach(agent => {
    agent.tasksCompleted += 1;
  });

  // Log Virtual Office Activity
  logOfficeActivity({
    agentId: 'agent-gov',
    agentName: 'Governor AI',
    agentRole: 'Hiến Pháp & Quản Trị',
    department: 'Leadership',
    actionType: 'CreateContent',
    title: `Tiếp nhận & Bàn giao hợp đồng #${contractNumber}`,
    detail: `Hợp đồng "${title}" trị giá $${(finalBudgetMinor / 100).toFixed(2)} từ khách hàng ${clientName} đã được hoàn tất và bàn giao tự động.`,
    impactMinor: finalBudgetMinor,
    badgeColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
  });

  // Autonomous Recruiter check: If contracts backlog is high, auto-hire candidates
  if (state.clientContracts.length % 3 === 0 && state.candidatePool.length > 0) {
    const candidateToHire = state.candidatePool.find(c => c.status === 'Available');
    if (candidateToHire && state.snapshot.cash_minor > 2000000) {
      candidateToHire.status = 'Hired';
      state.customAgents.push({
        id: `agent-auto-${Date.now().toString(36)}`,
        name: candidateToHire.name,
        role: candidateToHire.role,
        department: candidateToHire.department,
        description: candidateToHire.bio,
        salary_minor: candidateToHire.expectedSalaryMinor,
        tasksCompleted: 1,
        status: 'Active',
        hiredAtCycle: cycle,
        skillLevel: 3,
        taskMultiplier: 1.5,
        trainedSkills: candidateToHire.skills.map(s => s.name),
        trainingCount: 1,
      });
      logOfficeActivity({
        agentId: 'agent-recruiter',
        agentName: 'Recruiter AI',
        agentRole: 'Tuyển Dụng',
        department: 'Ops',
        actionType: 'RecruitTalent',
        title: `Tự động tuyển dụng ${candidateToHire.name}`,
        detail: `Do khối lượng hợp đồng khách hàng gia tăng, công ty đã tự động onboard chuyên gia ${candidateToHire.name} (${candidateToHire.role}).`,
        impactMinor: -candidateToHire.expectedSalaryMinor,
        badgeColor: 'text-blue-400 bg-blue-500/10 border-blue-500/20',
      });
    }
  }

  recalculateCompanyHealth();

  res.json({
    success: true,
    contract: newContract,
    message: `Hợp đồng #${contractNumber} đã được tiếp nhận và xử lý thành công!`,
  });
});

app.post('/api/contracts/:id/accept', (req, res) => {
  const { rating = 5, feedback = 'Nghiệm thu thành công, sản phẩm đạt chất lượng cao.' } = req.body;
  const contract = state.clientContracts.find(c => c.id === req.params.id);
  if (!contract) {
    return res.status(404).json({ success: false, reason: 'Contract not found' });
  }

  contract.status = 'Completed';
  contract.rating = rating;
  contract.clientFeedback = feedback;

  res.json({
    success: true,
    contract,
    message: 'Đã nghiệm thu hợp đồng thành công!',
  });
});

app.get('/api/office-activities', (req, res) => {
  res.json({ activities: state.officeActivities.slice(0, 50) });
});

app.get('/api/candidates', (req, res) => {
  res.json({ candidates: state.candidatePool });
});

// AI Candidate Interview Simulator
app.post('/api/candidates/interview', async (req, res) => {
  const { candidateId } = req.body;
  const candidate = state.candidatePool.find(c => c.id === candidateId);
  if (!candidate) {
    return res.status(404).json({ success: false, reason: 'Candidate not found' });
  }

  candidate.status = 'Interviewing';

  let evaluation = {
    technicalScore: 94,
    portfolioScore: 96,
    cultureScore: 95,
    governorVerdict: 'Approve Recommendation',
    summary: `${candidate.name} sở hữu kinh nghiệm thực chiến vượt trội (${candidate.yearsExperience} năm) với các chỉ số ROI chứng minh rõ ràng. Phù hợp hoàn hảo với vai trò ${candidate.role}.`,
    negotiatedSalaryMinor: candidate.expectedSalaryMinor,
  };

  if (ai) {
    try {
      const prompt = `You are the Recruiter AI & Governor AI of an Autonomous Media Corporation.
Candidate to interview:
- Name: ${candidate.name}
- Role: ${candidate.role}
- Experience: ${candidate.yearsExperience} years
- Bio: ${candidate.bio}
- Expected Salary: $${(candidate.expectedSalaryMinor / 100).toFixed(2)}/mo

Evaluate this candidate for senior expertise and calculate an interview score out of 100.
Return STRICT JSON:
{
  "technicalScore": 95,
  "portfolioScore": 94,
  "cultureScore": 96,
  "governorVerdict": "Approve Recommendation",
  "summary": "1-2 sentence assessment in Vietnamese",
  "negotiatedSalaryMinor": ${candidate.expectedSalaryMinor}
}`;
      const resp = await ai.models.generateContent({
        model: 'gemini-3.8-flash',
        contents: prompt,
        config: { responseMimeType: 'application/json', temperature: 0.4 },
      });
      const parsed = JSON.parse(resp.text || '{}');
      if (parsed.technicalScore) evaluation = parsed;
    } catch (e) {
      console.warn('AI interview evaluation fallback:', e);
    }
  }

  candidate.interviewNotes = evaluation.summary;

  logOfficeActivity({
    agentId: 'agent-recruiter',
    agentName: 'Recruiter AI',
    agentRole: 'Tuyển Dụng',
    department: 'Ops',
    actionType: 'RecruitTalent',
    title: `Phỏng vấn chuyên sâu 3 vòng ứng viên ${candidate.name} (${candidate.role})`,
    detail: `Điểm chuyên môn: ${evaluation.technicalScore}/100. Kết quả: ${evaluation.governorVerdict}. Ghi chú: ${evaluation.summary}`,
    badgeColor: 'text-cyan-400 bg-cyan-500/10 border-cyan-500/20',
  });

  res.json({ success: true, candidate, evaluation });
});

// Autonomous / Manual Candidate Hiring
app.post('/api/candidates/hire', (req, res) => {
  const { candidateId } = req.body;
  const candidate = state.candidatePool.find(c => c.id === candidateId);
  if (!candidate) {
    return res.status(404).json({ success: false, reason: 'Candidate not found' });
  }

  if (state.snapshot.runway_days < 45 && state.snapshot.cash_minor < 2000000) {
    return res.status(400).json({
      success: false,
      reason: `Governor Veto: Runway hiện tại (${state.snapshot.runway_days} ngày) dưới ngưỡng an toàn 45 ngày. Đóng băng tuyển dụng!`,
    });
  }

  candidate.status = 'Hired';

  // Add to customAgents
  const newAgent = {
    id: `agent-${candidate.id}`,
    name: candidate.name,
    role: candidate.role,
    department: candidate.department,
    description: candidate.bio,
    salary_minor: candidate.expectedSalaryMinor,
    tasksCompleted: 0,
    status: 'Active' as const,
    hiredAtCycle: state.snapshot.cycle_count,
    skillLevel: candidate.level === 'Principal Expert' ? 4 : candidate.level === 'Director' ? 5 : 3,
    taskMultiplier: candidate.level === 'Principal Expert' ? 1.75 : 1.5,
    trainedSkills: candidate.skills.map(s => s.name),
    trainingCount: 1,
  };

  state.customAgents.push(newAgent);

  state.employees.push({
    id: newAgent.id,
    name: newAgent.name,
    role: newAgent.role,
    salary_minor: newAgent.salary_minor,
    hiredAtCycle: state.snapshot.cycle_count,
  });

  state.snapshot.expenses_minor += newAgent.salary_minor;
  state.snapshot.capacity += 10;
  state.snapshot.backlog = Math.max(0, state.snapshot.backlog - 4);

  // Book Double-Entry Accounting for Onboarding
  state.ledger.unshift({
    id: `tx-hire-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `Tuyển Dụng Nhân Tài Cao Cấp: ${newAgent.name} (${newAgent.role})`,
    debitAccount: 'Chi Phí Phát Triển Đội Ngũ Chuyên Gia',
    creditAccount: 'Cash & Cash Equivalents',
    amount_minor: Math.min(newAgent.salary_minor, 35000),
    cycle: state.snapshot.cycle_count,
  });

  // Log Office Event
  logOfficeActivity({
    agentId: 'agent-gov',
    agentName: 'Governor AI',
    agentRole: 'Hiến Pháp & Quỹ Tiền',
    department: 'Leadership',
    actionType: 'RecruitTalent',
    title: `Chính thức gia nhập: ${candidate.name} giữ chức ${candidate.role} (${candidate.level})`,
    detail: `Tăng công suất xử lý toàn công ty thêm +10 slots. Dự phóng cải thiện ROI +${(candidate.roiProjectionBps / 100).toFixed(1)}%.`,
    impactMinor: -Math.min(newAgent.salary_minor, 35000),
    badgeColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
  });

  recalculateCompanyHealth();

  res.json({
    success: true,
    candidate,
    agent: newAgent,
    snapshot: state.snapshot,
    message: `Đã tuyển dụng thành công ${candidate.name} vào vị trí ${candidate.role}!`,
  });
});

// Autonomous Auto-Pilot Tick (Self-running continuous revenue and production engine)
app.post('/api/auto-pilot/tick', async (req, res) => {
  const cycle = state.snapshot.cycle_count + 1;
  state.snapshot.cycle_count = cycle;
  state.autonomousSettings.lastTickTimestamp = new Date().toISOString();

  // 1. Autonomous Revenue Generation from active campaigns
  const revBase = Math.floor(Math.random() * 65000 + 45000); // +$450 - $1,100
  state.snapshot.cash_minor += revBase;
  state.snapshot.revenue_minor += Math.floor(revBase * 0.8);
  state.snapshot.content_revenue_minor += revBase;
  state.snapshot.conversion_bps = Math.min(650, state.snapshot.conversion_bps + Math.floor(Math.random() * 4 + 1));
  state.snapshot.audience_growth_bps = Math.min(450, state.snapshot.audience_growth_bps + Math.floor(Math.random() * 5 + 1));

  // 2. Book double-entry revenue
  state.ledger.unshift({
    id: `tx-auto-rev-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `Auto-Pilot: Doanh thu đối soát bán hàng tự động & hoa hồng TikTok Shop`,
    debitAccount: 'Cash & Cash Equivalents',
    creditAccount: 'Affiliate Media Revenue (Automated)',
    amount_minor: revBase,
    cycle,
  });

  // 3. Auto-hire if backlog is high and autoHire is enabled
  let autoHiredMessage = null;
  if (state.autonomousSettings.autoHireWhenBacklogHigh && state.snapshot.backlog > 14 && state.snapshot.runway_days > 90) {
    const availableCand = state.candidatePool.find(c => c.status === 'Available');
    if (availableCand) {
      availableCand.status = 'Hired';
      const autoAgent = {
        id: `agent-${availableCand.id}`,
        name: availableCand.name,
        role: availableCand.role,
        department: availableCand.department,
        description: availableCand.bio,
        salary_minor: availableCand.expectedSalaryMinor,
        tasksCompleted: 1,
        status: 'Active' as const,
        hiredAtCycle: cycle,
        skillLevel: 3,
        taskMultiplier: 1.5,
      };
      state.customAgents.push(autoAgent);
      state.snapshot.capacity += 8;
      state.snapshot.backlog = Math.max(2, state.snapshot.backlog - 6);
      autoHiredMessage = `Đã tự động tuyển dụng ${availableCand.name} (${availableCand.role}) để giải tỏa nghẽn hàng đợi!`;

      logOfficeActivity({
        agentId: 'agent-recruiter',
        agentName: 'Recruiter AI',
        agentRole: 'Tuyển Dụng',
        department: 'Ops',
        actionType: 'RecruitTalent',
        title: `Auto-Hire: Tự động tuyển dụng ${availableCand.name}`,
        detail: `Hàng đợi backlog tăng cao. Tự động giải phóng áp lực vận hành bằng nhân sự ${availableCand.role}.`,
        badgeColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
      });
    }
  }

  // 4. Record Office Events
  logOfficeActivity({
    agentId: 'agent-growth',
    agentName: 'Growth Lead',
    agentRole: 'Kinh Doanh & Traffic',
    department: 'Growth',
    actionType: 'ExecutePayout',
    title: `Auto-Pilot Tick: Tự động thu về +$${(revBase / 100).toFixed(2)} doanh thu`,
    detail: `Hệ thống tự động phát hành nội dung đa nền tảng, ghi nhận +${(Math.random() * 12 + 8).toFixed(0)} đơn hàng thành công.`,
    impactMinor: revBase,
    badgeColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
  });

  // 5. Autonomous 24/7 Livestream & Channel Growth Engine
  state.channels.forEach(ch => {
    ch.followers += Math.floor(Math.random() * 120 + 60);
    ch.views30d += Math.floor(Math.random() * 15000 + 8000);
    ch.estimatedValuationMinor = Math.round(ch.followers * 6 + (ch.monthlyDonationMinor * 3.5));

    if (ch.activeStreamSession && ch.activeStreamSession.streamStatus === 'Live') {
      ch.activeStreamSession.liveDurationSec += 60;
      const donInc = Math.floor(Math.random() * 2500 + 1000); // +$10.00 - $35.00 donate
      ch.activeStreamSession.donationReceivedMinor += donInc;
      ch.monthlyDonationMinor += donInc;
      state.snapshot.cash_minor += donInc;
      state.snapshot.revenue_minor += donInc;

      // Every 4 cycles, AI Director auto-switches topic/virtual set to keep retention fresh
      if (cycle % 4 === 0) {
        const nextTheme = AUTONOMOUS_LIVE_THEMES[cycle % AUTONOMOUS_LIVE_THEMES.length];
        ch.activeStreamSession.currentGameOrTopic = nextTheme.topic;
        ch.activeStreamSession.title = nextTheme.title;
        ch.activeStreamSession.hostAgentName = nextTheme.host;
        ch.activeStreamSession.hostAgentAvatar = nextTheme.avatar;
        ch.activeStreamSession.streamType = nextTheme.streamType;
        ch.activeStreamSession.personaStyle = nextTheme.personaStyle;
        ch.activeStreamSession.virtualSet = nextTheme.virtualSet;
        ch.activeStreamSession.aiDecisionRationale = nextTheme.rationale;

        logOfficeActivity({
          agentId: 'agent-streamops',
          agentName: 'Kenji Sato AI',
          agentRole: 'Đạo Diễn Live & Quản Trị Kênh',
          department: 'Ops',
          actionType: 'CreateContent',
          title: `AI Auto-Rotate: Chuyển sang ${nextTheme.virtualSet}`,
          detail: `Thuật toán tự động chuyển chủ đề "${nextTheme.topic}". Tối ưu hóa retention và tăng donate fans.`,
          badgeColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
        });
      }
    }
  });

  // 6. Auto-Audit every 10 cycles
  let latestAuditReport = null;
  if (cycle % 10 === 0) {
    latestAuditReport = generateAutoAuditReport(cycle);
  }

  recalculateCompanyHealth();
  trimMemoryState();

  res.json({
    success: true,
    cycle,
    revenueGainedMinor: revBase,
    autoHiredMessage,
    snapshot: state.snapshot,
    auditReport: latestAuditReport,
    pnl: calculatePnL(),
  });
});

app.post('/api/auto-pilot/settings', (req, res) => {
  const { isAutoPilotActive, intervalSeconds, autoHireWhenBacklogHigh, autoReinvestProfitPct } = req.body;
  if (typeof isAutoPilotActive === 'boolean') state.autonomousSettings.isAutoPilotActive = isAutoPilotActive;
  if (typeof intervalSeconds === 'number') state.autonomousSettings.intervalSeconds = intervalSeconds;
  if (typeof autoHireWhenBacklogHigh === 'boolean') state.autonomousSettings.autoHireWhenBacklogHigh = autoHireWhenBacklogHigh;
  if (typeof autoReinvestProfitPct === 'number') state.autonomousSettings.autoReinvestProfitPct = autoReinvestProfitPct;

  res.json({ success: true, settings: state.autonomousSettings });
});

// Full Multi-Role Creative Production Suite Generator (Copywriter + Music + Photo + Video + Producer)
app.post('/api/generate-creative-suite', async (req, res) => {
  const { niche, productCategory } = req.body;
  const category = productCategory || niche || 'AI Smart Workspace & Desk Setup Gadgets';

  let production = {
    id: `prod-${Date.now().toString(36)}`,
    campaignTitle: `3 Món Đồ Công Nghệ AI Giúp Tôi Tiết Kiệm 14 Tiếng Mỗi Tuần`,
    niche: category,
    projectedRevenueMinor: 145000, // $1,450 projected
    costMinor: 15000, // $150 render/creation cost
    copywriting: {
      headline: `Bí Quyết Tăng 300% Năng Suất Làm Việc Với 3 Phụ Kiện AI Này`,
      hook3s: `Đừng mua thêm bàn phím cơ nữa nếu bạn chưa biết 3 món đồ AI này vừa ra mắt trong tháng.`,
      retentionFormula: `Mở đầu phản trực giác ➔ Khơi gợi nỗi đau mất thời gian ➔ Trình diễn giải pháp AI ➔ Tặng coupon giảm 25% độc quyền`,
      bodyPainPoints: [
        `Ghi chép cuộc họp thủ công mất hàng giờ mỗi tuần`,
        `Dây nhợ lộn xộn làm giảm tập trung và thẩm mỹ góc làm việc`,
        `Không bảo mật dữ liệu cục bộ khi dùng các công cụ đám mây`,
      ],
      ctaText: `Bấm ngay vào link bio và nhập mã AGENTSOS để nhận voucher độc quyền 25% trước khi hết slot.`,
      targetAudience: `Dân văn phòng, lập trình viên, content creator, người yêu công nghệ (22-38 tuổi)`,
      complianceChecked: true,
    },
    audioTrack: {
      title: `Cyberpunk Lo-Fi Productivity Beats (128 BPM)`,
      genre: 'Lo-Fi Chill' as const,
      bpm: 128,
      mood: 'Tập trung cao độ, hiện đại, kích thích hành động',
      voiceoverTone: 'Confident & Crisp' as const,
      voiceSpeed: '1.1x (Nhịp điệu nhanh giữ chân người nghe)',
      loudnessLufs: -14.0,
    },
    visualShots: [
      {
        shotIndex: 1,
        framing: 'Macro Detail Close-Up' as const,
        lighting: 'Studio Softbox Glow' as const,
        imagePrompt: 'Macro 8k photo of ultra-minimalist glowing AI desk hub with sleek aluminum texture, modern clean desk setup, cinematic depth of field',
        durationSec: 3,
        textOverlay: '🔥 3 MÓN ĐỒ AI ĐỔI ĐỜI GÓC SETUP',
      },
      {
        shotIndex: 2,
        framing: '45-Degree Desk Top-Down' as const,
        lighting: 'Cyberpunk Neon Accent' as const,
        imagePrompt: 'Top-down desk view showing AI smart Pebble mouse transcribing meeting notes automatically to tablet screen, tidy setup',
        durationSec: 12,
        textOverlay: '1. Chuột AI tự động tóm tắt cuộc họp',
      },
      {
        shotIndex: 3,
        framing: 'Side Split Comparison' as const,
        lighting: 'Studio Softbox Glow' as const,
        imagePrompt: 'Split screen comparing chaotic messy notebook vs crystal-clear AI dashboard summarizer on ultra-wide monitor',
        durationSec: 15,
        textOverlay: '2. Hub USB-C chạy Local LLM bảo mật 100%',
      },
      {
        shotIndex: 4,
        framing: 'POV Handheld Showcase' as const,
        lighting: 'Warm Natural Daylight' as const,
        imagePrompt: 'POV hand holding phone showing exclusive discount badge code AGENTSOS with glowing TikTok Shop button',
        durationSec: 10,
        textOverlay: '🎁 MÃ GIẢM 25%: AGENTSOS (LINK BIO)',
      },
    ],
    renderSettings: {
      resolution: '1080x1920 (Vertical 9:16)',
      fps: 60,
      codec: 'libx264 / yuv420p (+faststart web-optimized)',
      aspectRatio: '9:16',
    },
    governorApproved: true,
    publishedChannels: ['TikTok Shop', 'YouTube Shorts', 'Instagram Reels', 'Facebook Video'],
    attributionEpc: '$0.84 / Click',
  };

  if (ai) {
    try {
      const prompt = `You are the Multi-Disciplinary Media Studio of an AI Media Enterprise.
Generate a comprehensive production package for: "${category}".
Roles involved:
- Senior Copywriter (Viral Hook 3s, Retention, Pain points, CTA)
- Music Producer (Beat genre, BPM, Voiceover tone, -14 LUFS)
- Prompt Photographer (Camera framing, lighting, 4 visual shots with image prompts)
- Video Editor & Director (FFmpeg specs, duration, text overlays)
- Compliance Officer (FTC verification, EPC projection)

Return STRICT JSON matching the schema:
{
  "campaignTitle": "Title in Vietnamese",
  "projectedRevenueMinor": 150000,
  "costMinor": 15000,
  "copywriting": {
    "headline": "...",
    "hook3s": "...",
    "retentionFormula": "...",
    "bodyPainPoints": ["...", "..."],
    "ctaText": "...",
    "targetAudience": "...",
    "complianceChecked": true
  },
  "audioTrack": {
    "title": "...",
    "genre": "Lo-Fi Chill",
    "bpm": 128,
    "mood": "...",
    "voiceoverTone": "Confident & Crisp",
    "voiceSpeed": "1.1x",
    "loudnessLufs": -14.0
  },
  "visualShots": [
    { "shotIndex": 1, "framing": "Macro Detail Close-Up", "lighting": "Studio Softbox Glow", "imagePrompt": "...", "durationSec": 3, "textOverlay": "..." },
    { "shotIndex": 2, "framing": "45-Degree Desk Top-Down", "lighting": "Cyberpunk Neon Accent", "imagePrompt": "...", "durationSec": 12, "textOverlay": "..." },
    { "shotIndex": 3, "framing": "Side Split Comparison", "lighting": "Studio Softbox Glow", "imagePrompt": "...", "durationSec": 15, "textOverlay": "..." },
    { "shotIndex": 4, "framing": "POV Handheld Showcase", "lighting": "Warm Natural Daylight", "imagePrompt": "...", "durationSec": 10, "textOverlay": "..." }
  ],
  "renderSettings": {
    "resolution": "1080x1920 (Vertical 9:16)",
    "fps": 60,
    "codec": "libx264 / yuv420p (+faststart web-optimized)",
    "aspectRatio": "9:16"
  },
  "governorApproved": true,
  "publishedChannels": ["TikTok Shop", "YouTube Shorts", "Instagram Reels"],
  "attributionEpc": "$0.88 / Click"
}`;
      const resp = await ai.models.generateContent({
        model: 'gemini-3.8-flash',
        contents: prompt,
        config: { responseMimeType: 'application/json', temperature: 0.5 },
      });
      const parsed = JSON.parse(resp.text || '{}');
      if (parsed.campaignTitle) {
        production = {
          ...production,
          ...parsed,
          id: `prod-${Date.now().toString(36)}`,
          niche: category,
        };
      }
    } catch (e) {
      console.warn('Creative suite generation fallback:', e);
    }
  }

  state.creativeProductions.unshift(production);
  trimMemoryState();

  logOfficeActivity({
    agentId: 'agent-content',
    agentName: 'Content Studio',
    agentRole: 'Sáng Tạo Toàn Diện',
    department: 'Growth',
    actionType: 'CreateContent',
    title: `Sản xuất trọn gói kịch bản + âm nhạc + visual: "${production.campaignTitle}"`,
    detail: `Kịch bản đạt chuẩn hook 3s; Âm nhạc ${production.audioTrack.genre} ${production.audioTrack.bpm} BPM (-14 LUFS); 4 góc chụp hình ảnh; Render FFmpeg 60fps.`,
    impactMinor: -production.costMinor,
    badgeColor: 'text-pink-400 bg-pink-500/10 border-pink-500/20',
  });

  res.json({ success: true, production });
});

app.get('/api/system-alerts', (req, res) => {
  res.json({
    alerts: state.systemAlerts,
    activeCount: state.systemAlerts.filter(a => !a.resolved).length,
  });
});

app.post('/api/system-alerts/resolve', (req, res) => {
  const { alertId } = req.body;
  const target = state.systemAlerts.find(a => a.id === alertId);
  if (target) {
    target.resolved = true;
  }
  res.json({ success: true, alerts: state.systemAlerts });
});

app.post('/api/system-alerts/trigger', (req, res) => {
  const { type, level, title, description, mitigationAction } = req.body;
  const newAlert = {
    id: `alert-${Date.now().toString(36)}`,
    level: level || 'Warning',
    type: type || 'High Expense Spike',
    title: title || 'Governor phát hiện biến động chi phí bất thường',
    description: description || 'Hệ thống tự động kích hoạt ngưỡng an toàn hiến định.',
    discoveredBy: 'Governor AI',
    cycle: state.snapshot.cycle_count,
    timestamp: new Date().toISOString(),
    resolved: false,
    mitigationAction: mitigationAction || 'Tạm hoãn chi tiêu phát sinh ngoài kế hoạch.',
  };
  state.systemAlerts.unshift(newAlert);
  res.json({ success: true, alert: newAlert, alerts: state.systemAlerts });
});

app.get('/api/communication-stream', (req, res) => {
  res.json({
    stream: state.communicationStream,
    totalCount: state.communicationStream.length,
    activeCycle: state.snapshot.cycle_count,
  });
});

app.post('/api/communication-stream', (req, res) => {
  const { type, fromAgent, toAgent, subject, content, actionItem, tag, priority } = req.body;
  
  const newMessage = {
    id: `comm-${Date.now().toString(36)}`,
    type: type || 'Chat',
    fromAgent: fromAgent || 'CEO',
    fromRole: fromAgent === 'CEO' ? 'Tổng Giám Đốc' : fromAgent === 'CFO' ? 'Giám Đốc Tài Chính' : fromAgent === 'Growth' ? 'Kinh Doanh' : 'Điều Hành',
    toAgent: toAgent || 'Toàn Thể Công Ty',
    toRole: 'Hội đồng',
    subject: subject || undefined,
    content: content || 'Đã đồng bộ chỉ số chu kỳ và cập nhật hạn ngạch tự động.',
    actionItem: actionItem || undefined,
    tag: tag || '#AutonomousCoordination',
    cycle: state.snapshot.cycle_count,
    timestamp: new Date().toISOString(),
    priority: priority || 'Normal',
  };

  state.communicationStream.unshift(newMessage);
  res.json({ success: true, message: newMessage, stream: state.communicationStream });
});

app.get('/api/auto-audit', (req, res) => {
  res.json({
    reports: state.auditReports,
    latestReport: state.auditReports[0] || null,
    nextAuditCycle: Math.ceil((state.snapshot.cycle_count + 0.1) / 10) * 10,
    currentCycle: state.snapshot.cycle_count,
  });
});

app.post('/api/trigger-audit', (req, res) => {
  const report = generateAutoAuditReport(state.snapshot.cycle_count);
  res.json({ success: true, report, reports: state.auditReports });
});

// Run a complete autonomous cycle
app.post('/api/run-cycle', async (req, res) => {
  const snapshotBefore = { ...state.snapshot };
  state.snapshot.cycle_count += 1;

  const roles: AgentRole[] = ['CEO', 'CFO', 'COO', 'Growth', 'Content', 'Recruiter', 'Analyst', 'Experiment'];
  
  // Run agents in parallel
  const proposalPromises = roles.map(role => generateAgentProposalWithAI(role, state.snapshot));
  const rawProposals = await Promise.all(proposalPromises);

  const governedProposals: GovernedProposal[] = [];
  const cycleReceipts: ExecutionReceipt[] = [];

  for (const proposal of rawProposals) {
    const govResult = evaluateGovernor(proposal, state.snapshot);
    let executed = false;

    if (govResult.decision === 'Approve') {
      const receipt = executeProposal(proposal);
      state.receipts.unshift(receipt);
      cycleReceipts.push(receipt);
      executed = true;
    }

    governedProposals.push({
      proposal,
      decision: govResult.decision,
      reason: govResult.reason,
      evaluatedAt: new Date().toISOString(),
      executed,
    });
  }

  recalculateCompanyHealth();

  // Auto-Audit Routine: Triggers a formal fiduciary summary report every 10 business cycles
  let latestAuditReport = null;
  if (state.snapshot.cycle_count % 10 === 0) {
    latestAuditReport = generateAutoAuditReport(state.snapshot.cycle_count);
    // Log entry in double-entry ledger
    state.ledger.unshift({
      id: `tx-audit-${state.snapshot.cycle_count}`,
      timestamp: new Date().toISOString(),
      description: `Báo Cáo Kiểm Toán Định Kỳ 10 Chu Kỳ (Kỳ #${state.snapshot.cycle_count}): Doanh thu ${latestAuditReport.variancePercent >= 0 ? '+' : ''}${latestAuditReport.variancePercent}% so với kế hoạch ngân sách`,
      debitAccount: 'Chi Phí Kiểm Toán Fiduciary & Tuân Thủ',
      creditAccount: 'Cash & Cash Equivalents',
      amount_minor: 0,
      cycle: state.snapshot.cycle_count,
    });
  }

  const cycleRecord = {
    cycleNumber: state.snapshot.cycle_count,
    timestamp: new Date().toISOString(),
    proposals: governedProposals,
    snapshotBefore,
    snapshotAfter: { ...state.snapshot },
  };

  state.cycles.push(cycleRecord);

  res.json({
    cycleNumber: state.snapshot.cycle_count,
    proposals: governedProposals,
    receipts: cycleReceipts,
    snapshot: state.snapshot,
    cycleHistory: getCycleTrendHistory(),
    auditReport: latestAuditReport,
    auditReports: state.auditReports,
  });
});

// Executive Deliberation / War Room with Gemini AI
app.post('/api/agent-debate', async (req, res) => {
  const { topic } = req.body;
  const userTopic = topic || 'Should the company pivot 50% of budget into TikTok Shop video automation vs maintaining reserve cash?';

  if (ai) {
    try {
      const prompt = `You are orchestrating an executive boardroom debate for an Autonomous AI Enterprise.
Company State:
- Status: ${state.snapshot.status}
- Cash: $${(state.snapshot.cash_minor / 100).toFixed(2)}
- Runway: ${state.snapshot.runway_days} days
- Monthly Revenue: $${(state.snapshot.revenue_minor / 100).toFixed(2)}
- Monthly Expenses: $${(state.snapshot.expenses_minor / 100).toFixed(2)}

Dilemma Topic: "${userTopic}"

Produce a structured debate involving 4 key agents:
1. CEO (Strategy, bold moves, market share)
2. CFO (Fiduciary vigilance, cash preservation, risk of insolvency)
3. COO (Operational capacity, execution realism, bottleneck warnings)
4. Governor (Final constitutional verdict, statutory policy decision)

Return a STRICT JSON array of objects (no markdown, just raw JSON):
[
  { "agent": "CEO", "stance": "Pro/Aggressive", "argument": "...", "proposedAction": "..." },
  { "agent": "CFO", "stance": "Cautious/Counter", "argument": "...", "proposedAction": "..." },
  { "agent": "COO", "stance": "Pragmatic/Operational", "argument": "...", "proposedAction": "..." },
  { "agent": "Governor", "stance": "Constitutional Ruling", "argument": "...", "finalRuling": "Approve / Reject / Conditional Pass with Cap", "policyJustification": "..." }
]`;

      const response = await ai.models.generateContent({
        model: 'gemini-3.8-flash',
        contents: prompt,
        config: {
          responseMimeType: 'application/json',
          temperature: 0.5,
        },
      });

      const parsed = JSON.parse(response.text || '[]');
      return res.json({ debate: parsed, topic: userTopic });
    } catch (err) {
      console.warn('Debate generation fallback:', err);
    }
  }

  // Fallback debate
  res.json({
    topic: userTopic,
    debate: [
      {
        agent: 'CEO',
        stance: 'Pro-Aggressive Expansion',
        argument: `Our audience growth rate of ${(state.snapshot.audience_growth_bps / 100).toFixed(2)}% shows strong product-market fit. Hesitating now will forfeit first-mover advantage to competitor syndicates.`,
        proposedAction: 'Authorize $1,500 injection into automated creator pipelines.',
      },
      {
        agent: 'CFO',
        stance: 'Fiscal Defense & Liquidity Buffer',
        argument: `With runway at ${state.snapshot.runway_days} days and fixed burn of $${(state.snapshot.expenses_minor / 100).toFixed(2)}/mo, reckless capital flight risks triggering Distress status under economic-core law.`,
        proposedAction: 'Cap any pilot at strictly $250 with immediate 14-day break-even audit.',
      },
      {
        agent: 'COO',
        stance: 'Capacity & Workflow Realism',
        argument: `Our current backlog sits at ${state.snapshot.backlog} against capacity ${state.snapshot.capacity}. Rushing 100 new videos without queue stabilization will trigger media worker timeouts.`,
        proposedAction: 'Queue 15 high-confidence scripts first to benchmark rendering latency.',
      },
      {
        agent: 'Governor',
        stance: 'Constitutional Verdict',
        argument: 'Section 4 of Company OS Constitution permits bounded exploration only when double-entry reserves exceed 90 days.',
        finalRuling: 'Conditional Pass with Cap ($300.00 maximum)',
        policyJustification: 'Permits CEO experiment while honoring CFO liquidity threshold and COO capacity limit.',
      },
    ],
  });
});

// Real-Time AI Content & Script Generator for Media Pipeline
app.post('/api/generate-content', async (req, res) => {
  const { niche, productCategory } = req.body;
  const category = productCategory || 'AI Productivity & Desk Setup Hardware';

  if (ai) {
    try {
      const prompt = `You are the Content Agent of Company OS, an autonomous media company.
Generate a high-converting, viral short-form video concept and script for: "${category}".
Return STRICT JSON:
{
  "title": "Eye-catching title with high CTR",
  "hook": "First 3 seconds verbal and visual hook",
  "scriptOutline": [
    { "timestamp": "0:00 - 0:03", "visual": "...", "audio": "..." },
    { "timestamp": "0:03 - 0:15", "visual": "...", "audio": "..." },
    { "timestamp": "0:15 - 0:35", "visual": "...", "audio": "..." },
    { "timestamp": "0:35 - 0:45", "visual": "...", "audio": "..." }
  ],
  "affiliateOffer": "Suggested high-commission affiliate product",
  "projectedEpc": "$0.58",
  "callToAction": "Link in bio / exclusive coupon code discount",
  "governorComplianceCheck": "Passes platform spam & financial disclosure regulations"
}`;

      const response = await ai.models.generateContent({
        model: 'gemini-3.8-flash',
        contents: prompt,
        config: {
          responseMimeType: 'application/json',
          temperature: 0.6,
        },
      });

      const parsed = JSON.parse(response.text || '{}');
      return res.json({ content: parsed });
    } catch (e) {
      console.warn('Content generation fallback:', e);
    }
  }

  res.json({
    content: {
      title: `The 3 AI Hardware Tools That Actually Saved Me 14 Hours This Week`,
      hook: `Stop buying mechanical keyboards until you see what these 3 AI tools do to your daily workflow.`,
      scriptOutline: [
        { timestamp: '0:00 - 0:03', visual: 'Fast zoom-in on sleek magnetic desk stand with glowing screen', audio: 'Stop buying productivity gadgets until you see this.' },
        { timestamp: '0:03 - 0:15', visual: 'Screen record demonstrating one-click automated meeting summarization', audio: 'First: This AI audio pebble transcribes and files your client action items automatically.' },
        { timestamp: '0:15 - 0:35', visual: 'Side-by-side comparison of chaotic spreadsheet vs instant dashboard', audio: 'Second: Plug this into your USB-C dock, and it runs local LLM shortcuts with zero latency.' },
        { timestamp: '0:35 - 0:45', visual: 'Pointing to affiliate discount coupon banner overlay', audio: 'Use code AGENTSOS for 25% off via the link below before the batch closes.' },
      ],
      affiliateOffer: 'ErgoTech AI Smart Workspace Hub (18% Commission Rate via Awin)',
      projectedEpc: '$0.74',
      callToAction: 'Tap the bio link and apply code AGENTSOS for instant $30 rebate.',
      governorComplianceCheck: 'Clear FTC affiliate sponsorship disclosure mandated and verified.',
    },
  });
});

// Stress-Test Chaos Injection
app.post('/api/chaos-shock', (req, res) => {
  const { shockType } = req.body;

  switch (shockType) {
    case 'cash_drain':
      state.snapshot.cash_minor = Math.floor(state.snapshot.cash_minor * 0.35); // 65% cash loss
      state.ledger.unshift({
        id: `tx-shock-${Date.now().toString(36)}`,
        timestamp: new Date().toISOString(),
        description: 'CHAOS EVENT: Unforeseen Tax Audit & Vendor Dispute Settlement',
        debitAccount: 'Extraordinary Legal Losses',
        creditAccount: 'Cash & Cash Equivalents',
        amount_minor: Math.floor(state.snapshot.cash_minor * 0.65),
        cycle: state.snapshot.cycle_count,
      });
      break;
    case 'revenue_crash':
      state.snapshot.revenue_minor = Math.floor(state.snapshot.revenue_minor * 0.4); // 60% revenue drop
      state.snapshot.conversion_bps = Math.floor(state.snapshot.conversion_bps * 0.5);
      break;
    case 'burn_spike':
      state.snapshot.expenses_minor = Math.floor(state.snapshot.expenses_minor * 2.2); // 120% expense surge
      break;
    case 'queue_overload':
      state.snapshot.backlog = 85;
      break;
    case 'recovery':
      state.snapshot.cash_minor = 5000000;
      state.snapshot.revenue_minor = 1200000;
      state.snapshot.expenses_minor = 600000;
      state.snapshot.runway_days = 400;
      state.snapshot.status = 'Growth';
      state.snapshot.backlog = 8;
      break;
  }

  recalculateCompanyHealth();
  res.json({ snapshot: state.snapshot, appliedShock: shockType });
});

// Human Operator Override (Escalated Proposals)
app.post('/api/governor-override', (req, res) => {
  const { proposalId, decision } = req.body;
  // Locate proposal in recent cycles
  let found = false;
  for (const c of state.cycles) {
    for (const p of c.proposals) {
      if (p.proposal.id === proposalId) {
        p.decision = decision;
        if (decision === 'Approve') {
          const receipt = executeProposal(p.proposal);
          state.receipts.unshift(receipt);
          p.executed = true;
        }
        found = true;
        break;
      }
    }
  }

  res.json({ success: found, decision });
});

// Dynamic Agent Hiring (Hire a specialist when the company needs a new capability)
app.post('/api/hire-custom-agent', (req, res) => {
  const { name, role, department, description, salary_minor } = req.body;

  // Governor Constitutional Check: Do we have enough runway to support this hire?
  if (state.snapshot.runway_days < 45 && state.snapshot.cash_minor < 2000000) {
    return res.status(400).json({
      success: false,
      reason: `Governor Veto: Runway hiện tại (${state.snapshot.runway_days} ngày) dưới ngưỡng an toàn 45 ngày. Đóng băng tuyển dụng!`,
    });
  }

  const newAgent = {
    id: `agent-custom-${Date.now().toString(36)}`,
    name: name || 'Specialist Agent',
    role: role || 'AI Specialist',
    department: department || 'Growth',
    description: description || 'Chuyên viên xử lý tác vụ theo yêu cầu',
    salary_minor: salary_minor || 150000, // $1,500/mo
    tasksCompleted: 0,
    status: 'Active' as const,
    hiredAtCycle: state.snapshot.cycle_count,
  };

  state.customAgents.push(newAgent);
  
  // Add to employee payroll ledger
  state.employees.push({
    id: newAgent.id,
    name: newAgent.name,
    role: newAgent.role,
    salary_minor: newAgent.salary_minor,
    hiredAtCycle: state.snapshot.cycle_count,
  });

  // Adjust monthly expense & capacity
  state.snapshot.expenses_minor += newAgent.salary_minor;
  state.snapshot.capacity += 8;

  // Book Double-Entry Ledger Entry for Talent Onboarding
  state.ledger.unshift({
    id: `tx-hire-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `Tuyển Dụng Nhân Sự Mới: ${newAgent.name} (${newAgent.role})`,
    debitAccount: 'Chi Phí Tuyển Dụng & Vận Hành AI',
    creditAccount: 'Cash & Cash Equivalents',
    amount_minor: Math.min(newAgent.salary_minor, 50000), // setup/onboarding fee
    cycle: state.snapshot.cycle_count,
  });

  recalculateCompanyHealth();

  res.json({
    success: true,
    agent: newAgent,
    snapshot: state.snapshot,
    message: `Đã tuyển dụng thành công ${newAgent.name}!`,
  });
});

// Toggle Agent Status (Active / Paused)
app.post('/api/toggle-agent-status', (req, res) => {
  const { agentId } = req.body;
  const agent = state.customAgents.find((a: { id: string }) => a.id === agentId);
  if (!agent) {
    return res.status(404).json({ success: false, reason: 'Agent not found' });
  }

  agent.status = agent.status === 'Active' ? 'Paused' : 'Active';
  res.json({ success: true, agentId, status: agent.status });
});

// Agent Skill Training (Upskilling via Treasury Cash)
app.post('/api/train-agent', (req, res) => {
  const { agentId, course } = req.body;
  const agent: any = state.customAgents.find((a: any) => a.id === agentId);
  if (!agent) {
    return res.status(404).json({ success: false, reason: 'Không tìm thấy nhân sự AI.' });
  }

  const cost_minor = course?.cost_minor || 50000;
  if (state.snapshot.cash_minor < cost_minor) {
    return res.status(400).json({ success: false, reason: 'Kho bạc không đủ ngân quỹ để đầu tư khóa đào tạo này.' });
  }

  // Deduct training investment from cash
  state.snapshot.cash_minor -= cost_minor;

  // Apply skill upgrades
  agent.skillLevel = (agent.skillLevel || 1) + 1;
  const currentMult = agent.taskMultiplier || 1.0;
  agent.taskMultiplier = Math.round((currentMult + (course?.multiplierBoost || 0.25)) * 100) / 100;
  agent.tasksCompleted += (course?.tasksBonus || 10);
  agent.trainedSkills = [...(agent.trainedSkills || []), course?.name || 'Chuyên Môn Hóa AI'];
  agent.trainingCount = (agent.trainingCount || 0) + 1;

  // Slightly expand overall company capacity
  state.snapshot.capacity += 2;

  // Book Double-Entry Accounting
  state.ledger.unshift({
    id: `tx-train-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `Đào Tạo Kỹ Năng (Upskilling): ${agent.name} - ${course?.name || 'Khóa Đào Tạo'}`,
    debitAccount: 'Đầu Tư Phát Triển Nhân Sự AI (Human Capital / R&D)',
    creditAccount: 'Cash & Cash Equivalents',
    amount_minor: cost_minor,
    cycle: state.snapshot.cycle_count,
  });

  // Record Execution Receipt
  state.receipts.unshift({
    id: `rcpt-train-${Date.now().toString(36)}`,
    proposalId: `prop-train-${agent.id}`,
    agent: 'COO',
    action: 'RebalanceOperations',
    status: 'Completed',
    outcome: `Hoàn tất khóa đào tạo "${course?.name}" cho ${agent.name}. Hệ số năng suất tăng lên ${agent.taskMultiplier}x, giải quyết nguy cơ nghẽn cổ chai.`,
    cost_minor: cost_minor,
    timestamp: new Date().toISOString(),
  });

  recalculateCompanyHealth();

  res.json({
    success: true,
    agent,
    snapshot: state.snapshot,
    message: `Đào tạo thành công cho ${agent.name}! Hệ số hoàn thành tác vụ tăng lên ${agent.taskMultiplier}x.`,
  });
});

// Auto-Recruit Route: Recruiter autonomously hires specialized roles when hiring_need > 0 and cash >= threshold
app.post('/api/auto-recruit', (req, res) => {
  const { thresholdMinor } = req.body;
  const threshold = thresholdMinor || 4000000; // $40,000 default cash safety threshold

  // Condition 1: Hiring need must be positive
  if (state.snapshot.hiring_need <= 0) {
    return res.status(400).json({
      success: false,
      reason: 'Recruiter Thông Báo: Định mức biên chế hiện tại đã đủ (Hiring Need = 0).',
    });
  }

  // Condition 2: Company cash must exceed safety threshold
  if (state.snapshot.cash_minor < threshold) {
    return res.status(400).json({
      success: false,
      reason: `Governor Veto: Tiền mặt Kho Bạc ($${(state.snapshot.cash_minor / 100).toLocaleString()}) chưa vượt ngưỡng an toàn ($${(threshold / 100).toLocaleString()}).`,
    });
  }

  const specializedRoles = [
    {
      name: 'TikTok Shop Growth Specialist',
      role: 'Chuyên Viên Tối Ưu Hóa TikTok Shop',
      department: 'Growth' as const,
      description: 'Chuyên quét mã giảm giá, tối ưu thẻ affiliate và đàm phán hợp đồng hoa hồng độc quyền.',
      salary_minor: 120000,
    },
    {
      name: 'High-Ticket B2B Deal Closer',
      role: 'Chuyên Viên Chốt Deal Phần Mềm B2B',
      department: 'Growth' as const,
      description: 'Khai thác các ngách phần mềm CRM, Marketing Automation với mức hoa hồng định kỳ $200-$500/khách hàng.',
      salary_minor: 140000,
    },
    {
      name: 'Forensic Compliance & Tax Auditor',
      role: 'Kiểm Toán Viên Fiduciary & Thuế',
      department: 'Ops' as const,
      description: 'Giám sát tính minh bạch FTC trong video review và tự động hạch toán doanh thu affiliate xuyên biên giới.',
      salary_minor: 110000,
    },
    {
      name: 'LLM Prompt Cache Engineer',
      role: 'Kỹ Sư Hạ Tầng GPU & Bộ Nhớ Cache LLM',
      department: 'Tech' as const,
      description: 'Tối ưu hóa chi phí API Gemini, giảm 50% độ trễ batch video rendering cho Media Worker.',
      salary_minor: 150000,
    },
  ];

  const existingNames = new Set(state.customAgents.map((a: any) => a.name));
  const candidate = specializedRoles.find(r => !existingNames.has(r.name)) || specializedRoles[0];

  const newAgent = {
    id: `agent-auto-${Date.now().toString(36)}`,
    name: candidate.name,
    role: candidate.role,
    department: candidate.department,
    description: candidate.description,
    salary_minor: candidate.salary_minor,
    tasksCompleted: 4,
    status: 'Active' as const,
    hiredAtCycle: state.snapshot.cycle_count,
    skillLevel: 1,
    taskMultiplier: 1.0,
  };

  state.customAgents.push(newAgent);
  state.employees.push({
    id: newAgent.id,
    name: newAgent.name,
    role: newAgent.role,
    salary_minor: newAgent.salary_minor,
    hiredAtCycle: state.snapshot.cycle_count,
  });

  state.snapshot.hiring_need = Math.max(0, state.snapshot.hiring_need - 1);
  state.snapshot.expenses_minor += newAgent.salary_minor;
  state.snapshot.capacity += 8;

  state.ledger.unshift({
    id: `tx-autorecruit-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `Auto-Recruit: Recruiter Agent tự động tuyển dụng ${newAgent.name} (Hiring Need > 0 & Kho Bạc An Toàn)`,
    debitAccount: 'Chi Phí Tuyển Dụng & Phát Triển Nhân Sự AI',
    creditAccount: 'Cash & Cash Equivalents',
    amount_minor: 30000,
    cycle: state.snapshot.cycle_count,
  });

  state.receipts.unshift({
    id: `rcpt-autorecruit-${Date.now().toString(36)}`,
    proposalId: `prop-autorecruit-${newAgent.id}`,
    agent: 'Recruiter',
    action: 'ProposeHire',
    status: 'Completed',
    outcome: `Recruiter AI tự động kích hoạt tuyển dụng chuyên môn: ${newAgent.name} (${newAgent.role}). Hiring Need hạ xuống ${state.snapshot.hiring_need}.`,
    cost_minor: 30000,
    timestamp: new Date().toISOString(),
  });

  recalculateCompanyHealth();

  res.json({
    success: true,
    agent: newAgent,
    snapshot: state.snapshot,
    message: `Recruiter AI đã tự động tuyển dụng thành công ${newAgent.name} (${newAgent.role})!`,
  });
});

// Unified Autonomous Talent Cycle (Auto-Recruit & Auto-Train when company needs it)
app.post('/api/auto-talent-cycle', (req, res) => {
  const { 
    autoRecruit = true, 
    autoTrain = true, 
    cashSafetyThreshold = 3500000,
    maxTrainBudget = 60000
  } = req.body;

  const actions: string[] = [];
  let hiredAgent: any = null;
  let trainedAgent: any = null;
  let trainedCourse: any = null;

  // 1. AUTO-RECRUIT CHECK: Trigger if Hiring Need > 0 OR Backlog > Capacity
  const needHiring = state.snapshot.hiring_need > 0 || state.snapshot.backlog > state.snapshot.capacity;
  const isCashSafe = state.snapshot.cash_minor >= cashSafetyThreshold;

  if (autoRecruit && needHiring && isCashSafe) {
    const specializedRoles = [
      {
        name: 'TikTok Shop Growth Specialist',
        role: 'Chuyên Viên Tối Ưu Hóa TikTok Shop',
        department: 'Growth' as const,
        description: 'Chuyên quét mã giảm giá, tối ưu thẻ affiliate và đàm phán hợp đồng hoa hồng độc quyền.',
        salary_minor: 120000,
      },
      {
        name: 'High-Ticket B2B Deal Closer',
        role: 'Chuyên Viên Chốt Deal Phần Mềm B2B',
        department: 'Growth' as const,
        description: 'Khai thác các ngách phần mềm CRM, Marketing Automation với mức hoa hồng định kỳ $200-$500/khách hàng.',
        salary_minor: 140000,
      },
      {
        name: 'Forensic Compliance & Tax Auditor',
        role: 'Kiểm Toán Viên Fiduciary & Thuế',
        department: 'Ops' as const,
        description: 'Giám sát tính minh bạch FTC trong video review và tự động hạch toán doanh thu affiliate xuyên biên giới.',
        salary_minor: 110000,
      },
      {
        name: 'LLM Prompt Cache Engineer',
        role: 'Kỹ Sư Hạ Tầng GPU & Bộ Nhớ Cache LLM',
        department: 'Tech' as const,
        description: 'Tối ưu hóa chi phí API Gemini, giảm 50% độ trễ batch video rendering cho Media Worker.',
        salary_minor: 150000,
      },
    ];

    const existingNames = new Set(state.customAgents.map((a: any) => a.name));
    const candidate = specializedRoles.find(r => !existingNames.has(r.name)) || specializedRoles[0];

    hiredAgent = {
      id: `agent-auto-${Date.now().toString(36)}`,
      name: candidate.name,
      role: candidate.role,
      department: candidate.department,
      description: candidate.description,
      salary_minor: candidate.salary_minor,
      tasksCompleted: 4,
      status: 'Active' as const,
      hiredAtCycle: state.snapshot.cycle_count,
      skillLevel: 1,
      taskMultiplier: 1.0,
    };

    state.customAgents.push(hiredAgent);
    state.employees.push({
      id: hiredAgent.id,
      name: hiredAgent.name,
      role: hiredAgent.role,
      salary_minor: hiredAgent.salary_minor,
      hiredAtCycle: state.snapshot.cycle_count,
    });

    state.snapshot.hiring_need = Math.max(0, state.snapshot.hiring_need - 1);
    state.snapshot.expenses_minor += hiredAgent.salary_minor;
    state.snapshot.capacity += 8;

    state.ledger.unshift({
      id: `tx-autorecruit-${Date.now().toString(36)}`,
      timestamp: new Date().toISOString(),
      description: `[Auto-Recruit] Tuyển dụng tự động ${hiredAgent.name} (Giải tỏa Backlog: ${state.snapshot.backlog}/${state.snapshot.capacity})`,
      debitAccount: 'Chi Phí Tuyển Dụng & Phát Triển Nhân Sự AI',
      creditAccount: 'Cash & Cash Equivalents',
      amount_minor: 30000,
      cycle: state.snapshot.cycle_count,
    });

    state.receipts.unshift({
      id: `rcpt-autorecruit-${Date.now().toString(36)}`,
      proposalId: `prop-autorecruit-${hiredAgent.id}`,
      agent: 'Recruiter',
      action: 'ProposeHire',
      status: 'Completed',
      outcome: `Tự động bổ sung nhân sự chuyên môn ${hiredAgent.name} (${hiredAgent.role}).`,
      cost_minor: 30000,
      timestamp: new Date().toISOString(),
    });

    actions.push(`🤖 Tự động tuyển: ${hiredAgent.name} (+8 Capacity)`);
  }

  // 2. AUTO-TRAINING CHECK: Trigger if company has cash above reserve & agents can be upskilled
  if (autoTrain && state.snapshot.cash_minor >= (cashSafetyThreshold + maxTrainBudget)) {
    const candidateAgentIds = ['agent-content', 'agent-growth', 'agent-coo', 'agent-analyst'];
    const targetAgent: any = state.customAgents.find((a: any) => 
      candidateAgentIds.includes(a.id) && (a.taskMultiplier || 1.0) < 1.8
    );

    if (targetAgent) {
      const courses = [
        {
          id: 'course-prompt-cache',
          name: 'Tối Ưu Hóa Prompt Caching & Batching LLM',
          cost_minor: 40000,
          multiplierBoost: 0.25,
          tasksBonus: 8,
        },
        {
          id: 'course-video-pipeline',
          name: 'Điều Phối Dây Chuyền Render Video Tốc Độ Cao',
          cost_minor: 55000,
          multiplierBoost: 0.30,
          tasksBonus: 12,
        },
        {
          id: 'course-affiliate-deals',
          name: 'Thuật Toán Quét Ngách & Săn Deal Hoa Hồng Cao',
          cost_minor: 60000,
          multiplierBoost: 0.30,
          tasksBonus: 12,
        },
      ];

      const selectedCourse = targetAgent.id === 'agent-content' 
        ? courses[1] 
        : targetAgent.id === 'agent-growth' 
        ? courses[2] 
        : courses[0];

      if (state.snapshot.cash_minor >= selectedCourse.cost_minor) {
        state.snapshot.cash_minor -= selectedCourse.cost_minor;
        targetAgent.skillLevel = (targetAgent.skillLevel || 1) + 1;
        targetAgent.taskMultiplier = Math.round(((targetAgent.taskMultiplier || 1.0) + selectedCourse.multiplierBoost) * 100) / 100;
        targetAgent.tasksCompleted += selectedCourse.tasksBonus;
        targetAgent.trainedSkills = [...(targetAgent.trainedSkills || []), selectedCourse.name];
        targetAgent.trainingCount = (targetAgent.trainingCount || 0) + 1;
        state.snapshot.capacity += 2;

        state.ledger.unshift({
          id: `tx-autotrain-${Date.now().toString(36)}`,
          timestamp: new Date().toISOString(),
          description: `[Auto-Upskill] Đào tạo tự động ${targetAgent.name}: "${selectedCourse.name}" (Năng suất: ${targetAgent.taskMultiplier}x)`,
          debitAccount: 'Đầu Tư Phát Triển Nhân Sự AI (Human Capital / R&D)',
          creditAccount: 'Cash & Cash Equivalents',
          amount_minor: selectedCourse.cost_minor,
          cycle: state.snapshot.cycle_count,
        });

        trainedAgent = targetAgent;
        trainedCourse = selectedCourse;
        actions.push(`🎓 Tự động đào tạo: ${targetAgent.name} (Multiplier: ${targetAgent.taskMultiplier}x)`);
      }
    }
  }

  recalculateCompanyHealth();

  if (actions.length === 0) {
    return res.json({
      success: true,
      actionsTaken: false,
      snapshot: state.snapshot,
      customAgents: state.customAgents,
      message: 'Hệ thống đã rà soát: Đội ngũ đang hoạt động ở trạng thái cân bằng, chưa cần tuyển thêm hay đào tạo khẩn cấp.',
    });
  }

  res.json({
    success: true,
    actionsTaken: true,
    hiredAgent,
    trainedAgent,
    trainedCourse,
    actions,
    snapshot: state.snapshot,
    customAgents: state.customAgents,
    message: actions.join(' | '),
  });
});

// Automated Multi-Agent Pipeline (End-to-End Handoff)
app.post('/api/run-pipeline', async (req, res) => {
  const { topic } = req.body;
  const targetTopic = topic || 'AI Smart Workspace Gadgets 2026';
  const cycle = state.snapshot.cycle_count;

  // Step 1: Growth & Analyst research
  const growthAgent = state.customAgents.find((a: { id: string }) => a.id === 'agent-growth');
  if (growthAgent) growthAgent.tasksCompleted += 1;

  // Step 2: Content writes viral hook & script
  const contentAgent = state.customAgents.find((a: { id: string }) => a.id === 'agent-content');
  if (contentAgent) contentAgent.tasksCompleted += 1;

  // Step 3: Media Worker compiles visual assets
  const cooAgent = state.customAgents.find((a: { id: string }) => a.id === 'agent-coo');
  if (cooAgent) cooAgent.tasksCompleted += 1;

  // Simulation-only outcome. This backend is never the authoritative financial ledger.
  const simulatedRevenueGainMinor = Math.floor(Math.random() * 80000 + 75000); // synthetic +$750 - $1,550
  const costMinor = 15000; // synthetic $150 inference/render cost

  state.snapshot.cash_minor += (simulatedRevenueGainMinor - costMinor);
  state.snapshot.revenue_minor += simulatedRevenueGainMinor;
  state.snapshot.content_revenue_minor += simulatedRevenueGainMinor;
  state.snapshot.backlog = Math.max(0, state.snapshot.backlog - 2);

  // Book Double-Entry Ledger Entry
  state.ledger.unshift({
    id: `tx-pipeline-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `[SIMULATION] Dây Chuyền: mô phỏng xuất bản video & affiliate outcome '${targetTopic.substring(0, 30)}'`,
    debitAccount: 'SIMULATION_CASH',
    creditAccount: 'SIMULATION_AFFILIATE_REVENUE',
    amount_minor: simulatedRevenueGainMinor,
    cycle,
  });

  // Receipt
  state.receipts.unshift({
    id: `rcpt-pipeline-${Date.now().toString(36)}`,
    proposalId: `pipe-${Date.now().toString(36)}`,
    agent: 'Content',
    action: 'PublishContent',
    status: 'Completed',
    outcome: `[SIMULATION] Dây chuyền 4 khâu hoàn tất; synthetic outcome +${(simulatedRevenueGainMinor / 100).toFixed(2)} — không phải revenue đã nhận.`,
    cost_minor: costMinor,
    timestamp: new Date().toISOString(),
  });

  recalculateCompanyHealth();

  res.json({
    success: true,
    topic: targetTopic,
    simulatedRevenueGainMinor,
    simulated: true,
    costMinor,
    snapshot: state.snapshot,
    steps: [
      { step: '1. Nghiên cứu', by: 'Growth Lead', detail: 'Quét 14 mặt hàng affiliate hot trên TikTok Shop' },
      { step: '2. Kịch bản', by: 'Content Lead', detail: 'Tạo hook 3 giây và kịch bản 4 phân cảnh' },
      { step: '3. Sản xuất', by: 'Media Worker', detail: 'Render video và gắn affiliate tracking link' },
      { step: '4. Simulation ledger', by: 'Governor & CFO', detail: `Ghi nhận synthetic outcome +${(simulatedRevenueGainMinor / 100).toFixed(2)} trong simulation ledger` },
    ],
  });
});

// Get Last 10 Tasks for a specific Agent
app.get('/api/agent-tasks/:agentKey', (req, res) => {
  const { agentKey } = req.params;
  const cleanKey = decodeURIComponent(agentKey).toLowerCase();

  // Find agent in customAgents
  const agent = state.customAgents.find((a: { id: string; name: string; role: string }) => 
    a.id.toLowerCase() === cleanKey || 
    a.name.toLowerCase().includes(cleanKey) ||
    cleanKey.includes(a.name.toLowerCase()) ||
    a.role.toLowerCase().includes(cleanKey)
  );

  const matchedReceipts = state.receipts.filter((r) => 
    r.agent.toLowerCase().includes(cleanKey) || (agent && r.agent.toLowerCase().includes(agent.name.toLowerCase()))
  );

  // Map existing receipts
  const tasks = matchedReceipts.map((r, idx) => ({
    id: r.id,
    cycle: Math.max(1, state.snapshot.cycle_count - idx),
    title: `${r.agent} thực thi: ${r.action}`,
    action: r.action,
    outcome: r.outcome,
    cost_minor: r.cost_minor,
    status: r.status as 'Completed' | 'Pending' | 'Failed',
    timestamp: r.timestamp,
  }));

  // If fewer than 10 tasks, generate realistic operational history for this role
  if (tasks.length < 10) {
    const roleName = agent ? agent.name : cleanKey;
    const sampleTemplates: Record<string, { title: string; action: string; outcome: string; cost: number }[]> = {
      growth: [
        { title: 'Quét 20 sản phẩm tiếp thị liên kết hot trên TikTok Shop', action: 'ResearchOpportunity', outcome: 'Phát hiện 3 sản phẩm có tỷ lệ hoa hồng trên 22% và EPC > $0.85.', cost: 0 },
        { title: 'A/B test 4 mẫu tiêu đề giật tít cho video công nghệ', action: 'CreateExperiment', outcome: 'Mẫu tiêu đề câu hỏi phản biện tăng CTR thêm +18.4%.', cost: 15000 },
        { title: 'Thương lượng hợp đồng độc quyền nhà cung cấp phụ kiện', action: 'LaunchCampaign', outcome: 'Ký kết thành công coupon giảm giá độc quyền 25% cho cộng đồng.', cost: 0 },
        { title: 'Thiết lập link tracking tiếp thị đa kênh', action: 'ResearchOpportunity', outcome: 'Hoàn tất gắn UTM parameter và pixel đối soát hoa hồng thời gian thực.', cost: 0 },
        { title: 'Phân tích tệp khách hàng tiềm năng ngách Smart Workspace', action: 'ProduceReport', outcome: 'Nhận diện tệp người dùng 24-35 tuổi có nhu cầu mua thiết bị cao nhất.', cost: 0 },
      ],
      content: [
        { title: 'Soạn kịch bản video viral 45 giây bàn phím cơ công thái học', action: 'PublishContent', outcome: 'Kịch bản hoàn tất đạt chuẩn hook 3 giây giữ chân 72% người xem.', cost: 12000 },
        { title: 'Dựng chuỗi video 3 phần giới thiệu phụ kiện bàn làm việc AI', action: 'PublishContent', outcome: 'Xuất bản tự động trên đa nền tảng, thu hút 42,000 lượt xem tự nhiên.', cost: 15000 },
        { title: 'Tối ưu âm thanh và giọng đọc thuyết minh AI', action: 'PublishContent', outcome: 'Sử dụng voice AI biểu cảm cao, tăng thời gian xem trung bình lên 28 giây.', cost: 5000 },
        { title: 'Thiết kế thumbnail có độ tương phản cao', action: 'PublishContent', outcome: 'CTR ảnh bìa tăng từ 3.2% lên 6.8%.', cost: 3000 },
        { title: 'Gắn thẻ tài trợ và thông báo minh bạch FTC theo quy định', action: 'PublishContent', outcome: 'Đảm bảo tuân thủ chính sách quảng cáo 100%, không bị bóp tương tác.', cost: 0 },
      ],
      cfo: [
        { title: 'Kiểm toán quỹ tiền mặt và đối soát doanh thu sàn', action: 'ProduceReport', outcome: 'Khớp 100% sao kê tài khoản kho bạc và doanh thu hoa hồng thực nhận.', cost: 0 },
        { title: 'Cắt giảm 15% chi phí API LLM dư thừa', action: 'ReduceBudget', outcome: 'Bật bộ nhớ đệm prompt (Prompt Cache), tiết kiệm $350 chi phí máy chủ hàng tháng.', cost: 0 },
        { title: 'Lập mô hình dự phóng Runway cho 90 ngày tới', action: 'ProduceReport', outcome: 'Xác định ngưỡng an toàn tài chính ở mức 45 ngày sống còn.', cost: 0 },
        { title: 'Duyệt bảng lương và chi phí duy trì nhân sự AI', action: 'ProduceReport', outcome: 'Hạch toán chi phí lương đầy đủ vào sổ cái kế toán kép.', cost: 0 },
      ],
      coo: [
        { title: 'Tối ưu hàng đợi xử lý tác vụ media worker', action: 'RebalanceOperations', outcome: 'Giảm thời gian render video từ 4 phút xuống còn 1.2 phút.', cost: 0 },
        { title: 'Kiểm tra độ trễ mạng và thông lượng pipeline tự động', action: 'ProduceReport', outcome: 'Hệ thống vận hành trơn tru với 99.9% uptime.', cost: 0 },
        { title: 'Xử lý hàng đợi tồn đọng (Backlog cleaning)', action: 'RebalanceOperations', outcome: 'Giải quyết 8 tác vụ ứ đọng trong kỳ họp trước.', cost: 0 },
      ],
      governor: [
        { title: 'Phán quyết hiến pháp về đề xuất thử nghiệm tăng trưởng', action: 'ProduceReport', outcome: 'Phê duyệt có điều kiện: Giới hạn ngân sách thử nghiệm tối đa ở $350.', cost: 0 },
        { title: 'Kích hoạt rào chắn bảo vệ quỹ tiền mặt', action: 'ProduceReport', outcome: 'Đảm bảo không khoản chi nào vượt quá 10% tổng quỹ dự trữ.', cost: 0 },
        { title: 'Đánh giá rủi ro pháp lý và điều khoản đối tác', action: 'ProduceReport', outcome: 'Xác nhận hợp đồng tiếp thị không có điều khoản phát sinh chi phí ẩn.', cost: 0 },
      ],
    };

    const fallbackList = sampleTemplates[cleanKey.includes('cfo') ? 'cfo' : cleanKey.includes('content') ? 'content' : cleanKey.includes('growth') ? 'growth' : cleanKey.includes('coo') ? 'coo' : cleanKey.includes('gov') ? 'governor' : 'growth'] || sampleTemplates.growth;

    let fillIdx = 0;
    while (tasks.length < 10) {
      const template = fallbackList[fillIdx % fallbackList.length];
      const cycleNum = Math.max(1, state.snapshot.cycle_count - tasks.length);
      tasks.push({
        id: `mock-task-${cleanKey}-${tasks.length + 1}`,
        cycle: cycleNum,
        title: `${roleName}: ${template.title}`,
        action: template.action as ActionKind,
        outcome: template.outcome,
        cost_minor: template.cost,
        status: 'Completed',
        timestamp: new Date(Date.now() - (tasks.length + 1) * 3600000 * 5).toISOString(),
      });
      fillIdx++;
    }
  }

  res.json({
    agentName: agent ? agent.name : agentKey,
    agentRole: agent ? agent.role : '',
    tasks: tasks.slice(0, 10),
  });
});

// Vite Middleware Mounting for Dev Server
async function startServer() {
  const mode = process.env.COMPANY_OS_MODE ?? 'simulation';
  if (mode !== 'simulation') {
    throw new Error('server.ts is simulation-only; use apps/company-os for production company state and governed side effects');
  }
  const port = process.env.PORT ? parseInt(process.env.PORT, 10) : 3000;
  const host = process.env.HOST ?? '127.0.0.1';
  const remoteAllowed = process.env.SIMULATION_ALLOW_REMOTE === 'true';
  if (!['127.0.0.1', 'localhost', '::1'].includes(host) && !remoteAllowed) {
    throw new Error('simulation UI refuses non-loopback binding unless SIMULATION_ALLOW_REMOTE=true');
  }

  if (process.env.NODE_ENV === 'production') {
    app.use(express.static('dist'));
    app.get('*', (req, res) => {
      res.sendFile(path.resolve(__dirname, 'dist', 'index.html'));
    });
  } else {
    const vite = await createViteServer({
      server: { middlewareMode: true },
      appType: 'spa',
    });
    app.use(vite.middlewares);
  }

  app.listen(port, host, () => {
    console.log(`Company OS Simulation UI listening on http://${host}:${port}`);
    console.log('DATA MODE: SIMULATION — this service is not the authoritative company ledger or external-action plane.');
  });
}

startServer().catch((err) => {
  console.error('Failed to start server:', err);
  process.exit(1);
});
