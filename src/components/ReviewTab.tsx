import React, { useState } from 'react';
import { 
  CheckCircle2, 
  XCircle, 
  AlertTriangle, 
  Sparkles, 
  Code2, 
  Layers, 
  ShieldAlert, 
  TrendingUp, 
  Zap, 
  Bot, 
  Server, 
  DollarSign, 
  Cpu, 
  ArrowRight,
  BookOpen,
  ChevronDown,
  ChevronUp
} from 'lucide-react';

export const ReviewTab: React.FC = () => {
  const [expandedSection, setExpandedSection] = useState<string | null>('pillar-1');

  const toggleSection = (id: string) => {
    setExpandedSection(expandedSection === id ? null : id);
  };

  const scores = [
    { name: 'Kiến trúc & Đóng gói Modular', score: 8.5, max: 10, color: 'text-emerald-400', bar: 'bg-emerald-500' },
    { name: 'Cơ chế Quản trị & Fiduciary Rules', score: 8.0, max: 10, color: 'text-emerald-400', bar: 'bg-emerald-500' },
    { name: 'Trí tuệ Agent & Phản biện Đa tác tử', score: 5.2, max: 10, color: 'text-amber-400', bar: 'bg-amber-500' },
    { name: 'Thực thi Thực tế & Tooling Thế giới thực', score: 3.5, max: 10, color: 'text-rose-400', bar: 'bg-rose-500' },
    { name: 'Chuỗi Media Pipeline & Bán hàng Affiliate', score: 4.0, max: 10, color: 'text-rose-400', bar: 'bg-rose-500' },
    { name: 'Giao diện Điều hành & Giám sát (UI/UX)', score: 3.8, max: 10, color: 'text-rose-400', bar: 'bg-rose-500' },
  ];

  return (
    <div className="space-y-8 max-w-6xl mx-auto pb-16">
      {/* Hero Banner with Executive Verdict */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-slate-900 via-indigo-950/40 to-slate-900 border border-indigo-500/20 p-6 md:p-8 shadow-2xl">
        <div className="absolute top-0 right-0 w-96 h-96 bg-indigo-500/10 rounded-full blur-3xl pointer-events-none"></div>
        <div className="relative z-10 flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6">
          <div className="space-y-3 max-w-3xl">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-indigo-500/20 border border-indigo-500/30 text-xs font-semibold text-indigo-300">
              <ShieldAlert className="w-3.5 h-3.5" />
              Báo Cáo Kiểm Định Kỹ Thuật Khắc Khe (Technical Due Diligence Audit)
            </div>
            <h1 className="text-2xl md:text-3xl font-extrabold text-white tracking-tight">
              Đánh Giá Toàn Diện Repo: <span className="text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-indigo-300">ninhlee99/company-agents</span>
            </h1>
            <p className="text-slate-300 text-sm md:text-base leading-relaxed">
              Phân tích mổ xẻ kiến trúc Rust, 9 Executive Agents, động cơ kế toán kép, và cơ chế bảo toàn vốn. 
              Chỉ rõ những gì <span className="text-emerald-400 font-semibold">đã làm rất xuất sắc</span>, 
              những điểm nghẽn <span className="text-rose-400 font-semibold">"ảo tưởng tự trị" (Phantom Autonomy)</span> cần khắc phục, 
              và toàn bộ bộ tính năng đã được bổ sung hoàn thiện ngay trên hệ điều hành này.
            </p>
          </div>

          {/* Overall Score Badge */}
          <div className="bg-slate-950/80 border border-slate-800 rounded-xl p-5 text-center min-w-[200px] shadow-inner">
            <span className="text-xs uppercase font-mono tracking-wider text-slate-400 block mb-1">Điểm Tổng Thể</span>
            <div className="text-4xl md:text-5xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-400 to-orange-400">
              5.5 <span className="text-slate-500 text-xl font-normal">/ 10</span>
            </div>
            <div className="mt-2 text-xs font-medium text-amber-300/90 bg-amber-500/10 py-1 px-2.5 rounded-full border border-amber-500/20">
              Kiến Trúc Tốt • Thực Thi Cần Bổ Sung
            </div>
          </div>
        </div>

        {/* Quick Pillar Scores */}
        <div className="mt-8 grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
          {scores.map((s, idx) => (
            <div key={idx} className="bg-slate-900/60 border border-slate-800/80 rounded-lg p-3">
              <span className="text-[11px] text-slate-400 block line-clamp-1">{s.name}</span>
              <div className="flex items-baseline justify-between mt-1">
                <span className={`text-lg font-bold font-mono ${s.color}`}>{s.score}</span>
                <span className="text-[10px] text-slate-500 font-mono">/ {s.max}</span>
              </div>
              <div className="w-full bg-slate-800 h-1.5 rounded-full mt-2 overflow-hidden">
                <div className={`h-full rounded-full ${s.bar}`} style={{ width: `${(s.score / s.max) * 100}%` }}></div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 2-Column: Đã Đạt Được vs Khắc Khe Chưa Đạt Được */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* ĐÃ ĐẠT ĐƯỢC */}
        <div className="bg-slate-900/70 border border-emerald-500/20 rounded-xl p-6 space-y-4">
          <div className="flex items-center gap-2.5 text-emerald-400 font-bold text-lg border-b border-emerald-500/20 pb-3">
            <CheckCircle2 className="w-5 h-5 text-emerald-400" />
            <h3>1. Những Gì Đã Đạt Được (Điểm Sáng Kiến Trúc)</h3>
          </div>

          <div className="space-y-3.5 text-sm text-slate-300">
            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">✓</span>
              <div>
                <strong className="text-white">Kiến trúc Rust Workspace cực kỳ chuẩn mực:</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Tách biệt 11 crates chuyên biệt (economic-core, agent-runtime, company-domain, company-execution, v.v.). Tư duy domain-driven design và compile-time safety xuất sắc.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">✓</span>
              <div>
                <strong className="text-white">Governor Engine & Triết lý Phân quyền Chặt chẽ:</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Agent chỉ có quyền "Đề xuất" (Proposal). Quyền quyết định do Governor thực thi theo luật bất di bất dịch: không cho phép âm tiền, không cho phép tiêu vượt ngân sách còn lại, tự động phanh khi rơi vào Distress/Bankrupt.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">✓</span>
              <div>
                <strong className="text-white">Máy Trạng Thái Kinh Tế (8 Financial State Machines):</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Quy định rành mạch các trạng thái: Active, Growth, Warning, CostControl, Distress, Emergency, Liquidation, Bankrupt dựa trên số ngày runway thực tế.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">✓</span>
              <div>
                <strong className="text-white">Hỗ trợ Hybrid LLM Backend (Local + Remote):</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Mặc định dùng Ollama Qwen3 (1.7B / 4B) để tiết kiệm chi phí và bảo mật dữ liệu doanh nghiệp; có sẵn module mở rộng sang Gemini API và ChatGPT/Claude web relay.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* CHƯA ĐẠT ĐƯỢC - ĐÁNH GIÁ KHẮC KHE */}
        <div className="bg-slate-900/70 border border-rose-500/20 rounded-xl p-6 space-y-4">
          <div className="flex items-center gap-2.5 text-rose-400 font-bold text-lg border-b border-rose-500/20 pb-3">
            <XCircle className="w-5 h-5 text-rose-400" />
            <h3>2. Đánh Giá Khắc Khe Những Gì CHƯA Đạt Được (Critical Flaws)</h3>
          </div>

          <div className="space-y-3.5 text-sm text-slate-300">
            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-rose-500/20 text-rose-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">!</span>
              <div>
                <strong className="text-rose-300">Ảo tưởng Tự trị ("Phantom Autonomy") - Thiếu Tool Execution Thực tế:</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Khi Recruiter đề xuất "ProposeHire", mã nguồn Rust chỉ thêm 1 bản ghi vào in-memory store! Không hề có API đăng bài tuyển dụng, không có kết nối Upwork/LinkedIn, không có phỏng vấn AI. Experiment và Media worker cũng phần lớn là mock.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-rose-500/20 text-rose-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">!</span>
              <div>
                <strong className="text-rose-300">Agents Chạy Cô Lập (Siloed Execution) - Không Có Phản Biện:</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Trong <code className="text-indigo-300 font-mono text-[11px]">runtime.rs</code>, 8 agents chạy đồng thời qua <code className="text-indigo-300 font-mono text-[11px]">join_all</code> nhưng không hề chia sẻ context với nhau. CEO không đối thoại với CFO; Growth đòi bung tiền marketing ngay khi CFO vừa cảnh báo cạn kiệt tiền mặt.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-rose-500/20 text-rose-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">!</span>
              <div>
                <strong className="text-rose-300">Thoái lui mù quáng về Rule tĩnh (Deterministic Degradation):</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Nếu LLM Ollama bị timeout hoặc sinh sai format JSON, hệ thống lập tức lùi về các hàm if/else cố định trong <code className="text-indigo-300 font-mono text-[11px]">roles.rs</code>. Khi đó công ty AI trở thành một shell script rule-based thông thường.
                </p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <span className="w-5 h-5 rounded-full bg-rose-500/20 text-rose-400 flex items-center justify-center shrink-0 text-xs font-bold mt-0.5">!</span>
              <div>
                <strong className="text-rose-300">Giao diện Dashboard Rust quá nghèo nàn:</strong>
                <p className="text-slate-400 text-xs mt-0.5">
                  Giao diện trong <code className="text-indigo-300 font-mono text-[11px]">apps/company-os</code> là bảng HTML string thô sơ, không biểu đồ, không có phòng điều hành can thiệp chiến lược, không thể inject biến cố thị trường để stress-test.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Bảng đối chiếu Tính Năng Công Bố vs Thực Tế Codebase */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-xl overflow-hidden shadow-xl">
        <div className="p-5 bg-slate-950 border-b border-slate-800 flex items-center justify-between">
          <div>
            <h3 className="text-base font-bold text-white flex items-center gap-2">
              <Layers className="w-5 h-5 text-indigo-400" />
              Bảng Ma Trận So Sánh: Tính Năng Thiết Kế vs Hiện Trạng Codebase
            </h3>
            <p className="text-xs text-slate-400 mt-1">So sánh khách quan dựa trên mã nguồn thực tế tại <code className="text-cyan-400 font-mono">github.com/ninhlee99/company-agents</code></p>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="bg-slate-950/80 text-slate-400 uppercase font-mono border-b border-slate-800">
                <th className="p-3.5 pl-5">Module / Agent</th>
                <th className="p-3.5">Mục Tiêu Công Bố (Docs)</th>
                <th className="p-3.5">Hiện Trạng Mã Nguồn (Codebase)</th>
                <th className="p-3.5">Khoảng Trống Cần Khắc Phục</th>
                <th className="p-3.5 pr-5">Đã Triển Khai Tại App Này</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              <tr className="hover:bg-slate-800/30">
                <td className="p-3.5 pl-5 font-semibold text-white flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-indigo-400"></span> Governor
                </td>
                <td className="p-3.5">Bảo đảm tính hợp hiến, ngăn ngừa bội chi và rủi ro</td>
                <td className="p-3.5 text-emerald-400 font-mono">Đã xong 90% (governor.rs)</td>
                <td className="p-3.5 text-amber-300">Chưa có cơ chế Byzantine fault tolerance khi prompt bị injection</td>
                <td className="p-3.5 pr-5 text-emerald-400 font-bold">✓ Full Governor Matrix + Human Override</td>
              </tr>

              <tr className="hover:bg-slate-800/30">
                <td className="p-3.5 pl-5 font-semibold text-white flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-blue-400"></span> CEO & CFO
                </td>
                <td className="p-3.5">Phân bổ vốn, chiến lược tăng trưởng & phòng hộ vỡ nợ</td>
                <td className="p-3.5 text-amber-300 font-mono">Chỉ sinh Proposal JSON đơn tuyến</td>
                <td className="p-3.5 text-rose-300">Không có phản biện đối chất (Debate); tính năng cắt ngân sách chỉ là rule tĩnh</td>
                <td className="p-3.5 pr-5 text-emerald-400 font-bold">✓ Multi-Agent War Room Deliberation</td>
              </tr>

              <tr className="hover:bg-slate-800/30">
                <td className="p-3.5 pl-5 font-semibold text-white flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-purple-400"></span> Content & Media Pipeline
                </td>
                <td className="p-3.5">Tự động dựng video, tạo hook, xuất bản lên TikTok / YT</td>
                <td className="p-3.5 text-rose-400 font-mono">Chỉ có params FFmpeg thô & mock stubs</td>
                <td className="p-3.5 text-rose-300">Chưa sinh kịch bản thực, chưa tích hợp TTS voiceover, chưa upload API</td>
                <td className="p-3.5 pr-5 text-emerald-400 font-bold">✓ AI Creator Studio + Scripting + EPC Live</td>
              </tr>

              <tr className="hover:bg-slate-800/30">
                <td className="p-3.5 pl-5 font-semibold text-white flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-cyan-400"></span> Affiliate Intelligence
                </td>
                <td className="p-3.5">Tìm kiếm offer hoa hồng cao (TikTok Shop, Awin, Amazon)</td>
                <td className="p-3.5 text-amber-300 font-mono">MockProvider + Awin/TikTok stub</td>
                <td className="p-3.5 text-rose-300">Chưa tự động lọc deal theo ROI thực tế; chưa trừ chi phí traffic</td>
                <td className="p-3.5 pr-5 text-emerald-400 font-bold">✓ Niche Product EPC Radar & Auto ROI</td>
              </tr>

              <tr className="hover:bg-slate-800/30">
                <td className="p-3.5 pl-5 font-semibold text-white flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-emerald-400"></span> Sổ Cái Kép (Ledger)
                </td>
                <td className="p-3.5">Hạch toán mọi khoản chi/thu vào double-entry ledger</td>
                <td className="p-3.5 text-emerald-400 font-mono">Tốt (economic-core + company-store)</td>
                <td className="p-3.5 text-amber-300">Chưa trừ chi phí token LLM vào COGS; thiếu giao diện tra cứu Nợ/Có</td>
                <td className="p-3.5 pr-5 text-emerald-400 font-bold">✓ Interactive Double-Entry Journal</td>
              </tr>

              <tr className="hover:bg-slate-800/30">
                <td className="p-3.5 pl-5 font-semibold text-white flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-orange-400"></span> Chaos & Stress Testing
                </td>
                <td className="p-3.5">Kiểm thử sự sống còn khi công ty gặp cú sốc tài chính</td>
                <td className="p-3.5 text-rose-400 font-mono">Chưa có công cụ inject khủng hoảng</td>
                <td className="p-3.5 text-rose-300">Không thể kiểm chứng phản xạ của Governor khi tiền mặt sụt 70%</td>
                <td className="p-3.5 pr-5 text-emerald-400 font-bold">✓ Chaos Engineering Sandbox</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      {/* Chi tiết 4 Giải Pháp & Cải Tiến Đã Implement Ngay Vào Hệ Thống Này */}
      <div className="space-y-4">
        <h3 className="text-xl font-bold text-white flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-cyan-400" />
          Các Nâng Cấp Đột Phá Đã Được Implement Hoàn Thiện Tại App Này
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 hover:border-indigo-500/40 transition-all">
            <div className="flex items-center gap-3 mb-2">
              <div className="p-2 rounded-lg bg-indigo-500/10 text-indigo-400">
                <Bot className="w-5 h-5" />
              </div>
              <h4 className="font-bold text-white text-base">1. Real-time Gemini 3.8 Flash Decision Engine</h4>
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              Thay vì phụ thuộc vào fallback rule cứng, hệ thống tích hợp trực tiếp SDK <code className="text-indigo-300">@google/genai</code>. Mỗi chu kỳ, 8 agents tiếp nhận đúng dữ liệu tài chính thời gian thực và tự suy luận chiến lược kinh tế khả thi với chỉ số confidence, evidence cụ thể.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 hover:border-purple-500/40 transition-all">
            <div className="flex items-center gap-3 mb-2">
              <div className="p-2 rounded-lg bg-purple-500/10 text-purple-400">
                <Zap className="w-5 h-5" />
              </div>
              <h4 className="font-bold text-white text-base">2. Executive War Room (Tranh Luận Đa Tác Tử)</h4>
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              Giải quyết triệt để khuyết điểm "Agents chạy cô lập". Cho phép người vận hành đưa vào bất kỳ tình huống chiến lược nào (cắt giảm ngân sách, mở rộng thị trường, bản quyền nội dung), CEO, CFO, COO và Governor sẽ tranh luận đa chiều để đi đến phán quyết hợp hiến.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 hover:border-emerald-500/40 transition-all">
            <div className="flex items-center gap-3 mb-2">
              <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400">
                <DollarSign className="w-5 h-5" />
              </div>
              <h4 className="font-bold text-white text-base">3. Sổ Cái Kép Trực Quan (Interactive Ledger & Burn Audit)</h4>
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              Tất cả đề xuất được duyệt đều tự động bút toán Nợ (Debit) và Có (Credit) vào Sổ Nhật Ký Chung. Cập nhật ngay bảng cân đối tài sản, số ngày runway và tự động kích hoạt chuyển trạng thái từ Active sang Distress nếu vượt ngưỡng rủi ro.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 hover:border-rose-500/40 transition-all">
            <div className="flex items-center gap-3 mb-2">
              <div className="p-2 rounded-lg bg-rose-500/10 text-rose-400">
                <AlertTriangle className="w-5 h-5" />
              </div>
              <h4 className="font-bold text-white text-base">4. Chaos Simulator & Human Override Console</h4>
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              Mô phỏng tức thì các biến cố khốc liệt: Sụt giảm 60% doanh thu, Phí máy chủ tăng 120%, Mất cọc pháp lý. Cho phép người vận hành duyệt (Human-in-the-loop) các quyết định trọng yếu (Material Proposals) vượt quá thẩm quyền của AI.
            </p>
          </div>
        </div>
      </div>

      {/* Lộ Trình Khuyến Nghị Cho Repo ninhlee99/company-agents */}
      <div className="p-6 rounded-xl bg-gradient-to-r from-slate-950 via-slate-900 to-indigo-950/50 border border-indigo-500/30 space-y-4">
        <h3 className="text-lg font-bold text-white flex items-center gap-2">
          <TrendingUp className="w-5 h-5 text-indigo-400" />
          Lộ Trình Đề Xuất Để Đưa Repo Lên Chuẩn Production (Production-Readiness Checklist)
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs">
          <div className="p-4 rounded-lg bg-slate-900/80 border border-slate-800">
            <span className="font-bold text-indigo-400 block mb-1">GIAI ĐOẠN 1: Tool Binding Thực Tế</span>
            <ul className="space-y-1.5 text-slate-300 list-disc list-inside">
              <li>Tích hợp API Upwork / Deel cho Recruiter</li>
              <li>Tích hợp TikTok Shop & Awin Affiliate live search API</li>
              <li>Chuyển FFmpeg worker từ stub sang render container thực tế</li>
            </ul>
          </div>

          <div className="p-4 rounded-lg bg-slate-900/80 border border-slate-800">
            <span className="font-bold text-purple-400 block mb-1">GIAI ĐOẠN 2: Multi-Agent Consensus Loop</span>
            <ul className="space-y-1.5 text-slate-300 list-disc list-inside">
              <li>Cho phép vòng phản biện: CFO được quyền yêu cầu CEO giải trình proposal trước khi gửi sang Governor</li>
              <li>Hạch toán chi phí LLM token trực tiếp vào tài khoản chi phí OPEX của từng cycle</li>
            </ul>
          </div>

          <div className="p-4 rounded-lg bg-slate-900/80 border border-slate-800">
            <span className="font-bold text-cyan-400 block mb-1">GIAI ĐOẠN 3: Byzantine Security & Hardening</span>
            <ul className="space-y-1.5 text-slate-300 list-disc list-inside">
              <li>Ngăn chặn Prompt Injection làm lệch lạc số liệu hạch toán kép</li>
              <li>Ký số mã hóa Ed25519 cho mọi Execution Receipt trong PostgreSQL</li>
              <li>Mở cổng Webhook cho Human Escalation qua Telegram/Slack</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};
