import React, { useState } from 'react';
import { CompanySnapshot, LedgerEntry } from '../types/company';
import { 
  Wallet, 
  TrendingUp, 
  TrendingDown, 
  ArrowDownLeft, 
  ArrowUpRight, 
  ShieldCheck, 
  Sliders, 
  Calendar,
  Users
} from 'lucide-react';

interface ManageFinancesProps {
  snapshot: CompanySnapshot;
  ledger: LedgerEntry[];
  employees: { id: string; role: string; name: string; salary_minor: number; hiredAtCycle: number }[];
}

export const ManageFinances: React.FC<ManageFinancesProps> = ({
  snapshot,
  ledger,
  employees,
}) => {
  const [filterType, setFilterType] = useState<'All' | 'Income' | 'Expense'>('All');

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 2,
    }).format(minor / 100);
  };

  const filteredEntries = ledger.filter((item) => {
    if (filterType === 'Income') {
      return item.creditAccount.toLowerCase().includes('revenue') || item.creditAccount.toLowerCase().includes('equity');
    }
    if (filterType === 'Expense') {
      return item.debitAccount.toLowerCase().includes('expense') || item.debitAccount.toLowerCase().includes('r&d') || item.debitAccount.toLowerCase().includes('payroll');
    }
    return true;
  });

  return (
    <div className="space-y-6 max-w-5xl mx-auto pb-12">
      {/* Header Info */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-md">
        <div>
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Wallet className="w-5 h-5 text-emerald-400" />
            Quản Lý Ví Tiền & Dòng Tiền Thu Chi
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Giao diện minh bạch như sao kê ngân hàng. Mọi khoản thu chi của AI đều được ghi lại tức thì.
          </p>
        </div>

        <div className="flex items-center gap-2 text-xs font-mono text-emerald-400 bg-emerald-500/10 px-3 py-1.5 rounded-lg border border-emerald-500/20">
          <ShieldCheck className="w-4 h-4" />
          <span>Kế toán minh bạch 100%</span>
        </div>
      </div>

      {/* 3 Summary Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-1">
          <span className="text-xs text-slate-400 block font-medium">Số Dư Hiện Tại Trong Ví</span>
          <div className="text-2xl font-black text-white font-mono">{formatMoney(snapshot.cash_minor)}</div>
          <span className="text-[11px] text-emerald-400 block pt-1">Được bảo toàn bởi Governor</span>
        </div>

        <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-1">
          <span className="text-xs text-slate-400 block font-medium">Tổng Thu Hàng Tháng</span>
          <div className="text-2xl font-black text-cyan-400 font-mono">{formatMoney(snapshot.revenue_minor)}</div>
          <span className="text-[11px] text-slate-400 block pt-1">Doanh thu affiliate & video</span>
        </div>

        <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-1">
          <span className="text-xs text-slate-400 block font-medium">Tổng Chi Hàng Tháng</span>
          <div className="text-2xl font-black text-rose-400 font-mono">{formatMoney(snapshot.expenses_minor)}</div>
          <span className="text-[11px] text-slate-400 block pt-1">Lương, máy chủ GPU & API</span>
        </div>
      </div>

      {/* Transaction History (Banking Style) */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-lg space-y-3 p-5">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-slate-800 pb-3">
          <h3 className="font-bold text-white text-base">Lịch Sử Biến Động Số Dư (Sao Kê)</h3>

          <div className="flex items-center gap-1.5 bg-slate-950 p-1 rounded-lg border border-slate-800">
            {(['All', 'Income', 'Expense'] as const).map((filter) => (
              <button
                key={filter}
                onClick={() => setFilterType(filter)}
                className={`px-3 py-1 rounded text-xs font-semibold transition-all ${
                  filterType === filter
                    ? 'bg-indigo-600 text-white shadow-sm'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                {filter === 'All' ? 'Tất cả' : filter === 'Income' ? 'Tiền Vào (+)' : 'Tiền Ra (-)'}
              </button>
            ))}
          </div>
        </div>

        {/* Transactions List */}
        <div className="space-y-2">
          {filteredEntries.map((tx) => {
            const isIncome = tx.creditAccount.toLowerCase().includes('revenue') || tx.creditAccount.toLowerCase().includes('equity');
            return (
              <div
                key={tx.id}
                className="p-3.5 rounded-lg bg-slate-950/70 border border-slate-800/80 hover:border-slate-700 transition-all flex items-center justify-between gap-4 text-xs"
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`w-9 h-9 rounded-xl flex items-center justify-center shrink-0 ${
                      isIncome ? 'bg-emerald-500/10 text-emerald-400' : 'bg-rose-500/10 text-rose-400'
                    }`}
                  >
                    {isIncome ? <ArrowDownLeft className="w-5 h-5" /> : <ArrowUpRight className="w-5 h-5" />}
                  </div>
                  <div>
                    <span className="font-bold text-white text-sm block">{tx.description}</span>
                    <div className="text-[11px] text-slate-400 flex items-center gap-2 mt-0.5">
                      <span>{new Date(tx.timestamp).toLocaleDateString()}</span>
                      <span>•</span>
                      <span className="font-mono text-purple-400">Chu kỳ #{tx.cycle}</span>
                      <span>•</span>
                      <span className="text-slate-500">{isIncome ? tx.creditAccount : tx.debitAccount}</span>
                    </div>
                  </div>
                </div>

                <div className="text-right shrink-0">
                  <span
                    className={`font-mono font-bold text-sm block ${
                      isIncome ? 'text-emerald-400' : 'text-rose-400'
                    }`}
                  >
                    {isIncome ? `+${formatMoney(tx.amount_minor)}` : `-${formatMoney(tx.amount_minor)}`}
                  </span>
                  <span className="text-[10px] text-slate-500 font-mono">Thành công</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Payroll / Employees Card */}
      {employees.length > 0 && (
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="font-bold text-white text-sm flex items-center gap-2">
              <Users className="w-4 h-4 text-indigo-400" />
              Bảng Lương Đội Ngũ Nhân Sự
            </h3>
            <span className="text-xs text-slate-400">Chi trả tự động qua chu kỳ</span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
            {employees.map((emp) => (
              <div key={emp.id} className="p-3 bg-slate-950/70 border border-slate-800 rounded-lg flex items-center justify-between">
                <div>
                  <span className="font-bold text-white block">{emp.name}</span>
                  <span className="text-[11px] text-slate-400">{emp.role}</span>
                </div>
                <div className="text-right">
                  <span className="font-mono font-bold text-rose-400 text-xs block">{formatMoney(emp.salary_minor)}/tháng</span>
                  <span className="text-[10px] text-slate-500 font-mono">Gia nhập kỳ #{emp.hiredAtCycle}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
