# Bản mẫu thiết kế huba desktop

Mở [index.html](index.html) để xem bản mẫu. Không cần build; HTML, CSS, JavaScript nằm cùng thư mục, không dùng CDN. Bản mẫu không gọi mạng, CLI, model, MCP hay Telegram; dữ liệu hoàn toàn minh họa, thay đổi chỉ nằm trong bộ nhớ. Không nhập token hoặc thông tin tài khoản thật.

- Menu trái mở tám khu vực; hàng đầu thu/mở menu.
- Chat: chọn phiên, soạn tin, đổi phiên vẫn giữ draft riêng; xem Nội dung gốc/Hoạt động; mở luồng ngắt/đóng/bàn giao.
- Phiên, tài khoản, dự án, hoạt động: tìm toàn danh sách; bảng có STT/nền xen kẽ/phân trang.
- Tài khoản: mở Quản lý để xem form, trạng thái khoá và luồng đăng nhập minh họa.
- Kết nối: đi hết ba bước Telegram, xem mô tả nguồn/quyền MCP.
- Thanh dưới chọn mặc định/chờ lựa chọn/rỗng/lỗi/mất kết nối. Đây là bộ chọn để duyệt thiết kế, không thuộc UI sản phẩm cuối.
- Báo cáo token: input/output/cache theo ngày/model/account/phiên, bộ lọc chung, drill-down usage và CSV mẫu. Chỉ thống kê token từ CLI/plugin; không nối billing/tracker hay dựng thêm dịch vụ phân tích. Mọi số đều là fixture, có một lượt cố ý thiếu usage.

Thiết kế kiến trúc, contract và luồng: [thiet-ke-desktop.md](../thiet-ke-desktop.md). Đề xuất sản phẩm là frontend chung trong Tauri + lõi Rust dùng chung, adapter OS/CLI/kênh tách riêng. Bản mẫu HTML này chưa phải frontend Tauri đã triển khai.

Luồng chi tiết: [bản vẽ HTML offline](../flows/index.html), [Markdown/Mermaid](../flows/index.md). Có 30 thành phần, 16 UC và 13 sơ đồ; chỉ rõ dữ liệu qua chat/MCP/model, context của CLI và các phần của một kết quả. Tài liệu riêng, không thay bản mẫu tám màn hoặc HTML đã gửi1811.

Kiểm prototype trên host ngày 15/11/2026 09:50 +07: syntax 3 JS mã0; Playwright **3/3**, console/page errors0 trong các luồng đã kiểm; `GATE_EXIT=0` ở `.tmp/ui-review/host-review.log`. Đã xem ảnh desktop, màn hẹp, hover, rỗng, lỗi và usage chưa biết trong `previews/`. Đã kiểm bộ lọc/tìm/CSV, nguyên tắc cache nằm trong input, 17/18 record đủ và một record thiếu; chỉ có phần biết dùng dấu ≥. Đây là nghiệm thu **bản mẫu fixture**, chưa nối CLI/plugin hoặc nghiệm thu dữ liệu thật.

Bộ kiểm: `.tmp/ui-review/{design.spec.cjs,reports.spec.cjs,playwright.config.cjs,check-js.cjs}`; harness `.tmp/ui-review/run-on-host.py` chạy qua cổng Node và ràng hash HTML đầu/cuối. Ca HTML offline đã đỏ vì bước chèn license thay nhầm chuỗi `</body>` nằm trong JS ECharts; exporter nay hoàn tất template trước khi nhúng script, sau sửa ca này và lượt chốt đều xanh. Sandbox không mở được Chromium; kiểm browser đã chạy headless trên host qua hòm thư huba, không mở browser giao diện.

Bộ regression mang theo Git cho máy khác: `tests/prototype.spec.cjs` + `tests/playwright.config.cjs` (draft/account khoá/điều hướng, báo cáo token/CSV/unknown, HTML offline). Chạy exporter trước, rồi dùng Playwright đã cài qua cổng Node: `bash ~/projects/scripts/cong-node chay e2e -- node <đường-dẫn-playwright-cli> test --config docs/ui/tests/playwright.config.cjs`. Có thể khai `NODE_PATH` trỏ kho gói hiện có và `HUBA_CHROMIUM_EXECUTABLE` nếu dùng Chromium ngoài; không cần cài riêng node_modules trong tài liệu. Các log `.tmp` bên trên là lịch sử, không phải đầu vào bắt buộc của bộ kiểm mới.

Kiểm lại bởi phiên kế nhiệm ngày 15/11/2026 21:14 +07: cùng bộ regression trên **Chromium 3/3 + WebKit 3/3**, mã thoát từng lượt0; assert console/page/request errors0. Log `.tmp/ui-{chromium,webkit}-successor-20261115.log`. Đây chỉ là kiểm chức năng fixture/HTML offline, **không** nghiệm thu UC thật hoặc kiểm hình13 sơ đồ bằng WebKit; lượt này không chụp/xem bộ ảnh WebKit. HTML tự chứa vẫn có hash `3b384ac7…` như bản gửi1811, không gửi lại.

Với môi trường CLI đổi `XDG_CACHE_HOME`, khai `PLAYWRIGHT_BROWSERS_PATH=<kho-browser-đã-cài>` khi chạy để dùng đúng cache. Lượt WebKit đầu chưa mở được browser vì tìm trong cache acc9; đặt đường tới kho browser hiện có rồi cả3 bài qua, không cài thêm gói. Chọn engine bằng `--browser chromium` hoặc `--browser webkit` sau lệnh `test`; không đặt `HUBA_CHROMIUM_EXECUTABLE` cho lượt WebKit.

Xuất tệp HTML tự chứa bằng `python3 docs/ui/export.py` (chạy từ cây huba); đầu ra `.tmp/huba-desktop-design.html`. Tệp xuất đã gộp CSS/JS/ECharts và license, không cần các tệp bên cạnh. Telegram xác nhận bản cập nhật có Báo cáo token ở tin **1811**, **1220808 byte**, SHA256 `3b384ac7d346b5f7cdef83d92ecf1b4a6b0f86d111e831c9e8d5b065cefa36da`; biên nhận `.tmp/gui-html-token-receipt.json`. Bản HTML cũ ở tin1742, Markdown cũ ở tin1736. Đây là xác nhận giao file, không phải nghiệm thu sản phẩm thật.
