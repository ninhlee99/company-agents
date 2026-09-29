import React, { useState } from 'react';
import { CompanySnapshot, GovernedProposal, CycleTrendPoint } from '../types/company';
import { 
  RotateCw, 
  CheckCircle2, 
  AlertTriangle, 
  TrendingUp, 
  TrendingDown, 
  Wallet, 
  Users, 
  Sparkles, 
  ArrowRight, 
  ShieldCheck, 
  Activity,
  Layers
} from 'lucide-react';
import {
  ResponsiveContainer,
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid,
  Legend
} from 'recharts';

interface BasicDashboardProps {
  snapshot: CompanySnapshot;
  recentProposals: GovernedProposal[];
  cycleHistory?: CycleTrendPoint[];
  onRunCycle: () => void;
  isRunningCycle: boolean;
  onOverride: (proposalId: string, decision: 'Approve' | 'Reject') => void;
  onNavigate: (view: 'dashboard' | 'agents' | 'finances' | 'pipeline') => void;
}

export const BasicDashboard: React.FC<BasicDashboardProps> = ({
  snapshot,
  recentProposals,
  cycleHistory,
  onRunCycle,
  isRunningCycle,
  onOverride,
  onNavigate,
}) => {
  const [chartMetric, setChartMetric] = useState<'all' | 'revenue_expense' | 'cash'>('all');

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const netMonthly = snapshot.revenue_minor - snapshot.expenses_minor;
  const pendingApprovals = recentProposals.filter((p) => p.decision === 'EscalateToHuman' && !p.executed);

  // Fallback 10-cycle trend if not provided by backend
  const trendData: CycleTrendPoint[] = cycleHistory && cycleHistory.length > 0 ? cycleHistory : [
    { cycle: 'Kỳ 5', cycleNum: 5, cash: 38000, revenue: 5200, expenses: 4900, netCashFlow: 300 },
    { cycle: 'Kỳ 6', cycleNum: 6, cash: 39500, revenue: 5800, expenses: 5100, netCashFlow: 700 },
    { cycle: 'Kỳ 7', cycleNum: 7, cash: 41000, revenue: 6400, expenses: 5300, netCashFlow: 1100 },
    { cycle: 'Kỳ 8', cycleNum: 8, cash: 41800, revenue: 6900, expenses: 5600, netCashFlow: 1300 },
    { cycle: 'Kỳ 9', cycleNum: 9, cash: 43200, revenue: 7500, expenses: 5700, netCashFlow: 1800 },
    { cycle: 'Kỳ 10', cycleNum: 10, cash: 44600, revenue: 8100, expenses: 5900, netCashFlow: 2200 },
    { cycle: 'Kỳ 11', cycleNum: 11, cash: 45800, revenue: 8600, expenses: 6000, netCashFlow: 2600 },
    { cycle: 'Kỳ 12', cycleNum: 12, cash: 46900, revenue: 9000, expenses: 6100, netCashFlow: 2900 },
    { cycle: 'Kỳ 13', cycleNum: 13, cash: 47600, revenue: 9200, expenses: 6150, netCashFlow: 3050 },
    { cycle: `Kỳ ${snapshot.cycle_count}`, cycleNum: snapshot.cycle_count, cash: Math.round(snapshot.cash_minor / 100), revenue: Math.round(snapshot.revenue_minor / 100), expenses: Math.round(snapshot.expenses_minor / 100), netCashFlow: Math.round(netMonthly / 100) },
  ];

  return (
    <div className="space-y-4 max-w-5xl mx-auto pb-10">
      {/* Top Action Bar */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
        <div className="flex items-center gap-3">
          <div className="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse shrink-0" />
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-white text-base">Hệ Thống Đang Vận Hành</span>
              <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-slate-800 text-slate-300">
                Kỳ #{snapshot.cycle_count}
              </span>
            </div>
            <p className="text-xs text-slate-400">9 vị trí AI tự động điều phối và tạo doanh thu</p>
          </div>
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          <button
            onClick={() => onNavigate('pipeline')}
            className="flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs shadow-md transition-all active:scale-95"
          >
            <Sparkles className="w-3.5 h-3.5" /> Chạy Dây Chuyền Bán Hàng
          </button>

          <button
            onClick={onRunCycle}
            disabled={isRunningCycle}
            className={`flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg font-bold text-xs text-white shadow-md transition-all ${
              isRunningCycle
                ? 'bg-slate-800 text-slate-400 cursor-not-allowed'
                : 'bg-indigo-600 hover:bg-indigo-500 active:scale-95'
            }`}
          >
            <RotateCw className={`w-3.5 h-3.5 ${isRunningCycle ? 'animate-spin' : ''}`} />
            {isRunningCycle ? 'Đang họp...' : 'Họp Chu Kỳ'}
          </button>
        </div>
      </div>

      {/* 4 Core Financial Numbers */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Kho Bạc</span>
            <Wallet className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-xl font-bold text-white font-mono mt-1">
            {formatMoney(snapshot.cash_minor)}
          </div>
          <div className="text-[11px] text-cyan-400 mt-0.5">
            Sống: {snapshot.runway_days >= 999 ? '∞ Tự lãi' : `${snapshot.runway_days} ngày`}
          </div>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Thu / Tháng</span>
            <TrendingUp className="w-4 h-4 text-cyan-400" />
          </div>
          <div className="text-xl font-bold text-cyan-400 font-mono mt-1">
            {formatMoney(snapshot.revenue_minor)}
          </div>
          <div className="text-[11px] text-slate-400 mt-0.5">Hoa hồng &amp; Media</div>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Chi / Tháng</span>
            <TrendingDown className="w-4 h-4 text-rose-400" />
          </div>
          <div className="text-xl font-bold text-rose-400 font-mono mt-1">
            {formatMoney(snapshot.expenses_minor)}
          </div>
          <div className="text-[11px] text-slate-400 mt-0.5">Server &amp; Lương AI</div>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs">
            <span>Lợi Nhuận</span>
            <Sparkles className="w-4 h-4 text-purple-400" />
          </div>
          <div className={`text-xl font-bold font-mono mt-1 ${netMonthly >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
            {netMonthly >= 0 ? `+${formatMoney(netMonthly)}` : formatMoney(netMonthly)}
          </div>
          <div className="text-[11px] text-slate-400 mt-0.5">
            {netMonthly >= 0 ? '🟢 Có lãi' : '🔴 Bù lỗ'}
          </div>
        </div>
      </div>

      {/* RECHARTS LINE GRAPH: 10-CYCLE FINANCIAL & CASH FLOW TREND */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3 shadow-sm">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-indigo-400" />
            <div>
              <span className="font-bold text-white text-xs block">Biểu Đồ Xu Hướng 10 Chu Kỳ Gần Nhất</span>
              <span className="text-[10px] text-slate-400">Doanh thu, chi phí và biến động dòng tiền thực tế</span>
            </div>
          </div>

          <div className="flex items-center gap-1 bg-slate-950 p-1 rounded-lg border border-slate-800 text-[11px]">
            <button
              onClick={() => setChartMetric('all')}
              className={`px-2 py-0.5 rounded font-semibold transition-all ${
                chartMetric === 'all' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              Tất cả
            </button>
            <button
              onClick={() => setChartMetric('revenue_expense')}
              className={`px-2 py-0.5 rounded font-semibold transition-all ${
                chartMetric === 'revenue_expense' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              Thu vs Chi
            </button>
            <button
              onClick={() => setChartMetric('cash')}
              className={`px-2 py-0.5 rounded font-semibold transition-all ${
                chartMetric === 'cash' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              Kho Bạc
            </button>
          </div>
        </div>

        {/* Recharts Container */}
        <div className="h-60 w-full pt-1">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={trendData} margin={{ top: 10, right: 10, left: -15, bottom: 0 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
              <XAxis
                dataKey="cycle"
                stroke="#64748b"
                tick={{ fill: '#94a3b8', fontSize: 11 }}
                tickLine={false}
              />
              <YAxis
                stroke="#64748b"
                tick={{ fill: '#94a3b8', fontSize: 10 }}
                tickFormatter={(val) => `$${val >= 1000 ? `${(val / 1000).toFixed(0)}k` : val}`}
                tickLine={false}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#0f172a',
                  borderColor: '#334155',
                  borderRadius: '0.75rem',
                  fontSize: '11px',
                  boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.5)',
                }}
                formatter={(val: any, name: any) => [
                  `$${Number(val || 0).toLocaleString()}`,
                  name === 'revenue'
                    ? 'Doanh Thu'
                    : name === 'expenses'
                    ? 'Chi Phí'
                    : name === 'netCashFlow'
                    ? 'Lợi Nhuận Ròng'
                    : 'Kho Bạc',
                ]}
                labelStyle={{ color: '#e2e8f0', fontWeight: 'bold' }}
              />
              <Legend
                wrapperStyle={{ fontSize: '11px', paddingTop: '8px' }}
                formatter={(val) =>
                  val === 'revenue'
                    ? 'Doanh Thu ($)'
                    : val === 'expenses'
                    ? 'Chi Phí ($)'
                    : val === 'netCashFlow'
                    ? 'Dòng Tiền / Lãi ($)'
                    : 'Số Dư Kho Bạc ($)'
                }
              />

              {(chartMetric === 'all' || chartMetric === 'revenue_expense') && (
                <Line
                  type="monotone"
                  dataKey="revenue"
                  stroke="#10b981"
                  strokeWidth={2.5}
                  dot={{ r: 3, fill: '#10b981' }}
                  activeDot={{ r: 5 }}
                />
              )}

              {(chartMetric === 'all' || chartMetric === 'revenue_expense') && (
                <Line
                  type="monotone"
                  dataKey="expenses"
                  stroke="#f43f5e"
                  strokeWidth={2}
                  strokeDasharray="4 4"
                  dot={{ r: 3, fill: '#f43f5e' }}
                  activeDot={{ r: 5 }}
                />
              )}

              {(chartMetric === 'all' || chartMetric === 'revenue_expense') && (
                <Line
                  type="monotone"
                  dataKey="netCashFlow"
                  stroke="#a855f7"
                  strokeWidth={2}
                  dot={{ r: 3, fill: '#a855f7' }}
                  activeDot={{ r: 5 }}
                />
              )}

              {(chartMetric === 'all' || chartMetric === 'cash') && (
                <Line
                  type="monotone"
                  dataKey="cash"
                  stroke="#38bdf8"
                  strokeWidth={2.5}
                  dot={{ r: 3, fill: '#38bdf8' }}
                  activeDot={{ r: 5 }}
                />
              )}
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Pending Approvals (Only when needed) */}
      {pendingApprovals.length > 0 && (
        <div className="bg-amber-950/20 border border-amber-500/40 rounded-xl p-4 space-y-2">
          <div className="flex items-center gap-1.5 text-amber-300 font-bold text-xs">
            <AlertTriangle className="w-4 h-4 text-amber-400" />
            Cần bạn duyệt ({pendingApprovals.length} đề xuất &gt; $1,000):
          </div>

          <div className="space-y-1.5">
            {pendingApprovals.map((item) => (
              <div
                key={item.proposal.id}
                className="bg-slate-900 border border-slate-800 rounded-lg p-3 flex flex-col sm:flex-row sm:items-center justify-between gap-2 text-xs"
              >
                <div>
                  <span className="font-bold text-white mr-2">{item.proposal.agent}:</span>
                  <span className="text-slate-300">{item.proposal.objective}</span>
                  <div className="text-[11px] text-rose-400 font-mono mt-0.5">
                    Chi phí: {formatMoney(item.proposal.cost_minor)}
                  </div>
                </div>

                <div className="flex items-center gap-2 shrink-0">
                  <button
                    onClick={() => onOverride(item.proposal.id, 'Approve')}
                    className="px-3 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs"
                  >
                    Duyệt
                  </button>
                  <button
                    onClick={() => onOverride(item.proposal.id, 'Reject')}
                    className="px-3 py-1 rounded bg-rose-600 hover:bg-rose-500 text-white font-bold text-xs"
                  >
                    Từ chối
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Main Grid: Activity & Navigation */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Activity Feed */}
        <div className="lg:col-span-2 bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3">
          <div className="flex items-center justify-between border-b border-slate-800 pb-2">
            <span className="font-bold text-white text-xs flex items-center gap-1.5">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" /> Hoạt Động Gần Nhất
            </span>
            <span className="text-[11px] text-slate-500">Đã qua kiểm toán</span>
          </div>

          <div className="space-y-2">
            {recentProposals.slice(0, 4).map((item, idx) => (
              <div
                key={idx}
                className="p-2.5 rounded-lg bg-slate-950 border border-slate-800/80 flex items-center justify-between gap-3 text-xs"
              >
                <div className="truncate">
                  <span className="px-1.5 py-0.5 rounded text-[10px] font-bold font-mono bg-indigo-500/20 text-indigo-300 mr-2">
                    {item.proposal.agent}
                  </span>
                  <span className="text-slate-200">{item.proposal.objective}</span>
                </div>

                <div className="shrink-0 text-right">
                  <span className={`text-[10px] font-bold ${
                    item.decision === 'Approve' ? 'text-emerald-400' : item.decision === 'Reject' ? 'text-rose-400' : 'text-amber-300'
                  }`}>
                    {item.decision === 'Approve' ? 'Đã duyệt' : item.decision === 'Reject' ? 'Từ chối' : 'Chờ duyệt'}
                  </span>
                  <div className="text-[10px] text-slate-500 font-mono">
                    {item.proposal.cost_minor > 0 ? formatMoney(item.proposal.cost_minor) : 'Miễn phí'}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Quick Management Short-Cuts */}
        <div className="space-y-2.5">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-2">
            <span className="font-bold text-white text-xs block">Truy Cập Nhanh</span>

            <button
              onClick={() => onNavigate('agents')}
              className="w-full text-left p-2.5 rounded-lg bg-slate-950 hover:bg-slate-800 border border-slate-800 text-xs text-slate-200 transition-all flex items-center justify-between"
            >
              <div className="flex items-center gap-2">
                <Users className="w-4 h-4 text-indigo-400" />
                <span className="font-medium">Quản Lý &amp; Tuyển Nhân Sự</span>
              </div>
              <ArrowRight className="w-3.5 h-3.5 text-slate-500" />
            </button>

            <button
              onClick={() => onNavigate('finances')}
              className="w-full text-left p-2.5 rounded-lg bg-slate-950 hover:bg-slate-800 border border-slate-800 text-xs text-slate-200 transition-all flex items-center justify-between"
            >
              <div className="flex items-center gap-2">
                <Wallet className="w-4 h-4 text-emerald-400" />
                <span className="font-medium">Ví Tiền &amp; Thu Chi</span>
              </div>
              <ArrowRight className="w-3.5 h-3.5 text-slate-500" />
            </button>

            <button
              onClick={() => onNavigate('pipeline')}
              className="w-full text-left p-2.5 rounded-lg bg-slate-950 hover:bg-slate-800 border border-slate-800 text-xs text-slate-200 transition-all flex items-center justify-between"
            >
              <div className="flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-purple-400" />
                <span className="font-medium">Dây Chuyền Bán Hàng</span>
              </div>
              <ArrowRight className="w-3.5 h-3.5 text-slate-500" />
            </button>
          </div>

          <div className="p-3 rounded-xl bg-indigo-950/20 border border-indigo-500/20 text-[11px] text-indigo-300 flex items-center gap-2">
            <ShieldCheck className="w-4 h-4 text-indigo-400 shrink-0" />
            <span>Governor tự động bảo toàn vốn và ngăn chi tiêu vượt mức.</span>
          </div>
        </div>
      </div>
    </div>
  );
};
