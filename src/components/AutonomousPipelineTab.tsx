import React, { useState } from 'react';
import { CompanySnapshot, FullCreativeProduction } from '../types/company';
import { 
  Sparkles, 
  Play, 
  CheckCircle2, 
  ArrowRight, 
  TrendingUp, 
  Video, 
  Search, 
  ShieldCheck, 
  Wallet, 
  RotateCw,
  Cpu,
  Layers,
  Zap,
  Terminal,
  Clock,
  ExternalLink,
  Flame,
  BarChart3,
  Building2,
  FileCheck,
  UserCheck,
  Award,
  Music,
  Camera,
  FileText,
  BadgeAlert
} from 'lucide-react';

interface AutonomousPipelineTabProps {
  snapshot: CompanySnapshot;
  onRunPipeline: (topic: string) => Promise<{ success: boolean; steps: any[]; simulatedRevenueGainMinor: number }>;
}

export const AutonomousPipelineTab: React.FC<AutonomousPipelineTabProps> = ({
  snapshot,
  onRunPipeline,
}) => {
  const [topic, setTopic] = useState('3 Món Đồ Công Nghệ AI Giúp Tôi Tiết Kiệm 14 Tiếng Mỗi Tuần');
  const [isRunning, setIsRunning] = useState(false);
  const [activeStage, setActiveStage] = useState<number | null>(null);
  const [deliverableTab, setDeliverableTab] = useState<'script' | 'audio' | 'visual' | 'accounting'>('script');
  const [logs, setLogs] = useState<string[]>([]);
  const [isApprovedByBoard, setIsApprovedByBoard] = useState(true);

  // Creative Production Details (Merged & Upgraded from Media Studio)
  const [production, setProduction] = useState<FullCreativeProduction>({
    id: 'prod-ent-01',
    campaignTitle: '3 Món Đồ Công Nghệ AI Giúp Tôi Tiết Kiệm 14 Tiếng Mỗi Tuần',
    niche: 'AI Smart Workspace & Desk Gadgets',
    projectedRevenueMinor: 145000,
    costMinor: 15000,
    copywriting: {
      headline: 'Bí Quyết Tăng 300% Năng Suất Làm Việc Với 3 Phụ Kiện AI Này',
      hook3s: 'Đừng mua thêm bàn phím cơ nữa nếu bạn chưa biết 3 món đồ AI này vừa ra mắt trong tháng.',
      retentionFormula: 'Mở đầu phản trực giác ➔ Khơi gợi nỗi đau mất thời gian ➔ Trình diễn giải pháp AI ➔ Tặng coupon giảm 25% độc quyền',
      bodyPainPoints: [
        'Ghi chép cuộc họp thủ công mất hàng giờ mỗi tuần',
        'Dây nhợ lộn xộn làm giảm tập trung và thẩm mỹ góc làm việc',
        'Không bảo mật dữ liệu cục bộ khi dùng các công cụ đám mây',
      ],
      ctaText: 'Bấm ngay vào link bio và nhập mã AGENTSOS để nhận voucher độc quyền 25% trước khi hết slot.',
      targetAudience: 'Dân văn phòng, lập trình viên, content creator, người yêu công nghệ (22-38 tuổi)',
      complianceChecked: true,
    },
    audioTrack: {
      title: 'Cyberpunk Lo-Fi Productivity Beats (128 BPM)',
      genre: 'Lo-Fi Chill',
      bpm: 128,
      mood: 'Tập trung cao độ, hiện đại, kích thích hành động',
      voiceoverTone: 'Confident & Crisp',
      voiceSpeed: '1.1x (Nhịp điệu nhanh giữ chân người nghe)',
      loudnessLufs: -14.0,
    },
    visualShots: [
      {
        shotIndex: 1,
        framing: 'Macro Detail Close-Up',
        lighting: 'Studio Softbox Glow',
        imagePrompt: 'Macro 8k photo of ultra-minimalist glowing AI desk hub with sleek aluminum texture, modern clean desk setup, cinematic depth of field',
        durationSec: 3,
        textOverlay: '🔥 3 MÓN ĐỒ AI ĐỔI ĐỜI GÓC SETUP',
      },
      {
        shotIndex: 2,
        framing: '45-Degree Desk Top-Down',
        lighting: 'Cyberpunk Neon Accent',
        imagePrompt: 'Top-down desk view showing AI smart Pebble mouse transcribing meeting notes automatically to tablet screen, tidy setup',
        durationSec: 12,
        textOverlay: '1. Chuột AI tự động tóm tắt cuộc họp',
      },
      {
        shotIndex: 3,
        framing: 'Side Split Comparison',
        lighting: 'Studio Softbox Glow',
        imagePrompt: 'Split screen comparing chaotic messy notebook vs crystal-clear AI dashboard summarizer on ultra-wide monitor',
        durationSec: 15,
        textOverlay: '2. Hub USB-C chạy Local LLM bảo mật 100%',
      },
      {
        shotIndex: 4,
        framing: 'POV Handheld Showcase',
        lighting: 'Warm Natural Daylight',
        imagePrompt: 'POV hand holding phone showing exclusive discount badge code AGENTSOS with glowing TikTok Shop button',
        durationSec: 10,
        textOverlay: '🎁 MÃ GIẢM 25%: AGENTSOS (LINK BIO)',
      },
    ],
    renderSettings: {
      resolution: '1080x1920 (Vertical 9:16)',
      fps: 60,
      codec: 'libx264 / yuv420p (+faststart web-optimized)',
      aspectRatio: '9:16',
    },
    governorApproved: true,
    publishedChannels: ['TikTok Shop', 'Shopee Video', 'YouTube Shorts'],
    attributionEpc: '$1.45 / Click',
  });

  const [resultNotice, setResultNotice] = useState<{ revenueGainMinor: number; topic: string } | null>(null);

  const presets = [
    '3 Món Đồ Công Nghệ AI Giúp Tôi Tiết Kiệm 14 Tiếng Mỗi Tuần',
    'Bàn Phím Cơ Không Dây & Đèn Màn Hình Công Thái Học',
    'Thiết Bị Thu Âm AI Khử Nhiễu Cho Creator Livestream',
    'Khóa Học & Công Cụ Tự Động Hóa Workflow Doanh Nghiệp',
  ];

  const handleExecuteEnterpriseWorkflow = async () => {
    if (!topic.trim() || isRunning) return;
    setIsRunning(true);
    setResultNotice(null);
    setLogs([`[00.00s] 🏢 KHỞI ĐỘNG HỘI ĐỒNG PHÊ DUYỆT DOANH NGHIỆP: "${topic}"`]);

    // Stage 1: Phòng Kinh Doanh & Trưởng Phòng Growth Duyệt
    setActiveStage(1);
    setLogs((prev) => [
      ...prev,
      `[00.45s] [CẤP 1 - PHÒNG KINH DOANH]: Trưởng phòng Elena Vance quét 500+ SKU Shopee/TikTok Shop. Xác nhận EPC $1.45, hoa hồng 25.0%. KÝ DUYỆT ĐỀ ÁN.`
    ]);
    await new Promise((r) => setTimeout(r, 700));

    // Stage 2: Phòng Sáng Tạo & Giám Đốc Vận Hành COO Duyệt
    setActiveStage(2);
    setLogs((prev) => [
      ...prev,
      `[01.15s] [CẤP 2 - PHÒNG SÁNG TẠO]: Content Lead AI soạn kịch bản 4 phân cảnh; Giám Đốc COO phê chuẩn chuẩn chất lượng QA (Hook 3s giữ chân 76%). KÝ DUYỆT SẢN PHẨM.`
    ]);
    await new Promise((r) => setTimeout(r, 700));

    // Stage 3: Hội Đồng Giám Đốc (Governor & CFO) Phê Chuẩn Ngân Sách
    setActiveStage(3);
    setLogs((prev) => [
      ...prev,
      `[01.85s] [CẤP 3 - BAN GIÁM ĐỐC & HIẾN PHÁP]: CFO AI duyệt ngân sách $150.00; Governor AI kiểm định Fiduciary & chuẩn thương mại FTC. ĐÓNG DẤU PHÊ CHUẨN HIẾN PHÁP.`
    ]);
    await new Promise((r) => setTimeout(r, 700));

    // Stage 4: Khối Kỹ Thuật & Dựng Phim Render FFmpeg
    setActiveStage(4);
    setLogs((prev) => [
      ...prev,
      `[02.50s] [CẤP 4 - KHỐI KỸ THUẬT]: Marcus Chen cân chỉnh âm thanh -14 LUFS; Liam Rossi render GPU FFmpeg 1080x1920 60FPS. TỰ ĐỘNG XUẤT BẢN ĐA KÊNH.`
    ]);
    await new Promise((r) => setTimeout(r, 700));

    // Stage 5: Phòng Kế Toán Đối Soát & Nạp Kho Bạc
    setActiveStage(5);
    setLogs((prev) => [
      ...prev,
      `[03.10s] [CẤP 5 - PHÒNG KẾ TOÁN]: Analyst & CFO đối soát doanh thu, trích 20% quỹ cổ tức, hạch toán Sổ Cái Kép và nạp thặng dư vào Kho Bạc.`
    ]);
    await new Promise((r) => setTimeout(r, 600));

    const res = await onRunPipeline(topic);
    setIsRunning(false);
    setActiveStage(null);

    if (res.success) {
      setResultNotice({
        revenueGainMinor: res.simulatedRevenueGainMinor,
        topic: topic,
      });
      setLogs((prev) => [
        ...prev,
        `[03.60s] ✅ HOÀN TẤT TOÀN DIỆN! Doanh thu +$${(res.simulatedRevenueGainMinor / 100).toFixed(2)} đã quyết toán vào Kho Bạc an toàn.`
      ]);
    }
  };

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  return (
    <div className="space-y-4 max-w-6xl mx-auto pb-12 text-slate-200">
      {/* 1. Executive Pipeline & Governance Header */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 shadow-md">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400 shrink-0">
            <Building2 className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className="text-sm font-bold text-white tracking-tight">
                Quy Trình Vận Hành &amp; Phê Duyệt Doanh Nghiệp (Multi-Tier Governance)
              </h2>
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-emerald-950/60 text-emerald-400 border border-emerald-800/40">
                <ShieldCheck className="w-3 h-3 text-emerald-400" />
                5 Cấp Phê Duyệt Thực Tế
              </span>
            </div>
            <p className="text-[11px] text-slate-400 mt-0.5">
              Trưởng Phòng Kinh Doanh ➔ Trưởng Phòng Sáng Tạo &amp; COO ➔ Ban Giám Đốc (Governor &amp; CFO) ➔ Khối Kỹ Thuật ➔ Kế Toán Kép
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 text-xs font-mono">
          <span className="bg-slate-950 px-3 py-1.5 rounded-lg border border-slate-800 text-slate-300">
            Ngân sách trần: <strong className="text-emerald-400">$500.00</strong>
          </span>
        </div>
      </div>

      {/* 2. Operational SLAs */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <span className="text-[10px] text-slate-400 block">1. Đề Xuất &amp; Thẩm Định</span>
          <div className="text-xs font-bold text-white mt-1">Trưởng Phòng Growth AI</div>
          <span className="text-[10px] text-blue-400 font-mono">EPC Radar &gt; $1.45/click</span>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <span className="text-[10px] text-slate-400 block">2. Kịch Bản &amp; QA Tiêu Chuẩn</span>
          <div className="text-xs font-bold text-white mt-1">Content Lead &amp; COO AI</div>
          <span className="text-[10px] text-purple-400 font-mono">Hook 3s Retention 76%</span>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <span className="text-[10px] text-slate-400 block">3. Phê Chuẩn Ngân Sách</span>
          <div className="text-xs font-bold text-white mt-1">Hội Đồng Governor &amp; CFO</div>
          <span className="text-[10px] text-emerald-400 font-mono">100% Fiduciary Passed</span>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <span className="text-[10px] text-slate-400 block">4. Render &amp; Quyết Toán</span>
          <div className="text-xs font-bold text-white mt-1">Media Tech &amp; Accounting</div>
          <span className="text-[10px] text-cyan-400 font-mono">FFmpeg 60FPS &bull; Sổ Cái Kép</span>
        </div>
      </div>

      {/* 3. Campaign Proposal & Action Launcher */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 space-y-3 shadow-md">
        <div className="flex items-center justify-between">
          <h3 className="text-xs font-bold text-white flex items-center gap-1.5">
            <FileText className="w-3.5 h-3.5 text-blue-400" />
            Đề Án Chiến Dịch Cần Trình Duyệt &amp; Sản Xuất:
          </h3>
          <span className="text-[10px] text-slate-400 font-mono">Sẵn sàng kích hoạt</span>
        </div>

        <div className="flex flex-col sm:flex-row gap-2">
          <input
            type="text"
            value={topic}
            onChange={(e) => setTopic(e.target.value)}
            placeholder="Nhập tên đề án / ngách sản phẩm..."
            className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3.5 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
          />

          <button
            onClick={handleExecuteEnterpriseWorkflow}
            disabled={isRunning}
            className={`flex items-center justify-center gap-1.5 px-5 py-2 rounded-lg font-bold text-xs text-white shadow-sm transition-all shrink-0 ${
              isRunning
                ? 'bg-slate-800 text-slate-400 cursor-not-allowed border border-slate-700'
                : 'bg-blue-600 hover:bg-blue-500 active:scale-95'
            }`}
          >
            {isRunning ? (
              <>
                <RotateCw className="w-3.5 h-3.5 animate-spin" />
                <span>Hội đồng đang thẩm định...</span>
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5 fill-white" />
                <span>Trình Duyệt &amp; Khởi Động Dây Chuyền</span>
              </>
            )}
          </button>
        </div>

        {/* Presets */}
        <div className="flex flex-wrap items-center gap-1.5 pt-1">
          <span className="text-[10px] text-slate-500 mr-1">Đề án mẫu:</span>
          {presets.map((p, idx) => (
            <button
              key={idx}
              onClick={() => setTopic(p)}
              className={`text-[11px] px-2.5 py-1 rounded-md border transition-all ${
                topic === p
                  ? 'bg-slate-800 text-white border-slate-600 font-medium'
                  : 'bg-slate-950 hover:bg-slate-800/80 text-slate-400 hover:text-white border-slate-800/80'
              }`}
            >
              {p}
            </button>
          ))}
        </div>
      </div>

      {/* 4. 5-Tier Corporate Workflow Canvas */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-1.5">
            <FileCheck className="w-3.5 h-3.5 text-emerald-400" />
            Tiến Trình Phê Duyệt Qua Các Phòng Ban &amp; Giám Đốc
          </h3>
          <span className="text-[10px] text-slate-400 font-mono">Chuẩn ISO Doanh Nghiệp</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-5 gap-2.5">
          {/* Tier 1: Growth Dept */}
          <div className={`p-3 rounded-xl border transition-all flex flex-col justify-between ${
            activeStage === 1
              ? 'bg-slate-900 border-blue-500 ring-1 ring-blue-500/50 shadow-md'
              : 'bg-slate-900/70 border-slate-800'
          }`}>
            <div>
              <div className="flex items-center justify-between text-[10px] font-mono text-blue-400 font-bold mb-1.5">
                <span>01. KINH DOANH</span>
                <Search className="w-3 h-3" />
              </div>
              <div className="flex items-center gap-2 mb-2">
                <img
                  src="https://images.unsplash.com/photo-1519085360753-af0119f7cbe7?w=100&auto=format&fit=crop&q=80"
                  alt="Elena Vance"
                  className="w-7 h-7 rounded-full object-cover border border-slate-700"
                />
                <div className="min-w-0">
                  <div className="text-[11px] font-bold text-white truncate">Elena Vance</div>
                  <div className="text-[9px] text-blue-400">Trưởng Phòng Growth</div>
                </div>
              </div>
              <p className="text-[10px] text-slate-400 leading-relaxed mb-2">
                Thẩm định EPC $1.45, hoa hồng 25%, quét 500+ deal sàn thương mại.
              </p>
            </div>
            <div className="pt-2 border-t border-slate-800/80 text-[10px] font-mono flex items-center justify-between">
              <span className="text-slate-500">Chữ ký:</span>
              <span className="text-emerald-400 font-bold">✓ ĐÃ KÝ DUYỆT</span>
            </div>
          </div>

          {/* Tier 2: Creative & COO Dept */}
          <div className={`p-3 rounded-xl border transition-all flex flex-col justify-between ${
            activeStage === 2
              ? 'bg-slate-900 border-purple-500 ring-1 ring-purple-500/50 shadow-md'
              : 'bg-slate-900/70 border-slate-800'
          }`}>
            <div>
              <div className="flex items-center justify-between text-[10px] font-mono text-purple-400 font-bold mb-1.5">
                <span>02. SÁNG TẠO &amp; COO</span>
                <Video className="w-3 h-3" />
              </div>
              <div className="flex items-center gap-2 mb-2">
                <img
                  src="https://images.unsplash.com/photo-1580489944761-15a19d654956?w=100&auto=format&fit=crop&q=80"
                  alt="COO AI"
                  className="w-7 h-7 rounded-full object-cover border border-slate-700"
                />
                <div className="min-w-0">
                  <div className="text-[11px] font-bold text-white truncate">COO &amp; Content AI</div>
                  <div className="text-[9px] text-purple-400">Giám Đốc Vận Hành</div>
                </div>
              </div>
              <p className="text-[10px] text-slate-400 leading-relaxed mb-2">
                Kịch bản 4 phân cảnh, hook 3s phản trực giác, âm thanh EBU R128.
              </p>
            </div>
            <div className="pt-2 border-t border-slate-800/80 text-[10px] font-mono flex items-center justify-between">
              <span className="text-slate-500">Chữ ký:</span>
              <span className="text-emerald-400 font-bold">✓ ĐÃ KÝ DUYỆT</span>
            </div>
          </div>

          {/* Tier 3: Board & Governor */}
          <div className={`p-3 rounded-xl border transition-all flex flex-col justify-between ${
            activeStage === 3
              ? 'bg-slate-900 border-emerald-500 ring-1 ring-emerald-500/50 shadow-md'
              : 'bg-slate-900/70 border-slate-800'
          }`}>
            <div>
              <div className="flex items-center justify-between text-[10px] font-mono text-emerald-400 font-bold mb-1.5">
                <span>03. BAN GIÁM ĐỐC</span>
                <ShieldCheck className="w-3 h-3" />
              </div>
              <div className="flex items-center gap-2 mb-2">
                <img
                  src="https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=100&auto=format&fit=crop&q=80"
                  alt="Governor AI"
                  className="w-7 h-7 rounded-full object-cover border border-slate-700"
                />
                <div className="min-w-0">
                  <div className="text-[11px] font-bold text-white truncate">Governor &amp; CFO</div>
                  <div className="text-[9px] text-emerald-400">Hội Đồng Fiduciary</div>
                </div>
              </div>
              <p className="text-[10px] text-slate-400 leading-relaxed mb-2">
                Phê chuẩn trần ngân sách $150.00, kiểm định điều lệ FTC an toàn.
              </p>
            </div>
            <div className="pt-2 border-t border-slate-800/80 text-[10px] font-mono flex items-center justify-between">
              <span className="text-slate-500">Dấu mộc:</span>
              <span className="text-emerald-400 font-bold">★ PHÊ CHUẨN</span>
            </div>
          </div>

          {/* Tier 4: Tech & Media Cluster */}
          <div className={`p-3 rounded-xl border transition-all flex flex-col justify-between ${
            activeStage === 4
              ? 'bg-slate-900 border-cyan-500 ring-1 ring-cyan-500/50 shadow-md'
              : 'bg-slate-900/70 border-slate-800'
          }`}>
            <div>
              <div className="flex items-center justify-between text-[10px] font-mono text-cyan-400 font-bold mb-1.5">
                <span>04. KHỐI KỸ THUẬT</span>
                <Cpu className="w-3 h-3" />
              </div>
              <div className="flex items-center gap-2 mb-2">
                <img
                  src="https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=100&auto=format&fit=crop&q=80"
                  alt="Liam Rossi"
                  className="w-7 h-7 rounded-full object-cover border border-slate-700"
                />
                <div className="min-w-0">
                  <div className="text-[11px] font-bold text-white truncate">Liam &amp; Marcus</div>
                  <div className="text-[9px] text-cyan-400">Audio &amp; FFmpeg Lead</div>
                </div>
              </div>
              <p className="text-[10px] text-slate-400 leading-relaxed mb-2">
                Render GPU 1080x1920 60FPS, phát hành tự động kèm mã UTM tracking.
              </p>
            </div>
            <div className="pt-2 border-t border-slate-800/80 text-[10px] font-mono flex items-center justify-between">
              <span className="text-slate-500">Trạng thái:</span>
              <span className="text-cyan-400 font-bold">● ĐÃ XUẤT BẢN</span>
            </div>
          </div>

          {/* Tier 5: Accounting & Treasury */}
          <div className={`p-3 rounded-xl border transition-all flex flex-col justify-between ${
            activeStage === 5
              ? 'bg-slate-900 border-emerald-500 ring-1 ring-emerald-500/50 shadow-md'
              : 'bg-slate-900/70 border-slate-800'
          }`}>
            <div>
              <div className="flex items-center justify-between text-[10px] font-mono text-emerald-400 font-bold mb-1.5">
                <span>05. KẾ TOÁN KHO BẠC</span>
                <Wallet className="w-3 h-3" />
              </div>
              <div className="flex items-center gap-2 mb-2">
                <img
                  src="https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=100&auto=format&fit=crop&q=80"
                  alt="CFO AI"
                  className="w-7 h-7 rounded-full object-cover border border-slate-700"
                />
                <div className="min-w-0">
                  <div className="text-[11px] font-bold text-white truncate">CFO &amp; Analyst AI</div>
                  <div className="text-[9px] text-emerald-400">Kiểm Toán Kế Toán Kép</div>
                </div>
              </div>
              <p className="text-[10px] text-slate-400 leading-relaxed mb-2">
                Đối soát hoa hồng, trích 20% cổ tức, quyết toán nạp kho bạc an toàn.
              </p>
            </div>
            <div className="pt-2 border-t border-slate-800/80 text-[10px] font-mono flex items-center justify-between">
              <span className="text-slate-500">Chứng từ:</span>
              <span className="text-emerald-400 font-bold">#LEDGER-TX</span>
            </div>
          </div>
        </div>
      </div>

      {/* 5. Interactive Deliverables Inspector (Absorbing the entire 5-in-1 Media Suite) */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 space-y-3.5 shadow-md">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
          <div className="flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-blue-400" />
            <h3 className="text-xs font-bold text-white">Hồ Sơ Sản Phẩm &amp; Tài Liệu Bàn Giao Của Chiến Dịch</h3>
          </div>

          {/* Subtabs for Deliverables */}
          <div className="flex items-center gap-1 bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs">
            <button
              onClick={() => setDeliverableTab('script')}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                deliverableTab === 'script' ? 'bg-slate-800 text-white font-semibold shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              Kịch Bản &amp; Hook 3s
            </button>
            <button
              onClick={() => setDeliverableTab('audio')}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                deliverableTab === 'audio' ? 'bg-slate-800 text-white font-semibold shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              Âm Thanh (-14 LUFS)
            </button>
            <button
              onClick={() => setDeliverableTab('visual')}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                deliverableTab === 'visual' ? 'bg-slate-800 text-white font-semibold shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              Phân Cảnh 8K ({production.visualShots.length})
            </button>
            <button
              onClick={() => setDeliverableTab('accounting')}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                deliverableTab === 'accounting' ? 'bg-slate-800 text-white font-semibold shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              Hạch Toán Kế Toán
            </button>
          </div>
        </div>

        {/* Tab 1: Kịch Bản & Hook 3s */}
        {deliverableTab === 'script' && (
          <div className="space-y-2.5 text-xs bg-slate-950/60 p-3.5 rounded-xl border border-slate-800/80">
            <div className="flex items-center justify-between border-b border-slate-800/80 pb-2">
              <span className="font-bold text-white text-xs">{production.copywriting.headline}</span>
              <span className="text-[10px] font-mono text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded border border-emerald-800/40">
                ✓ FTC Compliance Checked
              </span>
            </div>

            <div className="bg-slate-900/90 p-2.5 rounded-lg border border-purple-500/20">
              <span className="text-[10px] text-purple-400 font-bold uppercase block mb-0.5">Hook 3 Giây Giữ Chân:</span>
              <p className="italic text-slate-200 font-medium">"{production.copywriting.hook3s}"</p>
            </div>

            <div className="text-[11px] text-slate-400">
              <span className="text-slate-300 font-semibold">Công thức tâm lý:</span> {production.copywriting.retentionFormula}
            </div>

            <div className="bg-slate-900/60 p-2.5 rounded-lg border border-slate-800 text-[11px] text-slate-300">
              <span className="text-emerald-400 font-semibold block mb-0.5">Lời kêu gọi hành động (Call To Action):</span>
              {production.copywriting.ctaText}
            </div>
          </div>
        )}

        {/* Tab 2: Âm Thanh & Voiceover */}
        {deliverableTab === 'audio' && (
          <div className="space-y-2.5 text-xs bg-slate-950/60 p-3.5 rounded-xl border border-slate-800/80">
            <div className="flex items-center justify-between border-b border-slate-800/80 pb-2">
              <div className="flex items-center gap-2">
                <Music className="w-4 h-4 text-pink-400" />
                <span className="font-bold text-white">{production.audioTrack.title}</span>
              </div>
              <span className="text-[10px] font-mono text-pink-400 bg-pink-950/60 px-2 py-0.5 rounded border border-pink-800/40">
                EBU R128: -14.0 LUFS
              </span>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Thể loại nhạc:</span>
                <span className="text-slate-200 font-medium">{production.audioTrack.genre}</span>
              </div>
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Nhịp độ (BPM):</span>
                <span className="text-slate-200 font-mono font-bold">{production.audioTrack.bpm} BPM</span>
              </div>
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Tông giọng đọc:</span>
                <span className="text-slate-200 font-medium">{production.audioTrack.voiceoverTone}</span>
              </div>
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Tốc độ thoại:</span>
                <span className="text-purple-400 font-medium">{production.audioTrack.voiceSpeed}</span>
              </div>
            </div>
          </div>
        )}

        {/* Tab 3: Phân Cảnh 8K */}
        {deliverableTab === 'visual' && (
          <div className="space-y-2.5 text-xs bg-slate-950/60 p-3.5 rounded-xl border border-slate-800/80">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
              {production.visualShots.map((shot) => (
                <div key={shot.shotIndex} className="bg-slate-900/70 p-2.5 rounded-lg border border-slate-800 space-y-1">
                  <div className="flex items-center justify-between text-[10px] text-slate-400">
                    <span className="font-bold text-white">Phân cảnh #{shot.shotIndex} ({shot.durationSec}s)</span>
                    <span className="text-blue-400 font-mono">{shot.framing}</span>
                  </div>
                  <p className="text-[11px] text-slate-300 italic line-clamp-2">"{shot.imagePrompt}"</p>
                  <div className="text-[10px] text-emerald-400 font-medium pt-1 border-t border-slate-800/60">
                    Overlay: {shot.textOverlay}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Tab 4: Hạch Toán Kế Toán */}
        {deliverableTab === 'accounting' && (
          <div className="space-y-2.5 text-xs bg-slate-950/60 p-3.5 rounded-xl border border-slate-800/80">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-center text-xs">
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Chi phí sản xuất (Debit):</span>
                <span className="font-mono text-amber-400 font-bold">{formatMoney(production.costMinor)}</span>
              </div>
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Doanh thu dự phóng (Credit):</span>
                <span className="font-mono text-emerald-400 font-bold">{formatMoney(production.projectedRevenueMinor)}</span>
              </div>
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Lợi nhuận ròng dự kiến:</span>
                <span className="font-mono text-emerald-300 font-bold">+{formatMoney(production.projectedRevenueMinor - production.costMinor)}</span>
              </div>
              <div className="bg-slate-900/70 p-2 rounded-lg border border-slate-800">
                <span className="text-[10px] text-slate-500 block">Quỹ cổ tức trích lập (20%):</span>
                <span className="font-mono text-purple-400 font-bold">+{formatMoney(Math.round((production.projectedRevenueMinor - production.costMinor) * 0.2))}</span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* 6. Live Execution Terminal Log */}
      {logs.length > 0 && (
        <div className="bg-slate-950 border border-slate-800 rounded-xl p-3.5 font-mono text-[11px] space-y-1 shadow-inner">
          <div className="flex items-center justify-between text-slate-400 border-b border-slate-800 pb-1.5 mb-1.5 text-[10px]">
            <div className="flex items-center gap-1.5">
              <Terminal className="w-3.5 h-3.5 text-blue-400" />
              <span>Biên Bản Họp Hội Đồng &amp; Nhật Ký Phê Duyệt Trực Tiếp</span>
            </div>
            <span>Thời gian thực</span>
          </div>

          <div className="space-y-0.5 text-slate-300 max-h-36 overflow-y-auto">
            {logs.map((log, idx) => (
              <div key={idx} className="flex items-start gap-1">
                <span className="text-blue-400 select-none">&gt;</span>
                <span className={log.includes('✅') ? 'text-emerald-400 font-bold' : ''}>{log}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 7. Result Deliverable Notice */}
      {resultNotice && (
        <div className="bg-emerald-950/20 border border-emerald-500/30 rounded-xl p-4 space-y-2 animate-fadeIn">
          <div className="flex items-center justify-between flex-wrap gap-2">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span className="font-bold text-white text-xs">
                Chiến Dịch Đã Được Hội Đồng Phê Duyệt &amp; Nạp Tiền Vào Kho Bạc: "{resultNotice.topic}"
              </span>
            </div>
            <span className="font-mono font-bold text-emerald-400 text-sm">
              +{formatMoney(resultNotice.revenueGainMinor)}
            </span>
          </div>
          <p className="text-[11px] text-slate-300">
            Hội đồng Ban Giám Đốc (Governor, CEO, CFO, COO) đã đóng dấu quyết toán thành công. Chứng từ đã được lưu vĩnh viễn vào Sổ Cái Kế Toán Kép.
          </p>
        </div>
      )}
    </div>
  );
};
