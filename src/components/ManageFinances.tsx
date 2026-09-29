import React, { useState } from 'react';
import { CompanySnapshot, LedgerEntry, CustomAgent } from '../types/company';
import { AgentProfitabilityLedger } from './AgentProfitabilityLedger';
import { 
  Wallet, 
  ArrowDownLeft, 
  ArrowUpRight, 
  ShieldCheck, 
  Users
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
      maximumFractionDigits: 2,
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
    <div className="space-y-4 max-w-5xl mx-auto pb-10">
      {/* Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
        <div className="flex items-center gap-2">
          <Wallet className="w-5 h-5 text-emerald-400" />
          <div>
            <h2 className="text-base font-bold text-white">Ví Tiền &amp; Sao Kê Thu Chi</h2>
            <p className="text-xs text-slate-400">Minh bạch 100% theo nguyên tắc kế toán kép</p>
          </div>
        </div>

        <div className="flex items-center gap-1.5 text-xs text-emerald-400 font-mono bg-emerald-500/10 px-2.5 py-1 rounded-lg border border-emerald-500/20">
          <ShieldCheck className="w-3.5 h-3.5" />
          <span>Governor Đảm Bảo</span>
        </div>
      </div>

      {/* 3 Summary Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800">
          <span className="text-xs text-slate-400 block font-medium">Số Dư Kho Bạc</span>
          <div className="text-2xl font-black text-white font-mono mt-1">{formatMoney(snapshot.cash_minor)}</div>
          <span className="text-[11px] text-cyan-400 mt-0.5 block">Sống được {snapshot.runway_days} ngày</span>
        </div>

        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800">
          <span className="text-xs text-slate-400 block font-medium">Tổng Thu Hàng Tháng</span>
          <div className="text-2xl font-black text-cyan-400 font-mono mt-1">{formatMoney(snapshot.revenue_minor)}</div>
          <span className="text-[11px] text-slate-400 mt-0.5 block">Affiliate &amp; Content Ads</span>
        </div>

        <div className="p-4 rounded-xl bg-slate-900 border border-slate-800">
          <span className="text-xs text-slate-400 block font-medium">Tổng Chi Hàng Tháng</span>
          <div className="text-2xl font-black text-rose-400 font-mono mt-1">{formatMoney(snapshot.expenses_minor)}</div>
          <span className="text-[11px] text-slate-400 mt-0.5 block">Lương AI, GPU &amp; API</span>
        </div>
      </div>

      {/* Agent Profitability Ledger (Granular Employee ROI & Pipeline Cross-Reference) */}
      <AgentProfitabilityLedger snapshot={snapshot} agents={agents} />

      {/* Transaction History (Banking Style) */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-sm p-4 space-y-3">
        <div className="flex items-center justify-between border-b border-slate-800 pb-2">
          <span className="font-bold text-white text-xs">Biến Động Số Dư (Sao Kê)</span>

          <div className="flex gap-1 bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs">
            {(['All', 'Income', 'Expense'] as const).map((filter) => (
              <button
                key={filter}
                onClick={() => setFilterType(filter)}
                className={`px-2.5 py-1 rounded text-xs font-semibold transition-all ${
                  filterType === filter ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                }`}
              >
                {filter === 'All' ? 'Tất cả' : filter === 'Income' ? 'Tiền vào (+)' : 'Tiền ra (-)'}
              </button>
            ))}
          </div>
        </div>

        {/* Transactions List */}
        <div className="space-y-1.5 max-h-96 overflow-y-auto pr-1">
          {filteredEntries.map((tx) => {
            const isIncome = tx.creditAccount.toLowerCase().includes('revenue') || tx.creditAccount.toLowerCase().includes('equity');
            return (
              <div
                key={tx.id}
                className="p-3 rounded-lg bg-slate-950 border border-slate-800/80 hover:border-slate-700 transition-all flex items-center justify-between gap-3 text-xs"
              >
                <div className="flex items-center gap-2.5 truncate">
                  <div
                    className={`w-7 h-7 rounded-lg flex items-center justify-center shrink-0 ${
                      isIncome ? 'bg-emerald-500/10 text-emerald-400' : 'bg-rose-500/10 text-rose-400'
                    }`}
                  >
                    {isIncome ? <ArrowDownLeft className="w-4 h-4" /> : <ArrowUpRight className="w-4 h-4" />}
                  </div>
                  <div className="truncate">
                    <span className="font-semibold text-white block truncate">{tx.description}</span>
                    <span className="text-[10px] text-slate-500 font-mono">
                      Kỳ #{tx.cycle} • {new Date(tx.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </span>
                  </div>
                </div>

                <div className="text-right shrink-0">
                  <span
                    className={`font-mono font-bold text-xs block ${
                      isIncome ? 'text-emerald-400' : 'text-rose-400'
                    }`}
                  >
                    {isIncome ? `+${formatMoney(tx.amount_minor)}` : `-${formatMoney(tx.amount_minor)}`}
                  </span>
                  <span className="text-[10px] text-slate-500 font-mono">Khớp 100%</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Payroll / Employees Card */}
      {employees.length > 0 && (
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-2">
          <div className="flex items-center justify-between">
            <span className="font-bold text-white text-xs flex items-center gap-1.5">
              <Users className="w-4 h-4 text-indigo-400" /> Bảng Lương Nhân Sự AI ({employees.length} vị trí)
            </span>
            <span className="text-[11px] text-slate-400">Tự động chi trả hàng tháng</span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs">
            {employees.map((emp) => (
              <div key={emp.id} className="p-2.5 bg-slate-950 border border-slate-800 rounded-lg flex items-center justify-between">
                <div>
                  <span className="font-bold text-white block">{emp.name}</span>
                  <span className="text-[11px] text-slate-400">{emp.role}</span>
                </div>
                <div className="text-right">
                  <span className="font-mono font-bold text-rose-400 text-xs block">
                    ${(emp.salary_minor / 100).toLocaleString()}/th
                  </span>
                  <span className="text-[10px] text-slate-500 font-mono">Kỳ #{emp.hiredAtCycle}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
