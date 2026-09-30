#!/usr/bin/env bash
# ==============================================================================
# NEXUS CORP - 24/7 Autonomous Company Operations Runner
# Initializes local artifact storage, runs background workers,
# and generates daily end-of-day reports.
# ==============================================================================
set -euo pipefail

echo "======================================================================"
echo "🚀 KHỞI ĐỘNG CÔNG TY TỰ TRỊ 24/7: NEXUS CORP AI ENTERPRISE"
echo "======================================================================"

# 1. Khởi tạo các thư mục lưu trữ cục bộ (Local Artifacts)
mkdir -p artifacts/daily_reports
mkdir -p artifacts/videos
mkdir -p artifacts/livestream_logs

echo "📁 Đã khởi tạo cấu trúc thư mục lưu trữ tại local:"
echo "   - Báo cáo ngày: artifacts/daily_reports/"
echo "   - Video affiliate & media: artifacts/videos/"
echo "   - Nhật ký livestream: artifacts/livestream_logs/"

# 2. Sinh báo cáo tổng kết ngày hiện tại
echo "📊 Đang đồng bộ hóa dữ liệu và xuất báo cáo ngày..."
python3 scripts/generate_daily_report.py

# 3. Kiểm tra mã nguồn và trạng thái hệ thống
echo "🛡️ Đang kiểm tra an toàn hệ thống..."
bash scripts/secret_hygiene.sh

echo "======================================================================"
echo "✅ CÔNG TY ĐÃ SẴN SÀNG HOẠT ĐỘNG 24/7!"
echo "   - Mở giao diện: npm run dev (http://localhost:5173)"
echo "   - Vào Tab 'Cấu Hình & Tích Hợp' để quản lý API, TTS và Ngân Hàng"
echo "   - Cuối ngày kiểm tra thư mục 'artifacts/daily_reports/' để xem kết quả"
echo "======================================================================"
