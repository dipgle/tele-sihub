# Việc mở huba

- [ ] 🔴 **Chuyển phiên SANG acc opencode không chờ phiên mới** (đo 16/11 mac, 3 lượt sang acc9): `sessions::start_fresh_after_handover` có nhánh codex (`open_codex_window` + `wait_for_codex_session_id`) nhưng opencode rơi vào nhánh Claude (`wait_for_new_session_id`) ⟹ luôn `handover_new_session_never_appeared` ⟹ GIỮ cửa sổ cũ (hai phiên cùng module). Sửa: nhánh opencode dùng lại khối `/new` (`chuan_bi_mo_opencode` + `wait_opencode_session` + `write_binding`). Chưa sửa vì hàm ấy đang có mã dở chưa commit của 01a1dcde. Tạm: đo phiên acc9 mới lên danh sách rồi SIGTERM phiên cũ.
- [ ] **Phiên ma khác tài khoản cùng tty**: 16/11 `/session` hiện acc8 `ses_e349d2d1` ở ttys025 — liên kết `~/.opencode-acc8/huba/windows.json` từ 11/11 (window …947418) bám tty đã dùng lại cho acc9 `ses_e1e048d4` (window …852058). `tty_cua_lien_ket_moi_nhat` chỉ xét trong MỘT tài khoản; cần so cả window id. Chỉ hiển thị, không có tiến trình.
- [ ] acc9 `--standalone` đang ở `extra_args` của hàng acc9 (cấu hình theo máy) — thay bằng mã khi bản 01a1dcde (`opencode_private_server`) vào main. Máy chủ riêng ⟹ không có `state/opencode/service.json` ⟹ `opencode_context_windows_unreadable`, % ngữ cảnh rơi về bảng đoán.

- [ ] Hạ tầng nhắc chuyển phiên OpenCode 15/11 21:05: UI đo live346177/628000=55.1%, không có nhắc50%; process123358 NHAC_CHUYEN_PHIEN/NHAC_NGUONG unset, plugin khai chỉ `/home/user1/huba/kem/scripts/cong-bao-mat-plugin`, hash khớp workspace. Plugin setup(index.js116–154) chỉ gate quyền/prompt, không nối nhắc; accounts.rs374–386 mới nối hook Claude/Codex. Chưa sửa hạ tầng ngoài cây; cần phiên lõi/hạ tầng bổ sung và đo RED/GREEN. UI làm gọn/tự chuyển, không đổi cờ kiểm.
- [ ] Gom UI theo lệnh Hà: bộ kiểm mang theo Git tại docs/ui/tests và docs/flows/review.cjs, Python9/9/Playwright3/3/13sơđồ GATE0. QGgom fmt/clippy0 nhưng ngắt130 ở cargo test --no-run để chuyển phiên, KHÔNG toàn kho xanh; log `.tmp/qg-ui-commit.log`. Chỉ commit docs/UI/TODO/sổ UI/brief, loại config và exec.rs của lõi; push qua hòm thư huba, cần đọc lại remote trước khai đã đẩy. Brief `.tmp/brief-huba-ui-ke-nhiem-20261115-2105.md`.

- [ ] huba/ui 15/11 20:51: theo dõi request→model/tool→model→kết thúc lượt; bàn giao lõi `.tmp/chuyen-ui-theo-doi-vong-model-20261115.md`, thiết kế `docs/flows/request-trace.md`. Đối chiếu read-only acc9 seq4…102:7assistant(6tool-calls+1stop),15toolcompleted/15IDduynhất,146695ms→idle/succeeded. 19:42:02 là tạo hàng cuối, không phải đã kết thúc; cập nhật/idle19:42:15. A xem lại/B live: Hà CHƯA chọn, đề xuấtA trước nhưng không triển khai. Live phải theo updated row, không chỉseq; idle≠nghiệm thu nghiệp vụ. Claude/Codex/live chưa đo trong lượt này. Nền: không có.

- [ ] huba/ui 15/11: bản vẽ chi tiết `docs/flows/index.html` / `index.md`, nguồn `design.json` + `build.py`:30thành phần,16UC,13sơ đồ;22loại dữ liệu qua chat/MCP/model và12nhóm kết quả. Unit9/9 mã0; Chromium13sơ đồ/0console-page-request lỗi/GATE0, có đối chứng nhãn chồng RED→GREEN; đã xem ảnh13sơ đồ/toàn cảnh/hover/hẹp. HTML207930B/hashc141693f… khớp lượt kiểm. Không job nền. Chỉ thiết kế để duyệt, chưa triển khai; WebKit/UC sản phẩm thật chưa đo, mức raw vẫn cần chốt. Xem `docs/flows/REVIEW.md`.

- [x] huba/ui 15/11: thêm **Báo cáo token** vào thiết kế + HTML mẫu theo phạm vi user thu gọn (input/output/cache theo ngày/model/account/phiên, filter/source/CSV; không nối billing/tracker). ECharts6.0.0 offline có license/integrity. Host syntax3JS0, Playwright3/3 console0, đã xem ảnh; chỉ nghiệm thu fixture, chưa collector thật. HTML1220808B gửi Telegram1811, receipt `.tmp/gui-html-token-receipt.json`. Chi tiết §9 `docs/thiet-ke-desktop.md` và `docs/ui/README.md`. Chưa commit mạch này: cấu hình ngoài phạm vi cũng đổi từ2836B sang3169B lúc09:26, chưa quy được tác giả; không gom tệp đó.

- [ ] Hạ tầng chuyển phiên UI 15/11: cổng `acc-mo-vai.sh huba .tmp/brief-huba-ui-ke-nhiem-20261115-0920.md` chọn acc10, tạo pane%16/ttypts18 nhưng thoát2: “pane đã tạo nhưng không đo được CLI sống; không khai mở thành công”. Không mở vòng qua, không sửa script ngoài cây. Log `.tmp/chuyen-ui-20261115.log`; push thiết kế d454f92 đã remote-match0trễ. Cần phiên hạ tầng điều tra kiểm CLI/launcher; đề bài kế nhiệm thủ công qua UI: `huba/ui: đọc .tmp/brief-huba-ui-ke-nhiem-20261115-0920.md`.

- [ ] huba/ui 15/11: thiết kế trước triển khai (`docs/thiet-ke-desktop.md`, mẫu `docs/ui/index.html`). Đề xuất Tauri + Rust; ưu tiên Claude CLI + OpenCode, provider/model cấu hình; chưa đo để gọi rẻ nhất. MCP không tự thu đầy đủ payload model. Bản mới có Báo cáo token đã gửi Telegram1811 (biên nhận `.tmp/gui-html-token-receipt.json`). Rà lại 19:55 +07: hash HTML hiện tại khớp log `.tmp/ui-review/host-review.log:2`; log cũ ghi syntax0, Playwright3/3, GATE_EXIT0. Chỉ kiểm fixture bằng Chromium (`.tmp/ui-review/playwright.config.cjs:5`), chưa có bằng chứng WebKit hoặc app/dữ liệu thật; không chạy lại browser trong lượt này. Cần duyệt thiết kế, chốt mức raw; chưa triển khai lõi/collector/Tauri. Không tự coi việc mở lại làn là duyệt thiết kế.

- [ ] 15/11 08:46 quản lý acc: đã gộp origin d89947d (merge04d74e2); bản nguồn chọn acc10 thật (Free33, acc3 Free99), chặn auth/token hỏng, quota cũ và API key kể cả overlay; đọc mốc sự kiện thay mtime fork. 199 test liên quan qua (1 ignored), clippy/release0; log `.tmp/{test-accounts-final-20261115,kiem-accounts-host,kiem-chon-acc10}.log`. CHƯA CÀI: thiếu script khởi động ngoài cây, chờ ngoại lệ đã hỏi. QG đang đo; 2 watch fixture Mac trên Linux và 2 tuong_codex thiếu dependency còn đỏ, không khai full xanh.

- [ ] 15/11 07:50: sửa kẹt kết quả huba-run/Codex đã viết/kiểm, CHƯA CÀI. RED4 (ô thật bỏ Enter, 2 schema journal, draft chung tiền tố), GREEN93 test liên quan, clippy -Dwarnings/release build0; smoke terminal thật khối2923ký tự: finish_send tự Enter, journal đúng1/0trùng, `.tmp/probe-codex-missed-enter-fixed.log` GATE0. Retry dùng cùng cửa kiểm journal, không nhận Unverified là gửi xong. Đợi user duyệt 3binary + script chuẩn ở workspace/scripts và bản cài/kem/scripts (blob8166c7ea, selftest14/14); installer `.tmp/cai-codex-send.py` backup/rollback/giữ pane. Chưa nghiệm thu toàn route Telegram sau cài.

- [ ] Phụ thuộc sau gộp origin `f6e5c12` (15/11): Ubuntu thiếu `../scripts/mo-dau-phien.py`. Hai bài `tuong_codex_hai_chieu` và `mo_dau_phien_hien_man_va_vao_ngu_canh` đỏ vì thiếu khâu này; 5 bài cùng nhóm qua, 1 bài ghi tài khoản thật ignored. Không sửa/ghi cây hạ tầng ngoài phiên huba; chưa cài lại bản sau gộp. Cần phiên hạ tầng đồng bộ dependency rồi chạy lại 2 bài, không bỏ kiểm tra.

- [ ] **Mac 15/11: cài bản sửa Enter chỉ Codex/đóng cũ/port opencode** — trước299/20
  binary; thu hẹp theo Quang203/8, Claude giữ cũ; fmt/clippy/release mã0. Cài mã1: chặn ghi
  `hubd.new`; daemon CHƯA nhận sửa. Hai ca mở rộng cần ps/socket bị chặn.
  Chi tiết `docs/loi-phien-enter-opencode-20261115.md`; cần nghiệm thu thực tế
  tin Tele/kết quả huba → gửi đi, bàn giao → cũ đóng, acc8/9 mở độc lập.
- [ ] **acc9 = opencode + ChatGPT (theo `docs/kiem-tra-them-tai-khoan-va-nguon-model.md` §C2)** —
  Mac 15/11 21:0x: hàng acc9 `cli:"opencode"`, model `openai/gpt-6.1-sol`, bỏ env_file/api_key_env,
  MỞ khoá; acc10 KHOÁ. Bản `4517401` đã cài (tự cài GATE_EXIT=0). CHƯA đăng nhập ChatGPT
  (`~/.opencode-acc9/` chưa có) ⟹ Quang gõ `/accounts acc9 -l` + mã thiết bị; chưa mở TUI acc9 thật.
- [ ] **Các tiền nhiệm còn tồn** — xác minh từng cặp/việc nền trước đóng; bản vá
  ghép Codex đã chuẩn bị, không đoán theo tên dự án hay tắt hàng loạt.

- [ ] **Mac 15/11: cài và nghiệm thu đọc gộp tab** — 3 đường trong `keys.rs`
  đã sửa; 229 kiểm liên quan + fmt/clippy/release xanh. CHƯA chạy Terminal thật,
  chưa cài (sandbox chặn XPC; `--verify` trả 1). Số đo:
  `docs/kiem-doc-gop-tab-20261115.md`. Không coi test mô phỏng là nghiệm thu UI.
- [x] **Kế nhiệm Mac: 45dac16e/ttys005 đã đóng** — log `close_done` 08:58 +07
  (`logs/huba.log:117504`), sau lượt trước chưa đo được; không khai phiên này tự đóng.
- [ ] **iOS: kết quả chạy lại 3550/3551/3555** — đã báo inbox tfl5 tin 3 về sửa
  nhận diện Mac `7bf441f`; chưa nhận kết quả thực, không tự chạy trên cây iOS.
- [x] **Kiểm log Mac `/accounts` + bộ chọn acc10** — `/accounts` thật 08:41,
  4.397 ký tự, ack gửi thành công; log chuyển acc6 → acc10 08:21 (31%).
  Đây là đối chiếu log, chưa nhìn tin Telegram.

- [ ] 🔴 **Ubuntu: cập nhật lên `bc3b516`** — phiên AI Ubuntu làm (brief `.tmp/brief-huba-ubuntu-cap-nhat-20261115.md`) (Quang 15/11: «ghi chú luôn thì tôi đỡ phải nhắc»). Mac đã làm + đẩy; Ubuntu làm ĐỦ 3 bước, theo thứ tự:
  1. Kéo **CẢ HAI** kho: huba **và** kho chung `/mnt/data/projects` (`workspace-infra`, cần ≥ `4997f88` — mang `scripts/mo-dau-phien.py` + `acc-mo-vai.sh`). Thiếu kho chung ⟹ đúng lỗi mục cũ dưới đây.
  2. Chạy lại 2 bài từng đỏ: `cd rust && cargo test --offline --test tuong_codex --test codex_duoc_chon` ⟹ phải XANH (đỏ ⟹ ghi lại đây, KHÔNG bỏ kiểm).
  3. Cài lại hubd: dựng gói `bash scripts/dong-goi-linux.sh` rồi `bash install_update_linux.sh` trong thư mục gói (chép `mo-dau-phien.py`/`nhac-chuyen-phien.py` vào `~/huba/kem/scripts`, nâng codex đúng bản ghim `cli-ghim.env`). Lần ĐẦU trình cài/gói Linux mới chạy thật — chỉ mới `bash -n` trên Mac. Kiểm: `bash ~/huba/install_update_linux.sh --verify`.
  - Có gì mới trong bản này: acc codex được CHỌN bằng hạn mức thật (trước đây bị ép «khoá» — `quota::rank_all`), bàn giao sang codex, `/accounts` dài chia nhiều tin, mở tab đọc gộp, đoạn mở đầu phiên bắt buộc (luật + MCP), trình cài kiểm hệ điều hành. Chi tiết: `docs/kiem-tra-them-tai-khoan-va-nguon-model.md` §B0/B2.
  - Mục cũ (giữ làm bằng chứng): «Phụ thuộc sau gộp origin `f6e5c12` (15/11): Ubuntu thiếu `../scripts/mo-dau-phien.py`. Hai bài `tuong_codex_hai_chieu` và `mo_dau_phien_hien_man_va_vao_ngu_canh` đỏ vì thiếu khâu này» — gốc: kho chung chưa kéo `4997f88`.

- [ ] Hạ tầng QG Linux: lượt gom 15/11 `.tmp/qg-acc10-tele-20261115.log` thoát 2 vì vé biến mất khỏi `/tmp/qg-hang-doi-1000`. KHÔNG xanh. Kiểm trực tiếp trước commit: release build 0, 66/66 test liên quan (1 smoke desktop ignored), clippy all-targets -D warnings 0.

- [x] acc10: giữ root thay vì child; đọc đúng dự án/làn; chạy/dừng theo mốc toàn rollout, đếm child thực sự busy; dùng chung trạng thái và màn danh sách acc1–7. 62 test liên quan + clippy/build mã 0. Đã cài PID39986, giữ 4 pane; tin Telegram1451 có 4 root/4 link. Bằng chứng `.tmp/cai-acc10-tele.log`, `.tmp/acc10-main-telegram.json`.
- [ ] Nghiệm thu acc10 bằng UI: bấm /session, chọn root, quan sát bộ đếm khi có subagent thật đang chạy. Mẫu lúc gửi tin1451 có 0 child busy; lỗi/chờ câu hỏi đồng bộ chưa có mẫu Codex thật. Không coi render/send trực tiếp là nghiệm thu bấm UI.

- [x] Cài sửa Terminal sau login: `rust/src/keys_tmux.rs`, regression `rust/tests/desktop_sau_login.rs`; 32/32 test liên quan; Ptyxis gắn pane %2 thật; hubd PID 21438 hash khớp release, giữ 5/5 pane+PID, GATE_EXIT=0. Chủ máy duyệt ngoại lệ ghi bản cài ngoài cây lúc 23:27. Bằng chứng: `.tmp/cai-desktop-20261114.log`.
- [x] Kiểm bản cài nhận đúng phiên 01a1dad6 (pts/3), client /dev/pts/2 → huba-xem-2: `.tmp/kiem-ban-cai-desktop-20261114-lan2.log`, GATE_EXIT=0.
- [ ] Nghiệm thu Terminal sau login: chủ máy nhìn cửa sổ; thử `/new`/`/front` qua Telegram sau cài. Smoke kỹ thuật trước cài: `.tmp/thu-desktop-20261114.log` (không thay cho nghiệm thu UI).
