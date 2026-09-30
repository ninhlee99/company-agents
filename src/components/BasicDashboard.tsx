import React, { useState } from 'react';
import { CompanySnapshot, GovernedProposal, CycleTrendPoint, AutoAuditReport, DepartmentBudgetPoint, SystemAlert, CustomAgent, OfficeActivityEvent, CompanyPnL, AutonomousSettings } from '../types/company';
import { 
  TrendingUp, 
  TrendingDown, 
  Wallet, 
  Users, 
  ArrowRight, 
  ShieldCheck, 
  Activity,
  DollarSign,
  Video,
  Play,
  RotateCw,
  Clock,
  Sparkles
} from 'lucide-react';
import {
  ResponsiveContainer,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid
} from 'recharts';

interface BasicDashboardProps {
  snapshot: CompanySnapshot;
  recentProposals: GovernedProposal[];
  cycleHistory?: CycleTrendPoint[];
  departmentBudgets?: DepartmentBudgetPoint[];
  auditReports?: AutoAuditReport[];
  systemAlerts?: SystemAlert[];
  agents?: CustomAgent[];
  officeActivities?: OfficeActivityEvent[];
  pnl?: CompanyPnL;
  autonomousSettings?: AutonomousSettings;
  onOpenTraining?: (agentId?: string) => void;
  onRunCycle: () => void;
  isRunningCycle: boolean;
  onOverride: (proposalId: string, decision: 'Approve' | 'Reject') => void;
  onNavigate: (view: 'dashboard' | 'agents' | 'finances' | 'pipeline' | 'studio') => void;
  onTriggerAudit?: () => Promise<void>;
  onResolveAlert?: (alertId: string) => Promise<void>;
  onTriggerAlert?: (alertData: Partial<SystemAlert>) => Promise<void>;
  onToggleAutoPilot?: () => void;
}

export const BasicDashboard: React.FC<BasicDashboardProps> = ({
  snapshot,
  cycleHistory,
  agents = [],
  officeActivities = [],
  pnl,
  autonomousSettings,
  onRunCycle,
  isRunningCycle,
  onNavigate,
  onToggleAutoPilot,
}) => {
  const [chartPeriod, setChartPeriod] = useState<'all' | 'recent'>('all');

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const netMonthly = snapshot.revenue_minor - snapshot.expenses_minor;
  const isAuto = autonomousSettings?.isAutoPilotActive ?? false;

  // 10-cycle trend
  const trendData = cycleHistory && cycleHistory.length > 0 ? cycleHistory : [
    { cycle: 'Kỳ 5', revenue: 5200, expenses: 4900, cash: 38000 },
    { cycle: 'Kỳ 6', revenue: 5800, expenses: 5100, cash: 39500 },
    { cycle: 'Kỳ 7', revenue: 6400, expenses: 5300, cash: 41000 },
    { cycle: 'Kỳ 8', revenue: 6900, expenses: 5600, cash: 41800 },
    { cycle: 'Kỳ 9', revenue: 7500, expenses: 5700, cash: 43200 },
    { cycle: 'Kỳ 10', revenue: 8100, expenses: 5900, cash: 44600 },
    { cycle: 'Kỳ 11', revenue: 8600, expenses: 6000, cash: 45800 },
    { cycle: 'Kỳ 12', revenue: 9000, expenses: 6100, cash: 46900 },
    { cycle: 'Kỳ 13', revenue: 9200, expenses: 6150, cash: 47600 },
    { cycle: `Kỳ ${snapshot.cycle_count}`, revenue: Math.round(snapshot.revenue_minor / 100), expenses: Math.round(snapshot.expenses_minor / 100), cash: Math.round(snapshot.cash_minor / 100) },
  ];

  return (
    <div className="space-y-5 pb-12">
      {/* 4 Core Metric Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5">
        {/* Card 1: Treasury Cash */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
          <div className="flex items-center justify-between text-slate-400 mb-1.5">
            <span className="text-xs font-medium">Quỹ Tiền Mặt</span>
            <Wallet className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-xl font-bold font-mono text-white tracking-tight">
            {formatMoney(snapshot.cash_minor)}
          </div>
          <div className="mt-1 text-[11px] text-slate-400 flex items-center gap-1">
            <span className="text-emerald-400 font-semibold">Runway: {snapshot.runway_days} ngày</span>
            <span>• An toàn</span>
          </div>
        </div>

        {/* Card 2: Monthly Revenue */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
          <div className="flex items-center justify-between text-slate-400 mb-1.5">
            <span className="text-xs font-medium">Doanh Thu Hàng Tháng</span>
            <TrendingUp className="w-4 h-4 text-blue-400" />
          </div>
          <div className="text-xl font-bold font-mono text-white tracking-tight">
            {formatMoney(snapshot.revenue_minor)}
          </div>
          <div className="mt-1 text-[11px] text-slate-400 flex items-center gap-1">
            <span className="text-blue-400 font-semibold">+{(snapshot.audience_growth_bps / 100).toFixed(1)}% / tuần</span>
            <span>tăng trưởng</span>
          </div>
        </div>

        {/* Card 3: Monthly Expenses */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
          <div className="flex items-center justify-between text-slate-400 mb-1.5">
            <span className="text-xs font-medium">Chi Phí Vận Hành</span>
            <TrendingDown className="w-4 h-4 text-amber-400" />
          </div>
          <div className="text-xl font-bold font-mono text-white tracking-tight">
            {formatMoney(snapshot.expenses_minor)}
          </div>
          <div className="mt-1 text-[11px] text-slate-400">
            <span>{agents.length} nhân sự • Phí máy chủ & API</span>
          </div>
        </div>

        {/* Card 4: Net Monthly Profit */}
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
          <div className="flex items-center justify-between text-slate-400 mb-1.5">
            <span className="text-xs font-medium">Lợi Nhuận Ròng (Net)</span>
            <DollarSign className="w-4 h-4 text-emerald-400" />
          </div>
          <div className={`text-xl font-bold font-mono tracking-tight ${netMonthly >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
            {netMonthly >= 0 ? '+' : ''}{formatMoney(netMonthly)}
          </div>
          <div className="mt-1 text-[11px] text-slate-400">
            <span>Biên lợi nhuận gộp: <strong className="text-slate-200">{pnl?.grossMarginPercent ?? 82}%</strong></span>
          </div>
        </div>
      </div>

      {/* Main Grid: Left Column (Chart + P&L) and Right Column (Live Office Feed + Quick Actions) */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-5">
        {/* Left Column (2 Cols) */}
        <div className="lg:col-span-2 space-y-5">
          {/* Revenue & Cash Trend Chart */}
          <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
            <div className="flex items-center justify-between mb-4">
              <div>
                <h3 className="text-sm font-semibold text-white">Xu Hướng Tài Chính Doanh Nghiệp</h3>
                <p className="text-xs text-slate-400">Diễn biến doanh thu, chi phí và quỹ tiền mặt qua các chu kỳ</p>
              </div>
              <div className="flex items-center gap-1.5 text-xs bg-slate-950 p-1 rounded-lg border border-slate-800">
                <span className="flex items-center gap-1 text-[11px] text-blue-400 font-medium px-2">
                  <span className="w-2 h-2 rounded-full bg-blue-500"></span> Doanh thu
                </span>
                <span className="flex items-center gap-1 text-[11px] text-emerald-400 font-medium px-2">
                  <span className="w-2 h-2 rounded-full bg-emerald-500"></span> Quỹ tiền
                </span>
              </div>
            </div>

            <div className="h-60 w-full">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={trendData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                  <defs>
                    <linearGradient id="revenueGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#3B82F6" stopOpacity={0.25} />
                      <stop offset="95%" stopColor="#3B82F6" stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="cashGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#10B981" stopOpacity={0.2} />
                      <stop offset="95%" stopColor="#10B981" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#1E293B" vertical={false} />
                  <XAxis dataKey="cycle" stroke="#64748B" fontSize={11} tickLine={false} />
                  <YAxis stroke="#64748B" fontSize={11} tickLine={false} tickFormatter={(v) => `$${v}`} />
                  <Tooltip 
                    contentStyle={{ backgroundColor: '#0F172A', borderColor: '#334155', borderRadius: '8px', fontSize: '12px' }}
                    formatter={(value: any) => [`$${Number(value).toLocaleString()}`, '']}
                  />
                  <Area type="monotone" dataKey="revenue" stroke="#3B82F6" strokeWidth={2} fillOpacity={1} fill="url(#revenueGrad)" />
                  <Area type="monotone" dataKey="cash" stroke="#10B981" strokeWidth={2} fillOpacity={1} fill="url(#cashGrad)" />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          </div>

          {/* Clean P&L Statement */}
          <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
            <div className="flex items-center justify-between mb-3 border-b border-slate-800 pb-2.5">
              <div>
                <h3 className="text-sm font-semibold text-white">Báo Cáo Kết Quả Kinh Doanh (P&L)</h3>
                <p className="text-xs text-slate-400">Kỳ vận hành #{snapshot.cycle_count} • Chuẩn kế toán doanh nghiệp</p>
              </div>
              <button
                onClick={() => onNavigate('finances')}
                className="text-xs text-blue-400 hover:text-blue-300 font-medium flex items-center gap-1"
              >
                Xem Sổ Cái Chi Tiết <ArrowRight className="w-3 h-3" />
              </button>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
              <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800/80">
                <span className="text-slate-400 block text-[11px]">Tổng Doanh Thu</span>
                <span className="font-semibold text-white font-mono text-sm">{formatMoney(snapshot.revenue_minor)}</span>
              </div>
              <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800/80">
                <span className="text-slate-400 block text-[11px]">Giá Vốn (COGS)</span>
                <span className="font-semibold text-slate-300 font-mono text-sm">{formatMoney(Math.round(snapshot.revenue_minor * 0.18))}</span>
              </div>
              <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800/80">
                <span className="text-slate-400 block text-[11px]">Lợi Nhuận Gộp</span>
                <span className="font-semibold text-emerald-400 font-mono text-sm">{formatMoney(Math.round(snapshot.revenue_minor * 0.82))}</span>
              </div>
              <div className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800/80">
                <span className="text-slate-400 block text-[11px]">Cổ Tức Phân Phối (20%)</span>
                <span className="font-semibold text-emerald-300 font-mono text-sm">
                  {formatMoney(pnl?.dividendsDeclaredMinor ?? Math.max(0, Math.round(netMonthly * 0.2)))}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* Right Column (1 Col: Quick Actions + Live Activity Stream) */}
        <div className="space-y-5">
          {/* Quick Control Center */}
          <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
            <h3 className="text-sm font-semibold text-white mb-3">Điều Hành Nhanh</h3>
            <div className="space-y-2">
              <button
                onClick={() => onNavigate('studio')}
                className="w-full flex items-center justify-between p-2.5 rounded-lg bg-slate-800/60 hover:bg-slate-800 text-left text-xs font-medium text-slate-200 transition-colors border border-slate-700/60"
              >
                <div className="flex items-center gap-2">
                  <Video className="w-4 h-4 text-blue-400" />
                  <div>
                    <div className="text-white font-medium">Xưởng Media 5-in-1</div>
                    <div className="text-[11px] text-slate-400">Tạo trọn gói Video + Kịch bản + Audio</div>
                  </div>
                </div>
                <ArrowRight className="w-3.5 h-3.5 text-slate-400" />
              </button>

              <button
                onClick={() => onNavigate('agents')}
                className="w-full flex items-center justify-between p-2.5 rounded-lg bg-slate-800/60 hover:bg-slate-800 text-left text-xs font-medium text-slate-200 transition-colors border border-slate-700/60"
              >
                <div className="flex items-center gap-2">
                  <Users className="w-4 h-4 text-emerald-400" />
                  <div>
                    <div className="text-white font-medium">Nhân Sự & Tuyển Dụng</div>
                    <div className="text-[11px] text-slate-400">{agents.length} nhân sự đang hoạt động</div>
                  </div>
                </div>
                <ArrowRight className="w-3.5 h-3.5 text-slate-400" />
              </button>

              {onToggleAutoPilot && (
                <button
                  onClick={onToggleAutoPilot}
                  className={`w-full flex items-center justify-between p-2.5 rounded-lg text-left text-xs font-medium transition-colors border ${
                    isAuto
                      ? 'bg-blue-950/50 border-blue-800 text-blue-200 hover:bg-blue-900/60'
                      : 'bg-slate-800/60 hover:bg-slate-800 text-slate-200 border-slate-700/60'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    <Sparkles className="w-4 h-4 text-yellow-400" />
                    <div>
                      <div className="text-white font-medium">{isAuto ? 'Auto-Pilot Đang Chạy' : 'Bật Tự Động Hóa 24/7'}</div>
                      <div className="text-[11px] text-slate-400">Tự động xuất bản & kiếm doanh thu</div>
                    </div>
                  </div>
                  <span className={`text-[10px] px-2 py-0.5 rounded font-mono font-semibold ${isAuto ? 'bg-blue-500/20 text-blue-300' : 'bg-slate-700 text-slate-300'}`}>
                    {isAuto ? 'ON' : 'OFF'}
                  </span>
                </button>
              )}
            </div>
          </div>

          {/* Live Activity Stream (Clean Ticker) */}
          <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
            <div className="flex items-center justify-between mb-3 border-b border-slate-800 pb-2">
              <div className="flex items-center gap-1.5">
                <Activity className="w-4 h-4 text-blue-400" />
                <h3 className="text-sm font-semibold text-white">Nhật Ký Hoạt Động Doanh Nghiệp</h3>
              </div>
              <span className="text-[11px] text-slate-400">Thời gian thực</span>
            </div>

            <div className="space-y-2.5 max-h-72 overflow-y-auto pr-1">
              {officeActivities.length === 0 ? (
                <div className="text-center py-6 text-xs text-slate-500">
                  Chưa có hoạt động mới. Bấm "Chạy Chu Kỳ" hoặc bật Auto-Pilot để khởi động.
                </div>
              ) : (
                officeActivities.slice(0, 6).map((item) => (
                  <div key={item.id} className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800/80 text-xs">
                    <div className="flex items-center justify-between text-[11px] text-slate-400 mb-1">
                      <span className="font-semibold text-slate-300">{item.agentName} ({item.agentRole})</span>
                      <span className="font-mono text-[10px] text-slate-500">
                        {new Date(item.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                      </span>
                    </div>
                    <div className="text-slate-200 font-medium text-[12px]">{item.title}</div>
                    {item.detail && (
                      <p className="text-[11px] text-slate-400 mt-0.5 line-clamp-2">{item.detail}</p>
                    )}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
