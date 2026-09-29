import React from 'react';
import { CompanySnapshot, GovernedProposal, ExecutionReceipt } from '../types/company';
import { 
  Play, 
  RotateCw, 
  CheckCircle2, 
  AlertTriangle, 
  TrendingUp, 
  TrendingDown, 
  Wallet, 
  Calendar, 
  Users, 
  Sparkles,
  ArrowRight,
  ShieldCheck,
  XCircle,
  HelpCircle
} from 'lucide-react';

interface BasicDashboardProps {
  snapshot: CompanySnapshot;
  recentProposals: GovernedProposal[];
  recentReceipts: ExecutionReceipt[];
  onRunCycle: () => void;
  isRunningCycle: boolean;
  onOverride: (proposalId: string, decision: 'Approve' | 'Reject') => void;
  onNavigate: (view: 'dashboard' | 'agents' | 'finances' | 'create-content') => void;
}

export const BasicDashboard: React.FC<BasicDashboardProps> = ({
  snapshot,
  recentProposals,
  recentReceipts,
  onRunCycle,
  isRunningCycle,
  onOverride,
  onNavigate,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const netMonthly = snapshot.revenue_minor - snapshot.expenses_minor;
  const pendingApprovals = recentProposals.filter((p) => p.decision === 'EscalateToHuman' && !p.executed);

  const getStatusBadge = () => {
    if (snapshot.status === 'Growth' || snapshot.status === 'Active') {
      return {
        text: 'Hoạt động tốt (Khỏe mạnh)',
        bg: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
        dot: 'bg-emerald-400',
      };
    }
    if (snapshot.status === 'Warning' || snapshot.status === 'CostControl') {
      return {
        text: 'Cần chú ý chi tiêu',
        bg: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        dot: 'bg-amber-400',
      };
    }
    return {
      text: 'Nguy hiểm (Thiếu tiền mặt)',
      bg: 'bg-rose-500/10 text-rose-400 border-rose-500/20',
      dot: 'bg-rose-400',
    };
  };

  const statusInfo = getStatusBadge();

  return (
    <div className="space-y-6 max-w-5xl mx-auto pb-12">
      {/* Top Banner: One-Click Action */}
      <div className="bg-slate-900 border border-slate-800 rounded-2xl p-6 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-xl">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <span className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold border ${statusInfo.bg}`}>
              <span className={`w-2 h-2 rounded-full ${statusInfo.dot} animate-pulse`}></span>
              {statusInfo.text}
            </span>
            <span className="text-xs text-slate-400 font-mono">Chu kỳ #{snapshot.cycle_count}</span>
          </div>
          <h1 className="text-xl md:text-2xl font-bold text-white tracking-tight">
            Công ty Agents AI của bạn đang vận hành
          </h1>
          <p className="text-xs md:text-sm text-slate-400">
            9 nhân sự AI đang tự động theo dõi tài chính, sáng tạo nội dung và tìm kiếm cơ hội doanh thu.
          </p>
        </div>

        <button
          onClick={onRunCycle}
          disabled={isRunningCycle}
          className={`flex items-center justify-center gap-2 px-6 py-3.5 rounded-xl font-bold text-sm text-white shadow-lg transition-all shrink-0 w-full md:w-auto ${
            isRunningCycle
              ? 'bg-slate-800 text-slate-400 cursor-not-allowed border border-slate-700'
              : 'bg-indigo-600 hover:bg-indigo-500 active:scale-95 shadow-indigo-600/30'
          }`}
        >
          <RotateCw className={`w-4 h-4 ${isRunningCycle ? 'animate-spin' : ''}`} />
          {isRunningCycle ? 'AI đang họp & ra quyết định...' : '▶ Chạy Chu Kỳ Mới (1-Click)'}
        </button>
      </div>

      {/* 4 Core Financial Numbers - Super Basic & Understandable */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Cash in Bank */}
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-1">
          <div className="flex items-center justify-between text-slate-400 text-xs font-medium">
            <span>Tiền trong kho bạc</span>
            <Wallet className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-2xl font-black text-white font-mono">
            {formatMoney(snapshot.cash_minor)}
          </div>
          <div className="text-xs text-slate-400 flex items-center gap-1 pt-1">
            <span>Đủ sống trong:</span>
            <strong className="text-cyan-400 font-mono">
              {snapshot.runway_days >= 999 ? '∞ Tự sinh lời' : `${snapshot.runway_days} ngày`}
            </strong>
          </div>
        </div>

        {/* Monthly Revenue */}
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-1">
          <div className="flex items-center justify-between text-slate-400 text-xs font-medium">
            <span>Doanh thu / Tháng</span>
            <TrendingUp className="w-4 h-4 text-cyan-400" />
          </div>
          <div className="text-2xl font-black text-cyan-400 font-mono">
            {formatMoney(snapshot.revenue_minor)}
          </div>
          <div className="text-xs text-slate-400 pt-1">
            Từ Affiliate & Video Creator
          </div>
        </div>

        {/* Monthly Expenses */}
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-1">
          <div className="flex items-center justify-between text-slate-400 text-xs font-medium">
            <span>Chi phí / Tháng</span>
            <TrendingDown className="w-4 h-4 text-rose-400" />
          </div>
          <div className="text-2xl font-black text-rose-400 font-mono">
            {formatMoney(snapshot.expenses_minor)}
          </div>
          <div className="text-xs text-slate-400 pt-1">
            GPU, API Cloud & Nhân sự
          </div>
        </div>

        {/* Net Profit / Loss */}
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-1">
          <div className="flex items-center justify-between text-slate-400 text-xs font-medium">
            <span>Lợi nhuận ròng / Tháng</span>
            <Sparkles className="w-4 h-4 text-purple-400" />
          </div>
          <div className={`text-2xl font-black font-mono ${netMonthly >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
            {netMonthly >= 0 ? `+${formatMoney(netMonthly)}` : formatMoney(netMonthly)}
          </div>
          <div className="text-xs text-slate-400 pt-1">
            {netMonthly >= 0 ? '🟢 Doanh nghiệp có lãi' : '🔴 Đang đốt tiền dự phòng'}
          </div>
        </div>
      </div>

      {/* Urgent: Pending Human Approvals (Only shows if any proposal needs human decision) */}
      {pendingApprovals.length > 0 && (
        <div className="bg-amber-950/30 border border-amber-500/40 rounded-xl p-5 space-y-3">
          <div className="flex items-center gap-2 text-amber-300 font-bold text-sm">
            <AlertTriangle className="w-4 h-4 text-amber-400" />
            Có {pendingApprovals.length} quyết định cần bạn duyệt (Hành động có chi phí lớn &gt; $1,000)
          </div>

          <div className="space-y-2">
            {pendingApprovals.map((item) => (
              <div
                key={item.proposal.id}
                className="bg-slate-900/90 border border-slate-800 rounded-lg p-3.5 flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs"
              >
                <div>
                  <div className="flex items-center gap-2 font-bold text-white text-sm">
                    <span className="text-indigo-400 font-mono">{item.proposal.agent}:</span>
                    <span>{item.proposal.objective}</span>
                  </div>
                  <p className="text-slate-400 mt-1">
                    Chi phí: <strong className="text-rose-400 font-mono">{formatMoney(item.proposal.cost_minor)}</strong> • Lý do: {item.proposal.rationale}
                  </p>
                </div>

                <div className="flex items-center gap-2 shrink-0">
                  <button
                    onClick={() => onOverride(item.proposal.id, 'Approve')}
                    className="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs shadow-sm transition-all"
                  >
                    Duyệt Ngay
                  </button>
                  <button
                    onClick={() => onOverride(item.proposal.id, 'Reject')}
                    className="px-4 py-2 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-bold text-xs shadow-sm transition-all"
                  >
                    Từ Chối
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Main 2-Column: Quick Activity & Quick Navigation */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left: What AI Did Recently (Activity Feed) */}
        <div className="lg:col-span-2 bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-4 shadow-md">
          <div className="flex items-center justify-between border-b border-slate-800 pb-3">
            <h3 className="font-bold text-white text-sm flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              Các Hành Động Gần Nhất Của AI
            </h3>
            <span className="text-xs text-slate-400 font-mono">Đã qua Governor kiểm duyệt</span>
          </div>

          <div className="space-y-2.5">
            {recentProposals.slice(0, 4).map((item, idx) => (
              <div
                key={idx}
                className="p-3 rounded-lg bg-slate-950/70 border border-slate-800/80 flex items-start justify-between gap-3 text-xs"
              >
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded text-[11px] font-bold font-mono bg-indigo-500/20 text-indigo-300">
                      {item.proposal.agent}
                    </span>
                    <span className="font-semibold text-slate-200">{item.proposal.objective}</span>
                  </div>
                  <p className="text-slate-400 text-[11px] line-clamp-1">
                    {item.proposal.rationale}
                  </p>
                </div>

                <div className="text-right shrink-0">
                  <span
                    className={`inline-block px-2 py-0.5 rounded text-[10px] font-bold ${
                      item.decision === 'Approve'
                        ? 'bg-emerald-500/10 text-emerald-400'
                        : item.decision === 'Reject'
                        ? 'bg-rose-500/10 text-rose-400'
                        : 'bg-amber-500/10 text-amber-300'
                    }`}
                  >
                    {item.decision === 'Approve' ? 'Đã thực thi' : item.decision === 'Reject' ? 'Bị từ chối' : 'Cần người duyệt'}
                  </span>
                  <div className="text-[10px] text-slate-500 font-mono mt-0.5">
                    {item.proposal.cost_minor > 0 ? formatMoney(item.proposal.cost_minor) : 'Miễn phí'}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Right: Simple Management Short-Cuts */}
        <div className="space-y-3">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3 shadow-md">
            <h3 className="font-bold text-white text-sm">Quản Lý Nhanh</h3>

            <div className="space-y-2">
              <button
                onClick={() => onNavigate('agents')}
                className="w-full text-left p-3 rounded-lg bg-slate-950/80 hover:bg-slate-800/80 border border-slate-800 text-xs text-slate-200 transition-all flex items-center justify-between"
              >
                <div className="flex items-center gap-2.5">
                  <Users className="w-4 h-4 text-indigo-400" />
                  <div>
                    <span className="font-bold block">Quản lý 9 Nhân sự AI</span>
                    <span className="text-[11px] text-slate-400">Xem vai trò, nhiệm vụ & ngân sách</span>
                  </div>
                </div>
                <ArrowRight className="w-4 h-4 text-slate-500" />
              </button>

              <button
                onClick={() => onNavigate('finances')}
                className="w-full text-left p-3 rounded-lg bg-slate-950/80 hover:bg-slate-800/80 border border-slate-800 text-xs text-slate-200 transition-all flex items-center justify-between"
              >
                <div className="flex items-center gap-2.5">
                  <Wallet className="w-4 h-4 text-emerald-400" />
                  <div>
                    <span className="font-bold block">Lịch sử Thu / Chi</span>
                    <span className="text-[11px] text-slate-400">Xem dòng tiền như ví ngân hàng</span>
                  </div>
                </div>
                <ArrowRight className="w-4 h-4 text-slate-500" />
              </button>

              <button
                onClick={() => onNavigate('create-content')}
                className="w-full text-left p-3 rounded-lg bg-slate-950/80 hover:bg-slate-800/80 border border-slate-800 text-xs text-slate-200 transition-all flex items-center justify-between"
              >
                <div className="flex items-center gap-2.5">
                  <Sparkles className="w-4 h-4 text-purple-400" />
                  <div>
                    <span className="font-bold block">Tạo Kịch Bản Video AI</span>
                    <span className="text-[11px] text-slate-400">Sản xuất video bán hàng Affiliate</span>
                  </div>
                </div>
                <ArrowRight className="w-4 h-4 text-slate-500" />
              </button>
            </div>
          </div>

          {/* Quick Health Tip */}
          <div className="p-4 rounded-xl bg-indigo-950/20 border border-indigo-500/20 text-xs text-indigo-300 space-y-1">
            <span className="font-bold block flex items-center gap-1.5">
              <ShieldCheck className="w-4 h-4 text-indigo-400" />
              Nguyên tắc an toàn vốn:
            </span>
            <p className="text-slate-400 text-[11px] leading-relaxed">
              Governor tự động chặn đứng mọi khoản chi vượt ngân sách hoặc khi tiền mặt tụt dưới 30 ngày sống còn.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
