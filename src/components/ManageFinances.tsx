import React, { useState } from 'react';
import { CompanySnapshot, LedgerEntry, CustomAgent } from '../types/company';
import { 
  Wallet, 
  ArrowDownLeft, 
  ArrowUpRight, 
  ShieldCheck, 
  Users,
  Receipt,
  DollarSign
} from 'lucide-react';

interface ManageFinancesProps {
  snapshot: CompanySnapshot;
  ledger: LedgerEntry[];
  employees: { id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[];
  agents?: CustomAgent[];
}

export const ManageFinances: React.FC<ManageFinancesProps> = ({
  snapshot,
  ledger,
  employees,
  agents = [],
}) => {
  const [filterType, setFilterType] = useState<'All' | 'Income' | 'Expense'>('All');

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const filteredEntries = ledger.filter((item) => {
    if (filterType === 'Income') {
      return item.creditAccount.toLowerCase().includes('revenue') || item.creditAccount.toLowerCase().includes('equity');
    }
    if (filterType === 'Expense') {
      return item.debitAccount.toLowerCase().includes('expense') || item.debitAccount.toLowerCase().includes('r&d') || item.debitAccount.toLowerCase().includes('payroll') || item.debitAccount.toLowerCase().includes('tuyển');
    }
    return true;
  });

  return (
    <div className="space-y-5 pb-12">
      {/* Top Header */}
      <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 flex items-center justify-between">
        <div>
          <h3 className="text-sm font-semibold text-white">Quản Trị Tài Chính &amp; Sổ Cái Kế Toán</h3>
          <p className="text-xs text-slate-400">Minh bạch 100% dòng tiền theo nguyên tắc kế toán kép (Double-Entry Bookkeeping)</p>
        </div>
        <div className="flex items-center gap-1.5 text-xs text-emerald-400 font-mono bg-emerald-950/60 px-2.5 py-1 rounded-lg border border-emerald-800/40">
          <ShieldCheck className="w-3.5 h-3.5" />
          <span>Governor Đảm Bảo</span>
        </div>
      </div>

      {/* 4 Summary Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3.5">
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
          <span className="text-xs text-slate-400 font-medium block">Số Dư Kho Bạc</span>
          <div className="text-xl font-bold text-white font-mono mt-1">{formatMoney(snapshot.cash_minor)}</div>
          <span className="text-[11px] text-emerald-400 mt-0.5 block">Runway: {snapshot.runway_days} ngày sống còn</span>
        </div>

        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
          <span className="text-xs text-slate-400 font-medium block">Thu Nhập Hàng Tháng</span>
          <div className="text-xl font-bold text-blue-400 font-mono mt-1">{formatMoney(snapshot.revenue_minor)}</div>
          <span className="text-[11px] text-slate-400 mt-0.5 block">Affiliate &amp; Media Payouts</span>
        </div>

        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
          <span className="text-xs text-slate-400 font-medium block">Chi Phí Hàng Tháng</span>
          <div className="text-xl font-bold text-amber-400 font-mono mt-1">{formatMoney(snapshot.expenses_minor)}</div>
          <span className="text-[11px] text-slate-400 mt-0.5 block">Lương nhân sự ({agents.length}) &amp; API Cloud</span>
        </div>
      </div>

      {/* Double-Entry General Ledger */}
      <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-3">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800 pb-3">
          <div className="flex items-center gap-2">
            <Receipt className="w-4 h-4 text-blue-400" />
            <h4 className="text-sm font-semibold text-white">Sổ Cái Giao Dịch Doanh Nghiệp ({filteredEntries.length})</h4>
          </div>

          <div className="flex items-center gap-1 bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs">
            {(['All', 'Income', 'Expense'] as const).map((t) => (
              <button
                key={t}
                onClick={() => setFilterType(t)}
                className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                  filterType === t ? 'bg-slate-800 text-white shadow-sm' : 'text-slate-400 hover:text-white'
                }`}
              >
                {t === 'All' ? 'Tất cả' : t === 'Income' ? 'Thu nhập (+)' : 'Chi phí (-)'}
              </button>
            ))}
          </div>
        </div>

        {/* Ledger Table */}
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-slate-800 text-slate-400 text-[11px]">
                <th className="py-2 px-2.5">Thời Gian</th>
                <th className="py-2 px-2.5">Mô Tả Nghiệp Vụ</th>
                <th className="py-2 px-2.5">Tài Khoản Nợ (Debit)</th>
                <th className="py-2 px-2.5">Tài Khoản Có (Credit)</th>
                <th className="py-2 px-2.5 text-right">Số Tiền</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {filteredEntries.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-6 text-center text-slate-500">
                    Chưa có giao dịch phát sinh trong sổ cái.
                  </td>
                </tr>
              ) : (
                filteredEntries.map((tx) => {
                  const isRevenue = tx.creditAccount.toLowerCase().includes('revenue') || tx.creditAccount.toLowerCase().includes('equity');
                  return (
                    <tr key={tx.id} className="hover:bg-slate-800/30 transition-colors">
                      <td className="py-2.5 px-2.5 font-mono text-[10px] text-slate-500 whitespace-nowrap">
                        {new Date(tx.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })} • Kỳ #{tx.cycle}
                      </td>
                      <td className="py-2.5 px-2.5 font-medium text-slate-200">
                        {tx.description}
                      </td>
                      <td className="py-2.5 px-2.5 text-[11px] text-slate-400">
                        {tx.debitAccount}
                      </td>
                      <td className="py-2.5 px-2.5 text-[11px] text-slate-400">
                        {tx.creditAccount}
                      </td>
                      <td className="py-2.5 px-2.5 text-right font-mono font-semibold whitespace-nowrap">
                        <span className={isRevenue ? 'text-emerald-400' : 'text-slate-300'}>
                          {isRevenue ? '+' : '-'}{formatMoney(tx.amount_minor)}
                        </span>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
