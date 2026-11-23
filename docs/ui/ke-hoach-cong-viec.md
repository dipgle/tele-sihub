# Kế hoạch UI desktop — việc nhỏ, mỗi việc một phép đo

Nguồn: `docs/thiet-ke-desktop.md` §3 (màn), §6 (khoảng thiếu), §8 (thứ tự). Chủ máy 23/11: *«bóc tách kế hoạch và công việc chi tiết hơn
nữa, mỗi việc phải càng nhỏ nhanh ngắn càng tốt»* · chọn bắt đầu bằng **khung Tauri**.

Quy ước: mỗi việc ≤ ~15 phút (ước lượng của tôi, **chưa đo** — việc nào quá thì tách tiếp), có **«xong khi»** đo được. 🅲 = bạn làm
(cần mắt/tay/mật khẩu) · 🅷 = tôi làm · ⛔ = chờ bạn quyết. Việc đổi mã lõi (`rust/`) luôn có test đỏ → xanh. Thứ tự: xong cả khối mới sang khối sau.

> **Trạng thái 23/11:** W1.1–W1.8 XONG. `rust/src/uiserver.rs` (hàm thuần + `serve_loop`) · `tests/ui_server_security.rs` 18 ca ·
> `tests/ui_server_wire.rs` 5 ca qua ổ cắm thật · `localserver::read_request` có trần đầu 64 KiB / thân 1 MiB (2 ca đỏ→xanh).
> Đột biến: gỡ từng hàng rào (Host · Origin · trần thân · cookie · vé một lần · chặn thoát `ui/` · chặn `..` · HttpOnly) ⟹ test ĐỎ cả 8
> (một đột biến sống sót lần đầu — `..` — đã siết test rồi bắt được). Chạy thật `huba ui` + client python độc lập: 10/10 đúng.
> **Chưa làm:** W2 (API phiên) và mọi thứ sau; chưa chạy cổng `gate.sh` đầy đủ; chưa nghiệm thu bằng mắt trên trình duyệt thật.

## PHƯƠNG ÁN W — giao diện là trang web do huba phục vụ (thay K0–K2 nếu bỏ Tauri)

**Cách hoạt động (đều dựa trên mã đã có, đã đọc 23/11):**
- `huba ui` = lệnh mới, cùng khuôn `huba accounts`: mở cổng **127.0.0.1** (đã có `localserver::bind_loopback`), phục vụ tệp tĩnh + API JSON.
  Giai đoạn đầu chạy như tiến trình riêng (đọc snapshot bằng thư viện huba); khi cần gửi lệnh mới nhúng vào `hubad` để dùng chung hàng lệnh với Telegram.
- Cửa sổ: dùng `localserver::open_window` đã có — macOS mở Chrome/Edge/Brave bằng `--app` (cửa sổ riêng, không thanh địa chỉ), Windows `start` (Edge có sẵn),
  Linux `xdg-open`. ⚠ Máy Ubuntu này chỉ có **Firefox** (không Chrome/Chromium): sẽ là tab trình duyệt thường, không phải cửa sổ riêng.
- Giao diện: **tái dùng bản mẫu đã có** `docs/ui/{index.html,app.js,style.css,reports.js,vendor}` (ECharts offline; ảnh `previews/`, đã qua Playwright Chromium+WebKit 3/3 ngày 15/11),
  thay dữ liệu mẫu bằng gọi `/api/*`. Không dựng lại bằng React ở bước đầu.
- An toàn (vé hiện tại = giờ hệ thống + địa chỉ con trỏ, không đủ cho app sống lâu — thiết kế §6): token ngẫu nhiên từ hệ điều hành, đổi vé URL → cookie
  `HttpOnly; SameSite=Strict`; kiểm `Host` (chống DNS-rebinding) và `Origin` khi POST; giới hạn body; chặn `..`.
- Chọn thư mục native: trình duyệt không trả đường dẫn thật ⟹ `hubad` gọi hộp thoại hệ điều hành (Windows đã có mẫu FolderBrowserDialog ở `cai.ps1`).
- Đường lui sang Tauri: vỏ Tauri chỉ trỏ vào `http://127.0.0.1:<cổng>`, **frontend không đổi dòng nào**.

**Mất so với Tauri:** khay hệ thống, thông báo native (dùng Telegram + Web Notifications), cửa sổ riêng trên Linux không có Chrome, tự cập nhật theo kiểu app.
**Được:** không `sudo`, không thư viện WebView, không thêm tệp `.exe` chưa ký (Windows Smart App Control đã chặn `huba.exe`), Playwright kiểm thật trên Chromium/WebKit.

| # | Việc | Xong khi | Ai |
|---|---|---|---|
| W0.1 | Đóng tab cài `sudo` đang mở (không cần nữa) | tab đóng | 🅲 |
| W0.2 | Đọc `docs/ui/app.js`: liệt kê chỗ nào đang dùng dữ liệu mẫu | danh sách `file:line` ghi vào sổ này | 🅷 |
| ✅ W1.1 | Test ĐỎ: `GET /api/health` không cookie ⟹ 401 | test đỏ đúng lý do | 🅷 |
| ✅ W1.2 | Token 128-bit từ hệ điều hành (`getrandom` đã có trong `Cargo.lock`, chưa là phụ thuộc trực tiếp — kiểm bản và thêm) | hai token liền nhau khác nhau, dài 32 hex | 🅷 |
| ✅ W1.3 | Vé URL một lần → cookie `HttpOnly; SameSite=Strict` | dùng lại vé lần hai ⟹ 401 (đối chứng ngược) | 🅷 |
| ✅ W1.4 | Kiểm `Host` = `127.0.0.1:<cổng>` | `Host: evil.com` ⟹ 403; đúng ⟹ 200 | 🅷 |
| ✅ W1.5 | POST kiểm `Origin` | Origin lạ ⟹ 403 | 🅷 |
| ✅ W1.6 | Giới hạn body 1 MiB | body 2 MiB ⟹ 413; 1 KiB ⟹ 200 | 🅷 |
| ✅ W1.7 | Phục vụ tệp tĩnh từ thư mục `ui/` cạnh binary | `GET /` ra index.html; `GET /../x` ⟹ 400/404 | 🅷 |
| ✅ W1.8 | Lệnh `huba ui`: mở cổng, in URL, gọi `open_window` | chạy tay thấy trang mở | 🅷 + 🅲 liếc |
| W2.1 | `/api/sessions` trả JSON snapshot thật | số phiên = số phiên của `huba sessions` | 🅷 |
| W2.2 | Nối danh sách phiên của bản mẫu vào API | ảnh chụp có phiên thật | 🅷 |
| W2.3 | Ba trạng thái rỗng / lỗi-dữ liệu cũ / bình thường | giả lỗi đọc ⟹ «Dữ liệu cũ», không hiện rỗng | 🅷 |
| W2.4 | Playwright Chromium + WebKit: mở bằng cookie, N hàng = API, 0 lỗi console | exit 0 cả hai engine | 🅷 |
| W3.1 | Linux: ưu tiên `chromium`/`google-chrome` `--app` nếu có | máy có Chrome ⟹ cửa sổ riêng; máy chỉ Firefox ⟹ rơi về `xdg-open` (đo cả hai nhánh) | 🅷 |

K3 trở đi giữ nguyên (đọc nhật ký, gửi…), chỉ khác là gọi qua `/api/*`.

## K0 — Điều kiện máy (Linux, máy này) — CHỈ khi chọn Tauri
| # | Việc | Xong khi | Ai |
|---|---|---|---|
| K0.1 | Cài 7 gói apt (WebKitGTK, GTK3, libsoup, librsvg, appindicator, patchelf) | log `.tmp/cai-tauri-deps.log` có `DEPS_EXIT=0` | 🅲 gõ mật khẩu (tab đã mở) |
| K0.2 | Kiểm thư viện thấy được | `pkg-config --modversion webkit2gtk-4.1` ra số | 🅷 |
| K0.3 | Thử mạng tải crate (cargo có vướng IPv6 như npm không) | `cargo fetch` một crate Tauri xong, không treo | 🅷 |
| K0.4 | Cài `tauri-cli` 2.x (chạy nền) | `cargo tauri --version` ra 2.x | 🅷 |
| K0.5 | Chốt chỗ đặt: `huba/desktop/`, workspace Cargo riêng, KHÔNG đụng crate `rust/` | thư mục + `Cargo.toml` rỗng, `cargo metadata` đọc được | 🅷 |

## K1 — Khung chạy được (spike Linux) — §8 bước 1
| # | Việc | Xong khi | Ai |
|---|---|---|---|
| K1.1 | `cargo tauri init` + Vite + React-TS vào `desktop/` | `cargo tauri info` không báo thiếu | 🅷 |
| K1.2 | Chuyển `desktop/` sang pnpm: `package.json` + `pnpm-lock.yaml`, cài bằng `cong-node cai` (luật gom một chỗ) | `node_modules` nhập hardlink từ `.pnpm-store`, `quet-muon-node-modules.py` vẫn sạch | 🅷 |
| K1.3 | Dựng debug lần đầu (chạy nền, có thể lâu) | tệp `target/debug/<app>` có; ghi thời gian dựng | 🅷 |
| K1.4 | Mở cửa sổ trống | tiến trình sống ≥ 10 s + ảnh chụp màn hình có cửa sổ | 🅷 + 🅲 liếc ảnh |
| K1.5 | Header toàn chiều ngang: logo + «huba» (tĩnh) | ảnh khớp mẫu `docs/ui/index.html` | 🅷 |
| K1.6 | Menu trái 8 mục (Chat · Phiên · Báo cáo token · Tài khoản & model · Kết nối · Dự án · Hoạt động · Cài đặt), hàng đầu = nút thu gọn | thu/mở không làm header nhảy (đo chiều rộng header trước/sau) | 🅷 |
| K1.7 | Token giao diện: nền sáng trung tính, MỘT màu nhấn xanh, chữ + icon cho trạng thái | tệp token CSS duy nhất; không còn màu viết cứng ngoài tệp đó (grep) | 🅷 |

## K2 — Nối lõi, chỉ ĐỌC (danh sách phiên thật) — §6 dòng `sessions.rs`
| # | Việc | Xong khi | Ai |
|---|---|---|---|
| K2.1 | Đọc mã: hàm công khai nào trả snapshot phiên (`sessions.rs:5834` quanh đó) | ghi `file:line` + chữ ký hàm vào sổ này | 🅷 |
| K2.2 | `desktop/src-tauri` phụ thuộc crate `huba` bằng đường tương đối | `cargo build` xanh, 0 cảnh báo mới | 🅷 |
| K2.3 | Lệnh Tauri `ping` → trả phiên bản huba | frontend hiện đúng chuỗi phiên bản | 🅷 |
| K2.4 | Lệnh `list_sessions` gọi hàm ở K2.1, trả JSON | gọi từ frontend ra đúng **số phiên** như `huba sessions` (so hai số) | 🅷 |
| K2.5 | Hiện JSON thô trong `<pre>` | thấy cùng số phiên trên cửa sổ | 🅷 |
| K2.6 | Bảng phiên: cột STT liên tục, nền chẵn/lẻ, tìm + phân trang trên TOÀN tập | đổi trang STT vẫn nối tiếp; gõ tìm lọc đúng | 🅷 |
| K2.7 | Ba trạng thái bắt buộc: rỗng («Chưa có phiên» + nút Mở phiên) · lỗi/dữ liệu cũ (giữ ảnh trước, KHÔNG báo 0 phiên) · bình thường | **đối chứng ngược**: giả lỗi đọc ⟹ hiện «Dữ liệu cũ», không hiện rỗng | 🅷 |
| K2.8 | Tự làm mới 5 s + mốc «đọc lúc hh:mm:ss» | mốc đổi đúng nhịp; mở 2 phiên thật ⟹ số hàng đổi | 🅷 |

## K3 — Chat đọc một phiên — §3 «Chat», §6 `sessions.rs:6961`
| # | Việc | Xong khi | Ai |
|---|---|---|---|
| K3.1 | Lệnh `read_transcript(id, giới hạn)` dùng bộ đọc hiện có; ghi rõ nó **cắt/bỏ record lạ** | số lượt = số lượt trong nhật ký thật (đếm bằng `jq`) | 🅷 |
| K3.2 | Timeline: user/assistant text, Unicode, đa dòng | một phiên thật hiện đủ, không vỡ tiếng Việt | 🅷 |
| K3.3 | Markdown an toàn + nút copy code; KHÔNG chạy HTML/JS của model | chuỗi `<script>` trong nội dung hiện thành chữ (ca âm) | 🅷 |
| K3.4 | Ngăn phải 3 tab: Phiên / Nội dung gốc / Hoạt động (khung, chưa có dữ liệu) | 3 tab chuyển được, không lỗi console | 🅷 |
| K3.5 | ⛔ Tab «Nội dung gốc»: mức raw cần lấy | chờ bạn chốt «mức gốc» (§2) — **không tự suy** | 🅲 |

## K4 — Gửi một câu từ desktop — §6 `pipeline.rs:8242/11285` (hàm private)
| # | Việc | Xong khi | Ai |
|---|---|---|---|
| K4.1 | Đọc 2 hàm gửi hiện có, liệt kê rào cần giữ (kiểm đích, kiểm nhật ký) | danh sách rào có `file:line` | 🅷 |
| K4.2 | Test ĐỎ: hàm gửi dùng chung chưa tồn tại ngoài pipeline | test biên dịch/chạy đỏ đúng lý do | 🅷 |
| K4.3 | Tách `send_to_session` công khai, pipeline Telegram gọi lại | test xanh + toàn bộ test Telegram cũ vẫn xanh | 🅷 |
| K4.4 | Lệnh Tauri `send` + ô nhập; hiện «đã gửi tới bước nào» | gõ từ desktop ⟹ nhật ký phiên có đúng câu | 🅷 |
| K4.5 | Kết quả mơ hồ KHÔNG tự gửi lại; giữ nguyên bản nháp | giả «mất ACK» ⟹ bản nháp còn, không gửi đôi | 🅷 |
| K4.6 | Kiểm chéo desktop ↔ Telegram: gõ ở app, thấy 💤 ở Telegram | log hubd có tin `session_change` của lượt đó | 🅷 + 🅲 liếc |

## K5+ — chưa bóc nhỏ (bóc khi tới, mỗi khối một lượt)
Báo cáo token (tái dùng `docs/ui/reports.js` + ECharts offline) · Tài khoản & model (tách khỏi `accounts_page.rs`) · Kết nối/wizard
Telegram (không restart, không poll đôi) · Dự án (chọn thư mục native) · Hoạt động · Cài đặt (có revision) · Đóng gói `.deb` +
Windows + Mac (cần SDK, xem `huba/.tmp` ghi chú zigbuild) · E2E bằng UI thật trên WebView thật (Playwright không lái được WebKitGTK ⟹ phải dùng WebDriver hoặc người nhìn — chưa đo).

## Điều chưa biết (không đoán)
- Mức «gốc» chủ máy cần (§2) — chặn K3.5 và mọi thứ sau.
- Thời gian dựng Tauri lần đầu, và cargo có tải được crate qua IPv6 hỏng không (K0.3 sẽ đo).
- Cách nghiệm thu giao diện Tauri tự động trên Linux (WebKitGTK không có Playwright) — tạm dùng ảnh chụp màn hình + người xem.
