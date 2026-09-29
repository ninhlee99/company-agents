import React, { useState } from 'react';
import { CompanySnapshot, CustomAgent } from '../types/company';
import { 
  Users, 
  Plus, 
  Send, 
  CheckCircle2, 
  BarChart3, 
  Power, 
  X,
  AlertTriangle,
  Award,
  TrendingDown
} from 'lucide-react';
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
  onHireAgent: (data: { name: string; role: string; department: 'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech'; description: string; salary_minor: number }) => Promise<{ success: boolean; reason?: string }>;
  onToggleStatus: (agentId: string) => void;
}

export const ManageAgents: React.FC<ManageAgentsProps> = ({
  snapshot,
  agents,
  onHireAgent,
  onToggleStatus,
}) => {
  const [filter, setFilter] = useState<'All' | 'Leadership' | 'Growth' | 'Ops'>('All');
  const [showHireModal, setShowHireModal] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [hireError, setHireError] = useState<string | null>(null);

  // Form State
  const [name, setName] = useState('');
  const [role, setRole] = useState('');
  const [department, setDepartment] = useState<'Leadership' | 'Growth' | 'Ops' | 'Sales' | 'Tech'>('Growth');
  const [description, setDescription] = useState('');
  const [salaryUsd, setSalaryUsd] = useState(1500);

  const [quickInstruction, setQuickInstruction] = useState('');
  const [instructionSent, setInstructionSent] = useState(false);

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

  // Prepare data for Agent Performance Bar Chart
  const chartData = agents.map((a) => {
    const costUsd = a.salary_minor > 0 ? Math.round(a.salary_minor / 100) : 350; // estimated nominal compute cost for core agents
    const efficiencyRatio = Math.round((a.tasksCompleted / (costUsd || 1)) * 1000); // tasks per $1k spend
    return {
      name: a.name.replace(' Agent', '').replace(' Specialist', ''),
      fullName: a.name,
      tasks: a.tasksCompleted,
      cost: costUsd,
      dept: a.department,
      efficiency: efficiencyRatio,
    };
  });

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
      avgTasks: avgTasksPerAgent,
      status: avgTasksPerAgent >= 40 ? 'Tối ưu (Hiệu suất cao)' : avgTasksPerAgent >= 25 ? 'Bình thường' : 'Cần giao thêm việc',
    };
  });

  return (
    <div className="space-y-4 max-w-5xl mx-auto pb-10">
      {/* Top Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
        <div className="flex items-center gap-2">
          <Users className="w-5 h-5 text-indigo-400" />
          <div>
            <h2 className="text-base font-bold text-white">Đội Ngũ Nhân Sự AI ({agents.length} vị trí)</h2>
            <p className="text-xs text-slate-400">Theo dõi hiệu suất hoàn thành việc và chi phí vận hành từng nhân sự</p>
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

          {/* Hire Button */}
          <button
            onClick={() => setShowHireModal(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs shadow-md transition-all shrink-0"
          >
            <Plus className="w-3.5 h-3.5" /> Tuyển Thêm AI
          </button>
        </div>
      </div>

      {/* RECHARTS BAR CHART: VISUAL PERFORMANCE BREAKDOWN (TASKS COMPLETED VS COST) */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3 shadow-sm">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
          <div className="flex items-center gap-2">
            <BarChart3 className="w-4 h-4 text-emerald-400" />
            <div>
              <span className="font-bold text-white text-xs block">Hiệu Suất Nhân Sự AI: Khối Lượng Tác Vụ vs. Chi Phí</span>
              <span className="text-[10px] text-slate-400">Giúp nhà điều hành phát hiện nhân sự/phòng ban chưa đạt kỳ vọng</span>
            </div>
          </div>

          <div className="flex items-center gap-3 text-[11px]">
            <span className="flex items-center gap-1 text-indigo-400 font-mono">
              <span className="w-2.5 h-2.5 rounded-sm bg-indigo-500 inline-block" /> Việc hoàn thành
            </span>
            <span className="flex items-center gap-1 text-rose-400 font-mono">
              <span className="w-2.5 h-2.5 rounded-sm bg-rose-500 inline-block" /> Chi phí ($)
            </span>
          </div>
        </div>

        {/* Bar Chart Container */}
        <div className="h-64 w-full pt-1">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={chartData} margin={{ top: 10, right: 10, left: -15, bottom: 25 }}>
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
                formatter={(val: any, name: any) => [
                  name === 'tasks' ? `${val} tác vụ` : `$${Number(val || 0).toLocaleString()}`,
                  name === 'tasks' ? 'Số Việc Đã Làm' : 'Chi Phí Tháng',
                ]}
                labelStyle={{ color: '#e2e8f0', fontWeight: 'bold' }}
              />
              <Legend
                wrapperStyle={{ fontSize: '11px', paddingTop: '4px' }}
                formatter={(val) => (val === 'tasks' ? 'Số việc hoàn thành' : 'Chi phí ước tính ($)')}
              />
              <Bar dataKey="tasks" fill="#6366f1" radius={[4, 4, 0, 0]} name="tasks" />
              <Bar dataKey="cost" fill="#f43f5e" radius={[4, 4, 0, 0]} name="cost" />
            </BarChart>
          </ResponsiveContainer>
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

      {/* Agents Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        {filteredAgents.map((agent) => (
          <div
            key={agent.id}
            className="bg-slate-900 border border-slate-800 rounded-xl p-3.5 hover:border-slate-700 transition-all flex flex-col justify-between space-y-2.5 shadow-sm"
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

            <div className="pt-2 border-t border-slate-800 flex items-center justify-between text-[11px] text-slate-400 font-mono">
              <span>Đã làm: <strong className="text-slate-200">{agent.tasksCompleted} việc</strong></span>
              {agent.salary_minor > 0 ? (
                <span className="text-rose-400">${(agent.salary_minor / 100).toLocaleString()}/th</span>
              ) : (
                <span className="text-cyan-400">Core Agent</span>
              )}
            </div>
          </div>
        ))}
      </div>

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
    </div>
  );
};
