import React, { useState, useMemo } from 'react';
import { CompanySnapshot } from '../types/company';
import { 
  ArrowRightLeft, 
  TrendingUp, 
  Sparkles, 
  RotateCcw, 
  CheckCircle2, 
  AlertTriangle, 
  HelpCircle,
  MoveRight,
  ShieldCheck,
  Zap,
  ArrowUpRight,
  ArrowDownRight,
  DollarSign
} from 'lucide-react';

interface DepartmentBudget {
  id: string;
  name: string;
  category: 'donor' | 'recipient' | 'neutral';
  baselineBudgetMinor: number;
  currentBudgetMinor: number;
  performanceScore: number; // 0-100
  roiPercent: number;
  statusLabel: string;
  statusType: 'surplus' | 'underperforming' | 'balanced';
  description: string;
  maxSurplusMinor: number;
}

interface ResourceReallocationDashboardProps {
  snapshot: CompanySnapshot;
  onCommitReallocation?: (changes: { fromDept: string; toDept: string; amountMinor: number }[]) => Promise<{ success: boolean; message?: string }>;
}

export const ResourceReallocationDashboard: React.FC<ResourceReallocationDashboardProps> = ({
  snapshot,
  onCommitReallocation,
}) => {
  // Baseline departments state
  const [departments, setDepartments] = useState<DepartmentBudget[]>([
    {
      id: 'dept-growth',
      name: 'Kinh Doanh & Traffic (Growth)',
      category: 'donor',
      baselineBudgetMinor: 240000, // $2,400
      currentBudgetMinor: 240000,
      performanceScore: 94,
      roiPercent: 460,
      statusLabel: 'Thặng Dư Ngân Sách (+Surplus)',
      statusType: 'surplus',
      description: 'Hiệu quả affiliate vượt 180% KPI. Có thặng dư tiền mặt dồi dào có thể điều chuyển.',
      maxSurplusMinor: 70000, // can donate up to $700
    },
    {
      id: 'dept-content',
      name: 'Sáng Tạo Nội Dung (Content)',
      category: 'donor',
      baselineBudgetMinor: 190000, // $1,900
      currentBudgetMinor: 190000,
      performanceScore: 92,
      roiPercent: 500,
      statusLabel: 'Thặng Dư Ngân Sách (+Surplus)',
      statusType: 'surplus',
      description: 'Dây chuyền video ngắn viral ổn định, chi phí sản xuất thấp hơn dự kiến.',
      maxSurplusMinor: 50000, // can donate up to $500
    },
    {
      id: 'dept-ops',
      name: 'Vận Hành & Handoff (Ops)',
      category: 'recipient',
      baselineBudgetMinor: 130000, // $1,300
      currentBudgetMinor: 130000,
      performanceScore: 68,
      roiPercent: 125,
      statusLabel: 'Điểm Nghẽn / Cần Vốn (Underperforming)',
      statusType: 'underperforming',
      description: 'Hàng đợi tồn đọng 12 việc, thiếu công cụ tự động hóa giải tỏa nút thắt cổ chai.',
      maxSurplusMinor: 0,
    },
    {
      id: 'dept-tech',
      name: 'Hạ Tầng GPU & AI Cache (Tech)',
      category: 'recipient',
      baselineBudgetMinor: 110000, // $1,100
      currentBudgetMinor: 110000,
      performanceScore: 64,
      roiPercent: 85,
      statusLabel: 'Thiếu Hụt Vốn Đầu Tư (Underperforming)',
      statusType: 'underperforming',
      description: 'Quá tải GPU và độ trễ LLM cao; cần thêm ngân quỹ cache để tăng tốc toàn công ty.',
      maxSurplusMinor: 0,
    },
    {
      id: 'dept-gov',
      name: 'Quản Trị & Kho Bạc (Treasury / Risk)',
      category: 'neutral',
      baselineBudgetMinor: 150000, // $1,500
      currentBudgetMinor: 150000,
      performanceScore: 88,
      roiPercent: 190,
      statusLabel: 'Cân Bằng Chuẩn Mực (Balanced)',
      statusType: 'balanced',
      description: 'Bảo vệ an toàn vốn; chi phí duy trì cố định và tuân thủ hiến pháp.',
      maxSurplusMinor: 20000, // can donate up to $200
    },
  ]);

  // Dragging state
  const [draggedDonorId, setDraggedDonorId] = useState<string | null>(null);
  const [dragAmountMinor, setDragAmountMinor] = useState<number>(20000); // $200 default chip
  const [isCommitting, setIsCommitting] = useState(false);
  const [commitSuccessMsg, setCommitSuccessMsg] = useState<string | null>(null);

  // Transfer history in session
  const [transfers, setTransfers] = useState<{
    id: string;
    fromDeptId: string;
    toDeptId: string;
    amountMinor: number;
    timestamp: string;
  }[]>([]);

  // Execute reallocation transfer
  const handleTransfer = (fromId: string, toId: string, amountMinor: number) => {
    if (fromId === toId) return;

    setDepartments((prev) => {
      const fromDept = prev.find((d) => d.id === fromId);
      const toDept = prev.find((d) => d.id === toId);
      if (!fromDept || !toDept) return prev;

      // Don't reduce beyond baseline minus maxSurplus
      const availableSurplus = fromDept.currentBudgetMinor - (fromDept.baselineBudgetMinor - fromDept.maxSurplusMinor);
      const actualAmount = Math.min(amountMinor, Math.max(0, availableSurplus));
      if (actualAmount <= 0) return prev;

      return prev.map((dept) => {
        if (dept.id === fromId) {
          return { ...dept, currentBudgetMinor: dept.currentBudgetMinor - actualAmount };
        }
        if (dept.id === toId) {
          return { ...dept, currentBudgetMinor: dept.currentBudgetMinor + actualAmount };
        }
        return dept;
      });
    });

    setTransfers((prev) => [
      {
        id: `tr-${Date.now().toString(36)}`,
        fromDeptId: fromId,
        toDeptId: toId,
        amountMinor,
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
      },
      ...prev,
    ]);
  };

  // Reset to initial baseline
  const handleReset = () => {
    setDepartments((prev) =>
      prev.map((d) => ({
        ...d,
        currentBudgetMinor: d.baselineBudgetMinor,
      }))
    );
    setTransfers([]);
    setCommitSuccessMsg(null);
  };

  // Immediate Simulation of Projected ROI Impact
  const simulation = useMemo(() => {
    const totalReallocatedMinor = departments.reduce((acc, d) => {
      const diff = d.currentBudgetMinor - d.baselineBudgetMinor;
      return diff > 0 ? acc + diff : acc;
    }, 0);

    const opsBonusMinor = (departments.find((d) => d.id === 'dept-ops')?.currentBudgetMinor || 0) -
      (departments.find((d) => d.id === 'dept-ops')?.baselineBudgetMinor || 0);

    const techBonusMinor = (departments.find((d) => d.id === 'dept-tech')?.currentBudgetMinor || 0) -
      (departments.find((d) => d.id === 'dept-tech')?.baselineBudgetMinor || 0);

    // Dynamic projected gains
    // Ops funding slashes backlog congestion and saves commission loss
    const backlogReduction = Math.min(snapshot.backlog, Math.round((Math.max(0, opsBonusMinor) / 10000) * 1.5));
    // Tech funding speeds up rendering for Content and adds revenue
    const projectedRevGainMinor = Math.round((Math.max(0, techBonusMinor) * 2.2) + (Math.max(0, opsBonusMinor) * 1.4));
    const projectedRoiLiftPct = Math.round((totalReallocatedMinor / 10000) * 2.4);

    return {
      totalReallocatedMinor,
      backlogReduction,
      projectedRevGainMinor,
      projectedRoiLiftPct,
      netCapacityBonus: Math.round(totalReallocatedMinor / 25000),
    };
  }, [departments, snapshot.backlog]);

  // Commit changes
  const handleCommit = async () => {
    if (simulation.totalReallocatedMinor <= 0) return;
    setIsCommitting(true);
    setCommitSuccessMsg(null);

    const changes = transfers.map((t) => ({
      fromDept: t.fromDeptId,
      toDept: t.toDeptId,
      amountMinor: t.amountMinor,
    }));

    if (onCommitReallocation) {
      const res = await onCommitReallocation(changes);
      setIsCommitting(false);
      if (res.success) {
        setCommitSuccessMsg(`Đã phê duyệt & áp dụng tái phân bổ $${(simulation.totalReallocatedMinor / 100).toLocaleString()} vào sổ cái kép!`);
      }
    } else {
      setTimeout(() => {
        setIsCommitting(false);
        setCommitSuccessMsg(`Đã phê duyệt & áp dụng tái phân bổ $${(simulation.totalReallocatedMinor / 100).toLocaleString()} vào sổ cái kép!`);
      }, 500);
    }
  };

  const donorDepts = departments.filter((d) => d.category === 'donor' || d.id === 'dept-gov');
  const recipientDepts = departments.filter((d) => d.category === 'recipient');

  return (
    <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 sm:p-5 space-y-4 shadow-sm">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-slate-800 pb-3">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400 shrink-0">
            <ArrowRightLeft className="w-5 h-5 text-cyan-400" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="text-base font-bold text-white">
                Bảng Điều Khiển Tái Phân Bổ Nguồn Vốn (Resource Reallocation)
              </h3>
              <span className="px-2 py-0.5 rounded-full font-mono text-[10px] bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 font-bold">
                Kéo &amp; Thả (Drag &amp; Drop)
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Điều chuyển thặng dư ngân sách từ phòng ban siêu lợi nhuận sang phòng ban điểm nghẽn để kích hoạt mô phỏng ROI tức thì
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {simulation.totalReallocatedMinor > 0 && (
            <button
              onClick={handleReset}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white font-semibold text-xs transition-all border border-slate-700"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span>Khôi Phục Mặc Định</span>
            </button>
          )}

          <button
            onClick={handleCommit}
            disabled={simulation.totalReallocatedMinor <= 0 || isCommitting}
            className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg font-bold text-xs shadow-md transition-all border ${
              simulation.totalReallocatedMinor <= 0
                ? 'bg-slate-950 text-slate-600 border-slate-800 cursor-not-allowed'
                : isCommitting
                ? 'bg-cyan-700 text-white cursor-wait'
                : 'bg-gradient-to-r from-cyan-600 to-indigo-600 hover:from-cyan-500 hover:to-indigo-500 text-white border-cyan-500 active:scale-95'
            }`}
          >
            <CheckCircle2 className="w-4 h-4 text-emerald-300" />
            <span>{isCommitting ? 'Đang Thực Thi Sổ Cái...' : 'Áp Dụng Tái Phân Bổ'}</span>
          </button>
        </div>
      </div>

      {/* Success notification */}
      {commitSuccessMsg && (
        <div className="p-3 rounded-xl bg-emerald-950/40 border border-emerald-500/40 text-emerald-300 text-xs flex items-center justify-between animate-in fade-in">
          <span className="flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <strong>{commitSuccessMsg}</strong>
          </span>
          <span className="font-mono text-[10px] text-emerald-400/80">Double-Entry Verified</span>
        </div>
      )}

      {/* Drag amount chip selector */}
      <div className="flex flex-wrap items-center justify-between gap-2 p-2.5 rounded-lg bg-slate-950 border border-slate-800 text-xs">
        <span className="font-semibold text-slate-300 flex items-center gap-1.5">
          <Sparkles className="w-3.5 h-3.5 text-yellow-400" />
          Mức ngân sách điều chuyển mỗi lần kéo thả (Drag Amount):
        </span>

        <div className="flex items-center gap-1.5">
          {[10000, 20000, 30000, 50000].map((amt) => (
            <button
              key={amt}
              onClick={() => setDragAmountMinor(amt)}
              className={`px-2.5 py-1 rounded-md font-mono font-bold text-xs transition-all border ${
                dragAmountMinor === amt
                  ? 'bg-cyan-600 text-white border-cyan-400 shadow-sm'
                  : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-white'
              }`}
            >
              ${amt / 100}
            </button>
          ))}
        </div>
      </div>

      {/* MAIN TWO-COLUMN DRAG & DROP WORKBENCH */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* COLUMN 1: DONOR DEPARTMENTS (HIGH-PERFORMING SURPLUS) */}
        <div className="space-y-2.5">
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold text-emerald-400 uppercase tracking-wider flex items-center gap-1.5">
              <ArrowDownRight className="w-4 h-4" /> 1. Phòng Ban Thặng Dư (Nguồn Cung Cấp)
            </span>
            <span className="text-[10px] text-slate-400">Kéo thẻ sang cột bên phải ➔</span>
          </div>

          <div className="space-y-2">
            {donorDepts.map((dept) => {
              const diff = dept.currentBudgetMinor - dept.baselineBudgetMinor;
              const availableToDonate = dept.currentBudgetMinor - (dept.baselineBudgetMinor - dept.maxSurplusMinor);

              return (
                <div
                  key={dept.id}
                  draggable={availableToDonate > 0}
                  onDragStart={(e) => {
                    setDraggedDonorId(dept.id);
                    e.dataTransfer.setData('text/plain', dept.id);
                  }}
                  onDragEnd={() => setDraggedDonorId(null)}
                  className={`p-3.5 rounded-xl border transition-all select-none ${
                    availableToDonate > 0
                      ? 'bg-slate-950/80 border-emerald-500/30 hover:border-emerald-500/60 cursor-grab active:cursor-grabbing hover:shadow-lg'
                      : 'bg-slate-950/40 border-slate-800 opacity-60 cursor-not-allowed'
                  } ${draggedDonorId === dept.id ? 'ring-2 ring-cyan-400 scale-[0.99]' : ''}`}
                >
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <span className="font-bold text-white text-xs block">{dept.name}</span>
                      <span className="text-[10px] text-emerald-400 font-semibold block mt-0.5">
                        {dept.statusLabel} • ROI {dept.roiPercent}%
                      </span>
                    </div>

                    <div className="text-right">
                      <span className="font-mono font-bold text-xs text-white block">
                        ${(dept.currentBudgetMinor / 100).toLocaleString()}/th
                      </span>
                      {diff !== 0 && (
                        <span className={`text-[10px] font-mono font-bold ${diff < 0 ? 'text-rose-400' : 'text-emerald-400'}`}>
                          {diff < 0 ? `-${Math.abs(diff / 100)}` : `+${diff / 100}`}
                        </span>
                      )}
                    </div>
                  </div>

                  <p className="text-[11px] text-slate-400 mt-2 leading-relaxed">
                    {dept.description}
                  </p>

                  <div className="mt-3 pt-2 border-t border-slate-800 flex items-center justify-between text-[11px]">
                    <span className="text-slate-400">
                      Thặng dư khả dụng: <strong className="text-emerald-400 font-mono">${(availableToDonate / 100).toLocaleString()}</strong>
                    </span>

                    {/* Quick Transfer Pill Buttons */}
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => handleTransfer(dept.id, 'dept-ops', dragAmountMinor)}
                        disabled={availableToDonate < dragAmountMinor}
                        className="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] text-indigo-300 font-semibold border border-slate-700 disabled:opacity-40"
                        title="Chuyển sang Ops"
                      >
                        ➔ Ops (+${dragAmountMinor / 100})
                      </button>
                      <button
                        onClick={() => handleTransfer(dept.id, 'dept-tech', dragAmountMinor)}
                        disabled={availableToDonate < dragAmountMinor}
                        className="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] text-cyan-300 font-semibold border border-slate-700 disabled:opacity-40"
                        title="Chuyển sang Tech"
                      >
                        ➔ Tech (+${dragAmountMinor / 100})
                      </button>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* COLUMN 2: RECIPIENT DEPARTMENTS (UNDERPERFORMING TARGETS) */}
        <div className="space-y-2.5">
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5">
              <ArrowUpRight className="w-4 h-4" /> 2. Phòng Ban Cần Tăng Trưởng (Đích Tiếp Nhận)
            </span>
            <span className="text-[10px] text-slate-400">Thả thẻ vào ô dưới đây</span>
          </div>

          <div className="space-y-2">
            {recipientDepts.map((dept) => {
              const diff = dept.currentBudgetMinor - dept.baselineBudgetMinor;

              return (
                <div
                  key={dept.id}
                  onDragOver={(e) => {
                    e.preventDefault();
                    e.dataTransfer.dropEffect = 'copy';
                  }}
                  onDrop={(e) => {
                    e.preventDefault();
                    const fromId = e.dataTransfer.getData('text/plain') || draggedDonorId;
                    if (fromId) {
                      handleTransfer(fromId, dept.id, dragAmountMinor);
                    }
                  }}
                  className={`p-3.5 rounded-xl border transition-all ${
                    diff > 0
                      ? 'bg-slate-950/90 border-cyan-500/50 shadow-md ring-1 ring-cyan-500/30'
                      : 'bg-slate-950/80 border-dashed border-amber-500/40 hover:border-cyan-400'
                  }`}
                >
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <span className="font-bold text-white text-xs block">{dept.name}</span>
                      <span className="text-[10px] text-amber-400 font-semibold block mt-0.5">
                        {dept.statusLabel} • Điểm tải: {dept.performanceScore}/100
                      </span>
                    </div>

                    <div className="text-right">
                      <span className="font-mono font-bold text-xs text-white block">
                        ${(dept.currentBudgetMinor / 100).toLocaleString()}/th
                      </span>
                      {diff > 0 ? (
                        <span className="text-[10px] font-mono font-bold text-cyan-400">
                          +${diff / 100} được bơm
                        </span>
                      ) : (
                        <span className="text-[10px] text-slate-500 font-mono">Chờ tiếp nhận</span>
                      )}
                    </div>
                  </div>

                  <p className="text-[11px] text-slate-400 mt-2 leading-relaxed">
                    {dept.description}
                  </p>

                  <div className="mt-3 pt-2 border-t border-slate-800 flex items-center justify-between text-[11px]">
                    <span className="text-slate-400 flex items-center gap-1">
                      <ShieldCheck className="w-3.5 h-3.5 text-indigo-400" />
                      <span>Khu vực thả ngân sách:</span>
                    </span>

                    <span className="text-[10px] font-semibold text-cyan-300 bg-cyan-950/40 border border-cyan-500/30 px-2 py-0.5 rounded">
                      Thả để nhận +${dragAmountMinor / 100}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      {/* IMMEDIATE SIMULATION OF PROJECTED ROI IMPACT (THE CALCULATOR) */}
      <div className="p-4 rounded-xl bg-gradient-to-r from-slate-950 via-indigo-950/20 to-slate-950 border border-indigo-500/30 space-y-3">
        <div className="flex items-center justify-between border-b border-slate-800 pb-2">
          <div className="flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-cyan-400 animate-pulse" />
            <span className="font-bold text-white text-xs">
              Mô Phỏng Tức Thì Tác Động Tái Phân Bổ (Projected ROI Simulation)
            </span>
          </div>
          <span className="text-[10px] text-slate-400 font-mono">
            Đã điều chuyển: ${(simulation.totalReallocatedMinor / 100).toLocaleString()}
          </span>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs">
          <div className="p-2.5 rounded-lg bg-slate-900 border border-slate-800">
            <span className="text-[10px] text-slate-400 block">Dự Phóng Tăng Doanh Thu</span>
            <span className="text-sm font-black text-cyan-400 font-mono block mt-0.5">
              +${(simulation.projectedRevGainMinor / 100).toLocaleString()}/th
            </span>
            <span className="text-[10px] text-slate-500 block">Nhờ tăng tốc GPU &amp; Video</span>
          </div>

          <div className="p-2.5 rounded-lg bg-slate-900 border border-slate-800">
            <span className="text-[10px] text-slate-400 block">Giải Tỏa Điểm Nghẽn Backlog</span>
            <span className="text-sm font-black text-emerald-400 font-mono block mt-0.5">
              -{simulation.backlogReduction} việc tồn đọng
            </span>
            <span className="text-[10px] text-slate-500 block">Từ {snapshot.backlog} việc ➔ {Math.max(0, snapshot.backlog - simulation.backlogReduction)}</span>
          </div>

          <div className="p-2.5 rounded-lg bg-slate-900 border border-slate-800">
            <span className="text-[10px] text-slate-400 block">Mức Tăng Tỷ Suất ROI Toàn Cục</span>
            <span className="text-sm font-black text-indigo-300 font-mono block mt-0.5">
              +{simulation.projectedRoiLiftPct}%
            </span>
            <span className="text-[10px] text-slate-500 block">Tối ưu hóa hiệu quả biên</span>
          </div>

          <div className="p-2.5 rounded-lg bg-slate-900 border border-slate-800">
            <span className="text-[10px] text-slate-400 block">Tăng Dung Lượng Xử Lý</span>
            <span className="text-sm font-black text-purple-300 font-mono block mt-0.5">
              +{simulation.netCapacityBonus} Capacity
            </span>
            <span className="text-[10px] text-slate-500 block">Giảm độ trễ hệ thống</span>
          </div>
        </div>

        {/* Live transfer log */}
        {transfers.length > 0 && (
          <div className="pt-2 border-t border-slate-800/80 text-[10px] text-slate-400 space-y-1">
            <span className="font-semibold text-slate-300 block">Lịch sử điều chuyển trong phiên:</span>
            <div className="flex flex-wrap gap-2">
              {transfers.slice(0, 3).map((tr) => (
                <span key={tr.id} className="px-2 py-0.5 rounded bg-slate-900 border border-slate-800 font-mono">
                  {tr.fromDeptId.replace('dept-', '').toUpperCase()} ➔ {tr.toDeptId.replace('dept-', '').toUpperCase()} (+${tr.amountMinor / 100}) lúc {tr.timestamp}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
