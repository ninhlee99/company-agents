import React, { useState, useEffect } from 'react';
import { 
  CompanySnapshot, 
  LedgerEntry, 
  CustomAgent, 
  CandidateProfile,
  OfficeActivityEvent,
  CompanyPnL,
  ClientContract,
  CycleTrendPoint,
  AutonomousSettings,
  SocialChannel
} from './types/company';
import { Header } from './components/Header';
import { BasicDashboard } from './components/BasicDashboard';
import { ManageAgents } from './components/ManageAgents';
import { ChannelsAndLivestreamTab } from './components/ChannelsAndLivestreamTab';
import { AutonomousPipelineTab } from './components/AutonomousPipelineTab';
import { MediaStudioTab } from './components/MediaStudioTab';
import { ClientContractsTab } from './components/ClientContractsTab';
import { CreateContractTab } from './components/CreateContractTab';
import { ManageFinances } from './components/ManageFinances';

import { 
  LayoutDashboard, 
  Users, 
  Radio, 
  Sparkles, 
  Briefcase, 
  Wallet,
  CheckCircle2,
  Video
} from 'lucide-react';

export default function App() {
  const [activeTab, setActiveTab] = useState<'overview' | 'workforce' | 'livestream' | 'pipeline' | 'contracts' | 'order' | 'finances'>('overview');
  const [pipelineSubTab, setPipelineSubTab] = useState<'flow' | 'studio'>('flow');
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [dataMode, setDataMode] = useState<'SIMULATION' | 'UNKNOWN'>('UNKNOWN');

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

  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [employees, setEmployees] = useState<{ id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[]>([]);
  const [customAgents, setCustomAgents] = useState<CustomAgent[]>([]);
  const [candidatePool, setCandidatePool] = useState<CandidateProfile[]>([]);
  const [officeActivities, setOfficeActivities] = useState<OfficeActivityEvent[]>([]);
  const [clientContracts, setClientContracts] = useState<ClientContract[]>([]);
  const [channels, setChannels] = useState<SocialChannel[]>([]);
  const [pnl, setPnl] = useState<CompanyPnL | undefined>(undefined);
  const [cycleHistory, setCycleHistory] = useState<CycleTrendPoint[]>([]);
  const [autonomousSettings, setAutonomousSettings] = useState<AutonomousSettings>({
    isAutoPilotActive: true,
    intervalSeconds: 10,
    autoHireWhenBacklogHigh: true,
    autoReinvestProfitPct: 25,
    maxSpendPerAutoCycleMinor: 50000,
  });

  const triggerToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => {
      setToastMessage(null);
    }, 4000);
  };

  const loadState = async () => {
    try {
      const res = await fetch('/api/state');
      if (res.ok) {
        const data = await res.json();
        setSnapshot(data.snapshot);
        setLedger(data.ledger || []);
        setEmployees(data.employees || []);
        if (data.customAgents) setCustomAgents(data.customAgents);
        if (data.candidatePool) setCandidatePool(data.candidatePool);
        if (data.officeActivities) setOfficeActivities(data.officeActivities);
        if (data.clientContracts) setClientContracts(data.clientContracts);
        if (data.channels) setChannels(data.channels);
        if (data.pnl) setPnl(data.pnl);
        if (data.cycleHistory) setCycleHistory(data.cycleHistory);
        if (data.autonomousSettings) setAutonomousSettings(data.autonomousSettings);
        setDataMode(data.dataMode === 'SIMULATION' ? 'SIMULATION' : 'UNKNOWN');
      }
    } catch (err) {
      console.warn('Backend load note:', err);
    }
  };

  useEffect(() => {
    loadState();
  }, []);

  // 24/7 Autonomous Background Engine Sync
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const res = await fetch('/api/state');
        if (res.ok) {
          const data = await res.json();
          setSnapshot(data.snapshot);
          if (data.clientContracts) setClientContracts(data.clientContracts);
          if (data.officeActivities) setOfficeActivities(data.officeActivities);
          if (data.customAgents) setCustomAgents(data.customAgents);
          if (data.channels) setChannels(data.channels);
          if (data.pnl) setPnl(data.pnl);
        }
      } catch (e) {
        // silent background sync
      }
    }, 8000);

    return () => clearInterval(interval);
  }, []);

  // Run autonomous multi-agent pipeline
  const handleRunPipeline = async (topic: string) => {
    try {
      const res = await fetch('/api/run-pipeline', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ topic }),
      });
      const data = await res.json();
      if (res.ok) {
        triggerToast(`Dây chuyền hoàn tất cho "${topic}"!`);
        loadState();
        return { success: true, steps: data.steps, simulatedRevenueGainMinor: data.simulatedRevenueGainMinor };
      }
      return { success: false, steps: [], simulatedRevenueGainMinor: 0 };
    } catch (err) {
      return { success: false, steps: [], simulatedRevenueGainMinor: 0 };
    }
  };

  // Create new contract / deal
  const handleOrderContract = async (orderData: {
    clientName: string;
    clientEmail: string;
    title: string;
    category: 'VideoMarketing' | 'Copywriting' | 'MediaDesign' | 'MarketIntelligence' | 'FullCampaign';
    requirements: string;
    budgetMinor: number;
  }) => {
    try {
      const res = await fetch('/api/contracts/order', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(orderData),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        triggerToast(`Hợp đồng "${orderData.title}" đã được tiếp nhận & hoàn tất tự động!`);
        loadState();
        return { success: true, message: data.message };
      } else {
        return { success: false, message: data.reason || 'Lỗi xử lý hợp đồng' };
      }
    } catch (err: any) {
      return { success: false, message: err.message || 'Lỗi kết nối máy chủ' };
    }
  };

  // Client accept contract
  const handleAcceptContract = async (contractId: string, rating: number, feedback: string) => {
    try {
      const res = await fetch(`/api/contracts/${contractId}/accept`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ rating, feedback }),
      });
      if (res.ok) {
        triggerToast(`Đã nghiệm thu hợp đồng với đánh giá ${rating} sao!`);
        loadState();
      }
    } catch (err) {
      console.error(err);
    }
  };

  // AI Hire Agent Handler
  const handleHireAgent = async (data: { name: string; role: string; department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech'; description: string; salary_minor: number }) => {
    try {
      const res = await fetch('/api/hire-custom-agent', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });
      const resData = await res.json();
      if (res.ok) {
        triggerToast(`Đã bổ sung chuyên gia ${data.name} vào đội ngũ!`);
        loadState();
        return { success: true };
      }
      return { success: false, reason: resData.reason };
    } catch (err: any) {
      return { success: false, reason: err.message };
    }
  };

  // Interview candidate
  const handleInterviewCandidate = async (candidateId: string) => {
    const res = await fetch('/api/candidates/interview', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ candidateId }),
    });
    return res.json();
  };

  // Hire candidate
  const handleHireCandidate = async (candidateId: string) => {
    const res = await fetch('/api/candidates/hire', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ candidateId }),
    });
    const data = await res.json();
    if (res.ok) {
      triggerToast(`Đã tuyển dụng ${data.agent.name} thành công!`);
      loadState();
    }
    return data;
  };

  // Start livestream session
  const handleStartStream = async (channelId: string, title: string) => {
    try {
      const res = await fetch('/api/channels/stream/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channelId, title }),
      });
      if (res.ok) {
        triggerToast('Đã kích hoạt phiên livestream 24/7 với Host AI thành công!');
        loadState();
      }
    } catch (e) {
      console.error(e);
    }
  };

  // Pin product in live stream
  const handlePinProduct = async (channelId: string, productTitle: string, priceMinor: number) => {
    try {
      const res = await fetch('/api/channels/stream/pin-product', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channelId, productTitle, priceMinor }),
      });
      if (res.ok) {
        triggerToast(`Đã ghim sản phẩm "${productTitle}" vào phiên live!`);
        loadState();
      }
    } catch (e) {
      console.error(e);
    }
  };

  // Stop stream
  const handleStopStream = async (channelId: string) => {
    try {
      const res = await fetch('/api/channels/stream/stop', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channelId }),
      });
      if (res.ok) {
        triggerToast('Đã kết thúc phiên live và đối soát doanh thu vào kho bạc!');
        loadState();
      }
    } catch (e) {
      console.error(e);
    }
  };

  const activeContractsCount = clientContracts.filter(
    (c) => c.status !== 'Delivered' && c.status !== 'Completed'
  ).length;

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-blue-600 selection:text-white">
      {/* Sticky Top Header */}
      <div className="sticky top-0 z-50 bg-slate-950/95 backdrop-blur-md border-b border-slate-800 shadow-sm">
        <Header
          snapshot={snapshot}
          activeContractsCount={activeContractsCount}
          onNavigateToNewContract={() => setActiveTab('order')}
        />

        {/* Clean Executive Navigation Bar */}
        <nav className="border-t border-slate-800/80 bg-slate-900/40 px-4 lg:px-8">
          <div className="max-w-6xl mx-auto flex items-center justify-start gap-1.5 py-2 overflow-x-auto scrollbar-none">
            <button
              onClick={() => setActiveTab('overview')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'overview'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <LayoutDashboard className="w-3.5 h-3.5" />
              <span>Tổng Quan Điều Hành</span>
            </button>

            <button
              onClick={() => setActiveTab('workforce')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'workforce'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <Users className="w-3.5 h-3.5" />
              <span>Đội Ngũ AI &amp; Nhân Sự ({customAgents.length})</span>
            </button>

            <button
              onClick={() => setActiveTab('livestream')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'livestream'
                  ? 'bg-pink-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <Radio className="w-3.5 h-3.5 text-pink-400" />
              <span>Kênh &amp; Livestream 24/7 ({channels.length})</span>
            </button>

            <button
              onClick={() => setActiveTab('pipeline')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'pipeline'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Quy Trình &amp; Dây Chuyền</span>
            </button>

            <button
              onClick={() => setActiveTab('contracts')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'contracts' || activeTab === 'order'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <Briefcase className="w-3.5 h-3.5" />
              <span>Kinh Doanh &amp; Hợp Đồng ({clientContracts.length})</span>
            </button>

            <button
              onClick={() => setActiveTab('finances')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'finances'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <Wallet className="w-3.5 h-3.5" />
              <span>Tài Chính &amp; Sổ Cái</span>
            </button>
          </div>
        </nav>
      </div>

      {/* Main Content Viewport */}
      <main className="flex-1 max-w-6xl w-full mx-auto p-4 md:p-5">
        {/* Tab 1: Executive Overview */}
        {activeTab === 'overview' && (
          <BasicDashboard
            snapshot={snapshot}
            recentProposals={[]}
            cycleHistory={cycleHistory}
            agents={customAgents}
            officeActivities={officeActivities}
            pnl={pnl}
            autonomousSettings={autonomousSettings}
            onRunCycle={() => {}}
            isRunningCycle={false}
            onOverride={() => {}}
            onNavigate={(view) => {
              if (view === 'agents') setActiveTab('workforce');
              else if (view === 'pipeline' || view === 'studio') setActiveTab('pipeline');
              else if (view === 'finances') setActiveTab('finances');
            }}
          />
        )}

        {/* Tab 2: Workforce & AI Agents */}
        {activeTab === 'workforce' && (
          <ManageAgents
            snapshot={snapshot}
            agents={customAgents}
            candidates={candidatePool}
            onHireAgent={handleHireAgent}
            onInterviewCandidate={handleInterviewCandidate}
            onHireCandidate={handleHireCandidate}
          />
        )}

        {/* Tab 3: Livestream & Channels */}
        {activeTab === 'livestream' && (
          <ChannelsAndLivestreamTab
            channels={channels}
            onStartStream={handleStartStream}
            onPinProduct={handlePinProduct}
            onStopStream={handleStopStream}
          />
        )}

        {/* Tab 4: Workflows & Pipelines */}
        {activeTab === 'pipeline' && (
          <div className="space-y-4">
            <div className="flex items-center gap-2 border-b border-slate-800 pb-2">
              <button
                onClick={() => setPipelineSubTab('flow')}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                  pipelineSubTab === 'flow' ? 'bg-slate-800 text-white font-semibold' : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                <Sparkles className="w-3.5 h-3.5 text-blue-400" />
                <span>Dây Chuyền Tự Động 4 Khâu</span>
              </button>

              <button
                onClick={() => setPipelineSubTab('studio')}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                  pipelineSubTab === 'studio' ? 'bg-slate-800 text-white font-semibold' : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                <Video className="w-3.5 h-3.5 text-purple-400" />
                <span>Xưởng Sáng Tạo Media (5-in-1)</span>
              </button>
            </div>

            {pipelineSubTab === 'flow' ? (
              <AutonomousPipelineTab
                snapshot={snapshot}
                onRunPipeline={handleRunPipeline}
              />
            ) : (
              <MediaStudioTab
                snapshot={snapshot}
                onPublishToCycle={(title) => handleRunPipeline(title)}
              />
            )}
          </div>
        )}

        {/* Tab 5: Sales & Client Contracts */}
        {activeTab === 'contracts' && (
          <ClientContractsTab
            contracts={clientContracts}
            onAcceptContract={handleAcceptContract}
            onNavigateToOrder={() => setActiveTab('order')}
          />
        )}

        {/* Sub-view: Create Contract / Deal */}
        {activeTab === 'order' && (
          <CreateContractTab
            onSubmitContract={handleOrderContract}
            onNavigateToContracts={() => setActiveTab('contracts')}
          />
        )}

        {/* Tab 6: Finances & P&L Ledger */}
        {activeTab === 'finances' && (
          <ManageFinances
            snapshot={snapshot}
            ledger={ledger}
            employees={employees}
            agents={customAgents}
          />
        )}
      </main>

      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed bottom-5 right-5 z-50 bg-slate-900 border border-blue-500/50 text-white px-3.5 py-2.5 rounded-xl shadow-2xl flex items-center gap-2 text-xs animate-bounce">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{toastMessage}</span>
        </div>
      )}
    </div>
  );
}
