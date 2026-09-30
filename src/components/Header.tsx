import React from 'react';
import { CompanySnapshot, AutonomousSettings } from '../types/company';
import { 
  Building2, 
  RotateCw, 
  Play, 
  Pause,
  ArrowUpRight
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
    <header className="px-4 lg:px-8 py-3 border-b border-slate-800/80 bg-slate-950/90 backdrop-blur-md sticky top-0 z-40">
      <div className="max-w-6xl mx-auto flex items-center justify-between gap-4">
        {/* Brand */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-slate-900 border border-slate-700 flex items-center justify-center text-slate-200">
            <Building2 className="w-4 h-4 text-blue-400" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-semibold text-white text-sm tracking-tight">Company OS</span>
              <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium ${
                snapshot.status === 'Active' || snapshot.status === 'Growth'
                  ? 'bg-emerald-950/60 text-emerald-400 border border-emerald-800/50'
                  : 'bg-amber-950/60 text-amber-400 border border-amber-800/50'
              }`}>
                <span className="w-1.5 h-1.5 rounded-full bg-current"></span>
                {snapshot.status}
              </span>
            </div>
          </div>
        </div>

        {/* Center/Right Financial Metrics & Actions */}
        <div className="flex items-center gap-3">
          {/* Cash Balance */}
          <div className="hidden sm:flex items-center gap-4 px-3 py-1.5 rounded-lg bg-slate-900/60 border border-slate-800 text-xs">
            <div>
              <span className="text-slate-400 text-[11px] block">Quỹ Tiền Mặt</span>
              <span className="font-semibold text-emerald-400 font-mono text-sm">{formatMoney(snapshot.cash_minor)}</span>
            </div>
            <div className="w-px h-6 bg-slate-800"></div>
            <div>
              <span className="text-slate-400 text-[11px] block">Doanh Thu Tháng</span>
              <span className="font-semibold text-white font-mono text-sm">{formatMoney(snapshot.revenue_minor)}</span>
            </div>
          </div>

          {/* Auto-Pilot Toggle Button */}
          {onToggleAutoPilot && (
            <button
              onClick={onToggleAutoPilot}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors border ${
                isAuto
                  ? 'bg-blue-950/70 border-blue-800 text-blue-300 hover:bg-blue-900/80'
                  : 'bg-slate-900 border-slate-700 text-slate-300 hover:bg-slate-800'
              }`}
            >
              {isAuto ? <Pause className="w-3.5 h-3.5 text-blue-400" /> : <Play className="w-3.5 h-3.5 text-slate-400" />}
              <span>{isAuto ? 'Auto-Pilot: Đang Bật' : 'Bật Auto-Pilot'}</span>
            </button>
          )}

          {/* Manual Run Cycle */}
          <button
            onClick={onRunCycle}
            disabled={isRunningCycle}
            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 disabled:bg-slate-800 text-white font-medium text-xs shadow-sm transition-all active:scale-95"
          >
            <RotateCw className={`w-3.5 h-3.5 ${isRunningCycle ? 'animate-spin' : ''}`} />
            <span>{isRunningCycle ? 'Đang chạy...' : 'Chạy Chu Kỳ'}</span>
          </button>

          {/* Mode Switcher */}
          <button
            onClick={() => setUiMode(uiMode === 'basic' ? 'pro' : 'basic')}
            className="px-2.5 py-1.5 rounded-lg text-xs font-medium bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-400 hover:text-slate-200 transition-colors"
          >
            {uiMode === 'basic' ? 'Chế độ Pro' : 'Cơ bản'}
          </button>
        </div>
      </div>
    </header>
  );
};
