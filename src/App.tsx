import React, { useState, useEffect } from 'react';
import { CompanySnapshot, GovernedProposal, ExecutionReceipt, LedgerEntry } from './types/company';
import { Header } from './components/Header';
import { ReviewTab } from './components/ReviewTab';
import { CycleRunnerTab } from './components/CycleRunnerTab';
import { WarRoomTab } from './components/WarRoomTab';
import { AgentRosterTab } from './components/AgentRosterTab';
import { LedgerTab } from './components/LedgerTab';
import { ChaosSimulatorTab } from './components/ChaosSimulatorTab';
import { MediaStudioTab } from './components/MediaStudioTab';
import { 
  ShieldAlert, 
  RotateCw, 
  Users, 
  Bot, 
  Landmark, 
  Video, 
  AlertOctagon, 
  CheckCircle2,
  Sparkles
} from 'lucide-react';

export default function App() {
  const [activeTab, setActiveTab] = useState<'audit' | 'cycles' | 'war-room' | 'agents' | 'ledger' | 'media' | 'chaos'>('audit');
  const [isRunningCycle, setIsRunningCycle] = useState(false);
  const [hasGeminiKey, setHasGeminiKey] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const [snapshot, setSnapshot] = useState<CompanySnapshot>({
    status: 'Active',
    cash_minor: 4850000,
    revenue_minor: 950000,
    expenses_minor: 620000,
    budget_remaining_minor: 2400000,
    experiment_budget_minor: 450000,
    runway_days: 440,
    backlog: 12,
    capacity: 22,
    conversion_bps: 240,
    audience_growth_bps: 120,
    content_revenue_minor: 680000,
    content_cost_minor: 290000,
    hiring_need: 1,
    cycle_count: 14,
    currency: 'USD',
  });

  const [proposals, setProposals] = useState<GovernedProposal[]>([
    {
      proposal: {
        id: 'prop-ceo-14',
        agent: 'CEO',
        objective: 'Mở rộng thị trường affiliate AI gadgets sang ngách đồ công nghệ văn phòng cao cấp',
        action: 'AllocateExperimentBudget',
        cost_minor: 50000,
        expected_revenue_minor: 120000,
        risk: 'Low',
        confidence_bps: 8200,
        evidence: ['Tăng trưởng người theo dõi: +1.20%', 'Dòng tiền quỹ dương liên tục 3 chu kỳ'],
        rationale: 'Nhu cầu làm việc từ xa đang đẩy mạnh sức mua các công cụ tự động hóa bàn làm việc.',
        reversible: true,
        timestamp: new Date().toISOString(),
      },
      decision: 'Approve',
      reason: 'Đề xuất tuân thủ ngân sách còn lại và tỷ lệ rủi ro ở mức thấp an toàn.',
      evaluatedAt: new Date().toISOString(),
      executed: true,
    },
    {
      proposal: {
        id: 'prop-cfo-14',
        agent: 'CFO',
        objective: 'Kiểm toán số dư sổ cái kép và xác thực đối soát hoa hồng Awin quý 3',
        action: 'ProduceReport',
        cost_minor: 0,
        expected_revenue_minor: 0,
        risk: 'Low',
        confidence_bps: 9500,
        evidence: ['Tiền mặt kho bạc: $48,500.00', 'Gross margin: 34.7%'],
        rationale: 'Xác thực báo cáo độc lập trước khi tái phân bổ nguồn vốn sang các thử nghiệm mới.',
        reversible: true,
        timestamp: new Date().toISOString(),
      },
      decision: 'Approve',
      reason: 'Báo cáo độc lập không tiêu tốn ngân sách, khuyến khích tính minh bạch.',
      evaluatedAt: new Date().toISOString(),
      executed: true,
    },
    {
      proposal: {
        id: 'prop-growth-14',
        agent: 'Growth',
        objective: 'Triển khai A/B test widget so sánh cấu hình sản phẩm trên TikTok Shop',
        action: 'CreateExperiment',
        cost_minor: 25000,
        expected_revenue_minor: 75000,
        risk: 'Medium',
        confidence_bps: 7800,
        evidence: ['Conversion hiện tại: 2.40%', 'Projected EPC: $0.42'],
        rationale: 'Kỳ vọng tăng tỷ lệ click vào link mua hàng thêm 18 điểm cơ bản.',
        reversible: true,
        timestamp: new Date().toISOString(),
      },
      decision: 'Approve',
      reason: 'Chi phí $250.00 nằm trong hạn mức thử nghiệm có kiểm soát.',
      evaluatedAt: new Date().toISOString(),
      executed: true,
    },
    {
      proposal: {
        id: 'prop-recruiter-14',
        agent: 'Recruiter',
        objective: 'Ký hợp đồng với Lead Prompt & Video Automation Engineer',
        action: 'ProposeHire',
        cost_minor: 220000,
        expected_revenue_minor: 600000,
        risk: 'High',
        confidence_bps: 7400,
        evidence: ['Nhu cầu nhân sự: 1 vị trí', 'Doanh thu vượt chi phí'],
        rationale: 'Mở rộng gấp 3 lần sản lượng video AI với chi phí biên thấp hơn tuyển dụng agency.',
        reversible: false,
        timestamp: new Date().toISOString(),
      },
      decision: 'EscalateToHuman',
      reason: 'Hành động trọng yếu (Material Action: Chi phí > $1,000 và tính chất Bất biến). Chuyển quyền phê duyệt cho Operator.',
      evaluatedAt: new Date().toISOString(),
      executed: false,
    },
  ]);

  const [receipts, setReceipts] = useState<ExecutionReceipt[]>([]);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [employees, setEmployees] = useState<{ id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[]>([]);
  const [experiments, setExperiments] = useState<{ id: string; name: string; budget_minor: number; startCycle: number; status: string; roi_bps: number }[]>([]);

  // Fetch initial state from server
  const loadState = async () => {
    try {
      const res = await fetch('/api/state');
      if (res.ok) {
        const data = await res.json();
        setSnapshot(data.snapshot);
        setLedger(data.ledger);
        setReceipts(data.receipts);
        setEmployees(data.employees);
        setExperiments(data.activeExperiments);
        setHasGeminiKey(data.hasGeminiKey);
      }
    } catch (err) {
      console.warn('Backend fetch note, using client state:', err);
    }
  };

  useEffect(() => {
    loadState();
  }, []);

  const triggerToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 4000);
  };

  // Run a complete autonomous cycle
  const handleRunCycle = async () => {
    setIsRunningCycle(true);
    try {
      const res = await fetch('/api/run-cycle', {
        method: 'POST',
      });
      if (res.ok) {
        const data = await res.json();
        setSnapshot(data.snapshot);
        setProposals(data.proposals);
        setReceipts((prev) => [...data.receipts, ...prev]);
        triggerToast(`Chu kỳ #${data.cycleNumber} hoàn tất thành công! Đã hạch toán sổ cái kép.`);
        loadState();
      }
    } catch (err) {
      console.error('Cycle run error:', err);
      triggerToast('Lỗi thực thi chu kỳ. Đã ghi log hệ thống.');
    } finally {
      setIsRunningCycle(false);
    }
  };

  // Human Operator Override
  const handleOverride = async (proposalId: string, decision: 'Approve' | 'Reject') => {
    try {
      const res = await fetch('/api/governor-override', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ proposalId, decision }),
      });
      if (res.ok) {
        setProposals((prev) =>
          prev.map((item) => {
            if (item.proposal.id === proposalId) {
              return {
                ...item,
                decision,
                reason: `Đã được Operator can thiệp trực tiếp: ${decision === 'Approve' ? 'Chấp thuận' : 'Bác bỏ'}.`,
                executed: decision === 'Approve',
              };
            }
            return item;
          })
        );
        triggerToast(`Quyết định của Operator đã được ghi nhận: ${decision}`);
        loadState();
      }
    } catch (err) {
      console.error('Override error:', err);
    }
  };

  // Chaos shock handler
  const handleApplyShock = async (shockType: string) => {
    try {
      const res = await fetch('/api/chaos-shock', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ shockType }),
      });
      if (res.ok) {
        const data = await res.json();
        setSnapshot(data.snapshot);
        triggerToast(`Đã inject biến cố "${shockType}". Trạng thái công ty cập nhật: ${data.snapshot.status}!`);
        loadState();
      }
    } catch (err) {
      console.error('Shock error:', err);
    }
  };

  // Content studio publishing action
  const handlePublishContent = (title: string, costMinor: number) => {
    triggerToast(`Đã đưa kịch bản "${title.substring(0, 30)}..." vào hàng đợi xuất bản media.`);
    handleRunCycle();
  };

  const navItems = [
    { id: 'audit', label: '1. Review Khắc Khe Repo', icon: ShieldAlert, highlight: true },
    { id: 'cycles', label: '2. Chu Kỳ Tự Trị (Cycles)', icon: RotateCw },
    { id: 'war-room', label: '3. Phòng Tranh Luận (War Room)', icon: Users },
    { id: 'agents', label: '4. Trụ Sở 9 Agents', icon: Bot },
    { id: 'ledger', label: '5. Sổ Cái Kép (Ledger)', icon: Landmark },
    { id: 'media', label: '6. AI Media & Affiliate Studio', icon: Video },
    { id: 'chaos', label: '7. Giả Lập Khủng Hoảng (Chaos)', icon: AlertOctagon },
  ];

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-indigo-500 selection:text-white">
      {/* Top Header */}
      <Header
        snapshot={snapshot}
        onRunCycle={handleRunCycle}
        isRunningCycle={isRunningCycle}
        activeTab={activeTab}
        setActiveTab={setActiveTab as any}
        hasGeminiKey={hasGeminiKey}
      />

      {/* Navigation Sub-Header */}
      <div className="bg-slate-900/60 border-b border-slate-800/80 sticky top-[69px] z-40 backdrop-blur-md px-4 lg:px-8">
        <div className="max-w-7xl mx-auto flex items-center gap-1 overflow-x-auto py-2 scrollbar-none">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => setActiveTab(item.id as any)}
                className={`flex items-center gap-2 px-3.5 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                  isActive
                    ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/20'
                    : 'text-slate-400 hover:text-white hover:bg-slate-800/60'
                }`}
              >
                <Icon className="w-3.5 h-3.5" />
                <span>{item.label}</span>
                {item.highlight && !isActive && (
                  <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse"></span>
                )}
              </button>
            );
          })}
        </div>
      </div>

      {/* Main Content Area */}
      <main className="flex-1 max-w-7xl w-full mx-auto p-4 md:p-8">
        {activeTab === 'audit' && <ReviewTab />}
        {activeTab === 'cycles' && (
          <CycleRunnerTab
            snapshot={snapshot}
            recentProposals={proposals}
            recentReceipts={receipts}
            onRunCycle={handleRunCycle}
            isRunningCycle={isRunningCycle}
            onOverride={handleOverride}
          />
        )}
        {activeTab === 'war-room' && <WarRoomTab snapshot={snapshot} />}
        {activeTab === 'agents' && <AgentRosterTab snapshot={snapshot} />}
        {activeTab === 'ledger' && (
          <LedgerTab
            snapshot={snapshot}
            ledger={ledger}
            employees={employees}
            experiments={experiments}
          />
        )}
        {activeTab === 'media' && (
          <MediaStudioTab
            snapshot={snapshot}
            onPublishToCycle={handlePublishContent}
          />
        )}
        {activeTab === 'chaos' && (
          <ChaosSimulatorTab
            snapshot={snapshot}
            onApplyShock={handleApplyShock}
          />
        )}
      </main>

      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed bottom-6 right-6 z-50 bg-slate-900 border border-indigo-500/40 text-white px-4 py-3 rounded-xl shadow-2xl flex items-center gap-2.5 text-xs md:text-sm animate-bounce">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{toastMessage}</span>
        </div>
      )}
    </div>
  );
}
