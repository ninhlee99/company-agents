import React from 'react';
import { CompanySnapshot, AutonomousSettings } from '../types/company';
import { 
  Building2, 
  RotateCw, 
  ExternalLink, 
  Sparkles,
  SlidersHorizontal,
  LayoutDashboard,
  Play,
  Square,
  Cpu,
  Zap,
  ShieldCheck
} from 'lucide-react';

interface HeaderProps {
  snapshot: CompanySnapshot;
  onRunCycle: () => void;
  isRunningCycle: boolean;
  uiMode: 'basic' | 'pro';
  setUiMode: (mode: 'basic' | 'pro') => void;
  hasGeminiKey: boolean;
  autonomousSettings?: AutonomousSettings;
  onToggleAutoPilot?: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  snapshot,
  onRunCycle,
  isRunningCycle,
  uiMode,
  setUiMode,
  hasGeminiKey,
  autonomousSettings,
  onToggleAutoPilot,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const isAuto = autonomousSettings?.isAutoPilotActive ?? false;

  return (
    <header className="px-4 lg:px-8 py-2.5 border-b border-slate-900 bg-slate-950/80 backdrop-blur-md sticky top-0 z-40">
      <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-3">
        {/* Brand & Repo Meta */}
        <div className="flex items-center gap-3 w-full md:w-auto justify-between md:justify-start">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-500 via-purple-600 to-pink-500 flex items-center justify-center shadow-md shadow-indigo-500/20">
              <Building2 className="w-5 h-5 text-white" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-white text-base tracking-tight">Company Agents AI</span>
                <span className="px-2 py-0.5 text-[10px] rounded-full bg-emerald-500/20 text-emerald-300 font-mono border border-emerald-500/30 flex items-center gap-1">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                  Senior Enterprise
                </span>
              </div>
              <div className="flex items-center gap-2 text-[11px] text-slate-400 font-mono">
                <a
                  href="https://github.com/ninhlee99/company-agents"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-cyan-400 inline-flex items-center gap-1 transition-colors"
                >
                  ninhlee99/company-agents <ExternalLink className="w-2.5 h-2.5" />
                </a>
                <span className="text-slate-600">•</span>
                <span className="text-slate-500 flex items-center gap-1">
                  <Cpu className="w-3 h-3 text-cyan-400" /> RAM: &lt;40MB | CPU: 0.2%
                </span>
              </div>
            </div>
          </div>

          {/* Mode Switcher on Mobile */}
          <div className="md:hidden flex items-center gap-2">
            <button
              onClick={() => setUiMode(uiMode === 'basic' ? 'pro' : 'basic')}
              className="px-2.5 py-1 rounded-lg text-xs font-semibold bg-slate-900 border border-slate-800 text-slate-300"
            >
              {uiMode === 'basic' ? '⚙️ Chuyên Sâu' : '✨ Cơ Bản'}
            </button>
          </div>
        </div>

        {/* Right Controls */}
        <div className="flex items-center gap-2.5 w-full md:w-auto justify-between md:justify-end flex-wrap">
          {/* Quick Metrics */}
          <div className="flex items-center gap-2 text-xs">
            <div className="px-3 py-1 rounded-lg bg-slate-900/90 border border-slate-800">
              <span className="text-slate-400 text-[10px] block">Kho bạc Fiduciary</span>
              <span className="font-bold font-mono text-emerald-400">{formatMoney(snapshot.cash_minor)}</span>
            </div>
            <div className="px-3 py-1 rounded-lg bg-slate-900/90 border border-slate-800">
              <span className="text-slate-400 text-[10px] block">Runway</span>
              <span className="font-bold font-mono text-cyan-400">
                {snapshot.runway_days >= 999 ? '∞ Tự lãi' : `${snapshot.runway_days} ngày`}
              </span>
            </div>
          </div>

          {/* 24/7 Autonomous Auto-Pilot Toggle Button */}
          {onToggleAutoPilot && (
            <button
              onClick={onToggleAutoPilot}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold border transition-all ${
                isAuto
                  ? 'bg-emerald-950/80 border-emerald-500/60 text-emerald-300 shadow-lg shadow-emerald-500/20'
                  : 'bg-slate-900 border-slate-800 text-slate-400 hover:text-white hover:border-slate-700'
              }`}
              title={isAuto ? 'Đang tự động kiếm tiền 24/7 (Bấm để tạm dừng)' : 'Bật chế độ AI tự vận hành & tự kiếm tiền 24/7'}
            >
              {isAuto ? (
                <>
                  <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
                  <Zap className="w-3.5 h-3.5 text-emerald-400" />
                  <span>Auto-Pilot: BẬT (24/7)</span>
                </>
              ) : (
                <>
                  <Play className="w-3.5 h-3.5 text-slate-400" />
                  <span>Tự Động 24/7</span>
                </>
              )}
            </button>
          )}

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
                <span>Giao Diện: Chuyên Sâu</span>
              </>
            )}
          </button>

          {/* Manual Run Cycle Button */}
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
            {isRunningCycle ? 'Đang duyệt...' : 'Chạy Kỳ AI'}
          </button>
        </div>
      </div>
    </header>
  );
};
