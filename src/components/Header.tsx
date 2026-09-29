import React from 'react';
import { CompanySnapshot } from '../types/company';
import { 
  Building2, 
  RotateCw, 
  ExternalLink, 
  Sparkles,
  SlidersHorizontal,
  LayoutDashboard
} from 'lucide-react';

interface HeaderProps {
  snapshot: CompanySnapshot;
  onRunCycle: () => void;
  isRunningCycle: boolean;
  uiMode: 'basic' | 'pro';
  setUiMode: (mode: 'basic' | 'pro') => void;
  hasGeminiKey: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  snapshot,
  onRunCycle,
  isRunningCycle,
  uiMode,
  setUiMode,
  hasGeminiKey,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  return (
    <header className="sticky top-0 z-50 bg-slate-950/90 backdrop-blur-md border-b border-slate-800 px-4 lg:px-8 py-3">
      <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
        {/* Brand & Repo Meta */}
        <div className="flex items-center gap-3 w-full md:w-auto justify-between md:justify-start">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-md shadow-indigo-500/20">
              <Building2 className="w-5 h-5 text-white" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-white text-base tracking-tight">Company Agents AI</span>
                <span className="px-2 py-0.5 text-[10px] rounded-full bg-indigo-500/20 text-indigo-300 font-mono border border-indigo-500/30">
                  Basic &amp; Clean
                </span>
              </div>
              <a
                href="https://github.com/ninhlee99/company-agents"
                target="_blank"
                rel="noreferrer"
                className="text-[11px] text-slate-400 hover:text-cyan-400 inline-flex items-center gap-1 font-mono transition-colors"
              >
                ninhlee99/company-agents <ExternalLink className="w-2.5 h-2.5" />
              </a>
            </div>
          </div>

          {/* Mode Switcher on Mobile */}
          <div className="md:hidden">
            <button
              onClick={() => setUiMode(uiMode === 'basic' ? 'pro' : 'basic')}
              className="px-2.5 py-1 rounded-lg text-xs font-semibold bg-slate-900 border border-slate-800 text-slate-300"
            >
              {uiMode === 'basic' ? '⚙️ Chuyên Sâu' : '✨ Cơ Bản'}
            </button>
          </div>
        </div>

        {/* Right Controls */}
        <div className="flex items-center gap-3 w-full md:w-auto justify-between md:justify-end">
          {/* Quick Metrics */}
          <div className="flex items-center gap-3 text-xs">
            <div className="px-3 py-1 rounded-lg bg-slate-900 border border-slate-800">
              <span className="text-slate-400 text-[10px] block">Kho bạc</span>
              <span className="font-bold font-mono text-emerald-400">{formatMoney(snapshot.cash_minor)}</span>
            </div>
            <div className="px-3 py-1 rounded-lg bg-slate-900 border border-slate-800">
              <span className="text-slate-400 text-[10px] block">Runway</span>
              <span className="font-bold font-mono text-cyan-400">
                {snapshot.runway_days >= 999 ? '∞ Tự lãi' : `${snapshot.runway_days} ngày`}
              </span>
            </div>
          </div>

          {/* Mode Switcher Button (Desktop) */}
          <button
            onClick={() => setUiMode(uiMode === 'basic' ? 'pro' : 'basic')}
            className={`hidden md:flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all ${
              uiMode === 'basic'
                ? 'bg-indigo-600/10 border-indigo-500/30 text-indigo-300 hover:bg-indigo-600/20'
                : 'bg-purple-600/10 border-purple-500/30 text-purple-300 hover:bg-purple-600/20'
            }`}
            title="Chuyển đổi giao diện Cơ bản (Basic) hoặc Nâng cao (Pro)"
          >
            {uiMode === 'basic' ? (
              <>
                <LayoutDashboard className="w-3.5 h-3.5" />
                <span>Giao Diện: Cơ Bản</span>
              </>
            ) : (
              <>
                <SlidersHorizontal className="w-3.5 h-3.5" />
                <span>Giao Diện: Chuyên Sâu (Audit)</span>
              </>
            )}
          </button>

          {/* Quick Run Button */}
          <button
            onClick={onRunCycle}
            disabled={isRunningCycle}
            className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-bold text-white transition-all shadow-sm ${
              isRunningCycle
                ? 'bg-slate-800 text-slate-400 cursor-not-allowed'
                : 'bg-indigo-600 hover:bg-indigo-500 active:scale-95'
            }`}
          >
            <RotateCw className={`w-3.5 h-3.5 ${isRunningCycle ? 'animate-spin' : ''}`} />
            {isRunningCycle ? 'Đang chạy...' : 'Chạy AI'}
          </button>
        </div>
      </div>
    </header>
  );
};
