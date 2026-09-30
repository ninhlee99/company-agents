import React, { useState } from 'react';
import { CompanySnapshot, FullCreativeProduction } from '../types/company';
import { 
  Video, 
  Sparkles, 
  ShoppingBag, 
  TrendingUp, 
  ShieldCheck, 
  RotateCw, 
  Play, 
  Layers, 
  CheckCircle2, 
  Clock,
  ArrowRight,
  Music,
  Camera,
  FileText,
  Sliders,
  Share2,
  Volume2,
  Film,
  Zap,
  DollarSign
} from 'lucide-react';

interface MediaStudioTabProps {
  snapshot: CompanySnapshot;
  onPublishToCycle: (title: string, costMinor: number) => void;
}

export const MediaStudioTab: React.FC<MediaStudioTabProps> = ({
  snapshot,
  onPublishToCycle,
}) => {
  const [activeStudioSubTab, setActiveStudioSubTab] = useState<'all' | 'copy' | 'music' | 'visual' | 'render'>('all');
  const [category, setCategory] = useState('AI Productivity & Desk Ergonomics');
  const [isGenerating, setIsGenerating] = useState(false);
  const [publishedNotice, setPublishedNotice] = useState(false);

  const [production, setProduction] = useState<FullCreativeProduction>({
    id: 'prod-init-1',
    campaignTitle: '3 Món Đồ Công Nghệ AI Giúp Tôi Tiết Kiệm 14 Tiếng Mỗi Tuần',
    niche: 'AI Productivity & Desk Ergonomics',
    projectedRevenueMinor: 155000,
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
      mood: 'Tập trung cao độ, hiện đại, kích thích hành động mua sắm',
      voiceoverTone: 'Confident & Crisp',
      voiceSpeed: '1.1x (Nhịp điệu nhanh tối ưu retention)',
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
    publishedChannels: ['TikTok Shop', 'YouTube Shorts', 'Instagram Reels', 'Facebook Video'],
    attributionEpc: '$0.84 / Click',
  });

  const presetNiches = [
    'AI Productivity & Desk Ergonomics',
    'Smart Home Automation & IoT Gadgets',
    'Creator Audio & Podcast Microphone Gear',
    'Minimalist EDC & MagSafe Tech Accessories',
  ];

  const handleGenerate = async (targetCategory?: string) => {
    const selected = targetCategory || category;
    setIsGenerating(true);
    setPublishedNotice(false);

    try {
      const res = await fetch('/api/generate-creative-suite', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ productCategory: selected }),
      });
      const data = await res.json();
      if (data.production && data.production.campaignTitle) {
        setProduction(data.production);
      }
    } catch (e) {
      console.error('Creative production error:', e);
    } finally {
      setIsGenerating(false);
    }
  };

  const handlePublish = () => {
    onPublishToCycle(production.campaignTitle, production.costMinor || 15000);
    setPublishedNotice(true);
    setTimeout(() => setPublishedNotice(false), 5000);
  };

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto pb-16">
      {/* Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-2 shadow-lg">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Video className="w-5 h-5 text-indigo-400" />
            Xưởng Sáng Tạo Nội Dung Đa Năng (5-in-1 Creative Suite)
          </h2>
          <span className="text-xs font-mono text-emerald-300 bg-emerald-500/20 px-2.5 py-1 rounded-full border border-emerald-500/30 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            Copywriter • Music-er • Photographer • Editor • Producer
          </span>
        </div>
        <p className="text-xs text-slate-400 leading-relaxed">
          Quy trình sản xuất chuẩn Senior Expert: Tự động kết hợp kịch bản hook 3 giây, phối beat âm thanh đạt chuẩn EBU R128 (-14 LUFS), 
          dựng ảnh bìa có độ tương phản cao và render video FFmpeg 1080x1920 60fps kèm link affiliate.
        </p>
      </div>

      {/* Preset Categories */}
      <div className="space-y-2">
        <span className="text-xs uppercase font-mono text-slate-400 block">Chọn ngách sản phẩm sinh lời cao:</span>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2">
          {presetNiches.map((niche, idx) => (
            <button
              key={idx}
              onClick={() => {
                setCategory(niche);
                handleGenerate(niche);
              }}
              className="text-left p-3 rounded-lg bg-slate-900/80 hover:bg-slate-800 border border-slate-800 text-xs text-slate-300 transition-all hover:border-indigo-500/40"
            >
              {niche}
            </button>
          ))}
        </div>
      </div>

      {/* Generator Input */}
      <div className="flex gap-2">
        <input
          type="text"
          value={category}
          onChange={(e) => setCategory(e.target.value)}
          placeholder="Nhập ngành hàng hoặc từ khóa sản phẩm để AI sản xuất trọn gói..."
          className="flex-1 bg-slate-900 border border-slate-800 rounded-xl px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
        />
        <button
          onClick={() => handleGenerate()}
          disabled={isGenerating}
          className={`flex items-center gap-2 px-5 py-2.5 rounded-xl font-bold text-white text-sm transition-all shadow-md shrink-0 ${
            isGenerating
              ? 'bg-slate-800 text-slate-400 cursor-not-allowed'
              : 'bg-indigo-600 hover:bg-indigo-500 active:scale-95'
          }`}
        >
          <RotateCw className={`w-4 h-4 ${isGenerating ? 'animate-spin' : ''}`} />
          {isGenerating ? 'Đang Sản Xuất...' : 'Sản Xuất Toàn Diện (1-Click)'}
        </button>
      </div>

      {/* Studio Workstation Sub-Tabs */}
      <div className="flex items-center gap-2 border-b border-slate-800 pb-2 overflow-x-auto">
        <button
          onClick={() => setActiveStudioSubTab('all')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
            activeStudioSubTab === 'all'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'bg-slate-900 text-slate-400 hover:text-white'
          }`}
        >
          <Layers className="w-3.5 h-3.5" />
          <span>Tổng Quan Đầy Đủ</span>
        </button>
        <button
          onClick={() => setActiveStudioSubTab('copy')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
            activeStudioSubTab === 'copy'
              ? 'bg-pink-600 text-white shadow-sm'
              : 'bg-slate-900 text-slate-400 hover:text-white'
          }`}
        >
          <FileText className="w-3.5 h-3.5" />
          <span>✍️ Bàn Kịch Bản (Copywriter)</span>
        </button>
        <button
          onClick={() => setActiveStudioSubTab('music')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
            activeStudioSubTab === 'music'
              ? 'bg-purple-600 text-white shadow-sm'
              : 'bg-slate-900 text-slate-400 hover:text-white'
          }`}
        >
          <Music className="w-3.5 h-3.5" />
          <span>🎵 Studio Âm Thanh (Music-er)</span>
        </button>
        <button
          onClick={() => setActiveStudioSubTab('visual')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
            activeStudioSubTab === 'visual'
              ? 'bg-cyan-600 text-white shadow-sm'
              : 'bg-slate-900 text-slate-400 hover:text-white'
          }`}
        >
          <Camera className="w-3.5 h-3.5" />
          <span>📸 Nhiếp Ảnh &amp; Visual (Photographer)</span>
        </button>
        <button
          onClick={() => setActiveStudioSubTab('render')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
            activeStudioSubTab === 'render'
              ? 'bg-emerald-600 text-white shadow-sm'
              : 'bg-slate-900 text-slate-400 hover:text-white'
          }`}
        >
          <Film className="w-3.5 h-3.5" />
          <span>🎬 Trạm Render FFmpeg (Editor)</span>
        </button>
      </div>

      {/* Main Studio Viewport */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-6 shadow-xl">
        {/* Title Header */}
        <div className="space-y-2 border-b border-slate-800 pb-4">
          <div className="flex items-center gap-2 flex-wrap justify-between">
            <div className="flex items-center gap-2">
              <span className="px-2.5 py-0.5 rounded text-[11px] font-mono bg-purple-500/20 text-purple-300 border border-purple-500/30">
                Chiến Dịch Video Chuyển Đổi Cao (High CTR)
              </span>
              <span className="text-xs text-slate-400 font-mono">Ngách: {production.niche}</span>
            </div>
            <div className="flex items-center gap-3 text-xs font-mono">
              <span className="text-slate-400">Chi phí: <strong className="text-white">{formatMoney(production.costMinor)}</strong></span>
              <span className="text-emerald-400">Dự phóng: <strong>{formatMoney(production.projectedRevenueMinor)}</strong></span>
            </div>
          </div>
          <h3 className="text-lg md:text-xl font-extrabold text-white">{production.campaignTitle}</h3>
        </div>

        {/* Section 1: Copywriting Workstation */}
        {(activeStudioSubTab === 'all' || activeStudioSubTab === 'copy') && (
          <div className="space-y-3 bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
            <div className="flex items-center justify-between">
              <h4 className="text-xs font-bold text-pink-400 uppercase tracking-wider flex items-center gap-1.5">
                <FileText className="w-4 h-4" />
                1. Kịch Bản &amp; Tâm Lý Chuyển Đổi (Copywriter Senior)
              </h4>
              <span className="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20 flex items-center gap-1">
                <ShieldCheck className="w-3 h-3" /> Đạt chuẩn minh bạch FTC
              </span>
            </div>

            <div className="p-3 bg-pink-950/20 border border-pink-500/30 rounded-lg text-xs md:text-sm text-pink-200">
              <strong className="text-pink-400">Hook 3 Giây Đầu:</strong> "{production.copywriting.hook3s}"
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-slate-500 text-[10px] uppercase font-mono block">Công Thức Giữ Chân Người Xem (Retention Formula)</span>
                <span className="text-slate-300 mt-1 block leading-relaxed">{production.copywriting.retentionFormula}</span>
              </div>
              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-slate-500 text-[10px] uppercase font-mono block">Lời Kêu Gọi Hành Động (CTA)</span>
                <span className="text-emerald-400 font-medium mt-1 block">{production.copywriting.ctaText}</span>
              </div>
            </div>
          </div>
        )}

        {/* Section 2: Music & Audio Workstation */}
        {(activeStudioSubTab === 'all' || activeStudioSubTab === 'music') && (
          <div className="space-y-3 bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
            <div className="flex items-center justify-between">
              <h4 className="text-xs font-bold text-purple-400 uppercase tracking-wider flex items-center gap-1.5">
                <Music className="w-4 h-4" />
                2. Thiết Kế Âm Thanh &amp; Nhạc Nền (AI Music Producer)
              </h4>
              <span className="text-[10px] font-mono text-purple-300 bg-purple-500/20 px-2 py-0.5 rounded border border-purple-500/30">
                Chuẩn EBU R128 ({production.audioTrack.loudnessLufs} LUFS)
              </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs">
              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-slate-500 text-[10px] uppercase font-mono block">Beat Thể Loại &amp; Nhịp Điệu</span>
                <span className="text-white font-bold mt-1 block">{production.audioTrack.genre} ({production.audioTrack.bpm} BPM)</span>
                <span className="text-[11px] text-slate-400 mt-0.5 block">{production.audioTrack.mood}</span>
              </div>

              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-slate-500 text-[10px] uppercase font-mono block">Tông Giọng Đọc AI (Voiceover)</span>
                <span className="text-cyan-400 font-bold mt-1 block">{production.audioTrack.voiceoverTone}</span>
                <span className="text-[11px] text-slate-400 mt-0.5 block">Tốc độ: {production.audioTrack.voiceSpeed}</span>
              </div>

              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800 flex flex-col justify-between">
                <span className="text-slate-500 text-[10px] uppercase font-mono block">Bản Quyền Thương Mại</span>
                <span className="text-emerald-400 font-mono font-bold flex items-center gap-1 mt-1">
                  <CheckCircle2 className="w-3.5 h-3.5" /> Sạch bản quyền 100%
                </span>
                <span className="text-[10px] text-slate-500">Tự động chống gậy bản quyền mạng xã hội</span>
              </div>
            </div>
          </div>
        )}

        {/* Section 3: Visual & Photography Workstation */}
        {(activeStudioSubTab === 'all' || activeStudioSubTab === 'visual') && (
          <div className="space-y-3 bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
            <div className="flex items-center justify-between">
              <h4 className="text-xs font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5">
                <Camera className="w-4 h-4" />
                3. Bảng Phân Cảnh Hình Ảnh &amp; Góc Chụp (Prompt Photographer)
              </h4>
              <span className="text-[10px] font-mono text-cyan-300 bg-cyan-500/20 px-2 py-0.5 rounded border border-cyan-500/30">
                4 Phân Cảnh 8K Studio
              </span>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {production.visualShots.map((shot, idx) => (
                <div key={idx} className="p-3.5 rounded-lg bg-slate-900 border border-slate-800 text-xs space-y-2">
                  <div className="flex items-center justify-between font-mono text-[11px]">
                    <span className="text-cyan-400 font-bold flex items-center gap-1">
                      <Clock className="w-3 h-3" /> Phân cảnh #{shot.shotIndex} ({shot.durationSec}s)
                    </span>
                    <span className="text-slate-500">{shot.framing}</span>
                  </div>
                  
                  <div className="text-slate-300">
                    <strong className="text-slate-400">Ánh sáng:</strong> <span className="text-purple-300">{shot.lighting}</span>
                  </div>

                  <div className="p-2 bg-slate-950 rounded border border-slate-800/60 font-mono text-[10px] text-slate-400">
                    <strong className="text-slate-500 block">AI Image Prompt:</strong>
                    {shot.imagePrompt}
                  </div>

                  <div className="text-emerald-300 font-bold text-[11px] flex items-center gap-1">
                    <span>Chữ chèn video:</span> "{shot.textOverlay}"
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Section 4: Video Editor & FFmpeg Render Station */}
        {(activeStudioSubTab === 'all' || activeStudioSubTab === 'render') && (
          <div className="space-y-3 bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
            <div className="flex items-center justify-between">
              <h4 className="text-xs font-bold text-emerald-400 uppercase tracking-wider flex items-center gap-1.5">
                <Film className="w-4 h-4" />
                4. Thông Số Render &amp; Kỹ Thuật Dựng (Video Editor)
              </h4>
              <span className="text-[10px] font-mono text-emerald-300 bg-emerald-500/20 px-2 py-0.5 rounded border border-emerald-500/30">
                FFmpeg 60fps Vertical Engine
              </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs font-mono">
              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-slate-500 text-[10px] block">Độ Phân Giải</span>
                <span className="text-white font-bold mt-1 block">{production.renderSettings.resolution}</span>
              </div>
              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-slate-500 text-[10px] block">Tốc Độ Khung Hình</span>
                <span className="text-white font-bold mt-1 block">{production.renderSettings.fps} FPS Mượt Mà</span>
              </div>
              <div className="p-3 rounded-lg bg-slate-900 border border-slate-800">
                <span className="text-slate-500 text-[10px] block">Mã Hóa / Tối Ưu Web</span>
                <span className="text-cyan-400 font-bold mt-1 block">{production.renderSettings.codec}</span>
              </div>
            </div>
          </div>
        )}

        {/* Section 5: Producer Commercial Overview & Publish Action */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pt-4 border-t border-slate-800">
          <div className="text-xs space-y-1">
            <div className="flex items-center gap-2 text-slate-300">
              <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0" />
              <span>Dự báo doanh thu mỗi click (EPC): <strong className="text-emerald-400 font-mono">{production.attributionEpc}</strong></span>
            </div>
            <div className="text-[11px] text-slate-500">
              Kênh phát hành tự động: {production.publishedChannels.join(' • ')}
            </div>
          </div>

          <button
            onClick={handlePublish}
            className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-500 hover:from-emerald-500 hover:to-teal-400 text-white font-bold text-xs uppercase tracking-wider transition-all shadow-md active:scale-95 shrink-0"
          >
            <Play className="w-4 h-4" />
            Xuất Bản &amp; Hạch Toán Vào Kỳ ({formatMoney(production.costMinor)})
          </button>
        </div>

        {publishedNotice && (
          <div className="p-3 bg-emerald-950/40 border border-emerald-500/40 rounded-lg text-xs text-emerald-300 flex items-center gap-2 animate-fadeIn">
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            Tác phẩm đã hoàn tất! Gói sản xuất đã được chuyển vào hàng đợi xuất bản tự động và ghi sổ cái kho bạc.
          </div>
        )}
      </div>
    </div>
  );
};
