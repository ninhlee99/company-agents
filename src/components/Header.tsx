import React from 'react';
import { CompanySnapshot } from '../types/company';
import { 
  Building2, 
  Sparkles,
  ShieldCheck,
  FileText
} from 'lucide-react';

interface HeaderProps {
  snapshot: CompanySnapshot;
  activeContractsCount?: number;
  onNavigateToOrder?: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  snapshot,
  activeContractsCount = 0,
  onNavigateToOrder,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  return (
    <header className="px-4 lg:px-8 py-3.5 border-b border-slate-800/80 bg-slate-950/90 backdrop-blur-md sticky top-0 z-40">
      <div className="max-w-5xl mx-auto flex items-center justify-between gap-4">
        {/* Brand & Status */}
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center text-slate-200 shadow-sm">
            <Building2 className="w-5 h-5 text-blue-400" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-white text-sm tracking-tight">Cổng Thuê Khoán Doanh Nghiệp AI</span>
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-emerald-950/60 text-emerald-400 border border-emerald-800/50">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                Tự Động 24/7
              </span>
            </div>
            <p className="text-[11px] text-slate-400 hidden sm:block">
              Hệ thống nhận hợp đồng, tự phân công nhân sự và bàn giao sản phẩm tự động
            </p>
          </div>
        </div>

        {/* Right Financial & Action Elements */}
        <div className="flex items-center gap-3">
          {/* Contracts In Progress Metric */}
          <div className="hidden sm:flex items-center gap-4 px-3.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-xs">
            <div>
              <span className="text-slate-400 text-[10px] block">Hợp Đồng Đang Xử Lý</span>
              <span className="font-semibold text-white font-mono text-xs">{activeContractsCount} đơn</span>
            </div>
            <div className="w-px h-6 bg-slate-800"></div>
            <div>
              <span className="text-slate-400 text-[10px] block">Quỹ Bảo Chứng Dịch Vụ</span>
              <span className="font-semibold text-emerald-400 font-mono text-xs">{formatMoney(snapshot.cash_minor)}</span>
            </div>
          </div>

          {/* New Order CTA */}
          {onNavigateToOrder && (
            <button
              onClick={onNavigateToOrder}
              className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs shadow-sm transition-all active:scale-95"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>+ Đặt Hàng Dịch Vụ</span>
            </button>
          )}
        </div>
      </div>
    </header>
  );
};
