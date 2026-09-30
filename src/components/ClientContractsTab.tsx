import React, { useState } from 'react';
import { ClientContract } from '../types/company';
import { 
  FileText, 
  CheckCircle2, 
  Clock, 
  Download, 
  Star, 
  Sparkles, 
  ExternalLink,
  ShieldCheck,
  Video,
  Layers,
  ChevronRight,
  TrendingUp,
  Search,
  MessageSquare,
  PlusCircle,
  Briefcase
} from 'lucide-react';

interface ClientContractsTabProps {
  contracts: ClientContract[];
  onAcceptContract: (contractId: string, rating: number, feedback: string) => Promise<void>;
  onNavigateToOrder: () => void;
}

export const ClientContractsTab: React.FC<ClientContractsTabProps> = ({
  contracts,
  onAcceptContract,
  onNavigateToOrder,
}) => {
  const [filter, setFilter] = useState<'ALL' | 'ACTIVE' | 'DELIVERED'>('ALL');
  const [selectedContract, setSelectedContract] = useState<ClientContract | null>(null);
  const [rating, setRating] = useState(5);
  const [feedback, setFeedback] = useState('Sản phẩm đạt chất lượng xuất sắc, đúng hạn.');
  const [isSubmittingAccept, setIsSubmittingAccept] = useState(false);

  const filteredContracts = contracts.filter((c) => {
    if (filter === 'ACTIVE') return c.status !== 'Delivered' && c.status !== 'Completed';
    if (filter === 'DELIVERED') return c.status === 'Delivered' || c.status === 'Completed';
    return true;
  });

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const totalContractValue = contracts.reduce((acc, c) => acc + c.budgetMinor, 0);

  const handleAccept = async () => {
    if (!selectedContract) return;
    setIsSubmittingAccept(true);
    await onAcceptContract(selectedContract.id, rating, feedback);
    setIsSubmittingAccept(false);
    setSelectedContract({
      ...selectedContract,
      status: 'Completed',
      rating,
      clientFeedback: feedback,
    });
  };

  const getCategoryBadge = (category: string) => {
    switch (category) {
      case 'VideoMarketing':
        return { label: 'Video Marketing Viral', color: 'text-blue-400 bg-blue-500/10 border-blue-500/20' };
      case 'Copywriting':
        return { label: 'Copywriting Chuyển Đổi', color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20' };
      case 'MediaDesign':
        return { label: 'Thiết Kế & Prompt 8K', color: 'text-purple-400 bg-purple-500/10 border-purple-500/20' };
      case 'MarketIntelligence':
        return { label: 'Nghiên Cứu Thị Trường', color: 'text-amber-400 bg-amber-500/10 border-amber-500/20' };
      default:
        return { label: 'Chiến Dịch Trọn Gói', color: 'text-cyan-400 bg-cyan-500/10 border-cyan-500/20' };
    }
  };

  return (
    <div className="space-y-5 max-w-5xl mx-auto pb-12">
      {/* Top Banner & Quick Metrics */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="p-2 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-400">
              <Briefcase className="w-5 h-5" />
            </span>
            <div>
              <h2 className="text-base font-bold text-white">Quản Lý Hợp Đồng Kinh Doanh & Thuê Khoán</h2>
              <p className="text-xs text-slate-400">Theo dõi tiến độ AI tự động thực thi hợp đồng khách hàng và doanh thu đã thu về</p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3 w-full md:w-auto">
          <div className="text-right hidden sm:block mr-2">
            <span className="text-[10px] text-slate-400 block">Tổng Doanh Số Hợp Đồng:</span>
            <span className="text-sm font-bold text-emerald-400 font-mono">{formatMoney(totalContractValue)}</span>
          </div>

          <button
            onClick={onNavigateToOrder}
            className="flex-1 md:flex-none flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs transition-colors shadow-sm"
          >
            <Sparkles className="w-3.5 h-3.5" />
            <span>+ Nhập Hợp Đồng / Deal Mới</span>
          </button>
        </div>
      </div>

      {/* Filter Tabs */}
      <div className="flex items-center justify-between gap-2 border-b border-slate-800 pb-3">
        <div className="flex items-center gap-2">
          <button
            onClick={() => setFilter('ALL')}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              filter === 'ALL' ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            Tất cả ({contracts.length})
          </button>
          <button
            onClick={() => setFilter('ACTIVE')}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              filter === 'ACTIVE' ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            Đang thực hiện ({contracts.filter(c => c.status !== 'Delivered' && c.status !== 'Completed').length})
          </button>
          <button
            onClick={() => setFilter('DELIVERED')}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              filter === 'DELIVERED' ? 'bg-slate-800 text-white' : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            Đã hoàn thành ({contracts.filter(c => c.status === 'Delivered' || c.status === 'Completed').length})
          </button>
        </div>
      </div>

      {/* Contracts List */}
      <div className="space-y-3">
        {filteredContracts.length === 0 ? (
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-8 text-center space-y-3">
            <FileText className="w-8 h-8 text-slate-600 mx-auto" />
            <div className="text-sm font-medium text-slate-300">Chưa có hợp đồng nào trong mục này</div>
            <p className="text-xs text-slate-500 max-w-sm mx-auto">
              Hãy nhập thêm hợp đồng hoặc tiếp nhận yêu cầu từ đối tác để công ty AI tự động xử lý và thu tiền về kho bạc.
            </p>
            <button
              onClick={onNavigateToOrder}
              className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold"
            >
              Nhập Hợp Đồng Mới
            </button>
          </div>
        ) : (
          filteredContracts.map((contract) => {
            const badge = getCategoryBadge(contract.category);
            const isDelivered = contract.status === 'Delivered' || contract.status === 'Completed';

            return (
              <div
                key={contract.id}
                className="bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-xl p-4 transition-all shadow-sm space-y-3"
              >
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                  <div className="flex items-center gap-2.5 flex-wrap">
                    <span className="text-xs font-mono font-bold text-white">{contract.contractNumber}</span>
                    <span className={`text-[10px] font-semibold px-2 py-0.5 rounded border ${badge.color}`}>
                      {badge.label}
                    </span>
                    <span className={`text-[10px] font-semibold px-2 py-0.5 rounded ${
                      isDelivered 
                        ? 'bg-emerald-950/60 text-emerald-400 border border-emerald-800/50' 
                        : 'bg-blue-950/60 text-blue-400 border border-blue-800/50'
                    }`}>
                      {isDelivered ? '✓ Đã bàn giao thành phẩm' : '⚡ AI Đang sản xuất'}
                    </span>
                  </div>

                  <div className="text-sm font-bold text-emerald-400 font-mono">
                    {formatMoney(contract.budgetMinor)}
                  </div>
                </div>

                <div>
                  <h3 className="text-sm font-bold text-white">{contract.title}</h3>
                  <p className="text-xs text-slate-400 mt-1 line-clamp-2">{contract.requirements}</p>
                </div>

                {/* Progress Bar & Assigned Steps */}
                <div className="space-y-1.5 pt-1">
                  <div className="flex items-center justify-between text-[11px] text-slate-400">
                    <span className="flex items-center gap-1">
                      <Clock className="w-3 h-3 text-slate-500" />
                      <span>{contract.currentStage}</span>
                    </span>
                    <span className="font-mono font-semibold text-slate-300">{contract.progressPercent}%</span>
                  </div>
                  <div className="w-full bg-slate-950 h-1.5 rounded-full overflow-hidden">
                    <div
                      className={`h-full transition-all duration-500 ${
                        isDelivered ? 'bg-emerald-500' : 'bg-blue-500'
                      }`}
                      style={{ width: `${contract.progressPercent}%` }}
                    />
                  </div>
                </div>

                {/* Action Row */}
                <div className="flex items-center justify-between pt-2 border-t border-slate-800/60 text-xs">
                  <div className="text-[11px] text-slate-500">
                    Khách hàng: <strong className="text-slate-400">{contract.clientName}</strong> • Ngày tạo: {new Date(contract.createdAt).toLocaleDateString('vi-VN')}
                  </div>

                  <button
                    onClick={() => setSelectedContract(contract)}
                    className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-medium transition-colors"
                  >
                    <span>{isDelivered ? 'Xem Thành Phẩm & Nghiệm Thu' : 'Xem Tiến Độ Sản Xuất'}</span>
                    <ChevronRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Deliverable Inspection Modal / Drawer */}
      {selectedContract && (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto p-5 space-y-4 shadow-2xl">
            {/* Header */}
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <div>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-mono font-bold text-white">{selectedContract.contractNumber}</span>
                  <span className="text-xs text-slate-400">• {selectedContract.title}</span>
                </div>
                <h3 className="text-sm font-bold text-white mt-1">Hồ Sơ Bàn Giao Thành Phẩm Hợp Đồng</h3>
              </div>

              <button
                onClick={() => setSelectedContract(null)}
                className="text-slate-400 hover:text-white text-xs px-2 py-1 rounded-md bg-slate-800 hover:bg-slate-700"
              >
                ✕ Đóng
              </button>
            </div>

            {/* Contract Summary */}
            <div className="bg-slate-950 border border-slate-800 rounded-xl p-3.5 space-y-2 text-xs">
              <div className="flex justify-between text-slate-400">
                <span>Doanh thu hợp đồng:</span>
                <strong className="text-emerald-400 font-mono text-sm">{formatMoney(selectedContract.budgetMinor)}</strong>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>Trạng thái thanh toán:</span>
                <span className="text-emerald-400 font-semibold flex items-center gap-1">
                  <CheckCircle2 className="w-3.5 h-3.5" /> Đã ghi nhận vào kho bạc
                </span>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>Điểm kiểm toán chất lượng (FTC & SLA):</span>
                <span className="text-blue-400 font-bold">{selectedContract.deliverables?.qualityScore ?? 98}/100</span>
              </div>
            </div>

            {/* Deliverables Content */}
            {selectedContract.deliverables ? (
              <div className="space-y-3">
                <h4 className="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                  <Sparkles className="w-3.5 h-3.5 text-blue-400" />
                  <span>Chi Tiết Thành Phẩm Được AI Bàn Giao:</span>
                </h4>

                {/* Script */}
                {selectedContract.deliverables.scriptContent && (
                  <div className="bg-slate-950 border border-slate-800 rounded-xl p-3.5 space-y-1.5">
                    <div className="text-[11px] font-bold text-slate-300 flex items-center gap-1">
                      <FileText className="w-3.5 h-3.5 text-blue-400" />
                      <span>Kịch Bản Viral Hook 3s & 4 Phân Cảnh:</span>
                    </div>
                    <pre className="text-xs text-slate-300 whitespace-pre-wrap font-sans bg-slate-900/80 p-2.5 rounded-lg border border-slate-800/80">
                      {selectedContract.deliverables.scriptContent}
                    </pre>
                  </div>
                )}

                {/* Audio & Video Specs */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                  {selectedContract.deliverables.audioVoiceover && (
                    <div className="bg-slate-950 border border-slate-800 rounded-xl p-3 space-y-1">
                      <span className="text-[11px] font-bold text-slate-300 block">Âm Thanh & Lời Thoại:</span>
                      <p className="text-slate-400 text-[11px]">{selectedContract.deliverables.audioVoiceover}</p>
                    </div>
                  )}

                  {selectedContract.deliverables.videoSpecs && (
                    <div className="bg-slate-950 border border-slate-800 rounded-xl p-3 space-y-1">
                      <span className="text-[11px] font-bold text-slate-300 block">Thông Số Video Render:</span>
                      <p className="text-slate-400 text-[11px] font-mono">
                        {selectedContract.deliverables.videoSpecs.resolution} • {selectedContract.deliverables.videoSpecs.fps} • {selectedContract.deliverables.videoSpecs.duration}
                      </p>
                    </div>
                  )}
                </div>

                {/* Visual Prompts */}
                {selectedContract.deliverables.visualPrompts && (
                  <div className="bg-slate-950 border border-slate-800 rounded-xl p-3.5 space-y-2">
                    <span className="text-[11px] font-bold text-slate-300 block">Prompt Hình Ảnh 8K Studio:</span>
                    <div className="space-y-1.5">
                      {selectedContract.deliverables.visualPrompts.map((vp, idx) => (
                        <div key={idx} className="p-2 rounded bg-slate-900 border border-slate-800 text-[11px] text-slate-300 font-mono">
                          {vp}
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Download Zip Action */}
                <div className="p-3 bg-blue-950/20 border border-blue-500/30 rounded-xl flex items-center justify-between">
                  <div className="text-xs text-blue-300">
                    Gói tài nguyên hoàn chỉnh: <strong className="font-mono text-white">{selectedContract.contractNumber}-package.zip</strong>
                  </div>
                  <button
                    onClick={() => alert(`Đang tải gói thành phẩm ${selectedContract.contractNumber}-package.zip`)}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold"
                  >
                    <Download className="w-3.5 h-3.5" />
                    <span>Tải Về (.ZIP)</span>
                  </button>
                </div>
              </div>
            ) : (
              <div className="bg-slate-950 border border-slate-800 rounded-xl p-6 text-center space-y-2 text-xs text-slate-400">
                <Clock className="w-6 h-6 text-blue-400 mx-auto animate-spin" />
                <p>Công ty AI đang trong quá trình thực thi hợp đồng này.</p>
                <p className="text-slate-500 text-[11px]">Thành phẩm sẽ tự động hiển thị tại đây ngay khi các khâu hoàn tất.</p>
              </div>
            )}

            {/* Client Acceptance & Review */}
            {selectedContract.status === 'Delivered' && (
              <div className="bg-slate-950 border border-slate-800 rounded-xl p-4 space-y-3">
                <h4 className="text-xs font-bold text-white flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                  <span>Nghiệm Thu & Đánh Giá Chất Lượng:</span>
                </h4>

                <div className="flex items-center gap-1">
                  {[1, 2, 3, 4, 5].map((star) => (
                    <button
                      key={star}
                      type="button"
                      onClick={() => setRating(star)}
                      className="p-1 hover:scale-110 transition-transform"
                    >
                      <Star className={`w-5 h-5 ${star <= rating ? 'text-amber-400 fill-amber-400' : 'text-slate-600'}`} />
                    </button>
                  ))}
                  <span className="text-xs font-semibold text-slate-300 ml-2">{rating}/5 sao</span>
                </div>

                <textarea
                  value={feedback}
                  onChange={(e) => setFeedback(e.target.value)}
                  placeholder="Nhận xét chất lượng bàn giao..."
                  className="w-full bg-slate-900 border border-slate-800 rounded-lg p-2.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                  rows={2}
                />

                <button
                  onClick={handleAccept}
                  disabled={isSubmittingAccept}
                  className="w-full py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold transition-all shadow-sm"
                >
                  {isSubmittingAccept ? 'Đang gửi...' : '✓ Xác Nhận Nghiệm Thu Hoàn Tất'}
                </button>
              </div>
            )}

            {selectedContract.status === 'Completed' && (
              <div className="bg-emerald-950/20 border border-emerald-500/30 rounded-xl p-3 text-center text-xs text-emerald-300">
                ✓ Hợp đồng đã nghiệm thu hoàn tất với đánh giá {selectedContract.rating ?? 5} sao!
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
