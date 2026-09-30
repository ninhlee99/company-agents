import React, { useState } from 'react';
import { SocialChannel, LivestreamSession } from '../types/company';
import { 
  Radio, 
  Video, 
  ShoppingBag, 
  Users, 
  Eye, 
  TrendingUp, 
  MessageSquare, 
  Sparkles, 
  Share2, 
  CheckCircle2, 
  Clock, 
  Flame, 
  Tag, 
  Play, 
  Square,
  Globe
} from 'lucide-react';

interface ChannelsAndLivestreamTabProps {
  channels: SocialChannel[];
  onStartStream?: (channelId: string, title: string) => Promise<void>;
  onPinProduct?: (channelId: string, productTitle: string, priceMinor: number) => Promise<void>;
  onStopStream?: (channelId: string) => Promise<void>;
}

export const ChannelsAndLivestreamTab: React.FC<ChannelsAndLivestreamTabProps> = ({
  channels,
  onStartStream,
  onPinProduct,
  onStopStream,
}) => {
  const [selectedChannelId, setSelectedChannelId] = useState<string>(channels[0]?.id || 'chan-tiktok-1');
  const [newCommentText, setNewCommentText] = useState('');
  const [activeSubTab, setActiveSubTab] = useState<'studio' | 'network'>('studio');

  const selectedChannel = channels.find((c) => c.id === selectedChannelId) || channels[0];
  const activeStream = selectedChannel?.activeStreamSession;

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const totalFollowers = channels.reduce((acc, c) => acc + c.followers, 0);
  const totalViews = channels.reduce((acc, c) => acc + c.views30d, 0);
  const totalGmv = channels.reduce((acc, c) => acc + c.gmvMinor, 0);
  const activeStreamsCount = channels.filter((c) => c.status === 'LiveNow').length;

  return (
    <div className="space-y-5 max-w-6xl mx-auto pb-12">
      {/* Top Banner */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-sm">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-pink-500/10 border border-pink-500/20 flex items-center justify-center text-pink-400">
            <Radio className="w-5 h-5 animate-pulse" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-white">Quản Lý Kênh &amp; Phòng Livestream AI 24/7</h2>
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold bg-pink-950/60 text-pink-400 border border-pink-800/50">
                <span className="w-1.5 h-1.5 rounded-full bg-pink-400 animate-pulse"></span>
                {activeStreamsCount} Phiên Live Đang Phát
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              AI Host (Mia Thorne) phát trực tiếp 24/7, tự động tương tác chat và ghim giỏ hàng chốt deal
            </p>
          </div>
        </div>

        {/* Sub Navigation Buttons */}
        <div className="flex items-center gap-1.5 bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs w-full md:w-auto">
          <button
            onClick={() => setActiveSubTab('studio')}
            className={`flex-1 md:flex-none flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-md font-medium transition-colors ${
              activeSubTab === 'studio' ? 'bg-slate-800 text-white shadow-sm' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Radio className="w-3.5 h-3.5 text-pink-400" />
            <span>Phòng Live Ảo (Studio)</span>
          </button>
          <button
            onClick={() => setActiveSubTab('network')}
            className={`flex-1 md:flex-none flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-md font-medium transition-colors ${
              activeSubTab === 'network' ? 'bg-slate-800 text-white shadow-sm' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Globe className="w-3.5 h-3.5 text-blue-400" />
            <span>Mạng Lưới Kênh ({channels.length})</span>
          </button>
        </div>
      </div>

      {/* 4 Core Channel Metrics */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Tổng Người Theo Dõi (Followers)</span>
          <div className="text-xl font-bold text-white font-mono">{totalFollowers.toLocaleString()}</div>
          <p className="text-[10px] text-blue-400 flex items-center gap-1">
            <Users className="w-3 h-3" /> Trên 4 nền tảng mạng xã hội
          </p>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Lượt Xem 30 Ngày (Views)</span>
          <div className="text-xl font-bold text-white font-mono">{(totalViews / 1000000).toFixed(1)}M Views</div>
          <p className="text-[10px] text-purple-400 flex items-center gap-1">
            <Eye className="w-3 h-3" /> Tự động phân phối đa kênh
          </p>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Doanh Số Bán Hàng (GMV)</span>
          <div className="text-xl font-bold text-emerald-400 font-mono">{formatMoney(totalGmv)}</div>
          <p className="text-[10px] text-emerald-400 flex items-center gap-1">
            <ShoppingBag className="w-3 h-3" /> Hoa hồng đối soát tự động
          </p>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-1">
          <span className="text-[11px] text-slate-400 block font-medium">Phiên Live 24/7 Đang Chạy</span>
          <div className="text-xl font-bold text-pink-400 font-mono">{activeStreamsCount} Kênh Live</div>
          <p className="text-[10px] text-slate-400 flex items-center gap-1">
            <Sparkles className="w-3 h-3 text-pink-400" /> Host AI chốt đơn liên tục
          </p>
        </div>
      </div>

      {/* Sub Tab 1: Virtual Livestream Studio */}
      {activeSubTab === 'studio' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-5">
          {/* Left 8 Cols: Live Screen & Product Pin */}
          <div className="lg:col-span-8 space-y-4">
            {/* Channel Selector Pills */}
            <div className="flex items-center gap-2 overflow-x-auto pb-1">
              {channels.map((chan) => (
                <button
                  key={chan.id}
                  onClick={() => setSelectedChannelId(chan.id)}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap transition-colors border ${
                    selectedChannelId === chan.id
                      ? 'bg-slate-800 border-pink-500 text-white shadow-sm'
                      : 'bg-slate-900 border-slate-800 text-slate-400 hover:text-white'
                  }`}
                >
                  <span>{chan.avatar}</span>
                  <span>{chan.name}</span>
                  {chan.status === 'LiveNow' && (
                    <span className="w-2 h-2 rounded-full bg-pink-500 animate-pulse"></span>
                  )}
                </button>
              ))}
            </div>

            {/* Virtual Live Screen Frame */}
            <div className="bg-slate-950 border border-slate-800 rounded-2xl overflow-hidden shadow-2xl relative">
              {/* Live Overlay Top Bar */}
              <div className="absolute top-3 left-3 right-3 z-20 flex items-center justify-between text-xs">
                <div className="flex items-center gap-2 bg-black/60 backdrop-blur-md px-3 py-1.5 rounded-full border border-white/10 text-white">
                  <span className="w-2 h-2 rounded-full bg-pink-500 animate-pulse"></span>
                  <span className="font-bold text-[11px]">LIVE 24/7</span>
                  <span className="text-slate-400 text-[10px]">•</span>
                  <span className="flex items-center gap-1 text-[11px] font-mono text-pink-300">
                    <Eye className="w-3 h-3" /> {activeStream?.viewersCount.toLocaleString() ?? '1,480'} đang xem
                  </span>
                </div>

                <div className="flex items-center gap-2 bg-black/60 backdrop-blur-md px-3 py-1.5 rounded-full border border-white/10 text-white text-[11px]">
                  <span>🎙️ Host: <strong className="text-pink-300">{activeStream?.hostAgentName ?? 'Mia Thorne AI'}</strong></span>
                </div>
              </div>

              {/* Simulated Camera Video Viewport */}
              <div className="h-80 sm:h-96 w-full bg-gradient-to-b from-slate-900 via-slate-950 to-slate-900 flex flex-col items-center justify-center relative p-6 text-center">
                {/* Background Grid & Virtual Set Accent */}
                <div className="absolute inset-0 opacity-10 bg-[radial-gradient(#ec4899_1px,transparent_1px)] [background-size:16px_16px]"></div>

                <div className="w-20 h-20 sm:w-24 sm:h-24 rounded-full bg-gradient-to-tr from-pink-500 to-purple-600 p-1 shadow-2xl animate-pulse z-10">
                  <div className="w-full h-full rounded-full bg-slate-950 flex items-center justify-center text-3xl sm:text-4xl">
                    🎙️
                  </div>
                </div>

                <h3 className="text-base sm:text-lg font-bold text-white mt-3 z-10">
                  {selectedChannel.name}
                </h3>
                <p className="text-xs text-pink-300 font-medium z-10 max-w-md mt-1">
                  {activeStream?.title ?? '🔴 LIVE 24/7: Siêu Sale Công Nghệ AI - Chốt Đơn Tự Động'}
                </p>

                {/* Real-Time Live Sales Ticker */}
                <div className="mt-4 z-10 inline-flex items-center gap-2 bg-emerald-950/80 border border-emerald-500/40 px-3.5 py-1.5 rounded-full text-xs font-mono text-emerald-300">
                  <Flame className="w-3.5 h-3.5 text-amber-400 animate-bounce" />
                  <span>Doanh thu phiên live: +{formatMoney(activeStream?.revenueEarnedMinor ?? 384000)}</span>
                  <span>({activeStream?.ordersCount ?? 76} đơn đã chốt)</span>
                </div>
              </div>

              {/* Bottom Pinned Product Card in Live Stream */}
              {activeStream?.pinnedProduct && (
                <div className="bg-slate-900/95 border-t border-slate-800 p-3.5 flex flex-col sm:flex-row items-center justify-between gap-3">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-slate-800 border border-slate-700 flex items-center justify-center text-xl">
                      {activeStream.pinnedProduct.imageUrl || '⌨️'}
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <span className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-pink-500 text-white">Ghim #1</span>
                        <h4 className="text-xs font-bold text-white line-clamp-1">{activeStream.pinnedProduct.title}</h4>
                      </div>
                      <div className="flex items-center gap-2 mt-0.5 text-xs">
                        <span className="font-bold text-emerald-400 font-mono">
                          {formatMoney(activeStream.pinnedProduct.priceMinor)}
                        </span>
                        <span className="line-through text-slate-500 text-[11px] font-mono">
                          {formatMoney(activeStream.pinnedProduct.originalPriceMinor)}
                        </span>
                        <span className="text-[10px] text-pink-400 font-semibold">
                          -{activeStream.pinnedProduct.discountPercent}%
                        </span>
                        <span className="text-[10px] text-slate-400">
                          (Hoa hồng: <strong className="text-emerald-400">{activeStream.pinnedProduct.commissionRatePercent}%</strong>)
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 w-full sm:w-auto">
                    <span className="text-[10px] text-slate-400 hidden sm:inline">
                      Còn {activeStream.pinnedProduct.stockRemaining} chiếc
                    </span>
                    <button
                      onClick={() => alert(`AI Host đang ghim flash sale cho ${activeStream.pinnedProduct.title}`)}
                      className="px-3 py-1.5 rounded-lg bg-pink-600 hover:bg-pink-500 text-white font-semibold text-xs transition-colors shadow-sm"
                    >
                      ⚡ Đẩy Flash Sale
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Right 4 Cols: Live Chat & AI Voice Response Engine */}
          <div className="lg:col-span-4 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                <MessageSquare className="w-4 h-4 text-pink-400" />
                <span>Luồng Chat &amp; AI Phản Hồi Tức Thì:</span>
              </h3>
              <span className="text-[10px] text-emerald-400 font-mono">Real-time</span>
            </div>

            <div className="bg-slate-900 border border-slate-800 rounded-xl p-3.5 h-[420px] flex flex-col justify-between shadow-sm">
              {/* Chat messages */}
              <div className="space-y-2.5 overflow-y-auto pr-1">
                {activeStream?.comments.map((cmt) => (
                  <div key={cmt.id} className="space-y-1 text-xs">
                    <div className="flex items-center gap-1.5">
                      <span>{cmt.avatar}</span>
                      <span className="font-semibold text-slate-300 text-[11px]">{cmt.userName}</span>
                      {cmt.isPurchased && (
                        <span className="text-[9px] px-1.5 py-0.2 rounded bg-emerald-500/20 text-emerald-400 font-bold border border-emerald-500/30">
                          Đã mua hàng
                        </span>
                      )}
                    </div>
                    <div className="bg-slate-950 p-2 rounded-lg border border-slate-800/80 text-[11px] text-slate-300">
                      {cmt.message}
                    </div>
                    {cmt.aiHostReply && (
                      <div className="ml-3 pl-2 border-l-2 border-pink-500 text-[10px] text-pink-300 bg-pink-950/20 p-1.5 rounded">
                        🎙️ <strong>Mia Thorne AI:</strong> {cmt.aiHostReply}
                      </div>
                    )}
                  </div>
                ))}
              </div>

              {/* Quick Prompt AI Host Action */}
              <div className="pt-2 border-t border-slate-800">
                <div className="text-[10px] text-slate-400 mb-1">
                  ⚡ AI Host tự động nhận diện câu hỏi và trả lời bằng giọng nói chuẩn 24/7.
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Sub Tab 2: Connected Channel Network */}
      {activeSubTab === 'network' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold text-slate-200">Danh Sách Mạng Lưới Kênh Đã Kết Nối ({channels.length}):</h3>
            <span className="text-[10px] text-slate-500 font-mono">Tự động phát hành &amp; Sync</span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {channels.map((chan) => (
              <div
                key={chan.id}
                className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3 hover:border-slate-700 transition-colors shadow-sm"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center text-2xl">
                      {chan.avatar}
                    </div>
                    <div>
                      <h4 className="text-xs font-bold text-white">{chan.name}</h4>
                      <span className="text-[11px] text-blue-400 font-mono">{chan.handle}</span>
                    </div>
                  </div>

                  <span className={`text-[10px] font-semibold px-2 py-0.5 rounded border ${
                    chan.status === 'LiveNow'
                      ? 'bg-pink-950/60 text-pink-400 border-pink-800/50'
                      : 'bg-emerald-950/60 text-emerald-400 border-emerald-800/50'
                  }`}>
                    {chan.status === 'LiveNow' ? '🔴 Đang Live 24/7' : '✓ Đã kết nối'}
                  </span>
                </div>

                <div className="grid grid-cols-3 gap-2 pt-2 border-t border-slate-800 text-xs">
                  <div>
                    <span className="text-[10px] text-slate-400 block">Người theo dõi</span>
                    <span className="font-bold text-white font-mono">{chan.followers.toLocaleString()}</span>
                  </div>
                  <div>
                    <span className="text-[10px] text-slate-400 block">Lượt xem tháng</span>
                    <span className="font-bold text-white font-mono">{(chan.views30d / 1000).toFixed(0)}k</span>
                  </div>
                  <div>
                    <span className="text-[10px] text-slate-400 block">Doanh số GMV</span>
                    <span className="font-bold text-emerald-400 font-mono">{formatMoney(chan.gmvMinor)}</span>
                  </div>
                </div>

                <div className="pt-2 border-t border-slate-800/80 flex items-center justify-between text-[11px] text-slate-400">
                  <span>Ngách: <strong className="text-slate-300">{chan.niche}</strong></span>
                  <span className="text-slate-500 font-mono">Đã chạy {chan.totalStreamsRun} phiên live</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
