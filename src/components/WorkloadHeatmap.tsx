import React, { useState, useMemo } from 'react';
import { WorkloadHeatmapCell, CustomAgent } from '../types/company';
import { 
  Flame, 
  AlertTriangle, 
  Activity, 
  Layers, 
  HelpCircle, 
  Info, 
  Sparkles,
  ArrowRight,
  ShieldCheck,
  CheckCircle2
} from 'lucide-react';

interface WorkloadHeatmapProps {
  agents?: CustomAgent[];
  onOpenTraining?: (agentId?: string) => void;
}

export const WorkloadHeatmap: React.FC<WorkloadHeatmapProps> = ({ 
  agents = [],
  onOpenTraining 
}) => {
  const [selectedDept, setSelectedDept] = useState<'All' | 'Leadership' | 'Growth' | 'Ops'>('All');
  const [viewMode, setViewMode] = useState<'all' | 'bottlenecks'>('all');
  const [hoveredCell, setHoveredCell] = useState<WorkloadHeatmapCell | null>(null);

  // Define 9 canonical agents
  const coreAgentList = useMemo(() => [
    { id: 'agent-gov', name: 'Governor', role: 'Hiến Pháp & Quỹ Tiền', department: 'Leadership' as const, baseLoad: 45 },
    { id: 'agent-ceo', name: 'CEO', role: 'Tổng Giám Đốc', department: 'Leadership' as const, baseLoad: 48 },
    { id: 'agent-cfo', name: 'CFO', role: 'Giám Đốc Tài Chính', department: 'Leadership' as const, baseLoad: 38 },
    { id: 'agent-coo', name: 'COO', role: 'Giám Đốc Vận Hành', department: 'Ops' as const, baseLoad: 68 },
    { id: 'agent-growth', name: 'Growth Lead', role: 'Kinh Doanh & Traffic', department: 'Growth' as const, baseLoad: 72 },
    { id: 'agent-content', name: 'Content Lead', role: 'Sáng Tạo Nội Dung', department: 'Growth' as const, baseLoad: 78 },
    { id: 'agent-recruiter', name: 'Recruiter', role: 'Tuyển Dụng', department: 'Ops' as const, baseLoad: 32 },
    { id: 'agent-analyst', name: 'Analyst', role: 'Phân Tích Dữ Liệu', department: 'Ops' as const, baseLoad: 58 },
    { id: 'agent-experiment', name: 'Experimenter', role: 'Nghiên Cứu A/B Test', department: 'Growth' as const, baseLoad: 54 },
  ], []);

  // Filter agents by department
  const filteredAgents = useMemo(() => {
    if (selectedDept === 'All') return coreAgentList;
    return coreAgentList.filter((a) => a.department === selectedDept);
  }, [coreAgentList, selectedDept]);

  // Generate deterministic, realistic 30-cycle heatmap workload data
  const heatmapData = useMemo(() => {
    const data: Record<string, WorkloadHeatmapCell[]> = {};

    coreAgentList.forEach((agent) => {
      data[agent.id] = [];
      for (let cycle = 1; cycle <= 30; cycle++) {
        // Deterministic pseudo-random variation based on agent and cycle
        const cycleFactor = Math.sin((cycle * 3.7 + agent.name.length) * 0.5) * 18;
        const trend = (cycle / 30) * 8; // gradual increase over cycles
        let workload = Math.round(agent.baseLoad + cycleFactor + trend);

        // Specific historical bottleneck scenarios
        let isBottleneck = false;
        let bottleneckType: WorkloadHeatmapCell['bottleneckType'] = undefined;
        let notes = undefined;

        if (agent.id === 'agent-content' && cycle === 8) {
          workload = 94;
          isBottleneck = true;
          bottleneckType = 'Queue Overflow';
          notes = 'Hàng đợi render video 60fps bị quá tải 94%; thiếu GPU quota.';
        } else if (agent.id === 'agent-growth' && cycle === 12) {
          workload = 92;
          isBottleneck = true;
          bottleneckType = 'Manual Escalation';
          notes = 'Spike chiến dịch TikTok Shop; 15 đề xuất ngân sách dồn ứ chờ duyệt.';
        } else if (agent.id === 'agent-coo' && cycle === 14) {
          workload = 91;
          isBottleneck = true;
          bottleneckType = 'Queue Overflow';
          notes = 'Backlog công việc vượt quá công suất (12 backlog / 22 capacity).';
        } else if (agent.id === 'agent-analyst' && cycle === 20) {
          workload = 89;
          isBottleneck = true;
          bottleneckType = 'Token Exhaustion';
          notes = 'Đối soát dòng tiền đa sàn thương mại điện tử dồn đọng cuối tháng.';
        } else if (agent.id === 'agent-cfo' && cycle === 25) {
          workload = 88;
          isBottleneck = true;
          bottleneckType = 'Compliance Lock';
          notes = 'Kiểm toán đột xuất kho bạc sau khi kích hoạt quỹ đầu tư thử nghiệm.';
        } else if (workload >= 86) {
          isBottleneck = true;
          bottleneckType = 'Queue Overflow';
          notes = `Tải vận hành vượt ngưỡng an toàn (${workload}%). Cần upskill hoặc phân bổ thêm tác vụ.`;
        }

        // Clamp between 15% and 98%
        workload = Math.max(18, Math.min(98, workload));
        const tasksProcessed = Math.max(1, Math.round((workload / 100) * 14));

        data[agent.id].push({
          cycle,
          agentId: agent.id,
          agentName: agent.name,
          agentRole: agent.role,
          department: agent.department,
          workloadPct: workload,
          tasksProcessed,
          isBottleneck,
          bottleneckType,
          notes,
        });
      }
    });

    return data;
  }, [coreAgentList]);

  // Overall Statistics
  const stats = useMemo(() => {
    let totalCells = 0;
    let totalLoad = 0;
    let bottleneckCount = 0;
    let highestLoadAgent = { name: '', avgLoad: 0 };

    coreAgentList.forEach((agent) => {
      const cells = heatmapData[agent.id] || [];
      const sum = cells.reduce((acc, c) => acc + c.workloadPct, 0);
      const avg = sum / cells.length;
      if (avg > highestLoadAgent.avgLoad) {
        highestLoadAgent = { name: agent.name, avgLoad: Math.round(avg) };
      }
      cells.forEach((c) => {
        totalCells++;
        totalLoad += c.workloadPct;
        if (c.isBottleneck) bottleneckCount++;
      });
    });

    return {
      avgSystemLoad: Math.round((totalLoad / totalCells) * 10) / 10,
      bottleneckCount,
      highestLoadAgent,
    };
  }, [coreAgentList, heatmapData]);

  // Get cell color based on workload & view mode
  const getCellClasses = (cell: WorkloadHeatmapCell) => {
    if (viewMode === 'bottlenecks') {
      if (cell.isBottleneck) {
        return 'bg-rose-600 text-white font-bold ring-2 ring-rose-400/80 animate-pulse';
      }
      return 'bg-slate-900/40 border border-slate-800/40 text-slate-600 opacity-40';
    }

    if (cell.isBottleneck || cell.workloadPct >= 86) {
      return 'bg-gradient-to-br from-rose-600 to-rose-700 text-white ring-1 ring-rose-400 font-bold shadow-sm shadow-rose-900/50';
    }
    if (cell.workloadPct >= 72) {
      return 'bg-amber-600/80 text-amber-100 hover:bg-amber-500';
    }
    if (cell.workloadPct >= 50) {
      return 'bg-emerald-600/70 text-emerald-100 hover:bg-emerald-500';
    }
    if (cell.workloadPct >= 35) {
      return 'bg-cyan-900/60 text-cyan-200 hover:bg-cyan-800';
    }
    return 'bg-slate-800/60 text-slate-400 hover:bg-slate-700';
  };

  return (
    <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3.5 shadow-sm">
      {/* Header */}
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3 border-b border-slate-800 pb-3">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
            <Flame className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-white text-xs block">
                Bản Đồ Nhiệt Tải Vận Hành &amp; Điểm Nghẽn 30 Chu Kỳ (Workload Heatmap)
              </span>
              <span className="px-2 py-0.2 rounded-full font-mono text-[10px] bg-amber-500/20 text-amber-300 border border-amber-500/30 font-bold">
                Chu Kỳ #1 ➔ #30
              </span>
            </div>
            <span className="text-[10px] text-slate-400">
              Phát hiện điểm nghẽn năng suất (Bottlenecks) và tình trạng quá tải cục bộ của từng Agent
            </span>
          </div>
        </div>

        {/* View Mode & Filter Controls */}
        <div className="flex flex-wrap items-center gap-2 text-[11px]">
          {/* Department Filter */}
          <div className="flex bg-slate-950 p-0.5 rounded-lg border border-slate-800">
            {(['All', 'Leadership', 'Growth', 'Ops'] as const).map((dept) => (
              <button
                key={dept}
                onClick={() => setSelectedDept(dept)}
                className={`px-2 py-0.5 rounded font-semibold transition-all ${
                  selectedDept === dept ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                }`}
              >
                {dept === 'All' ? 'Tất cả' : dept === 'Leadership' ? 'Lãnh Đạo' : dept === 'Growth' ? 'Kinh Doanh' : 'Vận Hành'}
              </button>
            ))}
          </div>

          {/* View Mode Switcher */}
          <div className="flex bg-slate-950 p-0.5 rounded-lg border border-slate-800">
            <button
              onClick={() => setViewMode('all')}
              className={`px-2 py-0.5 rounded font-semibold transition-all ${
                viewMode === 'all' ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              Tất Cả Tải (% Load)
            </button>
            <button
              onClick={() => setViewMode('bottlenecks')}
              className={`px-2 py-0.5 rounded font-semibold transition-all flex items-center gap-1 ${
                viewMode === 'bottlenecks' ? 'bg-rose-600 text-white shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              <AlertTriangle className="w-3 h-3 text-rose-300" />
              <span>Chỉ Điểm Nghẽn</span>
            </button>
          </div>

          {onOpenTraining && (
            <button
              onClick={() => onOpenTraining()}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-[11px] font-bold shadow-sm transition-all active:scale-95"
            >
              <Sparkles className="w-3 h-3 text-yellow-300" />
              <span>Đào Tạo Kỹ Năng (Upskill)</span>
            </button>
          )}
        </div>
      </div>

      {/* Quick Summary Highlights */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
        <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800">
          <span className="text-[10px] text-slate-400 block">Tải Bình Quân Hệ Thống</span>
          <span className="font-bold text-white text-sm font-mono">{stats.avgSystemLoad}%</span>
          <span className="text-[10px] text-emerald-400 block">🟢 Ngưỡng an toàn (&lt;75%)</span>
        </div>

        <div className="p-2.5 rounded-lg bg-rose-950/20 border border-rose-500/30">
          <span className="text-[10px] text-rose-400 block font-semibold flex items-center gap-1">
            <AlertTriangle className="w-3 h-3" /> Điểm Nghẽn Phát Hiện
          </span>
          <span className="font-bold text-rose-300 text-sm font-mono">{stats.bottleneckCount} chu kỳ</span>
          <span className="text-[10px] text-rose-400/80 block">Tải công việc &ge; 86%</span>
        </div>

        <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800">
          <span className="text-[10px] text-slate-400 block">Vị Trí Tải Cao Nhất</span>
          <span className="font-bold text-amber-400 text-xs truncate block">{stats.highestLoadAgent.name}</span>
          <span className="text-[10px] text-slate-400 block">TB: {stats.highestLoadAgent.avgLoad}% công suất</span>
        </div>

        <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800">
          <span className="text-[10px] text-slate-400 block">Giải Pháp Governor</span>
          <span className="font-bold text-indigo-300 text-xs block">Agent Upskilling</span>
          <span className="text-[10px] text-slate-400 block">Tăng Task Multiplier 1.25x-1.5x</span>
        </div>
      </div>

      {/* Heatmap Grid Container */}
      <div className="overflow-x-auto scrollbar-thin scrollbar-thumb-slate-700 pb-2">
        <div className="min-w-[760px] space-y-1.5">
          {/* Cycle Axis Header */}
          <div className="flex items-center text-[10px] text-slate-400 border-b border-slate-800 pb-1">
            <div className="w-32 shrink-0 font-semibold text-slate-300">Nhân Sự AI \ Chu Kỳ</div>
            <div className="flex-1 grid grid-cols-30 gap-1 text-center font-mono">
              {Array.from({ length: 30 }, (_, i) => i + 1).map((c) => (
                <span
                  key={c}
                  className={`text-[9px] ${c === 14 ? 'text-indigo-400 font-bold underline' : ''}`}
                  title={`Chu kỳ #${c}`}
                >
                  {c % 5 === 0 || c === 1 || c === 14 ? `${c}` : '·'}
                </span>
              ))}
            </div>
          </div>

          {/* Agent Rows */}
          {filteredAgents.map((agent) => {
            const cells = heatmapData[agent.id] || [];
            return (
              <div key={agent.id} className="flex items-center gap-1 group">
                {/* Agent Label */}
                <div className="w-32 shrink-0 flex flex-col justify-center pr-2">
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-xs text-white truncate group-hover:text-indigo-300 transition-colors">
                      {agent.name}
                    </span>
                    <span className="text-[9px] font-mono text-slate-400">
                      {agent.department.slice(0, 3)}
                    </span>
                  </div>
                  <span className="text-[9px] text-slate-400 truncate">{agent.role}</span>
                </div>

                {/* 30 Cycle Cells */}
                <div className="flex-1 grid grid-cols-30 gap-1">
                  {cells.map((cell) => {
                    const isHovered = hoveredCell?.agentId === cell.agentId && hoveredCell?.cycle === cell.cycle;
                    return (
                      <div
                        key={cell.cycle}
                        onMouseEnter={() => setHoveredCell(cell)}
                        onClick={() => setHoveredCell(cell)}
                        className={`h-7 rounded cursor-pointer transition-all flex items-center justify-center text-[9px] font-mono relative ${getCellClasses(
                          cell
                        )} ${isHovered ? 'scale-125 z-20 shadow-lg ring-2 ring-white' : ''}`}
                        title={`Chu kỳ #${cell.cycle} - ${agent.name}: ${cell.workloadPct}% tải`}
                      >
                        {cell.isBottleneck ? '!' : cell.workloadPct >= 80 ? cell.workloadPct : ''}
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Heatmap Legend */}
      <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-slate-800 text-[10px]">
        <div className="flex items-center gap-2">
          <span className="text-slate-400 font-semibold">Mức Độ Tải:</span>
          <div className="flex items-center gap-1.5 font-mono">
            <span className="flex items-center gap-1">
              <span className="w-2.5 h-2.5 rounded bg-slate-800 inline-block" /> &lt;35%
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2.5 h-2.5 rounded bg-cyan-900 inline-block" /> 35-50%
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2.5 h-2.5 rounded bg-emerald-600 inline-block" /> 50-70% (Tối ưu)
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2.5 h-2.5 rounded bg-amber-600 inline-block" /> 70-85% (Cao)
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2.5 h-2.5 rounded bg-rose-600 inline-block" /> &ge;86% (Nghẽn cổ chai)
            </span>
          </div>
        </div>

        <span className="text-slate-400 italic">
          💡 Rê chuột vào từng ô vuông để xem chi tiết chẩn đoán và nguyên nhân nghẽn
        </span>
      </div>

      {/* Interactive Tooltip Card for Selected / Hovered Cell */}
      {hoveredCell && (
        <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-xs flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 animate-in fade-in duration-150">
          <div className="space-y-0.5">
            <div className="flex items-center gap-2">
              <strong className="text-white text-xs">{hoveredCell.agentName}</strong>
              <span className="text-[10px] text-slate-400">({hoveredCell.agentRole} • {hoveredCell.department})</span>
              <span className="px-1.5 py-0.2 rounded text-[10px] font-mono bg-slate-800 text-indigo-300">
                Chu kỳ #{hoveredCell.cycle}
              </span>
              {hoveredCell.isBottleneck ? (
                <span className="px-2 py-0.2 rounded-full font-bold text-[10px] bg-rose-500/20 text-rose-300 border border-rose-500/30 flex items-center gap-1">
                  <AlertTriangle className="w-3 h-3" /> Điểm Nghẽn: {hoveredCell.bottleneckType}
                </span>
              ) : (
                <span className="px-2 py-0.2 rounded-full text-[10px] bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center gap-1">
                  <CheckCircle2 className="w-3 h-3" /> Vận Hành Ổn Định
                </span>
              )}
            </div>
            <p className="text-[11px] text-slate-300">
              {hoveredCell.notes || `Khối lượng công việc bình thường. Đã xử lý ${hoveredCell.tasksProcessed} tác vụ trong chu kỳ này.`}
            </p>
          </div>

          <div className="flex items-center gap-3 shrink-0">
            <div className="text-right font-mono">
              <div className="text-xs font-bold text-white">{hoveredCell.workloadPct}% Công Suất</div>
              <div className="text-[10px] text-slate-400">{hoveredCell.tasksProcessed} việc/kỳ</div>
            </div>

            {onOpenTraining && (
              <button
                onClick={() => onOpenTraining(hoveredCell.agentId)}
                className="px-2.5 py-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-[10px] transition-all flex items-center gap-1 shrink-0"
              >
                <span>Upskill Agent</span>
                <ArrowRight className="w-3 h-3" />
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
