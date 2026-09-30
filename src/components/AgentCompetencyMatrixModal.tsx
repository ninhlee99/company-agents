import React, { useState, useMemo } from 'react';
import { CustomAgent } from '../types/company';
import { 
  X, 
  Award, 
  Sparkles, 
  Check, 
  Sliders, 
  Info,
  TrendingUp,
  BrainCircuit,
  Zap,
  ShieldCheck,
  Scale
} from 'lucide-react';
import {
  ResponsiveContainer,
  RadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  Radar,
  Legend,
  Tooltip
} from 'recharts';

interface AgentCompetencyMatrixModalProps {
  isOpen: boolean;
  onClose: () => void;
  agents: CustomAgent[];
  onOpenTraining?: (agentId: string) => void;
}

export interface CompetencyScore {
  communication: number; // 0-100
  executionSpeed: number; // 0-100
  decisionAccuracy: number; // 0-100
  fiduciaryCompliance: number; // 0-100
  autonomousSolving: number; // 0-100
  resourceEfficiency: number; // 0-100
}

const BASELINE_COMPETENCY: Record<string, CompetencyScore> = {
  'agent-ceo': {
    communication: 94,
    executionSpeed: 82,
    decisionAccuracy: 96,
    fiduciaryCompliance: 92,
    autonomousSolving: 95,
    resourceEfficiency: 88,
  },
  'agent-coo': {
    communication: 96,
    executionSpeed: 94,
    decisionAccuracy: 89,
    fiduciaryCompliance: 92,
    autonomousSolving: 91,
    resourceEfficiency: 93,
  },
  'agent-gov': {
    communication: 84,
    executionSpeed: 76,
    decisionAccuracy: 99,
    fiduciaryCompliance: 100,
    autonomousSolving: 93,
    resourceEfficiency: 97,
  },
  'agent-growth': {
    communication: 89,
    executionSpeed: 93,
    decisionAccuracy: 85,
    fiduciaryCompliance: 80,
    autonomousSolving: 90,
    resourceEfficiency: 86,
  },
  'agent-content': {
    communication: 86,
    executionSpeed: 97,
    decisionAccuracy: 82,
    fiduciaryCompliance: 78,
    autonomousSolving: 88,
    resourceEfficiency: 83,
  },
  'agent-analyst': {
    communication: 82,
    executionSpeed: 86,
    decisionAccuracy: 96,
    fiduciaryCompliance: 95,
    autonomousSolving: 89,
    resourceEfficiency: 92,
  },
  'agent-cfo': {
    communication: 85,
    executionSpeed: 84,
    decisionAccuracy: 97,
    fiduciaryCompliance: 98,
    autonomousSolving: 90,
    resourceEfficiency: 99,
  },
  'agent-experiment': {
    communication: 81,
    executionSpeed: 91,
    decisionAccuracy: 84,
    fiduciaryCompliance: 82,
    autonomousSolving: 95,
    resourceEfficiency: 85,
  },
  'agent-recruiter': {
    communication: 92,
    executionSpeed: 88,
    decisionAccuracy: 88,
    fiduciaryCompliance: 89,
    autonomousSolving: 86,
    resourceEfficiency: 88,
  },
};

const AGENT_COLORS = [
  '#6366f1', // Indigo
  '#10b981', // Emerald
  '#f59e0b', // Amber
  '#ec4899', // Pink
  '#06b6d4', // Cyan
  '#8b5cf6', // Purple
  '#ef4444', // Red
];

export const AgentCompetencyMatrixModal: React.FC<AgentCompetencyMatrixModalProps> = ({
  isOpen,
  onClose,
  agents,
  onOpenTraining,
}) => {
  // Preselect 3 key agents for direct comparison
  const [selectedAgentIds, setSelectedAgentIds] = useState<string[]>([
    'agent-growth',
    'agent-content',
    'agent-coo',
  ]);

  // Skill dimension metadata
  const dimensions = [
    { key: 'communication', label: 'Giao Tiếp (Communication)', icon: '💬', desc: 'Đồng bộ handoff, báo cáo & hiểu chỉ thị' },
    { key: 'executionSpeed', label: 'Tốc Độ Thực Thi (Execution Speed)', icon: '⚡', desc: 'Khả năng hoàn thành tác vụ/chu kỳ' },
    { key: 'decisionAccuracy', label: 'Độ Chính Xác (Decision Accuracy)', icon: '🎯', desc: 'Tỷ lệ quyết định đúng mục tiêu' },
    { key: 'fiduciaryCompliance', label: 'Tuân Thủ Fiduciary (Compliance)', icon: '⚖️', desc: 'Bảo vệ ngân sách & hiến pháp' },
    { key: 'autonomousSolving', label: 'Tự Chủ Giải Quyết (Problem Solving)', icon: '🧠', desc: 'Xử lý ngoại lệ không cần can thiệp' },
    { key: 'resourceEfficiency', label: 'Hiệu Quả Vốn (Resource Efficiency)', icon: '💎', desc: 'Tối ưu chi phí API & lương/việc' },
  ];

  // Calculate scores factoring in skill levels & multipliers
  const computeScores = (agent: CustomAgent): CompetencyScore => {
    const base = BASELINE_COMPETENCY[agent.id] || {
      communication: 80,
      executionSpeed: 80,
      decisionAccuracy: 80,
      fiduciaryCompliance: 80,
      autonomousSolving: 80,
      resourceEfficiency: 80,
    };

    const multiplierBonus = Math.round(((agent.taskMultiplier || 1.0) - 1.0) * 25);
    const skillBonus = ((agent.skillLevel || 1) - 1) * 3;

    return {
      communication: Math.min(100, base.communication + skillBonus),
      executionSpeed: Math.min(100, base.executionSpeed + multiplierBonus + skillBonus),
      decisionAccuracy: Math.min(100, base.decisionAccuracy + Math.round(multiplierBonus * 0.7) + skillBonus),
      fiduciaryCompliance: Math.min(100, base.fiduciaryCompliance + skillBonus),
      autonomousSolving: Math.min(100, base.autonomousSolving + Math.round(multiplierBonus * 0.8) + skillBonus),
      resourceEfficiency: Math.min(100, base.resourceEfficiency + Math.round(multiplierBonus * 0.9) + skillBonus),
    };
  };

  // Build Radar Chart Data
  const radarData = useMemo(() => {
    const selectedAgents = agents.filter((a) => selectedAgentIds.includes(a.id));

    return dimensions.map((dim) => {
      const row: any = {
        subject: dim.label.split(' ')[0], // Short name for axis
        fullName: dim.label,
      };

      selectedAgents.forEach((agent) => {
        const scores = computeScores(agent);
        row[agent.id] = (scores as any)[dim.key];
      });

      return row;
    });
  }, [agents, selectedAgentIds]);

  const toggleAgent = (id: string) => {
    if (selectedAgentIds.includes(id)) {
      if (selectedAgentIds.length <= 1) return; // Keep at least one
      setSelectedAgentIds(selectedAgentIds.filter((item) => item !== id));
    } else {
      if (selectedAgentIds.length >= 4) {
        // limit to 4 max on radar chart for clarity
        setSelectedAgentIds([...selectedAgentIds.slice(1), id]);
      } else {
        setSelectedAgentIds([...selectedAgentIds, id]);
      }
    }
  };

  const selectedAgentObjects = useMemo(() => {
    return agents.filter((a) => selectedAgentIds.includes(a.id));
  }, [agents, selectedAgentIds]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-3 sm:p-5 overflow-y-auto animate-in fade-in duration-200">
      <div className="bg-slate-900 border border-slate-700/80 rounded-2xl w-full max-w-5xl max-h-[92vh] flex flex-col shadow-2xl overflow-hidden my-auto">
        {/* Header */}
        <div className="p-4 sm:p-5 border-b border-slate-800 flex items-center justify-between bg-slate-950/70">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400 shrink-0">
              <BrainCircuit className="w-5 h-5 text-indigo-400" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-base font-bold text-white">
                  Ma Trận Năng Lực Chuẩn Hóa Nhân Sự AI (Competency Matrix)
                </h3>
                <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                  Radar Benchmark
                </span>
              </div>
              <p className="text-xs text-slate-400 mt-0.5">
                So sánh đa chiều 6 năng lực chuẩn hóa: Giao tiếp, Tốc độ thực thi, Độ chính xác, Tuân thủ Fiduciary, Giải quyết vấn đề &amp; Hiệu quả vốn
              </p>
            </div>
          </div>

          <button
            onClick={onClose}
            className="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content Area */}
        <div className="p-4 sm:p-6 overflow-y-auto space-y-6 flex-1 text-xs">
          {/* Agent Selection Pill Bar */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-slate-300 flex items-center gap-1.5">
                <Sliders className="w-3.5 h-3.5 text-indigo-400" />
                Chọn nhân sự để so sánh trên biểu đồ Radar (Tối đa 4 người cùng lúc):
              </span>
              <span className="text-[11px] text-slate-500 font-mono">
                Đang chọn: {selectedAgentIds.length}/4
              </span>
            </div>

            <div className="flex flex-wrap gap-2">
              {agents.map((agent, idx) => {
                const isSelected = selectedAgentIds.includes(agent.id);
                const colorIndex = selectedAgentIds.indexOf(agent.id);
                const color = colorIndex >= 0 ? AGENT_COLORS[colorIndex % AGENT_COLORS.length] : undefined;

                return (
                  <button
                    key={agent.id}
                    onClick={() => toggleAgent(agent.id)}
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold text-xs transition-all border ${
                      isSelected
                        ? 'bg-slate-800 text-white shadow-sm'
                        : 'bg-slate-950/60 text-slate-400 border-slate-800 hover:border-slate-700 hover:text-slate-200'
                    }`}
                    style={isSelected && color ? { borderColor: color, boxShadow: `0 0 10px ${color}25` } : {}}
                  >
                    <div
                      className="w-2.5 h-2.5 rounded-full shrink-0"
                      style={{ backgroundColor: isSelected && color ? color : '#475569' }}
                    />
                    <span>{agent.name}</span>
                    <span className="text-[10px] text-slate-400 font-mono">
                      ({agent.role})
                    </span>
                    {isSelected && <Check className="w-3 h-3 text-emerald-400 shrink-0 ml-0.5" />}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Radar Chart & High-Level Comparison Grid */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-center bg-slate-950/60 p-4 rounded-xl border border-slate-800">
            {/* Radar Chart */}
            <div className="lg:col-span-7 h-72 sm:h-80 w-full flex items-center justify-center">
              <ResponsiveContainer width="100%" height="100%">
                <RadarChart cx="50%" cy="50%" outerRadius="75%" data={radarData}>
                  <PolarGrid stroke="#334155" strokeDasharray="3 3" />
                  <PolarAngleAxis 
                    dataKey="subject" 
                    tick={{ fill: '#94a3b8', fontSize: 11, fontWeight: 600 }} 
                  />
                  <PolarRadiusAxis 
                    angle={30} 
                    domain={[0, 100]} 
                    tick={{ fill: '#64748b', fontSize: 9 }}
                  />
                  <Tooltip
                    content={({ active, payload }) => {
                      if (active && payload && payload.length) {
                        const dataItem = payload[0].payload;
                        return (
                          <div className="bg-slate-900 border border-slate-700 rounded-lg p-2.5 shadow-xl text-xs space-y-1 z-50">
                            <span className="font-bold text-white block border-b border-slate-800 pb-1">
                              {dataItem.fullName}
                            </span>
                            {payload.map((entry: any, i: number) => {
                              const agent = agents.find((a) => a.id === entry.dataKey);
                              return (
                                <div key={i} className="flex items-center justify-between gap-3 text-[11px]">
                                  <span style={{ color: entry.color }} className="font-semibold">
                                    {agent?.name || entry.name}:
                                  </span>
                                  <span className="font-mono font-bold text-white">
                                    {entry.value}/100
                                  </span>
                                </div>
                              );
                            })}
                          </div>
                        );
                      }
                      return null;
                    }}
                  />
                  {selectedAgentObjects.map((agent, index) => {
                    const color = AGENT_COLORS[index % AGENT_COLORS.length];
                    return (
                      <Radar
                        key={agent.id}
                        name={agent.name}
                        dataKey={agent.id}
                        stroke={color}
                        fill={color}
                        fillOpacity={0.25}
                        strokeWidth={2}
                      />
                    );
                  })}
                  <Legend 
                    wrapperStyle={{ paddingTop: 10, fontSize: '11px' }}
                    formatter={(value) => <span className="text-slate-300 font-semibold">{value}</span>}
                  />
                </RadarChart>
              </ResponsiveContainer>
            </div>

            {/* Quick Summary Insights */}
            <div className="lg:col-span-5 space-y-3">
              <span className="font-bold text-white text-xs block border-b border-slate-800 pb-1.5 flex items-center gap-1.5">
                <Sparkles className="w-4 h-4 text-yellow-400" />
                Đánh Giá Tổng Hợp Điểm Mạnh &amp; Khuyến Nghị:
              </span>

              <div className="space-y-2.5 max-h-72 overflow-y-auto pr-1">
                {selectedAgentObjects.map((agent, index) => {
                  const scores = computeScores(agent);
                  const color = AGENT_COLORS[index % AGENT_COLORS.length];
                  
                  // Compute average composite score
                  const avgScore = Math.round(
                    (scores.communication +
                      scores.executionSpeed +
                      scores.decisionAccuracy +
                      scores.fiduciaryCompliance +
                      scores.autonomousSolving +
                      scores.resourceEfficiency) / 6
                  );

                  // Find top dimension
                  const entries = Object.entries(scores) as [keyof CompetencyScore, number][];
                  entries.sort((a, b) => b[1] - a[1]);
                  const topDim = dimensions.find((d) => d.key === entries[0][0]);

                  return (
                    <div
                      key={agent.id}
                      className="p-3 rounded-lg bg-slate-900 border border-slate-800 space-y-1.5"
                      style={{ borderLeftWidth: 3, borderLeftColor: color }}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-1.5">
                          <strong className="text-white text-xs">{agent.name}</strong>
                          <span className="text-[10px] text-slate-400">({agent.role})</span>
                        </div>
                        <span className="font-mono font-black text-xs px-2 py-0.5 rounded bg-slate-950 border border-slate-800 text-indigo-300">
                          {avgScore}/100
                        </span>
                      </div>

                      <p className="text-[11px] text-slate-300 leading-relaxed">
                        Thế mạnh vượt trội: <span className="text-emerald-400 font-semibold">{topDim?.label.split(' ')[0]} ({entries[0][1]}/100)</span>.
                        {agent.taskMultiplier && agent.taskMultiplier > 1.0 ? (
                          <span className="text-cyan-300 ml-1">
                            Đã nâng cấp kỹ năng (Multiplier: {agent.taskMultiplier}x).
                          </span>
                        ) : (
                          <span className="text-slate-400 ml-1">
                            Tiềm năng tăng trưởng cao nếu tham gia khóa đào tạo chuyên sâu.
                          </span>
                        )}
                      </p>

                      {onOpenTraining && (
                        <div className="pt-1 flex justify-end">
                          <button
                            onClick={() => {
                              onClose();
                              onOpenTraining(agent.id);
                            }}
                            className="text-[10px] text-indigo-400 hover:text-indigo-300 font-semibold underline flex items-center gap-1"
                          >
                            <Zap className="w-3 h-3 text-yellow-400" /> Nâng cấp kỹ năng cho {agent.name}
                          </button>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          {/* Granular Dimension Comparison Table */}
          <div className="space-y-2">
            <span className="font-bold text-white text-xs block">
              Bảng So Sánh Chi Tiết Từng Chỉ Số Năng Lực Chuẩn Hóa
            </span>

            <div className="overflow-x-auto rounded-xl border border-slate-800">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="bg-slate-950 text-slate-400 text-[10px] uppercase tracking-wider font-semibold border-b border-slate-800">
                    <th className="py-2.5 px-3">Kích Thước Năng Lực</th>
                    {selectedAgentObjects.map((agent, i) => (
                      <th
                        key={agent.id}
                        className="py-2.5 px-3 text-center"
                        style={{ color: AGENT_COLORS[i % AGENT_COLORS.length] }}
                      >
                        {agent.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-800/60 bg-slate-900/50">
                  {dimensions.map((dim) => {
                    // Find max score for this dimension among selected agents
                    const scores = selectedAgentObjects.map((a) => (computeScores(a) as any)[dim.key] as number);
                    const maxScore = Math.max(...scores);

                    return (
                      <tr key={dim.key} className="hover:bg-slate-800/40 transition-colors">
                        <td className="py-2.5 px-3">
                          <div className="flex items-center gap-1.5 font-medium text-white text-[11px]">
                            <span>{dim.icon}</span>
                            <span>{dim.label}</span>
                          </div>
                          <span className="text-[10px] text-slate-500 block">{dim.desc}</span>
                        </td>

                        {selectedAgentObjects.map((agent) => {
                          const agentScores = computeScores(agent);
                          const val = (agentScores as any)[dim.key] as number;
                          const isMax = val === maxScore && selectedAgentObjects.length > 1;

                          return (
                            <td key={agent.id} className="py-2.5 px-3 text-center">
                              <div className="inline-flex items-center gap-1">
                                <span className={`font-mono font-bold text-xs ${
                                  isMax ? 'text-emerald-400' : val >= 85 ? 'text-white' : 'text-slate-400'
                                }`}>
                                  {val}/100
                                </span>
                                {isMax && (
                                  <span title="Dẫn đầu tiêu chí này" className="inline-flex">
                                    <Award className="w-3.5 h-3.5 text-yellow-400" />
                                  </span>
                                )}
                              </div>
                              <div className="w-16 h-1.5 bg-slate-800 rounded-full mx-auto mt-1 overflow-hidden">
                                <div
                                  className={`h-full rounded-full ${
                                    isMax ? 'bg-emerald-400' : 'bg-indigo-500'
                                  }`}
                                  style={{ width: `${val}%` }}
                                />
                              </div>
                            </td>
                          );
                        })}
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-slate-800 bg-slate-950/70 flex items-center justify-between">
          <div className="flex items-center gap-2 text-[11px] text-slate-400">
            <Info className="w-4 h-4 text-indigo-400 shrink-0" />
            <span>
              Điểm số năng lực được tự động cập nhật sau mỗi khóa Đào Tạo Kỹ Năng (Upskilling) và chu kỳ kinh doanh.
            </span>
          </div>

          <button
            onClick={onClose}
            className="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs transition-colors shadow-sm"
          >
            Đóng Ma Trận
          </button>
        </div>
      </div>
    </div>
  );
};
