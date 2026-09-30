import React, { useState } from 'react';
import { CompanySnapshot } from '../types/company';
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
  RotateCw
} from 'lucide-react';

interface AutonomousPipelineTabProps {
  snapshot: CompanySnapshot;
  onRunPipeline: (topic: string) => Promise<{ success: boolean; steps: any[]; simulatedRevenueGainMinor: number }>;
}

export const AutonomousPipelineTab: React.FC<AutonomousPipelineTabProps> = ({
  snapshot,
  onRunPipeline,
}) => {
  const [topic, setTopic] = useState('Đồ Công Nghệ Smart Home AI');
  const [isRunning, setIsRunning] = useState(false);
  const [activeStep, setActiveStep] = useState<number | null>(null);
  const [result, setResult] = useState<{ simulatedRevenueGainMinor: number; steps: any[] } | null>(null);

  const presets = [
    'Đồ Công Nghệ Smart Home AI',
    'Phụ Kiện Bàn Làm Việc Desk Setup',
    'Thiết Bị Quay Video Tự Động Cho Creator',
    'Khóa Học & Công Cụ Tự Động Hóa AI',
  ];

  const handleRun = async () => {
    if (!topic.trim() || isRunning) return;
    setIsRunning(true);
    setResult(null);

    // Simulate animated step progression
    setActiveStep(1);
    await new Promise((r) => setTimeout(r, 600));
    setActiveStep(2);
    await new Promise((r) => setTimeout(r, 600));
    setActiveStep(3);
    await new Promise((r) => setTimeout(r, 600));
    setActiveStep(4);

    const res = await onRunPipeline(topic);
    setIsRunning(false);
    if (res.success) {
      setResult({
        simulatedRevenueGainMinor: res.simulatedRevenueGainMinor,
        steps: res.steps,
      });
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
    <div className="space-y-4 max-w-4xl mx-auto pb-10">
      {/* Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-sm">
        <div className="flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-purple-400" />
          <div>
            <h2 className="text-base font-bold text-white">Dây Chuyền Phối Hợp Tự Động</h2>
            <p className="text-xs text-slate-400">Các Agent tự động bàn giao công việc qua 4 khâu khép kín</p>
          </div>
        </div>

        <div className="flex items-center gap-1.5 text-xs text-emerald-400 font-mono bg-emerald-500/10 px-2.5 py-1 rounded-lg border border-emerald-500/20">
          <ShieldCheck className="w-3.5 h-3.5" />
          <span>Tự kiểm soát ngân sách</span>
        </div>
      </div>

      {/* Input & Run Form */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3">
        <label className="text-xs text-slate-300 block font-medium">Chọn hoặc nhập chủ đề sản phẩm:</label>
        
        <div className="flex gap-2">
          <input
            type="text"
            value={topic}
            onChange={(e) => setTopic(e.target.value)}
            placeholder="Nhập ngách sản phẩm..."
            className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3.5 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
          />
          <button
            onClick={handleRun}
            disabled={isRunning}
            className={`flex items-center gap-1.5 px-5 py-2 rounded-lg font-bold text-xs text-white shadow-md transition-all shrink-0 ${
              isRunning
                ? 'bg-slate-800 text-slate-400 cursor-not-allowed'
                : 'bg-emerald-600 hover:bg-emerald-500 active:scale-95'
            }`}
          >
            {isRunning ? (
              <>
                <RotateCw className="w-3.5 h-3.5 animate-spin" />
                <span>Đang phối hợp...</span>
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5" />
                <span>Kích Hoạt Dây Chuyền</span>
              </>
            )}
          </button>
        </div>

        {/* Quick Presets */}
        <div className="flex flex-wrap gap-1.5 pt-1">
          {presets.map((p) => (
            <button
              key={p}
              onClick={() => setTopic(p)}
              className="text-[11px] px-2.5 py-1 rounded-md bg-slate-950 hover:bg-slate-800 text-slate-400 hover:text-white border border-slate-800 transition-all"
            >
              {p}
            </button>
          ))}
        </div>
      </div>

      {/* 4 Pipeline Stages Visualization */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
        {/* Stage 1 */}
        <div className={`p-3.5 rounded-xl border transition-all ${
          activeStep === 1 || (result && activeStep === null)
            ? 'bg-indigo-950/30 border-indigo-500/60 shadow-md'
            : 'bg-slate-900 border-slate-800'
        }`}>
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-mono text-indigo-400 font-bold">Khâu 1: Nghiên Cứu</span>
            <Search className="w-4 h-4 text-indigo-400" />
          </div>
          <h4 className="font-bold text-white text-xs mt-1">Growth &amp; Analyst</h4>
          <p className="text-[11px] text-slate-400 mt-1">
            Quét sàn affiliate, chọn sản phẩm có tỷ lệ chuyển đổi cao &amp; hoa hồng tốt.
          </p>
          {(activeStep === 1 || result) && (
            <div className="mt-2 text-[10px] text-emerald-400 font-mono flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" /> Hoàn tất
            </div>
          )}
        </div>

        {/* Stage 2 */}
        <div className={`p-3.5 rounded-xl border transition-all ${
          activeStep === 2 || (result && activeStep === null)
            ? 'bg-pink-950/30 border-pink-500/60 shadow-md'
            : 'bg-slate-900 border-slate-800'
        }`}>
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-mono text-pink-400 font-bold">Khâu 2: Sáng Tạo</span>
            <Video className="w-4 h-4 text-pink-400" />
          </div>
          <h4 className="font-bold text-white text-xs mt-1">Content Lead</h4>
          <p className="text-[11px] text-slate-400 mt-1">
            Tạo hook 3 giây đầu giữ chân người xem và kịch bản 4 phân cảnh bán hàng.
          </p>
          {(activeStep === 2 || result) && (
            <div className="mt-2 text-[10px] text-emerald-400 font-mono flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" /> Hoàn tất
            </div>
          )}
        </div>

        {/* Stage 3 */}
        <div className={`p-3.5 rounded-xl border transition-all ${
          activeStep === 3 || (result && activeStep === null)
            ? 'bg-cyan-950/30 border-cyan-500/60 shadow-md'
            : 'bg-slate-900 border-slate-800'
        }`}>
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-mono text-cyan-400 font-bold">Khâu 3: Dựng Video</span>
            <TrendingUp className="w-4 h-4 text-cyan-400" />
          </div>
          <h4 className="font-bold text-white text-xs mt-1">Media Worker &amp; COO</h4>
          <p className="text-[11px] text-slate-400 mt-1">
            Tổng hợp hình ảnh, chạy lệnh render FFmpeg và tạo link theo dõi hoa hồng.
          </p>
          {(activeStep === 3 || result) && (
            <div className="mt-2 text-[10px] text-emerald-400 font-mono flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" /> Hoàn tất
            </div>
          )}
        </div>

        {/* Stage 4 */}
        <div className={`p-3.5 rounded-xl border transition-all ${
          activeStep === 4 || (result && activeStep === null)
            ? 'bg-emerald-950/30 border-emerald-500/60 shadow-md'
            : 'bg-slate-900 border-slate-800'
        }`}>
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-mono text-emerald-400 font-bold">Khâu 4: Kế Toán</span>
            <Wallet className="w-4 h-4 text-emerald-400" />
          </div>
          <h4 className="font-bold text-white text-xs mt-1">Governor &amp; CFO</h4>
          <p className="text-[11px] text-slate-400 mt-1">
            Ghi nhận doanh thu đối soát vào kho bạc và cập nhật sổ cái kép minh bạch.
          </p>
          {(activeStep === 4 || result) && (
            <div className="mt-2 text-[10px] text-emerald-400 font-mono flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" /> Hoàn tất
            </div>
          )}
        </div>
      </div>

      {/* Result Card */}
      {result && (
        <div className="bg-emerald-950/20 border border-emerald-500/30 rounded-xl p-4 space-y-2 animate-fadeIn">
          <div className="flex items-center justify-between">
            <span className="font-bold text-white text-xs flex items-center gap-1.5">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              Dây chuyền hoàn tất thành công!
            </span>
            <span className="font-mono font-bold text-emerald-400 text-sm">
              +{formatMoney(result.simulatedRevenueGainMinor)}
            </span>
          </div>

          <div className="text-[11px] text-slate-300">
            Số tiền đã được tự động cộng vào kho bạc và ghi sổ cái kép. Các Agent liên quan đã được tăng chỉ số năng suất.
          </div>
        </div>
      )}
    </div>
  );
};
