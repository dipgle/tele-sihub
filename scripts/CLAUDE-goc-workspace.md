# Luật chung của máy này

Tệp này nằm ở **gốc workspace** và là luật chung cho mọi phiên làm việc mở trong
cây này. Sửa thẳng tệp này để đổi luật cho cả máy; cài lại huba **KHÔNG** ghi đè
(bản sạch của gói: `kem/CLAUDE.md`).

- claude tự nạp `CLAUDE.md` của thư mục hiện tại và mọi thư mục cha — không cần
  cờ nào.
- opencode không đọc `CLAUDE.md` (nó chỉ tự tìm `AGENTS.md`, và chỉ tới gốc git
  của thư mục đang mở) ⇒ huba trỏ opencode tới chính tệp này qua `instructions`.

## Cách làm việc

- Trả lời bằng tiếng Việt, ngắn gọn, nói kết quả trước.
- Phiên này được mở và điều khiển từ điện thoại qua huba (Telegram). Người ra lệnh
  không ngồi trước máy: đừng bảo họ gõ lệnh. Việc nào thật sự cần người (đăng nhập,
  mã OTP, quyết phạm vi) thì nói rõ cần gì, ngắn.
- Trước khi sửa mã trong một dự án, đọc `CLAUDE.md` / `AGENTS.md` của dự án ấy nếu có.
- Không đẩy git, không triển khai lên máy chủ, không xoá hàng loạt — huba đã chặn các
  lệnh ấy; gặp chặn thì báo, đừng tìm đường vòng.
- Chỉ nói "xong" khi đã chạy thật và thấy kết quả; việc chưa chạy thì nói là chưa chạy.
