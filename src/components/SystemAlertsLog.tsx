import React, { useState } from 'react';
import { SystemAlert } from '../types/company';
import { 
  AlertTriangle, 
  ShieldAlert, 
  CheckCircle2, 
  AlertCircle, 
  Info, 
  Flame, 
  Wallet, 
  TrendingDown, 
  ChevronDown, 
  ChevronUp, 
  ShieldCheck,
  RefreshCw,
  Sparkles,
  XCircle
} from 'lucide-react';

interface SystemAlertsLogProps {
  alerts: SystemAlert[];
  onResolveAlert?: (alertId: string) => void;
  onTriggerAlert?: (alertData: Partial<SystemAlert>) => void;
}

export const SystemAlertsLog: React.FC<SystemAlertsLogProps> = ({
  alerts,
  onResolveAlert,
  onTriggerAlert,
}) => {
  const [filter, setFilter] = useState<'All' | 'Active' | 'Resolved'>('Active');
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [isSimulating, setIsSimulating] = useState(false);

  const activeAlerts = alerts.filter((a) => !a.resolved);
  const resolvedAlerts = alerts.filter((a) => a.resolved);

  const displayedAlerts = alerts.filter((a) => {
    if (filter === 'Active') return !a.resolved;
    if (filter === 'Resolved') return a.resolved;
    return true;
  });

  const getAlertIcon = (type: SystemAlert['type'], level: SystemAlert['level']) => {
    if (type === 'High Expense Spike') return <Flame className="w-4 h-4 text-rose-400" />;
    if (type === 'Budget Exhaustion') return <TrendingDown className="w-4 h-4 text-amber-400" />;
    if (type === 'Low Runway') return <Wallet className="w-4 h-4 text-rose-400" />;
    if (level === 'Critical') return <AlertCircle className="w-4 h-4 text-rose-400" />;
    return <ShieldAlert className="w-4 h-4 text-amber-400" />;
  };

  const getBadgeStyle = (level: SystemAlert['level'], resolved: boolean) => {
    if (resolved) {
      return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30';
    }
    switch (level) {
      case 'Critical':
        return 'bg-rose-500/20 text-rose-300 border-rose-500/40 animate-pulse';
      case 'Warning':
        return 'bg-amber-500/20 text-amber-300 border-amber-500/40';
      case 'Info':
        return 'bg-indigo-500/20 text-indigo-300 border-indigo-500/30';
      default:
        return 'bg-slate-800 text-slate-300 border-slate-700';
    }
  };

  const handleSimulateGovernorAudit = () => {
    if (onTriggerAlert) {
      setIsSimulating(true);
      const simulationOptions = [
        {
          type: 'High Expense Spike' as const,
          level: 'Warning' as const,
          title: 'Phát hiện tăng vọt chi phí API Token (+34%)',
          description: 'Governor phát hiện các batch prompt xử lý video song song tiêu tốn $42.00 vượt dự toán chu kỳ. Đã kích hoạt điều tiết hạn mức.',
          mitigationAction: 'Bật Prompt Caching và chuyển batch processing sang khung giờ thấp điểm.',
        },
        {
          type: 'Budget Exhaustion' as const,
          level: 'Warning' as const,
          title: 'Ngân sách chiến dịch TikTok Shop chạm trần 90%',
          description: 'Governor phát hiện tỷ lệ tiêu hao ngân sách thử nghiệm vượt 90% trần an toàn. Yêu cầu kiểm toán dòng tiền trước khi cấp thêm vốn.',
          mitigationAction: 'Chỉ giải ngân thêm khi doanh thu đối soát từ TikTok Affiliate được thanh toán.',
        },
        {
          type: 'Low Runway' as const,
          level: 'Critical' as const,
          title: 'Cảnh Báo Dự Báo Runway: Biến Động Ngân Sách Quý',
          description: 'Governor mô phỏng kịch bản runway nếu chi phí server tăng gấp 2 lần. Hệ thống tự động kích hoạt hàng rào bảo vệ vốn khẩn cấp.',
          mitigationAction: 'Khóa toàn bộ đề xuất chi tiêu trên $500 của tất cả Agent.',
        },
      ];
      const pick = simulationOptions[Math.floor(Math.random() * simulationOptions.length)];
      onTriggerAlert(pick);
      setTimeout(() => setIsSimulating(false), 500);
    }
  };

  return (
    <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3 shadow-sm">
      {/* Header bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5 border-b border-slate-800 pb-2.5">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
            <ShieldAlert className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-white text-xs block">
                Nhật Ký Cảnh Báo Hệ Thống (System Alerts Log)
              </span>
              {activeAlerts.length > 0 && (
                <span className="px-1.5 py-0.2 rounded-full font-mono text-[10px] bg-rose-500/20 text-rose-300 border border-rose-500/40 font-bold animate-pulse">
                  {activeAlerts.length} Cảnh Báo
                </span>
              )}
            </div>
            <span className="text-[10px] text-slate-400">
              Governor Agent giám sát 24/7 phát hiện biến động chi phí, cạn ngân sách và an toàn vốn
            </span>
          </div>
        </div>

        {/* Controls: Filter and Trigger */}
        <div className="flex items-center gap-2">
          <div className="flex bg-slate-950 p-0.5 rounded-lg border border-slate-800 text-[11px]">
            <button
              onClick={() => setFilter('Active')}
              className={`px-2 py-0.5 rounded font-semibold transition-all ${
                filter === 'Active' ? 'bg-amber-600 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              Chờ xử lý ({activeAlerts.length})
            </button>
            <button
              onClick={() => setFilter('All')}
              className={`px-2 py-0.5 rounded font-semibold transition-all ${
                filter === 'All' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              Tất cả ({alerts.length})
            </button>
            <button
              onClick={() => setFilter('Resolved')}
              className={`px-2 py-0.5 rounded font-semibold transition-all ${
                filter === 'Resolved' ? 'bg-emerald-600 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              Đã khắc phục ({resolvedAlerts.length})
            </button>
          </div>

          <button
            onClick={handleSimulateGovernorAudit}
            disabled={isSimulating}
            className="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 text-[11px] font-semibold transition-all active:scale-95 disabled:opacity-50"
            title="Kích hoạt Governor quét và phát hiện rủi ro mới"
          >
            <Sparkles className={`w-3 h-3 ${isSimulating ? 'animate-spin' : ''}`} />
            <span className="hidden sm:inline">Quét Rủi Ro</span>
          </button>
        </div>
      </div>

      {/* Alert list */}
      {displayedAlerts.length === 0 ? (
        <div className="p-4 rounded-xl bg-slate-950/60 border border-slate-800/80 text-center space-y-1">
          <ShieldCheck className="w-5 h-5 text-emerald-400 mx-auto" />
          <p className="text-xs text-slate-300 font-medium">Không có cảnh báo nguy cấp nào chưa được xử lý.</p>
          <p className="text-[11px] text-slate-500">Governor ghi nhận mọi chỉ số tài chính đều nằm trong ngưỡng an toàn hiến định.</p>
        </div>
      ) : (
        <div className="space-y-2">
          {displayedAlerts.map((alert) => {
            const isExpanded = expandedId === alert.id;
            return (
              <div
                key={alert.id}
                className={`p-3 rounded-xl border transition-all text-xs ${
                  alert.resolved
                    ? 'bg-slate-950/40 border-slate-800/60 opacity-80'
                    : alert.level === 'Critical'
                    ? 'bg-rose-950/20 border-rose-500/40'
                    : 'bg-slate-950 border-amber-500/30'
                }`}
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="flex items-start gap-2.5">
                    <div className="p-1.5 rounded-lg bg-slate-900 border border-slate-800 shrink-0 mt-0.5">
                      {getAlertIcon(alert.type, alert.level)}
                    </div>

                    <div>
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold border ${getBadgeStyle(alert.level, alert.resolved)}`}>
                          {alert.resolved ? 'ĐÃ KHẮC PHỤC' : alert.type.toUpperCase()}
                        </span>
                        <strong className="text-white text-xs">{alert.title}</strong>
                      </div>

                      <p className="text-slate-300 mt-1 leading-relaxed text-[11px]">
                        {alert.description}
                      </p>

                      {alert.mitigationAction && (
                        <div className="mt-2 p-2 rounded-lg bg-emerald-950/20 border border-emerald-500/20 text-emerald-300 text-[11px] flex items-center gap-1.5">
                          <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                          <span><strong>Biện pháp Governor:</strong> {alert.mitigationAction}</span>
                        </div>
                      )}

                      <div className="flex items-center gap-3 text-[10px] text-slate-500 font-mono mt-1.5">
                        <span>Phát hiện bởi: <strong className="text-indigo-300">{alert.discoveredBy}</strong></span>
                        <span>•</span>
                        <span>Kỳ #{alert.cycle}</span>
                        <span>•</span>
                        <span>{new Date(alert.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                      </div>
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="flex items-center gap-2 shrink-0">
                    {!alert.resolved && onResolveAlert && (
                      <button
                        onClick={() => onResolveAlert(alert.id)}
                        className="px-2.5 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-[10px] transition-all active:scale-95 shadow-sm"
                      >
                        Xác Nhận Xử Lý
                      </button>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
