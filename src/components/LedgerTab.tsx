import React from 'react';
import { CompanySnapshot, LedgerEntry } from '../types/company';
import { DollarSign, Landmark, ArrowUpRight, ArrowDownLeft, ShieldCheck, PieChart, FileText, CheckCircle2 } from 'lucide-react';

interface LedgerTabProps {
  snapshot: CompanySnapshot;
  ledger: LedgerEntry[];
  employees: { id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[];
  experiments: { id: string; name: string; budget_minor: number; startCycle: number; status: string; roi_bps: number }[];
}

export const LedgerTab: React.FC<LedgerTabProps> = ({
  snapshot,
  ledger,
  employees,
  experiments,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 2,
    }).format(minor / 100);
  };

  // Financial summary calculations
  const totalAssets = snapshot.cash_minor;
  const totalEquity = snapshot.cash_minor; // In early-stage media co with zero debt
  const totalLiabilities = 0; // Debt-free autonomous co
  const monthlyProfit = snapshot.revenue_minor - snapshot.expenses_minor;

  return (
    <div className="space-y-6 max-w-6xl mx-auto pb-16">
      {/* Intro Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 shadow-lg">
        <div>
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Landmark className="w-5 h-5 text-emerald-400" />
            Lõi Kinh Tế & Sổ Cái Kép (Double-Entry Ledger)
          </h2>
          <p className="text-slate-400 text-xs mt-1">
            Triển khai nguyên mẫu từ <code className="text-cyan-400 font-mono">crates/economic-core</code>. Mọi giao dịch chi tiêu của AI Agent đều phải tuân thủ nguyên lý bất biến: Nợ = Có.
          </p>
        </div>
        <div className="flex items-center gap-2 text-xs font-mono bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 px-3 py-1.5 rounded-lg">
          <ShieldCheck className="w-4 h-4" />
          <span>Fiduciary Rule: Tiền mặt không thể âm</span>
        </div>
      </div>

      {/* Balance Sheet Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        {/* Treasury Cash */}
        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 shadow-md">
          <span className="text-slate-400 text-xs font-mono uppercase">Tiền Mặt Khả Dụng (Assets)</span>
          <div className="text-2xl font-black text-emerald-400 font-mono mt-1">
            {formatMoney(snapshot.cash_minor)}
          </div>
          <div className="text-[11px] text-slate-500 mt-1 flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
            Tài sản thanh khoản tức thì
          </div>
        </div>

        {/* Monthly Revenue */}
        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 shadow-md">
          <span className="text-slate-400 text-xs font-mono uppercase">Doanh Thu Tháng (MRR)</span>
          <div className="text-2xl font-black text-cyan-400 font-mono mt-1">
            {formatMoney(snapshot.revenue_minor)}
          </div>
          <div className="text-[11px] text-slate-400 mt-1">
            Từ Creator Media & Affiliate Deals
          </div>
        </div>

        {/* Monthly Expenses */}
        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 shadow-md">
          <span className="text-slate-400 text-xs font-mono uppercase">Chi Phí Tháng (Monthly Burn)</span>
          <div className="text-2xl font-black text-rose-400 font-mono mt-1">
            {formatMoney(snapshot.expenses_minor)}
          </div>
          <div className="text-[11px] text-slate-400 mt-1">
            Lương talent, GPU cluster & Tools
          </div>
        </div>

        {/* Net Monthly Profit */}
        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 shadow-md">
          <span className="text-slate-400 text-xs font-mono uppercase">Lợi Nhuận Ròng Tháng</span>
          <div className={`text-2xl font-black font-mono mt-1 ${monthlyProfit >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
            {monthlyProfit >= 0 ? `+${formatMoney(monthlyProfit)}` : formatMoney(monthlyProfit)}
          </div>
          <div className="text-[11px] text-slate-400 mt-1">
            {monthlyProfit >= 0 ? 'Dương dòng tiền (Self-sustaining)' : 'Đang đốt quỹ dự phòng'}
          </div>
        </div>
      </div>

      {/* Runway & Experiment Budget Row */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Runway Analysis */}
        <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <span className="font-bold text-white text-sm">Thời Gian Sống Còn (Runway Days)</span>
            <span className="text-xs font-mono text-cyan-400 font-bold">
              {snapshot.runway_days >= 999 ? 'Không giới hạn (Lãi ròng)' : `${snapshot.runway_days} Ngày`}
            </span>
          </div>
          <div className="w-full bg-slate-800 h-2.5 rounded-full overflow-hidden">
            <div
              className={`h-full rounded-full transition-all ${
                snapshot.runway_days > 90 ? 'bg-emerald-500' : snapshot.runway_days > 30 ? 'bg-amber-500' : 'bg-rose-500'
              }`}
              style={{ width: `${Math.min(100, (snapshot.runway_days / 365) * 100)}%` }}
            ></div>
          </div>
          <p className="text-xs text-slate-400">
            {snapshot.runway_days > 90
              ? 'Mức an toàn cao: Đạt chuẩn cho phép CEO & Growth chi tiêu thử nghiệm.'
              : 'Cảnh báo thanh khoản: Governor sẽ tự động khóa mọi đề xuất chi tiêu mới.'}
          </p>
        </div>

        {/* Experiment Fund Pool */}
        <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <span className="font-bold text-white text-sm">Quỹ Thử Nghiệm Bounded (R&D Pool)</span>
            <span className="text-xs font-mono text-purple-400 font-bold">
              {formatMoney(snapshot.experiment_budget_minor)}
            </span>
          </div>
          <div className="w-full bg-slate-800 h-2.5 rounded-full overflow-hidden">
            <div
              className="h-full rounded-full bg-purple-500 transition-all"
              style={{ width: `${Math.min(100, (snapshot.experiment_budget_minor / (snapshot.cash_minor || 1)) * 500)}%` }}
            ></div>
          </div>
          <p className="text-xs text-slate-400">
            Quỹ cách ly độc lập để Experiment Agent và Growth Agent thử nghiệm A/B mà không ảnh hưởng quỹ tiền mặt sống còn.
          </p>
        </div>
      </div>

      {/* General Journal (Sổ Nhật Ký Kế Toán Kép) */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-xl">
        <div className="p-4 bg-slate-950 border-b border-slate-800 flex items-center justify-between">
          <div>
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <FileText className="w-4 h-4 text-emerald-400" />
              Sổ Nhật Ký Kế Toán Kép (Double-Entry General Journal)
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">Bút toán định khoản tự động phát sinh từ các Cycle được Governor phê chuẩn</p>
          </div>
          <span className="text-xs font-mono text-slate-400">Tổng bút toán: {ledger.length}</span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse font-mono">
            <thead>
              <tr className="bg-slate-950/80 text-slate-400 uppercase text-[11px] border-b border-slate-800">
                <th className="p-3 pl-5">Thời Gian</th>
                <th className="p-3">Chu Kỳ</th>
                <th className="p-3">Mô Tả Giao Dịch</th>
                <th className="p-3">Tài Khoản NỢ (Debit)</th>
                <th className="p-3">Tài Khoản CÓ (Credit)</th>
                <th className="p-3 pr-5 text-right">Số Tiền</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {ledger.map((entry) => (
                <tr key={entry.id} className="hover:bg-slate-800/30">
                  <td className="p-3 pl-5 text-slate-500 text-[11px]">
                    {new Date(entry.timestamp).toLocaleDateString()} {new Date(entry.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  </td>
                  <td className="p-3 text-purple-400 font-bold">#{entry.cycle}</td>
                  <td className="p-3 text-white font-sans text-xs">{entry.description}</td>
                  <td className="p-3 text-cyan-300 font-semibold">{entry.debitAccount}</td>
                  <td className="p-3 text-amber-300 font-semibold">{entry.creditAccount}</td>
                  <td className="p-3 pr-5 text-right font-bold text-emerald-400 text-xs">
                    {formatMoney(entry.amount_minor)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
