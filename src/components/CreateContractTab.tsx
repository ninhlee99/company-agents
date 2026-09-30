import React, { useState } from 'react';
import { 
  Sparkles, 
  Video, 
  FileText, 
  Layers, 
  Search, 
  Rocket, 
  CheckCircle2, 
  Clock, 
  ShieldCheck,
  Zap
} from 'lucide-react';

interface CreateContractTabProps {
  onSubmitContract: (orderData: {
    clientName: string;
    clientEmail: string;
    title: string;
    category: 'VideoMarketing' | 'Copywriting' | 'MediaDesign' | 'MarketIntelligence' | 'FullCampaign';
    requirements: string;
    budgetMinor: number;
  }) => Promise<{ success: boolean; message: string }>;
  onNavigateToContracts: () => void;
}

export const CreateContractTab: React.FC<CreateContractTabProps> = ({
  onSubmitContract,
  onNavigateToContracts,
}) => {
  const [category, setCategory] = useState<'VideoMarketing' | 'Copywriting' | 'MediaDesign' | 'MarketIntelligence' | 'FullCampaign'>('VideoMarketing');
  const [title, setTitle] = useState('');
  const [requirements, setRequirements] = useState('');
  const [clientName, setClientName] = useState('Công Ty Đối Tác');
  const [clientEmail, setClientEmail] = useState('partner@enterprise.com');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [successResult, setSuccessResult] = useState<string | null>(null);

  const servicePackages = [
    {
      id: 'VideoMarketing' as const,
      title: 'Video Marketing Viral (TikTok & Reels)',
      priceUSD: 450,
      priceMinor: 45000,
      icon: Video,
      sla: 'Bàn giao trong 24h',
      description: 'Gói trọn gói kịch bản Hook 3s + Voiceover chuẩn EBU R128 + Render 1080x1920 60fps kèm link affiliate.',
      deliverables: ['Kịch bản 4 phân cảnh', 'Prompt ảnh 8K Studio', 'Thông số Render 60fps', 'Bản quyền nhạc thương mại'],
    },
    {
      id: 'Copywriting' as const,
      title: 'Kịch Bản Copywriting Chuyển Đổi Cao',
      priceUSD: 280,
      priceMinor: 28000,
      icon: FileText,
      sla: 'Bàn giao trong 12h',
      description: 'Bộ 5 kịch bản bán hàng direct-response đánh trúng tâm lý và nỗi đau khách hàng, tối ưu thuật toán giữ chân.',
      deliverables: ['5 Mẫu kịch bản short-form', 'Lời thoại Voiceover AI', 'Tiêu đề A/B Test', 'Call-to-Action chuyển đổi'],
    },
    {
      id: 'MediaDesign' as const,
      title: 'Bộ Nhận Diện & Prompt Visual 8K',
      priceUSD: 320,
      priceMinor: 32000,
      icon: Layers,
      sla: 'Bàn giao trong 12h',
      description: 'Bộ prompt hình ảnh studio 8K chi tiết cao, góc chụp Macro Hero Shot nâng tầm giá trị sản phẩm.',
      deliverables: ['4 Góc chụp Macro 8K', 'Lighting setup softbox', 'Preset bảng màu hiện đại', 'Thumbnail CTR cao'],
    },
    {
      id: 'MarketIntelligence' as const,
      title: 'Nghiên Cứu Thị Trường & EPC Radar',
      priceUSD: 350,
      priceMinor: 35000,
      icon: Search,
      sla: 'Bàn giao trong 18h',
      description: 'Báo cáo quét sàn TikTok Shop & Awin, lọc top sản phẩm có tỷ lệ hoàn vốn ROI > 30% và EPC cao nhất.',
      deliverables: ['Top 20 sản phẩm hot', 'Phân tích EPC & Volume', 'Chiến lược định vị ngách', 'Dự phóng doanh thu'],
    },
    {
      id: 'FullCampaign' as const,
      title: 'Chiến Dịch Tự Động Full-Funnel',
      priceUSD: 850,
      priceMinor: 85000,
      icon: Rocket,
      sla: 'Bàn giao trong 36h',
      description: 'Giải pháp tổng thể từ A-Z: Nghiên cứu thị trường ➔ Kịch bản ➔ Sản xuất Video ➔ Kiểm duyệt chất lượng FTC.',
      deliverables: ['Trọn gói 4 khâu khép kín', '3 Biến thể video A/B test', 'Báo cáo kiểm toán Fiduciary', 'Bảo đảm cam kết SLA'],
    },
  ];

  const currentPackage = servicePackages.find((p) => p.id === category) || servicePackages[0];

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim() || !requirements.trim()) {
      alert('Vui lòng nhập đầy đủ tên dự án/sản phẩm và yêu cầu thuê khoán.');
      return;
    }

    setIsSubmitting(true);
    const res = await onSubmitContract({
      clientName,
      clientEmail,
      title,
      category,
      requirements,
      budgetMinor: currentPackage.priceMinor,
    });
    setIsSubmitting(false);

    if (res.success) {
      setSuccessResult(res.message);
    }
  };

  return (
    <div className="space-y-5 max-w-4xl mx-auto pb-12">
      {/* Header Banner */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex items-center justify-between shadow-sm">
        <div>
          <h2 className="text-base font-bold text-white flex items-center gap-2">
            <Zap className="w-5 h-5 text-blue-400" />
            <span>Cổng Thuê Khoán Dịch Vụ Doanh Nghiệp AI</span>
          </h2>
          <p className="text-xs text-slate-400 mt-0.5">
            Giao việc cho hệ sinh thái AI độc lập — AI tự động tiếp nhận, phân công nhân sự và bàn giao sản phẩm
          </p>
        </div>

        <div className="hidden sm:flex items-center gap-1.5 text-xs text-emerald-400 font-mono bg-emerald-500/10 px-2.5 py-1 rounded-lg border border-emerald-500/20">
          <ShieldCheck className="w-3.5 h-3.5" />
          <span>Cam kết SLA 100%</span>
        </div>
      </div>

      {successResult ? (
        <div className="bg-slate-900 border border-emerald-500/40 rounded-xl p-6 text-center space-y-4 shadow-xl animate-fadeIn">
          <div className="w-12 h-12 rounded-full bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center mx-auto text-emerald-400">
            <CheckCircle2 className="w-6 h-6" />
          </div>
          <div className="space-y-1">
            <h3 className="text-sm font-bold text-white">Hợp Đồng Đã Được Tiếp Nhận Thành Công!</h3>
            <p className="text-xs text-slate-300 max-w-md mx-auto">{successResult}</p>
          </div>
          <p className="text-[11px] text-slate-400">
            Hệ thống AI đã tự động phân bổ công việc cho Growth Lead, Content Lead và Media Worker.
          </p>
          <div className="pt-2 flex justify-center gap-3">
            <button
              onClick={() => {
                setSuccessResult(null);
                setTitle('');
                setRequirements('');
              }}
              className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold"
            >
              + Tạo hợp đồng khác
            </button>
            <button
              onClick={onNavigateToContracts}
              className="px-4 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold"
            >
              Xem danh sách hợp đồng ➔
            </button>
          </div>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-5">
          {/* Step 1: Select Service Package */}
          <div className="space-y-2.5">
            <label className="text-xs font-bold text-slate-300 block">
              1. Chọn Gói Dịch Vụ Thuê Khoán:
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {servicePackages.map((pkg) => {
                const Icon = pkg.icon;
                const isSelected = category === pkg.id;

                return (
                  <div
                    key={pkg.id}
                    onClick={() => setCategory(pkg.id)}
                    className={`cursor-pointer p-4 rounded-xl border transition-all ${
                      isSelected
                        ? 'bg-slate-850 border-blue-500 shadow-md ring-1 ring-blue-500/30'
                        : 'bg-slate-900 border-slate-800 hover:border-slate-700'
                    }`}
                  >
                    <div className="flex items-start justify-between">
                      <div className={`p-2 rounded-lg ${isSelected ? 'bg-blue-500/20 text-blue-400' : 'bg-slate-800 text-slate-400'}`}>
                        <Icon className="w-4 h-4" />
                      </div>
                      <span className="text-xs font-bold text-emerald-400 font-mono">
                        ${pkg.priceUSD}
                      </span>
                    </div>

                    <h4 className="text-xs font-bold text-white mt-2.5">{pkg.title}</h4>
                    <p className="text-[11px] text-slate-400 mt-1 leading-relaxed">{pkg.description}</p>

                    <div className="mt-3 pt-2 border-t border-slate-800/80 flex items-center justify-between text-[10px] text-slate-500">
                      <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3 text-slate-400" />
                        <span>{pkg.sla}</span>
                      </span>
                      {isSelected && <span className="text-blue-400 font-semibold">✓ Đang chọn</span>}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Step 2: Contract Details & Requirements */}
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-4">
            <label className="text-xs font-bold text-slate-300 block">
              2. Chi Tiết Yêu Cầu Dự Án:
            </label>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Tên khách hàng / Tổ chức:</label>
                <input
                  type="text"
                  value={clientName}
                  onChange={(e) => setClientName(e.target.value)}
                  placeholder="Ví dụ: Công Ty Cổ Phần Công Nghệ X"
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                  required
                />
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Email nhận thông báo bàn giao:</label>
                <input
                  type="email"
                  value={clientEmail}
                  onChange={(e) => setClientEmail(e.target.value)}
                  placeholder="contact@company.com"
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                  required
                />
              </div>
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">Tiêu đề dự án / Tên sản phẩm cần làm:</label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Ví dụ: Bàn Phím Công Thái Học Không Dây AI Ergonomic Hub 2026"
                className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                required
              />
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                Yêu cầu chi tiết, tệp khách hàng mục tiêu & mong muốn đặc biệt:
              </label>
              <textarea
                value={requirements}
                onChange={(e) => setRequirements(e.target.value)}
                placeholder="Mô tả các tính năng cốt lõi của sản phẩm, đối tượng người xem (vd: dân văn phòng 22-35 tuổi), tone giọng mong muốn (chuyên nghiệp, hài hước hay dồn dập)..."
                rows={4}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                required
              />
            </div>

            {/* Price & Commitment Summary */}
            <div className="bg-slate-950 border border-slate-800 rounded-xl p-3.5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 text-xs">
              <div className="space-y-0.5">
                <span className="text-slate-400 text-[11px]">Tổng chi phí thuê khoán trọn gói:</span>
                <div className="font-bold text-emerald-400 font-mono text-base">
                  ${currentPackage.priceUSD}.00 USD
                </div>
              </div>

              <div className="text-[11px] text-slate-400">
                ⚡ Tự động phân bổ cho 4 phòng ban AI • SLA cam kết: <strong className="text-white">{currentPackage.sla}</strong>
              </div>
            </div>

            {/* Submit Button */}
            <button
              type="submit"
              disabled={isSubmitting}
              className="w-full py-3 rounded-xl bg-blue-600 hover:bg-blue-500 disabled:bg-slate-800 text-white font-bold text-xs shadow-md transition-all active:scale-95 flex items-center justify-center gap-2"
            >
              {isSubmitting ? (
                <span>Đang xử lý & phân bổ cho đội ngũ AI...</span>
              ) : (
                <>
                  <Sparkles className="w-4 h-4" />
                  <span>Ký Hợp Đồng & Giao Việc Cho AI Vận Hành</span>
                </>
              )}
            </button>
          </div>
        </form>
      )}
    </div>
  );
};
