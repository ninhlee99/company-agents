import React, { useState } from 'react';
import { CompanySnapshot, GovernedProposal, ExecutionReceipt } from '../types/company';
import { 
  Play, 
  RotateCw, 
  CheckCircle2, 
  XCircle, 
  AlertCircle, 
  ShieldCheck, 
  DollarSign, 
  TrendingUp, 
  Clock, 
  UserCheck, 
  HelpCircle,
  FileText,
  Zap,
  ArrowRight
} from 'lucide-react';

interface CycleRunnerTabProps {
  snapshot: CompanySnapshot;
  recentProposals: GovernedProposal[];
  recentReceipts: ExecutionReceipt[];
  onRunCycle: () => void;
  isRunningCycle: boolean;
  onOverride: (proposalId: string, decision: 'Approve' | 'Reject') => void;
}

export const CycleRunnerTab: React.FC<CycleRunnerTabProps> = ({
  snapshot,
  recentProposals,
  recentReceipts,
  onRunCycle,
  isRunningCycle,
  onOverride,
}) => {
  const [selectedFilter, setSelectedFilter] = useState<'All' | 'Approve' | 'Reject' | 'EscalateToHuman'>('All');

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 2,
    }).format(minor / 100);
  };

  const getDecisionBadge = (decision: string) => {
    switch (decision) {
      case 'Approve':
        return (
          <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <CheckCircle2 className="w-3.5 h-3.5" /> Chấp thuận (Approved)
          </span>
        );
      case 'Reject':
        return (
          <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/20">
            <XCircle className="w-3.5 h-3.5" /> Phủ quyết (Rejected)
          </span>
        );
      case 'EscalateToHuman':
        return (
          <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-500/10 text-amber-300 border border-amber-500/30 animate-pulse">
            <UserCheck className="w-3.5 h-3.5" /> Cần Operator Duyệt (Material)
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-blue-500/10 text-blue-400 border border-blue-500/20">
            <AlertCircle className="w-3.5 h-3.5" /> Yêu cầu sửa đổi (Revision)
          </span>
        );
    }
  };

  const getRiskBadge = (risk: string) => {
    switch (risk) {
      case 'Low':
        return <span className="text-[11px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded">Rủi ro: Thấp (Low)</span>;
      case 'Medium':
        return <span className="text-[11px] font-mono text-amber-400 bg-amber-500/10 px-2 py-0.5 rounded">Rủi ro: Trung bình (Medium)</span>;
      case 'High':
      case 'Critical':
        return <span className="text-[11px] font-mono text-rose-400 bg-rose-500/10 px-2 py-0.5 rounded font-bold">Rủi ro: Cao (Critical)</span>;
      default:
        return null;
    }
  };

  const filteredProposals = recentProposals.filter((p) => {
    if (selectedFilter === 'All') return true;
    return p.decision === selectedFilter;
  });

  return (
    <div className="space-y-6 max-w-6xl mx-auto pb-16">
      {/* Control Banner */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 md:p-6 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-lg">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-xl font-bold text-white">Bộ Chạy Chu Kỳ Tự Trị (Decision Cycle Engine)</h2>
            <span className="px-2 py-0.5 rounded text-xs font-mono bg-purple-500/20 text-purple-300 border border-purple-500/30">
              POST /api/run
            </span>
          </div>
          <p className="text-slate-400 text-xs md:text-sm mt-1">
            Kích hoạt 8 agents chạy phân tích và đề xuất hành vi kinh tế đồng thời, Governor đánh giá theo luật hiến định và hạch toán sổ cái kép.
          </p>
        </div>

        <button
          onClick={onRunCycle}
          disabled={isRunningCycle}
          className={`flex items-center gap-2.5 px-6 py-3 rounded-xl font-bold text-white transition-all shadow-lg text-sm shrink-0 ${
            isRunningCycle
              ? 'bg-slate-800 text-slate-400 cursor-not-allowed border border-slate-700'
              : 'bg-gradient-to-r from-indigo-600 via-purple-600 to-cyan-500 hover:from-indigo-500 hover:to-cyan-400 hover:shadow-indigo-500/30 active:scale-95'
          }`}
        >
          <RotateCw className={`w-4 h-4 ${isRunningCycle ? 'animate-spin' : ''}`} />
          {isRunningCycle ? 'Đang Thực Thi 8 Agents & Governor...' : 'Kích Hoạt Chu Kỳ Mới'}
        </button>
      </div>

      {/* 4-Step Cycle Flow Visualizer */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-3 text-xs">
        <div className={`p-4 rounded-xl border ${isRunningCycle ? 'bg-indigo-950/40 border-indigo-500 animate-pulse' : 'bg-slate-900/80 border-slate-800'}`}>
          <div className="text-slate-400 font-mono text-[10px] uppercase">Giai Đoạn 1</div>
          <div className="font-bold text-white text-sm mt-0.5 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-cyan-400"></span>
            Snapshot Dữ Liệu
          </div>
          <p className="text-slate-400 text-[11px] mt-1">Kiểm toán tiền mặt, ngân sách, backlog và trạng thái công ty.</p>
        </div>

        <div className={`p-4 rounded-xl border ${isRunningCycle ? 'bg-purple-950/40 border-purple-500 animate-pulse' : 'bg-slate-900/80 border-slate-800'}`}>
          <div className="text-slate-400 font-mono text-[10px] uppercase">Giai Đoạn 2</div>
          <div className="font-bold text-white text-sm mt-0.5 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-purple-400"></span>
            Đề Xuất Song Song (LLM)
          </div>
          <p className="text-slate-400 text-[11px] mt-1">8 agents chạy prompt với Gemini 3.8 Flash sinh proposal & evidence.</p>
        </div>

        <div className={`p-4 rounded-xl border ${isRunningCycle ? 'bg-amber-950/40 border-amber-500 animate-pulse' : 'bg-slate-900/80 border-slate-800'}`}>
          <div className="text-slate-400 font-mono text-[10px] uppercase">Giai Đoạn 3</div>
          <div className="font-bold text-white text-sm mt-0.5 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-amber-400"></span>
            Governor Phê Duyệt
          </div>
          <p className="text-slate-400 text-[11px] mt-1">Kiểm tra giới hạn hiến định, trần ngân sách và chính sách rủi ro.</p>
        </div>

        <div className={`p-4 rounded-xl border ${isRunningCycle ? 'bg-emerald-950/40 border-emerald-500 animate-pulse' : 'bg-slate-900/80 border-slate-800'}`}>
          <div className="text-slate-400 font-mono text-[10px] uppercase">Giai Đoạn 4</div>
          <div className="font-bold text-white text-sm mt-0.5 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-emerald-400"></span>
            Hạch Toán Sổ Cái Kép
          </div>
          <p className="text-slate-400 text-[11px] mt-1">Bút toán Nợ/Có, trừ quỹ dự phòng và xuất biên lai Receipt.</p>
        </div>
      </div>

      {/* Filter Tabs */}
      <div className="flex items-center justify-between gap-4 border-b border-slate-800 pb-3">
        <div className="flex items-center gap-2">
          <span className="text-xs text-slate-400 uppercase font-mono mr-2">Lọc Đề Xuất:</span>
          {(['All', 'Approve', 'EscalateToHuman', 'Reject'] as const).map((filter) => (
            <button
              key={filter}
              onClick={() => setSelectedFilter(filter)}
              className={`px-3 py-1 rounded-lg text-xs font-semibold transition-all ${
                selectedFilter === filter
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'bg-slate-900 text-slate-400 hover:text-white border border-slate-800'
              }`}
            >
              {filter === 'All' ? 'Tất cả' : filter === 'Approve' ? 'Đã duyệt' : filter === 'EscalateToHuman' ? 'Cần người duyệt' : 'Bị từ chối'}
            </button>
          ))}
        </div>
        <div className="text-xs text-slate-400 font-mono">
          Hiển thị: <strong className="text-white">{filteredProposals.length}</strong> đề xuất
        </div>
      </div>

      {/* Proposals List */}
      <div className="space-y-4">
        {filteredProposals.length === 0 ? (
          <div className="p-8 text-center bg-slate-900/50 rounded-xl border border-slate-800 text-slate-400">
            Chưa có đề xuất nào trong danh mục này. Hãy bấm "Kích Hoạt Chu Kỳ Mới" ở trên để chạy cycle.
          </div>
        ) : (
          filteredProposals.map((item, idx) => {
            const { proposal, decision, reason, executed } = item;
            return (
              <div
                key={proposal.id || idx}
                className="bg-slate-900/90 border border-slate-800 rounded-xl p-5 hover:border-slate-700 transition-all shadow-md space-y-3"
              >
                {/* Proposal Header */}
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800/80 pb-3">
                  <div className="flex items-center gap-2.5">
                    <span className="px-2.5 py-1 rounded-md text-xs font-bold font-mono bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                      {proposal.agent}
                    </span>
                    <span className="font-semibold text-white text-sm">{proposal.objective}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {getRiskBadge(proposal.risk)}
                    {getDecisionBadge(decision)}
                  </div>
                </div>

                {/* Economic Specs & Evidence */}
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
                  <div className="bg-slate-950/60 p-2.5 rounded-lg border border-slate-800/60">
                    <span className="text-slate-400 text-[10px] uppercase font-mono block">Chi phí Dự kiến</span>
                    <span className="font-bold font-mono text-white text-sm">
                      {proposal.cost_minor > 0 ? formatMoney(proposal.cost_minor) : '$0.00'}
                    </span>
                  </div>

                  <div className="bg-slate-950/60 p-2.5 rounded-lg border border-slate-800/60">
                    <span className="text-slate-400 text-[10px] uppercase font-mono block">Doanh thu Kế hoạch</span>
                    <span className="font-bold font-mono text-emerald-400 text-sm">
                      {proposal.expected_revenue_minor > 0 ? formatMoney(proposal.expected_revenue_minor) : 'N/A'}
                    </span>
                  </div>

                  <div className="bg-slate-950/60 p-2.5 rounded-lg border border-slate-800/60">
                    <span className="text-slate-400 text-[10px] uppercase font-mono block">Độ Tự Tin (Confidence)</span>
                    <span className="font-bold font-mono text-cyan-400 text-sm">
                      {(proposal.confidence_bps / 100).toFixed(1)}%
                    </span>
                  </div>

                  <div className="bg-slate-950/60 p-2.5 rounded-lg border border-slate-800/60">
                    <span className="text-slate-400 text-[10px] uppercase font-mono block">Khả năng Đảo ngược</span>
                    <span className={`font-bold font-mono text-sm ${proposal.reversible ? 'text-blue-400' : 'text-amber-400'}`}>
                      {proposal.reversible ? 'Reversible (Có thể hủy)' : 'Irreversible (Bất biến)'}
                    </span>
                  </div>
                </div>

                {/* Rationale & Evidence */}
                <div className="text-xs space-y-1.5 bg-slate-950/40 p-3 rounded-lg border border-slate-800/50">
                  <div className="text-slate-300">
                    <strong className="text-indigo-300 font-mono">Lập luận Kinh tế:</strong> {proposal.rationale}
                  </div>
                  {proposal.evidence && proposal.evidence.length > 0 && (
                    <div className="flex flex-wrap gap-1.5 mt-2">
                      <span className="text-slate-400 font-mono text-[10px]">Căn cứ:</span>
                      {proposal.evidence.map((ev, i) => (
                        <span key={i} className="px-2 py-0.5 rounded bg-slate-800 text-slate-300 text-[11px] font-mono">
                          {ev}
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                {/* Governor Reason & Human Action */}
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs pt-1">
                  <div className="text-slate-400 italic flex items-center gap-1.5">
                    <ShieldCheck className="w-4 h-4 text-indigo-400 shrink-0" />
                    <span><strong>Governor:</strong> {reason}</span>
                  </div>

                  {decision === 'EscalateToHuman' && !executed && (
                    <div className="flex items-center gap-2 shrink-0">
                      <button
                        onClick={() => onOverride(proposal.id, 'Approve')}
                        className="px-3 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-semibold transition-all shadow-sm"
                      >
                        Chấp Thuận (Authorize)
                      </button>
                      <button
                        onClick={() => onOverride(proposal.id, 'Reject')}
                        className="px-3 py-1 rounded bg-rose-600 hover:bg-rose-500 text-white font-semibold transition-all shadow-sm"
                      >
                        Bác Bỏ (Veto)
                      </button>
                    </div>
                  )}

                  {executed && (
                    <span className="text-[11px] font-mono text-emerald-400 flex items-center gap-1">
                      <CheckCircle2 className="w-3.5 h-3.5" /> Đã hạch toán sổ cái kép
                    </span>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Execution Receipts Archive */}
      {recentReceipts.length > 0 && (
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3">
          <h3 className="text-sm font-bold text-white flex items-center gap-2 font-mono uppercase">
            <FileText className="w-4 h-4 text-indigo-400" />
            Nhật Ký Biên Lai Thực Thi (Execution Receipts)
          </h3>
          <div className="space-y-2">
            {recentReceipts.slice(0, 5).map((rcpt) => (
              <div key={rcpt.id} className="p-3 bg-slate-950/70 rounded-lg border border-slate-800/80 flex items-start justify-between gap-3 text-xs">
                <div>
                  <div className="flex items-center gap-2 font-mono text-[11px]">
                    <span className="text-indigo-400 font-bold">{rcpt.agent}</span>
                    <span className="text-slate-500">•</span>
                    <span className="text-slate-300 font-semibold">{rcpt.action}</span>
                    <span className="text-slate-500">•</span>
                    <span className="text-emerald-400 font-bold">{formatMoney(rcpt.cost_minor)}</span>
                  </div>
                  <p className="text-slate-400 mt-1 text-xs">{rcpt.outcome}</p>
                </div>
                <span className="text-[10px] font-mono text-slate-500 whitespace-nowrap">
                  {new Date(rcpt.timestamp).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
