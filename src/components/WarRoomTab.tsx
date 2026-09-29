import React, { useState } from 'react';
import { CompanySnapshot, DebateTurn } from '../types/company';
import { 
  Users, 
  MessageSquare, 
  Gavel, 
  Send, 
  Sparkles, 
  ShieldCheck, 
  TrendingUp, 
  AlertTriangle,
  RotateCw,
  HelpCircle
} from 'lucide-react';

interface WarRoomTabProps {
  snapshot: CompanySnapshot;
}

export const WarRoomTab: React.FC<WarRoomTabProps> = ({ snapshot }) => {
  const [topic, setTopic] = useState('');
  const [isDebating, setIsDebating] = useState(false);
  const [debateRounds, setDebateRounds] = useState<DebateTurn[]>([
    {
      agent: 'CEO',
      stance: 'Chủ Trương Đột Phá (Pro-Aggressive Expansion)',
      argument: `Với tỷ lệ tăng trưởng khán giả hiện tại là +${(snapshot.audience_growth_bps / 100).toFixed(2)}%, nếu chỉ co cụm phòng thủ chúng ta sẽ đánh mất vị thế độc quyền thị trường creator media.`,
      proposedAction: 'Ủy quyền trích $1,500 từ ngân sách thử nghiệm để mở rộng 3 kênh TikTok affiliate mới.',
    },
    {
      agent: 'CFO',
      stance: 'Phản Biện Fiduciary & Kiểm Soát Rủi Ro',
      argument: `Số ngày runway hiện tại là ${snapshot.runway_days} ngày với chi phí cố định $${(snapshot.expenses_minor / 100).toFixed(2)}/tháng. Nếu giải ngân $1,500 mà CAC tăng đột biến, hệ thống sẽ rơi vào trạng thái Warning ngay lập tức.`,
      proposedAction: 'Giới hạn mức thử nghiệm tối đa ở $300, giải ngân theo 3 mốc (milestones) kèm điều kiện hòa vốn sau 10 ngày.',
    },
    {
      agent: 'COO',
      stance: 'Thực Tế Vận Hành & Khả Năng Thực Thi',
      argument: `Hàng đợi backlog đang ở mức ${snapshot.backlog} so với công suất ${snapshot.capacity}. Đội ngũ kỹ sư prompt và media worker chưa thể render thêm 50 video mỗi tuần nếu chưa tối ưu cache.`,
      proposedAction: 'Ưu tiên tối ưu hóa pipeline dựng video tự động trước khi nhận thêm deal affiliate.',
    },
    {
      agent: 'Governor',
      stance: 'Phán Quyết Hiến Định Cuối Cùng',
      argument: 'Hiến pháp Company OS quy định: Mọi hành động mở rộng phải bảo đảm số ngày runway tối thiểu trên 90 ngày và không làm tăng backlog quá 70% công suất.',
      finalRuling: 'CHẤP THUẬN CÓ ĐIỀU KIỆN (Trần Ngân Sách: $350.00)',
      policyJustification: 'Dung hòa mục tiêu tăng trưởng của CEO với trần an toàn của CFO và hạn ngạch năng lực của COO.',
    },
  ]);

  const presetTopics = [
    'Dòng tiền suy giảm còn 30 ngày: Cắt giảm chi phí R&D hay bung tiền kéo traffic affiliate?',
    'Một video review sản phẩm AI đạt 2M views: Đổ $2,000 ad spend để scale hay giữ lợi nhuận?',
    'Nền tảng TikTok Shop giảm hoa hồng từ 20% xuống 7%: Nên đổi ngách hay thương lượng hợp đồng riêng?',
    'Nên tuyển thêm Full-time AI Prompt Specialist hay thuê ngoài Contractor Upwork theo dự án?',
  ];

  const handleRunDebate = async (selectedTopic?: string) => {
    const finalTopic = selectedTopic || topic;
    if (!finalTopic.trim()) return;

    setIsDebating(true);
    try {
      const res = await fetch('/api/agent-debate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ topic: finalTopic }),
      });
      const data = await res.json();
      if (data.debate && Array.isArray(data.debate)) {
        setDebateRounds(data.debate);
      }
    } catch (e) {
      console.error('Debate failed:', e);
    } finally {
      setIsDebating(false);
    }
  };

  const getAgentColor = (agent: string) => {
    switch (agent) {
      case 'CEO':
        return 'border-indigo-500/30 bg-indigo-950/20 text-indigo-400';
      case 'CFO':
        return 'border-emerald-500/30 bg-emerald-950/20 text-emerald-400';
      case 'COO':
        return 'border-cyan-500/30 bg-cyan-950/20 text-cyan-400';
      case 'Governor':
        return 'border-purple-500/40 bg-purple-950/30 text-purple-300';
      default:
        return 'border-slate-800 bg-slate-900 text-slate-300';
    }
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto pb-16">
      {/* Header Info */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-2 shadow-lg">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <Users className="w-5 h-5 text-indigo-400" />
              Phòng Tranh Luận Điều Hành (Executive War Room)
            </h2>
            <span className="px-2 py-0.5 rounded text-xs font-mono bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
              Gemini 3.8 Flash Powered
            </span>
          </div>
          <span className="text-xs text-slate-400 font-mono">Giải quyết điểm nghẽn "Agent Chạy Cô Lập"</span>
        </div>
        <p className="text-xs text-slate-400 leading-relaxed">
          Nơi CEO, CFO, COO và Governor trực tiếp tranh luận, phản biện chéo các tình huống tiến thoái lưỡng nan của doanh nghiệp. 
          Không còn việc AI tự đề xuất mù quáng mà phải bảo vệ luận điểm của mình trước hội đồng trước khi Governor ban hành phán quyết.
        </p>
      </div>

      {/* Preset Dilemma Topics */}
      <div className="space-y-2">
        <span className="text-xs uppercase font-mono text-slate-400 block">Chọn tình huống nan giải tiêu biểu:</span>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
          {presetTopics.map((pt, idx) => (
            <button
              key={idx}
              onClick={() => {
                setTopic(pt);
                handleRunDebate(pt);
              }}
              className="text-left p-3 rounded-lg bg-slate-900/80 hover:bg-slate-800 border border-slate-800 text-xs text-slate-300 transition-all hover:border-indigo-500/40 flex items-center justify-between gap-2"
            >
              <span>{pt}</span>
              <Sparkles className="w-3.5 h-3.5 text-indigo-400 shrink-0" />
            </button>
          ))}
        </div>
      </div>

      {/* Custom Topic Input */}
      <div className="flex gap-2">
        <input
          type="text"
          value={topic}
          onChange={(e) => setTopic(e.target.value)}
          placeholder="Hoặc nhập câu hỏi / tình huống chiến lược để 4 Agents tranh luận..."
          className="flex-1 bg-slate-900 border border-slate-800 rounded-xl px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition-all"
          onKeyDown={(e) => e.key === 'Enter' && handleRunDebate()}
        />
        <button
          onClick={() => handleRunDebate()}
          disabled={isDebating}
          className={`flex items-center gap-2 px-5 py-2.5 rounded-xl font-bold text-white text-sm transition-all shadow-md shrink-0 ${
            isDebating
              ? 'bg-slate-800 text-slate-400 cursor-not-allowed'
              : 'bg-indigo-600 hover:bg-indigo-500 active:scale-95'
          }`}
        >
          <RotateCw className={`w-4 h-4 ${isDebating ? 'animate-spin' : ''}`} />
          {isDebating ? 'Đang Tranh Luận...' : 'Tổ Chức Tranh Luận'}
        </button>
      </div>

      {/* Live Debate Stream */}
      <div className="space-y-4">
        {debateRounds.map((turn, i) => (
          <div
            key={i}
            className={`p-5 rounded-xl border ${getAgentColor(turn.agent)} transition-all shadow-md`}
          >
            <div className="flex items-center justify-between border-b border-white/10 pb-2.5 mb-3">
              <div className="flex items-center gap-2.5">
                <span className="font-extrabold text-sm uppercase tracking-wide font-mono px-2 py-0.5 rounded bg-black/30">
                  {turn.agent}
                </span>
                <span className="text-xs font-semibold text-slate-300">
                  Quan điểm: {turn.stance}
                </span>
              </div>
              {turn.agent === 'Governor' && (
                <span className="inline-flex items-center gap-1 text-xs font-mono text-purple-300 font-bold bg-purple-500/20 px-2 py-0.5 rounded-full border border-purple-500/30">
                  <Gavel className="w-3.5 h-3.5" /> Constitutional Verdict
                </span>
              )}
            </div>

            <div className="text-xs md:text-sm text-slate-200 leading-relaxed mb-3">
              "{turn.argument}"
            </div>

            {turn.proposedAction && (
              <div className="text-xs bg-black/20 p-2.5 rounded-lg border border-white/5 text-slate-300">
                <strong className="text-indigo-300 font-mono">Đề xuất Hành động:</strong> {turn.proposedAction}
              </div>
            )}

            {turn.finalRuling && (
              <div className="space-y-2 mt-2 bg-purple-950/40 p-3 rounded-lg border border-purple-500/30 text-xs">
                <div>
                  <strong className="text-purple-300 font-mono uppercase">Phán Quyết Có Hiệu Lực:</strong>{' '}
                  <span className="font-bold text-white text-sm">{turn.finalRuling}</span>
                </div>
                {turn.policyJustification && (
                  <div className="text-slate-300 text-xs">
                    <strong className="text-purple-400">Căn cứ Hiến định:</strong> {turn.policyJustification}
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
