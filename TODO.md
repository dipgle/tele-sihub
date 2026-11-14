# Việc mở huba

- [ ] Hạ tầng QG Linux: lượt gom 15/11 `.tmp/qg-acc10-tele-20261115.log` thoát 2 vì vé biến mất khỏi `/tmp/qg-hang-doi-1000`. KHÔNG xanh. Kiểm trực tiếp trước commit: release build 0, 66/66 test liên quan (1 smoke desktop ignored), clippy all-targets -D warnings 0.

- [x] acc10: giữ root thay vì child; đọc đúng dự án/làn; chạy/dừng theo mốc toàn rollout, đếm child thực sự busy; dùng chung trạng thái và màn danh sách acc1–7. 62 test liên quan + clippy/build mã 0. Đã cài PID39986, giữ 4 pane; tin Telegram1451 có 4 root/4 link. Bằng chứng `.tmp/cai-acc10-tele.log`, `.tmp/acc10-main-telegram.json`.
- [ ] Nghiệm thu acc10 bằng UI: bấm /session, chọn root, quan sát bộ đếm khi có subagent thật đang chạy. Mẫu lúc gửi tin1451 có 0 child busy; lỗi/chờ câu hỏi đồng bộ chưa có mẫu Codex thật. Không coi render/send trực tiếp là nghiệm thu bấm UI.

- [x] Cài sửa Terminal sau login: `rust/src/keys_tmux.rs`, regression `rust/tests/desktop_sau_login.rs`; 32/32 test liên quan; Ptyxis gắn pane %2 thật; hubd PID 21438 hash khớp release, giữ 5/5 pane+PID, GATE_EXIT=0. Chủ máy duyệt ngoại lệ ghi bản cài ngoài cây lúc 23:27. Bằng chứng: `.tmp/cai-desktop-20261114.log`.
- [x] Kiểm bản cài nhận đúng phiên 01a1dad6 (pts/3), client /dev/pts/2 → huba-xem-2: `.tmp/kiem-ban-cai-desktop-20261114-lan2.log`, GATE_EXIT=0.
- [ ] Nghiệm thu Terminal sau login: chủ máy nhìn cửa sổ; thử `/new`/`/front` qua Telegram sau cài. Smoke kỹ thuật trước cài: `.tmp/thu-desktop-20261114.log` (không thay cho nghiệm thu UI).
