import React, { useState } from 'react';
import { CompanySnapshot, MediaContentGenerated } from '../types/company';
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
  ArrowRight
} from 'lucide-react';

interface MediaStudioTabProps {
  snapshot: CompanySnapshot;
  onPublishToCycle: (title: string, costMinor: number) => void;
}

export const MediaStudioTab: React.FC<MediaStudioTabProps> = ({
  snapshot,
  onPublishToCycle,
}) => {
  const [category, setCategory] = useState('AI Productivity & Desk Ergonomics');
  const [isGenerating, setIsGenerating] = useState(false);
  const [publishedNotice, setPublishedNotice] = useState(false);
  const [content, setContent] = useState<MediaContentGenerated>({
    title: '3 Món Đồ Công Nghệ AI Giúp Tôi Tiết Kiệm 14 Tiếng Mỗi Tuần',
    hook: 'Đừng mua thêm bàn phím cơ nữa nếu bạn chưa biết 3 món đồ AI này vừa ra mắt trong tháng.',
    scriptOutline: [
      {
        timestamp: '0:00 - 0:03',
        visual: 'Góc quay cận cảnh đế sạc từ tính thông minh phát sáng đồng bộ với màn hình máy tính',
        audio: 'Đừng chi tiền cho phụ kiện công nghệ linh tinh nếu bạn chưa xem qua 3 thiết bị này.',
      },
      {
        timestamp: '0:03 - 0:15',
        visual: 'Quay màn hình thao tác tóm tắt cuộc họp tự động chỉ với 1 nút bấm trên con chuột AI',
        audio: 'Thứ nhất: Con chuột tích hợp AI ghi âm và tự động phân loại to-do list gửi vào Notion sau cuộc gọi.',
      },
      {
        timestamp: '0:15 - 0:35',
        visual: 'So sánh bảng tính tính toán rối rắm với bảng điều khiển trực quan cập nhật tự động',
        audio: 'Thứ hai: Cục hub USB-C chạy mô hình ngôn ngữ cục bộ (Local LLM), không lo lộ dữ liệu bảo mật.',
      },
      {
        timestamp: '0:35 - 0:45',
        visual: 'Chỉ tay vào sticker giảm giá độc quyền trên góc màn hình',
        audio: 'Nhấp ngay link ở bio và nhập mã AGENTSOS để nhận voucher độc quyền 25% trước khi hết slot.',
      },
    ],
    affiliateOffer: 'ErgoTech AI Smart Workspace Hub (Hoa hồng 18.5% qua sàn Awin / TikTok Shop)',
    projectedEpc: '$0.78 / Click',
    callToAction: 'Nhấp link bio và áp mã AGENTSOS để giảm ngay $30.',
    governorComplianceCheck: 'Đạt chuẩn: Bắt buộc gắn thẻ #QuangCao #Affiliate theo quy định minh bạch FTC.',
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
      const res = await fetch('/api/generate-content', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ productCategory: selected }),
      });
      const data = await res.json();
      if (data.content && data.content.title) {
        setContent(data.content);
      }
    } catch (e) {
      console.error('Content generation error:', e);
    } finally {
      setIsGenerating(false);
    }
  };

  const handlePublish = () => {
    onPublishToCycle(content.title, 15000); // $150 publishing cost
    setPublishedNotice(true);
    setTimeout(() => setPublishedNotice(false), 5000);
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto pb-16">
      {/* Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-2 shadow-lg">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Video className="w-5 h-5 text-indigo-400" />
            Media & Affiliate Creator Studio (AI Content Pipeline)
          </h2>
          <span className="text-xs font-mono text-purple-300 bg-purple-500/20 px-2 py-0.5 rounded border border-purple-500/30">
            Automated FFmpeg + Gemini Engine
          </span>
        </div>
        <p className="text-xs text-slate-400 leading-relaxed">
          Giải quyết lỗ hổng "Media worker chỉ là stub". Tại đây, Content Agent và Growth Agent tự động nghiên cứu ngách sản phẩm, 
          sinh kịch bản video short-form có tỷ lệ giữ chân người xem cao, gắn link affiliate và ước tính doanh thu trên mỗi click (EPC).
        </p>
      </div>

      {/* Preset Categories */}
      <div className="space-y-2">
        <span className="text-xs uppercase font-mono text-slate-400 block">Chọn ngách sản phẩm chuyển đổi cao:</span>
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
          placeholder="Nhập ngành hàng hoặc từ khóa sản phẩm để AI sản xuất kịch bản..."
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
          {isGenerating ? 'Đang Tạo Kịch Bản...' : 'Tạo Kịch Bản Mới'}
        </button>
      </div>

      {/* Generated Content Card */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 space-y-5 shadow-xl">
        {/* Title & Hook */}
        <div className="space-y-2 border-b border-slate-800 pb-4">
          <div className="flex items-center gap-2">
            <span className="px-2 py-0.5 rounded text-[11px] font-mono bg-purple-500/20 text-purple-300 border border-purple-500/30">
              Tiêu Đề Video Viral (High CTR)
            </span>
            <span className="text-xs text-slate-500 font-mono">Dành cho TikTok, Reels & Shorts</span>
          </div>
          <h3 className="text-lg md:text-xl font-extrabold text-white">{content.title}</h3>
          <div className="p-3 bg-indigo-950/30 border border-indigo-500/30 rounded-lg text-xs md:text-sm text-indigo-200">
            <strong className="text-indigo-400">Hook 3 Giây Đầu:</strong> "{content.hook}"
          </div>
        </div>

        {/* Script Timeline Breakdown */}
        <div className="space-y-2">
          <span className="text-xs uppercase font-mono text-slate-400 block">Phân Cảnh Kịch Bản (Visual & Audio):</span>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {content.scriptOutline.map((item, idx) => (
              <div key={idx} className="p-3.5 rounded-lg bg-slate-950/80 border border-slate-800 text-xs space-y-1.5">
                <div className="flex items-center justify-between font-mono text-[11px]">
                  <span className="text-cyan-400 font-bold flex items-center gap-1">
                    <Clock className="w-3 h-3" /> {item.timestamp}
                  </span>
                  <span className="text-slate-500">Phân cảnh #{idx + 1}</span>
                </div>
                <div>
                  <strong className="text-slate-300">Hình ảnh:</strong> <span className="text-slate-400">{item.visual}</span>
                </div>
                <div>
                  <strong className="text-slate-300">Lời thoại:</strong> <span className="text-slate-200 italic">"{item.audio}"</span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Commercial & Affiliate Data */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs bg-slate-950/60 p-4 rounded-xl border border-slate-800">
          <div>
            <span className="text-slate-500 text-[10px] uppercase font-mono block">Sản Phẩm Affiliate Tích Hợp</span>
            <span className="font-semibold text-white mt-0.5 block">{content.affiliateOffer}</span>
          </div>
          <div>
            <span className="text-slate-500 text-[10px] uppercase font-mono block">Dự Báo Doanh Thu (Projected EPC)</span>
            <span className="font-bold text-emerald-400 text-sm mt-0.5 block">{content.projectedEpc}</span>
          </div>
          <div>
            <span className="text-slate-500 text-[10px] uppercase font-mono block">Lời Kêu Gọi Hành Động (CTA)</span>
            <span className="text-slate-300 mt-0.5 block">{content.callToAction}</span>
          </div>
        </div>

        {/* Compliance & Action */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pt-2">
          <div className="text-xs text-slate-400 flex items-center gap-2">
            <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0" />
            <span>{content.governorComplianceCheck}</span>
          </div>

          <button
            onClick={handlePublish}
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-500 hover:from-emerald-500 hover:to-teal-400 text-white font-bold text-xs uppercase tracking-wider transition-all shadow-md active:scale-95 shrink-0"
          >
            <Play className="w-4 h-4" />
            Đưa Vào Hàng Đợi Xuất Bản & Trích Ngân Sách ($150)
          </button>
        </div>

        {publishedNotice && (
          <div className="p-3 bg-emerald-950/40 border border-emerald-500/40 rounded-lg text-xs text-emerald-300 flex items-center gap-2 animate-fadeIn">
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            Kịch bản đã được duyệt và đưa vào hàng đợi rendering. Một đề xuất PublishContent vừa được chuyển sang Governor trong cycle tiếp theo!
          </div>
        )}
      </div>
    </div>
  );
};
