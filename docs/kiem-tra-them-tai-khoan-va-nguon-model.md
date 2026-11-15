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
- 🔴 **Đoạn mở đầu phiên = MỘT khâu chung** `scripts/mo-dau-phien.py` (Quang 14/11 «soạn luật đầu
  tiên cho phiên + mcp tool là bắt buộc trước khi dán vào ngữ cảnh»): ① xác định tài khoản → ② luật
  (khối `phien.sh main` + `<gốc>/CLAUDE.md` khi CLI không tự đọc) → ③ MCP codetrail bắt tay THẬT
  (≥ 1 công cụ). ①②(luật gốc)③ BẮT BUỘC: huba chạy `--kiem` trước khi mở, hỏng ⟹ KHÔNG mở;
  `phien.sh` không bắt buộc (máy cài bằng gói không có). Script nằm CẠNH cổng bảo mật
  (`accounts::script_canh_cong` — Mac `~/projects/scripts/`, gói Linux `kem/scripts/`).
- 🔴 **Đo cả MÀN lẫn NGỮ CẢNH.** Bản `44d6bd8` nghiệm thu bằng «model chép lại khối khởi động» —
  đúng ngữ cảnh mà màn codex TRỐNG: màn chỉ in hook qua `systemMessage` («↳ Hook · …», đo 14/11
  cửa sổ Terminal thật). Đo: mở cửa sổ thật, đọc màn thấy dòng `TÀI KHOẢN PHIÊN:` + `MCP …: ✓`.
  ⚠ `phien.sh` chống lặp 120 s — lượt đo trong 120 s chỉ thấy «⏭ bỏ qua khởi động» (không phải lỗi).
  Codex: ✅ (Mac 14/11). Claude: hook riêng `.claude/settings.json` (chưa qua khâu chung). opencode: CHƯA.

### B0. Cài MÁY MỚI — không sửa tay gì (Quang 14/11)
- Không biến môi trường riêng máy nào là BẮT BUỘC: luật + MCP tính từ gốc workspace / `.mcp.json`
  (env `HUBA_*` của gói Linux chỉ ĐÈ). Bản CLI ghim ở `cli-ghim.env` — cả hai bộ cài đọc, lệch bản ⟹ cài lại.
- Mac có cây mã: `bash install_update.sh` tự dựng plist từ mẫu + bootstrap khi chưa có (mẫu trỏ `bin/hubd`).
- Mac MỚI, không cây mã: gói `bash scripts/dong-goi-mac.sh` ⟹ `.tmp/huba-mac-<sha>.tar.gz`; máy đích
  `bash huba-mac-<sha>/install_mac.sh` (cùng khuôn gói Linux: `~/huba`, `kem/`, ký bằng định danh của máy,
  plist khai `HUBA_OPENCODE_PLUGIN`/`BAO_MAT_POLICY`/`HUBA_MCP_PROJECT_AGENT`/`HUBA_LUAT_GOC`/PATH).
  Đo 15/11 (`82bcc9d`): `HUBA_THU=1` trong HOME giả NGOÀI `/tmp` (policy cho qua mọi ghi `/tmp` ⟹ thử dưới
  `/tmp` thì cổng 19/30 oan) — cổng 30/30, `huba lenh-mo acc10` của gói: TÀI KHOẢN + MCP ✓ 21; bỏ
  `kem/bin/project-agent` ⟹ từ chối mở. CHƯA chạy trên máy Mac thật mới (ký, launchctl, cài CLI).
- Linux: gói mang `kem/scripts/{mo-dau-phien,nhac-chuyen-phien}.py` + `cli-ghim.env`; thiếu ⟹ trình cài dừng.
- Còn PHẢI có người: đăng nhập tài khoản (`/accounts <acc> -l`), khoá bí mật Telegram (`huba setup`),
  chứng chỉ ký Mac lần đầu (`make-signing-cert.sh`).
- Đọc `CLAUDE.md` dự án (codex: `project_doc_fallback_filenames`, trần `project_doc_max_bytes` ≥ cỡ luật gốc).
- Luật chung nạp lúc mở phiên (hook SessionStart).
- MCP codetrail gắn được (`HUBA_MCP_PROJECT_AGENT`); thiếu binary ⇒ log, không im.
- Effort mặc định `high`; ô `effort` của hàng acc thắng.
- 🔴 **Cửa sổ ngữ cảnh**: CLI tự khai số nào (codex `model_context_window`) thì mang vào
  `LiveSession.context_window` — bảng đoán `sessions::cua_so` chỉ là chỗ rơi. Đo: `huba sessions --json` có
  `context_window` + `%` trên `/session` khớp `/status` của CLI. Thiếu ⇒ `auto_handover` bàn giao oan.
- Mọi lời gọi chạy CLI trên phiên (`fork_call`: `/ask` · `/handover` · `auto_handover`) phải có nhánh cho CLI
  này — không có thì TỪ CHỐI RÕ, đừng rơi xuống nhánh `claude`.

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
| B3 `/session` liệt kê | ✅ | ✅ | ✅ bản cài 15/11: 4 root/4 link trong tin Telegram1451; source=cli, không lấy child mới hơn. Mốc lượt toàn rollout giữ đúng chạy/dừng ở lượt dài; đếm direct child busy. Chưa nghiệm thu bấm UI hoặc bộ đếm khi có child thật đang chạy |
| `%` ngữ cảnh trên điện thoại + `auto_handover` | ✅ bảng `cua_so` | ✅ bảng (1M) | ✅ số codex TỰ KHAI (`LiveSession.context_window`, 14/11) — trước đó đoán 200k ⇒ in 73–79% khi thật 33%, `auto_handover` bắn oan 3 lần (`codex --resume` mã 2, vô hại). Codex KHÔNG đi `auto_handover`/`/ask`/`/handover` (fork chỉ-đọc chưa đo) — tự chuyển khi hook nhắc |
| Nhắc chuyển phiên ≥ 50 % | ✅ hook Stop/UserPromptSubmit | chưa đo | ✅ cùng `nhac-chuyen-phien.py` (huba `b89db79` · workspace `ba629d8`; cửa sổ = `model_context_window` codex tự khai) |
| B3 bảng hỏi → nút `/pick` | ✅ | ✅ (dịch khuôn) | ❌ chưa (`request_user_input` trong rollout) |
| `/new` + `/close` trên Telegram | ✅ | ✅ | chưa đo sau `a17b03a` |

### B4 bổ sung (14/11)
- Chạy hết bộ kiểm trên Ubuntu: `tests/watch.rs` ×2 dựng đường hubd kiểu macOS ⟹ đỏ trên Linux (không phải lỗi mã).
