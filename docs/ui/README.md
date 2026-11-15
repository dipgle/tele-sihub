# Bản mẫu thiết kế huba desktop

Mở [index.html](index.html) để xem bản mẫu. Không cần build; HTML, CSS, JavaScript nằm cùng thư mục, không dùng CDN. Bản mẫu không gọi mạng, CLI, model, MCP hay Telegram; dữ liệu hoàn toàn minh họa, thay đổi chỉ nằm trong bộ nhớ. Không nhập token hoặc thông tin tài khoản thật.

- Menu trái mở đủ bảy khu vực; hàng đầu thu/mở menu.
- Chat: chọn phiên, soạn tin, đổi phiên vẫn giữ draft riêng; xem Nội dung gốc/Hoạt động; mở luồng ngắt/đóng/bàn giao.
- Phiên, tài khoản, dự án, hoạt động: tìm toàn danh sách; bảng có STT/nền xen kẽ/phân trang.
- Tài khoản: mở Quản lý để xem form, trạng thái khoá và luồng đăng nhập minh họa.
- Kết nối: đi hết ba bước Telegram, xem mô tả nguồn/quyền MCP.
- Thanh dưới chọn mặc định/chờ lựa chọn/rỗng/lỗi/mất kết nối. Đây là bộ chọn để duyệt thiết kế, không thuộc UI sản phẩm cuối.

Thiết kế kiến trúc, contract và luồng: [thiet-ke-desktop.md](../thiet-ke-desktop.md). Đề xuất sản phẩm là frontend chung trong Tauri + lõi Rust dùng chung, adapter OS/CLI/kênh tách riêng. Bản mẫu HTML này chưa phải frontend Tauri đã triển khai.

Kiểm tại 15/11/2026 08:53 +07: kiểm cú pháp JS qua cổng Node mã 0; soát mã đã sửa các lỗi draft, bộ lọc, tìm nhanh, giữ khoá account và empty state. Playwright chạy trong sandbox không mở được Chromium (`setsockopt: Operation not permitted`), mã 1 trước khi có page. **Chưa nghiệm thu trình duyệt, chưa có ảnh thực tế.** Lượt kiểm headless host đã được xếp hòm thư huba; log dự kiến `.tmp/ui-review/host-review.log`. Chỉ khi có kết quả và đã xem ảnh mới cập nhật trạng thái này.

Xuất tệp HTML tự chứa bằng `python3 docs/ui/export.py` (chạy từ cây huba); đầu ra `.tmp/huba-desktop-design.html`. Tệp xuất đã gộp CSS/JS, không cần các tệp bên cạnh. Telegram xác nhận gửi bản xuất 63927 byte ở tin 1742 ngày 15/11; đây là xác nhận giao file, không phải nghiệm thu UI. Bản Markdown trước đó ở tin 1736. Lượt host review ban đầu chưa chạy vì đặt hòm thư sai cấp thư mục; đã xác định đúng cấu trúc trong `rust/src/pipeline.rs:7608`. Cần xếp lại kiểm host nếu tiếp tục duyệt mẫu.
