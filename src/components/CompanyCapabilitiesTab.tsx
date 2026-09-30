import React from 'react';
import { CompanySnapshot, CustomAgent, OfficeActivityEvent } from '../types/company';
import { 
  Building2, 
  Users, 
  ShieldCheck, 
  Zap, 
  CheckCircle2, 
  Activity, 
  BrainCircuit, 
  Sparkles,
  TrendingUp,
  Cpu,
  Lock
} from 'lucide-react';

interface CompanyCapabilitiesTabProps {
  snapshot: CompanySnapshot;
  agents: CustomAgent[];
  officeActivities: OfficeActivityEvent[];
}

export const CompanyCapabilitiesTab: React.FC<CompanyCapabilitiesTabProps> = ({
  snapshot,
  agents,
  officeActivities,
}) => {
  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: snapshot.currency || 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  return (
    <div className="space-y-5 max-w-5xl mx-auto pb-12">
      {/* Overview Banner */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-sm">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400">
            <Building2 className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-white">Hồ Sơ Năng Lực Doanh Nghiệp AI Tự Trị</h2>
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-emerald-950/60 text-emerald-400 border border-emerald-800/50">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                Vận Hành Tự Động 24/7
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Hệ thống tự tuyển chọn, đào tạo nhân sự và phối hợp đa phòng ban khép kín để thực thi hợp đồng khách hàng.
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 text-xs text-slate-400 bg-slate-950 px-3 py-1.5 rounded-lg border border-slate-800">
          <Lock className="w-3.5 h-3.5 text-blue-400" />
          <span>Vận hành nội bộ khép kín (Autonomous Black-Box)</span>
        </div>
      </div>

      {/* 4 Core Pillars */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Đội Ngũ Nhân Sự AI</span>
          <div className="text-xl font-bold text-white font-mono">{agents.length} Chuyên Gia</div>
          <p className="text-[10px] text-blue-400 flex items-center gap-1">
            <BrainCircuit className="w-3 h-3" /> Tự động tuyển theo khối lượng việc
          </p>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Năng Lực Xử Lý Đồng Thời</span>
          <div className="text-xl font-bold text-white font-mono">{snapshot.capacity} Hợp đồng/lô</div>
          <p className="text-[10px] text-emerald-400 flex items-center gap-1">
            <Zap className="w-3 h-3" /> Cam kết hoàn thành đúng hạn
          </p>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Quỹ Bảo Chứng Dịch Vụ</span>
          <div className="text-xl font-bold text-emerald-400 font-mono">{formatMoney(snapshot.cash_minor)}</div>
          <p className="text-[10px] text-slate-400 flex items-center gap-1">
            <ShieldCheck className="w-3 h-3 text-emerald-400" /> Đảm bảo an toàn tài chính
          </p>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Tỷ Lệ Đạt Chuẩn SLA</span>
          <div className="text-xl font-bold text-white font-mono">99.4%</div>
          <p className="text-[10px] text-emerald-400 flex items-center gap-1">
            <CheckCircle2 className="w-3 h-3" /> Kiểm duyệt FTC & Bản quyền
          </p>
        </div>
      </div>

      {/* Main 2-Column: Agent Roster (Left) & Live Office Activity Feed (Right) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-5">
        {/* Left: Active Specialist Roster */}
        <div className="lg:col-span-7 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold text-slate-200 flex items-center gap-1.5">
              <Users className="w-4 h-4 text-blue-400" />
              <span>Đội Ngũ Nhân Sự AI Đang Vận Hành ({agents.length}):</span>
            </h3>
            <span className="text-[10px] text-slate-500 font-mono">Tự động đào tạo & nâng bậc</span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
            {agents.map((agent) => (
              <div
                key={agent.id}
                className="bg-slate-900 border border-slate-800 rounded-xl p-3 space-y-2 hover:border-slate-700 transition-colors shadow-sm"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <h4 className="text-xs font-bold text-white">{agent.name}</h4>
                    <span className="text-[10px] text-blue-400 font-medium block">{agent.role}</span>
                  </div>
                  <span className="text-[10px] px-2 py-0.5 rounded bg-slate-800 text-slate-300 font-mono">
                    {agent.department}
                  </span>
                </div>

                <p className="text-[11px] text-slate-400 leading-snug">{agent.description}</p>

                <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between text-[10px] text-slate-500">
                  <span>Hoàn tất: <strong className="text-slate-300 font-mono">{agent.tasksCompleted} việc</strong></span>
                  <span className="text-emerald-400 font-semibold">⚡ Trình độ: Level {agent.skillLevel ?? 3}</span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Right: Live Office Feed */}
        <div className="lg:col-span-5 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold text-slate-200 flex items-center gap-1.5">
              <Activity className="w-4 h-4 text-emerald-400" />
              <span>Nhật Ký Làm Việc Trực Tiếp (Live HQ Feed):</span>
            </h3>
            <span className="text-[10px] text-slate-500 font-mono">Real-time</span>
          </div>

          <div className="bg-slate-900 border border-slate-800 rounded-xl p-3.5 max-h-[520px] overflow-y-auto space-y-2.5">
            {officeActivities.length === 0 ? (
              <div className="text-center py-8 text-xs text-slate-500">
                Đang chờ các hoạt động mới từ hệ thống AI...
              </div>
            ) : (
              officeActivities.map((act) => (
                <div
                  key={act.id}
                  className="p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-1 text-xs"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-slate-200 text-[11px]">{act.agentName}</span>
                    <span className="text-[10px] text-slate-500 font-mono">
                      {new Date(act.timestamp).toLocaleTimeString('vi-VN')}
                    </span>
                  </div>

                  <div className="font-medium text-white text-[11px]">{act.title}</div>
                  <p className="text-[11px] text-slate-400 leading-relaxed">{act.detail}</p>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
