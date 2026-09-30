import express from 'express';
import { createServer as createViteServer } from 'vite';
import { GoogleGenAI } from '@google/genai';
import dotenv from 'dotenv';
import path from 'path';

dotenv.config();

const app = express();
app.use(express.json());

const SIMULATED_DATA_MODE = 'SIMULATED' as const;

function simulatedMutationsEnabled(): boolean {
  return process.env.ALLOW_SIMULATED_ACTIONS === 'true' && process.env.NODE_ENV !== 'production';
}

function rejectSimulatedMutation(req: express.Request, res: express.Response, next: express.NextFunction) {
  if (['GET', 'HEAD', 'OPTIONS'].includes(req.method) || simulatedMutationsEnabled()) {
    return next();
  }

  res.setHeader('X-Company-Data-Mode', SIMULATED_DATA_MODE);
  return res.status(503).json({
    success: false,
    error: 'simulated_actions_disabled',
    dataMode: SIMULATED_DATA_MODE,
    message: 'This React/Express server is a simulated UI harness. Mutating demo actions are disabled by default and never allowed in production.',
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

// In-Memory Database
const state: {
  snapshot: CompanySnapshot;
  ledger: LedgerEntry[];
  receipts: ExecutionReceipt[];
  cycles: {
    cycleNumber: number;
    timestamp: string;
    proposals: GovernedProposal[];
    snapshotBefore: CompanySnapshot;
    snapshotAfter: CompanySnapshot;
  }[];
  employees: { id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[];
  customAgents: { id: string; name: string; role: string; department: string; description: string; salary_minor: number; tasksCompleted: number; status: 'Active' | 'Paused'; hiredAtCycle: number }[];
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
    { id: 'agent-gov', name: 'Governor', role: 'Hiến Pháp & Quỹ Tiền', department: 'Leadership', description: 'Phủ quyết chi tiêu nguy hiểm, chống phá sản', salary_minor: 70000, tasksCompleted: 42, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-ceo', name: 'CEO', role: 'Tổng Giám Đốc', department: 'Leadership', description: 'Chiến lược tăng trưởng & phân bổ nguồn vốn', salary_minor: 95000, tasksCompleted: 35, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-cfo', name: 'CFO', role: 'Giám Đốc Tài Chính', department: 'Leadership', description: 'Kiểm toán kho bạc và cắt giảm chi tiêu', salary_minor: 90000, tasksCompleted: 22, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-coo', name: 'COO', role: 'Giám Đốc Vận Hành', department: 'Ops', description: 'Điều phối hàng đợi và tiến độ công việc', salary_minor: 80000, tasksCompleted: 50, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-growth', name: 'Growth Lead', role: 'Kinh Doanh & Traffic', department: 'Growth', description: 'Tìm ngách sản phẩm hoa hồng cao', salary_minor: 75000, tasksCompleted: 62, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-content', name: 'Content Lead', role: 'Sáng Tạo Nội Dung', department: 'Growth', description: 'Kịch bản video short-form bán hàng', salary_minor: 60000, tasksCompleted: 78, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-recruiter', name: 'Recruiter', role: 'Tuyển Dụng', department: 'Ops', description: 'Đề xuất bổ sung vị trí mới khi có lãi', salary_minor: 65000, tasksCompleted: 14, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-analyst', name: 'Analyst', role: 'Phân Tích Dữ Liệu', department: 'Ops', description: 'Đối soát số liệu và tính toán hoa hồng', salary_minor: 50000, tasksCompleted: 45, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-experiment', name: 'Experimenter', role: 'Nghiên Cứu A/B Test', department: 'Growth', description: 'Thử nghiệm mẫu kịch bản và thị trường', salary_minor: 55000, tasksCompleted: 29, status: 'Active', hiredAtCycle: 1 },
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
};

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

// REST Endpoints
app.get('/api/state', (req, res) => {
  res.json({
    snapshot: state.snapshot,
    ledger: state.ledger.slice(0, 50),
    receipts: state.receipts.slice(0, 50),
    employees: state.employees,
    customAgents: state.customAgents,
    activeExperiments: state.activeExperiments,
    recentCycles: state.cycles.slice(-5),
    cycleHistory: getCycleTrendHistory(),
    departmentBudgets: getDepartmentBudgetHistory(),
    communicationStream: state.communicationStream,
    systemAlerts: state.systemAlerts,
    auditReports: state.auditReports,
    hasGeminiKey: Boolean(apiKey && apiKey !== 'MY_GEMINI_API_KEY'),
  });
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

  // Yield Real Monetization
  const revenueGainMinor = Math.floor(Math.random() * 80000 + 75000); // +$750 - $1,550
  const costMinor = 15000; // $150 inference/render cost

  state.snapshot.cash_minor += (revenueGainMinor - costMinor);
  state.snapshot.revenue_minor += revenueGainMinor;
  state.snapshot.content_revenue_minor += revenueGainMinor;
  state.snapshot.backlog = Math.max(0, state.snapshot.backlog - 2);

  // Book Double-Entry Ledger Entry
  state.ledger.unshift({
    id: `tx-pipeline-${Date.now().toString(36)}`,
    timestamp: new Date().toISOString(),
    description: `Dây Chuyền Tự Động: Xuất bản video & Thu hoa hồng '${targetTopic.substring(0, 30)}'`,
    debitAccount: 'Cash & Cash Equivalents',
    creditAccount: 'Doanh Thu Tiếp Thị Liên Kết (Affiliate Revenue)',
    amount_minor: revenueGainMinor,
    cycle,
  });

  // Receipt
  state.receipts.unshift({
    id: `rcpt-pipeline-${Date.now().toString(36)}`,
    proposalId: `pipe-${Date.now().toString(36)}`,
    agent: 'Content',
    action: 'PublishContent',
    status: 'Completed',
    outcome: `Dây chuyền phối hợp hoàn tất 4 khâu: Nghiên cứu -> Kịch bản -> Dựng video -> Nhận đối soát +$${(revenueGainMinor / 100).toFixed(2)}.`,
    cost_minor: costMinor,
    timestamp: new Date().toISOString(),
  });

  recalculateCompanyHealth();

  res.json({
    success: true,
    topic: targetTopic,
    revenueGainMinor,
    costMinor,
    snapshot: state.snapshot,
    steps: [
      { step: '1. Nghiên cứu', by: 'Growth Lead', detail: 'Quét 14 mặt hàng affiliate hot trên TikTok Shop' },
      { step: '2. Kịch bản', by: 'Content Lead', detail: 'Tạo hook 3 giây và kịch bản 4 phân cảnh' },
      { step: '3. Sản xuất', by: 'Media Worker', detail: 'Render video và gắn affiliate tracking link' },
      { step: '4. Kế toán', by: 'Governor & CFO', detail: `Ghi nhận doanh thu ròng +$${(revenueGainMinor / 100).toFixed(2)} vào kho bạc` },
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
  const port = process.env.PORT ? parseInt(process.env.PORT, 10) : 3000;

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

  app.listen(port, '0.0.0.0', () => {
    console.log(`Company OS Suite listening on http://0.0.0.0:${port}`);
  });
}

startServer().catch((err) => {
  console.error('Failed to start server:', err);
  process.exit(1);
});
