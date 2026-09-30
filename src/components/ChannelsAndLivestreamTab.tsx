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
  Share2, 
  CheckCircle2, 
  Clock, 
  Flame, 
  Play, 
  Square,
  Globe,
  Heart,
  DollarSign,
  Coffee,
  Gem,
  Send,
  RefreshCw,
  ShoppingBag,
  Award,
  Sliders,
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
      hostAgentName: 'Mia Thorne AI (VTuber)',
      hostAgentAvatar: '🎙️',
      title: '🔴 LIVE 24/7: Kể Chuyện Kỳ Án Hồ Sương Mù & Đọc Tâm Sự Giấu Tên Cùng 3,400 Bạn Đêm Khuya',
      currentGameOrTopic: 'Vụ án bí ẩn Ngọn Hải Đăng Cổ & Lắng nghe tâm sự fan',
      streamStatus: 'Live',
      viewersCount: 3420,
      peakViewers: 4890,
      donationReceivedMinor: 68500,
      liveDurationSec: 16400,
      comments: [
        { id: 'c-1', userName: 'AnNhiên_Sleep', avatar: '🌙', message: 'Giọng host kể chuyện truyền cảm và cuốn hút quá, nghe chill thật sự!', timestamp: 'Vừa xong', aiHostReply: 'Cảm ơn An Nhiên nhé! Đêm nay Mia sẽ kể tiếp hồi 3 vụ án bí ẩn lúc 23h30 nha ☕' },
        { id: 'c-2', userName: 'DucMinh_98', avatar: '🌟', message: 'Vừa gửi tặng 500 Sao cho Mia! Đọc thư tâm sự của mình gửi nha!', timestamp: '1 phút trước', isDonation: true, donationAmountMinor: 500, giftIcon: '🌟', aiHostReply: 'Cảm ơn anh Minh đã donate 500 Sao! Mia nhận được lá thư ẩn danh của anh rồi, chút nữa Mia đọc nhé!' },
        { id: 'c-3', userName: 'HoangLong_Gamer', avatar: '🎮', message: 'Kênh này bao giờ live chơi game kinh dị tiếp vậy bạn?', timestamp: '2 phút trước', aiHostReply: 'Lát nữa 0h Ren Kuro AI sẽ tiếp sóng chơi Outlast II và react meme cùng mọi người nha Long ơi!' },
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
    niche: 'AI Gameplay (Minecraft, Valorant, Horror) & React Meme',
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
    niche: 'Nhạc Lofi Thư Giãn, Học Bài & Q&A Trò Chuyện 24/7',
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
  const [newCommentText, setNewCommentText] = useState('');
  const [activeSubTab, setActiveSubTab] = useState<'studio' | 'network' | 'flipping'>('studio');
  const [customTopicInput, setCustomTopicInput] = useState('');
  const [isChangingTopic, setIsChangingTopic] = useState(false);

  const selectedChannel = safeChannels.find((c) => c.id === selectedChannelId) || safeChannels[0];
  const activeStream = selectedChannel?.activeStreamSession;

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

  const handleSendComment = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newCommentText.trim()) return;

    if (onDonate) {
      await onDonate(
        selectedChannel.id,
        'KhánGiả_ẨnDanh',
        0,
        'Tin Nhắn Chat',
        '💬',
        newCommentText
      );
    }
    setNewCommentText('');
  };

  const handleQuickDonate = async (amountMinor: number, giftName: string, giftIcon: string) => {
    if (onDonate) {
      await onDonate(
        selectedChannel.id,
        'FanHâmMộ_VIP',
        amountMinor,
        giftName,
        giftIcon,
        `Tặng ${giftName} cho stream thêm vui vẻ náo nhiệt!`
      );
    }
  };

  const handleChangeTopicSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!customTopicInput.trim()) return;
    if (onChangeTopic) {
      await onChangeTopic(selectedChannel.id, customTopicInput);
    }
    setCustomTopicInput('');
    setIsChangingTopic(false);
  };

  return (
    <div className="space-y-5 max-w-6xl mx-auto pb-12 text-slate-200">
      {/* Top Banner Header */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-md">
        <div className="flex items-center gap-3.5">
          <div className="w-11 h-11 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400 shrink-0">
            <Radio className="w-6 h-6 animate-pulse" />
          </div>
          <div>
            <div className="flex items-center gap-2.5 flex-wrap">
              <h2 className="text-base font-bold text-white tracking-tight">
                Phòng Livestream AI 24/7 &amp; Mạng Lưới Kênh Giải Trí
              </h2>
              <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded text-[11px] font-semibold bg-emerald-950/60 text-emerald-400 border border-emerald-800/50">
                <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
                {activeStreamsCount} Kênh Đang Phát Trực Tiếp
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Tập trung trò chuyện, tâm sự đêm khuya, chơi game giải trí, xây dựng cộng đồng triệu fans &amp; thanh khoản chuyển nhượng kênh
            </p>
          </div>
        </div>

        {/* Sub-Navigation Buttons */}
        <div className="flex items-center gap-1 bg-slate-950 p-1.5 rounded-lg border border-slate-800 text-xs w-full md:w-auto">
          <button
            onClick={() => setActiveSubTab('studio')}
            className={`flex-1 md:flex-none flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-md font-medium transition-colors ${
              activeSubTab === 'studio' ? 'bg-purple-600/30 text-purple-300 border border-purple-500/40 font-semibold' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Radio className="w-3.5 h-3.5 text-purple-400" />
            <span>Phòng Live Trực Tiếp</span>
          </button>
          <button
            onClick={() => setActiveSubTab('network')}
            className={`flex-1 md:flex-none flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-md font-medium transition-colors ${
              activeSubTab === 'network' ? 'bg-blue-600/30 text-blue-300 border border-blue-500/40 font-semibold' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Globe className="w-3.5 h-3.5 text-blue-400" />
            <span>Mạng Lưới Kênh ({safeChannels.length})</span>
          </button>
          <button
            onClick={() => setActiveSubTab('flipping')}
            className={`flex-1 md:flex-none flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-md font-medium transition-colors ${
              activeSubTab === 'flipping' ? 'bg-amber-600/30 text-amber-300 border border-amber-500/40 font-semibold' : 'text-slate-400 hover:text-white'
            }`}
          >
            <Award className="w-3.5 h-3.5 text-amber-400" />
            <span>Định Giá &amp; Bán Kênh</span>
          </button>
        </div>
      </div>

      {/* 4 Core Summary Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3.5">
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs mb-1">
            <span>Tổng Người Theo Dõi</span>
            <Users className="w-4 h-4 text-blue-400" />
          </div>
          <div className="text-xl font-bold text-white">
            {totalFollowers.toLocaleString()}
          </div>
          <div className="text-[11px] text-emerald-400 mt-1 flex items-center gap-1 font-medium">
            <TrendingUp className="w-3 h-3" />
            <span>+14.2% tuần này</span>
          </div>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs mb-1">
            <span>Lượt Xem Toàn Mạng (30d)</span>
            <Eye className="w-4 h-4 text-purple-400" />
          </div>
          <div className="text-xl font-bold text-white">
            {(totalViews / 1000000).toFixed(1)}M lượt xem
          </div>
          <div className="text-[11px] text-purple-400 mt-1 flex items-center gap-1 font-medium">
            <span>Viral tự động bởi AI</span>
          </div>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs mb-1">
            <span>Thu Nhập Donate / Tháng</span>
            <Heart className="w-4 h-4 text-pink-400" />
          </div>
          <div className="text-xl font-bold text-white">
            {formatMoney(totalMonthlyDonations)}
          </div>
          <div className="text-[11px] text-pink-400 mt-1 font-medium">
            Từ Super Chat &amp; Quà tặng ảo
          </div>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between text-slate-400 text-xs mb-1">
            <span>Tổng Giá Trị Định Giá Kênh</span>
            <DollarSign className="w-4 h-4 text-amber-400" />
          </div>
          <div className="text-xl font-bold text-emerald-400">
            {formatMoney(totalValuation)}
          </div>
          <div className="text-[11px] text-amber-400 mt-1 font-medium">
            Thanh khoản M&amp;A chuyển nhượng
          </div>
        </div>
      </div>

      {/* VIEW 1: VIRTUAL LIVESTREAM STUDIO */}
      {activeSubTab === 'studio' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-5">
          {/* Left Column: Live Video Feed & Stream Dashboard (7 Cols) */}
          <div className="lg:col-span-7 space-y-4">
            {/* Live Camera Viewport Frame */}
            <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-lg relative">
              {/* Channel Selector Header Bar */}
              <div className="bg-slate-950 px-4 py-3 border-b border-slate-800 flex items-center justify-between gap-3">
                <div className="flex items-center gap-2">
                  <span className="text-sm">{selectedChannel.avatar}</span>
                  <div>
                    <div className="text-xs font-bold text-white flex items-center gap-2">
                      <span>{selectedChannel.name}</span>
                      <span className="text-[10px] text-slate-400 font-normal">{selectedChannel.handle}</span>
                    </div>
                    <div className="text-[10px] text-slate-400">
                      Nền tảng: <span className="text-slate-300 font-medium">{selectedChannel.platform}</span> • Thể loại: <span className="text-purple-300 font-medium">{selectedChannel.category}</span>
                    </div>
                  </div>
                </div>

                {/* Stream Switcher Select */}
                <select
                  value={selectedChannelId}
                  onChange={(e) => setSelectedChannelId(e.target.value)}
                  className="bg-slate-900 border border-slate-700 text-slate-200 text-xs rounded-lg px-2.5 py-1.5 focus:outline-none focus:border-purple-500"
                >
                  {safeChannels.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.platform}: {c.name.substring(0, 26)}
                    </option>
                  ))}
                </select>
              </div>

              {/* Main Viewport Screen */}
              <div className="relative aspect-video bg-gradient-to-br from-slate-950 via-slate-900 to-indigo-950 flex flex-col items-center justify-center p-6 text-center overflow-hidden">
                {/* Background Ambient Glow */}
                <div className="absolute inset-0 opacity-25 bg-[radial-gradient(#818cf8_1px,transparent_1px)] [background-size:16px_16px]"></div>
                
                {/* Status Badges Overlay */}
                <div className="absolute top-3 left-3 flex items-center gap-2 z-10">
                  {activeStream?.streamStatus === 'Live' ? (
                    <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-rose-600/90 text-white text-[11px] font-bold tracking-wider uppercase shadow-lg animate-pulse">
                      <span className="w-2 h-2 rounded-full bg-white"></span>
                      LIVE 24/7
                    </span>
                  ) : (
                    <span className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-slate-800 text-slate-400 text-[11px] font-medium border border-slate-700">
                      ĐANG NGHỈ
                    </span>
                  )}
                  <span className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-black/60 backdrop-blur-md text-white text-[11px] font-semibold border border-white/10">
                    <Users className="w-3.5 h-3.5 text-rose-400" />
                    <span>{activeStream?.viewersCount ? activeStream.viewersCount.toLocaleString() : '0'} Viewers</span>
                  </span>
                </div>

                <div className="absolute top-3 right-3 flex items-center gap-2 z-10">
                  <span className="px-2.5 py-1 rounded-md bg-black/60 backdrop-blur-md text-pink-300 text-[11px] font-semibold border border-pink-500/20 flex items-center gap-1">
                    <Heart className="w-3.5 h-3.5 text-pink-400 fill-pink-400" />
                    <span>Donate: {formatMoney(activeStream?.donationReceivedMinor || 0)}</span>
                  </span>
                </div>

                {/* Virtual AI Avatar Representation */}
                <div className="relative z-10 flex flex-col items-center">
                  <div className="w-24 h-24 rounded-full bg-gradient-to-tr from-purple-600 to-pink-500 p-1 shadow-2xl shadow-purple-500/30 mb-3 animate-pulse">
                    <div className="w-full h-full rounded-full bg-slate-900 flex items-center justify-center text-4xl border-2 border-white/20">
                      {selectedChannel.category === 'Gaming & Reaction' ? '🎮' : '🎙️'}
                    </div>
                  </div>

                  <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-slate-900/90 border border-purple-500/30 text-xs text-purple-200 font-semibold mb-1 shadow">
                    <Sparkles className="w-3.5 h-3.5 text-purple-400" />
                    <span>{activeStream?.hostAgentName || 'Mia Thorne AI (VTuber Host)'}</span>
                  </div>

                  {/* Audio Wave Visualizer Simulation */}
                  <div className="flex items-center gap-1 my-2">
                    <span className="w-1 h-3 bg-purple-400 rounded-full animate-bounce"></span>
                    <span className="w-1 h-6 bg-pink-400 rounded-full animate-bounce [animation-delay:0.2s]"></span>
                    <span className="w-1 h-8 bg-blue-400 rounded-full animate-bounce [animation-delay:0.4s]"></span>
                    <span className="w-1 h-5 bg-purple-400 rounded-full animate-bounce [animation-delay:0.1s]"></span>
                    <span className="w-1 h-2 bg-pink-400 rounded-full animate-bounce [animation-delay:0.3s]"></span>
                  </div>

                  {/* Speech Bubble */}
                  <div className="max-w-md bg-black/75 backdrop-blur-md border border-white/10 text-white rounded-xl px-4 py-2.5 text-xs shadow-xl mt-1">
                    <p className="italic text-slate-200 font-medium">
                      "{activeStream?.comments?.[0]?.aiHostReply || 'Chào cả nhà! Chúc mọi người một buổi tối thật thư giãn và vui vẻ nhé!'}"
                    </p>
                  </div>
                </div>

                {/* Bottom Viewport Info Bar */}
                <div className="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/90 via-black/60 to-transparent p-3 pt-6 flex items-center justify-between text-xs text-left z-10">
                  <div className="truncate mr-4">
                    <div className="font-bold text-white text-xs truncate">
                      {activeStream?.title || 'Phiên livestream trò chuyện và kể chuyện AI 24/7'}
                    </div>
                    <div className="text-[11px] text-purple-300 flex items-center gap-1.5 mt-0.5">
                      <Gamepad2 className="w-3 h-3 text-purple-400" />
                      <span>Chủ đề / Game: <strong>{activeStream?.currentGameOrTopic || selectedChannel.niche}</strong></span>
                    </div>
                  </div>
                  <div className="text-right shrink-0">
                    <span className="text-[10px] text-slate-400 block">Thời gian live liên tục</span>
                    <span className="font-mono text-xs font-semibold text-emerald-400">
                      {Math.floor((activeStream?.liveDurationSec || 0) / 3600)}h {Math.floor(((activeStream?.liveDurationSec || 0) % 3600) / 60)}m
                    </span>
                  </div>
                </div>
              </div>

              {/* Live Host Control Actions */}
              <div className="bg-slate-950 p-3.5 border-t border-slate-800 flex flex-wrap items-center justify-between gap-3">
                <div className="flex items-center gap-2">
                  {activeStream?.streamStatus === 'Live' ? (
                    <button
                      onClick={() => onStopStream && onStopStream(selectedChannel.id)}
                      className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-rose-600/20 text-rose-300 border border-rose-500/30 text-xs font-semibold hover:bg-rose-600/30 transition-colors"
                    >
                      <Square className="w-3.5 h-3.5" />
                      <span>Tạm Dừng Live</span>
                    </button>
                  ) : (
                    <button
                      onClick={() => onStartStream && onStartStream(selectedChannel.id, `🔴 LIVE 24/7: ${selectedChannel.niche}`, selectedChannel.niche, selectedChannel.category)}
                      className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-emerald-600 text-white text-xs font-semibold hover:bg-emerald-500 transition-colors shadow-sm"
                    >
                      <Play className="w-3.5 h-3.5 fill-white" />
                      <span>Bắt Đầu Live Mới</span>
                    </button>
                  )}

                  <button
                    onClick={() => setIsChangingTopic(!isChangingTopic)}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium border border-slate-700 transition-colors"
                  >
                    <Sliders className="w-3.5 h-3.5 text-purple-400" />
                    <span>Đổi Game / Chủ Đề</span>
                  </button>
                </div>

                {/* Quick Donate Simulation Buttons */}
                <div className="flex items-center gap-1.5">
                  <span className="text-[11px] text-slate-400 mr-1 hidden sm:inline">Tặng quà ảo:</span>
                  <button
                    onClick={() => handleQuickDonate(500, 'Super Star 500x', '🌟')}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs font-medium hover:bg-amber-500/20 transition-colors"
                    title="Donate $5.00"
                  >
                    <span>🌟 $5</span>
                  </button>
                  <button
                    onClick={() => handleQuickDonate(1000, 'Cà Phê Đêm Khuya', '☕')}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-orange-500/10 border border-orange-500/20 text-orange-300 text-xs font-medium hover:bg-orange-500/20 transition-colors"
                    title="Donate $10.00"
                  >
                    <span>☕ $10</span>
                  </button>
                  <button
                    onClick={() => handleQuickDonate(2500, 'Kim Cương Trực Tuyến', '💎')}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-cyan-500/10 border border-cyan-500/20 text-cyan-300 text-xs font-medium hover:bg-cyan-500/20 transition-colors"
                    title="Donate $25.00"
                  >
                    <span>💎 $25</span>
                  </button>
                </div>
              </div>

              {/* Change Topic Collapsible Form */}
              {isChangingTopic && (
                <form onSubmit={handleChangeTopicSubmit} className="bg-slate-900 border-t border-slate-800 p-3.5 flex items-center gap-2">
                  <input
                    type="text"
                    value={customTopicInput}
                    onChange={(e) => setCustomTopicInput(e.target.value)}
                    placeholder="Nhập game mới hoặc chủ đề kể chuyện (VD: Chơi Minecraft sinh tồn, Kể chuyện kỳ án số 5)..."
                    className="flex-1 bg-slate-950 border border-slate-700 text-white text-xs rounded-lg px-3 py-2 focus:outline-none focus:border-purple-500"
                  />
                  <button
                    type="submit"
                    className="px-3.5 py-2 rounded-lg bg-purple-600 hover:bg-purple-500 text-white text-xs font-semibold transition-colors"
                  >
                    Cập Nhật
                  </button>
                </form>
              )}
            </div>

            {/* Recent Donations Honor Roll Box */}
            <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <Award className="w-4 h-4 text-amber-400" />
                  <h3 className="text-xs font-bold text-white uppercase tracking-wider">
                    Vinh Danh Donors &amp; Quà Tặng Ảo
                  </h3>
                </div>
                <span className="text-[11px] text-slate-400">
                  Tổng {activeStream?.recentDonations?.length || 0} lượt ủng hộ
                </span>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-3 gap-2.5">
                {(activeStream?.recentDonations || []).slice(0, 3).map((don) => (
                  <div key={don.id} className="bg-slate-950/80 border border-slate-800 rounded-lg p-2.5 flex items-start gap-2.5">
                    <div className="w-8 h-8 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-base shrink-0">
                      {don.giftIcon}
                    </div>
                    <div className="truncate">
                      <div className="flex items-center justify-between text-xs font-bold text-white">
                        <span className="truncate">{don.donor}</span>
                        <span className="text-emerald-400 font-mono text-[11px]">{formatMoney(don.amountMinor)}</span>
                      </div>
                      <p className="text-[10px] text-slate-400 truncate mt-0.5">"{don.message}"</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Right Column: Live Chat & AI Voice Interaction Stream (5 Cols) */}
          <div className="lg:col-span-5 bg-slate-900 border border-slate-800 rounded-xl flex flex-col h-[580px] shadow-lg">
            {/* Chat Box Header */}
            <div className="bg-slate-950 px-4 py-3 border-b border-slate-800 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <MessageSquare className="w-4 h-4 text-purple-400" />
                <h3 className="text-xs font-bold text-white">Live Chat &amp; Phản Hồi Giọng Nói AI</h3>
              </div>
              <span className="text-[10px] font-semibold text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded border border-emerald-800/40">
                ● Tự Động Trả Lời
              </span>
            </div>

            {/* Chat Comments Stream */}
            <div className="flex-1 p-3.5 space-y-3 overflow-y-auto font-sans text-xs">
              {(activeStream?.comments || []).map((cmt) => (
                <div
                  key={cmt.id}
                  className={`p-2.5 rounded-xl transition-all ${
                    cmt.isDonation
                      ? 'bg-amber-950/20 border border-amber-500/30'
                      : 'bg-slate-950/70 border border-slate-800/80'
                  }`}
                >
                  {/* User Comment Header */}
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-1.5">
                      <span className="text-sm">{cmt.avatar}</span>
                      <span className="font-bold text-white text-[11px]">{cmt.userName}</span>
                      {cmt.isDonation && (
                        <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30">
                          DONATE {formatMoney(cmt.donationAmountMinor || 0)}
                        </span>
                      )}
                    </div>
                    <span className="text-[10px] text-slate-500">{cmt.timestamp}</span>
                  </div>

                  {/* Comment Message */}
                  <p className="text-slate-200 text-xs pl-5">{cmt.message}</p>

                  {/* AI Host Live Voice Reply */}
                  {cmt.aiHostReply && (
                    <div className="mt-2 ml-4 p-2 rounded-lg bg-purple-950/40 border border-purple-500/20 text-purple-200 text-[11px] flex items-start gap-2">
                      <span className="shrink-0 text-xs mt-0.5">🎙️</span>
                      <div>
                        <span className="font-bold text-purple-300 text-[10px] block">
                          {activeStream?.hostAgentName || 'Mia Thorne AI'}:
                        </span>
                        <span>{cmt.aiHostReply}</span>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>

            {/* Chat Input Box */}
            <form onSubmit={handleSendComment} className="p-3 bg-slate-950 border-t border-slate-800 flex items-center gap-2">
              <input
                type="text"
                value={newCommentText}
                onChange={(e) => setNewCommentText(e.target.value)}
                placeholder="Nhập tin nhắn trò chuyện hoặc gửi lời nhắn cho AI Host..."
                className="flex-1 bg-slate-900 border border-slate-700 text-white text-xs rounded-lg px-3 py-2 focus:outline-none focus:border-purple-500 placeholder:text-slate-500"
              />
              <button
                type="submit"
                className="p-2 rounded-lg bg-purple-600 hover:bg-purple-500 text-white transition-colors"
                title="Gửi tin nhắn"
              >
                <Send className="w-4 h-4" />
              </button>
            </form>
          </div>
        </div>
      )}

      {/* VIEW 2: MULTI-CHANNEL NETWORK LIST */}
      {activeSubTab === 'network' && (
        <div className="space-y-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
            <div>
              <h3 className="text-sm font-bold text-white">Mạng Lưới Kênh Nội Dung Tự Động Của NEXUS</h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Các kênh được vận hành hoàn toàn bởi AI Agents với nội dung viral, lôi cuốn hàng triệu người xem
              </p>
            </div>
            <span className="text-xs font-semibold px-3 py-1 rounded-lg bg-blue-500/10 text-blue-400 border border-blue-500/20">
              {safeChannels.length} Kênh Hoạt Động
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {safeChannels.map((channel) => (
              <div key={channel.id} className="bg-slate-900 border border-slate-800 rounded-xl p-5 hover:border-slate-700 transition-all shadow-sm">
                <div className="flex items-start justify-between gap-3 mb-3">
                  <div className="flex items-center gap-3">
                    <div className="w-12 h-12 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center text-2xl">
                      {channel.avatar}
                    </div>
                    <div>
                      <h4 className="text-sm font-bold text-white">{channel.name}</h4>
                      <p className="text-xs text-slate-400">{channel.handle} • {channel.platform}</p>
                    </div>
                  </div>

                  <span className={`px-2.5 py-1 rounded text-[10px] font-bold ${
                    channel.status === 'LiveNow'
                      ? 'bg-rose-950/80 text-rose-300 border border-rose-800/60 animate-pulse'
                      : channel.status === 'Listed'
                      ? 'bg-amber-950/80 text-amber-300 border border-amber-800/60'
                      : 'bg-emerald-950/80 text-emerald-300 border border-emerald-800/60'
                  }`}>
                    {channel.status === 'LiveNow' ? '🔴 ĐANG LIVE' : channel.status === 'Listed' ? '🏷️ ĐANG NIÊM YẾT' : '🟢 HOẠT ĐỘNG'}
                  </span>
                </div>

                <div className="bg-slate-950 rounded-lg p-3 grid grid-cols-3 gap-2 text-center my-3 border border-slate-800">
                  <div>
                    <span className="text-[10px] text-slate-400 block">Followers</span>
                    <span className="text-xs font-bold text-white">{channel.followers.toLocaleString()}</span>
                  </div>
                  <div>
                    <span className="text-[10px] text-slate-400 block">Lượt xem (30d)</span>
                    <span className="text-xs font-bold text-purple-400">{(channel.views30d / 1000000).toFixed(1)}M</span>
                  </div>
                  <div>
                    <span className="text-[10px] text-slate-400 block">Donate / Tháng</span>
                    <span className="text-xs font-bold text-emerald-400">{formatMoney(channel.monthlyDonationMinor)}</span>
                  </div>
                </div>

                <div className="text-xs text-slate-400 mb-4">
                  <span className="text-slate-300 font-medium">Nội dung ngách:</span> {channel.niche}
                </div>

                <div className="flex items-center justify-between pt-3 border-t border-slate-800 text-xs">
                  <div className="flex items-center gap-1.5 text-amber-400 font-medium">
                    <span>Định giá kênh:</span>
                    <strong className="font-mono">{formatMoney(channel.estimatedValuationMinor)}</strong>
                  </div>

                  <button
                    onClick={() => {
                      setSelectedChannelId(channel.id);
                      setActiveSubTab('studio');
                    }}
                    className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-medium transition-colors"
                  >
                    <Radio className="w-3.5 h-3.5 text-purple-400" />
                    <span>Xem Live Stream</span>
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* VIEW 3: CHANNEL VALUATION & FLIPPING / SELLING */}
      {activeSubTab === 'flipping' && (
        <div className="space-y-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-sm">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <div className="flex items-center gap-2">
                  <Award className="w-5 h-5 text-amber-400" />
                  <h3 className="text-base font-bold text-white">Thị Trường Định Giá &amp; Chuyển Nhượng Kênh (Channel Flipping)</h3>
                </div>
                <p className="text-xs text-slate-400 mt-1 max-w-2xl">
                  Mô hình kiếm doanh thu lớn: Công ty AI tự động xây dựng kênh từ 0 lên hàng trăm nghìn followers, sau đó niêm yết bán lại cho các thương hiệu / cá nhân có nhu cầu với giá trị cao.
                </p>
              </div>

              <div className="bg-slate-950 px-4 py-3 rounded-xl border border-slate-800 text-right">
                <span className="text-xs text-slate-400 block">Tổng Thanh Khoản Tiềm Năng</span>
                <span className="text-xl font-bold text-emerald-400 font-mono">{formatMoney(totalValuation)}</span>
              </div>
            </div>
          </div>

          <div className="grid grid-cols-1 gap-4">
            {safeChannels.map((channel) => (
              <div key={channel.id} className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
                <div className="flex items-center gap-3.5">
                  <div className="w-12 h-12 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center text-2xl shrink-0">
                    {channel.avatar}
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <h4 className="text-sm font-bold text-white">{channel.name}</h4>
                      <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-slate-800 text-slate-300 border border-slate-700">
                        {channel.platform}
                      </span>
                    </div>
                    <div className="text-xs text-slate-400 mt-0.5">
                      {channel.followers.toLocaleString()} Followers • Tỷ lệ tương tác: <span className="text-purple-400 font-semibold">{((channel.engagementRateBps || 800) / 100).toFixed(1)}%</span> • {channel.niche}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-3 w-full md:w-auto justify-between md:justify-end border-t md:border-t-0 pt-3 md:pt-0 border-slate-800">
                  <div className="text-left md:text-right">
                    <span className="text-[10px] text-slate-400 block">Định Giá Chuyển Nhượng</span>
                    <span className="text-base font-bold text-emerald-400 font-mono">{formatMoney(channel.estimatedValuationMinor)}</span>
                  </div>

                  {channel.saleStatus === 'Sold' ? (
                    <span className="px-3.5 py-2 rounded-lg bg-slate-800 text-slate-400 text-xs font-semibold border border-slate-700">
                      ĐÃ BÁN THÀNH CÔNG
                    </span>
                  ) : (
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => onListChannelForSale && onListChannelForSale(channel.id, channel.saleStatus === 'Listed' ? 'NotForSale' : 'Listed')}
                        className={`px-3 py-2 rounded-lg text-xs font-semibold transition-colors ${
                          channel.saleStatus === 'Listed'
                            ? 'bg-amber-600/20 text-amber-300 border border-amber-500/30 hover:bg-amber-600/30'
                            : 'bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700'
                        }`}
                      >
                        {channel.saleStatus === 'Listed' ? 'Hủy Niêm Yết' : 'Niêm Yết Bán Kênh'}
                      </button>

                      <button
                        onClick={() => onSellChannel && onSellChannel(channel.id)}
                        className="px-3.5 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold transition-colors shadow-sm"
                      >
                        Chuyển Nhượng Ngay (+{formatMoney(channel.estimatedValuationMinor)})
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
