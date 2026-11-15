# huba/ui

## 2026-11-15 08:53 +07 — thiết kế desktop

Yêu cầu ban đầu: chat local + quản lý + Telegram + ứng dụng cài được. Chủ máy bổ sung: thiết kế UI/luồng trước; MCP/input-output gốc; giảm sửa theo OS. Bản thiết kế `docs/thiet-ke-desktop.md:7`, bản mẫu `docs/ui/index.html:1` (7 khu vực, fixture minh họa, không kết nối thật).

Đề xuất: frontend chung Tauri, lõi Rust hiện có, adapter OS độc lập CLI/kênh; một daemon/hàng lệnh/kho event. MCP facade truy cập nguồn, không mặc nhiên lấy đầy đủ payload model. Chưa chốt mức raw chủ máy cần.

Đã đo: syntax JS qua cong-node mã0; soát độc lập sửa draft/account khoá/filter/search/empty. Playwright sandbox mã1: Chromium setsockopt bị chặn, chưa mở page, chưa ảnh. Hòm thư host đã xếp; log dự kiến `.tmp/ui-review/host-review.log` có GATE_EXIT. Không sửa Rust, không dựng/cài app; config M từ trước giữ nguyên.

NỢ MỞ CỦA LÀN: duyệt thiết kế và mức raw; đọc host review khi có; kiểm bố cục/luồng bằng browser; spike vỏ trên ba OS trước triển khai đầy đủ. Không có tiến trình nền; một yêu cầu host đang chờ. Chạy lại: `python3 /mnt/data/projects/huba/.tmp/ui-review/run-on-host.py`.

## 2026-11-15 09:20 +07 — gửi file thật, làm gọn/chuyển phiên 50%

Telegram xác nhận Markdown1736/27212B, HTML1742/63927B; receipts .tmp/gui-thiet-ke-receipt.json và gui-html-thiet-ke-receipt.json. HTML tự chứa xuất từ docs/ui/export.py. Sửa cách xếp hòm thư: cần claude-UID/slug/ID/scratchpad; bản cũ thiếu slug nên không ai chạy UI review. Không có kiểm browser/ảnh mới. Không còn job gửi treo.

Chủ máy hỏi chỉ2plugin Claude/OpenCode có phủ mọi model/rẻ nhất không: thiết kế thêm hướng ưu tiên2CLI, provider catalog dữ liệu; chưa kết luận mọi provider/thuê bao hay chi phí khi chưa đo; không xoá Codex. Brief .tmp/brief-huba-ui-ke-nhiem-20261115-0920.md. Không có việc nền.

Chuyển phiên: d454f92 đã push/remote-match. Cổng chọn acc10, tạo pane%16/ttypts18, nhưng SUCCESSOR_EXIT2 do không đo được CLI sống; trạng thái TẮC, không mở vòng qua. Log .tmp/chuyen-ui-20261115.log. Không có phiên mới được xác nhận, không còn job nền.
