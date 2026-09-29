import React from 'react';
import { CompanySnapshot } from '../types/company';
import { ShieldCheck, Flame, RefreshCw, AlertTriangle, Building2, TrendingUp, Award, ExternalLink } from 'lucide-react';

interface HeaderProps {
  snapshot: CompanySnapshot;
  onRunCycle: () => void;
  isRunningCycle: boolean;
  activeTab: string;
  setActiveTab: (tab: string) => void;
  hasGeminiKey: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  snapshot,
  onRunCycle,
  isRunningCycle,
  activeTab,
  setActiveTab,
  hasGeminiKey,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Growth':
        return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30';
      case 'Active':
        return 'bg-blue-500/10 text-blue-400 border-blue-500/30';
      case 'Warning':
      case 'CostControl':
        return 'bg-amber-500/10 text-amber-400 border-amber-500/30';
      case 'Distress':
      case 'Emergency':
        return 'bg-orange-500/10 text-orange-400 border-orange-500/30';
      case 'Liquidation':
      case 'Bankrupt':
        return 'bg-rose-500/10 text-rose-400 border-rose-500/30';
      default:
        return 'bg-slate-500/10 text-slate-400 border-slate-500/30';
    }
  };

  return (
    <header className="sticky top-0 z-50 bg-slate-950/85 backdrop-blur-md border-b border-slate-800/80 px-4 lg:px-8 py-3">
      <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
        {/* Brand & Repo Meta */}
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 via-purple-600 to-cyan-500 flex items-center justify-center shadow-lg shadow-indigo-500/20 ring-1 ring-white/20">
            <Building2 className="w-5 h-5 text-white" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-white text-lg tracking-tight">Autonomous Company OS</span>
              <span className="px-2 py-0.5 text-xs rounded-full bg-indigo-500/20 text-indigo-300 font-mono border border-indigo-500/30">
                v1.2-enterprise
              </span>
            </div>
            <div className="flex items-center gap-2 text-xs text-slate-400">
              <span>Fork & Audit of:</span>
              <a
                href="https://github.com/ninhlee99/company-agents"
                target="_blank"
                rel="noreferrer"
                className="text-cyan-400 hover:text-cyan-300 underline inline-flex items-center gap-1 font-mono"
              >
                ninhlee99/company-agents <ExternalLink className="w-3 h-3" />
              </a>
              {hasGeminiKey && (
                <span className="inline-flex items-center gap-1 text-[11px] text-emerald-400 font-medium bg-emerald-500/10 px-1.5 py-0.2 rounded border border-emerald-500/20">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                  Gemini 3.8 Live
                </span>
              )}
            </div>
          </div>
        </div>

        {/* Live Metrics Ticker */}
        <div className="flex flex-wrap items-center gap-3">
          {/* Company Status */}
          <div className={`px-3 py-1 rounded-lg text-xs font-semibold border flex items-center gap-1.5 ${getStatusColor(snapshot.status)}`}>
            <span className="w-2 h-2 rounded-full bg-current animate-pulse"></span>
            Status: {snapshot.status}
          </div>

          {/* Treasury Cash */}
          <div className="px-3 py-1 rounded-lg text-xs bg-slate-900 border border-slate-800 text-slate-200">
            <span className="text-slate-400 block text-[10px] uppercase font-mono">Treasury Cash</span>
            <span className="font-bold font-mono text-emerald-400 text-sm">{formatMoney(snapshot.cash_minor)}</span>
          </div>

          {/* Runway */}
          <div className="px-3 py-1 rounded-lg text-xs bg-slate-900 border border-slate-800 text-slate-200">
            <span className="text-slate-400 block text-[10px] uppercase font-mono">Runway</span>
            <span className="font-bold font-mono text-cyan-400 text-sm">
              {snapshot.runway_days >= 999 ? '∞ Profitable' : `${snapshot.runway_days} Days`}
            </span>
          </div>

          {/* Cycle # */}
          <div className="px-3 py-1 rounded-lg text-xs bg-slate-900 border border-slate-800 text-slate-200">
            <span className="text-slate-400 block text-[10px] uppercase font-mono">Cycle Count</span>
            <span className="font-bold font-mono text-purple-400 text-sm">#{snapshot.cycle_count}</span>
          </div>

          {/* Run Cycle Button */}
          <button
            onClick={onRunCycle}
            disabled={isRunningCycle}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-semibold text-white shadow-md transition-all ${
              isRunningCycle
                ? 'bg-slate-700 cursor-not-allowed text-slate-400'
                : 'bg-gradient-to-r from-indigo-600 via-indigo-500 to-cyan-500 hover:from-indigo-500 hover:to-cyan-400 hover:shadow-indigo-500/25 active:scale-95'
            }`}
          >
            <RefreshCw className={`w-4 h-4 ${isRunningCycle ? 'animate-spin' : ''}`} />
            {isRunningCycle ? 'Đang thực thi chu kỳ...' : 'Chạy Chu Kỳ Mới'}
          </button>
        </div>
      </div>
    </header>
  );
};
