import React, { useState } from 'react';
import { CompanySnapshot } from '../types/company';
import { 
  Bot, 
  ShieldCheck, 
  TrendingUp, 
  Cpu, 
  FileText, 
  Eye, 
  Code, 
  Briefcase, 
  Video, 
  Search, 
  Microscope, 
  Check, 
  ChevronRight,
  Sparkles
} from 'lucide-react';

interface AgentRosterTabProps {
  snapshot: CompanySnapshot;
}

export const AgentRosterTab: React.FC<AgentRosterTabProps> = ({ snapshot }) => {
  const [selectedAgent, setSelectedAgent] = useState<string>('Governor');

  const agents = [
    {
      role: 'Governor',
      title: 'Policy & Authorization Engine',
      desc: 'Bảo hộ hiến pháp công ty, kiểm soát trần rủi ro, ngăn ngừa hành động đơn phương gây phá sản và cưỡng chế người vận hành can thiệp khi có đề xuất lớn (Material).',
      permission: 'Permission::Governor (Tối cao)',
      actions: ['EvaluateProposal', 'Veto', 'Approve', 'EscalateToHuman', 'EnforceDistressPolicy'],
      color: 'border-purple-500/40 bg-purple-950/20 text-purple-300',
      icon: ShieldCheck,
      systemPrompt: `# Governor Agent Prompt
You are the constitutional guardian of Company OS.
Core Directives:
1. Guarantee company survival above all growth opportunities.
2. In Distress or Emergency status, REJECT any proposal with cost > $0.
3. Automatically escalate any proposal exceeding $1,000 or marked irreversible to human operator.
4. Verify double-entry consistency: No spending without funded treasury reserve.`,
    },
    {
      role: 'CEO',
      title: 'Strategy & Capital Allocation',
      desc: 'Xác định luận điểm phát triển kinh doanh, đề xuất phân bổ vốn đầu tư cho R&D và nắm bắt cơ hội thị trường truyền thông số.',
      permission: 'Permission::Propose',
      actions: ['AllocateExperimentBudget', 'ResearchOpportunity', 'ProduceReport', 'ReduceBudget'],
      color: 'border-indigo-500/40 bg-indigo-950/20 text-indigo-300',
      icon: Briefcase,
      systemPrompt: `# CEO Agent Prompt
You are the Chief Executive Officer of an autonomous media company.
You allocate capital toward high-conviction creator syndicates, approve growth hypotheses, and guard corporate vision.
You must respect the CFO's cash constraints and Governor policies.`,
    },
    {
      role: 'CFO',
      title: 'Solvency, Cash & Unit Economics',
      desc: 'Kiểm toán sổ cái kép (double-entry ledger), tính toán ngày runway, giám sát tỷ lệ đốt tiền (burn rate) và chủ động kích hoạt cơ chế thắt lưng buộc bụng.',
      permission: 'Permission::Propose',
      actions: ['ReduceBudget', 'ProduceReport', 'EscalateIncident'],
      color: 'border-emerald-500/40 bg-emerald-950/20 text-emerald-300',
      icon: TrendingUp,
      systemPrompt: `# CFO Agent Prompt
You are the Chief Financial Officer.
Your primary KPI is maintaining runway > 90 days.
If expenses > revenue or runway < 30 days, you MUST propose ReduceBudget.
Every cent spent must be tracked via double-entry accounting entries.`,
    },
    {
      role: 'COO',
      title: 'Operations & Capacity Management',
      desc: 'Theo dõi hàng đợi tác vụ (backlog), phân phối tải cho các worker rendering FFmpeg, cân bằng năng lực sản xuất với nhu cầu.',
      permission: 'Permission::Propose',
      actions: ['RebalanceOperations', 'ProduceReport'],
      color: 'border-cyan-500/40 bg-cyan-950/20 text-cyan-300',
      icon: Cpu,
      systemPrompt: `# COO Agent Prompt
You are the Chief Operating Officer.
You ensure task backlog never exceeds worker capacity.
If backlog > capacity, you propose queue rebalancing and worker priority shifting.`,
    },
    {
      role: 'Growth',
      title: 'Profitable Demand & CAC/LTV Loops',
      desc: 'Tối ưu hóa tỷ lệ chuyển đổi (conversion rate), xây dựng phễu lưu lượng truy cập cho các link affiliate và phân tích chi phí thu hút khách hàng.',
      permission: 'Permission::Propose',
      actions: ['CreateExperiment', 'ResearchOpportunity'],
      color: 'border-pink-500/40 bg-pink-950/20 text-pink-300',
      icon: Sparkles,
      systemPrompt: `# Growth Agent Prompt
You drive audience growth and buyer conversion for affiliate products.
Every growth initiative must demonstrate positive unit economics (projected EPC > CAC).`,
    },
    {
      role: 'Content',
      title: 'Media Pipeline & Creator Registry',
      desc: 'Sản xuất kịch bản short-form, tạo hook 3 giây giữ chân người xem, gắn mã theo dõi tiếp thị liên kết (TikTok Shop, Awin).',
      permission: 'Permission::Propose',
      actions: ['PublishContent', 'ResearchOpportunity', 'CreateExperiment'],
      color: 'border-amber-500/40 bg-amber-950/20 text-amber-300',
      icon: Video,
      systemPrompt: `# Content Agent Prompt
You orchestrate the autonomous media pipeline.
You draft high-retention video outlines, integrate commercial affiliate deals, and maximize engagement without triggering platform spam flags.`,
    },
    {
      role: 'Recruiter',
      title: 'Headcount Economics & Capacity Hiring',
      desc: 'Định lượng chi phí tuyển dụng nhân sự/kỹ sư prompt chuyên biệt, so sánh giữa nhân sự toàn thời gian và thuê ngoài contractor.',
      permission: 'Permission::Propose',
      actions: ['ProposeHire', 'ProduceReport'],
      color: 'border-teal-500/40 bg-teal-950/20 text-teal-300',
      icon: Search,
      systemPrompt: `# Recruiter Agent Prompt
You expand company talent when capacity deficits emerge.
You only propose hiring if company has positive cashflow and runway > 60 days.`,
    },
    {
      role: 'Analyst',
      title: 'Verified Decision Support & Statistics',
      desc: 'Kiểm tra chéo số liệu attribution, loại bỏ attribution ảo (phantom revenue) và cung cấp báo cáo hỗ trợ ra quyết định.',
      permission: 'Permission::Propose',
      actions: ['ProduceReport'],
      color: 'border-blue-500/40 bg-blue-950/20 text-blue-300',
      icon: FileText,
      systemPrompt: `# Analyst Agent Prompt
You provide objective, non-hallucinated verification of business metrics.
You calculate exact attribution decay and statistical significance for experiments.`,
    },
    {
      role: 'Experiment',
      title: 'Bounded Opportunity Discovery',
      desc: 'Thiết kế các thử nghiệm A/B quy mô nhỏ, có trần ngân sách cách ly hoàn toàn với quỹ tiền mặt cốt lõi.',
      permission: 'Permission::Propose',
      actions: ['CreateExperiment'],
      color: 'border-violet-500/40 bg-violet-950/20 text-violet-300',
      icon: Microscope,
      systemPrompt: `# Experiment Agent Prompt
You run fast, low-cost A/B tests to discover emerging creator trends.
Budget is strictly hardcapped to prevent capital leakage.`,
    },
  ];

  const currentAgent = agents.find((a) => a.role === selectedAgent) || agents[0];
  const Icon = currentAgent.icon;

  return (
    <div className="space-y-6 max-w-6xl mx-auto pb-16">
      {/* Intro */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex items-center justify-between shadow-lg">
        <div>
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Bot className="w-5 h-5 text-indigo-400" />
            Trụ Sở 9 AI Agents Điều Hành (Autonomous Executive Roster)
          </h2>
          <p className="text-slate-400 text-xs mt-1">
            Được triển khai theo mô hình phân tầng chặt chẽ từ <code className="text-cyan-400 font-mono">crates/agent-runtime</code> của dự án gốc.
          </p>
        </div>
        <div className="text-right font-mono text-xs">
          <span className="text-slate-500 block">Quy Mô Nhân Sự</span>
          <span className="text-indigo-400 font-bold">9 AI Executives • 2 Specialists</span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left: Agent List */}
        <div className="space-y-2 lg:col-span-1">
          <span className="text-xs uppercase font-mono text-slate-400 block px-1">Danh Sách Agents:</span>
          {agents.map((ag) => {
            const AgIcon = ag.icon;
            const isSelected = ag.role === selectedAgent;
            return (
              <button
                key={ag.role}
                onClick={() => setSelectedAgent(ag.role)}
                className={`w-full text-left p-3.5 rounded-xl border transition-all flex items-center justify-between ${
                  isSelected
                    ? `${ag.color} shadow-lg ring-1 ring-white/10`
                    : 'bg-slate-900/70 border-slate-800/80 text-slate-300 hover:bg-slate-800/60'
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className={`p-2 rounded-lg ${isSelected ? 'bg-white/10' : 'bg-slate-800 text-slate-400'}`}>
                    <AgIcon className="w-4 h-4" />
                  </div>
                  <div>
                    <div className="font-bold text-sm text-white flex items-center gap-1.5">
                      {ag.role}
                      {ag.role === 'Governor' && (
                        <span className="text-[10px] font-mono bg-purple-500/30 text-purple-300 px-1.5 py-0.2 rounded">Hiến định</span>
                      )}
                    </div>
                    <div className="text-[11px] text-slate-400 truncate max-w-[170px]">{ag.title}</div>
                  </div>
                </div>
                <ChevronRight className={`w-4 h-4 ${isSelected ? 'text-white' : 'text-slate-600'}`} />
              </button>
            );
          })}
        </div>

        {/* Right: Detailed Agent Dossier */}
        <div className="lg:col-span-2 space-y-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 shadow-xl space-y-4">
            {/* Header */}
            <div className="flex items-center justify-between border-b border-slate-800 pb-4">
              <div className="flex items-center gap-3">
                <div className="p-3 rounded-xl bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                  <Icon className="w-6 h-6" />
                </div>
                <div>
                  <h3 className="text-xl font-bold text-white">{currentAgent.role} Agent</h3>
                  <p className="text-xs text-slate-400 font-mono">{currentAgent.title}</p>
                </div>
              </div>
              <span className="px-3 py-1 rounded-full text-xs font-mono bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">
                {currentAgent.permission}
              </span>
            </div>

            {/* Description */}
            <div className="text-sm text-slate-300 leading-relaxed bg-slate-950/60 p-4 rounded-xl border border-slate-800/60">
              {currentAgent.desc}
            </div>

            {/* Authorized Actions */}
            <div>
              <span className="text-xs uppercase font-mono text-slate-400 block mb-2">Hành Động Được Phép (ActionKind):</span>
              <div className="flex flex-wrap gap-2">
                {currentAgent.actions.map((act) => (
                  <span
                    key={act}
                    className="px-2.5 py-1 rounded-lg text-xs font-mono bg-slate-950 border border-slate-800 text-cyan-300"
                  >
                    {act}
                  </span>
                ))}
              </div>
            </div>

            {/* Raw System Prompt File */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs uppercase font-mono text-slate-400 flex items-center gap-1.5">
                  <Code className="w-3.5 h-3.5 text-indigo-400" />
                  Mã Nguồn System Prompt (agents/{currentAgent.role.toLowerCase()}/agent.md):
                </span>
                <span className="text-[10px] font-mono text-slate-500">ReadOnly</span>
              </div>
              <pre className="p-4 rounded-xl bg-slate-950 border border-slate-800 text-xs font-mono text-slate-300 overflow-x-auto whitespace-pre-wrap leading-relaxed shadow-inner">
                {currentAgent.systemPrompt}
              </pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
