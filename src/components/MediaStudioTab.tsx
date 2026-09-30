import React, { useState } from 'react';
import { CompanySnapshot, FullCreativeProduction } from '../types/company';
import { 
  Video, 
  Sparkles, 
  TrendingUp, 
  ShieldCheck, 
  RotateCw, 
  Play, 
  CheckCircle2, 
  Music, 
  Camera, 
  FileText, 
  Sliders, 
  ArrowRight,
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
  const [activeTab, setActiveTab] = useState<'all' | 'copy' | 'audio' | 'visual' | 'video'>('all');
  const [category, setCategory] = useState('AI Smart Workspace & Desk Gadgets');
  const [isGenerating, setIsGenerating] = useState(false);
  const [publishedNotice, setPublishedNotice] = useState(false);

  const [production, setProduction] = useState<FullCreativeProduction>({
    id: 'prod-init-1',
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
    publishedChannels: ['TikTok Shop', 'YouTube Shorts', 'Instagram Reels'],
    attributionEpc: '$0.84 / Click',
  });

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const handleGenerateProduction = async () => {
    setIsGenerating(true);
    setPublishedNotice(false);
    try {
      const res = await fetch('/api/generate-creative-suite', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ niche: category }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.production) {
          setProduction(data.production);
        }
      }
    } catch (e) {
      console.error(e);
    } finally {
      setIsGenerating(false);
    }
  };

  const handlePublish = () => {
    onPublishToCycle(production.campaignTitle, production.costMinor);
    setPublishedNotice(true);
    setTimeout(() => setPublishedNotice(false), 4000);
  };

  return (
    <div className="space-y-5 pb-12">
      {/* Top Creation Control Bar */}
      <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
          <div>
            <h3 className="text-sm font-semibold text-white">Xưởng Sáng Tạo Media 5-trong-1</h3>
            <p className="text-xs text-slate-400">Tự động sản xuất đồng bộ: Kịch Bản + Âm Nhạc + Hình Ảnh + Video Rendering</p>
          </div>

          <div className="flex items-center gap-2 w-full sm:w-auto">
            <input
              type="text"
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              placeholder="Nhập ngách sản phẩm..."
              className="bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:border-blue-500 w-full sm:w-64"
            />
            <button
              onClick={handleGenerateProduction}
              disabled={isGenerating}
              className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 disabled:bg-slate-800 text-white font-medium text-xs shadow-sm transition-colors whitespace-nowrap"
            >
              <Sparkles className={`w-3.5 h-3.5 ${isGenerating ? 'animate-spin' : ''}`} />
              <span>{isGenerating ? 'Đang sản xuất...' : 'Sản Xuất Mới'}</span>
            </button>
          </div>
        </div>
      </div>

      {publishedNotice && (
        <div className="p-3 rounded-lg bg-emerald-950/60 border border-emerald-800/60 text-emerald-300 text-xs flex items-center gap-2 animate-fadeIn">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>Chiến dịch đã được xuất bản tự động đa nền tảng và hạch toán doanh thu vào sổ cái kép!</span>
        </div>
      )}

      {/* Campaign Summary Card */}
      <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-slate-800 pb-3">
          <div>
            <span className="text-[11px] text-blue-400 font-medium block">Chiến Dịch Đang Tạo</span>
            <h2 className="text-base font-bold text-white mt-0.5">{production.campaignTitle}</h2>
          </div>

          <div className="flex items-center gap-3 text-xs">
            <div className="text-right">
              <span className="text-[10px] text-slate-500 block">Dự phóng doanh thu:</span>
              <span className="font-bold font-mono text-emerald-400 text-sm">
                {formatMoney(production.projectedRevenueMinor)}
              </span>
            </div>
            <button
              onClick={handlePublish}
              className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-xs shadow-sm transition-colors"
            >
              <Play className="w-3.5 h-3.5" /> Xuất Bản Ngay
            </button>
          </div>
        </div>

        {/* Section Navigation Tabs */}
        <div className="flex items-center gap-1 pt-3 border-b border-slate-800 pb-2.5 overflow-x-auto text-xs">
          {[
            { id: 'all', label: 'Tất Cả Khâu', icon: Sliders },
            { id: 'copy', label: '1. Kịch Bản (Copywriter)', icon: FileText },
            { id: 'audio', label: '2. Âm Nhạc (Sound Producer)', icon: Music },
            { id: 'visual', label: '3. Hình Ảnh (Photographer)', icon: Camera },
            { id: 'video', label: '4. Dựng Video (Video Editor)', icon: Video },
          ].map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                onClick={() => setActiveTab(item.id as any)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-medium whitespace-nowrap transition-colors ${
                  activeTab === item.id
                    ? 'bg-slate-800 text-white shadow-sm'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                <Icon className="w-3.5 h-3.5" />
                <span>{item.label}</span>
              </button>
            );
          })}
        </div>

        {/* Dynamic Studio Output Content */}
        <div className="pt-4 space-y-4 text-xs">
          {/* 1. Copywriting Section */}
          {(activeTab === 'all' || activeTab === 'copy') && (
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-2.5">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-white flex items-center gap-1.5">
                  <FileText className="w-4 h-4 text-blue-400" /> Kịch Bản &amp; Hook 3 Giây (Senior Copywriter)
                </span>
                <span className="text-[10px] text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded border border-emerald-800/40">
                  ✓ FTC Verified
                </span>
              </div>

              <div className="space-y-1.5">
                <div className="text-slate-300 font-medium text-[13px]">
                  <strong>Tiêu đề:</strong> {production.copywriting.headline}
                </div>
                <div className="text-blue-300 bg-blue-950/30 p-2.5 rounded-lg border border-blue-900/40">
                  <strong>Hook 3s:</strong> "{production.copywriting.hook3s}"
                </div>
                <div className="text-slate-400">
                  <strong>Công thức giữ chân:</strong> {production.copywriting.retentionFormula}
                </div>
                <div className="text-emerald-300 bg-emerald-950/30 p-2.5 rounded-lg border border-emerald-900/40 font-mono">
                  <strong>Call To Action (CTA):</strong> {production.copywriting.ctaText}
                </div>
              </div>
            </div>
          )}

          {/* 2. Music & Sound Section */}
          {(activeTab === 'all' || activeTab === 'audio') && (
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-2.5">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-white flex items-center gap-1.5">
                  <Music className="w-4 h-4 text-purple-400" /> Thiết Kế Âm Thanh &amp; Nhạc Nền (AI Music Producer)
                </span>
                <span className="text-[10px] text-purple-400 font-mono">
                  {production.audioTrack.loudnessLufs} LUFS (EBU R128)
                </span>
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-[11px]">
                <div className="p-2 rounded bg-slate-900 border border-slate-800">
                  <span className="text-slate-500 block">Thể loại:</span>
                  <span className="font-semibold text-white">{production.audioTrack.genre}</span>
                </div>
                <div className="p-2 rounded bg-slate-900 border border-slate-800">
                  <span className="text-slate-500 block">Nhịp độ (BPM):</span>
                  <span className="font-semibold text-white font-mono">{production.audioTrack.bpm} BPM</span>
                </div>
                <div className="p-2 rounded bg-slate-900 border border-slate-800">
                  <span className="text-slate-500 block">Tông giọng đọc:</span>
                  <span className="font-semibold text-white">{production.audioTrack.voiceoverTone}</span>
                </div>
                <div className="p-2 rounded bg-slate-900 border border-slate-800">
                  <span className="text-slate-500 block">Tốc độ thoại:</span>
                  <span className="font-semibold text-white">{production.audioTrack.voiceSpeed}</span>
                </div>
              </div>
            </div>
          )}

          {/* 3. Visual Photography Section */}
          {(activeTab === 'all' || activeTab === 'visual') && (
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-2.5">
              <span className="font-semibold text-white flex items-center gap-1.5">
                <Camera className="w-4 h-4 text-emerald-400" /> Phân Cảnh Hình Ảnh 8K (Prompt Photographer)
              </span>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                {production.visualShots.map((shot) => (
                  <div key={shot.shotIndex} className="p-2.5 rounded-lg bg-slate-900 border border-slate-800 space-y-1">
                    <div className="flex items-center justify-between text-[11px]">
                      <span className="font-bold text-blue-400">Phân cảnh #{shot.shotIndex} ({shot.durationSec}s)</span>
                      <span className="text-slate-400 font-mono text-[10px]">{shot.framing}</span>
                    </div>
                    <div className="text-slate-300 text-[11px] font-mono bg-slate-950 p-1.5 rounded border border-slate-800/80 line-clamp-2">
                      {shot.imagePrompt}
                    </div>
                    <div className="text-yellow-400 font-semibold text-[10px]">
                      Text Overlay: {shot.textOverlay}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 4. Video Rendering Specs Section */}
          {(activeTab === 'all' || activeTab === 'video') && (
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-2">
              <span className="font-semibold text-white flex items-center gap-1.5">
                <Video className="w-4 h-4 text-pink-400" /> Thông Số Render Kỹ Thuật (Motion Video Director)
              </span>

              <div className="grid grid-cols-3 gap-2 text-center text-[11px]">
                <div className="p-2 rounded bg-slate-900 border border-slate-800">
                  <span className="text-slate-500 block">Độ phân giải</span>
                  <span className="font-mono font-semibold text-white">{production.renderSettings.resolution}</span>
                </div>
                <div className="p-2 rounded bg-slate-900 border border-slate-800">
                  <span className="text-slate-500 block">Khung hình</span>
                  <span className="font-mono font-semibold text-white">{production.renderSettings.fps} FPS</span>
                </div>
                <div className="p-2 rounded bg-slate-900 border border-slate-800">
                  <span className="text-slate-500 block">Bộ nén (Codec)</span>
                  <span className="font-mono font-semibold text-white">libx264 (FFmpeg)</span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
