import React from 'react';
import { CompanySnapshot, CompanyKPIs } from '../types/company';
import { 
  Zap, 
  Flame, 
  TrendingUp, 
  ArrowUpRight, 
  ArrowDownRight, 
  Percent, 
  CheckCircle2, 
  ShieldCheck,
  Activity,
  Layers
} from 'lucide-react';

interface KPIOverviewProps {
  snapshot: CompanySnapshot;
  kpis?: CompanyKPIs;
  onNavigate?: (view: 'dashboard' | 'agents' | 'finances' | 'pipeline') => void;
}

export const KPIOverview: React.FC<KPIOverviewProps> = ({ snapshot, kpis, onNavigate }) => {
  const netMonthly = snapshot.revenue_minor - snapshot.expenses_minor;
  const burnRatio = Math.round((snapshot.expenses_minor / Math.max(1, snapshot.revenue_minor)) * 100) / 100;
  const netMarginPercent = Math.round((netMonthly / Math.max(1, snapshot.revenue_minor)) * 1000) / 10;

  // Defaults if not supplied
  const defaultKPIs: CompanyKPIs = {
    revenueVelocity: {
      value: `$${(snapshot.revenue_minor / 100).toLocaleString()}/th`,
      changeRate: 14.2,
      periodLabel: 'So với chu kỳ trước',
      trend: 'up',
    },
    burnRateEfficiency: {
      value: `${burnRatio}x Chi/Thu`,
      ratio: burnRatio,
      statusText: netMonthly >= 0 ? `Thặng dư +$${(netMonthly / 100).toLocaleString()}/th` : 'Thâm hụt cần trợ vốn',
      trend: netMonthly >= 0 ? 'up' : 'down',
    },
    roiPerCycle: {
      value: '24.6% ROI',
      percentage: 24.6,
      statusText: 'Lợi tức trên ngân sách thử nghiệm & lương AI',
      trend: 'up',
    },
  };

  const activeKPIs = kpis || defaultKPIs;

  return (
    <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
      {/* 1. REVENUE VELOCITY */}
      <div className="bg-gradient-to-br from-slate-900 to-indigo-950/30 border border-indigo-500/30 rounded-xl p-4 shadow-sm hover:border-indigo-500/50 transition-all flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
            <Zap className="w-3.5 h-3.5 text-indigo-400" />
            Tốc Độ Doanh Thu (Revenue Velocity)
          </span>
          <span className="inline-flex items-center gap-0.5 px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
            <ArrowUpRight className="w-3 h-3" />
            +{activeKPIs.revenueVelocity.changeRate}%
          </span>
        </div>

        <div className="mt-2.5">
          <div className="text-2xl font-black text-white font-mono tracking-tight">
            {activeKPIs.revenueVelocity.value}
          </div>
          <div className="text-[11px] text-slate-400 mt-0.5 flex items-center justify-between">
            <span>{activeKPIs.revenueVelocity.periodLabel}</span>
            <span className="text-indigo-400 font-medium">↑ Gia tốc ổn định</span>
          </div>
        </div>

        {/* Mini progress track */}
        <div className="mt-3 w-full bg-slate-950 rounded-full h-1.5 overflow-hidden border border-indigo-500/20">
          <div className="bg-gradient-to-r from-indigo-500 to-cyan-400 h-1.5 rounded-full w-[78%]" />
        </div>
      </div>

      {/* 2. BURN RATE EFFICIENCY */}
      <div className="bg-gradient-to-br from-slate-900 to-emerald-950/20 border border-emerald-500/30 rounded-xl p-4 shadow-sm hover:border-emerald-500/50 transition-all flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
            <Flame className="w-3.5 h-3.5 text-rose-400" />
            Hiệu Suất Đốt Vốn (Burn Rate Efficiency)
          </span>
          <span className="inline-flex items-center gap-0.5 px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
            <CheckCircle2 className="w-3 h-3" />
            {netMarginPercent >= 0 ? `+${netMarginPercent}% Biên Lãi` : `${netMarginPercent}%`}
          </span>
        </div>

        <div className="mt-2.5">
          <div className="text-2xl font-black text-white font-mono tracking-tight">
            {activeKPIs.burnRateEfficiency.value}
          </div>
          <div className="text-[11px] text-emerald-400/90 mt-0.5 flex items-center justify-between font-mono">
            <span>{activeKPIs.burnRateEfficiency.statusText}</span>
            <span className="text-[10px] text-slate-400">Runway {snapshot.runway_days >= 999 ? '∞' : `${snapshot.runway_days}d`}</span>
          </div>
        </div>

        {/* Mini progress track */}
        <div className="mt-3 w-full bg-slate-950 rounded-full h-1.5 overflow-hidden border border-emerald-500/20">
          <div className="bg-gradient-to-r from-emerald-500 to-teal-400 h-1.5 rounded-full w-[65%]" />
        </div>
      </div>

      {/* 3. ROI PER CYCLE */}
      <div className="bg-gradient-to-br from-slate-900 to-purple-950/20 border border-purple-500/30 rounded-xl p-4 shadow-sm hover:border-purple-500/50 transition-all flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
            <TrendingUp className="w-3.5 h-3.5 text-purple-400" />
            Lợi Tức / Chu Kỳ (ROI per Cycle)
          </span>
          <span className="inline-flex items-center gap-0.5 px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-purple-500/20 text-purple-300 border border-purple-500/30">
            <Percent className="w-3 h-3" />
            Vượt Kế Hoạch
          </span>
        </div>

        <div className="mt-2.5">
          <div className="text-2xl font-black text-purple-400 font-mono tracking-tight">
            {activeKPIs.roiPerCycle.value}
          </div>
          <div className="text-[11px] text-slate-400 mt-0.5 flex items-center justify-between">
            <span className="truncate mr-1">{activeKPIs.roiPerCycle.statusText}</span>
            <span className="text-purple-300 font-medium shrink-0 font-mono">+3.8% MoM</span>
          </div>
        </div>

        {/* Mini progress track */}
        <div className="mt-3 w-full bg-slate-950 rounded-full h-1.5 overflow-hidden border border-purple-500/20">
          <div className="bg-gradient-to-r from-purple-500 to-pink-500 h-1.5 rounded-full w-[82%]" />
        </div>
      </div>
    </div>
  );
};
