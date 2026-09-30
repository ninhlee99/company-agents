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
  RotateCw,
  Cpu,
  Layers,
  Zap,
  Terminal,
  Clock,
  ExternalLink,
  Flame,
  BarChart3
} from 'lucide-react';

interface AutonomousPipelineTabProps {
  snapshot: CompanySnapshot;
  onRunPipeline: (topic: string) => Promise<{ success: boolean; steps: any[]; simulatedRevenueGainMinor: number }>;
}

interface PipelineHistoryItem {
  id: string;
  topic: string;
  niche: string;
  agents: string[];
  renderTime: string;
  revenueGainMinor: number;
  status: 'Completed' | 'Processing';
  timestamp: string;
}

const DEFAULT_HISTORY: PipelineHistoryItem[] = [
  {
    id: 'pipe-hist-01',
    topic: 'Bàn Phím Cơ Không Dây & Đèn Màn Hình Công Thái Học',
    niche: 'Desk Setup AI',
    agents: ['Growth Lead AI', 'Content Lead AI', 'Liam Rossi', 'Governor AI'],
    renderTime: '1.4s (60 FPS)',
    revenueGainMinor: 80885,
    status: 'Completed',
    timestamp: '15 phút trước',
  },
  {
    id: 'pipe-hist-02',
    topic: 'Mic Thu Âm AI Khử Nhiễu & Đèn Key Light Creator',
    niche: 'Creator Studio',
    agents: ['Growth Lead AI', 'Elena Vance', 'Media Worker', 'CFO AI'],
    renderTime: '1.2s (60 FPS)',
    revenueGainMinor: 65400,
    status: 'Completed',
    timestamp: '42 phút trước',
  },
  {
    id: 'pipe-hist-03',
    topic: 'Khóa Học & Công Cụ Tự Động Hóa Workflow Doanh Nghiệp',
    niche: 'AI SaaS & Tech',
    agents: ['Growth Lead AI', 'Content Lead AI', 'COO AI', 'Governor AI'],
    renderTime: '1.6s (60 FPS)',
    revenueGainMinor: 112000,
    status: 'Completed',
    timestamp: '2 giờ trước',
  },
];

export const AutonomousPipelineTab: React.FC<AutonomousPipelineTabProps> = ({
  snapshot,
  onRunPipeline,
}) => {
  const [topic, setTopic] = useState('Đồ Công Nghệ Smart Home AI');
  const [isRunning, setIsRunning] = useState(false);
  const [activeStep, setActiveStep] = useState<number | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [history, setHistory] = useState<PipelineHistoryItem[]>(DEFAULT_HISTORY);
  const [result, setResult] = useState<{ simulatedRevenueGainMinor: number; steps: any[]; topic: string } | null>(null);

  const presets = [
    { label: 'Đồ Công Nghệ Smart Home AI', tag: 'Smart Home' },
    { label: 'Phụ Kiện Bàn Làm Việc Desk Setup', tag: 'Workspace' },
    { label: 'Thiết Bị Quay Video Tự Động Cho Creator', tag: 'Creator Studio' },
    { label: 'Khóa Học & Công Cụ Tự Động Hóa AI', tag: 'SaaS & Tech' },
    { label: 'Tai Nghe Chống Ồn & Loa Hi-Res', tag: 'Audio Gear' },
  ];

  const handleRun = async () => {
    if (!topic.trim() || isRunning) return;
    setIsRunning(true);
    setResult(null);
    setLogs([`[00.00s] Khởi động Dây Chuyền Tự Trị 4 Khâu: "${topic}"`]);

    // Stage 1
    setActiveStep(1);
    setLogs((prev) => [...prev, `[00.40s] [Khâu 1 - Growth Lead]: Quét 500+ SKU trên TikTok Shop & Amazon... Tìm thấy 14 sản phẩm EPC $1.45+, hoa hồng 22-30%.`]);
    await new Promise((r) => setTimeout(r, 650));

    // Stage 2
    setActiveStep(2);
    setLogs((prev) => [...prev, `[01.10s] [Khâu 2 - Content Lead]: Soạn kịch bản Neuro-Copywriting 4 phân cảnh, hook 3s giữ chân 76% người xem.`]);
    await new Promise((r) => setTimeout(r, 650));

    // Stage 3
    setActiveStep(3);
    setLogs((prev) => [...prev, `[01.75s] [Khâu 3 - Media Worker]: Tổng hợp visual 8K, voiceover EBU R128, render video FFmpeg 1080x1920 60fps.`]);
    await new Promise((r) => setTimeout(r, 650));

    // Stage 4
    setActiveStep(4);
    setLogs((prev) => [...prev, `[02.30s] [Khâu 4 - Governor & CFO]: Đối soát dòng tiền, cập nhật sổ cái kép và ghi nhận doanh thu vào kho bạc.`]);
    await new Promise((r) => setTimeout(r, 650));

    const res = await onRunPipeline(topic);
    setIsRunning(false);
    setActiveStep(null);

    if (res.success) {
      setResult({
        simulatedRevenueGainMinor: res.simulatedRevenueGainMinor,
        steps: res.steps,
        topic: topic,
      });

      const newHistoryItem: PipelineHistoryItem = {
        id: `pipe-hist-${Date.now().toString(36)}`,
        topic: topic,
        niche: 'Affiliate Campaign',
        agents: ['Growth Lead AI', 'Content Lead AI', 'Media Worker', 'Governor AI'],
        renderTime: '1.5s (60 FPS)',
        revenueGainMinor: res.simulatedRevenueGainMinor,
        status: 'Completed',
        timestamp: 'Vừa xong',
      };
      setHistory((prev) => [newHistoryItem, ...prev.slice(0, 4)]);
      setLogs((prev) => [...prev, `[02.80s] ✅ Hoàn tất dây chuyền! Doanh thu +$${(res.simulatedRevenueGainMinor / 100).toFixed(2)} đã chuyển vào kho bạc.`]);
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
      {/* 1. Operational Overview & Pipeline KPIs */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-1">
            <span>Tốc Độ Xử Lý / Render</span>
            <Cpu className="w-3.5 h-3.5 text-blue-400" />
          </div>
          <div className="text-base font-bold text-white font-mono">
            ~1.4s • 60 FPS
          </div>
          <div className="text-[10px] text-blue-400 font-mono mt-0.5">
            FFmpeg GPU Accelerated
          </div>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-1">
            <span>Chỉ Số EPC Định Hướng</span>
            <TrendingUp className="w-3.5 h-3.5 text-emerald-400" />
          </div>
          <div className="text-base font-bold text-emerald-400 font-mono">
            $1.45 / click
          </div>
          <div className="text-[10px] text-emerald-400 font-mono mt-0.5">
            Hoa hồng trung bình: 24.5%
          </div>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-1">
            <span>Tỷ Lệ Giữ Chân (Retention)</span>
            <Flame className="w-3.5 h-3.5 text-pink-400" />
          </div>
          <div className="text-base font-bold text-white font-mono">
            76.4% qua 3s đầu
          </div>
          <div className="text-[10px] text-pink-400 font-mono mt-0.5">
            Neuro-Copywriting Hook
          </div>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-1">
            <span>Kiểm Duyệt Fiduciary</span>
            <ShieldCheck className="w-3.5 h-3.5 text-purple-400" />
          </div>
          <div className="text-base font-bold text-purple-300 font-mono">
            100% Tuân Thủ
          </div>
          <div className="text-[10px] text-purple-400 font-mono mt-0.5">
            Governor Double-Entry Check
          </div>
        </div>
      </div>

      {/* 2. Interactive Launcher Bar */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 space-y-3 shadow-md">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-blue-400" />
            <h3 className="text-xs font-bold text-white">Khởi Động Dây Chuyền Sản Xuất &amp; Phối Hợp Tự Động</h3>
          </div>
          <span className="text-[10px] text-emerald-400 font-mono bg-emerald-950/60 px-2 py-0.5 rounded border border-emerald-800/40">
            ● Hệ thống sẵn sàng 100%
          </span>
        </div>

        {/* Input & Action */}
        <div className="flex flex-col sm:flex-row gap-2">
          <input
            type="text"
            value={topic}
            onChange={(e) => setTopic(e.target.value)}
            placeholder="Nhập ngách sản phẩm hoặc chiến dịch cần chạy..."
            className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3.5 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
          />

          <button
            onClick={handleRun}
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
                <span>Đang phối hợp 4 khâu...</span>
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5 fill-white" />
                <span>Kích Hoạt Dây Chuyền</span>
              </>
            )}
          </button>
        </div>

        {/* Presets */}
        <div className="flex flex-wrap items-center gap-1.5 pt-1">
          <span className="text-[10px] text-slate-500 mr-1">Ngách tiềm năng:</span>
          {presets.map((p, idx) => (
            <button
              key={idx}
              onClick={() => setTopic(p.label)}
              className={`text-[11px] px-2.5 py-1 rounded-md border transition-all ${
                topic === p.label
                  ? 'bg-slate-800 text-white border-slate-600 font-medium'
                  : 'bg-slate-950 hover:bg-slate-800/80 text-slate-400 hover:text-white border-slate-800/80'
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      {/* 3. 4-Stage Architectural Canvas */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
        {/* Stage 1 */}
        <div className={`p-3.5 rounded-xl border transition-all flex flex-col justify-between ${
          activeStep === 1
            ? 'bg-slate-900 border-blue-500/80 shadow-md ring-1 ring-blue-500/40'
            : result || activeStep !== null
            ? 'bg-slate-900/80 border-slate-700'
            : 'bg-slate-900/60 border-slate-800'
        }`}>
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-[10px] font-mono text-blue-400 font-bold">KHÂU 01: RADAR SĂN DEAL</span>
              <Search className="w-3.5 h-3.5 text-blue-400" />
            </div>

            <div className="flex items-center gap-2.5 mb-2.5">
              <img
                src="https://images.unsplash.com/photo-1519085360753-af0119f7cbe7?w=100&auto=format&fit=crop&q=80"
                alt="Growth Lead"
                className="w-8 h-8 rounded-full object-cover border border-slate-700 shrink-0"
              />
              <div className="min-w-0">
                <h4 className="text-xs font-bold text-white truncate">Growth Lead AI</h4>
                <p className="text-[10px] text-blue-400">EPC Radar &amp; Arbitrage</p>
              </div>
            </div>

            <p className="text-[11px] text-slate-400 leading-relaxed mb-3">
              Quét sàn Shopee/TikTok Shop, phân tích EPC &gt; $1.20, hoa hồng 20-30% và tỷ lệ chuyển đổi cao.
            </p>
          </div>

          <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between text-[10px]">
            <span className="text-slate-500">Đầu ra: Deal Sheet &amp; Tags</span>
            {activeStep === 1 ? (
              <span className="text-blue-400 font-mono font-bold animate-pulse">● Đang quét...</span>
            ) : result || (activeStep && activeStep > 1) ? (
              <span className="text-emerald-400 font-mono font-bold flex items-center gap-1">
                <CheckCircle2 className="w-3 h-3" /> Đạt chuẩn
              </span>
            ) : (
              <span className="text-slate-500 font-mono">Sẵn sàng</span>
            )}
          </div>
        </div>

        {/* Stage 2 */}
        <div className={`p-3.5 rounded-xl border transition-all flex flex-col justify-between ${
          activeStep === 2
            ? 'bg-slate-900 border-blue-500/80 shadow-md ring-1 ring-blue-500/40'
            : result || activeStep !== null
            ? 'bg-slate-900/80 border-slate-700'
            : 'bg-slate-900/60 border-slate-800'
        }`}>
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-[10px] font-mono text-purple-400 font-bold">KHÂU 02: KỊCH BẢN VIRAL</span>
              <Video className="w-3.5 h-3.5 text-purple-400" />
            </div>

            <div className="flex items-center gap-2.5 mb-2.5">
              <img
                src="https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=100&auto=format&fit=crop&q=80"
                alt="Content Lead"
                className="w-8 h-8 rounded-full object-cover border border-slate-700 shrink-0"
              />
              <div className="min-w-0">
                <h4 className="text-xs font-bold text-white truncate">Content Lead AI</h4>
                <p className="text-[10px] text-purple-400">Neuro-Copywriting</p>
              </div>
            </div>

            <p className="text-[11px] text-slate-400 leading-relaxed mb-3">
              Soạn kịch bản short-form 4 phân cảnh, tối ưu hook 3s đầu giữ chân người xem và CTA kích thích mua hàng.
            </p>
          </div>

          <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between text-[10px]">
            <span className="text-slate-500">Đầu ra: Script Outline 4K</span>
            {activeStep === 2 ? (
              <span className="text-purple-400 font-mono font-bold animate-pulse">● Đang viết...</span>
            ) : result || (activeStep && activeStep > 2) ? (
              <span className="text-emerald-400 font-mono font-bold flex items-center gap-1">
                <CheckCircle2 className="w-3 h-3" /> Đạt chuẩn
              </span>
            ) : (
              <span className="text-slate-500 font-mono">Sẵn sàng</span>
            )}
          </div>
        </div>

        {/* Stage 3 */}
        <div className={`p-3.5 rounded-xl border transition-all flex flex-col justify-between ${
          activeStep === 3
            ? 'bg-slate-900 border-blue-500/80 shadow-md ring-1 ring-blue-500/40'
            : result || activeStep !== null
            ? 'bg-slate-900/80 border-slate-700'
            : 'bg-slate-900/60 border-slate-800'
        }`}>
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-[10px] font-mono text-cyan-400 font-bold">KHÂU 03: RENDER FFMPEG</span>
              <TrendingUp className="w-3.5 h-3.5 text-cyan-400" />
            </div>

            <div className="flex items-center gap-2.5 mb-2.5">
              <img
                src="https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=100&auto=format&fit=crop&q=80"
                alt="Liam Rossi"
                className="w-8 h-8 rounded-full object-cover border border-slate-700 shrink-0"
              />
              <div className="min-w-0">
                <h4 className="text-xs font-bold text-white truncate">Liam Rossi &amp; COO</h4>
                <p className="text-[10px] text-cyan-400">Media Production Engine</p>
              </div>
            </div>

            <p className="text-[11px] text-slate-400 leading-relaxed mb-3">
              Ghép visual 8K, âm thanh EBU R128 (-14 LUFS) và render xuất bản 1080x1920 60fps h264 tự động.
            </p>
          </div>

          <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between text-[10px]">
            <span className="text-slate-500">Đầu ra: Video MP4 60FPS</span>
            {activeStep === 3 ? (
              <span className="text-cyan-400 font-mono font-bold animate-pulse">● Đang render...</span>
            ) : result || (activeStep && activeStep > 3) ? (
              <span className="text-emerald-400 font-mono font-bold flex items-center gap-1">
                <CheckCircle2 className="w-3 h-3" /> Đạt chuẩn
              </span>
            ) : (
              <span className="text-slate-500 font-mono">Sẵn sàng</span>
            )}
          </div>
        </div>

        {/* Stage 4 */}
        <div className={`p-3.5 rounded-xl border transition-all flex flex-col justify-between ${
          activeStep === 4
            ? 'bg-slate-900 border-emerald-500/80 shadow-md ring-1 ring-emerald-500/40'
            : result || activeStep !== null
            ? 'bg-slate-900/80 border-slate-700'
            : 'bg-slate-900/60 border-slate-800'
        }`}>
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-[10px] font-mono text-emerald-400 font-bold">KHÂU 04: KẾ TOÁN KÉP</span>
              <Wallet className="w-3.5 h-3.5 text-emerald-400" />
            </div>

            <div className="flex items-center gap-2.5 mb-2.5">
              <img
                src="https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=100&auto=format&fit=crop&q=80"
                alt="Governor AI"
                className="w-8 h-8 rounded-full object-cover border border-slate-700 shrink-0"
              />
              <div className="min-w-0">
                <h4 className="text-xs font-bold text-white truncate">Governor &amp; CFO AI</h4>
                <p className="text-[10px] text-emerald-400">Fiduciary Settlement</p>
              </div>
            </div>

            <p className="text-[11px] text-slate-400 leading-relaxed mb-3">
              Đối soát doanh thu, trích 20% cổ tức và nạp thặng dư vào quỹ kho bạc theo nguyên tắc Double-Entry.
            </p>
          </div>

          <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between text-[10px]">
            <span className="text-slate-500">Đầu ra: Double-Entry Ledger</span>
            {activeStep === 4 ? (
              <span className="text-emerald-400 font-mono font-bold animate-pulse">● Đang đối soát...</span>
            ) : result ? (
              <span className="text-emerald-400 font-mono font-bold flex items-center gap-1">
                <CheckCircle2 className="w-3 h-3" /> Đã nạp kho bạc
              </span>
            ) : (
              <span className="text-slate-500 font-mono">Sẵn sàng</span>
            )}
          </div>
        </div>
      </div>

      {/* 4. Live Execution Terminal Log */}
      {logs.length > 0 && (
        <div className="bg-slate-950 border border-slate-800 rounded-xl p-3 font-mono text-[11px] space-y-1 shadow-inner">
          <div className="flex items-center justify-between text-slate-400 border-b border-slate-800 pb-1.5 mb-1.5 text-[10px]">
            <div className="flex items-center gap-1.5">
              <Terminal className="w-3 h-3 text-blue-400" />
              <span>Nhật Ký Thực Thi Dây Chuyền Trực Tiếp</span>
            </div>
            <span>Thời gian thực</span>
          </div>

          <div className="space-y-0.5 text-slate-300 max-h-32 overflow-y-auto">
            {logs.map((log, idx) => (
              <div key={idx} className="flex items-start gap-1">
                <span className="text-blue-400 select-none">&gt;</span>
                <span className={log.includes('✅') ? 'text-emerald-400 font-bold' : ''}>{log}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 5. Result Deliverable Card */}
      {result && (
        <div className="bg-emerald-950/20 border border-emerald-500/30 rounded-xl p-4 space-y-2.5 animate-fadeIn">
          <div className="flex items-center justify-between flex-wrap gap-2 border-b border-emerald-500/20 pb-2">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span className="font-bold text-white text-xs">
                Chiến Dịch Sản Xuất Thành Công: "{result.topic}"
              </span>
            </div>
            <span className="font-mono font-bold text-emerald-400 text-sm">
              +{formatMoney(result.simulatedRevenueGainMinor)} Doanh Thu
            </span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 text-xs">
            <div className="bg-slate-950/60 p-2 rounded-lg border border-slate-800">
              <span className="text-[10px] text-slate-500 block">Định Dạng Video:</span>
              <span className="font-mono text-slate-200">1080x1920 60FPS • H.264</span>
            </div>
            <div className="bg-slate-950/60 p-2 rounded-lg border border-slate-800">
              <span className="text-[10px] text-slate-500 block">Trạng Thái Xuất Bản:</span>
              <span className="text-emerald-400 font-medium">TikTok Shop &amp; YouTube Shorts</span>
            </div>
            <div className="bg-slate-950/60 p-2 rounded-lg border border-slate-800">
              <span className="text-[10px] text-slate-500 block">Chứng Từ Kho Bạc:</span>
              <span className="font-mono text-purple-400">#LEDGER-AUTOPILOT-TX</span>
            </div>
          </div>
        </div>
      )}

      {/* 6. Recent Production History */}
      <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5 space-y-3">
        <div className="flex items-center justify-between border-b border-slate-800 pb-2">
          <div className="flex items-center gap-2">
            <BarChart3 className="w-3.5 h-3.5 text-blue-400" />
            <h4 className="text-xs font-bold text-white">Lịch Sử Các Đợt Sản Xuất Gần Đây ({history.length})</h4>
          </div>
          <span className="text-[10px] text-slate-400 font-mono">Tự động lưu trữ</span>
        </div>

        <div className="space-y-2">
          {history.map((item) => (
            <div key={item.id} className="p-2.5 rounded-lg bg-slate-950/60 border border-slate-800/80 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 text-xs hover:border-slate-700 transition-colors">
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="font-bold text-white text-xs">{item.topic}</span>
                  <span className="px-1.5 py-0.2 rounded text-[9px] bg-blue-950/60 text-blue-400 border border-blue-800/40">
                    {item.niche}
                  </span>
                </div>
                <div className="text-[10px] text-slate-400 mt-0.5 flex items-center gap-2">
                  <span>Nhân sự: {item.agents.join(', ')}</span>
                  <span>• Render: {item.renderTime}</span>
                </div>
              </div>

              <div className="flex items-center gap-3 shrink-0">
                <span className="font-mono font-bold text-emerald-400 text-xs">
                  +{formatMoney(item.revenueGainMinor)}
                </span>
                <span className="text-[10px] text-slate-500 font-mono">{item.timestamp}</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
