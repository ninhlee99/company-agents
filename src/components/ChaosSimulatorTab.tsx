import React, { useState } from 'react';
import { CompanySnapshot } from '../types/company';
import { 
  AlertOctagon, 
  Flame, 
  TrendingDown, 
  ServerCrash, 
  ShieldAlert, 
  RefreshCw, 
  CheckCircle2, 
  ArrowRight,
  Zap
} from 'lucide-react';

interface ChaosSimulatorTabProps {
  snapshot: CompanySnapshot;
  onApplyShock: (type: string) => Promise<void>;
}

export const ChaosSimulatorTab: React.FC<ChaosSimulatorTabProps> = ({
  snapshot,
  onApplyShock,
}) => {
  const [loadingType, setLoadingType] = useState<string | null>(null);
  const [lastEvent, setLastEvent] = useState<string | null>(null);

  const handleShock = async (type: string, name: string) => {
    setLoadingType(type);
    try {
      await onApplyShock(type);
      setLastEvent(name);
    } finally {
      setLoadingType(null);
    }
  };

  const shocks = [
    {
      id: 'revenue_crash',
      name: 'Khủng Hoảng Doanh Thu (-60%)',
      desc: 'Nền tảng TikTok Shop quét vi phạm chính sách hàng loạt, hạ tỷ lệ hoa hồng affiliate 60% và giảm lượng traffic tự nhiên.',
      impact: 'Doanh thu giảm từ $9,500 xuống ~$3,800/tháng. Conversion rớt 50%.',
      icon: TrendingDown,
      badge: 'Revenue Shock',
      color: 'border-rose-500/30 hover:border-rose-500 bg-rose-950/20 text-rose-300',
    },
    {
      id: 'burn_spike',
      name: 'Chi Phí Máy Chủ & GPU Đột Biến (+120%)',
      desc: 'Cụm máy chủ render video FFmpeg và chi phí token suy luận LLM bùng nổ ngoài dự kiến do lỗi caching.',
      impact: 'Chi phí hàng tháng tăng vọt từ $6,200 lên ~$13,600/tháng. Đốt quỹ tiền mặt thần tốc.',
      icon: Flame,
      badge: 'Expense Shock',
      color: 'border-orange-500/30 hover:border-orange-500 bg-orange-950/20 text-orange-300',
    },
    {
      id: 'cash_drain',
      name: 'Mất Cọc & Truy Thu Thuế (-65% Tiền Mặt)',
      desc: 'Khoản phạt pháp lý và kiểm toán thuế bất ngờ rút cạn 65% số dư tiền mặt trong kho bạc ngân hàng.',
      impact: 'Tiền mặt bốc hơi xuống còn ~$16,000. Runway tụt thẳng về vùng nguy hiểm Distress.',
      icon: AlertOctagon,
      badge: 'Liquidity Shock',
      color: 'border-amber-500/30 hover:border-amber-500 bg-amber-950/20 text-amber-300',
    },
    {
      id: 'queue_overload',
      name: 'Nghẽn Tác Vụ Nghiêm Trọng (Backlog: 85 items)',
      desc: 'Chiến dịch Creator Affiliate tạo lượng đơn hàng vượt quá khả năng xử lý của worker pods.',
      impact: 'Backlog nhảy vọt lên 85 tác vụ so với công suất 22. Tỷ lệ timeout tăng cao.',
      icon: ServerCrash,
      badge: 'Operational Shock',
      color: 'border-purple-500/30 hover:border-purple-500 bg-purple-950/20 text-purple-300',
    },
    {
      id: 'recovery',
      name: 'Tái Thiết Lập Kho Bạc & Phục Hồi Tăng Trưởng',
      desc: 'Bơm vốn cổ đông mới $50,000, tối ưu hóa toàn bộ cụm worker và đưa công ty về trạng thái Growth thịnh vượng.',
      impact: 'Tiền mặt $50,000, Doanh thu $12,000/tháng, Runway đạt mức an toàn vô hạn.',
      icon: RefreshCw,
      badge: 'System Recovery',
      color: 'border-emerald-500/30 hover:border-emerald-500 bg-emerald-950/20 text-emerald-300',
    },
  ];

  return (
    <div className="space-y-6 max-w-5xl mx-auto pb-16">
      {/* Intro */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-2 shadow-lg">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <ShieldAlert className="w-5 h-5 text-rose-400" />
            Giả Lập Khủng Hoảng (Chaos & Stress Testing Simulator)
          </h2>
          <span className="text-xs font-mono text-slate-400">Kiểm thử tính tự thích ứng của Governor</span>
        </div>
        <p className="text-xs text-slate-400 leading-relaxed">
          Trong repo gốc, dự án chưa có công cụ kiểm thử thực nghiệm khi thị trường biến động xấu. 
          Tại đây, bạn có thể inject trực tiếp các biến cố kinh tế để xem cách Governor chuyển đổi trạng thái từ Active sang Distress/Warning và buộc CFO cắt giảm ngân sách khẩn cấp.
        </p>
      </div>

      {/* Current Resilience State */}
      <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-between gap-4">
        <div>
          <span className="text-[10px] uppercase font-mono text-slate-500 block">Trạng Thái Sức Khỏe Hiện Tại:</span>
          <div className="text-base font-bold text-white mt-0.5 flex items-center gap-2">
            <span className="w-2.5 h-2.5 rounded-full bg-cyan-400 animate-pulse"></span>
            {snapshot.status} Status • Runway: {snapshot.runway_days} ngày • Tiền mặt: ${(snapshot.cash_minor / 100).toLocaleString()}
          </div>
        </div>

        {lastEvent && (
          <div className="text-right text-xs">
            <span className="text-slate-500 block text-[10px] uppercase font-mono">Biến Cố Vừa Inject:</span>
            <span className="text-amber-400 font-semibold">{lastEvent}</span>
          </div>
        )}
      </div>

      {/* Shock Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {shocks.map((s) => {
          const SIcon = s.icon;
          const isLoading = loadingType === s.id;
          return (
            <div
              key={s.id}
              className={`p-5 rounded-xl border ${s.color} transition-all shadow-md flex flex-col justify-between space-y-4`}
            >
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="p-2 rounded-lg bg-black/30 w-fit">
                    <SIcon className="w-5 h-5" />
                  </div>
                  <span className="text-[10px] font-mono uppercase px-2 py-0.5 rounded bg-black/40">
                    {s.badge}
                  </span>
                </div>
                <h3 className="font-bold text-white text-base">{s.name}</h3>
                <p className="text-xs text-slate-300 leading-relaxed">{s.desc}</p>
                <div className="text-[11px] p-2 rounded bg-black/30 text-slate-400 font-mono">
                  <strong className="text-white">Tác động:</strong> {s.impact}
                </div>
              </div>

              <button
                onClick={() => handleShock(s.id, s.name)}
                disabled={Boolean(loadingType)}
                className={`w-full py-2.5 px-4 rounded-lg font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all ${
                  isLoading
                    ? 'bg-slate-800 text-slate-500'
                    : 'bg-white/10 hover:bg-white/20 active:scale-95 text-white'
                }`}
              >
                <Zap className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
                {isLoading ? 'Đang tác động hệ thống...' : 'Inject Biến Cố Này'}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
};
