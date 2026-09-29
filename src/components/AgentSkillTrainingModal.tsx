import React, { useState } from 'react';
import { CustomAgent, CompanySnapshot, SkillTrainingCourse } from '../types/company';
import { 
  GraduationCap, 
  X, 
  Sparkles, 
  TrendingUp, 
  Zap, 
  Award, 
  DollarSign, 
  CheckCircle2, 
  AlertCircle,
  ArrowRight,
  ShieldCheck,
  Brain
} from 'lucide-react';

interface AgentSkillTrainingModalProps {
  isOpen: boolean;
  onClose: () => void;
  agents: CustomAgent[];
  snapshot: CompanySnapshot;
  onTrainAgent: (agentId: string, course: SkillTrainingCourse) => Promise<{ success: boolean; reason?: string }>;
  initialSelectedAgentId?: string | null;
}

export const AgentSkillTrainingModal: React.FC<AgentSkillTrainingModalProps> = ({
  isOpen,
  onClose,
  agents,
  snapshot,
  onTrainAgent,
  initialSelectedAgentId,
}) => {
  const [selectedAgentId, setSelectedAgentId] = useState<string>(
    initialSelectedAgentId || (agents[0]?.id || 'agent-content')
  );
  const [selectedCourseId, setSelectedCourseId] = useState<string>('course-prompt-cache');
  const [isTraining, setIsTraining] = useState(false);
  const [trainSuccess, setTrainSuccess] = useState<string | null>(null);
  const [trainError, setTrainError] = useState<string | null>(null);

  if (!isOpen) return null;

  const selectedAgent = agents.find((a) => a.id === selectedAgentId) || agents[0];

  // 5 Specialized Upskilling Training Courses
  const trainingCourses: SkillTrainingCourse[] = [
    {
      id: 'course-prompt-cache',
      name: 'Tối Ưu Hóa Prompt Caching & Batching LLM',
      department: 'All',
      description: 'Huấn luyện kỹ thuật pinning prompt prefix và hàng đợi batching, giúp giảm 45% độ trễ và đẩy mạnh tốc độ xử lý.',
      cost_minor: 40000, // $400
      multiplierBoost: 0.25, // +25%
      tasksBonus: 8,
      badge: '🧠 Prompt Architect',
      levelRequired: 1,
    },
    {
      id: 'course-affiliate-deals',
      name: 'Thuật Toán Quét Ngách & Săn Deal Hoa Hồng Cao',
      department: 'Growth',
      description: 'Nâng cấp khả năng phát hiện sản phẩm TikTok Shop có tỷ suất EPC > $1.20 và đàm phán hợp đồng độc quyền với các brand.',
      cost_minor: 60000, // $600
      multiplierBoost: 0.30, // +30%
      tasksBonus: 12,
      badge: '🎯 Deal Hunter Pro',
      levelRequired: 1,
    },
    {
      id: 'course-forensic-audit',
      name: 'Kiểm Toán Dòng Tiền & Phát Hiện Bất Thường',
      department: 'Leadership',
      description: 'Chuyên môn hóa kỹ thuật đối soát vi mô và lập báo cáo tài chính tuân thủ chuẩn Fiduciary của Hiến pháp.',
      cost_minor: 50000, // $500
      multiplierBoost: 0.25, // +25%
      tasksBonus: 10,
      badge: '⚖️ Forensic Auditor',
      levelRequired: 1,
    },
    {
      id: 'course-video-pipeline',
      name: 'Điều Phối Dây Chuyền Render Video Tốc Độ Cao',
      department: 'Growth',
      description: 'Tự động hóa pipeline tổng hợp video 60fps, loại bỏ hoàn toàn hiện tượng nghẽn cổ chai (bottleneck) ở hàng đợi.',
      cost_minor: 55000, // $550
      multiplierBoost: 0.30, // +30%
      tasksBonus: 12,
      badge: '⚡ Media Pipeline Master',
      levelRequired: 1,
    },
    {
      id: 'course-capital-alloc',
      name: 'Phân Bổ Vốn Chiến Lược & Quản Trị Cấp Cao',
      department: 'Leadership',
      description: 'Nâng cao năng lực mô phỏng rủi ro, cân bằng tỷ lệ chi trả cổ tức và bảo vệ quỹ sinh tồn tối thiểu 90 ngày.',
      cost_minor: 75000, // $750
      multiplierBoost: 0.40, // +40%
      tasksBonus: 16,
      badge: '👑 Executive Master',
      levelRequired: 1,
    },
  ];

  const selectedCourse = trainingCourses.find((c) => c.id === selectedCourseId) || trainingCourses[0];

  const currentLevel = selectedAgent?.skillLevel || 1;
  const currentMultiplier = selectedAgent?.taskMultiplier || 1.0;
  const projectedMultiplier = Math.round((currentMultiplier + selectedCourse.multiplierBoost) * 100) / 100;
  const projectedTasks = (selectedAgent?.tasksCompleted || 0) + selectedCourse.tasksBonus;

  const currentSalary = selectedAgent?.salary_minor ? Math.round(selectedAgent.salary_minor / 100) : 350;
  const currentCostPerTask = Math.round((currentSalary / Math.max(1, selectedAgent?.tasksCompleted || 1)) * 10) / 10;
  const projectedCostPerTask = Math.round((currentSalary / Math.max(1, projectedTasks)) * 10) / 10;

  const canAfford = snapshot.cash_minor >= selectedCourse.cost_minor;

  const handleStartTraining = async () => {
    if (!selectedAgent || !canAfford) return;
    setIsTraining(true);
    setTrainError(null);
    setTrainSuccess(null);

    const res = await onTrainAgent(selectedAgent.id, selectedCourse);
    setIsTraining(false);

    if (res.success) {
      setTrainSuccess(
        `Chúc mừng! ${selectedAgent.name} đã hoàn thành khóa đào tạo "${selectedCourse.name}". Multiplier công việc tăng lên ${projectedMultiplier}x (+${selectedCourse.tasksBonus} việc hoàn thành)!`
      );
    } else {
      setTrainError(res.reason || 'Không thể thực hiện đào tạo. Vui lòng kiểm tra lại ngân quỹ.');
    }
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4 overflow-y-auto">
      <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-3xl w-full p-5 md:p-6 space-y-4 shadow-2xl relative">
        {/* Close Button */}
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Modal Title */}
        <div className="flex items-center gap-3 border-b border-slate-800 pb-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white shadow-md shadow-indigo-500/20">
            <GraduationCap className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <span>Đào Tạo Kỹ Năng &amp; Nâng Cao Năng Suất (Agent Skill Training)</span>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-mono bg-purple-500/20 text-purple-300 border border-purple-500/30">
                Upskilling
              </span>
            </h3>
            <p className="text-xs text-slate-400">
              Đầu tư ngân quỹ kho bạc để tăng hệ số nhân (Multiplier) công việc, cải thiện hiệu suất và phá bỏ điểm nghẽn
            </p>
          </div>
        </div>

        {/* Treasury Status Alert */}
        <div className="flex items-center justify-between p-3 rounded-xl bg-slate-950 border border-slate-800 text-xs">
          <div className="flex items-center gap-2">
            <DollarSign className="w-4 h-4 text-emerald-400" />
            <span className="text-slate-300">Ngân quỹ Kho Bạc sẵn sàng đầu tư:</span>
            <strong className="font-mono text-emerald-400 font-bold">
              ${(snapshot.cash_minor / 100).toLocaleString()}
            </strong>
          </div>
          <span className="text-[11px] text-slate-400 font-mono">Runway: {snapshot.runway_days} ngày</span>
        </div>

        {/* Step 1: Select Agent */}
        <div className="space-y-2">
          <label className="text-xs font-bold text-slate-300 block">
            1. Chọn Nhân Sự AI Cần Đào Tạo (Select Agent):
          </label>
          <div className="grid grid-cols-3 sm:grid-cols-5 md:grid-cols-9 gap-1.5">
            {agents.map((agent) => (
              <button
                key={agent.id}
                type="button"
                onClick={() => {
                  setSelectedAgentId(agent.id);
                  setTrainSuccess(null);
                  setTrainError(null);
                }}
                className={`p-2 rounded-lg border text-center transition-all flex flex-col items-center gap-1 ${
                  selectedAgentId === agent.id
                    ? 'bg-indigo-600/20 border-indigo-500 text-white shadow-sm ring-1 ring-indigo-500'
                    : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-white hover:bg-slate-850'
                }`}
              >
                <span className="font-bold text-[11px] truncate w-full">{agent.name.replace(' Agent', '')}</span>
                <span className="text-[9px] font-mono text-indigo-300">
                  {agent.taskMultiplier ? `${agent.taskMultiplier}x` : '1.0x'}
                </span>
              </button>
            ))}
          </div>
        </div>

        {/* Step 2: Select Course */}
        <div className="space-y-2">
          <label className="text-xs font-bold text-slate-300 block">
            2. Chọn Khóa Đào Tạo Chuyên Môn (Training Curriculum):
          </label>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {trainingCourses.map((course) => (
              <div
                key={course.id}
                onClick={() => {
                  setSelectedCourseId(course.id);
                  setTrainSuccess(null);
                  setTrainError(null);
                }}
                className={`p-3 rounded-xl border cursor-pointer transition-all flex flex-col justify-between ${
                  selectedCourseId === course.id
                    ? 'bg-gradient-to-br from-indigo-950/40 to-slate-900 border-indigo-500 shadow-sm ring-1 ring-indigo-500/50'
                    : 'bg-slate-950 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-xs text-white">{course.name}</span>
                    <span className="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-amber-500/10 text-amber-300 border border-amber-500/30">
                      ${course.cost_minor / 100}
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-400 line-clamp-2">{course.description}</p>
                </div>

                <div className="flex items-center justify-between pt-2 border-t border-slate-800/80 mt-2 text-[10px] font-mono">
                  <span className="text-purple-300 font-bold">{course.badge}</span>
                  <span className="text-emerald-400 font-bold">
                    +{Math.round(course.multiplierBoost * 100)}% Multiplier (+{course.tasksBonus} việc)
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Step 3: Projected ROI Impact Breakdown */}
        {selectedAgent && (
          <div className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
            <span className="text-xs font-bold text-indigo-300 flex items-center gap-1.5">
              <Zap className="w-3.5 h-3.5 text-yellow-400" />
              <span>Dự Báo Hiệu Quả Đầu Tư Sau Khi Đào Tạo (Upskilling ROI Simulation):</span>
            </span>

            <div className="grid grid-cols-3 gap-2 text-center text-xs">
              <div className="p-2 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-[10px] text-slate-400 block">Hệ Số Năng Suất (Multiplier)</span>
                <div className="flex items-center justify-center gap-1 mt-1 font-mono font-bold">
                  <span className="text-slate-400">{currentMultiplier}x</span>
                  <ArrowRight className="w-3 h-3 text-indigo-400" />
                  <span className="text-emerald-400">{projectedMultiplier}x</span>
                </div>
              </div>

              <div className="p-2 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-[10px] text-slate-400 block">Tác Vụ Hoàn Thành (Tasks)</span>
                <div className="flex items-center justify-center gap-1 mt-1 font-mono font-bold">
                  <span className="text-slate-400">{selectedAgent.tasksCompleted}</span>
                  <ArrowRight className="w-3 h-3 text-indigo-400" />
                  <span className="text-emerald-400">{projectedTasks}</span>
                </div>
              </div>

              <div className="p-2 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-[10px] text-slate-400 block">Chi Phí / Tác Vụ</span>
                <div className="flex items-center justify-center gap-1 mt-1 font-mono font-bold">
                  <span className="text-slate-400">${currentCostPerTask}</span>
                  <ArrowRight className="w-3 h-3 text-indigo-400" />
                  <span className="text-emerald-400">${projectedCostPerTask}</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Feedback messages */}
        {trainSuccess && (
          <div className="p-3 rounded-lg bg-emerald-950/40 border border-emerald-500/50 text-emerald-300 text-xs flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 shrink-0" />
            <span>{trainSuccess}</span>
          </div>
        )}

        {trainError && (
          <div className="p-3 rounded-lg bg-rose-950/40 border border-rose-500/50 text-rose-300 text-xs flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{trainError}</span>
          </div>
        )}

        {/* Action Button */}
        <div className="flex items-center justify-between pt-2 border-t border-slate-800">
          <div className="text-xs text-slate-400">
            Chi phí đào tạo: <strong className="text-white font-mono">${selectedCourse.cost_minor / 100} USD</strong> (trừ trực tiếp từ Kho Bạc)
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-xl text-xs font-semibold bg-slate-800 hover:bg-slate-700 text-slate-300 transition-colors"
            >
              Đóng
            </button>

            <button
              type="button"
              onClick={handleStartTraining}
              disabled={isTraining || !canAfford}
              className={`flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-bold text-white transition-all shadow-md ${
                !canAfford
                  ? 'bg-slate-800 text-slate-500 cursor-not-allowed'
                  : isTraining
                  ? 'bg-indigo-700 cursor-wait'
                  : 'bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 active:scale-95'
              }`}
            >
              <Sparkles className={`w-3.5 h-3.5 ${isTraining ? 'animate-spin' : ''}`} />
              <span>{isTraining ? 'Đang Huấn Luyện...' : `Đầu Tư Đào Tạo ($${selectedCourse.cost_minor / 100})`}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
