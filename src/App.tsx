import React, { useState, useEffect } from 'react';
import { 
  CompanySnapshot, 
  LedgerEntry, 
  CustomAgent, 
  CandidateProfile,
  OfficeActivityEvent,
  CompanyPnL,
  ClientContract
} from './types/company';
import { Header } from './components/Header';
import { ClientContractsTab } from './components/ClientContractsTab';
import { CreateContractTab } from './components/CreateContractTab';
import { CompanyCapabilitiesTab } from './components/CompanyCapabilitiesTab';
import { ManageFinances } from './components/ManageFinances';

import { 
  FileText, 
  Sparkles, 
  Building2, 
  Wallet,
  CheckCircle2
} from 'lucide-react';

export default function App() {
  const [activeTab, setActiveTab] = useState<'contracts' | 'order' | 'capabilities' | 'finances'>('contracts');
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
  const [officeActivities, setOfficeActivities] = useState<OfficeActivityEvent[]>([]);
  const [clientContracts, setClientContracts] = useState<ClientContract[]>([]);

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
        if (data.officeActivities) setOfficeActivities(data.officeActivities);
        if (data.clientContracts) setClientContracts(data.clientContracts);
        setDataMode(data.dataMode === 'SIMULATION' ? 'SIMULATION' : 'UNKNOWN');
      }
    } catch (err) {
      console.warn('Backend load note:', err);
    }
  };

  useEffect(() => {
    loadState();
  }, []);

  // 24/7 Autonomous Background Engine (Auto-updates contract & office activities)
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
        }
      } catch (e) {
        // silent background sync
      }
    }, 8000);

    return () => clearInterval(interval);
  }, []);

  // Client creates a new contract
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
        triggerToast(`Hợp đồng "${orderData.title}" đã được AI tiếp nhận & hoàn tất thành công!`);
        loadState();
        return { success: true, message: data.message };
      } else {
        return { success: false, message: data.reason || 'Lỗi xử lý hợp đồng' };
      }
    } catch (err: any) {
      return { success: false, message: err.message || 'Lỗi kết nối máy chủ' };
    }
  };

  // Client accepts and rates deliverables
  const handleAcceptContract = async (contractId: string, rating: number, feedback: string) => {
    try {
      const res = await fetch(`/api/contracts/${contractId}/accept`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ rating, feedback }),
      });
      if (res.ok) {
        triggerToast(`Đã nghiệm thu hợp đồng thành công với đánh giá ${rating} sao!`);
        loadState();
      }
    } catch (err) {
      console.error(err);
    }
  };

  const activeContractsCount = clientContracts.filter(
    (c) => c.status !== 'Delivered' && c.status !== 'Completed'
  ).length;

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-blue-600 selection:text-white">
      {/* Simulation Info Bar */}
      {dataMode === 'SIMULATION' && (
        <div className="bg-slate-900 border-b border-slate-800 px-4 py-1.5 text-center text-[11px] font-medium text-slate-400">
          CỔNG THUÊ KHOÁN KHÁCH HÀNG — Doanh nghiệp AI vận hành hoàn toàn tự động 24/7 (Black-Box Autonomous). Khách hàng giao việc dưới dạng hợp đồng và nhận bàn giao thành phẩm.
        </div>
      )}

      {/* Sticky Top Header */}
      <div className="sticky top-0 z-50 bg-slate-950/95 backdrop-blur-md border-b border-slate-800 shadow-sm">
        <Header
          snapshot={snapshot}
          activeContractsCount={activeContractsCount}
          onNavigateToOrder={() => setActiveTab('order')}
        />

        {/* Clean Client Navigation Bar */}
        <nav className="border-t border-slate-800/80 bg-slate-900/40 px-4 lg:px-8">
          <div className="max-w-5xl mx-auto flex items-center justify-start gap-2 py-2 overflow-x-auto scrollbar-none">
            <button
              onClick={() => setActiveTab('contracts')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'contracts'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <FileText className="w-3.5 h-3.5" />
              <span>Hợp Đồng Của Tôi ({clientContracts.length})</span>
            </button>

            <button
              onClick={() => setActiveTab('order')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'order'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Đặt Hàng Thuê Khoán</span>
            </button>

            <button
              onClick={() => setActiveTab('capabilities')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === 'capabilities'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
              }`}
            >
              <Building2 className="w-3.5 h-3.5" />
              <span>Hồ Sơ Năng Lực Doanh Nghiệp ({customAgents.length} AI)</span>
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
              <span>Đối Soát Hóa Đơn & Sổ Cái</span>
            </button>
          </div>
        </nav>
      </div>

      {/* Main Content */}
      <main className="flex-1 max-w-5xl w-full mx-auto p-4 md:p-5">
        {activeTab === 'contracts' && (
          <ClientContractsTab
            contracts={clientContracts}
            onAcceptContract={handleAcceptContract}
            onNavigateToOrder={() => setActiveTab('order')}
          />
        )}

        {activeTab === 'order' && (
          <CreateContractTab
            onSubmitContract={handleOrderContract}
            onNavigateToContracts={() => setActiveTab('contracts')}
          />
        )}

        {activeTab === 'capabilities' && (
          <CompanyCapabilitiesTab
            snapshot={snapshot}
            agents={customAgents}
            officeActivities={officeActivities}
          />
        )}

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
