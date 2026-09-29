import React, { useState } from 'react';
import { CompanySnapshot } from '../types/company';
import { 
  Users, 
  Briefcase, 
  TrendingUp, 
  Cpu, 
  Video, 
  Search, 
  ShieldCheck, 
  Microscope, 
  FileText, 
  Sparkles,
  CheckCircle2,
  Send,
  RotateCw
} from 'lucide-react';

interface ManageAgentsProps {
  snapshot: CompanySnapshot;
  onRunCycle: () => void;
}

export const ManageAgents: React.FC<ManageAgentsProps> = ({ snapshot, onRunCycle }) => {
  const [activeFilter, setActiveFilter] = useState<'All' | 'Leadership' | 'Growth' | 'Ops'>('All');
  const [quickTask, setQuickTask] = useState('');
  const [taskSent, setTaskSent] = useState(false);

  const team = [
    {
      id: 'Governor',
      name: 'Governor',
      role: 'Bảo Vệ Hiến Pháp & Giám Sát Chi Tiêu',
      simpleJob: 'Có quyền phủ quyết (Veto). Ngăn không cho công ty phá sản, khóa ví tiền khi có nguy cơ.',
      category: 'Leadership',
      budget: 'Không giới hạn quyền phủ quyết',
      icon: ShieldCheck,
      color: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
      badge: 'Quyền Tối Cao',
    },
    {
      id: 'CEO',
      name: 'CEO (Giám Đốc Điều Hành)',
      role: 'Định Hướng Chiến Lược & Phân Bổ Vốn',
      simpleJob: 'Quyết định mở rộng thị trường mới, duyệt ngân sách nghiên cứu và bảo vệ tầm nhìn dài hạn.',
      category: 'Leadership',
      budget: `$${(snapshot.budget_remaining_minor / 100).toLocaleString()}`,
      icon: Briefcase,
      color: 'text-indigo-400 bg-indigo-500/10 border-indigo-500/20',
      badge: 'Ban Giám Đốc',
    },
    {
      id: 'CFO',
      name: 'CFO (Giám Đốc Tài Chính)',
      role: 'Quản Lý Tiền Mặt & Chống Thua Lỗ',
      simpleJob: 'Theo dõi từng đồng chi tiêu. Nếu công ty đốt tiền quá nhanh, CFO tự động yêu cầu cắt giảm ngay.',
      category: 'Leadership',
      budget: 'Kiểm toán toàn bộ kho bạc',
      icon: TrendingUp,
      color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
      badge: 'Tài Chính',
    },
    {
      id: 'COO',
      name: 'COO (Giám Đốc Vận Hành)',
      role: 'Điều Phối Khối Lượng Công Việc',
      simpleJob: 'Đảm bảo hệ thống máy chủ và hàng đợi không bị nghẽn, các tác vụ được hoàn thành đúng hạn.',
      category: 'Ops',
      budget: '22 tác vụ/chu kỳ',
      icon: Cpu,
      color: 'text-cyan-400 bg-cyan-500/10 border-cyan-500/20',
      badge: 'Vận Hành',
    },
    {
      id: 'Growth',
      name: 'Growth Lead (Trưởng Nhóm Tăng Trưởng)',
      role: 'Tìm Kiếm Khách Hàng & Traffic',
      simpleJob: 'Tăng lượt xem, tối ưu hóa tỷ lệ chuyển đổi cho các link mua hàng affiliate trên TikTok/Reels.',
      category: 'Growth',
      budget: '$500 / thử nghiệm',
      icon: Sparkles,
      color: 'text-pink-400 bg-pink-500/10 border-pink-500/20',
      badge: 'Tăng Trưởng',
    },
    {
      id: 'Content',
      name: 'Content Lead (Trưởng Nhóm Nội Dung)',
      role: 'Sản Xuất Video & Kịch Bản Bán Hàng',
      simpleJob: 'Viết kịch bản short-form có hook hấp dẫn, chèn sản phẩm hoa hồng cao vào video.',
      category: 'Growth',
      budget: '$300 / chiến dịch',
      icon: Video,
      color: 'text-amber-400 bg-amber-500/10 border-amber-500/20',
      badge: 'Nội Dung',
    },
    {
      id: 'Recruiter',
      name: 'Recruiter (Trưởng Phòng Tuyển Dụng)',
      role: 'Tuyển Dụng & Bổ Sung Nhân Sự',
      simpleJob: 'Chỉ đề xuất tuyển thêm người khi công ty làm ăn có lãi và số ngày sống còn > 60 ngày.',
      category: 'Ops',
      budget: 'Theo phê duyệt của Operator',
      icon: Search,
      color: 'text-teal-400 bg-teal-500/10 border-teal-500/20',
      badge: 'Nhân Sự',
    },
    {
      id: 'Analyst',
      name: 'Analyst (Chuyên Viên Phân Tích)',
      role: 'Báo Cáo Số Liệu & Kiểm Chứng Doanh Thu',
      simpleJob: 'Kiểm tra doanh thu đối soát hoa hồng thực tế, loại bỏ doanh thu ảo trước khi chia thưởng.',
      category: 'Ops',
      budget: 'Miễn phí (Phân tích dữ liệu)',
      icon: FileText,
      color: 'text-blue-400 bg-blue-500/10 border-blue-500/20',
      badge: 'Dữ Liệu',
    },
    {
      id: 'Experiment',
      name: 'Experiment Specialist (Nghiên Cứu Thử Nghiệm)',
      role: 'Thử Nghiệm A/B Test Tốc Độ Cao',
      simpleJob: 'Chạy các thử nghiệm nhỏ với ngân sách cách ly để tìm ra mẫu video viral mới nhất.',
      category: 'Growth',
      budget: `$${(snapshot.experiment_budget_minor / 100).toLocaleString()} (Quỹ riêng)`,
      icon: Microscope,
      color: 'text-violet-400 bg-violet-500/10 border-violet-500/20',
      badge: 'R&D',
    },
  ];

  const handleSendTask = (e: React.FormEvent) => {
    e.preventDefault();
    if (!quickTask.trim()) return;
    setTaskSent(true);
    setTimeout(() => {
      setTaskSent(false);
      setQuickTask('');
    }, 4000);
  };

  const filteredTeam = team.filter((member) => {
    if (activeFilter === 'All') return true;
    return member.category === activeFilter;
  });

  return (
    <div className="space-y-6 max-w-5xl mx-auto pb-12">
      {/* Header Info */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-md">
        <div>
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Users className="w-5 h-5 text-indigo-400" />
            Quản Lý Đội Ngũ 9 Nhân Sự AI
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Mỗi Agent có một nhiệm vụ rõ ràng và bị giám sát bởi Governor để tránh lãng phí vốn.
          </p>
        </div>

        {/* Quick Filter */}
        <div className="flex items-center gap-1.5 bg-slate-950 p-1 rounded-lg border border-slate-800">
          {(['All', 'Leadership', 'Growth', 'Ops'] as const).map((filter) => (
            <button
              key={filter}
              onClick={() => setActiveFilter(filter)}
              className={`px-3 py-1 rounded-md text-xs font-semibold transition-all ${
                activeFilter === filter
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              {filter === 'All' ? 'Tất cả (9)' : filter === 'Leadership' ? 'Ban Giám Đốc' : filter === 'Growth' ? 'Tăng Trưởng' : 'Vận Hành'}
            </button>
          ))}
        </div>
      </div>

      {/* Quick Task Dispatch Form */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4">
        <form onSubmit={handleSendTask} className="flex gap-2">
          <input
            type="text"
            value={quickTask}
            onChange={(e) => setQuickTask(e.target.value)}
            placeholder="Nhập chỉ đạo nhanh cho toàn bộ ban giám đốc AI (Ví dụ: 'Tập trung đẩy mạnh video công nghệ AI')..."
            className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
          />
          <button
            type="submit"
            className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs shadow-md transition-all shrink-0"
          >
            <Send className="w-3.5 h-3.5" /> Giao Việc
          </button>
        </form>

        {taskSent && (
          <div className="mt-2 text-xs text-emerald-400 flex items-center gap-1.5 animate-fadeIn">
            <CheckCircle2 className="w-3.5 h-3.5" />
            Chỉ đạo đã được gửi tới CEO và các Agent! Ban giám đốc sẽ họp và thực thi trong chu kỳ tiếp theo.
          </div>
        )}
      </div>

      {/* Team Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredTeam.map((member) => {
          const Icon = member.icon;
          return (
            <div
              key={member.id}
              className="bg-slate-900 border border-slate-800 rounded-xl p-5 hover:border-slate-700 transition-all shadow-md flex flex-col justify-between space-y-4"
            >
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <div className={`p-2.5 rounded-xl border ${member.color}`}>
                    <Icon className="w-5 h-5" />
                  </div>
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-slate-800 text-slate-300">
                    {member.badge}
                  </span>
                </div>

                <div>
                  <h3 className="font-bold text-white text-base">{member.name}</h3>
                  <p className="text-xs text-slate-400 font-medium mt-0.5">{member.role}</p>
                </div>

                <div className="p-3 bg-slate-950/70 border border-slate-800/80 rounded-lg text-xs text-slate-300 leading-relaxed">
                  <strong className="text-white block mb-0.5">Việc phải làm:</strong>
                  {member.simpleJob}
                </div>
              </div>

              <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between text-xs">
                <span className="text-slate-400 text-[11px]">Hạn mức:</span>
                <span className="font-mono font-bold text-cyan-400 text-[11px]">{member.budget}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
