import React from 'react';
import { CompanySnapshot } from '../types/company';
import { 
  Building2, 
  Sparkles,
  ShieldCheck,
  TrendingUp,
  Wallet,
  DollarSign,
  Briefcase
} from 'lucide-react';

interface HeaderProps {
  snapshot: CompanySnapshot;
  activeContractsCount?: number;
  onNavigateToNewContract?: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  snapshot,
  activeContractsCount = 0,
  onNavigateToNewContract,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const netMonthly = snapshot.revenue_minor - snapshot.expenses_minor;

  return (
    <header className="px-4 lg:px-8 py-3.5 border-b border-slate-800/80 bg-slate-950/95 backdrop-blur-md sticky top-0 z-40">
      <div className="max-w-6xl mx-auto flex items-center justify-between gap-4">
        {/* Brand & Executive Identity */}
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center text-slate-200 shadow-sm">
            <Building2 className="w-5 h-5 text-blue-400" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-white text-base tracking-tight">NEXUS CORP</span>
              <span className="text-slate-500 font-mono text-xs hidden lg:inline">• AI Enterprise</span>
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-blue-950/60 text-blue-400 border border-blue-800/50">
                👔 Chủ Tịch / Founder
              </span>
              <span className="hidden md:inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-emerald-950/60 text-emerald-400 border border-emerald-800/50">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                Tự Động 24/7
              </span>
            </div>
            <p className="text-[11px] text-slate-400 hidden sm:block">
              Tập đoàn AI tự trị: Thống kê tài chính, đội ngũ nhân sự, quy trình & hợp đồng kinh doanh
            </p>
          </div>
        </div>

        {/* Right Executive Financial Metrics & Quick Action */}
        <div className="flex items-center gap-3">
          {/* Key Metrics */}
          <div className="hidden sm:flex items-center gap-4 px-3.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-xs">
            <div>
              <span className="text-slate-400 text-[10px] block">Quỹ Tiền Mặt</span>
              <span className="font-semibold text-emerald-400 font-mono text-xs">{formatMoney(snapshot.cash_minor)}</span>
            </div>
            <div className="w-px h-6 bg-slate-800"></div>
            <div>
              <span className="text-slate-400 text-[10px] block">Doanh Thu Tháng</span>
              <span className="font-semibold text-white font-mono text-xs">{formatMoney(snapshot.revenue_minor)}</span>
            </div>
            <div className="w-px h-6 bg-slate-800"></div>
            <div>
              <span className="text-slate-400 text-[10px] block">Lợi Nhuận Ròng</span>
              <span className={`font-semibold font-mono text-xs ${netMonthly >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
                {netMonthly >= 0 ? `+${formatMoney(netMonthly)}` : formatMoney(netMonthly)}
              </span>
            </div>
          </div>

          {/* Quick Create Deal/Contract Button */}
          {onNavigateToNewContract && (
            <button
              onClick={onNavigateToNewContract}
              className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs shadow-sm transition-all active:scale-95"
            >
              <Briefcase className="w-3.5 h-3.5" />
              <span>+ Thêm Hợp Đồng / Deal Mới</span>
            </button>
          )}
        </div>
      </div>
    </header>
  );
};
