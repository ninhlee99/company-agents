import React, { useState, useEffect } from 'react';
import { CompanySnapshot, DebateTurn, AgentMessage } from '../types/company';
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
  HelpCircle,
  FileText,
  MessagesSquare,
  Clock,
  Tag,
  ArrowRight,
  Filter,
  CheckCircle2,
  RefreshCw,
  Zap
} from 'lucide-react';

interface WarRoomTabProps {
  snapshot: CompanySnapshot;
}

export const WarRoomTab: React.FC<WarRoomTabProps> = ({ snapshot }) => {
  const [activeSubTab, setActiveSubTab] = useState<'stream' | 'debate'>('stream');
  
  // Executive Debate State
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
      stance: 'Phản Quyết Hiến Định Cuối Cùng',
      argument: 'Hiến pháp Company OS quy định: Mọi hành động mở rộng phải bảo đảm số ngày runway tối thiểu trên 90 ngày và không làm tăng backlog quá 70% công suất.',
      finalRuling: 'CHẤP THUẬN CÓ ĐIỀU KIỆN (Trần Ngân Sách: $350.00)',
      policyJustification: 'Dung hòa mục tiêu tăng trưởng của CEO với trần an toàn của CFO và hạn ngạch năng lực của COO.',
    },
  ]);

  // Communication Stream State
  const [messages, setMessages] = useState<AgentMessage[]>([
    {
      id: 'comm-1',
      type: 'Memo',
      fromAgent: 'Growth Lead',
      fromRole: 'Trưởng Nhóm Kinh Doanh',
      toAgent: 'Toàn Thể Công Ty',
      toRole: 'Hội đồng Điều hành',
      subject: 'Chiến lược mở rộng ngách Setup Bàn Làm Việc Thông Minh',
      content: 'Đã hoàn tất phân tích thị trường affiliate tuần này. EPC trung bình đạt $0.92/click với tỷ lệ chuyển đổi đơn hàng 3.4%. Đề xuất Content Lead tập trung 70% công suất vào dòng sản phẩm bàn phím cơ & đèn màn hình công thái học.',
      actionItem: 'Content Lead sản xuất 4 kịch bản video hook 3s kèm link TikTok Shop.',
      tag: '#MarketResearch',
      cycle: snapshot.cycle_count,
      timestamp: new Date(Date.now() - 3600000 * 3).toISOString(),
      priority: 'High',
    },
    {
      id: 'comm-2',
      type: 'Chat',
      fromAgent: 'Content Lead',
      fromRole: 'Sáng Tạo Nội Dung',
      toAgent: 'Growth Lead',
      toRole: 'Trưởng Nhóm Kinh Doanh',
      content: 'Đã nhận chỉ đạo! 4 kịch bản hoàn tất đạt chuẩn FTC. Media Worker đã render xong bản draft 1080p 60fps. Cần Growth duyệt UTM tracking code trước khi publish lên mạng xã hội.',
      tag: '#AffiliateProduction',
      cycle: snapshot.cycle_count,
      timestamp: new Date(Date.now() - 3600000 * 2.5).toISOString(),
    },
    {
      id: 'comm-3',
      type: 'Chat',
      fromAgent: 'Growth Lead',
      fromRole: 'Trưởng Nhóm Kinh Doanh',
      toAgent: 'Content Lead',
      toRole: 'Sáng Tạo Nội Dung',
      content: 'Đã đối soát link TikTok Shop! Tỷ lệ hoa hồng 22% tự động ghi nhận vào ví kho bạc. Tiến hành xuất bản tự động trên hệ thống ngay!',
      tag: '#Publishing',
      cycle: snapshot.cycle_count,
      timestamp: new Date(Date.now() - 3600000 * 2).toISOString(),
    },
    {
      id: 'comm-4',
      type: 'Memo',
      fromAgent: 'CFO',
      fromRole: 'Giám Đốc Tài Chính',
      toAgent: 'CEO & Governor',
      toRole: 'Ban Lãnh Đạo',
      subject: `Báo cáo kiểm toán chi phí hạ tầng máy chủ & token AI Chu kỳ #${snapshot.cycle_count}`,
      content: 'Tổng chi phí token LLM và render video trong kỳ là $62.00, thấp hơn 24% so với ngân sách dự kiến nhờ kích hoạt cơ chế Prompt Caching. Số ngày runway an toàn ở mức 440 ngày.',
      actionItem: 'Duy trì ngân sách thử nghiệm $4,500.00 cho kỳ tới.',
      tag: '#TreasuryAudit',
      cycle: snapshot.cycle_count,
      timestamp: new Date(Date.now() - 3600000 * 1.5).toISOString(),
      priority: 'Normal',
    },
    {
      id: 'comm-5',
      type: 'Chat',
      fromAgent: 'Governor',
      fromRole: 'Hiến Pháp & Quỹ Tiền',
      toAgent: 'CFO',
      toRole: 'Giám Đốc Tài Chính',
      content: 'Hiến pháp ghi nhận báo cáo an toàn vốn. Trần chi tiêu thử nghiệm tiếp tục được phê chuẩn. Tuyệt đối không giải ngân vượt $1,000 cho một chiến dịch đơn lẻ mà chưa qua hội đồng phê duyệt.',
      tag: '#ConstitutionalVeto',
      cycle: snapshot.cycle_count,
      timestamp: new Date(Date.now() - 3600000 * 1).toISOString(),
    },
    {
      id: 'comm-6',
      type: 'Chat',
      fromAgent: 'COO',
      fromRole: 'Giám Đốc Vận Hành',
      toAgent: 'Recruiter',
      toRole: 'Tuyển Dụng',
      content: 'Tải lượng hàng đợi backlog hiện tại là 12/22 (54% công suất). Hệ thống đang cân bằng tốt, chưa cần kích hoạt tuyển thêm Full-time AI Agent trong kỳ này.',
      tag: '#WorkforcePlanning',
      cycle: snapshot.cycle_count,
      timestamp: new Date(Date.now() - 3600000 * 0.7).toISOString(),
    },
    {
      id: 'comm-7',
      type: 'Chat',
      fromAgent: 'Experimenter',
      fromRole: 'Nghiên Cứu A/B Test',
      toAgent: 'Growth Lead',
      toRole: 'Trưởng Nhóm Kinh Doanh',
      content: 'A/B test biến thể thumbnail nền tối có độ tương phản cao cho kết quả CTR +18.4% so với ảnh chụp phong cách tối giản. Đã cập nhật template tự động cho Media Studio.',
      tag: '#ABTesting',
      cycle: snapshot.cycle_count,
      timestamp: new Date(Date.now() - 3600000 * 0.3).toISOString(),
    },
  ]);

  const [streamFilter, setStreamFilter] = useState<'All' | 'Memo' | 'Chat'>('All');
  const [isSimulatingMessage, setIsSimulatingMessage] = useState(false);

  // Load stream from server
  const loadCommunicationStream = async () => {
    try {
      const res = await fetch('/api/communication-stream');
      if (res.ok) {
        const data = await res.json();
        if (data.stream && data.stream.length > 0) {
          setMessages(data.stream);
        }
      }
    } catch (e) {
      console.warn('Stream fetch note:', e);
    }
  };

  useEffect(() => {
    loadCommunicationStream();
  }, []);

  const handleSimulateCoordination = async () => {
    setIsSimulatingMessage(true);

    const coordinationPresets = [
      {
        type: 'Memo' as const,
        fromAgent: 'CFO',
        toAgent: 'Toàn Thể Công Ty',
        subject: 'Cân đối dòng tiền và chỉ tiêu hoa hồng affiliate kỳ mới',
        content: `Số dư kho bạc hiện tại là $${(snapshot.cash_minor / 100).toLocaleString()}. Đề xuất phòng Growth tăng tốc tiếp cận các mặt hàng có hoa hồng > 18% để đạt mục tiêu doanh thu $10,000/tháng.`,
        actionItem: 'Growth Lead cập nhật danh mục 5 mặt hàng tiềm năng cao.',
        tag: '#TreasuryGuideline',
        priority: 'Normal' as const,
      },
      {
        type: 'Chat' as const,
        fromAgent: 'Content Lead',
        toAgent: 'Media Worker',
        content: 'Đã hoàn thiện kịch bản 30s giải thích cơ chế tản nhiệt bàn phím cơ. Hãy render bản dọc 9:16 với subtitle tự động và gắn link gian hàng TikTok Shop!',
        tag: '#VideoProduction',
      },
      {
        type: 'Chat' as const,
        fromAgent: 'Governor',
        toAgent: 'Growth Lead',
        content: 'Cảnh báo tuân thủ chính sách: Đảm bảo toàn bộ video thương mại đều có nhãn "Quảng cáo tiếp thị liên kết" để tránh rủi ro vi phạm thuật toán nền tảng.',
        tag: '#ComplianceCheck',
      },
      {
        type: 'Chat' as const,
        fromAgent: 'COO',
        toAgent: 'CFO',
        content: 'Báo cáo hiệu năng hàng đợi: Thông lượng xử lý đơn hàng và đối soát hoa hồng đạt 100% không phát sinh lỗi timeout.',
        tag: '#OpsEfficiency',
      },
    ];

    const pick = coordinationPresets[Math.floor(Math.random() * coordinationPresets.length)];

    try {
      const res = await fetch('/api/communication-stream', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(pick),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.stream) {
          setMessages(data.stream);
        }
      }
    } catch (e) {
      console.warn('Simulate error:', e);
    } finally {
      setIsSimulatingMessage(false);
    }
  };

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
      case 'Growth Lead':
      case 'Growth':
        return 'border-pink-500/30 bg-pink-950/20 text-pink-400';
      case 'Content Lead':
      case 'Content':
        return 'border-amber-500/30 bg-amber-950/20 text-amber-300';
      default:
        return 'border-slate-800 bg-slate-900 text-slate-300';
    }
  };

  const filteredMessages = messages.filter((m) => {
    if (streamFilter === 'All') return true;
    return m.type === streamFilter;
  });

  return (
    <div className="space-y-4 max-w-5xl mx-auto pb-16">
      {/* Top Header & Sub-Tab Switcher */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
        <div className="flex items-center gap-2.5">
          <Users className="w-5 h-5 text-indigo-400" />
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-white">Phòng Điều Hành & Phối Hợp (War Room)</h2>
              <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                Autonomous Multi-Agent
              </span>
            </div>
            <p className="text-xs text-slate-400">
              Minh bạch hóa dòng phối hợp, ghi nhớ nội bộ và tranh luận đối trọng giữa các AI Agent
            </p>
          </div>
        </div>

        {/* View Switcher: Communication Stream vs Executive Debate */}
        <div className="flex bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs w-full sm:w-auto">
          <button
            onClick={() => setActiveSubTab('stream')}
            className={`flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-md font-semibold transition-all ${
              activeSubTab === 'stream'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <MessagesSquare className="w-3.5 h-3.5 text-indigo-300" />
            <span>Dòng Trao Đổi Tự Trị ({messages.length})</span>
          </button>

          <button
            onClick={() => setActiveSubTab('debate')}
            className={`flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-md font-semibold transition-all ${
              activeSubTab === 'debate'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Gavel className="w-3.5 h-3.5 text-purple-400" />
            <span>Hội Đồng Tranh Luận</span>
          </button>
        </div>
      </div>

      {/* ======================================================== */}
      {/* 1. COMMUNICATION STREAM VIEW (CHAT LOGS & INTERNAL MEMOS) */}
      {/* ======================================================== */}
      {activeSubTab === 'stream' && (
        <div className="space-y-4">
          {/* Stream Filter & Trigger Bar */}
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-3.5 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
            <div className="flex items-center gap-2">
              <Filter className="w-3.5 h-3.5 text-slate-400" />
              <span className="text-xs text-slate-400 font-medium">Lọc hình thức:</span>
              <div className="flex bg-slate-950 p-0.5 rounded-lg border border-slate-800 text-xs">
                {(['All', 'Memo', 'Chat'] as const).map((mode) => (
                  <button
                    key={mode}
                    onClick={() => setStreamFilter(mode)}
                    className={`px-2.5 py-1 rounded font-semibold transition-all ${
                      streamFilter === mode ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
                    }`}
                  >
                    {mode === 'All' ? 'Tất cả' : mode === 'Memo' ? 'Bản Ghi Nhớ (Memos)' : 'Chat Trực Tiếp (Chat Logs)'}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex items-center gap-2 w-full sm:w-auto">
              <button
                onClick={handleSimulateCoordination}
                disabled={isSimulatingMessage}
                className="flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs shadow-md transition-all active:scale-95 disabled:opacity-50"
              >
                <Zap className={`w-3.5 h-3.5 ${isSimulatingMessage ? 'animate-bounce' : ''}`} />
                <span>{isSimulatingMessage ? 'Đang điều phối...' : 'Kích Hoạt Trao Đổi Mới'}</span>
              </button>

              <button
                onClick={loadCommunicationStream}
                className="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-all"
                title="Làm mới dòng trao đổi"
              >
                <RefreshCw className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          {/* Stream Messages List */}
          <div className="space-y-3">
            {filteredMessages.map((msg) => (
              <div
                key={msg.id}
                className={`p-4 rounded-xl border transition-all shadow-sm ${
                  msg.type === 'Memo'
                    ? 'bg-slate-900 border-indigo-500/30'
                    : 'bg-slate-900 border-slate-800 hover:border-slate-700'
                }`}
              >
                {/* Header: Senders, Recipients, Badges */}
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-1.5 border-b border-slate-800 pb-2.5 mb-2.5">
                  <div className="flex items-center gap-2 flex-wrap text-xs">
                    <span
                      className={`px-2 py-0.5 rounded font-mono font-bold text-[10px] uppercase border ${
                        msg.type === 'Memo'
                          ? 'bg-indigo-500/20 text-indigo-300 border-indigo-500/30'
                          : 'bg-slate-800 text-slate-300 border-slate-700'
                      }`}
                    >
                      {msg.type === 'Memo' ? 'Bản Ghi Nhớ' : 'Chat Trực Tiếp'}
                    </span>

                    <span className="font-bold text-white flex items-center gap-1">
                      <span className="text-indigo-400">{msg.fromAgent}</span>
                      <span className="text-slate-500 text-[10px]">({msg.fromRole})</span>
                    </span>

                    <ArrowRight className="w-3 h-3 text-slate-600" />

                    <span className="text-slate-300 flex items-center gap-1">
                      <span className="text-cyan-400 font-semibold">{msg.toAgent}</span>
                      {msg.toRole && <span className="text-slate-500 text-[10px]">({msg.toRole})</span>}
                    </span>
                  </div>

                  <div className="flex items-center gap-2 text-[10px] font-mono text-slate-500">
                    <span className="px-1.5 py-0.5 rounded bg-slate-950 border border-slate-800 text-slate-400">
                      Kỳ #{msg.cycle}
                    </span>
                    <span>{new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                  </div>
                </div>

                {/* Subject for Memo */}
                {msg.subject && (
                  <div className="font-bold text-white text-xs mb-1.5 flex items-center gap-1.5 text-indigo-200">
                    <FileText className="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                    <span>Chủ đề: {msg.subject}</span>
                  </div>
                )}

                {/* Message Body */}
                <p className="text-xs text-slate-300 leading-relaxed pl-1">
                  "{msg.content}"
                </p>

                {/* Action Item if present */}
                {msg.actionItem && (
                  <div className="mt-2.5 p-2 rounded-lg bg-emerald-950/20 border border-emerald-500/20 text-xs text-emerald-300 flex items-center gap-2">
                    <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                    <span><strong>Nhiệm vụ hành động:</strong> {msg.actionItem}</span>
                  </div>
                )}

                {/* Footer Tag */}
                <div className="mt-2.5 flex items-center justify-between text-[10px] text-slate-500 font-mono pt-2 border-t border-slate-950">
                  <span className="flex items-center gap-1 text-slate-400">
                    <Tag className="w-3 h-3 text-indigo-400" />
                    {msg.tag}
                  </span>
                  <span className="text-[10px] text-slate-600">Được ghi nhận vào nhật ký kiểm toán</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ======================================================== */}
      {/* 2. EXECUTIVE DEBATE VIEW (HISTORICAL / STRATEGIC DILEMMAS) */}
      {/* ======================================================== */}
      {activeSubTab === 'debate' && (
        <div className="space-y-4">
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
              className="flex-1 bg-slate-900 border border-slate-800 rounded-xl px-4 py-2.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition-all"
              onKeyDown={(e) => e.key === 'Enter' && handleRunDebate()}
            />
            <button
              onClick={() => handleRunDebate()}
              disabled={isDebating}
              className={`flex items-center gap-2 px-5 py-2.5 rounded-xl font-bold text-white text-xs transition-all shadow-md shrink-0 ${
                isDebating
                  ? 'bg-slate-800 text-slate-400 cursor-not-allowed'
                  : 'bg-indigo-600 hover:bg-indigo-500 active:scale-95'
              }`}
            >
              <RotateCw className={`w-3.5 h-3.5 ${isDebating ? 'animate-spin' : ''}`} />
              {isDebating ? 'Đang Tranh Luận...' : 'Tổ Chức Tranh Luận'}
            </button>
          </div>

          {/* Live Debate Stream */}
          <div className="space-y-3">
            {debateRounds.map((turn, i) => (
              <div
                key={i}
                className={`p-4 rounded-xl border ${getAgentColor(turn.agent)} transition-all shadow-md`}
              >
                <div className="flex items-center justify-between border-b border-white/10 pb-2 mb-2.5">
                  <div className="flex items-center gap-2">
                    <span className="font-extrabold text-xs uppercase tracking-wide font-mono px-2 py-0.5 rounded bg-black/30">
                      {turn.agent}
                    </span>
                    <span className="text-xs font-semibold text-slate-300">
                      Quan điểm: {turn.stance}
                    </span>
                  </div>
                  {turn.agent === 'Governor' && (
                    <span className="inline-flex items-center gap-1 text-[11px] font-mono text-purple-300 font-bold bg-purple-500/20 px-2 py-0.5 rounded-full border border-purple-500/30">
                      <Gavel className="w-3.5 h-3.5" /> Phán Quyết Hiến Định
                    </span>
                  )}
                </div>

                <div className="text-xs text-slate-200 leading-relaxed mb-2.5">
                  "{turn.argument}"
                </div>

                {turn.proposedAction && (
                  <div className="text-xs bg-black/20 p-2.5 rounded-lg border border-white/5 text-slate-300">
                    <strong className="text-indigo-300 font-mono">Đề xuất Hành động:</strong> {turn.proposedAction}
                  </div>
                )}

                {turn.finalRuling && (
                  <div className="space-y-1.5 mt-2 bg-purple-950/40 p-3 rounded-lg border border-purple-500/30 text-xs">
                    <div>
                      <strong className="text-purple-300 font-mono uppercase">Phán Quyết Có Hiệu Lực:</strong>{' '}
                      <span className="font-bold text-white text-xs">{turn.finalRuling}</span>
                    </div>
                    {turn.policyJustification && (
                      <div className="text-slate-300 text-[11px]">
                        <strong className="text-purple-400">Căn cứ Hiến định:</strong> {turn.policyJustification}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
