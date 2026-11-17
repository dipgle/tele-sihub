# Rà soát: mọi chỗ mang TÊN theo CLI / model (17/11/2026)

Câu hỏi (Quang 17/11): *«Liệt kê tất cả những chỗ liên quan mà có tên định danh theo cli model và
giải thích nó làm gì, đóng vai trò nào trong các dự án»* — trong bối cảnh *«CLI + model phải nằm
ngoài mọi luồng công việc, huba là cổng để điều hướng»*.

**Phép đo** (chỉ đọc, HEAD huba `e222222` + sửa dở chưa biên dịch ở `accounts.rs`/`sessions.rs`):
- Định nghĩa Rust (`fn/struct/enum/const/static/mod/type`) có tên chứa
  `claude|codex|opencode|model|cli|deepseek|gpt|chatgpt` trong `huba/rust/src`: **155**
  (gồm bài kiểm; 6 cái là trùng chữ, không liên quan — mục 0).
- Số dòng nhắc `codex|opencode|claude` theo tệp: `sessions.rs` 576 · `accounts.rs` 414 ·
  `pipeline.rs` 215 · `opencode.rs` 182 · `keys.rs` 158 · `quota.rs` 133 · `account_login.rs` 117 ·
  `runtime.rs` 106 · `config.rs` 97 · `accounts_page.rs` 79 …
- Chỗ RẼ NHÁNH `is_codex()/is_opencode()/is_claude()` trong `sessions.rs` + `pipeline.rs`: **17**.
- Script dùng chung `~/projects/scripts`: 11 tệp (mục 9).

Cột **Vai** đối chiếu với sơ đồ đích (huba = một luồng công việc; CLI chỉ ở bộ chuyển):
- **LUỒNG** — tên CLI nằm TRONG luồng công việc (mở / chờ id / bàn giao rẽ theo CLI).
- **CHUYỂN** — chi tiết riêng của một CLI (dòng lệnh, sổ, màn hình). Bản chất phải có, chỉ cần nằm sau một giao diện chung.
- **TÊN SAI** — dùng chung cho mọi CLI nhưng mang tên một CLI (thường là «claude»).

---

## 0. Trùng chữ — KHÔNG liên quan CLI/model
`telegram.rs:2442 client` (HTTP client) · `main.rs:20 struct Cli` (bộ đọc tham số dòng lệnh của huba) ·
`browser.rs:498 sc_click` · `bin/browser_mcp.rs:305 result_click` · `win_procs.rs:43 CONSOLE_CLIENTS`,
`:72 is_console_client` (tiến trình console Windows).

## 1. Cấu hình tài khoản — nguồn mọi nhánh rẽ
| Định danh | Làm gì | Vai |
|---|---|---|
| khoá `claude_accounts` (huba.config.json) · `config.rs:800 ClaudeAccountCfg` | Danh sách MỌI tài khoản — cả codex, opencode | TÊN SAI |
| `config.rs:952 claude_accounts_or_ambient` | Liệt kê tài khoản, thêm tài khoản mặc định của máy | TÊN SAI |
| trường `cli` · `config.rs:889 is_opencode` · `:903 is_codex` · `:914 is_claude` · `:919 cli_name` | Quyết định hàng này chạy CLI nào — mọi `if` rẽ nhánh đọc từ đây | CHUYỂN (bị gọi trong LUỒNG) |
| trường `model` · `opencode_config` · `env_file` · `api_key_env` · `config_dir` | Model; cấu hình riêng opencode; tệp/biến khoá; thư mục sổ (CLAUDE_CONFIG_DIR / CODEX_HOME / phân vùng opencode) | CHUYỂN |
| khoá `claude_cli` · `opencode_cli` · `codex_cli` · `config.rs:782/786/790 default_*_cli` | Đường tới binary của từng CLI | CHUYỂN |
| `config.rs:963 claude_transcript_root` | Gốc nhật ký phiên Claude | CHUYỂN |

## 2. Dựng dòng lệnh mở + tường quyền
| Định danh | Làm gì | Vai |
|---|---|---|
| `accounts.rs:194 codex_launch_line` · `:132 CODEX_WALL_FLAGS` · `:149 CODEX_WALL_CONFIG` · `:190 CODEX_EFFORT_MAC_DINH` | Dòng lệnh codex + cờ tường (chỉ ghi trong thư mục, không mạng) + effort mặc định | CHUYỂN |
| `accounts.rs:547 ensure_codex_wall` · `:488 codex_hooks_json` · `:426 codex_gate` · `:453 codex_mcp_bin` · `:154 codex_mcp_codetrail_args_cho` | Dựng + kiểm tường codex: hook PreToolUse gọi cổng bảo mật chung, gắn MCP codetrail | CHUYỂN (tên `codex_gate` = TÊN SAI: nó trỏ cổng CHUNG) |
| `accounts.rs:1439 opencode_launch_line` · `:1555 _ps` (Windows) · `:660 opencode_xdg_assigns` · `:1635 opencode_env_for_child` · `:1423 opencode_private_server` · `:1249 OPENCODE_VARIANT_MAC_DINH` | Dòng lệnh opencode: phân vùng XDG riêng mỗi acc, `--standalone`, biến môi trường, variant mặc định | CHUYỂN |
| `accounts.rs:1079 opencode_config_content` · `:970 opencode_permission_wall` · `:1010 opencode_security_plugin` | `OPENCODE_CONFIG_CONTENT`: tường quyền suy từ `DENIED_TOOLS` + plugin cổng bảo mật + MCP | CHUYỂN |
| `accounts.rs:78 opencode_slug` · `:101 opencode_profile_dir` · `:114 codex_home` | Thư mục sổ riêng của từng acc opencode/codex | CHUYỂN |
| `sessions.rs:10104 opencode_terminal_command` · `:10066 chuan_bi_mo_opencode` | Dòng lệnh cửa sổ opencode; việc phải làm trước khi mở (giết dịch vụ mang cấu hình cũ) | CHUYỂN |

## 3. Mở phiên · chờ id · bàn giao — chỗ CLI chen vào LUỒNG
| Định danh | Làm gì | Vai |
|---|---|---|
| `pipeline.rs:15786` (nhánh `r.is_codex()` trong route `/new`) | `/new` acc codex rẽ sang đường riêng ngay ở tầng route | LUỒNG |
| `sessions.rs:8416` (`start_in_terminal` → `start_opencode_in_terminal` :8517) | `/new` acc opencode rẽ sang hàm mở riêng | LUỒNG |
| `sessions.rs:9812 open_codex_window` · `:9890 CODEX_CHUA_MO_PHIEN` | Mở codex; câu từ chối cho các đường mở KHÁC `/new` trên codex | LUỒNG |
| `sessions.rs:8613 wait_opencode_session` · `:9278 wait_for_codex_session_id` · `:9258 codex_successor_on_tty` · (claude: `:8789 wait_for_new_session_id`) | Ba cách chờ phiên chào đời, mỗi CLI một cách | LUỒNG |
| `sessions.rs:9551` (bàn giao rẽ `is_codex()`) | Bàn giao sang codex đi nhánh riêng; opencode rơi vào nhánh Claude (TODO.md đã ghi lỗi) | LUỒNG |
| `sessions.rs:7411 claude_failed` | Câu lỗi khi tiến trình mở phiên thoát ≠ 0 | TÊN SAI |

## 4. Nhìn phiên đang chạy (`/session`, theo dõi)
| Định danh | Làm gì | Vai |
|---|---|---|
| `sessions.rs:5077 list_account_cli` · `:1731 is_claude_process` · `:5392 unlisted_claude_processes` | Hỏi CLI của từng acc đang chạy gì; nhận tiến trình claude; cửa sổ có CLI mà chưa có hàng | CHUYỂN / TÊN SAI (`unlisted_claude_processes` nay đếm cả CLI khác) |
| `sessions.rs:2078 is_opencode_tui` · `:2092 opencode_project_label` · `:5870 opencode_live_session` · `:9774 listed_in_opencode` · `:9796 is_opencode_session_id` | Nhận TUI opencode trong `ps`, nhãn dự án, chuyển hàng opencode về khuôn chung | CHUYỂN |
| `sessions.rs:6738 codex_rows` · module `codex_sessions.rs` (`:28 is_codex_tui`) | Hàng `/session` cho acc codex | CHUYỂN |
| module `opencode.rs` (182 dòng nhắc) — vd `:1163 parse_model`, `:389 cau_hoi_treo_kieu_claude`, `:754 limit_context_tu_api_model` | Đọc kho/API opencode: phiên, model, câu hỏi treo (đổi sang khuôn `AskUserQuestion` của Claude), cửa sổ ngữ cảnh | CHUYỂN (khuôn chung đang mang tên Claude) |
| `sessions.rs:2961 cli_turn_known` · `:2972 cli_turn_started` | Phiên đang chạy lượt không, bắt đầu khi nào — theo lời CLI | CHUYỂN |
| `session_identity_history.rs` · `session_identity_reducer.rs` | Gộp danh tính phiên (dự án/module) từ mọi nguồn qua một luật chung | chung (tên trung tính) |

## 5. Gõ / đọc màn hình
| Định danh | Làm gì | Vai |
|---|---|---|
| `keys.rs:2461 cli` · `:2497 is_claude` | Tab này có chạy CLI trợ lý / có phải `claude` | CHUYỂN |
| `keys.rs:960 codex_box` | Nhận ô nhập codex (không có viền như Claude) | CHUYỂN |
| `keys.rs:3005 send_exit_opencode` · `:3021 OPENCODE_EXIT_CTRL_C` | Cách thoát TUI opencode | CHUYỂN |
| `keys.rs:6132 parse_choices` (tên trung tính, nhưng chỉ hiểu hộp có số kiểu Claude cũ) | Đọc hộp chọn — hỏng với hộp trust claude 2.1.326 không đánh số (ca 17/11) | CHUYỂN lẫn vào chung |
| `pipeline.rs:5719 opencode_shot_say` · `:6776 EXITED_CLI_MARK` | Câu `/shot` riêng opencode; dấu `claude` in khi thoát | CHUYỂN |

## 6. Đăng nhập tài khoản (`/accounts <acc> -l`)
`account_login.rs`: `:138 CODEX_LOGIN_ARGS` · `:141 la_man_codex` · `:149 codex_device` · `:235 codex_after` ·
`:251 guide_codex` · `:473 codex_status` · `:510 start_codex` · `:197 OPENCODE_LOGIN_ARGS` · `:200 la_man_opencode` ·
`:205 opencode_device` · `:220 opencode_after` · `:529 start_opencode` · `:559 opencode_status` · `:352 cli_pid`;
`accounts.rs:654 codex_login_line` · `:693 opencode_login_line` · `:679 opencode_login_provider` · `:707 opencode_auth_list_line`;
`accounts_page.rs:730 mo_khoa_opencode`.
Làm gì: mở cửa sổ đăng nhập đúng CLI, đọc mã thiết bị trên màn, gửi về điện thoại, đọc kết cục. — **CHUYỂN** (mỗi CLI một luồng đăng nhập; Claude đi đường riêng không mang tên).

## 7. Hạn mức · chọn tài khoản · model
| Định danh | Làm gì | Vai |
|---|---|---|
| `quota.rs:1584 codex_quota` · `:1613 codex_quota_tu` · `:1375 codex_auth` · `accounts.rs:785 codex_rate_limits` · `:852 codex_han_muc_say` · `:720 codex_email` · `:738 codex_claims` · `:909 codex_info_lines` · `:954 codex_base_line` | Hạn mức + danh tính acc codex (đọc nhật ký phiên/auth.json) | CHUYỂN |
| `quota.rs:1490 is_chatgpt_opencode` · `:1496 chatgpt_opencode_quota` · `opencode.rs:552 CHATGPT_USAGE_URL` · `:589 chatgpt_rate_limits` · `:612 chatgpt_token` · `:672 chatgpt_email` · `:702 fetch_chatgpt_rate_limits` | Hạn mức acc opencode đăng nhập ChatGPT (hỏi `chatgpt.com/wham/usage`) | CHUYỂN (theo NGUỒN model, không theo CLI) |
| `quota.rs:153 ModelUse` · `:173 ModelUseSnapshot` · `:224 ten_model` · `:259 doc_model_use` · `:317 ModelPct` · `:336 doc_models` | Mức dùng theo model từ `.claude.json`; rút gọn tên model Claude để in | CHUYỂN (chỉ đọc được sổ Claude) |
| `runtime.rs:1020 bo_so_usage_cli_cu` | Bỏ sổ `/usage` cũ khi một hàng đổi CLI | CHUYỂN |

## 8. Effort · chẩn đoán · trust
| Định danh | Làm gì | Vai |
|---|---|---|
| `effort.rs:224 OPENCODE_LEVELS` · `:227 parse_opencode_level` · `:244 opencode_variant` · `:261 set_opencode` | `/effort` cho phiên opencode (variant) | CHUYỂN |
| `accounts.rs:2529 checks_opencode` · `:2545 checks_codex` · `:2596 check_opencode_key` · `:2633 check_opencode_model` · `:2654 check_opencode_config` | `huba doctor` / `/accounts` kiểm từng loại hàng | CHUYỂN |
| `accounts.rs:245 codex_trust_block` · `:266 claude_trust_json` (sửa dở 17/11) | Ghi «tin thư mục» vào sổ codex / sổ Claude | CHUYỂN |
| `runtime.rs:2099 link_cli_onto_path` | Trỏ `~/.local/bin/huba` vào bản dựng — «cli» ở đây là CLI của chính huba | trùng chữ |

## 9. Ngoài huba — script dùng chung `~/projects/scripts` (vai trò với MỌI dự án)
| Tệp | Làm gì cho các dự án | Chỗ mang tên CLI |
|---|---|---|
| `CLAUDE.md` (gốc + mỗi dự án) · thư mục `.claude/` (settings, skills, agents) | Luật mọi phiên phải đọc trước — kể cả phiên codex/opencode (dán qua `mo-dau-phien.py`) | TÊN SAI: luật chung mang tên Claude |
| `mo-dau-phien.py` | Soạn ĐOẠN MỞ ĐẦU cho mọi phiên: xác định acc, dán luật gốc, kiểm MCP codetrail | rẽ `cli == "codex"/"claude"` (`:74-96`, `:190`): đọc `CODEX_HOME` hay `CLAUDE_CONFIG_DIR` |
| `cong_bao_mat.py` + `bao-mat-policy.json` | CỔNG BẢO MẬT DUY NHẤT — mọi quyết định được/không của mọi phiên | adapter: `guard-write-path.py` (hook Claude) · `cong-bao-mat-plugin/index.js` (opencode) · `decide_codex*` `:323,:357` (hook codex) |
| `guard-write-path.py` | Chặn ghi ra ngoài dự án của phiên (hook `PreToolUse`) | chỉ Claude có hook này; CLI khác đi qua cổng bằng adapter riêng |
| `nhac-chuyen-phien.py` · `do-context-phien.py` | Nhắc chuyển phiên khi ngữ cảnh ≥ 50 % | hook Claude; nguồn đọc rẽ theo CLI (nhật ký Claude / codex / `opencode_db`) |
| `acc-mo-vai.sh` (+ `acc-mo-vai-codex.rs`, `-paths.py`, `-linux.py`) | Mở phiên KẾ NHIỆM ở acc tốt nhất cho mọi dự án | 143 dòng nhắc; kiểm trust riêng Claude (`:1151`) và Codex (`:1140`) — từ chối nếu acc chưa trust |
| `phien.sh` · `main-khoi-dong.sh` | Banner mở phiên: bảng tài khoản, hạn mức | in theo loại acc (vd acc8 «opencode Go») |

---

### Đọc bảng thế nào
- **LUỒNG** (mục 3) là chỗ trái nguyên tắc «CLI + model nằm ngoài luồng công việc»: cùng một việc
  (mở phiên, chờ phiên chào đời, bàn giao) được viết 3 lần, mỗi CLI một lần, và rẽ ngay ở tầng route.
- **TÊN SAI** là chỗ dữ liệu/luật chung mang tên một CLI (`claude_accounts`, `ClaudeAccountCfg`,
  `CLAUDE.md`, `.claude/`, `claude_failed`, `codex_gate`, `unlisted_claude_processes`).
- Trust hiện có **ba bản**: huba Codex (`codex_trust_block`), huba Claude (`claude_trust_json`, sửa dở),
  và `acc-mo-vai.sh` (chỉ KIỂM, từ chối) — cùng một quyết định của máy, ba chỗ cài.

---

# Sơ đồ (17/11) — tầng GIAO TIẾP tách khỏi tầng QUẢN LÝ MODEL

Ký hiệu làn: **[Chủ]** chủ máy trên Telegram · **[GT]** tầng giao tiếp · **[QL]** tầng quản lý model ·
**[CV]** bộ chuyển CLI (phần duy nhất biết CLI) · **[CLI]** CLI + model. `⚠ nay:` = mã hiện tại lệch ở bước ấy.

## A. Sơ đồ khối

### A1. HIỆN TẠI (theo mã)
```
 Telegram
    │
┌───▼────────────────────────────┐
│ pipeline (route)               │
│  /new ─┬─ codex? ─▶ mở codex   │ ← rẽ CLI ở route
│        └─ còn lại ▼            │   pipeline.rs:15786
├────────────────────────────────┤
│ sessions                       │
│  mở: claude│opencode│codex     │ ← 3 hàm mở
│  chờ id: 3 hàm                 │   :8613 :8789 :9278
│  bàn giao: rẽ codex            │   :9551
│  kiểm khoá / hạn mức           │ ← QL lọt vào GT
├────────────────────────────────┤
│ keys (gõ / đọc màn)            │
│  hộp claude · ô codex ·        │ ← GT biết CLI
│  phím thoát opencode           │
├────────────────────────────────┤
│ accounts · opencode · quota    │
│  dòng lệnh + tường + trust +   │ ← trộn trong
│  danh tính + hạn mức + phiên   │   cùng module
└────────────────────────────────┘
 + script chung tự rẽ CLI:
   mo-dau-phien · nhac-chuyen-phien
   · acc-mo-vai (trust bản thứ 3)
```

### A2. ĐÍCH
```
      Chủ máy (Telegram)
             │ lệnh / nút
┌────────────▼───────────────────┐
│ TẦNG GIAO TIẾP                 │
│  Cổng route (chat_id)          │
│  Điều phối phiên:              │
│   mở · gõ · đọc · bàn giao ·   │
│   đóng · theo dõi · báo        │
│  Ghép đề bài (+ luật chung)    │
│  Cửa sổ (tmux / Terminal)      │
│  CHỈ biết: PHIÊN, CỬA SỔ       │
└────────────┬───────────────────┘
             │ giao diện chung (A3)
┌────────────▼───────────────────┐
│ TẦNG QUẢN LÝ MODEL             │
│  Sổ tài khoản · đăng nhập      │
│  Hạn mức · chọn tài khoản      │
│  Model · effort                │
│  Chuẩn bị máy: trust · tường · │
│   MCP (cổng bảo mật chung)     │
│ ┌────────────────────────────┐ │
│ │ BỘ CHUYỂN CLI              │ │
│ │ claude │ codex │ opencode  │ │
│ └────────────────────────────┘ │
└────────────┬───────────────────┘
             │
     CLI + model (thay được)
```

### A3. Giao diện chung giữa hai tầng (GT chỉ gọi những câu này)
| Câu hỏi của GT | QL / CV trả về |
|---|---|
| chọn tài khoản (yêu cầu, tài khoản chỉ định?) | tài khoản, hoặc lý do không có |
| đặc tả mở (tài khoản, thư mục, đề bài) | dòng lệnh + env — máy ĐÃ chuẩn bị (trust, tường) |
| màn này đang ở đâu (tty) | ô nhập trống / có chữ · hộp chọn (các lựa chọn) · đã thoát |
| ai đang ngồi trên cửa sổ (tài khoản, tty) | id phiên · chưa chào đời |
| đọc phiên (id) | trạng thái chuẩn: đang chạy / xong lượt / câu hỏi treo / nội dung cuối |
| hỏi chỉ đọc (id, câu) | câu trả lời từ bản fork, phiên gốc nguyên vẹn |
| đặt effort (id, mức) | cách đặt của CLI ấy (cờ lúc mở / phím trong phiên) |

## B. Luồng dữ liệu (ĐÍCH)
```
 [Telegram]
   │ update
   ▼
 (tin_den) ──lệnh──▶ Cổng route ─────────────┐
 huba.sqlite                                 │ yêu cầu mở /
                                             │ gõ / đọc
   ┌─────────────────────────────────────────▼──┐
   │ Điều phối phiên (GT)                       │
   └──┬───────────▲──────────────┬───────────▲──┘
      │ câu hỏi A3 │ trạng thái   │ phím      │ màn
      ▼            │ chuẩn        ▼           │
   ┌──────────────┴──┐       ┌───────────────┴──┐
   │ QL + Bộ chuyển  │       │ Cửa sổ (tmux)    │
   └─┬──────▲─────┬──┘       └───────┬──────────┘
     │ ghi  │đọc  │đọc               │ chạy
     ▼      │     │                  ▼
 (sổ CLI theo acc)│               [CLI+model]
  .claude.json    │                  │ ghi
  CODEX_HOME      │                  ▼
  .opencode-accN  └──────────── (nhật ký phiên)
  trust·đăng nhập·              .jsonl · rollout ·
  hạn mức                        opencode db

 Kho của huba:
  (huba.config.json) sổ tài khoản, gốc workspace ─▶ QL
  (huba.env) khoá — chỉ tên biến trong cấu hình ─▶ QL
  (CLAUDE.md) luật chung ─▶ GT ghép đề bài
  (huba.sqlite) runs · cursors · spend · tin_den ◀─ GT
 Đi ra Telegram: chỉ khi trạng thái ĐỔI (GT).
```

## C. Luồng UC (ĐÍCH, ghi chỗ mã hiện tại lệch)

### UC-1 `/new` — mở phiên mới
1. [Chủ] `/new -s <dự án> "<việc>"` (có thể `-a <acc>`)
2. [GT] kiểm chat_id · ghi `tin_den` · tách cờ
3. [QL] chọn tài khoản (chỉ định, hoặc theo hạn mức); acc khoá ⇒ hỏi chủ — `⚠ nay:` codex rẽ riêng ngay ở route `pipeline.rs:15786`
4. [QL] chuẩn bị máy: trust gốc workspace · tường quyền · MCP — `⚠ nay:` trust chỉ có cho codex; Claude thì đọc màn rồi bấm hộ (hỏng 17/11)
5. [CV] đặc tả mở: dòng lệnh + env theo CLI của acc
6. [GT] ghép đề bài: việc + «đọc CLAUDE.md» + luật dự án
7. [GT] mở cửa sổ ở gốc workspace
8. [CV] đọc màn: có hộp chặn ⇒ [GT] báo chủ kèm màn, KHÔNG gõ — `⚠ nay:` thiếu bước này ⇒ gõ đề bài vào hộp trust, Enter chọn «No» (acc3)
9. [CV] tìm id phiên trên tty — `⚠ nay:` 3 hàm chờ; lưới đỡ nhận dòng tạm `win-<tty>` làm id
10. [GT] gắn id ↔ cửa sổ · chuyển con trỏ · báo «đã mở»; không chào đời ⇒ báo lý do, không báo «đã mở»

### UC-2 `/tell` (chữ thường) — nói với phiên đang theo
1. [Chủ] gõ chữ
2. [GT] lấy phiên đang theo (con trỏ)
3. [CV] màn: ô nhập trống? hộp chọn đang mở?
4. [GT] có hộp ⇒ báo, không gõ; ô sẵn ⇒ gõ + Enter
5. [CV] đọc lại: chữ đã rời ô chưa ⇒ [GT] xác nhận (react / sửa dòng ack)
6. [CV] lượt xong (theo nhật ký) ⇒ [GT] «vừa xong» MỘT lần

### UC-3 `/shot` — xem màn
1. [GT] chụp cửa sổ của phiên
2. [CV] tách dòng lệnh · hộp chọn · ô nhập (theo cách CLI vẽ)
3. [GT] định dạng qua MỘT cửa (`say_from_session`) + nút ▶️/🖥/☑ ⇒ Telegram

### UC-4 Bàn giao / chuyển phiên (ngữ cảnh ≥ 50 % hoặc `/handover`)
1. [CV] nhắc ≥ 50 % (hook/plugin theo CLI) — hoặc [Chủ] `/handover`
2. [GT] xin bản bàn giao: [CV] fork CHỈ ĐỌC phiên cũ theo CLI
3. [QL] chọn acc mới theo hạn mức
4. [GT] chạy UC-1 bước 4–10, đề bài = bản bàn giao — `⚠ nay:` `sessions.rs:9551` rẽ codex; opencode rơi nhầm nhánh Claude
5. [GT] phiên mới sống ⇒ đóng phiên cũ; không ⇒ GIỮ phiên cũ, báo

### UC-5 `/accounts <acc> -l` — đăng nhập tài khoản
1. [QL] lệnh đăng nhập theo CLI (`claude /login` · `codex login --device-auth` · `opencode auth login`)
2. [GT] mở cửa sổ đăng nhập
3. [CV] đọc mã thiết bị / đường dẫn trên màn ⇒ [GT] gửi chủ
4. [Chủ] nhập mã trên web
5. [CV] đọc kết cục ⇒ [QL] cập nhật sổ, kiểm danh tính trùng

### UC-6 `/effort <mức>`
1. [QL] chuẩn hoá mức
2. [CV] cách đặt theo CLI: cờ lúc mở (codex) · phím đổi variant (opencode) · lệnh trong phiên (claude)
3. [GT] gõ nếu cần · báo — `⚠ nay:` `effort.rs:261 set_opencode` gõ phím thẳng, không qua A3

### UC-7 `/ask` — hỏi chỉ đọc
1. [GT] câu hỏi + phiên đích
2. [CV] fork CHỈ ĐỌC theo CLI (`Read,Grep,Glob`)
3. [GT] gửi câu trả lời; phiên gốc cùng byte, cùng mtime
