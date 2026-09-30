import React, { useState } from 'react';
import { CompanySnapshot, CustomAgent, AgentTaskItem, CandidateProfile } from '../types/company';
import { 
  Users, 
  Plus, 
  Send, 
  CheckCircle2, 
  BarChart3, 
  Power, 
  X,
  History,
  TrendingUp,
  Clock,
  Sparkles,
  Layers,
  ArrowRight,
  Download,
  Award,
  AlertTriangle,
  ArrowUpDown,
  Flame,
  ChevronDown,
  ChevronUp,
  GraduationCap,
  Zap,
  Bot,
  ToggleLeft,
  ToggleRight,
  BrainCircuit,
  Briefcase,
  UserCheck,
  Star,
  FileCheck2,
  ShieldCheck,
  RotateCw
} from 'lucide-react';
import { AgentCompetencyMatrixModal } from './AgentCompetencyMatrixModal';
import {
  ResponsiveContainer,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid,
  Legend
} from 'recharts';

interface ManageAgentsProps {
  snapshot: CompanySnapshot;
  agents: CustomAgent[];
  candidates?: CandidateProfile[];
  onHireAgent: (data: { name: string; role: string; department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech'; description: string; salary_minor: number }) => Promise<{ success: boolean; reason?: string }>;
  onToggleStatus: (agentId: string) => void;
  onOpenTraining?: (agentId?: string) => void;
  onAutoRecruit?: (thresholdMinor?: number) => Promise<{ success: boolean; reason?: string }>;
  onAutoTalentCycle?: (options: { autoRecruit: boolean; autoTrain: boolean; cashSafetyThreshold: number }) => Promise<{ success: boolean; message: string; actionsTaken: boolean }>;
  onInterviewCandidate?: (candidateId: string) => Promise<any>;
  onHireCandidate?: (candidateId: string) => Promise<any>;
}

export const ManageAgents: React.FC<ManageAgentsProps> = ({
  snapshot,
  agents,
  candidates = [],
  onHireAgent,
  onToggleStatus,
  onOpenTraining,
  onAutoRecruit,
  onAutoTalentCycle,
  onInterviewCandidate,
  onHireCandidate,
}) => {
  const [mainView, setMainView] = useState<'roster' | 'talent_market'>('roster');
  const [filter, setFilter] = useState<'All' | 'Leadership' | 'Growth' | 'Ops'>('All');
  const [showHireModal, setShowHireModal] = useState(false);
  const [showCompetencyMatrix, setShowCompetencyMatrix] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [hireError, setHireError] = useState<string | null>(null);

  // Candidate Interview Modal State
  const [interviewingCandidate, setInterviewingCandidate] = useState<CandidateProfile | null>(null);
  const [interviewResult, setInterviewResult] = useState<any>(null);
  const [isInterviewing, setIsInterviewing] = useState(false);
  const [hireSuccessNotice, setHireSuccessNotice] = useState<string | null>(null);

  // Form State
  const [name, setName] = useState('');
  const [role, setRole] = useState('');
  const [department, setDepartment] = useState<'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech'>('Growth');
  const [description, setDescription] = useState('');
  const [salaryUsd, setSalaryUsd] = useState(1500);

  const [quickInstruction, setQuickInstruction] = useState('');
  const [instructionSent, setInstructionSent] = useState(false);

  // Departmental Performance Report State
  const [reportSort, setReportSort] = useState<'roi' | 'overpaid' | 'tasks' | 'cost'>('roi');
  const [reportDeptFilter, setReportDeptFilter] = useState<'All' | 'Leadership' | 'Growth' | 'Ops'>('All');
  const [showDetailedReportList, setShowDetailedReportList] = useState(true);

  // Task History Modal State
  const [selectedAgentForHistory, setSelectedAgentForHistory] = useState<CustomAgent | null>(null);
  const [taskHistory, setTaskHistory] = useState<AgentTaskItem[]>([]);
  const [isLoadingHistory, setIsLoadingHistory] = useState(false);

  // Auto-Recruit Engine State
  const [autoRecruitEnabled, setAutoRecruitEnabled] = useState(true);
  const [cashSafetyThreshold, setCashSafetyThreshold] = useState(40000); // $40,000 threshold
  const [isAutoRecruiting, setIsAutoRecruiting] = useState(false);
  const [autoRecruitFeedback, setAutoRecruitFeedback] = useState<{ message: string; type: 'success' | 'warning' | 'error' } | null>(null);

  const handleTriggerAutoRecruit = async () => {
    if (!onAutoRecruit) return;
    setIsAutoRecruiting(true);
    setAutoRecruitFeedback(null);

    const res = await onAutoRecruit(cashSafetyThreshold * 100);
    setIsAutoRecruiting(false);
    if (res.success) {
      setAutoRecruitFeedback({
        message: 'Recruiter AI đã tự động tuyển dụng thành công 1 nhân sự chuyên môn mới!',
        type: 'success',
      });
    } else {
      setAutoRecruitFeedback({
        message: res.reason || 'Không thể tự động tuyển dụng lúc này.',
        type: 'warning',
      });
    }
  };

  const handleStartInterview = async (candidate: CandidateProfile) => {
    setInterviewingCandidate(candidate);
    setIsInterviewing(true);
    setInterviewResult(null);

    if (onInterviewCandidate) {
      const res = await onInterviewCandidate(candidate.id);
      if (res && res.evaluation) {
        setInterviewResult(res.evaluation);
      }
    } else {
      setTimeout(() => {
        setInterviewResult({
          technicalScore: 96,
          portfolioScore: 94,
          cultureScore: 95,
          governorVerdict: 'Approve Recommendation',
          summary: `${candidate.name} sở hữu kỹ năng chuyên sâu cấp độ Senior/Lead với kinh nghiệm ${candidate.yearsExperience} năm. Đạt chuẩn gia nhập bộ máy tự trị.`,
          negotiatedSalaryMinor: candidate.expectedSalaryMinor,
        });
      }, 800);
    }
    setIsInterviewing(false);
  };

  const handleConfirmHireCandidate = async (candidate: CandidateProfile) => {
    if (onHireCandidate) {
      const res = await onHireCandidate(candidate.id);
      if (res && res.success) {
        setHireSuccessNotice(`Đã tuyển dụng thành công ${candidate.name} (${candidate.role})!`);
        setTimeout(() => setHireSuccessNotice(null), 4000);
        setInterviewingCandidate(null);
      }
    }
  };

  const handleHireSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !role.trim()) return;
    setIsSubmitting(true);
    setHireError(null);

    const res = await onHireAgent({
      name: name.trim(),
      role: role.trim(),
      department,
      description: description.trim() || 'Chuyên viên xử lý công việc tự động',
      salary_minor: salaryUsd * 100,
    });

    setIsSubmitting(false);
    if (res.success) {
      setShowHireModal(false);
      setName('');
      setRole('');
      setDescription('');
    } else {
      setHireError(res.reason || 'Không thể tuyển dụng.');
    }
  };

  const handleSendInstruction = (e: React.FormEvent) => {
    e.preventDefault();
    if (!quickInstruction.trim()) return;
    setInstructionSent(true);
    setTimeout(() => {
      setInstructionSent(false);
      setQuickInstruction('');
    }, 3000);
  };

  const openTaskHistoryModal = async (agent: CustomAgent) => {
    setSelectedAgentForHistory(agent);
    setIsLoadingHistory(true);
    setTaskHistory([]);

    try {
      const res = await fetch(`/api/agent-tasks/${encodeURIComponent(agent.name || agent.id)}`);
      if (res.ok) {
        const data = await res.json();
        setTaskHistory(data.tasks || []);
      }
    } catch (err) {
      console.warn('Task history load note:', err);
    } finally {
      setIsLoadingHistory(false);
    }
  };

  const filteredAgents = agents.filter((a) => {
    if (filter === 'All') return true;
    return a.department === filter;
  });

  const getDepartmentBadge = (dept: string) => {
    switch (dept) {
      case 'Leadership':
        return 'bg-purple-500/10 text-purple-400 border-purple-500/20';
      case 'Growth':
        return 'bg-pink-500/10 text-pink-400 border-pink-500/20';
      default:
        return 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20';
    }
  };

  // Calculate Salary/Efficiency Ratio for an agent
  const getEfficiencyScore = (agent: CustomAgent) => {
    const cost = agent.salary_minor > 0 ? agent.salary_minor / 100 : 350;
    // Expected task benchmark: ~1 task per $15 of monthly compensation/infrastructure
    const expectedTasks = Math.max(8, Math.round(cost / 15));
    const score = Math.min(100, Math.max(15, Math.round((agent.tasksCompleted / expectedTasks) * 100)));
    return {
      score,
      isHigh: score >= 75,
      isMedium: score >= 45 && score < 75,
      isLow: score < 45,
      label: score >= 75 ? 'Rất xứng đáng' : score >= 45 ? 'Đạt kỳ vọng' : 'Cần tối ưu',
      cost,
    };
  };

  // Prepare comprehensive data for Departmental Performance Report
  const agentReportData = agents
    .filter((a) => reportDeptFilter === 'All' || a.department === reportDeptFilter)
    .map((a) => {
      const salaryUsd = a.salary_minor > 0 ? Math.round(a.salary_minor / 100) : 350;
      const tasks = a.tasksCompleted;
      const costPerTask = Math.round((salaryUsd / Math.max(1, tasks)) * 10) / 10;
      
      let tier: 'HighPerformer' | 'Balanced' | 'Overpaid' = 'Balanced';
      let label = '⚖️ Đạt Chuẩn (Balanced)';
      let badgeClass = 'bg-cyan-500/15 text-cyan-300 border-cyan-500/30';
      let recommendation = 'Đạt chuẩn tác vụ. Duy trì ổn định mức đãi ngộ hiện thời.';

      if (costPerTask <= 16) {
        tier = 'HighPerformer';
        label = '🌟 High-Performer (Hiệu Suất Cao)';
        badgeClass = 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40 font-bold';
        recommendation = 'Năng suất xuất sắc ($' + costPerTask + '/việc). Khuyến nghị thưởng hạn ngạch & ưu tiên cấp tài nguyên.';
      } else if (costPerTask > 35) {
        tier = 'Overpaid';
        label = '⚠️ Cần Tối Ưu Lương (Overpaid)';
        badgeClass = 'bg-rose-500/20 text-rose-300 border-rose-500/40 font-bold';
        recommendation = 'Chi phí/việc cao ($' + costPerTask + '/việc). Governor khuyến nghị giao thêm tác vụ hoặc điều chỉnh mức lương.';
      }

      return {
        id: a.id,
        name: a.name.replace(' Agent', '').replace(' Specialist', ''),
        fullName: a.name,
        role: a.role,
        dept: a.department,
        tasks,
        cost: salaryUsd,
        costPerTask,
        tier,
        label,
        badgeClass,
        recommendation,
        status: a.status,
      };
    });

  const sortedReportData = [...agentReportData].sort((a, b) => {
    if (reportSort === 'roi') return a.costPerTask - b.costPerTask; // best value first
    if (reportSort === 'overpaid') return b.costPerTask - a.costPerTask; // overpaid first
    if (reportSort === 'tasks') return b.tasks - a.tasks; // most tasks
    if (reportSort === 'cost') return b.cost - a.cost; // highest salary
    return 0;
  });

  // Calculate summary metrics for report
  const topPerformer = [...agentReportData].sort((a, b) => a.costPerTask - b.costPerTask)[0];
  const overpaidAlert = [...agentReportData].sort((a, b) => b.costPerTask - a.costPerTask)[0];
  const totalReportTasks = agentReportData.reduce((sum, a) => sum + a.tasks, 0);
  const totalReportCost = agentReportData.reduce((sum, a) => sum + a.cost, 0);
  const avgCostPerTask = totalReportTasks > 0 ? (totalReportCost / totalReportTasks).toFixed(1) : '0';
  const highPerformerCount = agentReportData.filter((a) => a.tier === 'HighPerformer').length;
  const balancedCount = agentReportData.filter((a) => a.tier === 'Balanced').length;
  const overpaidCount = agentReportData.filter((a) => a.tier === 'Overpaid').length;

  // Calculate department totals
  const deptStats = ['Leadership', 'Growth', 'Ops'].map((deptName) => {
    const deptAgents = agents.filter((a) => a.department === deptName);
    const totalTasks = deptAgents.reduce((sum, a) => sum + a.tasksCompleted, 0);
    const totalCost = deptAgents.reduce((sum, a) => sum + (a.salary_minor > 0 ? a.salary_minor / 100 : 350), 0);
    const avgTasksPerAgent = deptAgents.length > 0 ? Math.round(totalTasks / deptAgents.length) : 0;
    return {
      department: deptName,
      agentCount: deptAgents.length,
      totalTasks,
      totalCost,
      status: avgTasksPerAgent >= 40 ? 'Tối ưu (Hiệu suất cao)' : avgTasksPerAgent >= 25 ? 'Bình thường' : 'Cần giao thêm việc',
    };
  });

  // Export Agent Performance Breakdown Data to CSV
  const exportPerformanceToCSV = () => {
    const headers = [
      'Mã Nhân Sự (Agent ID)',
      'Tên Nhân Sự (Agent Name)',
      'Chức Danh (Role)',
      'Phòng Ban (Department)',
      'Số Tác Vụ Hoàn Thành (Tasks Completed)',
      'Chi Phí Lương Tháng (USD)',
      'Chi Phí / Tác Vụ (USD/Task)',
      'Xếp Hạng Hiệu Suất (Performance Tier)',
      'Khuyến Nghị Governor (Governor Recommendation)',
      'Trạng Thái (Status)',
      'Chu Kỳ Gia Nhập (Hired Cycle)',
    ];

    const rows = sortedReportData.map((a) => {
      return [
        `"${a.id}"`,
        `"${a.fullName.replace(/"/g, '""')}"`,
        `"${a.role.replace(/"/g, '""')}"`,
        `"${a.dept}"`,
        a.tasks,
        a.cost,
        a.costPerTask,
        `"${a.label.replace(/"/g, '""')}"`,
        `"${a.recommendation.replace(/"/g, '""')}"`,
        `"${a.status === 'Active' ? 'Đang hoạt động' : 'Tạm dừng'}"`,
        snapshot.cycle_count,
      ].join(',');
    });

    const csvContent = '\uFEFF' + [headers.join(','), ...rows].join('\r\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);
    link.setAttribute('download', `bao_cao_hieu_suat_nhan_su_ky_${snapshot.cycle_count}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-4 max-w-5xl mx-auto pb-10">
      {/* Top Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
        <div className="flex items-center gap-2">
          <Users className="w-5 h-5 text-indigo-400" />
          <div>
            <h2 className="text-base font-bold text-white">Đội Ngũ Nhân Sự AI ({agents.length} vị trí)</h2>
            <p className="text-xs text-slate-400">Theo dõi tỷ lệ Hiệu Suất / Tiền Lương và lịch sử công việc từng người</p>
          </div>
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          {/* Filter */}
          <div className="flex bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs">
            {(['All', 'Leadership', 'Growth', 'Ops'] as const).map((cat) => (
              <button
                key={cat}
                onClick={() => setFilter(cat)}
                className={`px-2.5 py-1 rounded-md font-semibold transition-all ${
                  filter === cat ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                }`}
              >
                {cat === 'All' ? 'Tất cả' : cat === 'Leadership' ? 'Ban Giám Đốc' : cat === 'Growth' ? 'Kinh Doanh' : 'Vận Hành'}
              </button>
            ))}
          </div>

          {/* Training Button */}
          {onOpenTraining && (
            <button
              onClick={() => onOpenTraining()}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white font-bold text-xs shadow-md transition-all shrink-0 active:scale-95"
            >
              <GraduationCap className="w-3.5 h-3.5 text-yellow-300" /> Đào Tạo Kỹ Năng
            </button>
          )}

          {/* Competency Matrix Button */}
          <button
            onClick={() => setShowCompetencyMatrix(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 font-bold text-xs border border-indigo-500/40 shadow-sm transition-all shrink-0 active:scale-95"
            title="So sánh năng lực chuẩn hóa đa chiều qua biểu đồ Radar"
          >
            <BrainCircuit className="w-3.5 h-3.5 text-indigo-400" /> Ma Trận Năng Lực
          </button>

          {/* Hire Button */}
          <button
            onClick={() => setShowHireModal(true)}
            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs shadow-md transition-all shrink-0 active:scale-95"
          >
            <Plus className="w-3.5 h-3.5" /> Tuyển Agent Mới
          </button>
        </div>
      </div>

      {/* Top View Switcher */}
      <div className="flex items-center justify-between border-b border-slate-800 pb-3 flex-wrap gap-2">
        <div className="flex bg-slate-900 p-1 rounded-xl border border-slate-800 text-xs">
          <button
            onClick={() => setMainView('roster')}
            className={`flex items-center gap-1.5 px-3.5 py-2 rounded-lg font-bold transition-all ${
              mainView === 'roster'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Users className="w-4 h-4" />
            <span>Đội Ngũ Hiện Tại ({agents.length} Nhân Sự)</span>
          </button>
          <button
            onClick={() => setMainView('talent_market')}
            className={`flex items-center gap-1.5 px-3.5 py-2 rounded-lg font-bold transition-all ${
              mainView === 'talent_market'
                ? 'bg-gradient-to-r from-purple-600 to-pink-600 text-white shadow-md'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Briefcase className="w-4 h-4 text-pink-300" />
            <span>Sàn Tuyển Dụng Nhân Tài ({candidates.length} Chuyên Gia Senior)</span>
          </button>
        </div>

        {hireSuccessNotice && (
          <div className="px-3 py-1.5 rounded-lg bg-emerald-950/80 border border-emerald-500/40 text-emerald-300 text-xs flex items-center gap-1.5 animate-fadeIn">
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            {hireSuccessNotice}
          </div>
        )}
      </div>

      {/* AUTO-RECRUIT CONTROLLER CARD */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 shadow-sm space-y-3">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-slate-800 pb-3">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 shrink-0">
              <Bot className="w-4 h-4 text-cyan-400" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-white text-xs block">
                  Cơ Chế Tự Động Tuyển Dụng AI (Auto-Recruit Engine)
                </span>
                <span className={`px-2 py-0.2 rounded-full font-mono text-[10px] font-bold border ${
                  autoRecruitEnabled 
                    ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30' 
                    : 'bg-slate-800 text-slate-400 border-slate-700'
                }`}>
                  {autoRecruitEnabled ? '🟢 ĐANG BẬT' : '⚪ ĐÃ TẮT'}
                </span>
              </div>
              <span className="text-[10px] text-slate-400">
                Ủy quyền cho Recruiter Agent tự động tuyển dụng vai trò chuyên môn khi Hiring Need &gt; 0 và Kho Bạc đạt chuẩn an toàn
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {/* Toggle switch */}
            <button
              onClick={() => setAutoRecruitEnabled(!autoRecruitEnabled)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold transition-all border ${
                autoRecruitEnabled
                  ? 'bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-300 border-emerald-500/40'
                  : 'bg-slate-800 hover:bg-slate-700 text-slate-300 border-slate-700'
              }`}
            >
              {autoRecruitEnabled ? (
                <ToggleRight className="w-4 h-4 text-emerald-400" />
              ) : (
                <ToggleLeft className="w-4 h-4 text-slate-400" />
              )}
              <span>{autoRecruitEnabled ? 'Tự Động Tuyển: BẬT' : 'Tự Động Tuyển: TẮT'}</span>
            </button>

            {/* Quick Trigger Button */}
            {onAutoRecruit && (
              <button
                onClick={handleTriggerAutoRecruit}
                disabled={isAutoRecruiting || !autoRecruitEnabled}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold transition-all border ${
                  !autoRecruitEnabled
                    ? 'bg-slate-950 text-slate-600 border-slate-800 cursor-not-allowed'
                    : isAutoRecruiting
                    ? 'bg-indigo-700 text-white cursor-wait'
                    : 'bg-indigo-600 hover:bg-indigo-500 text-white border-indigo-500 shadow-md active:scale-95'
                }`}
                title="Yêu cầu Recruiter quét và thực hiện tự động tuyển dụng ngay nếu đủ điều kiện"
              >
                <Sparkles className={`w-3.5 h-3.5 ${isAutoRecruiting ? 'animate-spin' : 'text-yellow-300'}`} />
                <span>{isAutoRecruiting ? 'Đang Quét Tuyển...' : 'Quét & Tuyển Ngay'}</span>
              </button>
            )}
          </div>
        </div>

        {/* Status conditions & Safety Threshold */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-2.5 text-xs">
          {/* Condition 1: Hiring Need */}
          <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
            <div>
              <span className="text-[10px] text-slate-400 block">Điều Kiện 1: Nhu Cầu Nhân Sự</span>
              <span className="font-bold text-white text-xs font-mono">
                Hiring Need = {snapshot.hiring_need} vị trí
              </span>
            </div>
            <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full border ${
              snapshot.hiring_need > 0 
                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30' 
                : 'bg-slate-800 text-slate-400 border-slate-700'
            }`}>
              {snapshot.hiring_need > 0 ? '✓ Đạt chuẩn' : '⚪ Đủ quân số'}
            </span>
          </div>

          {/* Condition 2: Cash Safety Threshold */}
          <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
            <div>
              <span className="text-[10px] text-slate-400 block">Điều Kiện 2: Ngưỡng An Toàn Kho Bạc</span>
              <div className="flex items-center gap-1.5 mt-0.5">
                <span className="font-bold text-white text-xs font-mono">
                  ${(snapshot.cash_minor / 100).toLocaleString()}
                </span>
                <span className="text-[10px] text-slate-500">&ge;</span>
                <select
                  value={cashSafetyThreshold}
                  onChange={(e) => setCashSafetyThreshold(Number(e.target.value))}
                  className="bg-slate-900 border border-slate-700 rounded px-1.5 py-0.2 text-[10px] text-indigo-300 font-mono focus:outline-none"
                >
                  <option value={30000}>$30,000</option>
                  <option value={35000}>$35,000</option>
                  <option value={40000}>$40,000 (Khuyên dùng)</option>
                  <option value={45000}>$45,000</option>
                </select>
              </div>
            </div>
            <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full border ${
              snapshot.cash_minor >= cashSafetyThreshold * 100 
                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30' 
                : 'bg-rose-500/10 text-rose-400 border-rose-500/30'
            }`}>
              {snapshot.cash_minor >= cashSafetyThreshold * 100 ? '✓ Đạt chuẩn' : '✕ Chưa đủ'}
            </span>
          </div>

          {/* Recruiter Autonomous Status */}
          <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
            <div>
              <span className="text-[10px] text-slate-400 block">Trạng Thái Recruiter Agent</span>
              <span className="font-bold text-xs text-indigo-300 truncate block">
                {autoRecruitEnabled && snapshot.hiring_need > 0 && snapshot.cash_minor >= cashSafetyThreshold * 100
                  ? 'Sẵn sàng bổ sung chuyên môn'
                  : !autoRecruitEnabled
                  ? 'Chế độ tự động đang tạm tắt'
                  : 'Đang theo dõi định mức'}
              </span>
            </div>
            <ShieldCheck className={`w-4 h-4 shrink-0 ${
              autoRecruitEnabled && snapshot.hiring_need > 0 && snapshot.cash_minor >= cashSafetyThreshold * 100
                ? 'text-emerald-400'
                : 'text-slate-500'
            }`} />
          </div>
        </div>

        {/* Feedback message */}
        {autoRecruitFeedback && (
          <div className={`p-2.5 rounded-lg border text-xs flex items-center gap-2 ${
            autoRecruitFeedback.type === 'success'
              ? 'bg-emerald-950/40 border-emerald-500/40 text-emerald-300'
              : 'bg-amber-950/40 border-amber-500/40 text-amber-300'
          }`}>
            <CheckCircle2 className="w-4 h-4 shrink-0" />
            <span>{autoRecruitFeedback.message}</span>
          </div>
        )}
      </div>

      {mainView === 'talent_market' ? (
        /* TALENT SCOUTING & RECRUITMENT MARKET */
        <div className="space-y-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-2 shadow-sm">
            <div className="flex items-center justify-between flex-wrap gap-2">
              <h3 className="text-base font-bold text-white flex items-center gap-2">
                <Briefcase className="w-5 h-5 text-pink-400" />
                Sàn Tuyển Chọn Chuyên Gia Cấp Cao (Senior Talent Pool)
              </h3>
              <span className="text-xs font-mono text-emerald-300 bg-emerald-500/20 px-2.5 py-1 rounded-full border border-emerald-500/30 flex items-center gap-1.5">
                <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
                Được Thẩm Định Bởi Governor AI
              </span>
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              Các chuyên gia AI độc lập cấp Senior &amp; Lead sẵn sàng bổ sung năng lực chuyên sâu vào bộ máy doanh nghiệp. 
              Bạn có thể kích hoạt quy trình Phỏng Vấn AI 3 Vòng (Technical, Portfolio, ROI) trước khi quyết định ký hợp đồng.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {candidates.map((cand) => {
              const isHired = agents.some((a) => a.name === cand.name || a.id.includes(cand.id));
              return (
                <div
                  key={cand.id}
                  className={`bg-slate-900 border rounded-xl p-5 space-y-4 shadow-md transition-all ${
                    isHired
                      ? 'border-emerald-500/40 bg-slate-900/60 opacity-80'
                      : 'border-slate-800 hover:border-indigo-500/50'
                  }`}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex items-center gap-3">
                      <div className="w-12 h-12 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center text-2xl shadow-inner">
                        {cand.avatar}
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <h4 className="font-bold text-white text-sm">{cand.name}</h4>
                          <span className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold border ${
                            cand.level === 'Principal Expert'
                              ? 'bg-purple-500/20 text-purple-300 border-purple-500/30'
                              : cand.level === 'Director'
                              ? 'bg-amber-500/20 text-amber-300 border-amber-500/30'
                              : 'bg-indigo-500/20 text-indigo-300 border-indigo-500/30'
                          }`}>
                            {cand.level}
                          </span>
                        </div>
                        <span className="text-xs text-slate-400 block mt-0.5">{cand.role}</span>
                      </div>
                    </div>

                    <div className="text-right">
                      <span className="text-[10px] uppercase font-mono text-slate-500 block">Lương Kỳ Vọng</span>
                      <span className="text-emerald-400 font-mono font-bold text-sm">
                        ${(cand.expectedSalaryMinor / 100).toLocaleString()}/th
                      </span>
                    </div>
                  </div>

                  <p className="text-xs text-slate-300 leading-relaxed bg-slate-950/60 p-3 rounded-lg border border-slate-800/80">
                    {cand.bio}
                  </p>

                  {/* Skills Grid */}
                  <div className="space-y-1.5">
                    <span className="text-[10px] uppercase font-mono text-slate-500 block">Kỹ Năng &amp; Điểm Chuyên Môn:</span>
                    <div className="grid grid-cols-2 gap-2">
                      {cand.skills.map((skill, idx) => (
                        <div key={idx} className="p-2 rounded bg-slate-950 border border-slate-800/80 text-[11px] flex items-center justify-between">
                          <span className="text-slate-400 truncate pr-1">{skill.name}</span>
                          <span className="font-mono font-bold text-cyan-400">{skill.score}/100</span>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Track Record & ROI */}
                  <div className="p-2.5 rounded-lg bg-indigo-950/20 border border-indigo-500/30 flex items-center justify-between text-xs">
                    <div className="flex items-center gap-1.5 text-indigo-300">
                      <Star className="w-3.5 h-3.5 text-yellow-400 fill-yellow-400" />
                      <span>Kinh nghiệm: <strong>{cand.yearsExperience} năm</strong></span>
                    </div>
                    <div className="text-emerald-400 font-mono font-bold">
                      Dự phóng ROI: +{(cand.roiProjectionBps / 100).toFixed(1)}%
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="flex items-center gap-2 pt-2 border-t border-slate-800">
                    {isHired ? (
                      <div className="w-full py-2 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs font-bold flex items-center justify-center gap-1.5">
                        <CheckCircle2 className="w-4 h-4" /> Đã Gia Nhập Công Ty
                      </div>
                    ) : (
                      <>
                        <button
                          onClick={() => handleStartInterview(cand)}
                          className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold transition-all border border-slate-700"
                        >
                          <FileCheck2 className="w-3.5 h-3.5 text-cyan-400" />
                          <span>Phỏng Vấn AI</span>
                        </button>
                        <button
                          onClick={() => handleConfirmHireCandidate(cand)}
                          className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white text-xs font-bold transition-all shadow-md active:scale-95"
                        >
                          <UserCheck className="w-3.5 h-3.5" />
                          <span>Tuyển Ngay</span>
                        </button>
                      </>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ) : (
        /* ROSTER VIEW */
        <>
          {/* Top Header */}
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
            <div className="flex items-center gap-2">
              <Users className="w-5 h-5 text-indigo-400" />
              <div>
                <h2 className="text-base font-bold text-white">Đội Ngũ Nhân Sự AI ({agents.length} vị trí)</h2>
                <p className="text-xs text-slate-400">Theo dõi tỷ lệ Hiệu Suất / Tiền Lương và lịch sử công việc từng người</p>
              </div>
            </div>

            <div className="flex items-center gap-2 w-full sm:w-auto">
              {/* Filter */}
              <div className="flex bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs">
                {(['All', 'Leadership', 'Growth', 'Ops'] as const).map((cat) => (
                  <button
                    key={cat}
                    onClick={() => setFilter(cat)}
                    className={`px-2.5 py-1 rounded-md font-semibold transition-all ${
                      filter === cat ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                    }`}
                  >
                    {cat === 'All' ? 'Tất cả' : cat === 'Leadership' ? 'Ban Giám Đốc' : cat === 'Growth' ? 'Kinh Doanh' : 'Vận Hành'}
                  </button>
                ))}
              </div>

              {/* Training Button */}
              {onOpenTraining && (
                <button
                  onClick={() => onOpenTraining()}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white font-bold text-xs shadow-md transition-all shrink-0 active:scale-95"
                >
                  <GraduationCap className="w-3.5 h-3.5 text-yellow-300" /> Đào Tạo Kỹ Năng
                </button>
              )}

              {/* Hire Button */}
              <button
                onClick={() => setShowHireModal(true)}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs shadow-md transition-all shrink-0"
              >
                <Plus className="w-3.5 h-3.5" /> Tuyển Thêm AI
              </button>
            </div>
          </div>
        </>
      )}

      {/* AI INTERVIEW SIMULATOR MODAL */}
      {interviewingCandidate && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-fadeIn">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl relative">
            <button
              onClick={() => setInterviewingCandidate(null)}
              className="absolute top-4 right-4 text-slate-400 hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>

            <div className="flex items-center gap-3">
              <div className="w-12 h-12 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center text-2xl">
                {interviewingCandidate.avatar}
              </div>
              <div>
                <h3 className="text-base font-bold text-white">Hội Đồng Đánh Giá: {interviewingCandidate.name}</h3>
                <span className="text-xs text-slate-400 font-mono">{interviewingCandidate.role} • {interviewingCandidate.yearsExperience} năm kinh nghiệm</span>
              </div>
            </div>

            {isInterviewing ? (
              <div className="py-8 text-center space-y-3">
                <RotateCw className="w-8 h-8 text-indigo-400 animate-spin mx-auto" />
                <p className="text-xs text-slate-300">Governor AI và Recruiter AI đang tiến hành thẩm định chuyên môn 3 vòng...</p>
              </div>
            ) : interviewResult ? (
              <div className="space-y-4 text-xs">
                <div className="grid grid-cols-3 gap-2">
                  <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-center">
                    <span className="text-[10px] text-slate-500 block">Kỹ Thuật</span>
                    <span className="text-base font-bold text-cyan-400 mt-1 block">{interviewResult.technicalScore}/100</span>
                  </div>
                  <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-center">
                    <span className="text-[10px] text-slate-500 block">Portfolio</span>
                    <span className="text-base font-bold text-purple-400 mt-1 block">{interviewResult.portfolioScore}/100</span>
                  </div>
                  <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 text-center">
                    <span className="text-[10px] text-slate-500 block">Văn Hóa/ROI</span>
                    <span className="text-base font-bold text-emerald-400 mt-1 block">{interviewResult.cultureScore}/100</span>
                  </div>
                </div>

                <div className="p-3.5 rounded-lg bg-indigo-950/30 border border-indigo-500/30 text-indigo-200 leading-relaxed">
                  <strong className="text-indigo-400 block mb-1">Đánh Giá Từ Governor AI:</strong>
                  {interviewResult.summary}
                </div>

                <div className="flex items-center justify-between p-3 rounded-lg bg-slate-950 border border-slate-800 font-mono">
                  <span className="text-slate-400">Lương Chốt Dự Kiến:</span>
                  <span className="text-emerald-400 font-bold">${(interviewResult.negotiatedSalaryMinor / 100).toLocaleString()}/th</span>
                </div>

                <div className="flex items-center justify-end gap-2 pt-2">
                  <button
                    onClick={() => setInterviewingCandidate(null)}
                    className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs"
                  >
                    Đóng
                  </button>
                  <button
                    onClick={() => handleConfirmHireCandidate(interviewingCandidate)}
                    className="px-5 py-2 rounded-lg bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white font-bold text-xs shadow-md active:scale-95"
                  >
                    Ký Hợp Đồng &amp; Tuyển Dụng Ngay
                  </button>
                </div>
              </div>
            ) : null}
          </div>
        </div>
      )}

      {/* DEPARTMENTAL PERFORMANCE REPORT (TASKS COMPLETED VS SALARY COST: HIGH-PERFORMERS VS OVERPAID) */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3.5 shadow-sm">
        {/* Report Header */}
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3 border-b border-slate-800 pb-3">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 shrink-0">
              <BarChart3 className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-white text-xs block">
                  Báo Cáo Hiệu Suất Phòng Ban &amp; Nhân Sự (Departmental Performance Report)
                </span>
                <span className="px-2 py-0.2 rounded-full font-mono text-[10px] bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 font-bold">
                  {agentReportData.length} Nhân Sự
                </span>
              </div>
              <span className="text-[10px] text-slate-400">
                So sánh Số Tác Vụ Hoàn Thành (Tasks) vs. Chi Phí Lương (Salary Cost) để đánh giá High-Performers vs. Overpaid Agents
              </span>
            </div>
          </div>

          {/* Controls: Department Filter, Sorting, Export */}
          <div className="flex flex-wrap items-center gap-2 text-[11px]">
            {/* Filter by Department */}
            <div className="flex bg-slate-950 p-0.5 rounded-lg border border-slate-800">
              {(['All', 'Leadership', 'Growth', 'Ops'] as const).map((dept) => (
                <button
                  key={dept}
                  onClick={() => setReportDeptFilter(dept)}
                  className={`px-2 py-0.5 rounded font-semibold transition-all ${
                    reportDeptFilter === dept ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                  }`}
                >
                  {dept === 'All' ? 'Tất cả' : dept === 'Leadership' ? 'Lãnh Đạo' : dept === 'Growth' ? 'Kinh Doanh' : 'Vận Hành'}
                </button>
              ))}
            </div>

            {/* Sort selector */}
            <div className="flex items-center gap-1 bg-slate-950 px-2 py-1 rounded-lg border border-slate-800 text-slate-300">
              <ArrowUpDown className="w-3 h-3 text-slate-400" />
              <select
                value={reportSort}
                onChange={(e) => setReportSort(e.target.value as any)}
                className="bg-transparent text-[11px] font-semibold text-white focus:outline-none cursor-pointer"
              >
                <option value="roi" className="bg-slate-900 text-white">🌟 High-Performer trước</option>
                <option value="overpaid" className="bg-slate-900 text-white">⚠️ Nguy cơ Overpaid</option>
                <option value="tasks" className="bg-slate-900 text-white">⚡ Tác vụ nhiều nhất</option>
                <option value="cost" className="bg-slate-900 text-white">💰 Lương cao nhất</option>
              </select>
            </div>

            <button
              onClick={exportPerformanceToCSV}
              className="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-400 border border-emerald-500/30 text-[11px] font-semibold transition-all active:scale-95 shadow-sm"
              title="Xuất file CSV báo cáo hiệu suất nhân sự để phân tích trên Excel / Google Sheets"
            >
              <Download className="w-3 h-3" />
              <span>Xuất CSV</span>
            </button>
          </div>
        </div>

        {/* 4 Summary Highlight Cards */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
          {/* 1. Top Performer */}
          <div className="p-2.5 rounded-lg bg-emerald-950/20 border border-emerald-500/30">
            <div className="flex items-center justify-between text-emerald-400 text-[10px] font-bold">
              <span className="flex items-center gap-1">
                <Award className="w-3 h-3" /> Ngôi Sao Hiệu Suất
              </span>
              <span>🌟 High-Performer</span>
            </div>
            <div className="mt-1 text-xs font-bold text-white truncate">
              {topPerformer ? topPerformer.fullName : 'N/A'}
            </div>
            <div className="text-[10px] text-emerald-400/90 font-mono mt-0.5">
              {topPerformer ? `${topPerformer.tasks} việc • $${topPerformer.costPerTask}/việc` : ''}
            </div>
          </div>

          {/* 2. Overpaid Alert */}
          <div className="p-2.5 rounded-lg bg-rose-950/20 border border-rose-500/30">
            <div className="flex items-center justify-between text-rose-400 text-[10px] font-bold">
              <span className="flex items-center gap-1">
                <AlertTriangle className="w-3 h-3" /> Cần Tối Ưu Lương
              </span>
              <span>⚠️ Overpaid</span>
            </div>
            <div className="mt-1 text-xs font-bold text-white truncate">
              {overpaidAlert && overpaidAlert.tier === 'Overpaid' ? overpaidAlert.fullName : 'Không có rủi ro cao'}
            </div>
            <div className="text-[10px] text-rose-300 font-mono mt-0.5">
              {overpaidAlert && overpaidAlert.tier === 'Overpaid'
                ? `${overpaidAlert.tasks} việc • $${overpaidAlert.costPerTask}/việc`
                : 'Mọi nhân sự đều đạt chuẩn'}
            </div>
          </div>

          {/* 3. Avg Cost Per Task */}
          <div className="p-2.5 rounded-lg bg-indigo-950/20 border border-indigo-500/30">
            <div className="text-slate-400 text-[10px]">
              Chi Phí TB / Tác Vụ
            </div>
            <div className="mt-1 text-sm font-black text-indigo-300 font-mono">
              ${avgCostPerTask} <span className="text-[10px] font-normal text-slate-400">/ việc</span>
            </div>
            <div className="text-[10px] text-slate-400 mt-0.5">
              Tổng {totalReportTasks} tác vụ hoàn tất
            </div>
          </div>

          {/* 4. Healthy Workforce Ratio */}
          <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800">
            <div className="text-slate-400 text-[10px]">
              Tỷ Lệ Đạt Chuẩn &amp; Vượt Trội
            </div>
            <div className="mt-1 text-sm font-black text-white font-mono">
              {agentReportData.length > 0
                ? `${Math.round(((highPerformerCount + balancedCount) / agentReportData.length) * 100)}%`
                : '100%'}
            </div>
            <div className="text-[10px] text-emerald-400 mt-0.5">
              {highPerformerCount} High-Performer • {overpaidCount} Overpaid
            </div>
          </div>
        </div>

        {/* Bar Chart Container */}
        <div className="h-64 w-full pt-1">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={sortedReportData} margin={{ top: 10, right: 10, left: -15, bottom: 25 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
              <XAxis
                dataKey="name"
                stroke="#64748b"
                tick={{ fill: '#94a3b8', fontSize: 10 }}
                interval={0}
                angle={-20}
                textAnchor="end"
              />
              <YAxis
                stroke="#64748b"
                tick={{ fill: '#94a3b8', fontSize: 10 }}
                tickLine={false}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#0f172a',
                  borderColor: '#334155',
                  borderRadius: '0.75rem',
                  fontSize: '11px',
                  boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.5)',
                }}
                formatter={(val: any, name: any, item: any) => {
                  const agent = item.payload;
                  if (name === 'tasks') return [`${val} tác vụ`, 'Khối Lượng Đã Làm (Tasks)'];
                  if (name === 'cost') return [`$${Number(val || 0).toLocaleString()}/th`, 'Chi Phí Lương (Salary)'];
                  return [val, name];
                }}
                labelFormatter={(name, items) => {
                  const agent = items[0]?.payload;
                  return agent ? `${agent.fullName} (${agent.role} • ${agent.dept}) - ${agent.label}` : name;
                }}
                labelStyle={{ color: '#e2e8f0', fontWeight: 'bold' }}
              />
              <Legend
                wrapperStyle={{ fontSize: '11px', paddingTop: '4px' }}
                formatter={(val) =>
                  val === 'tasks' ? 'Số Việc Hoàn Thành (Tasks Completed)' : 'Chi Phí Lương Tháng ($ Salary Cost)'
                }
              />
              <Bar dataKey="tasks" fill="#6366f1" radius={[4, 4, 0, 0]} name="tasks" />
              <Bar dataKey="cost" fill="#f59e0b" radius={[4, 4, 0, 0]} name="cost" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Detailed Breakdown List / Table */}
        <div className="border-t border-slate-800 pt-3">
          <div className="flex items-center justify-between pb-2">
            <span className="text-xs font-bold text-white flex items-center gap-1.5">
              <span>Bảng Đánh Giá Chi Tiết Từng Nhân Sự: High-Performers vs. Overpaid</span>
            </span>
            <button
              onClick={() => setShowDetailedReportList(!showDetailedReportList)}
              className="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 font-semibold"
            >
              <span>{showDetailedReportList ? 'Thu gọn' : 'Mở rộng bảng chi tiết'}</span>
              {showDetailedReportList ? <ChevronUp className="w-3.5 h-3.5" /> : <ChevronDown className="w-3.5 h-3.5" />}
            </button>
          </div>

          {showDetailedReportList && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2 pt-1">
              {sortedReportData.map((agent) => (
                <div
                  key={agent.id}
                  className={`p-3 rounded-xl border text-xs transition-all flex flex-col justify-between ${
                    agent.tier === 'HighPerformer'
                      ? 'bg-emerald-950/10 border-emerald-500/30'
                      : agent.tier === 'Overpaid'
                      ? 'bg-rose-950/15 border-rose-500/40'
                      : 'bg-slate-950 border-slate-800'
                  }`}
                >
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <div className="flex items-center gap-2">
                        <strong className="text-white text-xs">{agent.fullName}</strong>
                        <span className={`px-1.5 py-0.2 rounded text-[10px] font-mono border ${agent.badgeClass}`}>
                          {agent.label}
                        </span>
                      </div>
                      <div className="text-[11px] text-slate-400 mt-0.5">
                        {agent.role} • <span className="text-slate-300">{agent.dept}</span>
                      </div>
                    </div>

                    <div className="text-right shrink-0">
                      <div className="text-xs font-bold font-mono text-white">
                        ${agent.costPerTask}<span className="text-[10px] font-normal text-slate-400">/việc</span>
                      </div>
                      <div className="text-[10px] text-slate-400 font-mono">
                        {agent.tasks} việc • ${agent.cost}/th
                      </div>
                    </div>
                  </div>

                  <div className="mt-2 text-[11px] text-slate-300 bg-slate-900/60 p-2 rounded-lg border border-slate-800/80 flex items-start justify-between gap-1.5">
                    <div className="flex items-start gap-1.5 flex-1">
                      <span className="text-indigo-400 font-bold shrink-0">Governor:</span>
                      <span>{agent.recommendation}</span>
                    </div>
                    {onOpenTraining && (
                      <button
                        onClick={() => onOpenTraining(agent.id)}
                        className="px-2 py-1 rounded bg-indigo-600/30 hover:bg-indigo-600 text-indigo-300 hover:text-white font-bold text-[10px] shrink-0 border border-indigo-500/40 transition-all flex items-center gap-1 active:scale-95"
                        title="Đào tạo kỹ năng để tăng hệ số công việc"
                      >
                        <Zap className="w-2.5 h-2.5 text-yellow-300" />
                        <span>Upskill</span>
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Department Health Highlights */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-2.5 pt-2 border-t border-slate-800/80">
          {deptStats.map((d) => (
            <div
              key={d.department}
              className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 text-xs flex items-center justify-between"
            >
              <div>
                <span className="font-bold text-white block">
                  {d.department === 'Leadership' ? 'Ban Giám Đốc' : d.department === 'Growth' ? 'Khối Kinh Doanh' : 'Khối Vận Hành'}
                </span>
                <span className="text-[11px] text-slate-400">
                  {d.totalTasks} việc • ${(d.totalCost).toLocaleString()}/tháng
                </span>
              </div>
              <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${
                d.status.includes('Tối ưu')
                  ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                  : d.status.includes('Bình thường')
                  ? 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20'
                  : 'bg-amber-500/10 text-amber-300 border border-amber-500/20'
              }`}>
                {d.status}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Quick Instruction Bar */}
      <form onSubmit={handleSendInstruction} className="flex gap-2">
        <input
          type="text"
          value={quickInstruction}
          onChange={(e) => setQuickInstruction(e.target.value)}
          placeholder="Giao việc nhanh cho toàn bộ nhân sự (Ví dụ: 'Tập trung đẩy mạnh video công nghệ AI')..."
          className="flex-1 bg-slate-900 border border-slate-800 rounded-lg px-3.5 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
        />
        <button
          type="submit"
          className="flex items-center gap-1 px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs shrink-0 transition-all"
        >
          <Send className="w-3.5 h-3.5" /> Gửi Lệnh
        </button>
      </form>

      {instructionSent && (
        <div className="text-xs text-emerald-400 flex items-center gap-1.5">
          <CheckCircle2 className="w-3.5 h-3.5" /> Lệnh đã chuyển đến CEO và các phòng ban!
        </div>
      )}

      {/* Agents Grid with Salary/Efficiency Ratio Progress Bar and Task History Button */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        {filteredAgents.map((agent) => {
          const efficiency = getEfficiencyScore(agent);

          return (
            <div
              key={agent.id}
              className="bg-slate-900 border border-slate-800 rounded-xl p-3.5 hover:border-slate-700 transition-all flex flex-col justify-between space-y-3 shadow-sm"
            >
              <div>
                <div className="flex items-center justify-between">
                  <span className={`text-[10px] font-mono px-2 py-0.5 rounded-full border ${getDepartmentBadge(agent.department)}`}>
                    {agent.department}
                  </span>

                  <button
                    onClick={() => onToggleStatus(agent.id)}
                    className={`flex items-center gap-1 text-[11px] font-mono px-2 py-0.5 rounded-md transition-all ${
                      agent.status === 'Active'
                        ? 'text-emerald-400 hover:bg-emerald-500/10'
                        : 'text-slate-500 hover:bg-slate-800'
                    }`}
                    title="Bật / Tạm dừng hoạt động"
                  >
                    <Power className="w-3 h-3" />
                    <span>{agent.status === 'Active' ? 'Đang chạy' : 'Tạm dừng'}</span>
                  </button>
                </div>

                <div className="mt-2">
                  <h3 className="font-bold text-white text-sm">{agent.name}</h3>
                  <p className="text-xs text-indigo-300 font-medium">{agent.role}</p>
                  <p className="text-[11px] text-slate-400 mt-1 line-clamp-2">{agent.description}</p>
                </div>
              </div>

              {/* SALARY / EFFICIENCY PROGRESS BAR */}
              <div className="bg-slate-950 p-2.5 rounded-lg border border-slate-800/80 space-y-1.5">
                <div className="flex items-center justify-between text-[10px]">
                  <span className="text-slate-400 font-medium">Tỷ Lệ Hiệu Suất / Lương</span>
                  <span
                    className={`font-bold font-mono ${
                      efficiency.isHigh
                        ? 'text-emerald-400'
                        : efficiency.isMedium
                        ? 'text-indigo-400'
                        : 'text-amber-400'
                    }`}
                  >
                    {efficiency.score}% ({efficiency.label})
                  </span>
                </div>

                {/* Progress Track */}
                <div className="w-full bg-slate-800 h-2 rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all duration-500 ${
                      efficiency.isHigh
                        ? 'bg-emerald-500'
                        : efficiency.isMedium
                        ? 'bg-indigo-500'
                        : 'bg-amber-500'
                    }`}
                    style={{ width: `${efficiency.score}%` }}
                  />
                </div>

                <div className="flex items-center justify-between text-[10px] text-slate-500 font-mono">
                  <span>{agent.tasksCompleted} việc hoàn thành</span>
                  <span>{agent.salary_minor > 0 ? `$${(agent.salary_minor / 100).toLocaleString()}/th` : 'Core Agent'}</span>
                </div>
              </div>

              {/* Task History & Training Trigger Buttons */}
              <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between gap-2">
                <button
                  onClick={() => openTaskHistoryModal(agent)}
                  className="flex-1 flex items-center justify-center gap-1.5 py-1.5 px-2.5 rounded-lg bg-slate-800/80 hover:bg-slate-800 text-slate-200 hover:text-white font-medium text-xs transition-all active:scale-95 border border-slate-700/60"
                >
                  <History className="w-3.5 h-3.5 text-indigo-400" />
                  <span>10 Tác Vụ</span>
                </button>

                {onOpenTraining && (
                  <button
                    onClick={() => onOpenTraining(agent.id)}
                    className="flex-1 flex items-center justify-center gap-1 py-1.5 px-2.5 rounded-lg bg-gradient-to-r from-purple-600/20 to-indigo-600/20 hover:from-purple-600/40 hover:to-indigo-600/40 text-purple-300 hover:text-white font-bold text-xs transition-all active:scale-95 border border-purple-500/30"
                  >
                    <GraduationCap className="w-3.5 h-3.5 text-yellow-300" />
                    <span>Đào Tạo ({agent.taskMultiplier ? `${agent.taskMultiplier}x` : '1.0x'})</span>
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* TASK HISTORY MODAL (Lists the last 10 completed tasks for a specific agent) */}
      {selectedAgentForHistory && (
        <div className="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-2xl w-full p-5 shadow-2xl space-y-4 max-h-[85vh] flex flex-col">
            {/* Modal Header */}
            <div className="flex items-center justify-between border-b border-slate-800 pb-3 shrink-0">
              <div className="flex items-center gap-2">
                <History className="w-4 h-4 text-indigo-400" />
                <div>
                  <h3 className="font-bold text-white text-sm">
                    Lịch Sử 10 Tác Vụ Gần Nhất: <span className="text-indigo-400">{selectedAgentForHistory.name}</span>
                  </h3>
                  <span className="text-[11px] text-slate-400">{selectedAgentForHistory.role} • {selectedAgentForHistory.tasksCompleted} tác vụ tích lũy</span>
                </div>
              </div>

              <button
                onClick={() => setSelectedAgentForHistory(null)}
                className="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-all"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Modal Content / Task List */}
            <div className="flex-1 overflow-y-auto space-y-2.5 pr-1">
              {isLoadingHistory ? (
                <div className="py-12 text-center text-xs text-slate-400 space-y-2">
                  <div className="w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto" />
                  <p>Đang tải dữ liệu kiểm toán tác vụ...</p>
                </div>
              ) : taskHistory.length > 0 ? (
                taskHistory.map((task, idx) => (
                  <div
                    key={task.id || idx}
                    className="p-3 rounded-lg bg-slate-950 border border-slate-800 hover:border-slate-700 transition-all text-xs space-y-1.5"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-1.5">
                        <span className="w-5 h-5 rounded-full bg-indigo-500/20 text-indigo-300 font-mono text-[10px] font-bold flex items-center justify-center shrink-0">
                          #{idx + 1}
                        </span>
                        <span className="font-bold text-white text-xs">{task.title}</span>
                      </div>

                      <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0">
                        {task.status === 'Completed' ? 'Đã hoàn tất' : task.status}
                      </span>
                    </div>

                    <p className="text-[11px] text-slate-300 pl-6">{task.outcome}</p>

                    <div className="flex items-center justify-between text-[10px] text-slate-500 font-mono pl-6 pt-1 border-t border-slate-900">
                      <span>Kỳ #{task.cycle} • {new Date(task.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                      <span>{task.cost_minor > 0 ? `Chi: $${(task.cost_minor / 100).toFixed(2)}` : 'Không tốn ngân sách'}</span>
                    </div>
                  </div>
                ))
              ) : (
                <div className="py-8 text-center text-xs text-slate-400">
                  Chưa có tác vụ nào được ghi nhận cho nhân sự này.
                </div>
              )}
            </div>

            {/* Modal Footer */}
            <div className="border-t border-slate-800 pt-3 flex items-center justify-between shrink-0">
              <span className="text-[11px] text-slate-500">
                Mọi hành động đều được ký nhận và lưu trong sổ kế toán kép.
              </span>
              <button
                onClick={() => setSelectedAgentForHistory(null)}
                className="px-4 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs transition-all"
              >
                Đóng
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Hire Modal */}
      {showHireModal && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-md w-full p-5 shadow-2xl space-y-4">
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <h3 className="font-bold text-white text-sm flex items-center gap-2">
                <Plus className="w-4 h-4 text-indigo-400" />
                Tuyển Thêm Vị Trí AI Chuyên Môn
              </h3>
              <button
                onClick={() => setShowHireModal(false)}
                className="text-slate-400 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {hireError && (
              <div className="p-2.5 rounded-lg bg-rose-950/40 border border-rose-500/30 text-rose-300 text-xs">
                {hireError}
              </div>
            )}

            <form onSubmit={handleHireSubmit} className="space-y-3 text-xs">
              <div>
                <label className="text-slate-300 block mb-1">Tên Agent</label>
                <input
                  type="text"
                  required
                  placeholder="Ví dụ: TikTok Ads Specialist, Legal Bot, Support AI..."
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="text-slate-300 block mb-1">Chức Danh / Lĩnh Vực Chuyên Môn</label>
                <input
                  type="text"
                  required
                  placeholder="Ví dụ: Chuyên Viên Chạy Quảng Cáo Tiếp Thị..."
                  value={role}
                  onChange={(e) => setRole(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-slate-300 block mb-1">Phòng Ban</label>
                  <select
                    value={department}
                    onChange={(e) => setDepartment(e.target.value as any)}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                  >
                    <option value="Growth">Kinh Doanh (Growth)</option>
                    <option value="Ops">Vận Hành (Ops)</option>
                    <option value="Sales">Bán Hàng (Sales)</option>
                    <option value="Tech">Kỹ Thuật (Tech)</option>
                  </select>
                </div>

                <div>
                  <label className="text-slate-300 block mb-1">Chi Phí Duy Trì ($/tháng)</label>
                  <input
                    type="number"
                    min="100"
                    step="50"
                    value={salaryUsd}
                    onChange={(e) => setSalaryUsd(Number(e.target.value))}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500 font-mono"
                  />
                </div>
              </div>

              <div>
                <label className="text-slate-300 block mb-1">Mô Tả Nhiệm Vụ</label>
                <textarea
                  rows={2}
                  placeholder="Mô tả ngắn gọn việc vị trí này cần làm..."
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-400">
                Governor kiểm toán: Vốn hiện có <strong>${(snapshot.cash_minor / 100).toLocaleString()}</strong> ({snapshot.runway_days} ngày sống).
              </div>

              <div className="flex items-center justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowHireModal(false)}
                  className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs"
                >
                  Hủy
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting}
                  className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs flex items-center gap-1.5 shadow-md"
                >
                  {isSubmitting ? 'Đang duyệt...' : 'Phê Duyệt Tuyển Dụng'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Agent Competency Matrix Modal (Standardized Skills & Radar Comparison) */}
      <AgentCompetencyMatrixModal
        isOpen={showCompetencyMatrix}
        onClose={() => setShowCompetencyMatrix(false)}
        agents={agents}
        onOpenTraining={onOpenTraining}
      />
    </div>
  );
};
