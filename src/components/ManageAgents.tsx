import React, { useState } from 'react';
import { CompanySnapshot, CustomAgent, CandidateProfile } from '../types/company';
import { 
  Users, 
  Plus, 
  CheckCircle2, 
  Power, 
  Briefcase, 
  GraduationCap, 
  ShieldCheck, 
  Star, 
  X,
  Sparkles,
  ArrowRight,
  TrendingUp
} from 'lucide-react';

interface ManageAgentsProps {
  snapshot: CompanySnapshot;
  agents: CustomAgent[];
  candidates?: CandidateProfile[];
  onHireAgent: (data: { name: string; role: string; department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech'; description: string; salary_minor: number }) => Promise<{ success: boolean; reason?: string }>;
  onToggleStatus?: (agentId: string) => void;
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
  onInterviewCandidate,
  onHireCandidate,
}) => {
  const [tab, setTab] = useState<'roster' | 'market'>('roster');
  const [deptFilter, setDeptFilter] = useState<'All' | 'Leadership' | 'Growth' | 'Ops'>('All');
  const [showHireModal, setShowHireModal] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [hireError, setHireError] = useState<string | null>(null);

  // Candidate Interview State
  const [selectedCandidate, setSelectedCandidate] = useState<CandidateProfile | null>(null);
  const [interviewResult, setInterviewResult] = useState<any>(null);
  const [isInterviewing, setIsInterviewing] = useState(false);
  const [hireSuccessNotice, setHireSuccessNotice] = useState<string | null>(null);

  // Form State
  const [name, setName] = useState('');
  const [role, setRole] = useState('');
  const [department, setDepartment] = useState<'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech'>('Growth');
  const [description, setDescription] = useState('');
  const [salaryUsd, setSalaryUsd] = useState(1400);

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const filteredAgents = agents.filter(
    (a) => deptFilter === 'All' || a.department === deptFilter
  );

  const handleStartInterview = async (candidate: CandidateProfile) => {
    setSelectedCandidate(candidate);
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
          technicalScore: 95,
          portfolioScore: 94,
          cultureScore: 96,
          governorVerdict: 'Approve Recommendation',
          summary: `${candidate.name} đạt tiêu chuẩn chuyên gia cấp cao với kinh nghiệm ${candidate.yearsExperience} năm. Phù hợp hoàn hảo với vai trò ${candidate.role}.`,
          negotiatedSalaryMinor: candidate.expectedSalaryMinor,
        });
      }, 600);
    }
    setIsInterviewing(false);
  };

  const handleConfirmHire = async (candidate: CandidateProfile) => {
    if (onHireCandidate) {
      const res = await onHireCandidate(candidate.id);
      if (res && res.success) {
        setHireSuccessNotice(`Đã tuyển dụng thành công ${candidate.name} (${candidate.role})!`);
        setTimeout(() => setHireSuccessNotice(null), 3500);
        setSelectedCandidate(null);
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
      description: description.trim() || 'Chuyên viên xử lý nhiệm vụ tự động',
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

  return (
    <div className="space-y-5 pb-12">
      {/* Top Header & Navigation Switcher */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border-b border-slate-800 pb-4">
        <div className="flex items-center gap-2 bg-slate-900/80 p-1 rounded-xl border border-slate-800 text-xs">
          <button
            onClick={() => setTab('roster')}
            className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg font-medium transition-colors ${
              tab === 'roster'
                ? 'bg-blue-600 text-white shadow-sm'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Users className="w-4 h-4" />
            <span>Đội Ngũ Nhân Sự ({agents.length})</span>
          </button>
          <button
            onClick={() => setTab('market')}
            className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg font-medium transition-colors ${
              tab === 'market'
                ? 'bg-blue-600 text-white shadow-sm'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Briefcase className="w-4 h-4" />
            <span>Sàn Tuyển Dụng Chuyên Gia ({candidates.length})</span>
          </button>
        </div>

        <div className="flex items-center gap-2">
          {tab === 'roster' && (
            <>
              {/* Department Filter */}
              <div className="flex bg-slate-900/80 p-1 rounded-lg border border-slate-800 text-xs">
                {(['All', 'Leadership', 'Growth', 'Ops'] as const).map((cat) => (
                  <button
                    key={cat}
                    onClick={() => setDeptFilter(cat)}
                    className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                      deptFilter === cat ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-white'
                    }`}
                  >
                    {cat === 'All' ? 'Tất cả' : cat === 'Leadership' ? 'Ban Giám Đốc' : cat === 'Growth' ? 'Kinh Doanh' : 'Vận Hành'}
                  </button>
                ))}
              </div>

              {/* Custom Hire Button */}
              <button
                onClick={() => setShowHireModal(true)}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-medium text-xs shadow-sm transition-colors"
              >
                <Plus className="w-3.5 h-3.5" /> Tuyển Thêm
              </button>
            </>
          )}
        </div>
      </div>

      {hireSuccessNotice && (
        <div className="p-3 rounded-lg bg-emerald-950/60 border border-emerald-800/60 text-emerald-300 text-xs flex items-center gap-2 animate-fadeIn">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{hireSuccessNotice}</span>
        </div>
      )}

      {/* TAB 1: ROSTER VIEW */}
      {tab === 'roster' && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3.5">
          {filteredAgents.map((agent) => (
            <div key={agent.id} className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 flex flex-col justify-between">
              <div>
                <div className="flex items-start justify-between gap-2 mb-2">
                  <div>
                    <h4 className="text-sm font-semibold text-white">{agent.name}</h4>
                    <p className="text-xs text-blue-400">{agent.role}</p>
                  </div>
                  <span className={`px-2 py-0.5 rounded text-[10px] font-medium ${
                    agent.status === 'Active'
                      ? 'bg-emerald-950/60 text-emerald-400 border border-emerald-800/40'
                      : 'bg-slate-800 text-slate-400 border border-slate-700'
                  }`}>
                    {agent.status === 'Active' ? 'Đang hoạt động' : 'Tạm dừng'}
                  </span>
                </div>

                <p className="text-xs text-slate-400 line-clamp-2 mb-3">
                  {agent.description || 'Chuyên viên xử lý công việc tự động.'}
                </p>

                {agent.trainedSkills && agent.trainedSkills.length > 0 && (
                  <div className="flex flex-wrap gap-1 mb-3">
                    {agent.trainedSkills.slice(0, 3).map((skill, idx) => (
                      <span key={idx} className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-300">
                        {skill}
                      </span>
                    ))}
                  </div>
                )}
              </div>

              <div className="pt-3 border-t border-slate-800 flex items-center justify-between text-xs">
                <div>
                  <span className="text-[11px] text-slate-500 block">Lương tháng</span>
                  <span className="font-mono font-semibold text-white">{formatMoney(agent.salary_minor)}</span>
                </div>

                <div className="flex items-center gap-1.5">
                  {onOpenTraining && (
                    <button
                      onClick={() => onOpenTraining(agent.id)}
                      className="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-medium text-[11px] transition-colors"
                    >
                      Đào tạo
                    </button>
                  )}
                  <button
                    onClick={() => onToggleStatus?.(agent.id)}
                    className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white transition-colors"
                    title={agent.status === 'Active' ? 'Tạm dừng' : 'Kích hoạt'}
                  >
                    <Power className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* TAB 2: TALENT MARKET VIEW */}
      {tab === 'market' && (
        <div className="space-y-4">
          <div className="p-4 rounded-xl bg-slate-900/40 border border-slate-800 flex items-center justify-between">
            <div>
              <h3 className="text-sm font-semibold text-white">Thị Trường Ứng Viên Cấp Cao</h3>
              <p className="text-xs text-slate-400">Sàng lọc chuyên gia Senior Expert có năng lực chuyên môn và chỉ số ROI rõ ràng</p>
            </div>
            <span className="text-xs font-mono text-emerald-400 bg-emerald-950/60 border border-emerald-800/40 px-2.5 py-1 rounded-lg">
              Kho bạc: {formatMoney(snapshot.cash_minor)}
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {candidates.map((cand) => (
              <div key={cand.id} className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 flex flex-col justify-between">
                <div>
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <div>
                      <div className="flex items-center gap-1.5">
                        <h4 className="text-sm font-semibold text-white">{cand.name}</h4>
                        <span className="px-1.5 py-0.5 rounded text-[10px] bg-blue-950/60 text-blue-400 border border-blue-800/40 font-medium">
                          {cand.level}
                        </span>
                      </div>
                      <p className="text-xs text-blue-400 font-medium">{cand.role}</p>
                    </div>
                    <span className="text-xs font-mono font-semibold text-emerald-400 bg-slate-950 px-2 py-0.5 rounded border border-slate-800">
                      {formatMoney(cand.expectedSalaryMinor)}/th
                    </span>
                  </div>

                  <p className="text-xs text-slate-400 mb-3">{cand.bio}</p>

                  <div className="space-y-1.5 mb-3 bg-slate-950/60 p-2.5 rounded-lg border border-slate-800/80">
                    <span className="text-[11px] font-semibold text-slate-300 block">Kỹ năng chuyên môn:</span>
                    {cand.skills.map((s, idx) => (
                      <div key={idx} className="flex items-center justify-between text-[11px] text-slate-400">
                        <span>{s.name}</span>
                        <span className="font-mono text-slate-300 font-semibold">{s.score}/100</span>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="pt-3 border-t border-slate-800 flex items-center justify-between">
                  <span className="text-xs text-emerald-400 font-medium flex items-center gap-1">
                    <TrendingUp className="w-3.5 h-3.5" /> Dự phóng ROI: +{(cand.roiProjectionBps / 100).toFixed(1)}%
                  </span>

                  <button
                    onClick={() => handleStartInterview(cand)}
                    className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-medium text-xs transition-colors"
                  >
                    <span>Phỏng Vấn AI 3 Vòng</span>
                    <ArrowRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* AI Interview Modal */}
      {selectedCandidate && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 max-w-lg w-full shadow-2xl text-xs space-y-4 animate-fadeIn">
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <div>
                <h3 className="text-sm font-semibold text-white">Kết Quả Phỏng Vấn AI 3 Vòng</h3>
                <p className="text-xs text-slate-400">{selectedCandidate.name} • {selectedCandidate.role}</p>
              </div>
              <button
                onClick={() => setSelectedCandidate(null)}
                className="text-slate-400 hover:text-white p-1 rounded-lg"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {isInterviewing ? (
              <div className="py-8 text-center text-slate-400 space-y-2">
                <Sparkles className="w-6 h-6 text-blue-400 animate-spin mx-auto" />
                <p>AI Governor &amp; Recruiter đang tiến hành phỏng vấn 3 vòng...</p>
              </div>
            ) : interviewResult ? (
              <div className="space-y-3">
                <div className="grid grid-cols-3 gap-2 text-center">
                  <div className="p-2 rounded-lg bg-slate-950 border border-slate-800">
                    <span className="text-[10px] text-slate-500 block">Chuyên môn</span>
                    <span className="font-bold text-sm text-blue-400 font-mono">{interviewResult.technicalScore}/100</span>
                  </div>
                  <div className="p-2 rounded-lg bg-slate-950 border border-slate-800">
                    <span className="text-[10px] text-slate-500 block">Kinh nghiệm</span>
                    <span className="font-bold text-sm text-purple-400 font-mono">{interviewResult.portfolioScore}/100</span>
                  </div>
                  <div className="p-2 rounded-lg bg-slate-950 border border-slate-800">
                    <span className="text-[10px] text-slate-500 block">Văn hóa</span>
                    <span className="font-bold text-sm text-emerald-400 font-mono">{interviewResult.cultureScore}/100</span>
                  </div>
                </div>

                <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-white">Phán quyết Governor AI:</span>
                    <span className="text-emerald-400 font-semibold">{interviewResult.governorVerdict}</span>
                  </div>
                  <p className="text-slate-400 text-[11px]">{interviewResult.summary}</p>
                </div>

                <div className="flex items-center justify-between pt-2">
                  <div>
                    <span className="text-[10px] text-slate-500 block">Mức lương thỏa thuận:</span>
                    <span className="font-bold font-mono text-white text-sm">
                      {formatMoney(interviewResult.negotiatedSalaryMinor || selectedCandidate.expectedSalaryMinor)}/tháng
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => setSelectedCandidate(null)}
                      className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 font-medium"
                    >
                      Đóng
                    </button>
                    <button
                      onClick={() => handleConfirmHire(selectedCandidate)}
                      className="px-3.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-semibold shadow-sm"
                    >
                      Duyệt Tuyển Ngay
                    </button>
                  </div>
                </div>
              </div>
            ) : null}
          </div>
        </div>
      )}

      {/* Manual Hire Modal */}
      {showHireModal && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 max-w-md w-full shadow-2xl text-xs space-y-4">
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <h3 className="text-sm font-semibold text-white">Tuyển Dụng Nhân Sự AI Tùy Chỉnh</h3>
              <button onClick={() => setShowHireModal(false)} className="text-slate-400 hover:text-white">
                <X className="w-4 h-4" />
              </button>
            </div>

            {hireError && (
              <div className="p-2.5 rounded bg-rose-950/60 border border-rose-800/60 text-rose-300">
                {hireError}
              </div>
            )}

            <form onSubmit={handleHireSubmit} className="space-y-3">
              <div>
                <label className="text-slate-400 block mb-1">Tên Nhân Sự</label>
                <input
                  type="text"
                  placeholder="Ví dụ: Growth Hacker Lead"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-blue-500"
                  required
                />
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="text-slate-400 block mb-1">Vai Trò</label>
                  <input
                    type="text"
                    placeholder="Chuyên viên..."
                    value={role}
                    onChange={(e) => setRole(e.target.value)}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-blue-500"
                    required
                  />
                </div>
                <div>
                  <label className="text-slate-400 block mb-1">Phòng Ban</label>
                  <select
                    value={department}
                    onChange={(e) => setDepartment(e.target.value as any)}
                    className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white focus:outline-none focus:border-blue-500"
                  >
                    <option value="Growth">Kinh Doanh (Growth)</option>
                    <option value="Ops">Vận Hành (Ops)</option>
                    <option value="Leadership">Ban Giám Đốc</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="text-slate-400 block mb-1">Lương Tháng ($USD)</label>
                <input
                  type="number"
                  value={salaryUsd}
                  onChange={(e) => setSalaryUsd(Number(e.target.value))}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-white font-mono focus:outline-none focus:border-blue-500"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setShowHireModal(false)}
                  className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 font-medium"
                >
                  Hủy
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting}
                  className="px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold"
                >
                  {isSubmitting ? 'Đang xử lý...' : 'Xác Nhận Tuyển'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
