# 15/11 — phiên cũ, Enter và xung đột opencode

Quang báo: phiên tăng, chữ huba/Telegram nằm trong ô; ảnh chạy `opencode` báo
cổng 49374 bị PID 19715 chiếm. Phiên sửa: acc10 `01a1dcde`.

## Nguyên nhân đo được

1. `sessions::start_fresh_after_handover` trả cố định `new_id:None, old_kept:true`
   cho Codex. Không tới bước đóng chung, không vào sổ thử đóng. Log119847–849:
   03:05Z mở ttys016, id rỗng, giữ 2b43f096; sau đó `AlreadyDone`.
2. `keys::box_start` chỉ biết khung/dấu ❯ của Claude. Codex dùng › và chân model;
   không nhận ô → `still_in_box=false` → không Enter, báo đã gửi. Ảnh thật
   log121599 được giữ phần ô nhập trong fixture `codex-input-2026-11-15.txt`.
   Quang xác nhận chỉ Codex bị; Claude vẫn bình thường. Hai cửa chưa có cách
   gửi riêng cho Codex: `press("enter")` ghi byte, chưa phải phím Enter rời.
3. Opencode 2.2.7 dùng cổng dịch vụ mặc định 49374 cho mọi profile, dù XDG khác
   nhau. Sổ `.opencode-acc8/state/opencode/service.json` xác nhận PID19715 và
   URL127.0.0.1:49374, khớp ảnh. Không in mật khẩu trong sổ.

## Đã sửa

- Nhận ô Codex, cắt trước footer; không lấy hội thoại/placeholder/ảnh Codex
  trích trong Claude. Câu ngắn Codex phải khớp trọn ô. Hai cửa đi chung
  `enter_after_text`: chỉ Codex dùng Enter rời; Claude giữ đường gửi byte và
  ngưỡng cũ. `shell_window_control.rs` được giữ nguyên như HEAD.
- Bàn giao Codex dò root rollout của TUI trên tty vừa mở, đúng acc, mới hơn lúc
  mở và duy nhất; có id mới đi chung bước đóng/sổ thử lại. Thiếu bằng chứng giữ
  cũ. `/quit` phải được gửi và Codex thực sự thoát trước khi gõ `exit` của shell.
- Launcher opencode mặc định `--standalone` trên Unix/PowerShell, cả TUI/fork;
  giữ `--server` chủ máy khai. Không tắt dịch vụ nền của cửa sổ khác. CLI thật
  chấp nhận cờ ở cả TUI và `run --help` (HOME/XDG thử trong cây, không gọi model).
  Lệnh `opencode` trần của chủ máy vẫn theo mặc định riêng của CLI.
- Acc9 cùng CLI/model/MCP/plugin với acc8, giữ profile và env-file riêng;
  cả hai vẫn locked. Daemon `config_reloaded` lúc04:11:43Z (11:11 +07).

## Kiểm và giới hạn

RED trước: 4/5 ca Codex đỏ, 1/2 ca máy chủ riêng đỏ; GREEN sau: 299/299 trong
20 binary chép ngoài deps. Fmt/clippy -D warnings/release mã0. Lượt mở rộng có
2 ca bị sandbox chặn: sessions::a_remembered_window_must_still_belong_to_that_process
(`ps`), accounts_page::http_that_qua_socket_dat_cookie_roi_tra_bang (socket).
Không hạ chuẩn hoặc đổi chúng thành ignored. Log `.tmp/ke-nhiem-test-*.log`.

Sau khi thu hẹp theo xác nhận của Quang: kiểm Claude cũ RED (1/4), sau sửa
GREEN; lượt mới 203/203 trong8 binary, fmt/clippy/release mã0. Các ca giữ đường Claude,
shell và nhận đúng Codex được kiểm riêng; không coi đây là nghiệm thu UI.

Cài thật đã thử `install_update.sh --no-build`: mã1, `cp …/hubd.new: Operation
not permitted`. Bản sửa chưa vào daemon, chưa nghiệm thu Telegram/Terminal và
chưa dọn những phiên còn tồn. Không tắt PID19715 trong lượt này.
Riêng45dac16e đã `close_done` 01:58Z (log117504), d8b33f60 02:03Z (117826).
