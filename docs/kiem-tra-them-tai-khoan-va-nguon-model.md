# Sổ kiểm tra — thêm TÀI KHOẢN hoặc NGUỒN MODEL mới vào huba

Mở 14/11/2026 (Quang: *«ghi lại nhật ký những thứ cần kiểm tra khi thêm tài khoản và nguồn model mới»*).
Nguồn model = một CLI mới (claude · opencode · codex · …) hoặc một nhà cung cấp mới sau CLI cũ.

🔴 **Luật nền:** *mọi tài khoản phải có đầu vào như nhau, cùng trải nghiệm — đừng bắt người dùng nhớ nhiều*
(Quang 14/11). Chỗ khác nhau giữa các CLI do **huba và cổng bảo mật tự bù**, không do người gõ lệnh nhớ.
⇒ Không thêm cờ bắt buộc / câu từ chối chỉ áp cho một loại acc.

Mỗi mục: **cách đo** (phép đo phải đổi được trạng thái — có ca ĐỎ và ca XANH). «Chưa đo» là trạng thái
riêng, không phải xanh.

## A. Thêm TÀI KHOẢN (CLI đã có)

| # | Kiểm gì | Cách đo |
|---|---|---|
| A1 | Hàng trong `huba.config.json` (`name`, `config_dir`, `cli`, `locked`) **và** thư mục `config_dir` có thật | banner `phien.sh mo` dòng «LỆCH với huba.config.json» phải rỗng |
| A2 | Đăng nhập từ điện thoại | `/accounts <acc> -l` trên Telegram → tệp đăng nhập xuất hiện trong `config_dir` |
| A3 | `/accounts` + trang `huba accounts` hiện đúng loại CLI, email, gói; xếp tên tự nhiên (acc2 < acc10) | nhìn tin Telegram + trang thật |
| A4 | Hạn mức ĐO được (không phải số chết) | banner «TÀI KHOẢN»: acc nằm trong MẪU SỐ «ĐO», có `fetchedAt` |
| A5 | `/new -a <acc> [-s <dự án>]` mở cửa sổ/tab thật, ghép lại được theo tty | `/new` trên Telegram → `/session` thấy phiên, nút «👁 Vào phiên» dẫn đúng |
| A6 | `/close` · `/stop` tắt hẳn (CLI nhận lệnh thoát của chính nó) | sau lệnh: tiến trình mất, cửa sổ đóng |
| A7 | `acc-mo-vai.sh` xét acc này trong nhóm ứng viên (nếu muốn) | đọc dòng `LY_DO=` của chính script |

## B. Thêm NGUỒN MODEL / CLI mới — làm hết A, cộng thêm

### B1. Tường (luật 1 của huba) — **an ninh = chặn việc khác**
- **Đo tên công cụ + payload THẬT của MỌI công cụ ghi** trước khi tin cổng. Cách: `CODEX_HOME`/config tạm
  trong scratchpad (tệp đăng nhập = **symlink**, không chép bí mật), hook ghi nguyên stdin ra `cap.jsonl`,
  giao một việc «tạo tệp, sửa tệp, tạo tệp tuyệt đối ở thư mục khác».
  📌 Ca thật 14/11: codex sửa tệp bằng `apply_patch` (đường nằm TRONG thân patch, tương đối theo `cwd`) —
  cổng chỉ biết `edit|write|…` nên **lọt hoàn toàn**; `/new acc10` không `-s` = ghi được cả workspace.
- Mỗi công cụ ghi đi qua `decide_write` ⇒ **gắn dự án ở lần ghi đầu** (như `guard-write-path` của Claude).
- Lệnh shell đi qua `command_denied` (cùng danh sách `deny_tools` với `--disallowedTools`).
- WebFetch/WebSearch/mạng: tắt hoặc qua cổng.
- Không hộp thoại nào chờ người ngồi máy: «tin cậy thư mục», «tin cậy hook», xin quyền (codex: `-a never`,
  trust ghi sẵn vào `config.toml`, `--dangerously-bypass-hook-trust` chỉ khi hook duy nhất là hook của huba).
- **Đối chứng ngược hai chiều**: thêm ca vào `python3 ~/projects/scripts/cong_bao_mat.py --tu-kiem`
  (RED trước khi sửa, GREEN sau) **và** một lượt CLI thật: ghi trong dự án ⇒ qua; ghi sang dự án khác ⇒
  CHẶN + `security.log` có dòng deny + tệp KHÔNG xuất hiện.
- Bản chép cổng trong gói (`~/huba/kem/scripts/`) phải khớp nguồn (`cmp`) — hook của acc gọi BẢN CHÉP.

### B2. Ngữ cảnh như phiên Claude
- 🔴 **Khối KHỞI ĐỘNG PHIÊN** (`bash ~/projects/scripts/phien.sh main` — hook SessionStart của
  `.claude/settings.json`) phải tới model. Đo: hỏi model chép dòng «KHỞI ĐỘNG PHIÊN». ⚠ `phien.sh`
  chống lặp 120 s — lượt đo trong 120 s chỉ thấy dòng «⏭ bỏ qua khởi động» (không phải lỗi).
  Codex: `huba-nap-luat.py` (`44d6bd8`). opencode: CHƯA đo.
- Đọc `CLAUDE.md` dự án (codex: `project_doc_fallback_filenames`, trần `project_doc_max_bytes` ≥ cỡ luật gốc).
- Luật chung nạp lúc mở phiên (hook SessionStart).
- MCP codetrail gắn được (`HUBA_MCP_PROJECT_AGENT`); thiếu binary ⇒ log, không im.
- Effort mặc định `high`; ô `effort` của hàng acc thắng.

### B3. Nhìn và điều khiển từ điện thoại (bài kiểm cầu nối)
| Mục | Đo |
|---|---|
| `/session` liệt kê phiên của CLI này (nguồn nhật ký phiên riêng của nó) | mở phiên → `/session` thấy |
| `watch.rs` báo «vừa xong» / «vừa tắt» đúng một lần | để phiên xong lượt → đúng 1 tin |
| Câu hỏi lựa chọn hiện nút, `/pick` · `/key` bấm được | cho CLI hỏi bảng 1 câu và 2 câu |
| `/shot`, `/type`, ⏎ gửi đi được | thao tác trên Telegram, xem màn |
| `/ask` · `/handover` chạy trên bản FORK chỉ-đọc, phiên gốc không đổi byte/mtime | so `stat` trước/sau |
| `spend` có ghi | `sqlite3 data/huba.sqlite` |

### B4. Bẫy CLI đã trả giá
- `codex exec` không đóng stdin ⇒ **đứng chờ «Reading additional input from stdin…» tới hết giờ** (14/11,
  mã 124) ⇒ chạy không tương tác luôn `< /dev/null`.
- `codex` với `CODEX_HOME` dưới `/tmp`: cảnh báo không tạo PATH alias — vô hại cho lượt đo.
- `CODEX_HOME` chưa tồn tại ⇒ codex thoát 1 (`ensure_dirs` tạo trước).
- Claude: `--disallowedTools` nhận nhiều đối số (ăn mất prompt đặt sau nó) — luật 10.

## C. Trạng thái các nguồn hiện có (chỉ ghi cái ĐÃ đo)

| Mục | claude | opencode (acc8) | codex (acc10) |
|---|---|---|---|
| B1 cổng ghi gắn dự án | ✅ guard-write-path | ✅ plugin | ✅ cổng nguồn 14/11 (tự kiểm 30/30 + lượt codex thật qua/chặn) · ⚠ bản chép `~/huba/kem` CHƯA cài lại |
| B1 tắt tìm web | ✅ `--disallowedTools` | ✅ cổng | ✅ `web_search="disabled"` (`bd6ee9e`, đo hai chiều: mặc định codex TỰ tìm web) |
| MCP mặc định | — | — | `codex_apps` tắt bằng `features.apps=false` · `codex_tui` (quản luồng) KHÔNG tắt được |
| MCP codetrail gọi được | ✅ | ✅ | ✅ `default_tools_approval_mode="approve"` (`3e5aca5`; đo hai chiều: không khoá ⟹ «requires approval, but approval policy is never») |
| B3 `/session` liệt kê | ✅ | ✅ | ✅ `huba sessions --json` máy thật (`06cbaa6`) · ⚠ hubd CHƯA cài lại · Telegram chưa đo |
| Nhắc chuyển phiên ≥ 50 % | ✅ hook Stop/UserPromptSubmit | chưa đo | ✅ cùng `nhac-chuyen-phien.py` (huba `b89db79` · workspace `ba629d8`; cửa sổ = `model_context_window` codex tự khai) |
| B3 bảng hỏi → nút `/pick` | ✅ | ✅ (dịch khuôn) | ❌ chưa (`request_user_input` trong rollout) |
| `/new` + `/close` trên Telegram | ✅ | ✅ | chưa đo sau `a17b03a` |

### B4 bổ sung (14/11)
- Chạy hết bộ kiểm trên Ubuntu: `tests/watch.rs` ×2 dựng đường hubd kiểu macOS ⟹ đỏ trên Linux (không phải lỗi mã).
