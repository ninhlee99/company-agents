import React, { useState } from 'react';
import { CompanySnapshot, GovernedProposal, CycleTrendPoint, AutoAuditReport, DepartmentBudgetPoint, SystemAlert, CompanyKPIs } from '../types/company';
import { KPIOverview } from './KPIOverview';
import { SystemAlertsLog } from './SystemAlertsLog';
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
  Layers,
  FileCheck2,
  CalendarCheck,
  Scale,
  PieChart,
  X
} from 'lucide-react';
import {
  ResponsiveContainer,
  LineChart,
  Line,
  AreaChart,
  Area,
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
  departmentBudgets?: DepartmentBudgetPoint[];
  auditReports?: AutoAuditReport[];
  systemAlerts?: SystemAlert[];
  kpis?: CompanyKPIs;
  onRunCycle: () => void;
  isRunningCycle: boolean;
  onOverride: (proposalId: string, decision: 'Approve' | 'Reject') => void;
  onNavigate: (view: 'dashboard' | 'agents' | 'finances' | 'pipeline') => void;
  onTriggerAudit?: () => Promise<void>;
  onResolveAlert?: (alertId: string) => Promise<void>;
  onTriggerAlert?: (alertData: Partial<SystemAlert>) => Promise<void>;
}

export const BasicDashboard: React.FC<BasicDashboardProps> = ({
  snapshot,
  recentProposals,
  cycleHistory,
  departmentBudgets,
  auditReports,
  systemAlerts,
  kpis,
  onRunCycle,
  isRunningCycle,
  onOverride,
  onNavigate,
  onTriggerAudit,
  onResolveAlert,
  onTriggerAlert,
}) => {
  const [chartView, setChartView] = useState<'financial_trend' | 'department_budget'>('financial_trend');
  const [chartMetric, setChartMetric] = useState<'all' | 'revenue_expense' | 'cash'>('all');
  const [showAuditModal, setShowAuditModal] = useState(false);
  const [isAuditing, setIsAuditing] = useState(false);

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const netMonthly = snapshot.revenue_minor - snapshot.expenses_minor;
  const pendingApprovals = recentProposals.filter((p) => p.decision === 'EscalateToHuman' && !p.executed);

  // Latest Auto-Audit Report (or calculated live if not loaded yet)
  const defaultAudit: AutoAuditReport = {
    id: 'audit-cycle-10-default',
    cycleMilestone: 10,
    timestamp: new Date().toISOString(),
    plannedRevenueMinor: 800000, // $8,000 baseline budget
    actualRevenueMinor: snapshot.revenue_minor,
    varianceRevenueMinor: snapshot.revenue_minor - 800000,
    variancePercent: Math.round(((snapshot.revenue_minor - 800000) / 800000) * 1000) / 10,
    plannedExpensesMinor: 600000,
    actualExpensesMinor: snapshot.expenses_minor,
    cashReserveMinor: snapshot.cash_minor,
    verdict: snapshot.revenue_minor >= 800000 ? 'ExceededTarget' : 'UnderTarget',
    summary: `Kỳ kiểm toán #10: Doanh thu thực tế ($${(snapshot.revenue_minor / 100).toLocaleString()}/th) so với ngân sách kế hoạch ($8,000/th) đạt mức lệch ${
      snapshot.revenue_minor >= 800000 ? '+' : ''
    }${Math.round(((snapshot.revenue_minor - 800000) / 800000) * 1000) / 10}%.`,
    governorNote: 'Hiến pháp: Đạt chuẩn bảo toàn vốn và tăng trưởng tự trị. Được phép tiếp tục mở rộng.',
  };

  const latestAudit: AutoAuditReport = auditReports && auditReports.length > 0 ? auditReports[0] : defaultAudit;
  const nextMilestone = Math.ceil((snapshot.cycle_count + 0.1) / 10) * 10;

  const handleRunManualAudit = async () => {
    if (onTriggerAudit) {
      setIsAuditing(true);
      await onTriggerAudit();
      setIsAuditing(false);
      setShowAuditModal(true);
    } else {
      setShowAuditModal(true);
    }
  };

  // 10-cycle trend
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

  // 10-cycle department budget distribution for stacked area chart
  const budgetData: DepartmentBudgetPoint[] = departmentBudgets && departmentBudgets.length > 0 ? departmentBudgets : [
    { cycle: 'Kỳ 5', cycleNum: 5, leadership: 880, growth: 2150, ops: 1180, techAndMedia: 690, total: 4900 },
    { cycle: 'Kỳ 6', cycleNum: 6, leadership: 910, growth: 2240, ops: 1230, techAndMedia: 720, total: 5100 },
    { cycle: 'Kỳ 7', cycleNum: 7, leadership: 950, growth: 2330, ops: 1280, techAndMedia: 740, total: 5300 },
    { cycle: 'Kỳ 8', cycleNum: 8, leadership: 1000, growth: 2470, ops: 1350, techAndMedia: 780, total: 5600 },
    { cycle: 'Kỳ 9', cycleNum: 9, leadership: 1020, growth: 2510, ops: 1370, techAndMedia: 800, total: 5700 },
    { cycle: 'Kỳ 10', cycleNum: 10, leadership: 1060, growth: 2600, ops: 1420, techAndMedia: 820, total: 5900 },
    { cycle: 'Kỳ 11', cycleNum: 11, leadership: 1080, growth: 2640, ops: 1440, techAndMedia: 840, total: 6000 },
    { cycle: 'Kỳ 12', cycleNum: 12, leadership: 1100, growth: 2690, ops: 1460, techAndMedia: 850, total: 6100 },
    { cycle: 'Kỳ 13', cycleNum: 13, leadership: 1110, growth: 2710, ops: 1470, techAndMedia: 860, total: 6150 },
    { 
      cycle: `Kỳ ${snapshot.cycle_count}`, 
      cycleNum: snapshot.cycle_count, 
      leadership: Math.round((snapshot.expenses_minor / 100) * 0.18), 
      growth: Math.round((snapshot.expenses_minor / 100) * 0.44), 
      ops: Math.round((snapshot.expenses_minor / 100) * 0.24), 
      techAndMedia: Math.max(0, Math.round(snapshot.expenses_minor / 100) - Math.round((snapshot.expenses_minor / 100) * 0.18) - Math.round((snapshot.expenses_minor / 100) * 0.44) - Math.round((snapshot.expenses_minor / 100) * 0.24)), 
      total: Math.round(snapshot.expenses_minor / 100) 
    },
  ];

  return (
    <div className="space-y-4 max-w-5xl mx-auto pb-10">
      {/* KPI OVERVIEW: TREND-INDICATING CARDS AT THE TOP OF THE SCREEN */}
      <KPIOverview snapshot={snapshot} kpis={kpis} onNavigate={onNavigate} />

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

      {/* SYSTEM ALERTS LOG: HIGHLIGHTS CRITICAL COMPANY STATUS DISCOVERED BY GOVERNOR */}
      <SystemAlertsLog
        alerts={systemAlerts || []}
        onResolveAlert={onResolveAlert}
        onTriggerAlert={onTriggerAlert}
      />

      {/* AUTO-AUDIT SUMMARY CARD: COMPARING REVENUE VS INITIAL BUDGET PLAN */}
      <div className="bg-gradient-to-r from-slate-900 via-indigo-950/20 to-slate-900 border border-indigo-500/30 rounded-xl p-4 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-3">
        <div className="flex items-start gap-3">
          <div className="w-9 h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 shrink-0 mt-0.5">
            <FileCheck2 className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-white text-xs">Kiểm Toán Tự Động Định Kỳ (Auto-Audit Routine)</span>
              <span className="px-2 py-0.2 rounded-full font-mono text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-bold">
                Mỗi 10 Chu Kỳ
              </span>
            </div>
            <p className="text-xs text-slate-300 mt-0.5">
              Doanh thu thực tế: <strong className="text-white">${(snapshot.revenue_minor / 100).toLocaleString()}/th</strong> vs Kế hoạch ngân sách ban đầu (<strong className="text-slate-400">$8,000/th</strong>).
              Độ lệch: <strong className={latestAudit.variancePercent >= 0 ? 'text-emerald-400' : 'text-rose-400'}>
                {latestAudit.variancePercent >= 0 ? `+${latestAudit.variancePercent}% (Vượt Kế Hoạch)` : `${latestAudit.variancePercent}% (Dưới Kế Hoạch)`}
              </strong>
            </p>
            <div className="flex items-center gap-3 text-[11px] text-slate-400 mt-1 font-mono">
              <span>Đợt gần nhất: Kỳ #{latestAudit.cycleMilestone}</span>
              <span>•</span>
              <span>Kỳ kiểm toán tiếp theo: Kỳ #{nextMilestone}</span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <button
            onClick={() => setShowAuditModal(true)}
            className="flex-1 md:flex-none px-3.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs transition-all shadow-sm flex items-center justify-center gap-1.5"
          >
            <Scale className="w-3.5 h-3.5" /> Xem Báo Cáo Kiểm Toán
          </button>
        </div>
      </div>

      {/* RECHARTS FINANCIAL & BUDGET VISUALIZATION CARDS */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3 shadow-sm">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
          <div className="flex items-center gap-2">
            {chartView === 'financial_trend' ? (
              <Activity className="w-4 h-4 text-indigo-400" />
            ) : (
              <Layers className="w-4 h-4 text-purple-400" />
            )}
            <div>
              <span className="font-bold text-white text-xs block">
                {chartView === 'financial_trend'
                  ? 'Biểu Đồ Xu Hướng 10 Chu Kỳ Gần Nhất'
                  : 'Phân Bổ Ngân Sách Theo Phòng Ban 10 Chu Kỳ (Stacked Area)'}
              </span>
              <span className="text-[10px] text-slate-400">
                {chartView === 'financial_trend'
                  ? 'Doanh thu, chi phí và biến động dòng tiền thực tế'
                  : 'Cơ cấu phân bổ chi phí giữa Ban Lãnh Đạo, Kinh Doanh, Vận Hành & Media'}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {/* View Switcher: Financial Trend vs Department Stacked Area */}
            <div className="flex bg-slate-950 p-1 rounded-lg border border-slate-800 text-[11px]">
              <button
                onClick={() => setChartView('financial_trend')}
                className={`px-2.5 py-1 rounded-md font-semibold transition-all flex items-center gap-1.5 ${
                  chartView === 'financial_trend'
                    ? 'bg-indigo-600 text-white shadow-sm'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                <Activity className="w-3 h-3" />
                <span>Thu Chi</span>
              </button>
              <button
                onClick={() => setChartView('department_budget')}
                className={`px-2.5 py-1 rounded-md font-semibold transition-all flex items-center gap-1.5 ${
                  chartView === 'department_budget'
                    ? 'bg-indigo-600 text-white shadow-sm'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                <Layers className="w-3 h-3 text-purple-300" />
                <span>Ngân Sách Phòng Ban</span>
              </button>
            </div>

            {/* Quick Metrics filter for Financial Trend */}
            {chartView === 'financial_trend' && (
              <div className="hidden sm:flex items-center gap-1 bg-slate-950 p-1 rounded-lg border border-slate-800 text-[11px]">
                <button
                  onClick={() => setChartMetric('all')}
                  className={`px-2 py-0.5 rounded font-semibold transition-all ${
                    chartMetric === 'all' ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  Tất cả
                </button>
                <button
                  onClick={() => setChartMetric('revenue_expense')}
                  className={`px-2 py-0.5 rounded font-semibold transition-all ${
                    chartMetric === 'revenue_expense' ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  Thu vs Chi
                </button>
                <button
                  onClick={() => setChartMetric('cash')}
                  className={`px-2 py-0.5 rounded font-semibold transition-all ${
                    chartMetric === 'cash' ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  Kho Bạc
                </button>
              </div>
            )}
          </div>
        </div>

        {/* Chart Body */}
        {chartView === 'financial_trend' ? (
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
        ) : (
          /* STACKED AREA CHART: BUDGET DISTRIBUTION ACROSS DEPARTMENTS OVER 10 CYCLES */
          <div className="space-y-3">
            <div className="h-64 w-full pt-1">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={budgetData} margin={{ top: 10, right: 10, left: -15, bottom: 0 }}>
                  <defs>
                    <linearGradient id="colorLeadership" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#6366f1" stopOpacity={0.8} />
                      <stop offset="95%" stopColor="#6366f1" stopOpacity={0.15} />
                    </linearGradient>
                    <linearGradient id="colorOps" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#10b981" stopOpacity={0.8} />
                      <stop offset="95%" stopColor="#10b981" stopOpacity={0.15} />
                    </linearGradient>
                    <linearGradient id="colorGrowth" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#a855f7" stopOpacity={0.8} />
                      <stop offset="95%" stopColor="#a855f7" stopOpacity={0.15} />
                    </linearGradient>
                    <linearGradient id="colorTech" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#f59e0b" stopOpacity={0.8} />
                      <stop offset="95%" stopColor="#f59e0b" stopOpacity={0.15} />
                    </linearGradient>
                  </defs>
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
                      name === 'growth'
                        ? 'Kinh Doanh & Affiliate (Growth)'
                        : name === 'ops'
                        ? 'Vận Hành & Điều Phối (Ops)'
                        : name === 'leadership'
                        ? 'Ban Lãnh Đạo & Hiến Pháp (Leadership)'
                        : 'Kỹ Thuật, Token & Media (Tech)',
                    ]}
                    labelStyle={{ color: '#e2e8f0', fontWeight: 'bold' }}
                  />
                  <Legend
                    wrapperStyle={{ fontSize: '11px', paddingTop: '8px' }}
                    formatter={(val) =>
                      val === 'growth'
                        ? 'Kinh Doanh (Growth)'
                        : val === 'ops'
                        ? 'Vận Hành (Ops)'
                        : val === 'leadership'
                        ? 'Ban Giám Đốc (Leadership)'
                        : 'Kỹ Thuật & Media'
                    }
                  />
                  <Area
                    type="monotone"
                    dataKey="leadership"
                    stackId="1"
                    stroke="#6366f1"
                    fillOpacity={1}
                    fill="url(#colorLeadership)"
                  />
                  <Area
                    type="monotone"
                    dataKey="ops"
                    stackId="1"
                    stroke="#10b981"
                    fillOpacity={1}
                    fill="url(#colorOps)"
                  />
                  <Area
                    type="monotone"
                    dataKey="growth"
                    stackId="1"
                    stroke="#a855f7"
                    fillOpacity={1}
                    fill="url(#colorGrowth)"
                  />
                  <Area
                    type="monotone"
                    dataKey="techAndMedia"
                    stackId="1"
                    stroke="#f59e0b"
                    fillOpacity={1}
                    fill="url(#colorTech)"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>

            {/* Department Budget Proportions Summary */}
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 pt-2 border-t border-slate-800">
              <div className="p-2 rounded-lg bg-slate-950 border border-slate-800/80 text-xs">
                <span className="text-[10px] text-indigo-400 font-bold block flex items-center gap-1">
                  <span className="w-2 h-2 rounded-full bg-indigo-500 inline-block" /> Ban Giám Đốc
                </span>
                <div className="font-bold text-white font-mono mt-0.5">18% Ngân Sách</div>
                <span className="text-[10px] text-slate-500">Giám sát & Hiến pháp</span>
              </div>

              <div className="p-2 rounded-lg bg-slate-950 border border-slate-800/80 text-xs">
                <span className="text-[10px] text-purple-400 font-bold block flex items-center gap-1">
                  <span className="w-2 h-2 rounded-full bg-purple-500 inline-block" /> Khối Kinh Doanh
                </span>
                <div className="font-bold text-white font-mono mt-0.5">44% Ngân Sách</div>
                <span className="text-[10px] text-slate-500">Traffic, TikTok & Content</span>
              </div>

              <div className="p-2 rounded-lg bg-slate-950 border border-slate-800/80 text-xs">
                <span className="text-[10px] text-emerald-400 font-bold block flex items-center gap-1">
                  <span className="w-2 h-2 rounded-full bg-emerald-500 inline-block" /> Khối Vận Hành
                </span>
                <div className="font-bold text-white font-mono mt-0.5">24% Ngân Sách</div>
                <span className="text-[10px] text-slate-500">Kế toán & Thông lượng</span>
              </div>

              <div className="p-2 rounded-lg bg-slate-950 border border-slate-800/80 text-xs">
                <span className="text-[10px] text-amber-400 font-bold block flex items-center gap-1">
                  <span className="w-2 h-2 rounded-full bg-amber-500 inline-block" /> Kỹ Thuật & Media
                </span>
                <div className="font-bold text-white font-mono mt-0.5">14% Ngân Sách</div>
                <span className="text-[10px] text-slate-500">GPU, API & Prompt Cache</span>
              </div>
            </div>
          </div>
        )}
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

      {/* AUTO-AUDIT DETAIL MODAL */}
      {showAuditModal && (
        <div className="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-2xl w-full p-5 shadow-2xl space-y-4 max-h-[85vh] flex flex-col">
            <div className="flex items-center justify-between border-b border-slate-800 pb-3 shrink-0">
              <div className="flex items-center gap-2">
                <Scale className="w-5 h-5 text-indigo-400" />
                <div>
                  <h3 className="font-bold text-white text-sm">
                    Báo Cáo Kiểm Toán Định Kỳ 10 Chu Kỳ (Auto-Audit Report)
                  </h3>
                  <p className="text-[11px] text-slate-400">Đối chiếu Doanh Thu Thực Tế vs Kế Hoạch Ngân Sách Ban Đầu</p>
                </div>
              </div>

              <button
                onClick={() => setShowAuditModal(false)}
                className="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-all"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto space-y-4 pr-1 text-xs">
              {/* Variance Comparison Cards */}
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
                <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
                  <span className="text-[11px] text-slate-400 block font-medium">Doanh Thu Kế Hoạch</span>
                  <div className="text-lg font-bold text-slate-300 font-mono mt-0.5">
                    ${(latestAudit.plannedRevenueMinor / 100).toLocaleString()}
                  </div>
                  <span className="text-[10px] text-slate-500 font-mono">Chỉ tiêu tháng</span>
                </div>

                <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
                  <span className="text-[11px] text-slate-400 block font-medium">Doanh Thu Thực Tế</span>
                  <div className="text-lg font-bold text-emerald-400 font-mono mt-0.5">
                    ${(latestAudit.actualRevenueMinor / 100).toLocaleString()}
                  </div>
                  <span className="text-[10px] text-emerald-400/90 font-mono font-bold">
                    {latestAudit.variancePercent >= 0 ? `+${latestAudit.variancePercent}%` : `${latestAudit.variancePercent}%`}
                  </span>
                </div>

                <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 col-span-2 sm:col-span-1">
                  <span className="text-[11px] text-slate-400 block font-medium">Đánh Giá (Verdict)</span>
                  <div className={`text-sm font-bold font-mono mt-1 ${latestAudit.variancePercent >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
                    {latestAudit.variancePercent >= 0 ? 'VƯỢT KẾ HOẠCH' : 'DƯỚI CHỈ TIÊU'}
                  </div>
                  <span className="text-[10px] text-slate-500 font-mono">Kiểm toán tự động</span>
                </div>
              </div>

              {/* Summary narrative */}
              <div className="p-3.5 rounded-lg bg-slate-950 border border-slate-800/80 space-y-2">
                <span className="font-bold text-white text-xs block flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" /> Kết Quả Thẩm Tra Tài Chính
                </span>
                <p className="text-slate-300 leading-relaxed text-xs">
                  {latestAudit.summary}
                </p>
              </div>

              {/* Governor Constitutional Note */}
              <div className="p-3.5 rounded-lg bg-indigo-950/20 border border-indigo-500/30 space-y-1.5">
                <span className="font-bold text-indigo-300 text-xs block flex items-center gap-1.5">
                  <ShieldCheck className="w-4 h-4 text-indigo-400" /> Phán Quyết Fiduciary Của Governor
                </span>
                <p className="text-slate-300 text-xs">
                  {latestAudit.governorNote}
                </p>
              </div>

              {/* Historical Audit List */}
              {auditReports && auditReports.length > 1 && (
                <div className="space-y-2 pt-2">
                  <span className="font-bold text-white text-xs block">Lịch Sử Các Đợt Kiểm Toán Trước:</span>
                  <div className="space-y-1.5">
                    {auditReports.slice(1).map((rep) => (
                      <div key={rep.id} className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between text-[11px]">
                        <div>
                          <strong className="text-white mr-2">Kỳ #{rep.cycleMilestone}:</strong>
                          <span className="text-slate-400">Doanh thu ${(rep.actualRevenueMinor / 100).toLocaleString()} (Kế hoạch: ${(rep.plannedRevenueMinor / 100).toLocaleString()})</span>
                        </div>
                        <span className={`font-mono font-bold ${rep.variancePercent >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
                          {rep.variancePercent >= 0 ? `+${rep.variancePercent}%` : `${rep.variancePercent}%`}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="border-t border-slate-800 pt-3 flex items-center justify-between shrink-0">
              <span className="text-[11px] text-slate-500 font-mono">
                Chu kỳ kiểm toán tiếp theo: Kỳ #{nextMilestone}
              </span>
              <button
                onClick={() => setShowAuditModal(false)}
                className="px-4 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs transition-all"
              >
                Đóng
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
