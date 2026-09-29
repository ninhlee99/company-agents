import React, { useState, useEffect } from 'react';
import { CompanySnapshot, GovernedProposal, ExecutionReceipt, LedgerEntry, CustomAgent, CycleTrendPoint, AutoAuditReport, DepartmentBudgetPoint, SystemAlert, CompanyKPIs } from './types/company';
import { Header } from './components/Header';
import { BasicDashboard } from './components/BasicDashboard';
import { ManageAgents } from './components/ManageAgents';
import { ManageFinances } from './components/ManageFinances';
import { AutonomousPipelineTab } from './components/AutonomousPipelineTab';

// Pro Mode Components
import { ReviewTab } from './components/ReviewTab';
import { CycleRunnerTab } from './components/CycleRunnerTab';
import { WarRoomTab } from './components/WarRoomTab';
import { LedgerTab } from './components/LedgerTab';
import { ChaosSimulatorTab } from './components/ChaosSimulatorTab';

import { 
  LayoutDashboard, 
  Users, 
  Wallet, 
  Sparkles, 
  ShieldAlert, 
  RotateCw, 
  CheckCircle2,
  AlertOctagon
} from 'lucide-react';

export default function App() {
  const [uiMode, setUiMode] = useState<'basic' | 'pro'>('basic');
  
  // Basic Nav Tabs
  const [basicTab, setBasicTab] = useState<'dashboard' | 'agents' | 'finances' | 'pipeline'>('dashboard');

  // Pro Nav Tabs
  const [proTab, setProTab] = useState<'audit' | 'cycles' | 'war-room' | 'ledger' | 'chaos'>('audit');

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

  const [proposals, setProposals] = useState<GovernedProposal[]>([]);
  const [receipts, setReceipts] = useState<ExecutionReceipt[]>([]);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [employees, setEmployees] = useState<{ id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[]>([]);
  const [customAgents, setCustomAgents] = useState<CustomAgent[]>([
    { id: 'agent-gov', name: 'Governor', role: 'Hiến Pháp & Quỹ Tiền', department: 'Leadership', description: 'Phủ quyết chi tiêu nguy hiểm, chống phá sản', salary_minor: 0, tasksCompleted: 42, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-ceo', name: 'CEO', role: 'Tổng Giám Đốc', department: 'Leadership', description: 'Chiến lược tăng trưởng & phân bổ nguồn vốn', salary_minor: 0, tasksCompleted: 35, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-cfo', name: 'CFO', role: 'Giám Đốc Tài Chính', department: 'Leadership', description: 'Kiểm toán kho bạc và cắt giảm chi tiêu', salary_minor: 0, tasksCompleted: 38, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-coo', name: 'COO', role: 'Giám Đốc Vận Hành', department: 'Ops', description: 'Điều phối hàng đợi và tiến độ công việc', salary_minor: 0, tasksCompleted: 50, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-growth', name: 'Growth Lead', role: 'Kinh Doanh & Traffic', department: 'Growth', description: 'Tìm ngách sản phẩm hoa hồng cao', salary_minor: 0, tasksCompleted: 62, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-content', name: 'Content Lead', role: 'Sáng Tạo Nội Dung', department: 'Growth', description: 'Kịch bản video short-form bán hàng', salary_minor: 0, tasksCompleted: 78, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-recruiter', name: 'Recruiter', role: 'Tuyển Dụng', department: 'Ops', description: 'Đề xuất bổ sung vị trí mới khi có lãi', salary_minor: 0, tasksCompleted: 14, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-analyst', name: 'Analyst', role: 'Phân Tích Dữ Liệu', department: 'Ops', description: 'Đối soát số liệu và tính toán hoa hồng', salary_minor: 0, tasksCompleted: 45, status: 'Active', hiredAtCycle: 1 },
    { id: 'agent-experiment', name: 'Experimenter', role: 'Nghiên Cứu A/B Test', department: 'Growth', description: 'Thử nghiệm mẫu kịch bản và thị trường', salary_minor: 0, tasksCompleted: 29, status: 'Active', hiredAtCycle: 1 },
  ]);

  const [cycleHistory, setCycleHistory] = useState<CycleTrendPoint[]>([]);
  const [departmentBudgets, setDepartmentBudgets] = useState<DepartmentBudgetPoint[]>([]);
  const [auditReports, setAuditReports] = useState<AutoAuditReport[]>([]);
  const [systemAlerts, setSystemAlerts] = useState<SystemAlert[]>([]);

  const loadState = async () => {
    try {
      const res = await fetch('/api/state');
      if (res.ok) {
        const data = await res.json();
        setSnapshot(data.snapshot);
        setLedger(data.ledger || []);
        setReceipts(data.receipts || []);
        setEmployees(data.employees || []);
        if (data.customAgents && data.customAgents.length > 0) {
          setCustomAgents(data.customAgents);
        }
        if (data.cycleHistory) {
          setCycleHistory(data.cycleHistory);
        }
        if (data.departmentBudgets) {
          setDepartmentBudgets(data.departmentBudgets);
        }
        if (data.systemAlerts) {
          setSystemAlerts(data.systemAlerts);
        }
        if (data.auditReports) {
          setAuditReports(data.auditReports);
        }
        setHasGeminiKey(data.hasGeminiKey);
      }
    } catch (err) {
      console.warn('Backend load note:', err);
    }
  };

  useEffect(() => {
    loadState();
  }, []);

  const triggerToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  const handleRunCycle = async () => {
    setIsRunningCycle(true);
    try {
      const res = await fetch('/api/run-cycle', { method: 'POST' });
      if (res.ok) {
        const data = await res.json();
        setSnapshot(data.snapshot);
        setProposals(data.proposals);
        setReceipts((prev) => [...data.receipts, ...prev]);
        if (data.auditReports) {
          setAuditReports(data.auditReports);
        }
        if (data.auditReport) {
          triggerToast(`Kiểm toán định kỳ Kỳ #${data.auditReport.cycleMilestone}: Doanh thu ${data.auditReport.variancePercent >= 0 ? '+' : ''}${data.auditReport.variancePercent}% vs ngân sách!`);
        } else {
          triggerToast(`Chu kỳ #${data.cycleNumber} đã xong! AI đã ra quyết định.`);
        }
        loadState();
      }
    } catch (err) {
      console.error(err);
      triggerToast('Đã ghi nhận chu kỳ.');
    } finally {
      setIsRunningCycle(false);
    }
  };

  const handleTriggerAudit = async () => {
    try {
      const res = await fetch('/api/trigger-audit', { method: 'POST' });
      if (res.ok) {
        const data = await res.json();
        if (data.reports) setAuditReports(data.reports);
        triggerToast('Báo cáo kiểm toán 10 chu kỳ đã hoàn tất!');
      }
    } catch (err) {
      console.error(err);
    }
  };

  const handleResolveAlert = async (alertId: string) => {
    try {
      const res = await fetch('/api/system-alerts/resolve', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ alertId }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.alerts) setSystemAlerts(data.alerts);
        triggerToast('Đã xác nhận xử lý cảnh báo thành công!');
      }
    } catch (err) {
      console.error(err);
    }
  };

  const handleTriggerAlert = async (alertData: Partial<SystemAlert>) => {
    try {
      const res = await fetch('/api/system-alerts/trigger', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(alertData),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.alerts) setSystemAlerts(data.alerts);
        triggerToast(`Governor phát hiện cảnh báo mới: ${alertData.title || 'Biến động hệ thống'}`);
      }
    } catch (err) {
      console.error(err);
    }
  };

  const handleOverride = async (proposalId: string, decision: 'Approve' | 'Reject') => {
    try {
      const res = await fetch('/api/governor-override', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ proposalId, decision }),
      });
      if (res.ok) {
        setProposals((prev) =>
          prev.map((item) =>
            item.proposal.id === proposalId
              ? { ...item, decision, executed: decision === 'Approve' }
              : item
          )
        );
        triggerToast(`Quyết định: ${decision === 'Approve' ? 'Duyệt thành công' : 'Đã từ chối'}`);
        loadState();
      }
    } catch (err) {
      console.error(err);
    }
  };

  // Hire dynamic custom agent
  const handleHireAgent = async (data: {
    name: string;
    role: string;
    department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech';
    description: string;
    salary_minor: number;
  }) => {
    try {
      const res = await fetch('/api/hire-custom-agent', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });
      const result = await res.json();
      if (result.success) {
        setCustomAgents((prev) => [...prev, result.agent]);
        setSnapshot(result.snapshot);
        triggerToast(result.message);
        loadState();
        return { success: true };
      }
      return { success: false, reason: result.reason };
    } catch (err) {
      return { success: false, reason: 'Lỗi kết nối tuyển dụng.' };
    }
  };

  const handleToggleAgentStatus = async (agentId: string) => {
    try {
      const res = await fetch('/api/toggle-agent-status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agentId }),
      });
      if (res.ok) {
        setCustomAgents((prev) =>
          prev.map((a) => (a.id === agentId ? { ...a, status: a.status === 'Active' ? 'Paused' : 'Active' } : a))
        );
      }
    } catch (err) {
      console.error(err);
    }
  };

  // Run autonomous multi-agent pipeline
  const handleRunPipeline = async (topic: string) => {
    try {
      const res = await fetch('/api/run-pipeline', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ topic }),
      });
      const data = await res.json();
      if (data.success) {
        setSnapshot(data.snapshot);
        triggerToast(`Dây chuyền hoàn tất! Thu về +$${(data.revenueGainMinor / 100).toFixed(2)}.`);
        loadState();
        return { success: true, steps: data.steps, revenueGainMinor: data.revenueGainMinor };
      }
      return { success: false, steps: [], revenueGainMinor: 0 };
    } catch (err) {
      return { success: false, steps: [], revenueGainMinor: 0 };
    }
  };

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
        triggerToast(`Đã áp dụng biến cố "${shockType}". Trạng thái: ${data.snapshot.status}`);
        loadState();
      }
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-indigo-500 selection:text-white">
      {/* Sticky Master Header: Keeps Header & Navigation Bar Fixed at Top */}
      <div className="sticky top-0 z-50 bg-slate-950/95 backdrop-blur-md border-b border-slate-800 shadow-md">
        <Header
          snapshot={snapshot}
          onRunCycle={handleRunCycle}
          isRunningCycle={isRunningCycle}
          uiMode={uiMode}
          setUiMode={setUiMode}
          hasGeminiKey={hasGeminiKey}
        />

        {/* Clean Navigation Bar (Sticky with Header) */}
        <nav className="border-t border-slate-800/80 bg-slate-900/60 px-4 lg:px-8">
          <div className="max-w-5xl mx-auto flex items-center justify-between gap-2 py-1.5">
            {uiMode === 'basic' ? (
              <div className="flex items-center gap-1 overflow-x-auto scrollbar-none w-full">
                <button
                  onClick={() => setBasicTab('dashboard')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    basicTab === 'dashboard'
                      ? 'bg-indigo-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/60'
                  }`}
                >
                  <LayoutDashboard className="w-3.5 h-3.5" />
                  <span>1. Dashboard</span>
                </button>

                <button
                  onClick={() => setBasicTab('agents')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    basicTab === 'agents'
                      ? 'bg-indigo-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/60'
                  }`}
                >
                  <Users className="w-3.5 h-3.5" />
                  <span>2. Nhân Sự &amp; Tuyển Dụng</span>
                </button>

                <button
                  onClick={() => setBasicTab('pipeline')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    basicTab === 'pipeline'
                      ? 'bg-indigo-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/60'
                  }`}
                >
                  <Sparkles className="w-3.5 h-3.5 text-purple-400" />
                  <span>3. Dây Chuyền Bán Hàng</span>
                </button>

                <button
                  onClick={() => setBasicTab('finances')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    basicTab === 'finances'
                      ? 'bg-indigo-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/60'
                  }`}
                >
                  <Wallet className="w-3.5 h-3.5" />
                  <span>4. Ví Tiền &amp; Thu Chi</span>
                </button>
              </div>
            ) : (
              <div className="flex items-center gap-1 overflow-x-auto scrollbar-none w-full">
                <button
                  onClick={() => setProTab('audit')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    proTab === 'audit' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  <ShieldAlert className="w-3.5 h-3.5 text-rose-400" />
                  <span>Review Khắc Khe</span>
                </button>

                <button
                  onClick={() => setProTab('cycles')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    proTab === 'cycles' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  <RotateCw className="w-3.5 h-3.5" />
                  <span>Chu Kỳ Tự Trị</span>
                </button>

                <button
                  onClick={() => setProTab('war-room')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    proTab === 'war-room' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  <Users className="w-3.5 h-3.5" />
                  <span>War Room Tranh Luận</span>
                </button>

                <button
                  onClick={() => setProTab('ledger')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    proTab === 'ledger' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  <Wallet className="w-3.5 h-3.5" />
                  <span>Sổ Cái Kép</span>
                </button>

                <button
                  onClick={() => setProTab('chaos')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    proTab === 'chaos' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  <AlertOctagon className="w-3.5 h-3.5 text-amber-400" />
                  <span>Giả Lập Khủng Hoảng</span>
                </button>
              </div>
            )}
          </div>
        </nav>
      </div>

      {/* Main Content Area */}
      <main className="flex-1 max-w-5xl w-full mx-auto p-4 md:p-5">
        {uiMode === 'basic' ? (
          <>
            {basicTab === 'dashboard' && (
              <BasicDashboard
                snapshot={snapshot}
                recentProposals={proposals}
                cycleHistory={cycleHistory}
                departmentBudgets={departmentBudgets}
                auditReports={auditReports}
                systemAlerts={systemAlerts}
                onRunCycle={handleRunCycle}
                isRunningCycle={isRunningCycle}
                onOverride={handleOverride}
                onNavigate={(view) => setBasicTab(view)}
                onTriggerAudit={handleTriggerAudit}
                onResolveAlert={handleResolveAlert}
                onTriggerAlert={handleTriggerAlert}
              />
            )}
            {basicTab === 'agents' && (
              <ManageAgents
                snapshot={snapshot}
                agents={customAgents}
                onHireAgent={handleHireAgent}
                onToggleStatus={handleToggleAgentStatus}
              />
            )}
            {basicTab === 'pipeline' && (
              <AutonomousPipelineTab
                snapshot={snapshot}
                onRunPipeline={handleRunPipeline}
              />
            )}
            {basicTab === 'finances' && (
              <ManageFinances
                snapshot={snapshot}
                ledger={ledger}
                employees={employees}
              />
            )}
          </>
        ) : (
          <>
            {proTab === 'audit' && <ReviewTab />}
            {proTab === 'cycles' && (
              <CycleRunnerTab
                snapshot={snapshot}
                recentProposals={proposals}
                recentReceipts={receipts}
                onRunCycle={handleRunCycle}
                isRunningCycle={isRunningCycle}
                onOverride={handleOverride}
              />
            )}
            {proTab === 'war-room' && <WarRoomTab snapshot={snapshot} />}
            {proTab === 'ledger' && (
              <LedgerTab
                snapshot={snapshot}
                ledger={ledger}
                employees={employees}
                experiments={[]}
              />
            )}
            {proTab === 'chaos' && (
              <ChaosSimulatorTab
                snapshot={snapshot}
                onApplyShock={handleApplyShock}
              />
            )}
          </>
        )}
      </main>

      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed bottom-5 right-5 z-50 bg-slate-900 border border-indigo-500/50 text-white px-3.5 py-2.5 rounded-xl shadow-2xl flex items-center gap-2 text-xs animate-bounce">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{toastMessage}</span>
        </div>
      )}
    </div>
  );
}
