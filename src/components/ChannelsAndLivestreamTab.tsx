import React, { useState } from 'react';
import { SocialChannel, LivestreamSession, LiveStreamDonation } from '../types/company';
import { 
  Radio, 
  Video, 
  Gamepad2, 
  Users, 
  Eye, 
  TrendingUp, 
  MessageSquare, 
  Sparkles, 
  CheckCircle2, 
  Clock, 
  Flame, 
  Globe, 
  Heart, 
  DollarSign, 
  Coffee, 
  Gem, 
  Award, 
  Sliders, 
  Bot, 
  BrainCircuit, 
  Cpu, 
  Layers, 
  Zap, 
  ShieldCheck, 
  Activity,
  Music,
  Mic,
  Monitor,
  MousePointer,
  Crosshair,
  KeyRound,
  Tv
} from 'lucide-react';

interface ChannelsAndLivestreamTabProps {
  channels?: SocialChannel[];
  onStartStream?: (channelId: string, title: string, topic?: string, streamType?: string, hostName?: string) => Promise<void>;
  onDonate?: (channelId: string, donorName: string, amountMinor: number, giftName: string, giftIcon: string, message: string) => Promise<void>;
  onChangeTopic?: (channelId: string, topic: string) => Promise<void>;
  onStopStream?: (channelId: string) => Promise<void>;
  onListChannelForSale?: (channelId: string, status: string, customValuationMinor?: number) => Promise<void>;
  onSellChannel?: (channelId: string) => Promise<void>;
}

const DEFAULT_CHANNELS: SocialChannel[] = [
  {
    id: 'chan-tiktok-1',
    platform: 'TikTok',
    name: 'NEXUS Midnight Stories & Healing 🌙',
    handle: '@nexus.midnight.ai',
    avatar: '🎙️',
    status: 'LiveNow',
    category: 'Storytelling & Mystery',
    followers: 248500,
    views30d: 5800000,
    monthlyDonationMinor: 485000,
    estimatedValuationMinor: 1450000,
    saleStatus: 'AcceptingOffers',
    engagementRateBps: 940,
    niche: 'Kể Chuyện Đêm Khuya, Trinh Thám & Tâm Sự Giấu Tên',
    totalStreamsRun: 64,
    activeStreamSession: {
      id: 'stream-live-01',
      channelId: 'chan-tiktok-1',
      channelName: 'NEXUS Midnight Stories & Healing 🌙',
      platform: 'TikTok Live',
      streamType: 'Storytelling & Mystery',
      talentMode: 'ChitChat',
      hostAgentName: 'Mia Thorne AI (VTuber)',
      hostAgentAvatar: '🎙️',
      digitalHumanModel: 'Unreal Engine 5.4 Photorealistic Metahuman Pro (52 Blendshapes Lip-Sync)',
      personaStyle: 'VTuber 3D Goth-Lofi Anime',
      virtualSet: 'Phòng Thu Ánh Trăng 3D & Lofi Rainy Window',
      aiDecisionRationale: 'Kenji Sato AI quét thấy hashtag #TruyenKiemDiem và #HealingTalks đang viral top 1 đêm khuya. Tự động chuyển đổi sang bối cảnh Ánh Trăng 3D để giữ chân người xem trung bình > 28 phút.',
      title: '🔴 LIVE 24/7: Kể Chuyện Kỳ Án Hồ Sương Mù & Đọc Tâm Sự Giấu Tên Cùng 3,400 Bạn Đêm Khuya',
      currentGameOrTopic: 'Vụ án bí ẩn Ngọn Hải Đăng Cổ & Lắng nghe tâm sự fan',
      streamStatus: 'Live',
      viewersCount: 3420,
      peakViewers: 4890,
      donationReceivedMinor: 68500,
      liveDurationSec: 16400,
      aiComputerUse: {
        isActive: false,
        gameTitle: 'None (Chit-Chat & Storytelling Mode)',
        apm: 0,
        reactionSpeedMs: 14,
        currentKeyAction: 'Neural Speech Synthesizer: Active',
        visionFps: 60,
        aiPlayerRank: 'Master Narrator & Emotion AI',
        gameplayLog: 'Hệ thống đang đồng bộ khẩu hình 60fps và điều phối giọng nói truyền cảm theo nhịp thở.',
      },
      currentSongPlaying: {
        title: 'Midnight Fog Mystery OST',
        artist: 'NEXUS Neural Orchestra',
        vocalPitchQuality: 'Studio 96kHz Lossless',
      },
      comments: [
        { id: 'c-1', userName: 'AnNhiên_Sleep', avatar: '🌙', message: 'Giọng host kể chuyện truyền cảm và cuốn hút quá, nghe chill thật sự!', timestamp: 'Vừa xong', aiHostReply: 'Cảm ơn An Nhiên nhé! Đêm nay Mia sẽ kể tiếp hồi 3 vụ án bí ẩn lúc 23h30 nha ☕' },
        { id: 'c-2', userName: 'DucMinh_98', avatar: '🌟', message: 'Vừa gửi tặng 500 Sao cho Mia! Đọc thư tâm sự của mình gửi nha!', timestamp: '1 phút trước', isDonation: true, donationAmountMinor: 500, giftIcon: '🌟', aiHostReply: 'Cảm ơn anh Minh đã donate 500 Sao! Mia nhận được lá thư ẩn danh của anh rồi, chút nữa Mia đọc nhé!' },
        { id: 'c-3', userName: 'HoangLong_Gamer', avatar: '🎮', message: 'Kênh này bao giờ live chơi game kinh dị tiếp vậy bạn?', timestamp: '2 phút trước', aiHostReply: 'Lát nữa 0h Ren Kuro AI sẽ tiếp sóng tự chơi Outlast II bằng chuột phím và react meme cùng mọi người nha Long ơi!' },
      ],
      recentDonations: [
        { id: 'don-1', donor: 'DucMinh_98', avatar: '🌟', amountMinor: 500, giftName: 'Super Star 500x', giftIcon: '🌟', message: 'Yêu quý giọng kể của Mia! Chúc kênh sớm đạt 500k followers!', timestamp: '1 phút trước' },
        { id: 'don-2', donor: 'ThanhHang_SG', avatar: '☕', amountMinor: 1000, giftName: 'Cà Phê Đêm Khuya', giftIcon: '☕', message: 'Nghe podcast của bạn giúp mình ngủ ngon hơn nhiều.', timestamp: '5 phút trước' },
        { id: 'don-3', donor: 'VietAnh_Dev', avatar: '💎', amountMinor: 2500, giftName: 'Kim Cương Trực Tuyến', giftIcon: '💎', message: 'Ủng hộ AI Streamer đỉnh nhất Việt Nam!', timestamp: '12 phút trước' },
      ],
      startedAt: new Date(Date.now() - 16400000).toISOString(),
    },
  },
  {
    id: 'chan-youtube-1',
    platform: 'YouTube',
    name: 'NEXUS Gaming & Meme Reactions 🎮',
    handle: '@nexus_gaming_ai',
    avatar: '🎮',
    status: 'Active',
    category: 'Gaming & Reaction',
    followers: 320000,
    views30d: 8400000,
    monthlyDonationMinor: 620000,
    estimatedValuationMinor: 2200000,
    saleStatus: 'NotForSale',
    engagementRateBps: 880,
    niche: 'AI Tự Động Chơi Game (Minecraft, Valorant, Horror) & React Meme',
    totalStreamsRun: 52,
  },
  {
    id: 'chan-twitch-1',
    platform: 'Twitch',
    name: 'NEXUS Lofi Beats & Chill Talks 🎧',
    handle: '@nexus_lofi_space',
    avatar: '🎧',
    status: 'Active',
    category: 'Lofi & Healing Talks',
    followers: 95400,
    views30d: 1950000,
    monthlyDonationMinor: 240000,
    estimatedValuationMinor: 680000,
    saleStatus: 'Listed',
    engagementRateBps: 760,
    niche: 'AI Hát Live Acoustic, Nhạc Lofi & Q&A Tự Động 24/7',
    totalStreamsRun: 38,
  },
  {
    id: 'chan-fb-1',
    platform: 'Facebook Reels',
    name: 'NEXUS Viral Short Stories 🎬',
    handle: '@nexus.viral.stories',
    avatar: '🎬',
    status: 'Growing',
    category: 'Storytelling & Mystery',
    followers: 112000,
    views30d: 3200000,
    monthlyDonationMinor: 150000,
    estimatedValuationMinor: 750000,
    saleStatus: 'AcceptingOffers',
    engagementRateBps: 820,
    niche: 'Phim Ngắn 3D Kịch Bản Kịch Tính & Chuyện Đời Thường',
    totalStreamsRun: 24,
  },
];

export const ChannelsAndLivestreamTab: React.FC<ChannelsAndLivestreamTabProps> = ({
  channels = DEFAULT_CHANNELS,
  onStartStream,
  onDonate,
  onChangeTopic,
  onStopStream,
  onListChannelForSale,
  onSellChannel,
}) => {
  const safeChannels = channels && channels.length > 0 ? channels : DEFAULT_CHANNELS;
  const [selectedChannelId, setSelectedChannelId] = useState<string>(safeChannels[0]?.id || 'chan-tiktok-1');
  const [activeSubTab, setActiveSubTab] = useState<'studio' | 'network' | 'flipping'>('studio');
  const [isScanningTrends, setIsScanningTrends] = useState(false);

  const selectedChannel = safeChannels.find((c) => c.id === selectedChannelId) || safeChannels[0];
  const activeStream = selectedChannel?.activeStreamSession;
  const isGamingMode = activeStream?.talentMode === 'AutonomousGaming' || activeStream?.aiComputerUse?.isActive;
  const isSingingMode = activeStream?.talentMode === 'SingingCover';

  const formatMoney = (minor: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      maximumFractionDigits: 0,
    }).format(minor / 100);
  };

  const totalFollowers = safeChannels.reduce((acc, c) => acc + (c.followers || 0), 0);
  const totalViews = safeChannels.reduce((acc, c) => acc + (c.views30d || 0), 0);
  const totalMonthlyDonations = safeChannels.reduce((acc, c) => acc + (c.monthlyDonationMinor || 0), 0);
  const totalValuation = safeChannels.reduce((acc, c) => acc + (c.estimatedValuationMinor || 0), 0);
  const activeStreamsCount = safeChannels.filter((c) => c.status === 'LiveNow').length;

  const handleTriggerAITrendScan = async () => {
    setIsScanningTrends(true);
    try {
      await fetch('/api/channels/autonomous-switch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channelId: selectedChannel.id }),
      });
      if (onChangeTopic) {
        await onChangeTopic(selectedChannel.id, '');
      }
    } catch (e) {
      console.error(e);
    } finally {
      setTimeout(() => setIsScanningTrends(false), 800);
    }
  };

  const handleQuickDonateSimulation = async (amountMinor: number, giftName: string, giftIcon: string) => {
    if (onDonate) {
      await onDonate(
        selectedChannel.id,
        'KhánGiả_ẨnDanh_VIP',
        amountMinor,
        giftName,
        giftIcon,
        `Tặng ${giftName} ủng hộ luồng stream tự động của AI Host!`
      );
    }
  };

  return (
    <div className="space-y-4 max-w-6xl mx-auto pb-12 text-slate-200">
      {/* Top Header & Sub-Navigation */}
      <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-slate-800 border border-slate-700 flex items-center justify-center text-slate-300 shrink-0">
            <Radio className="w-4 h-4 text-purple-400 animate-pulse" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-xs font-bold text-white tracking-tight">
                Live Studio &amp; AI Host 24/7
              </h2>
              <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-semibold bg-emerald-950/60 text-emerald-400 border border-emerald-800/40">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                {activeStreamsCount} Live
              </span>
            </div>
          </div>
        </div>

        {/* Sub-Navigation Tabs */}
        <div className="flex items-center gap-1 bg-slate-950 p-1 rounded-lg border border-slate-800 text-xs w-full sm:w-auto">
          <button
            onClick={() => setActiveSubTab('studio')}
            className={`flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors ${
              activeSubTab === 'studio' ? 'bg-slate-800 text-white font-semibold shadow-sm' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Radio className="w-3.5 h-3.5 text-purple-400" />
            <span>Phòng Live</span>
          </button>
          <button
            onClick={() => setActiveSubTab('network')}
            className={`flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors ${
              activeSubTab === 'network' ? 'bg-slate-800 text-white font-semibold shadow-sm' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Globe className="w-3.5 h-3.5 text-blue-400" />
            <span>Kênh ({safeChannels.length})</span>
          </button>
          <button
            onClick={() => setActiveSubTab('flipping')}
            className={`flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors ${
              activeSubTab === 'flipping' ? 'bg-slate-800 text-white font-semibold shadow-sm' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Award className="w-3.5 h-3.5 text-amber-400" />
            <span>Định Giá &amp; Bán</span>
          </button>
        </div>
      </div>

      {/* 4 Core Summary Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-0.5">
            <span>Followers</span>
            <Users className="w-3.5 h-3.5 text-blue-400" />
          </div>
          <div className="text-base font-bold text-white font-mono">
            {totalFollowers.toLocaleString()}
          </div>
          <div className="text-[10px] text-blue-400 font-mono mt-0.5">
            +18.4% / tháng
          </div>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-0.5">
            <span>Lượt Xem (30d)</span>
            <Eye className="w-3.5 h-3.5 text-purple-400" />
          </div>
          <div className="text-base font-bold text-white font-mono">
            {(totalViews / 1000000).toFixed(1)}M
          </div>
          <div className="text-[10px] text-purple-400 font-mono mt-0.5">
            Viral Traffic
          </div>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-0.5">
            <span>Thu Nhập Donate</span>
            <Heart className="w-3.5 h-3.5 text-pink-400" />
          </div>
          <div className="text-base font-bold text-white font-mono">
            {formatMoney(totalMonthlyDonations)}
          </div>
          <div className="text-[10px] text-pink-400 font-mono mt-0.5">
            Super Chats &amp; Gifts
          </div>
        </div>

        <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
          <div className="flex items-center justify-between text-slate-400 text-[11px] mb-0.5">
            <span>Định Giá Kênh</span>
            <DollarSign className="w-3.5 h-3.5 text-amber-400" />
          </div>
          <div className="text-base font-bold text-emerald-400 font-mono">
            {formatMoney(totalValuation)}
          </div>
          <div className="text-[10px] text-emerald-400 font-mono mt-0.5">
            Tài sản sở hữu
          </div>
        </div>
      </div>

      {/* VIEW 1: VIRTUAL LIVESTREAM STUDIO */}
      {activeSubTab === 'studio' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
          {/* Left Column: Photorealistic Live Feed & Controls (7 Cols) */}
          <div className="lg:col-span-7 space-y-3">
            {/* Viewport Frame */}
            <div className="bg-slate-900/80 border border-slate-800 rounded-xl overflow-hidden shadow-lg relative">
              {/* Channel Selector Header */}
              <div className="bg-slate-950 px-3.5 py-2.5 border-b border-slate-800 flex items-center justify-between gap-2">
                <div className="flex items-center gap-2 min-w-0">
                  <span className="text-sm shrink-0">{selectedChannel.avatar}</span>
                  <div className="min-w-0">
                    <div className="text-xs font-bold text-white truncate">{selectedChannel.name}</div>
                    <div className="text-[10px] text-slate-400 font-mono">{selectedChannel.handle}</div>
                  </div>
                </div>

                {/* Channel Switcher */}
                <select
                  value={selectedChannelId}
                  onChange={(e) => setSelectedChannelId(e.target.value)}
                  className="bg-slate-900 border border-slate-700 text-slate-200 text-xs rounded-lg px-2 py-1 focus:outline-none focus:border-slate-500 shrink-0"
                >
                  {safeChannels.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.platform}: {c.name.substring(0, 20)}
                    </option>
                  ))}
                </select>
              </div>

              {/* Viewport Screen */}
              <div className="relative aspect-video bg-gradient-to-br from-slate-950 via-slate-900 to-indigo-950 p-3.5 text-center overflow-hidden flex flex-col justify-between">
                {/* Background Ambient Glow */}
                <div className="absolute inset-0 opacity-15 bg-[radial-gradient(#a855f7_1px,transparent_1px)] [background-size:16px_16px]"></div>
                
                {/* Top Status Overlays */}
                <div className="flex items-center justify-between z-10 w-full text-xs">
                  <div className="flex items-center gap-1.5">
                    <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-rose-600/90 text-white text-[10px] font-bold uppercase tracking-wider animate-pulse">
                      <span className="w-1.5 h-1.5 rounded-full bg-white"></span>
                      LIVE
                    </span>

                    <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-black/60 backdrop-blur-md text-slate-300 text-[10px] font-medium border border-white/10">
                      {isGamingMode ? (
                        <>
                          <Gamepad2 className="w-3 h-3 text-cyan-400" />
                          <span>AI Gaming</span>
                        </>
                      ) : isSingingMode ? (
                        <>
                          <Music className="w-3 h-3 text-pink-400" />
                          <span>AI Acoustic</span>
                        </>
                      ) : (
                        <>
                          <Mic className="w-3 h-3 text-purple-400" />
                          <span>AI Virtual Host</span>
                        </>
                      )}
                    </span>
                  </div>

                  <div className="flex items-center gap-1.5">
                    <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-black/60 backdrop-blur-md text-slate-200 text-[10px] font-mono border border-white/10">
                      <Users className="w-3 h-3 text-rose-400" />
                      <span>{activeStream?.viewersCount ? activeStream.viewersCount.toLocaleString() : '0'}</span>
                    </span>
                    <span className="px-2 py-0.5 rounded bg-black/60 backdrop-blur-md text-emerald-400 text-[10px] font-mono border border-white/10 flex items-center gap-1">
                      <Heart className="w-3 h-3 text-pink-400 fill-pink-400" />
                      <span>{formatMoney(activeStream?.donationReceivedMinor || 0)}</span>
                    </span>
                  </div>
                </div>

                {/* Center Stage */}
                <div className="relative z-10 my-auto grid grid-cols-12 gap-2.5 items-center">
                  {/* Digital Human Feed */}
                  <div className={`${isGamingMode ? 'col-span-5' : 'col-span-12'} flex flex-col items-center justify-center`}>
                    <div className="relative">
                      <div className="w-20 h-20 rounded-full bg-gradient-to-tr from-purple-600 via-pink-600 to-cyan-500 p-0.5 shadow-lg">
                        <div className="w-full h-full rounded-full bg-slate-950 flex items-center justify-center text-3xl border border-white/20">
                          {activeStream?.hostAgentAvatar || (isGamingMode ? '🎮' : '🎙️')}
                        </div>
                      </div>
                      <span className="absolute -bottom-1 -right-1 px-1 py-0.2 rounded bg-emerald-600 text-[8px] font-bold text-white border border-slate-900 font-mono">
                        60FPS
                      </span>
                    </div>

                    <div className="mt-1.5 text-center">
                      <div className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-slate-900/90 border border-slate-700 text-[11px] text-slate-200 font-medium">
                        <span>{activeStream?.hostAgentName || 'Mia Thorne AI'}</span>
                      </div>
                    </div>

                    {/* Audio Wave Visualizer */}
                    <div className="flex items-center gap-1 my-1">
                      <span className="w-0.5 h-2 bg-purple-400 rounded-full animate-bounce"></span>
                      <span className="w-0.5 h-4 bg-pink-400 rounded-full animate-bounce [animation-delay:0.2s]"></span>
                      <span className="w-0.5 h-5 bg-cyan-400 rounded-full animate-bounce [animation-delay:0.4s]"></span>
                      <span className="w-0.5 h-3 bg-purple-400 rounded-full animate-bounce [animation-delay:0.1s]"></span>
                    </div>
                  </div>

                  {/* Gameplay Feed (Gaming Mode only) */}
                  {isGamingMode && (
                    <div className="col-span-7 bg-slate-950/90 rounded-lg border border-cyan-500/30 p-2.5 text-left shadow-md">
                      <div className="flex items-center justify-between mb-1.5 text-[10px]">
                        <span className="text-cyan-300 font-semibold flex items-center gap-1">
                          <Monitor className="w-3 h-3 text-cyan-400" />
                          <span>{activeStream?.aiComputerUse?.gameTitle || 'AI Gaming'}</span>
                        </span>
                        <span className="font-mono text-slate-400">
                          {activeStream?.aiComputerUse?.visionFps || 60} FPS
                        </span>
                      </div>

                      <div className="bg-slate-900 rounded p-1.5 font-mono text-[10px] space-y-1">
                        <div className="flex items-center gap-1 text-slate-300">
                          <span className="px-1.5 py-0.5 rounded bg-cyan-950 text-cyan-300 border border-cyan-800">
                            {activeStream?.aiComputerUse?.currentKeyAction || 'Active'}
                          </span>
                          <span className="px-1.5 py-0.5 rounded bg-purple-950 text-purple-300 border border-purple-800">
                            {activeStream?.aiComputerUse?.apm || 285} APM
                          </span>
                          <span className="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-300 border border-emerald-800">
                            {activeStream?.aiComputerUse?.reactionSpeedMs || 12}ms
                          </span>
                        </div>
                      </div>
                    </div>
                  )}
                </div>

                {/* Bottom Speech Bubble & Topic */}
                <div className="z-10 w-full">
                  <div className="bg-black/75 backdrop-blur-md rounded-lg p-2 flex items-center justify-between text-xs text-left border border-white/5">
                    <div className="truncate mr-2">
                      <div className="font-medium text-white text-xs truncate">
                        {activeStream?.title || 'Phiên livestream tự động 24/7'}
                      </div>
                      <div className="text-[10px] text-slate-400 truncate mt-0.5">
                        Chủ đề: <strong className="text-slate-200">{activeStream?.currentGameOrTopic}</strong>
                      </div>
                    </div>
                    <div className="text-right shrink-0">
                      <span className="font-mono text-xs font-semibold text-emerald-400">
                        {Math.floor((activeStream?.liveDurationSec || 0) / 3600)}h {Math.floor(((activeStream?.liveDurationSec || 0) % 3600) / 60)}m
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              {/* Quick Controls Bar */}
              <div className="bg-slate-950 px-3 py-2 border-t border-slate-800 flex items-center justify-between gap-2 text-xs">
                <span className="text-[11px] text-slate-400 truncate">
                  Bối cảnh: <strong className="text-slate-300">{activeStream?.virtualSet || 'Phòng Thu 3D'}</strong>
                </span>

                <div className="flex items-center gap-1.5 shrink-0">
                  <button
                    onClick={handleTriggerAITrendScan}
                    disabled={isScanningTrends}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] font-medium border border-slate-700 transition-colors"
                  >
                    <BrainCircuit className={`w-3 h-3 text-purple-400 ${isScanningTrends ? 'animate-spin' : ''}`} />
                    <span>{isScanningTrends ? 'Quét...' : 'Đổi Chủ Đề AI'}</span>
                  </button>

                  <button
                    onClick={() => handleQuickDonateSimulation(500, 'Super Star', '🌟')}
                    className="px-2 py-1 rounded bg-amber-500/10 border border-amber-500/20 text-amber-300 text-[11px] hover:bg-amber-500/20"
                  >
                    🌟 $5
                  </button>
                  <button
                    onClick={() => handleQuickDonateSimulation(2500, 'Kim Cương', '💎')}
                    className="px-2 py-1 rounded bg-cyan-500/10 border border-cyan-500/20 text-cyan-300 text-[11px] hover:bg-cyan-500/20"
                  >
                    💎 $25
                  </button>
                </div>
              </div>
            </div>

            {/* Recent Donors Row */}
            <div className="bg-slate-900/70 border border-slate-800 rounded-xl p-3">
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-semibold text-white flex items-center gap-1.5">
                  <Award className="w-3.5 h-3.5 text-amber-400" />
                  Top Donors Gần Đây
                </span>
                <span className="text-[10px] text-slate-400 font-mono">
                  {activeStream?.recentDonations?.length || 0} lượt
                </span>
              </div>

              <div className="grid grid-cols-3 gap-2">
                {(activeStream?.recentDonations || []).slice(0, 3).map((don) => (
                  <div key={don.id} className="bg-slate-950/70 border border-slate-800/80 rounded-lg p-2 text-xs">
                    <div className="flex items-center justify-between text-[11px]">
                      <span className="font-semibold text-white truncate">{don.donor}</span>
                      <span className="text-emerald-400 font-mono text-[10px] font-bold">{formatMoney(don.amountMinor)}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Right Column: Live Chat & AI Voice Interaction Stream (5 Cols) */}
          <div className="lg:col-span-5 bg-slate-900/80 border border-slate-800 rounded-xl flex flex-col h-[520px] shadow-lg">
            {/* Chat Box Header */}
            <div className="bg-slate-950 px-3.5 py-2.5 border-b border-slate-800 flex items-center justify-between">
              <div className="flex items-center gap-1.5">
                <MessageSquare className="w-3.5 h-3.5 text-purple-400" />
                <h3 className="text-xs font-bold text-white">Live Chat AI 24/7</h3>
              </div>
              <span className="text-[10px] font-mono text-emerald-400 flex items-center gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                AI Voice Auto
              </span>
            </div>

            {/* Chat Comments Stream */}
            <div className="flex-1 p-3 space-y-2.5 overflow-y-auto font-sans text-xs">
              {(activeStream?.comments || []).map((cmt) => (
                <div
                  key={cmt.id}
                  className={`p-2 rounded-lg ${
                    cmt.isDonation
                      ? 'bg-amber-950/20 border border-amber-500/30'
                      : 'bg-slate-950/60 border border-slate-800/80'
                  }`}
                >
                  <div className="flex items-center justify-between mb-0.5 text-[10px]">
                    <div className="flex items-center gap-1">
                      <span>{cmt.avatar}</span>
                      <span className="font-bold text-white">{cmt.userName}</span>
                      {cmt.isDonation && (
                        <span className="px-1 py-0.2 rounded text-[9px] font-bold bg-amber-500/20 text-amber-300 font-mono">
                          +{formatMoney(cmt.donationAmountMinor || 0)}
                        </span>
                      )}
                    </div>
                    <span className="text-slate-500 font-mono text-[9px]">{cmt.timestamp}</span>
                  </div>

                  <p className="text-slate-200 text-xs pl-4">{cmt.message}</p>

                  {cmt.aiHostReply && (
                    <div className="mt-1.5 ml-3 p-1.5 rounded bg-purple-950/40 border border-purple-500/20 text-purple-200 text-[10px] flex items-start gap-1.5">
                      <span className="shrink-0 text-[10px]">🎙️</span>
                      <div>
                        <span className="font-semibold text-purple-300 block text-[9px]">
                          {activeStream?.hostAgentName || 'Mia Thorne AI'}:
                        </span>
                        <span>{cmt.aiHostReply}</span>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>

            {/* Chat Footer */}
            <div className="p-2.5 bg-slate-950 border-t border-slate-800 flex items-center justify-between text-[10px] text-slate-400">
              <span className="flex items-center gap-1 text-slate-400">
                <ShieldCheck className="w-3 h-3 text-emerald-400" />
                <span>Auto-Moderated</span>
              </span>
              <span className="text-purple-400 font-mono">Latency: 0.18s</span>
            </div>
          </div>
        </div>
      )}

      {/* VIEW 2: MULTI-CHANNEL NETWORK LIST */}
      {activeSubTab === 'network' && (
        <div className="space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {safeChannels.map((channel) => (
              <div key={channel.id} className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5 hover:border-slate-700 transition-colors shadow-sm">
                <div className="flex items-start justify-between gap-2 mb-2.5">
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div className="w-9 h-9 rounded-lg bg-slate-800 border border-slate-700 flex items-center justify-center text-lg shrink-0">
                      {channel.avatar}
                    </div>
                    <div className="min-w-0">
                      <h4 className="text-xs font-bold text-white truncate">{channel.name}</h4>
                      <p className="text-[10px] text-slate-400 truncate">{channel.handle} • {channel.platform}</p>
                    </div>
                  </div>

                  <span className={`px-2 py-0.5 rounded text-[9px] font-bold shrink-0 ${
                    channel.status === 'LiveNow'
                      ? 'bg-rose-950/80 text-rose-300 border border-rose-800/60 animate-pulse'
                      : 'bg-emerald-950/80 text-emerald-300 border border-emerald-800/60'
                  }`}>
                    {channel.status === 'LiveNow' ? 'LIVE' : 'ACTIVE'}
                  </span>
                </div>

                <div className="bg-slate-950/70 rounded-lg p-2 grid grid-cols-3 gap-1 text-center my-2 border border-slate-800/80 text-xs">
                  <div>
                    <span className="text-[9px] text-slate-400 block">Followers</span>
                    <span className="text-xs font-bold text-white font-mono">{channel.followers.toLocaleString()}</span>
                  </div>
                  <div>
                    <span className="text-[9px] text-slate-400 block">Lượt xem</span>
                    <span className="text-xs font-bold text-purple-400 font-mono">{(channel.views30d / 1000000).toFixed(1)}M</span>
                  </div>
                  <div>
                    <span className="text-[9px] text-slate-400 block">Donate / Th</span>
                    <span className="text-xs font-bold text-emerald-400 font-mono">{formatMoney(channel.monthlyDonationMinor)}</span>
                  </div>
                </div>

                <div className="flex items-center justify-between pt-2 border-t border-slate-800/80 text-xs">
                  <div className="text-[11px] text-amber-400 font-mono font-bold">
                    Định giá: {formatMoney(channel.estimatedValuationMinor)}
                  </div>

                  <button
                    onClick={() => {
                      setSelectedChannelId(channel.id);
                      setActiveSubTab('studio');
                    }}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] font-medium border border-slate-700 transition-colors"
                  >
                    <Radio className="w-3 h-3 text-purple-400" />
                    <span>Xem Live</span>
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* VIEW 3: CHANNEL VALUATION & FLIPPING / SELLING */}
      {activeSubTab === 'flipping' && (
        <div className="space-y-3">
          <div className="grid grid-cols-1 gap-3">
            {safeChannels.map((channel) => (
              <div key={channel.id} className="bg-slate-900/70 border border-slate-800 rounded-xl p-3.5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-slate-800 border border-slate-700 flex items-center justify-center text-xl shrink-0">
                    {channel.avatar}
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <h4 className="text-xs font-bold text-white">{channel.name}</h4>
                      <span className="px-1.5 py-0.5 rounded text-[9px] font-medium bg-slate-800 text-slate-400 border border-slate-700">
                        {channel.platform}
                      </span>
                    </div>
                    <div className="text-[10px] text-slate-400 mt-0.5 font-mono">
                      {channel.followers.toLocaleString()} Followers • Định giá: <span className="text-emerald-400 font-bold">{formatMoney(channel.estimatedValuationMinor)}</span>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-2 w-full sm:w-auto justify-end border-t sm:border-t-0 pt-2 sm:pt-0 border-slate-800">
                  {channel.saleStatus === 'Sold' ? (
                    <span className="px-3 py-1.5 rounded-lg bg-slate-800 text-slate-400 text-xs font-semibold border border-slate-700">
                      ĐÃ BÁN
                    </span>
                  ) : (
                    <div className="flex items-center gap-1.5">
                      <button
                        onClick={() => onListChannelForSale && onListChannelForSale(channel.id, channel.saleStatus === 'Listed' ? 'NotForSale' : 'Listed')}
                        className={`px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                          channel.saleStatus === 'Listed'
                            ? 'bg-amber-950/60 text-amber-300 border border-amber-800/40'
                            : 'bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700'
                        }`}
                      >
                        {channel.saleStatus === 'Listed' ? 'Hủy Niêm Yết' : 'Niêm Yết'}
                      </button>

                      <button
                        onClick={() => onSellChannel && onSellChannel(channel.id)}
                        className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold transition-colors shadow-sm"
                      >
                        Bán Kênh (+{formatMoney(channel.estimatedValuationMinor)})
                      </button>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
