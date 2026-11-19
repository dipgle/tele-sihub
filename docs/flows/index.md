# huba V2 — Bàn làm việc nhiều phiên · kiến trúc · luồng chi tiết

THIẾT KẾ V2 ĐỂ BÀN. Không phải sản phẩm đã triển khai. Đỏ = lỗi có bằng chứng; cam = khoảng chưa đo; tím = cần chốt; vàng = rủi ro cần kiểm. Đánh dấu không có nghĩa đã sửa.

huba là cầu nối tới phiên CLI thật. Chủ máy thao tác qua Desktop, Telegram hoặc terminal; CLI vẫn sở hữu vòng gọi model và lắp context. huba chuyển lệnh, kiểm nhận, thu nguồn được CLI cung cấp và trình bày lại — không tạo thêm một agent suy luận đứng giữa.

## Quy tắc đọc
- Ba lớp dùng chung ID để lần từ UC → command → phiên/lượt → request model → raw record → nội dung hiển thị. ID không có từ nguồn phải để chưa xác định, không ghép bằng thời gian gần nhau.
- Đọc sơ đồ từ trên xuống. Hàng vàng/hồng/tím có điều kiện, không phải mọi hàng chạy nối tiếp. Nhánh lỗi thay thế đường chính tại bước đó, không chạy sau thành công; các hàng lỗi liền nhau cùng báo đủ rồi dừng thao tác. Mermaid tách alt/else/break; phục hồi là tương tác mới khi đủ điều kiện. Mũi tên không biểu thị thời gian đo thực tế.
- Desktop và Telegram gọi cùng nghiệp vụ tại hubad; terminal đi thẳng CLI rồi collector quan sát lại. Không có poller Telegram thứ hai, không đẩy mọi sự kiện về mọi kênh nếu chưa được chọn theo dõi.
- Ba thứ khác nhau: chữ người dùng nhập; nhật ký CLI ghi; payload request/response thực gửi tới model. Bản gốc phải mang đúng nhãn nguồn và độ đầy đủ.
- Context model ≠ lịch sử đang hiển thị trên UI ≠ kho raw ≠ tổng token sử dụng. Mở màn/xem raw/lọc báo cáo không tự gọi model hoặc nạp lại prompt.
- Hàng rào quyền chạy ở core và lớp thực thi, không dựa vào model nhớ lời dặn. Nội dung model/tool/raw luôn được coi là dữ liệu không đáng tin khi render; không chạy HTML/JS hay lệnh do nó chứa.
- Chuẩn thiết kế mong muốn có thể chặt hơn mã hiện tại. Nguồn hiện có và khoảng chưa chứng minh được ghi cuối trang; đây không phải bản đồ phụ thuộc máy trích từ code/traces.
- Một phiên có khóa bất biến; dự án/module được giao, tiêu đề hội thoại và thư mục đang đứng là ba trường riêng. Chưa giao dự án thì hiện Phiên chung + ID; đường dẫn trong log không quyết định danh tính.
- Hai vùng dùng cùng phạm vi máy/tài khoản: A là phiên có panel trực tiếp trên bàn này, B là phiên mở còn lại. A giao B rỗng; A hợp B bằng tổng phiên mở đã xác minh. Bộ lọc không được che mất tổng chưa lọc; nguồn mù phải công bố mẫu số chưa xác minh.
- Nhiều cửa sổ có view ID và subscription riêng. Đóng panel không đóng CLI; đóng app không đồng nghĩa tắt dịch vụ. Mọi lệnh mang session key + generation, không dùng focus toàn cục làm đích ngầm.
- B luôn hiện ở đáy khung nhìn; chỉ A cuộn danh sách ô chat. App co theo chiều cao cửa sổ và dành chỗ cố định cho footer, không có chiều cao tối thiểu đẩy B ra ngoài màn hình. Bản thuyết minh dài vẫn là tài liệu; workspace.html mới là bản mẫu khung app để kiểm hành vi này.
- Mục tiêu là không cần mở cửa sổ terminal riêng cho việc thường ngày; không phải bỏ CLI, PTY hay đường tiếp quản. Runtime không có cửa sổ chỉ là đề xuất cần duyệt và đo đủ capability, không tự thay chế độ đang chạy.

## Danh mục UC
| UC | Người dùng / mục tiêu | Input | Output nhìn thấy | Sơ đồ | Điều kiện lỗi / nghiệm thu dự kiến |
| --- | --- | --- | --- | --- | --- |
| UC-01 | Chủ máy mở app / kết nối dịch vụ | Danh tính OS, bản app/core, workspace | Vào màn chính hoặc lỗi đúng bước | U1, D2 | Core lệch phiên bản, không quyền, probe lỗi không được biến thành danh sách rỗng |
| UC-02 | Chọn / thêm dự án | Folder thật, cấu hình mặc định, revision | Dự án đã lưu; luật được nhận diện | U3, C1 | Folder mất/quyền thiếu → không lưu thành sẵn sàng; không tự mở rộng quyền ghi |
| UC-03 | Thêm account / đăng nhập / đổi model | CLI, account, model, lần login | Trạng thái xác thực và phạm vi áp dụng | U3, C1 | Mã hết hạn/lần login cũ không được gửi sang cửa sổ khác; khóa account không xóa lịch sử |
| UC-04 | Mở phiên | Phiên chung hoặc dự án đã giao; account/model; runtime mode đã hỗ trợ | SessionKey, readiness và đường tiếp quản rõ ràng | U1, C1, U7 | Mở được process/pane chưa phải CLI sẵn sàng; không tự đổi mode hoặc bịa tên dự án |
| UC-05 | Gửi tin / nhận phản hồi | Nội dung, attachment refs, session ID, command ID | Đã nhận / chưa xác định / phản hồi nguồn | U2, D2, C2 | Double-click/mất ACK không được gõ lại mù; attachment chưa đọc không được nhận đã gửi đủ |
| UC-06 | Trả lời bảng hỏi nhiều câu | Question ID/revision, các đáp án | CLI ghi nhận lựa chọn, tiếp tục lượt | U2, D2 | Câu hỏi đổi, phiên chết, thiếu đáp án → làm mới/giữ lựa chọn, không bấm bảng cũ |
| UC-07 | Xem hoạt động tool / subagent | Phiên, lượt, tool-call ID, parent-child refs | Hoạt động đang chạy/kết quả/lỗi có nguồn | D1, C2 | Tool lỗi hoặc child chưa kết thúc không được hiện hoàn thành; không gộp output child thành parent gốc |
| UC-08 | Ngắt lượt | Phiên đích, xác nhận, capability | Lượt bị ngắt, phiên còn sống | U2, C2 | Đợi bằng chứng kết thúc/ngắt; không đồng nghĩa giết mọi process do tool đã mở |
| UC-09 | Đóng phiên / đóng cửa sổ app | Đích thao tác và xác nhận | Phiên đóng hoặc app ẩn, tác động được ghi rõ | U2, U4 | Đóng app không mặc định tắt hubad/CLI; không đóng nhầm tty đã tái sử dụng |
| UC-10 | Resume / fork hỏi / bàn giao | Nguồn, đích, checkpoint và capability | Liên kết phiên hoặc nhánh phụ rõ ràng | U4, C3 | Không hỗ trợ thì từ chối đúng nghĩa; không chuyển tài khoản/CLI như thể context nguyên vẹn |
| UC-11 | Kết nối / dùng Telegram | Bot, owner chat, scope theo dõi, lệnh | Nhận/gửi hai chiều qua cùng core | U3, D2 | Sai chat bị từ chối; getMe thành công chưa chứng minh giao tiếp hai chiều |
| UC-12 | Xem nội dung gốc / truy cập MCP | Record ref, client identity, scope | Nguồn đúng loại + coverage + raw theo quyền | D3, C4 | Không có payload thì hiện không cung cấp; MCP không tự nghe toàn bộ giao tiếp model |
| UC-13 | Xem báo cáo token / CSV | Khoảng ngày/múi giờ/account/model/dự án | KPI/chart/bảng/CSV cùng phạm vi | D4, C4 | Usage thiếu không thành0; stream/final, parent-child và cache không cộng đôi |
| UC-14 | Mất kết nối / restart / đọc lịch sử | Cursor, command đang treo, revision | Draft giữ nguyên; khôi phục trạng thái có bằng chứng | U4, D1, D2 | Không tự gửi lại command đã có khả năng thực thi; cursor mất → báo gap, không giả liên tục |
| UC-15 | Sửa cấu hình / quyền | Giá trị mới, expected revision, actor | Revision + audit + thời điểm áp dụng | U3, C1 | Xung đột revision → đối chiếu lại, không ghi đè; model không được tự cấp quyền |
| UC-16 | Xuất dữ liệu / chẩn đoán | Phạm vi, quyền xuất, xác nhận chủ máy | Manifest export và nhãn raw/derivative | D3, D4 | Không tự phát tán raw nhạy cảm; secret transport của huba không vào log hay bản chẩn đoán |
| UC-17 | Sắp ô chat và footer phiên | Scope máy/account, layout, tập session keys | A có chat riêng luôn thấy hoạt động; focus đổi cột phải; B footer full ngang | U5 | Không focus vẫn cập nhật trạng thái/tin/việc đang làm; chưa đủ nguồn thì báo chưa xác minh |
| UC-18 | Nhiều cửa sổ cùng theo dõi/điều khiển | ViewID, subscription, command target | Fanout có cursor; command đúng phiên, không lặp | U6, D2 | Không dùng focusglobal làm target; takeover phải có control lease |
| UC-19 | Nhận trạng thái mới và mất nguồn | Lifecycle event hoặc kết quả đối soát | Trạng thái có bằng chứng và mốc mới nhất | D5 | Mất nguồn không biến thành idle/closed; notification chỉ chuyển trạng thái |
| UC-20 | Giao dự án / sửa tên hiển thị | User choice, confirmed assignment, expected revision | Topic/scope/assignment riêng, identity key giữ nguyên | D6 | Tên nhắc trong log không được giao dự án; đổi tên không cấp quyền hay tự đổi cwd |
| UC-21 | Xem vòng xử lý và child/job | Turn/request/tool/child/job refs + revision | Live hoặc xem lại cùng graph và coverage | C5 | Tool khởi chạy completed không chứng minh job xong; không bịa HTTPrequest từ assistantrow |
| UC-22 | Tiếp quản khi không mở terminal riêng | Runtime mode, capabilities, lease, binding | Terminal nhúng/gốc cùng phiên hoặc từ chối rõ | U7 | Không mở phiên mới giả takeover; chưa có driver attach thì giữ terminal gốc |

## Tổng thể V2

Xem HTML: W0 bàn hai vùng, T0 terminal và các Pxx liên kết tới từng bước. Đặc tả tổng: ../thiet-ke-tong-the-v2.md. Khung W0 là minh họa, không nối dữ liệu thật.

## Vấn đề đánh dấu

### P01 · Tên phiên nhiễm đường dẫn trong log — LỖI ĐÃ XÁC NHẬN

**Bằng chứng/phạm vi:** Đã đối chiếu ca kiểm tra máy: daemonlog72701→72820 [projects]→[zkey], cùng ID; CLI sau đó [scripts]. sessions.rs2104–2159,2508–2529; báo cáo docs/ra-soat-ten-phien-goc-20261116.md.

**Đề xuất:** Giữ Phiên chung nếu chưa được giao dự án; tách scope, topic, identity và nguồn xác nhận. Không lấy path census làm danh tính.

**Bàn tiếp:** Cách giao dự án qua lời nói cần gợi ý/xác nhận rõ; sửa nhãn không tự đổi quyền/cwd/context. P01 đánh dấu seam suy diễn path; bước D6 đang vẽ là hướng chặn lỗi, không phải chọn dự án là sai.

**Tại:** D6/bước3

### P02 · Báo dừng có đường chờ vòng dò dài — CHƯA ĐO / CHƯA ĐỦ

**Bằng chứng/phạm vi:** Thăm dò cũ16/11: poll120s,278cycle/277khoảng; đây là nhịp vòng, chưa đo latency đầu-cuối. Số đo tại .tmp/do-tre-thong-bao-20261116.md; chưa khẳng định binary hiện tại đã sửa.

**Đề xuất:** Sự kiện lifecycle đánh thức nhánh trạng thái nhẹ; polling chỉ đối soát. Ghi mốc nguồn→ingest→fanout→UI/ACK Telegram.

**Bàn tiếp:** Chốt mục tiêu độ trễ theo capability sau đo; mất nguồn giữ unknown, không suy idle hoặc chết.

**Tại:** D5/bước2, D1/bước8, U5/bước5

### P03 · Ô chat phải là TOÀN BỘ lịch sử phiên — preview 40 dòng/6000 ký tự không đủ — CHƯA ĐO / CHƯA ĐỦ

**Bằng chứng/phạm vi:** Yêu cầu chủ máy 19/11: «Ô chat phải là toàn bộ lịch sử phiên». Mã hiện tại cắt: opencode.rs274–276,305–329 — 40 hàng/6000 ký tự; request-trace.md24–41 mới thăm dò một lượt lịch sử OpenCode, chưa chứng minh live/đủ 3 CLI. Preview kiểu này chỉ đủ cho DANH SÁCH phiên, không đủ cho ô chat.

**Đề xuất:** Ô chat tải lịch sử từ NGUỒN (transcript/journal) theo cursor nguồn + revision bản ghi mutable, tới lượt ĐẦU của phiên; ảo hoá danh sách để cuộn mượt; tìm kiếm trên TOÀN BỘ đã nạp; nguồn chưa đọc hết thì ghi rõ đang nạp tới đâu/thiếu đoạn nào (gap), KHÔNG cắt im lặng ở N dòng hay X ký tự.

**Bàn tiếp:** Nghiệm thu phải đếm được: phiên có ≥200 lượt ⇒ thấy từ lượt 1, cuộn tới đỉnh, tìm được câu ở lượt cũ, số bản ghi khớp nguồn; không còn ngưỡng 40/6000 trong đường hiển thị ô chat. Giữ raw bao lâu và dung lượng vẫn là P16/P19.

**Tại:** D1/bước1, C5/bước3, D3/bước4

### P04 · Ô chat làm việc + footer phiên; mật độ còn bàn — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** Phản hồi16/11 đã chỉ rõ: A phải thấy từng phiên chạy gì và chat riêng; focus đổi chi tiết cột phải; B chỉ cần footer full ngang phía dưới. Hướng bố cục này không còn là câu hỏi chưa chọn.

**Đề xuất:** A giữ hội thoại/input/trạng thái/việc đang làm ở mọi ô; focus chỉ đổi chi tiết. B ở dưới toàn bộ menu+A+chi tiết, có icon chạy/việc đang làm/tin mới, cập nhật không phụ thuộc focus.

**Bàn tiếp:** Còn bàn mật độ, số ô và co giãn theo màn; cùng phiên ở nhiều bàn chỉ đếm một lần mỗi scope. Bản mẫu tương tác không phải sản phẩm đã kết nối CLI.

**Tại:** U5/bước3, U6/bước2

### P05 · Nhiều cửa cùng điều khiển nhầm hoặc đúp — RỦI RO CẦN KIỂM

**Bằng chứng/phạm vi:** pipeline.rs913–925: thiếu target thì dùng focus:session chung; screenshot explicit cũng có đường đổi focus16120–16143. Chưa tái hiện race desktop V2, không bê con trỏ này sang nhiều view.

**Đề xuất:** Lệnh mang target+generation+command ID; một dispatcher/phiên, control lease cho thao tác terminal thô; nhận diện lại trước side effect.

**Bàn tiếp:** Cho gửi thêm khi đang chạy chỉ nếu adapter đã đo; lease hết hạn không replay lệnh mơ hồ. Cùng chữ khác command ID vẫn là hai ý định.

**Tại:** U6/bước5, D2/bước6, U7/bước7

### P06 · Không cửa sổ terminal riêng ≠ không CLI — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** Luật hiện hành CLAUDE.md24–44 đòi đường chủ máy tiếp quản; thiết kế cũ §4.3 còn mở terminal thật. Chưa nghiệm thu runtime không cửa sổ thay thế trên3OS.

**Đề xuất:** UI là nơi làm việc chính. Giữ CLI runtime/PTY khi cần, terminal nhúng và nút mở terminal gốc; chỉ bật chế độ mới khi capability đủ.

**Bàn tiếp:** Đề xuất chỉ mở terminal gốc khi cần, chưa duyệt bỏ yêu cầu cửa sổ hiện hành hoặc tự chuyển phiên đang chạy.

**Tại:** U7/bước3, U1/bước7, U4/bước14

### P07 · Cần journal gốc hay cả payload model? — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** thiet-ke-desktop.md54–80; MCP không tự quan sát mọi request. Chưa có phép đo payload đầy đủ từngCLI/provider/version.

**Đề xuất:** Nguồn J: journal; nguồn P: model body chỉ khi hook hỗ trợ. Raw có provenance/completeness; phần ẩn ghi không cung cấp.

**Bàn tiếp:** Chốt mức bắt buộc; không MITM/đổi thuê bao sang API tính phí chỉ để có P.

**Tại:** C4/bước4, C5/bước5, X0/bước6

### P08 · Child/job có thể còn chạy khi cha idle — CHƯA ĐO / CHƯA ĐỦ

**Bằng chứng/phạm vi:** Thăm dò lịch sử08:53 ghi memory/active-context.md:5lượt subagent+8job mẫu; tool completed là ACK khởi chạy, jobexit1 vẫn có thể statecompleted. Chưa đo live/recovery/3CLI.

**Đề xuất:** Graph cha-con/job theo nativeID; launch ACK, runtime và kết quả là ba trục riêng. Sau ngắt/đóng vẫn nêu job còn sống.

**Bàn tiếp:** Khi cha kết thúc có dừng job không phải policy được công bố; child không làm tăng số phiên người dùng.

**Tại:** C5/bước7, D5/bước5, U2/bước15

### P09 · Giao lệnh/permission/attachment cần parity từng adapter (sau khi bỏ Codex) — CHƯA ĐO / CHƯA ĐỦ

**Bằng chứng/phạm vi:** pipeline.rs8214–8227: thiếu nhật ký thì Codex bị từ chối khai đã gửi, non-Codex trả DaGui theo MÀN — nay Codex bỏ (P21) nên chỉ còn MỘT đường DaGui-theo-màn, chưa đo chắc. Asking/Question sessions.rs4244–4288 chưa có revision/toolID; multi-question/multi-select đã có, không phủ nhận tính năng cũ.

**Đề xuất:** Đã nhận ở huba khác CLI-accepted; revision cho câu hỏi; attachment có manifest và receipt theo nguồn. Không hỗ trợ thì nêu rõ và có takeover.

**Bàn tiếp:** Không bật nút giả; không nhận spawn0/composer mất chữ là chắc đã giao. Không nâng quyền để đạt parity.

**Tại:** D2/bước7, U2/bước10, U7/bước5

### P10 · Usage từng request chưa là báo cáo toàn kỳ — CHƯA ĐO / CHƯA ĐỦ

**Bằng chứng/phạm vi:** Thiết kế cũ §9 và request-trace.md: metric ngữ cảnh/quota không phải tổng token; prototype dùng fixture, chưa collector toàn kỳ.

**Đề xuất:** Canonical usage theo semantics/cache/counter generation; child rollup tránh cộng đôi; chart/CSV cùng filter và coverage.

**Bàn tiếp:** Không có nguồn thì chưa đo; không thêm billing/tracker hoặc model để ước lượng.

**Tại:** D4/bước3, C4/bước7

### P11 · Vỏ desktop, transport và quyền trên ba OS — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** thiet-ke-desktop.md7–52: Tauri/local IPC là đề xuất; kiểm HTML Chromium không chứng minh WebView2/WebKit/native ACL hay bộ cài.

**Đề xuất:** Frontend chung; core kiểm principal/scopes; host/CLI/channel adapter riêng. Chạy spike trên Windows/macOS/Linux trước chọn vỏ cuối.

**Bàn tiếp:** Không mở LAN mặc định; secrets ngoài DOM; lựa chọn framework/vỏ theo thử thật, chưa là quyết định đã triển khai.

**Tại:** U1/bước2, U3/bước8, U6/bước1

### P12 · Crash/reconnect/retention có thể làm mất hoặc lặp — RỦI RO CẦN KIỂM

**Bằng chứng/phạm vi:** Nguy cơ của thiết kế streaming/command mới, chưa khẳng định lỗi runtime cụ thể. Nguồn mutable/cursor/history thiếu đã có bằng chứng giới hạn.

**Đề xuất:** Raw+ingest cursor commit cùng transaction; projection và view cursors riêng; command uncertain hòa giải, không replay side effect.

**Bàn tiếp:** Chốt retention/export theo quyền và audit; gap phải thấy rõ, quyền bị thu hồi thì xử lý cache nhạy cảm của client theo policy.

**Tại:** U4/bước3, D1/bước4, U6/bước8

### P13 · Đã chốt: MỘT chủ máy, một người dùng — nhiều cửa xem, không nhiều người — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** Quang 19/11: «P13 huba chỉ cài trên máy cá nhân nên chỉ 1 người dùng, chưa có hướng nhiều người dùng». Mã hiện tại khớp giả định ấy: cổng duy nhất là telegram::update_sender theo chat id (CLAUDE.md:160–173), không có role/tenant; sessions.rs:5677 read_only_count là đếm PHIÊN. Chữ «nhiều người xem» trong §6 bản trước là VIẾT SAI.

**Đề xuất:** Giả định sản phẩm: huba cài trên máy cá nhân ⇒ một người dùng. Nhiều CỬA XEM của cùng một người (desktop + Telegram), không phải nhiều người. Ranh giới quyền = ranh giới sẵn có của máy: tài khoản OS chạy hubad + cổng chat_id; IPC desktop chỉ nhận tiến trình cùng UID (peer identity) + pairing một lần. Mọi route GHI chỉ từ hai cửa UI/Telegram.

**Bàn tiếp:** Không xây role/tenant/share. Nếu sau này có yêu cầu nhiều người dùng thật thì mở issue MỚI, không nới dần mô hình hiện tại. Điều còn phải làm chỉ là kiểm kỹ thuật: IPC từ chối tiến trình khác UID (đo được), không phải mô hình quyền.

**Tại:** U6/bước1, U1/bước10

### P14 · Ba tên gọi thiết kế dùng nhưng mã chưa có: host_id, generation, command ID — CHƯA ĐO / CHƯA ĐỦ

**Bằng chứng/phạm vi:** grep toàn rust/src (19/11): host_id 0 · generation 0 · command_id 0. Nghĩa từng cái: (1) host_id = phiên nằm ở MÁY nào; (2) generation = số đời của một binding (mở lại/đổi cửa sổ thì tăng) để lệnh gửi cho cửa sổ CŨ không rơi vào cửa sổ MỚI; (3) command ID = mã của một lệnh người dùng bấm, để retry không gõ hai lần. Hiện Telegram chống trùng bằng update_id + bảng tin_den — riêng kênh Telegram, không có cho UI.

**Đề xuất:** Chốt gọn: (a) BỎ host_id — P13 đã chốt một máy, thêm host chỉ làm rối; (b) THÊM generation: driver cấp số mỗi lần bind/rebind, lưu kèm binding, lệnh mang generation và bị từ chối nếu lệch; (c) THÊM command ID: UI sinh UUID mỗi lần bấm, lưu bảng command (id, session, actor, payload hash, trạng thái) — trùng ID cùng payload trả kết quả cũ, cùng ID khác payload bị từ chối.

**Bàn tiếp:** Đây là điều kiện để P05 (hai cửa cùng điều khiển) và P12 (crash/reconnect) có nghĩa — không có generation thì không phân biệt được cửa sổ cũ/mới; không có command ID thì mất ACK là gõ lại. Không hứa lease/fence trước khi hai thứ này có chỗ lưu thật.

**Tại:** D6/bước1, D2/bước2

### P15 · Không có NGƯỠNG nghiệm thu cho độ trễ, retention, gap — CHƯA ĐO / CHƯA ĐỦ

**Bằng chứng/phạm vi:** P02 đòi đo latency nhưng không nêu mục tiêu/trần; P03/P12 đòi retention/gap nhưng không nêu hạn mức; §7 freshness có ba mốc nhưng không nói bao lâu thì coi là cũ. Không có số ⇒ không viết được ca đỏ.

**Đề xuất:** Một bảng số: p95 nguồn→ingest→UI; retention raw (ngày · GB); mất quá N% thì CẤM gọi «liên tục»; queue tràn ⇒ gap/resnapshot; freshness hết hạn theo loại nguồn.

**Bàn tiếp:** Số đặt trong dữ liệu như §9, nhưng phải có mặc định để đo ngay từ lát cắt đầu.

**Tại:** D5/bước1, D1/bước3

### P16 · Lịch sử lưu là văn bản NGUYÊN VĂN của phiên — nhưng đã qua hook, và chưa có chính sách lưu/xoá — RỦI RO CẦN KIỂM

**Bằng chứng/phạm vi:** Chủ máy 19/11: «lịch sử là nguyên văn nhưng qua hook xử lý rồi». Đúng: cái huba nhận từ transcript/journal là văn bản THẬT của phiên (không phải bản tóm tắt), và nó đã đi qua chuỗi hook của máy (mo-dau-phien, nhac-chuyen-phien, cổng bảo mật). Vì vậy P16 KHÔNG phải «bytes gốc của model» và cũng không phải «dữ liệu chưa xử lý» — vấn đề còn lại là VÒNG ĐỜI của bản lưu: nội dung nhạy cảm có thể đã nằm trong văn bản phiên (tiền lệ một phiên đọc .env), mà §8 chỉ nói «theo quyền/policy».

**Đề xuất:** Ghi rõ 4 điều cho kho lịch sử: (a) quyền thư mục 0700 (hoặc mã hoá at-rest nếu có), tên tệp không lộ nội dung; (b) retention mặc định + xoá theo yêu cầu CÓ audit; (c) đánh dấu record khớp mẫu bí mật (token, khoá riêng) để cảnh báo khi xuất — không tự sửa nội dung; (d) ai/cửa nào được xuất raw (chỉ chủ máy, hai cửa UI/Telegram).

**Bàn tiếp:** Không mâu thuẫn luật 5 (không giấu chữ với chủ máy): vẫn hiện nguyên văn, chỉ định nghĩa lưu bao lâu, xoá thế nào, cảnh báo khi xuất. Không suy «đã qua hook» thành «đã sạch bí mật».

**Tại:** D3/bước1, C4/bước1

### P17 · Ưu tiên Ubuntu chạy ngon trước; Windows là mục tiêu sau, chưa hứa — RỦI RO CẦN KIỂM

**Bằng chứng/phạm vi:** Chủ máy 19/11: «P17 làm ưu tiên để chạy ngon trên ubuntu trước». Máy đang dùng là Ubuntu (huba cài ~/huba, systemd --user). keys_win.rs:1–12 tự khai «CHƯA CHẠY TRÊN WINDOWS THẬT»; macOS có keys.rs dùng osascript; chưa OS nào có bộ nghiệm thu đầy đủ cho app desktop.

**Đề xuất:** Ba mức, theo thứ tự: (1) UBUNTU là nền chính — ứng dụng + hubad + CLI + lease/terminal phải chạy và được nghiệm thu thật trên máy này; (2) macOS giữ ngang hiện trạng huba (Terminal qua osascript), không hứa app trước khi Ubuntu xong; (3) Windows để SAU, chỉ nhận khi có máy thật để đo — không tính cross-build là hỗ trợ.

**Bàn tiếp:** Bỏ câu «cùng core/contracts trên ba OS» như cam kết ngang hàng; thay bằng «một core, ba đích đến theo thứ tự ưu tiên». Mỗi OS một cột nghiệm thu riêng; không OS nào được khai hỗ trợ khi chưa chạy thật.

**Tại:** U1/bước12, D1/bước12

### P18 · Số hiển thị phải đọc từ catalog CLI, không bảng cứng (ca thật 18/11) — LỖI ĐÃ XÁC NHẬN

**Bằng chứng/phạm vi:** Ca thật: nhac-chuyen-phien.py khai cứng 200k cho opencode-go/deepseek-v4.1-flash trong khi catalog models-dev của chính kho là 1.000.000 ⇒ 206k token bị tính 103% ⇒ «99% ≥ 50%, chuyển phiên» khi ngữ cảnh thật 21%. Mẫu 11 phiên/3 ngày: 6 phiên acc8 thật16–22% báo80–112%; 5 phiên acc9 thật12–35% báo19–59%. Đã vá a73e394 (kho chung).

**Đề xuất:** Luật trong §9: mọi % ngữ cảnh/quota hiển thị PHẢI kèm nguồn (opencode: catalog trong opencode.db · claude: usage trong transcript · codex: model_context_window) + ca âm «cấy catalog sai ⇒ đỏ»; bảng cứng chỉ là dự phòng có hạn.

**Bàn tiếp:** Cùng lớp lỗi với P02 (nhịp vòng bị đọc thành latency): con số không có nguồn thì không được hiện như sự thật.

**Tại:** D4/bước1, D5/bước3

### P19 · Toàn bộ lịch sử trong ô chat: hiệu năng, bộ nhớ, gap và tìm kiếm — RỦI RO CẦN KIỂM

**Bằng chứng/phạm vi:** Yêu cầu 19/11 đổi ô chat từ preview sang toàn bộ lịch sử. Transcript Claude một phiên dài hàng chục MB; journal Codex/OpenCode là jsonl/DB tăng liên tục. Chưa đo: thời gian nạp đầu, RAM khi 4–8 ô cùng mở phiên dài, tốc độ tìm trên toàn bộ.

**Đề xuất:** Ảo hoá (chỉ render cửa sổ nhìn), nạp theo trang có cursor (ví dụ 500 bản ghi/lần) + prefetch khi cuộn, cache thưa có giới hạn theo ô; tìm kiếm chạy trên nguồn/DB (chỉ mục) chứ không quét DOM; ô không focus thì hạ mức cập nhật. Đo mẫu: 1 phiên ≥10 MB × 4 ô.

**Bàn tiếp:** Không hạ chuẩn bằng cách quay lại cắt N dòng — nếu quá nặng thì hiện «đang nạp» + cho nhảy tới đầu/cuối, vẫn phải tới được lượt 1. Ngưỡng cụ thể thuộc P15.

**Tại:** U5/bước1, D1/bước4

### P20 · Cột đánh dấu câu hỏi trong ô chat để focus nhanh — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** Yêu cầu chủ máy 19/11: «Có cột đánh dấu câu hỏi để focus nhanh». Nguồn câu hỏi đã có (pending_question đọc AskUserQuestion từ transcript; /pick thao tác trên bảng hỏi), nhưng CHƯA có chỗ hiển thị: ô chat hiện chỉ là dòng chữ, không mốc nào để nhảy tới câu hỏi cũ/mới. P09 đã ghi representation chưa giữ revision/tool-call ID. Quang 19/11: marker theo LƯỢT HỎI, nhiều câu cùng lượt thì GỘP.

**Đề xuất:** Cột mốc bên trái ô chat, **mỗi LƯỢT HỎI một marker** (Quang 19/11). Lượt hỏi có nhiều câu (AskUserQuestion nhiều questions / multi-select) ⇒ **GỘP vào một marker** của lượt ấy, không tách theo từng câu. Marker: đang chờ = nổi + đếm ở đầu ô; đã trả lời = mờ; bị từ chối = gạch. Bấm marker ⇒ cuộn tới lượt hỏi + mở khối trả lời của lượt; danh sách nhảy nhanh liệt kê theo lượt, lượt đang chờ đứng đầu. Nguồn = câu hỏi THẬT trong transcript + trạng thái chờ từ runtime.

**Bàn tiếp:** ĐÃ CHỐT (19/11): theo LƯỢT HỎI, nhiều câu cùng lượt thì gộp. Còn lại chỉ là chi tiết hiển thị: marker kèm số câu trong lượt (vd «2 câu») để không giấu; câu bị CLI tự trả lời (hết hạn) vẫn thuộc lượt ấy và hiện trạng thái hết hạn. Marker KHÔNG thay nút trả lời — trả lời vẫn qua route + gate.

**Tại:** U5/bước1, U2/bước3

### P21 · Bỏ HẾT mọi thứ liên quan Codex (mã · test · cấu hình · tài liệu · hook) — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** Quang 19/11 «bỏ hẳn codex đi». Đo mã: grep -ri codex rust/src = 469 dòng/14 tệp (accounts143 · sessions83 · account_login49 · keys46 · quota40 · pipeline25 · runtime21 · config20 · codex_sessions19 · accounts_page10 · opencode6 · main4 · lib2 · keys_tmux1), 79 symbol codex*/CODEX_*, 13 tệp test, acc10 có cli=codex trong huba.config.json.

**Đề xuất:** BỎ HẾT mọi thứ liên quan Codex (Quang 19/11 «cái gì liên quan bỏ hết»): (a) mã: driver open_codex_window, codex_wall, codex_sessions.rs, nhánh codex trong accounts/sessions/keys/quota/pipeline/runtime/config/account_login/accounts_page/main/lib; (b) test: xoá 8 tệp codex + ca codex trong 5 tệp dùng chung — cùng commit, không #[ignore] câm; (c) cấu hình: bỏ acc10 khỏi huba.config.json trong repo; máy đang có acc10 thì mã TỪ CHỐI với câu «CLI không còn hỗ trợ» + log codex_refused (không crash, không mở cửa sổ); (d) tài liệu/hook: bỏ mọi nhắc Codex trong CLAUDE.md/README/docs, bỏ hook codex_pretooluse khỏi đường chạy; (e) dữ liệu acc10 giữ read-only, không migrate.

**Bàn tiếp:** ĐÃ LÀM BƯỚC 1 (19/11, chưa cài): cửa mở phiên TỪ CHỐI acc codex — ensure_account_usable + nhánh codex trong pipeline /new không còn mở cửa sổ; câu từ chối «Codex đã bỏ khỏi huba (19/11)…»; log codex_refused. Test mới sessions::codex_bi_tu_choi_o_cua_mo_phien (đỏ nếu gỡ cửa chặn). lib 155/155 · clippy -D warnings 0. CHƯA LÀM bước 2–4: 469 dòng/14 tệp còn trong mã + 13 tệp test — gỡ trọn ở lượt riêng (đụng snapshot/pipeline/keys/quota/accounts_page nên phải xoá cùng lúc test).

**Tại:** U1/bước1, U3/bước1

### P22 · Cổng kênh ĐỘNG + UI mobile TỰ DỰNG (chưa bật cửa nào hôm nay) — CẦN BÀN / CHỐT

**Bằng chứng/phạm vi:** Quang 19/11: «tạm thời app để sau, giờ làm như thiết kế hiện tại trước đã, nhưng vẫn phải để cổng động để có thể gắn app mobile khác». Hiện trạng: telegram.rs là kênh DUY NHẤT được nối; lệnh đi qua commands::ROUTES (CLAUDE.md:184–186), chống trùng ở tin_den, xác thực ở update_sender theo chat id (CLAUDE.md:160–173). tfl5/mobile đã bị bỏ vì là trang xem: 0 lệnh trong 2 ngày trong khi 9.553 lượt đẩy snapshot + 7.030 poll (memory/ra-soat-2026-08-14.md).

**Đề xuất:** Dựng cổng Channel có hợp đồng: nhận (Order) · gửi (Outbound có buttons/media/reply_to) · ack (react/sửa/tin mới, mỗi kênh tự chọn) · khai năng lực (buttons/edit/react/media/streaming/max_chars/max_file/push). Adapter trả principal, LÕI quyết định (một chủ máy — P13). Lõi giữ dispatcher + commands::ROUTES + tin_den; adapter không có bảng lệnh riêng, không tự chạy lệnh. Thiếu năng lực thì NÓI («kênh này không hỗ trợ nút…»), không im. Push là năng lực của adapter (app phải tự lo hoặc khai push:false). Media theo P09.

**Bàn tiếp:** Nghiệm thu: adapter GIẢ trong tests/ nhận 1 câu → lõi chạy route → trả câu, chứng minh thêm kênh không sửa pipeline/commands/sessions; đối chứng ngược gỡ đăng ký ⇒ đỏ, adapter tự chạy lệnh riêng ⇒ đỏ. KHÔNG dựng app, không mở đường mạng, không bật cửa thứ ba đang sống; P17 vẫn Ubuntu trước. UI mobile là do CHÚNG TA tự dựng (Quang 19/11 «sẽ tự dựng ui mobile»), không dùng lại trang tfl5, không phụ thuộc UI bên thứ ba; cổng Channel phải UI-AGNOSTIC để vỏ đó cắm vào mà không sửa lõi. Chọn nền vỏ (PWA/Tauri mobile/native) vẫn để mở — thuộc P11/P22, chưa chốt.

**Tại:** U3/bước1, D2/bước1

## Thành phần tham gia một phiên — ai làm việc gì?

Đây là các vai trò logic; không đồng nghĩa mỗi vai trò phải là một tiến trình/dịch vụ. Nhiều vai trò nằm chung hubad hoặc CLI. Thành phần tùy chọn chỉ tham gia khi UC/capability cần; không bắt một tin chào đi qua mọi thành phần.

| Mã | Thành phần | Input → output / trách nhiệm | Tham gia khi nào |
| --- | --- | --- | --- |
| A01 | Chủ máy | Mục tiêu, câu hỏi, xác nhận, tệp → ý định thao tác | Mọi UC; không thay quyết định người dùng bằng model đoán |
| A02 | Khung chat Desktop | Draft + session đích → send/answer/action; event → timeline | Chat local; draft/UI state không tự vào model |
| A03 | Telegram client + adapter | Owner text/callback/file → route/command; event → tin/nút | Kênh tùy chọn; một collector Telegram; gate chat/user đúng loại update |
| A04 | Terminal nhúng / cửa sổ gốc | Control lease + binding → tiếp quản cùng CLI; capture/PTY → màn | Cứu hộ/tương tác chưa có API; không cần cửa sổ riêng thường xuyên là mục tiêu, chưa là capability đã đo |
| A05 | Vỏ Desktop / IPC client | UI command → local transport đã ghép; stream → UI | Đề xuất Tauri; không cho frontend chạy shell/đọc DB CLI tùy ý |
| A06 | Gate danh tính và scope | OS peer/pairing hoặc owner chat → quyền thao tác | Mọi cửa core; không tin actor/scope client tự khai |
| A07 | Router / application service | Phân loại quản lý hay chat → handler chung | help/list/config có thể dừng tại huba, không đi model |
| A08 | Sổ command + dispatcher | Intent/command ID → trạng thái và delivery attempt | Lệnh có side effect; idempotency và hòa giải sau crash |
| A09 | Config / account / project service | Revision + defaults + secret refs → cấu hình đã kiểm | Mở phiên, đổi cấu hình; không model inference |
| A10 | Session driver + binding | Command + target identity → thao tác CLI có kiểm nhận | Gửi, ngắt, chọn câu hỏi, đóng, resume nếu hỗ trợ |
| A11 | Host / OS adapter | Yêu cầu tiến trình/terminal → PID/TTY/pane/cửa sổ | Linux/macOS/Windows có backend riêng; không resize phiên sống |
| A12 | CLI runtime Claude / OpenCode / nguồn đã hỗ trợ | User messages + config → agent loop, journal, model calls | Chủ sở hữu phiên và semantics context; không phải huba tự dựng agent loop |
| A13 | Context builder của CLI | Luật + lịch sử chọn + input + tools + results → request | Mỗi request, kể cả request tiếp sau tool; prompt có thể bị runtime biến đổi |
| A14 | Provider adapter / SDK / auth transport | Messages/tools/settings + auth riêng → request; stream → event | Gọi đúng provider; auth header không phải prompt; adapter có thể đổi serialization |
| A15 | Model/provider | Request model → text/tool-call/usage/stop hoặc lỗi | Chỉ khi có inference; phần system/thinking không xuất thì không bịa |
| A16 | Tool coordinator + gate thực thi | Tool-call ID/args → kiểm quyền → tool result/error | Nếu model yêu cầu tool; tool-call mới là yêu cầu, chưa phải đã chạy |
| A17 | Tool native / shell / file / plugin | Args + quyền → nội dung/ref/side effect/error | Tool built-in không cần qua MCP; output có thể quay lại context |
| A18 | MCP client phía CLI | Tool-call/resource request → MCP protocol → result | Chỉ tool/resource dùng MCP; không phải mọi câu chat đều qua MCP |
| A19 | MCP server công cụ | RPC tool/resource + quyền → đọc/ghi hệ thống nguồn → result | Ví dụ project-agent; server còn phải gate quyền nghiệp vụ, không chỉ client |
| A20 | Nguồn dữ liệu/tool đích | File, code repo, DB, API, bộ nhớ → dữ liệu hoặc side effect | Theo tool; nguồn ngoài có thể có auth riêng, lỗi/timeout/rate-limit |
| A21 | Subagent runtime | Task được giao + context con chọn → kết quả con | Tùy CLI; phiên con và usage riêng; không mặc định share toàn context cha |
| A22 | Journal / DB / hook chính thức của CLI | Hoạt động runtime → bản ghi nguồn có schema/version | Nguồn quan sát; không mặc định là wire payload model |
| A23 | Plugin / collector của huba | Nguồn CLI + cursor → raw observation + gap/error | Không gọi model để trích xuất; capture payload chỉ ở hook được hỗ trợ |
| A24 | Raw store + cursor / provenance | Record nguồn → dữ liệu nguyên theo nguồn + hash/checkpoint | Đề xuất mới; quyền riêng/retention; không ghi ngược DB CLI để giả event |
| A25 | Normalizer / projection / event delivery | Raw → timeline/tool/status, event IDs/read cursors | Đề xuất service chung; UI/Telegram/MCP đọc cùng semantics |
| A26 | Usage normalizer + local report index | Raw usage → canonical request usage/coverage | Báo cáo token; không billing/tracker hay model phân tích mới |
| A27 | Output renderer / attachment viewer | Event/blocks/ref → chữ, code, tệp, trạng thái, nguồn | Không thực thi HTML/JS/lệnh từ model; preview cắt phải có nhãn |
| A28 | MCP facade của huba | Client đã cấp scope → đọc nguồn/thống kê hoặc command được phép | Cửa tùy chọn mới; khác MCP server công cụ A19. Không cần đi qua facade để vẽ UI |
| A29 | Secret store / env + audit | Secret name/ref → transport credential; action → audit ref | Cắt ngang các lớp; log key names, không chép credential vào prompt/diagnostics |
| A30 | Nguồn luật/memory/compaction của CLI | Tệp luật, memory được đọc, summary → phần context | Chỉ phần thực sự nạp; raw history/UI history không tự thành context |

## Dữ liệu nào qua chat, MCP, model, lưu và hiển thị?

Có điều kiện nghĩa là phải có thao tác/capability được nêu; không mặc định toàn bộ dữ liệu đi qua model. MCP công cụ = A18/A19; MCP huba = A28. Tài liệu này không coi MCP là đường dây của mọi giao tiếp.

| Dữ liệu | Đường đi | Qua MCP? | Vào model context? | Lưu / đầu ra |
| --- | --- | --- | --- | --- |
| Tin chat mới | Khung chat/Telegram → gate/core → driver → CLI → context builder → provider/model | Không bắt buộc | Có, sau CLI nhận và thực đưa vào request | Intent/ACK tại huba; input journal/raw theo nguồn; timeline |
| help / danh sách phiên / đổi filter | UI/Telegram → router/query huba → kết quả | Chỉ nếu client chọn gọi facade | Không cần | UI/ack; không thêm lượt model |
| Draft chưa gửi | Khung chat → bộ nhớ/local draft theo thiết kế | Không | Không | Draft theo phiên; không gọi đã gửi |
| Luật workspace/project | Nguồn tệp → CLI nạp/đọc → context builder | Không bắt buộc; có thể qua Read native | Có nếu nạp/đọc vào context | Refs/hash/manifest nếu quan sát được; không bịa auto-load |
| Schema tool/MCP catalog | Tool registry/MCP server → CLI expose tools → model | Discovery có thể qua MCP | Có thể chiếm token khi expose; lazy/eager tùy CLI | Catalog/capability theo phiên bản; không phải toàn DB |
| Model tool-call | Model → provider adapter → CLI tool coordinator | Chỉ nếu tool đó ánh xạ MCP | Là output request hiện tại; có thể thành lịch sử input sau | Tool-call ID, args, permission state; chưa coi đã thực thi |
| Native tool args/result | CLI → quyền → native tool → file/shell → CLI | Không | Result có thể vào request tiếp; không tự hết mọi byte | Raw tool result/ref + preview/truncation có nhãn |
| MCP tool args | CLI MCP client → server → handler → data source | Có, MCP công cụ | Args là output model hoặc request do client tạo | Tool trace theo quyền, không log credential transport |
| MCP tool result | Nguồn → MCP server → client → CLI → context builder | Có, MCP công cụ | Có nếu CLI chọn đưa vào request tiếp | Structured/text/content refs; giữ lỗi/isError khi nguồn có |
| Đọc MCP resource bởi app/client | Client → MCP facade huba → scope gate → raw/projection → client | Có, MCP huba | Không mặc định; chỉ khi client đó đưa kết quả vào model | Đọc theo cursor; cùng provenance, audit theo quyền |
| Raw output của tool quá dài | Tool → CLI representation → request tiếp | Tùy loại tool | Có thể chỉ phần trim/ref; raw không đồng nghĩa model đã đọc hết | Bản nguồn và derivative riêng; đánh dấu phần bị cắt |
| Tệp/ảnh người dùng gửi | UI/Telegram → attachment resolver → CLI → provider nếu hỗ trợ | Không bắt buộc | Có điều kiện: byte ảnh/text/ref theo khả năng CLI/provider | Manifest/path/hash theo nguồn; có path chưa phải có byte |
| Text assistant cuối | Model → CLI → journal/event → collector → projection → UI/kênh | Không cần; facade có thể cho đọc lại | Là output; có thể vào lịch sử input lượt sau | Content blocks, nguồn, finish/turn state; không chỉ một chuỗi text |
| Reasoning/thinking | Provider → CLI nếu được xuất → collector | Không tự có nhờ MCP | Có thể là runtime opaque/reasoning; không hứa được xem | Chỉ trường nguồn cung cấp; ẩn/không xuất phải ghi rõ |
| Usage token | CLI/provider event → collector → normalizer → report index | Facade có thể cho đọc theo quyền | Không cần gửi model để tính báo cáo | Input/output/cache/semantics/coverage; unknown không thành0 |
| Context-window metric | CLI khai → huba display | Không bắt buộc | Metadata không tự là prompt | Tách tổng usage; không dùng bảng đoán làm số thật |
| Journal CLI / model payload | Nguồn chính thức → collector → raw/projection | Facade đọc lại tùy quyền | Không tự gửi lại model; chúng mô tả input/output đã xảy ra | Hai source kinds khác nhau; payload chỉ nếu hook hỗ trợ |
| Màn hình terminal | Host/PTY capture → snapshot → UI/Telegram | Không bắt buộc | Không mặc định; gửi ảnh lại là UC khác | Nguồn màn hình, không gọi transcript/payload nguyên bản |
| Account/model/config update | Admin UI → core revision/audit → profile mới | Có thể nếu facade có tool quản trị được cấp | Không inference; setting ảnh hưởng request sau đúng phạm vi | Config giữ tên secret; khóa không xóa lịch sử |
| Credential/token provider hoặc MCP | Secret store/env → auth transport | Có thể dùng để auth MCP, không nằm tool args | KHÔNG chủ động đưa vào context | Không trong code/prompt/log/CSV; key names hoặc secret refs |
| Lỗi/gate từ chối/permission hỏi | Gate/CLI/tool → sự kiện → UI → chủ máy quyết | Tùy lỗi thuộc MCP hay native | CLI có thể đưa tool error vào request sau | Giữ loại lỗi và trạng thái, không báo thành công/rỗng |
| Brief/handover | Nguồn chọn → checkpoint/summary → chủ máy duyệt → phiên mới | Không bắt buộc | Có khi chủ máy chuyển brief; tóm tắt bằng model phát sinh usage | Predecessor + source refs; không gọi là chép nguyên context |

## X0 · Trọn một tương tác — khung chat, MCP, model và kết quả

Sơ đồ tổng nối đủ các nhóm thành phần của một lượt giả định có tool MCP. Đây là một kịch bản, không buộc mọi lượt phải gọi MCP. Sơ đồ rộng: kéo ngang; các sơ đồ U/D/C sau phóng riêng từng vòng.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy / chat
    participant P1 as huba core / driver
    participant P2 as CLI / context
    participant P3 as Provider / model
    participant P4 as Tool runner / MCP client
    participant P5 as MCP tool server
    participant P6 as File / DB / API
    participant P7 as Plugin / collector
    participant P8 as Raw / index / usage
    participant P9 as UI kết quả / kênh
    participant P10 as MCP facade huba
    Note over P0: Bước 1 / 01 · Khung chat gửi; quản lý thuần túy có thể dừng tại huba
    P0->>P1: Input + session ref + command ID
    Note over P1: Bước 2 / 02 · Không chuyển input sang model bằng một agent trung gian
    P1->>P1: Gate quyền, ghi intent, khóa đúng phiên
    Note over P1: Bước 3 / 03 · CLI-accepted khác huba-received
    P1->>P2: Giao chữ/tệp theo capability và kiểm nhận
    Note over P2: Bước 4 / 04 · Context builder của CLI lắp request
    P2->>P2: Luật + lịch sử chọn + schemas + input
    Note over P2: Bước 5 / 05 · Provider adapter serialize; phần ẩn chỉ provider biết
    P2->>P3: R1: request model /  auth ngoài prompt
    opt 06 · Đường quan sát song song; chưa chắc là wire payload
    P2-->>P7: Journal/event R1 nếu nguồn xuất
    end
    opt 07 · Model đề nghị gọi tool, chưa có kết quả dữ liệu
    P3->>P2: Output R1: tool-call ID + tên + args
    end
    opt 08 · Nếu tool là MCP; native tool đi thẳng tool native
    P2->>P4: Tool-call qua gate/permission
    end
    opt 09 · MCP client/server, không qua khung chat như tin người dùng
    P4->>P5: MCP request tool + arguments
    end
    opt 10 · MCP tool có thể đọc/ghi; side effect cần quyền đúng UC
    P5->>P6: Handler kiểm quyền rồi đọc nguồn
    end
    opt 11 · Nguồn trả kết quả, không trả system prompt mới
    P6->>P5: Dữ liệu / error / ref attachment
    end
    opt 12 · Server trả protocol result cho client
    P5->>P4: MCP result: content/structured/error
    end
    opt 13 · CLI nhận trước; giữ error/truncation metadata
    P4->>P2: Tool result + tool-call ID
    end
    opt 14 · Collector thu phần CLI thực sự xuất
    P2-->>P7: Nguồn tool-call/result/permission
    end
    opt 15 · Một lượt chat có thể có nhiều request model
    P2->>P2: Lắp context R2 có result được chọn
    end
    opt 16 · Không gửi mọi DB/raw/history của huba tự động
    P2->>P3: R2: context cập nhật + tool result
    end
    Note over P3: Bước 17 / 17 · Reasoning/tệp chỉ khi nguồn cung cấp
    P3->>P2: Final text/blocks + usage/stop nếu có
    Note over P2: Bước 18 / 18 · Thu journal hoặc hook chính thức theo schema
    P2->>P7: Assistant event + turn state + usage
    Note over P7: Bước 19 / 19 · Source kind / IDs / completeness; không cộng stream-final đôi
    P7->>P8: Lưu raw+cursor /  projection /  canonical usage
    Note over P8: Bước 20 / 20 · UI/kênh theo dõi nhận cùng semantics
    P8->>P9: Timeline, tool trace, state, usage, refs
    Note over P9: Bước 21 / 21 · Đọc không gọi model và không tự gửi vào context
    P9->>P0: Xem phản hồi, tool, tệp, lỗi, nguồn
    opt 22 · Nhánh riêng: chỉ khi dùng MCP facade
    P0-->>P10: Client có quyền đọc resource huba
    end
    opt 23 · Facade là cửa truy cập kho huba, không phải tool server bắt buộc
    P10-->>P8: Gate scopes /  query raw/projection/usage
    end
    opt 24 · Không cần model cho thao tác đọc
    P8-->>P10: Resource/page theo cursor + provenance
    end
    opt 25 · MCP không tự nạp mọi dữ liệu vào prompt
    P10-->>P0: Trả client /  chỉ vào model nếu client chọn
    end
```

**Input:** Chủ máy gửi một yêu cầu cần đọc dữ liệu; phiên/profile/model đã được kiểm. Không dùng dữ liệu thật trong ví dụ thiết kế.

**Output:** Kết quả gồm nội dung assistant, khối code/tệp nếu nguồn có, tool trace, trạng thái giao/lượt, usage/context metrics và provenance/coverage — không hứa phần provider không xuất.

**Ràng buộc:** Model không trực tiếp truy cập DB hoặc thực thi MCP. CLI điều phối tool; MCP server thực thi handler theo quyền; dữ liệu tool quay về CLI trước khi có thể trở thành input request kế. Collector là đường quan sát song song, không phải model thứ hai.

## U1 · Mở app → chọn dự án → mở phiên thật

Đường vào chính, có cả máy đã cài và lần đầu. Có thể mở phiên chưa nhập việc; huba không tạo phiên headless thay thế một cách âm thầm.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy
    participant P1 as Desktop UI
    participant P2 as hubad / core
    participant P3 as Host / runtime
    participant P4 as CLI
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Bắt đầu UC-01
    P0->>P1: Mở ứng dụng
    Note over P1: Bước 2 / Trước mọi thao tác
    P1->>P2: Ghép client, kiểm quyền và version
    opt Account/model có thể theo mặc định đã kiểm
    P0->>P1: Chọn Phiên chung hoặc dự án đã giao
    end
    Note over P1: Bước 5 / UC-04: chủ máy bấm Mở
    P1->>P2: Yêu cầu Mở phiên + request ID
    Note over P2: Bước 6 / Thiếu danh tính dự án giữ Phiên chung
    P2->>P2: Kiểm assignment, luật và capability
    Note over P2: Bước 7 / Chưa đủ mode mới thì giữ terminal gốc
    P2->>P3: Mở runtime theo mode đã chọn
    Note over P3: Bước 8 / Có đường tiếp quản; không tự headless hóa
    P3->>P4: Khởi động CLI có hàng rào
    Note over P4: Bước 9 / Xác nhận danh tính, không chỉ exit0
    P4->>P2: Nhận diện phiên, PID / binding
    Note over P2: Bước 10 / Chỉ công bố mức đã chứng minh
    P2->>P1: Chờ câu đầu / đang chạy / sẵn sàng
    Note over P0: Bước 12 / Desktop focus riêng Telegram focus
    P0->>P1: Chọn phiên để vào Chat
    else Nhánh lỗi tại bước 3 / Nếu core mất / version không khớp / không quyền: dừng
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P2: Bước 3 / Nếu core mất / version không khớp / không quyền: dừng
    P2->>P1: Báo lỗi /  không cho chạy lệnh
    end
    else Nhánh lỗi tại bước 11 / Nếu đã mở cửa sổ nhưng chưa nhận diện CLI
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P2: Bước 11 / Nếu đã mở cửa sổ nhưng chưa nhận diện CLI
    P2->>P1: Giữ trạng thái chờ /  mở chẩn đoán
    end
    end
```

**Input:** OS user, phiên bản app/core, workspace/project ref, account ref hoặc auto, model/effort, việc tùy chọn.

**Output:** SessionKey + runtime binding có bằng chứng và đường tiếp quản; UI hiển thị sẵn sàng hoặc trạng thái chưa xác định/lỗi đúng bước.

**Ràng buộc:** Danh sách không đọc được khác danh sách0 phiên. Pane mở được khác CLI sẵn sàng. Chưa có câu đầu thì ghi chờ câu đầu, không bịa phản hồi.

## U2 · Chat → hỏi đáp → ngắt lượt / đóng phiên

Một lượt người dùng, từ draft tới phản hồi. Các thao tác hỏi, ngắt, đóng là nhánh độc lập có kiểm trạng thái, không phải tự động chạy sau mỗi tin.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy
    participant P1 as UI / Telegram
    participant P2 as hubad / core
    participant P3 as CLI / nguồn sự kiện
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Draft thuộc đúng phiên
    P0->>P1: Chọn phiên /  soạn tin
    Note over P1: Bước 2 / Chủ máy bấm Gửi
    P1->>P2: Gửi command ID + nội dung
    Note over P2: Bước 3 / Chưa tác động CLI trước khi ghi nhận
    P2->>P2: Ghi ý định bền vững + kiểm quyền
    Note over P2: Bước 4 / Đường driver có kiểm nhận — D2
    P2->>P3: Giao lệnh tới đúng phiên
    opt Nếu có bằng chứng khớp command
    P3->>P2: Nguồn ghi nhận input, tiến trình
    end
    Note over P2: Bước 7 / ACK giao khác kết thúc lượt
    P2->>P1: CLI đã nhận /  vẽ hoạt động
    opt Nếu CLI yêu cầu lựa chọn
    P3->>P2: Câu hỏi + revision theo nguồn
    end
    opt Kênh theo dõi nhận cùng revision
    P2->>P1: Hiện bảng hỏi đúng phiên
    end
    opt Nếu bảng còn đúng revision
    P0->>P2: Trả đủ đáp án qua UI
    end
    opt Nếu adapter hỗ trợ và đúng phiên
    P2->>P3: Gửi lựa chọn /  đọc lại kết quả
    end
    Note over P3: Bước 12 / Output theo D1, vòng model theo C2
    P3->>P2: Phản hồi / lỗi + nguồn
    Note over P2: Bước 13 / Renderer không thực thi nội dung model
    P2->>P1: Timeline và trạng thái lượt
    opt Chỉ khi chủ máy bấm Ngắt và có capability
    P0->>P2: Ngắt lượt qua UI
    end
    opt Giữ phiên; báo riêng tác vụ nền còn lại
    P2->>P3: Ngắt /  xác minh lượt đã dừng
    end
    opt Chỉ khi chủ máy chọn Đóng phiên
    P0->>P2: Đóng phiên qua UI + xác nhận
    end
    opt Không đóng nhầm terminal tái sử dụng
    P2->>P3: Kiểm identity, đóng, đọc lại
    end
    opt Không biến timeout thành đã đóng
    P2->>P1: Đã đóng hoặc chưa đo được
    end
    else Nhánh lỗi tại bước 5 / Nếu thiếu bằng chứng nhận: không tự gửi lại
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P2: Bước 5 / Nếu thiếu bằng chứng nhận: không tự gửi lại
    P2->>P1: Còn trong ô / chưa xác định
    end
    end
```

**Input:** Đúng session ID, draft, attachment refs nếu hỗ trợ; question revision khi trả lời; xác nhận cho thao tác phá dở.

**Output:** Trạng thái giao lệnh tách khỏi trạng thái lượt model; phản hồi và hoạt động có nguồn; giữ draft khi chưa giao.

**Ràng buộc:** Không gọi đã gửi chỉ vì gõ chữ hoặc Enter trả0. Không retry mù. Đóng phiên không đồng nghĩa ngắt lượt; tool nền có thể còn sống phải được báo.

## U3 · Dự án · account/model · cấu hình · Telegram

Bốn nhánh thiết lập do chủ máy chủ động chọn. Cùng quyền quản trị và revision/audit; không mặc định nối Telegram để được chat local.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy / UI
    participant P1 as hubad / config
    participant P2 as Host / secret store
    participant P3 as CLI login
    participant P4 as Telegram
    opt Nhánh UC-02: thiết lập dự án
    P0->>P1: Chọn folder + mặc định dự án
    end
    opt Nếu thiếu quyền: báo lỗi; không tự nới
    P1->>P2: Kiểm folder, quyền, nguồn luật
    end
    opt Nhánh UC-03: quản lý account
    P0->>P1: Thêm account, model, CLI
    end
    opt Chủ máy bấm Đăng nhập
    P1->>P3: Mở login gắn account + attempt ID
    end
    opt Không trộn lần login hoặc cửa sổ
    P3->>P0: URL / mã / xác thực cần người dùng
    end
    opt Hết hạn / hủy / lỗi phải tách sẵn sàng
    P3->>P1: Kết quả xác thực theo nguồn
    end
    opt Nhánh UC-15: quyền quản trị đã kiểm
    P0->>P1: Lưu config + expected revision
    end
    opt Không đặt secret trong audit/prompt
    P1->>P2: Lưu secret riêng /  config giữ tên
    end
    opt Xung đột → so diff, không ghi đè
    P1->>P0: Revision + audit + phạm vi áp dụng
    end
    opt Nhánh UC-11: tùy chọn
    P0->>P1: Bật wizard Telegram
    end
    opt Dùng cùng collector, không poll đôi
    P1->>P4: Kiểm bot /  xác minh owner chat
    end
    opt Cần thao tác chủ máy hai chiều
    P0->>P4: Nhắn bot /  bấm kiểm gửi/nhận
    end
    opt Sai chat/token/mạng là lỗi riêng
    P4->>P1: Ghi kết quả và scope theo dõi
    end
    opt Không nhận getMe là nghiệm thu hai chiều
    P1->>P0: Kết nối hoặc lỗi đúng bước
    end
```

**Input:** Folder, loại CLI, model/account config, credential qua luồng riêng, bot/owner-chat config, expected revision.

**Output:** Cấu hình có version/audit; account xác thực hoặc lỗi cụ thể; Telegram kiểm hai chiều; biết thay đổi áp dụng lúc nào.

**Ràng buộc:** Secret chỉ ở kho secret/env; UI không đọc trả token đã lưu. Khóa account không xóa lịch sử. Không đổi model phiên sống nếu chưa có khả năng đã đo.

## U4 · Mất kết nối · resume · fork · bàn giao

Phục hồi việc đang làm mà không gửi trùng, không làm chủ máy hiểu nhầm context được chuyển nguyên vẹn giữa các CLI/account.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy / UI
    participant P1 as hubad / core
    participant P2 as CLI / host
    participant P3 as Kho cục bộ
    alt Đường chính khi các điều kiện hợp lệ
    opt Khi kết nối phục hồi
    P0->>P1: Ghép lại + cursor + command IDs
    end
    opt Đối chiếu trạng thái trước gửi lại
    P1->>P3: Đọc lệnh treo và mốc nguồn
    end
    opt Không gõ lại chỉ vì mất ACK
    P1->>P2: Thăm dò danh tính và bằng chứng
    end
    opt Thiếu bằng chứng giữ chưa xác định
    P1->>P0: Đã nhận / còn chờ / chưa xác định
    end
    opt Nhánh Resume: chủ máy chọn
    P0->>P1: Tiếp tục phiên cùng CLI/profile
    end
    opt Khôi phục theo CLI, không từ UI buffer
    P1->>P2: Resume chính thức nếu hỗ trợ
    end
    opt Nhánh Fork: nếu capability đã đo
    P0-->>P1: Hỏi nhánh chỉ đọc
    end
    opt Không chèn câu hỏi vào phiên gốc
    P1-->>P2: Fork ID mới, quyền chỉ đọc
    end
    opt Nhánh Handover: chủ máy chọn
    P0->>P1: Bàn giao: xem đích và checkpoint
    end
    opt Nếu cần model tóm tắt phải báo có gọi model
    P1->>P3: Lưu brief + refs + phần chưa biết
    end
    opt Có quan hệ predecessor, không hứa chép toàn context
    P1->>P2: Mở đích với brief được duyệt
    end
    opt Thiếu capability → báo chưa hỗ trợ
    P1->>P0: Kiểm đích rồi mới đề nghị đóng gốc
    end
    opt Nhánh Đóng app; không phải Đóng phiên
    P0->>P0: Đóng cửa sổ app, để dịch vụ chạy
    end
    else Nhánh lỗi tại bước 1 / Nếu mất kết nối
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P0: Bước 1 / Nếu mất kết nối
    P0->>P0: Giữ draft, đánh dấu dữ liệu cũ
    end
    end
```

**Input:** Session ref, command treo, cursor đọc, draft, capability và checkpoint/brief nếu bàn giao.

**Output:** Tiếp tục đọc hoặc làm việc với đúng danh tính; nhánh fork/checkpoint có provenance; trạng thái mơ hồ không bị che.

**Ràng buộc:** Không replay tác dụng phụ đã có thể xảy ra. Mở app lại không tự mở CLI mới. Brief là bản tóm tắt dẫn nguồn, không phải toàn bộ context gốc.

## U5 · A nhiều ô chat — focus cột phải — B footer đầy ngang

A là các khung chat có hội thoại, input, trạng thái và việc đang làm luôn nhìn thấy. Focus ô nào thì cột phải hiện chi tiết ô đó. B là footer dưới cả menu+A+chi tiết, không còn bảng lớn trong vùng làm việc; vẫn nhận tin và hoạt động khi không focus.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy
    participant P1 as Bàn / ViewID
    participant P2 as Session registry
    participant P3 as Subscriptions
    participant P4 as CLI sources
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Không mặc định có quản lý máy xa
    P0->>P1: Mở bàn, chọn scope máy/account
    Note over P1: Bước 2 / Bàn lưu layout, không tự tạo phiên
    P1->>P2: Xin snapshot + revision + freshness
    Note over P2: Bước 3 / Focus chỉ đổi chi tiết cột phải
    P2->>P1: Ô chat A + footer B + tổng theo khóa
    Note over P0: Bước 4 / Chỉ thay layout, không gọi Mở CLI
    P0->>P1: Mở panel của một phiên trong B
    Note over P1: Bước 5 / Không focus vẫn nhận cập nhật
    P1->>P3: Timeline A + activity/latest cho footer B
    Note over P3: Bước 6 / Nhiều panel không nhân bản collector
    P3->>P4: Thu nguồn một lần theo adapter
    Note over P3: Bước 7 / Không giành focus hoặc làm mất draft
    P3->>P1: Chat/activity A và icon/tin/việc tại B
    opt Trong view này; không ngắt hoặc đóng CLI
    P0->>P1: Đóng panel cuối, phiên trở về B
    end
    Note over P1: Bước 10 / Lọc/tìm có số đang hiện trên tổng
    P1->>P1: Đối soát A/B/tổng và phần chưa rõ
    else Nhánh lỗi tại bước 9 / Không loại hàng như thể phiên đã chết
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P2: Bước 9 / Không loại hàng như thể phiên đã chết
    P2->>P1: Nguồn mất: giữ ảnh cũ và nhãn unknown
    end
    end
```

**Input:** Scope host/account, view ID, layout đã lưu, session census có freshness/generation; layout không quyết định phiên sống hay chết.

**Output:** Mỗi ô A có chat/input/status/activity riêng; cột phải theo focus; footer B có icon chạy/chữ trạng thái/việc đang làm/tin gần nhất; root count và phần chưa xác minh, child/job riêng.

**Ràng buộc:** A=O∩Panels(view), B=O trừ A, với O là tập root mở đã xác minh cùng scope. Panel unverified/out-of-scope có nhãn riêng, ngoài A/B. Filter hiển thị không đổi tổng scope; đổi scope thì tính lại O. Đóng panel cuối mới trả về B; tổng nhiều view dùng hợp khóa, không cộng tổng. Có nguồn mù thì tổng toàn bộ chưa xác minh.

## U6 · Nhiều cửa sổ — subscription riêng, command đích rõ

Nhiều view có thể cùng xem một phiên hoặc khác phiên. Desktop/Telegram dùng chung service nhưng không giành lựa chọn, draft hoặc con trỏ đọc của nhau.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as View A / View B
    participant P1 as IPC / gate
    participant P2 as Query + fanout
    participant P3 as Command dispatcher
    participant P4 as CLI / driver
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Không tin user ID client tự khai
    P0->>P1: Ghép client + cấp scopes theo quyền
    Note over P0: Bước 2 / Mỗi cửa giữ layout/draft riêng
    P0->>P2: Subscribe ViewID + scope + cursor
    Note over P2: Bước 3 / Khoảng giữa snapshot/delta không mất event
    P2->>P0: Snapshot watermark rồi delta có revision
    Note over P0: Bước 4 / Chọn phiên tại cửa khác không đổi đích
    P0->>P1: Gửi command ID + SessionKey + generation
    Note over P1: Bước 5 / Retry cùng key trả trạng thái cũ
    P1->>P3: Gate quyền và nhận intent một lần
    Note over P3: Bước 6 / Đọc lại binding trước khi gửi
    P3->>P4: Serialize tác động tới đúng phiên
    Note over P4: Bước 7 / Kết quả fanout cho view có quyền
    P4->>P2: Nguồn ghi ACK/trạng thái/nội dung mới
    opt Không đóng CLI hay view khác
    P0->>P2: Unsubscribe hoặc đóng một cửa
    end
    else Nhánh lỗi tại bước 8 / Queue đầy không làm rơi event im lặng
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P2: Bước 8 / Queue đầy không làm rơi event im lặng
    P2->>P0: Gap hoặc cursor cũ: yêu cầu snapshot
    end
    else Nhánh lỗi tại bước 10 / Xử lý cache nhạy cảm theo policy
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P1: Bước 10 / Xử lý cache nhạy cảm theo policy
    P1->>P0: Scope bị thu hồi: ngừng stream/action
    end
    end
```

**Input:** Client identity/scopes, ViewID, session key, binding generation, subscription filter, snapshot/event cursor, command ID.

**Output:** Mỗi view nhận snapshot+delta đúng scope; lệnh serialize theo session và không bị chuyển đích khi cửa khác chọn phiên.

**Ràng buộc:** View focus không phải command target. Subscription chỉ đọc không giữ control lease. Core kiểm quyền mỗi lần, bounded queue báo gap/resync; không hứa exactly-once trên CLI không hỗ trợ.

## U7 · UI thay cửa sổ thường ngày — runtime và terminal tiếp quản

Giảm cửa sổ terminal riêng mà không biến phiên thành agent nền không thể tiếp quản. Chỉ đề xuất mode mới; chưa sửa quy tắc hoặc runtime đang dùng.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy / UI
    participant P1 as Capability / core
    participant P2 as Runtime / Host
    participant P3 as CLI / PTY
    participant P4 as Terminal viewer
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Tauri không tự cung cấp mọi driver
    P0->>P1: Xem capability theo OS/CLI/version
    Note over P1: Bước 2 / Chưa đo thì chưa hỗ trợ, không ẩn lỗi
    P1->>P0: Hiện mode hiện hành và giới hạn
    opt Cần duyệt trước thay luật cửa sổ hiện hành
    P0->>P1: Chọn mode UI-first nếu đủ điều kiện
    end
    opt Không gọi mode mới là bỏ CLI
    P1->>P2: Runtime giữ CLI và kênh điều khiển
    end
    Note over P0: Bước 5 / Chỉ bật thao tác có capability đã đo
    P0->>P1: Chat/hỏi/quyền/tệp bằng UI cấu trúc
    opt Đăng nhập/cứu hộ/tương tác chưa có UI
    P0->>P1: Yêu cầu tiếp quản cùng phiên
    end
    opt Chặn structured/raw input; kiểm composer
    P1->>P2: Barrier + drain/fence rồi cấp lease
    end
    opt Nếu không attach được: báo, không mở phiên mới
    P2->>P4: Attach PTY hoặc mở viewer gốc
    end
    opt Input ngoài huba ghi nguồn terminal nếu biết
    P4->>P3: Input terminal tới cùng CLI
    end
    opt Reconcile input/draft rồi mở gửi cấu trúc
    P0->>P1: Trả lease qua barrier, giữ CLI
    end
    else Nhánh lỗi tại bước 11 / Giữ đường terminal gốc, không hứa tự động hết
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P1: Bước 11 / Giữ đường terminal gốc, không hứa tự động hết
    P1->>P0: Thiếu driver / OS dialog cần tay người
    end
    end
```

**Input:** CLI/OS/version capability matrix, runtime mode người dùng đã duyệt, attach handle/generation, control lease và scope thao tác.

**Output:** Chat/trace/questions trong UI; khi cần thì cùng phiên mở terminal nhúng hoặc terminal gốc có bằng chứng, không sinh phiên thay thế.

**Ràng buộc:** Cửa sổ đồ họa, PTY/console và tiến trình CLI là ba lớp khác nhau. Không đủ attach/receive/permission/interrupt/close/reconnect thì không công bố terminal-free. Takeover phải qua barrier cho cả structured và raw input, fence writer bằng lease epoch; còn command uncertain thì chưa nhập terminal.

## D1 · Nguồn CLI → raw → projection → màn hình

Đường dữ liệu quan sát một chiều. Collector không phải tác nhân model và không ghi ngược vào kho CLI để giả sự kiện.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Nguồn CLI
    participant P1 as Collector / adapter
    participant P2 as Kho raw + cursor
    participant P3 as Projection / index
    participant P4 as UI / kênh đã chọn
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Journal/file hoặc DB snapshot được hỗ trợ
    P0->>P1: Record + vị trí nguồn + version
    Note over P1: Bước 2 / Nguồn lỗi → log và coverage gap
    P1->>P1: Kiểm generation, identity, schema
    Note over P1: Bước 3 / Commit nguyên tử với ingest checkpoint
    P1->>P2: Ghi bản gốc + hash + source ref
    Note over P2: Bước 4 / Chỉ sau commit mới nâng cursor nguồn
    P2->>P1: Mốc ingest đã bền vững
    Note over P1: Bước 5 / Worker phục hồi được từ raw chưa xử lý
    P1->>P3: Chuẩn hóa bằng raw ref
    Note over P3: Bước 7 / ID nguồn + thế hệ; không chỉ timestamp
    P3->>P3: Khử trùng và liên kết chắc chắn
    Note over P3: Bước 8 / Cập nhật màn, không gọi model
    P3->>P4: Event/timeline + read cursor
    opt Nếu reconnect; replay đọc không có side effect
    P4->>P3: Đọc tiếp bằng cursor
    end
    opt Nếu nguồn có liên hệ chứng minh được
    P0->>P1: Stream và final cùng request ref
    end
    opt Không tạo hai câu trả lời final trùng
    P3->>P4: Một message cập nhật, raw vẫn giữ
    end
    else Nhánh lỗi tại bước 6 / Nếu schema lạ/hỏng; không nuốt thành rỗng
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P1: Bước 6 / Nếu schema lạ/hỏng; không nuốt thành rỗng
    P1->>P3: Parse-error event + raw ref
    end
    else Nhánh lỗi tại bước 10 / Nếu cursor đã hết hạn hoặc nguồn thiếu
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P3: Bước 10 / Nếu cursor đã hết hạn hoặc nguồn thiếu
    P3->>P4: Gap / dữ liệu cũ / cần snapshot
    end
    end
```

**Input:** Sự kiện/journal/DB chính thức có schema/version; màn hình là nguồn tách riêng; payload chỉ khi điểm mở rộng hỗ trợ.

**Output:** Raw ref bất biến theo nguồn, cursor bền vững, event chuẩn hóa, timeline/hoạt động có completeness.

**Ràng buộc:** Lưu raw/parse error trước khi nâng cursor. Không có ID chắc chắn thì không ghép; không xóa record hỏng rồi nhận đã thu đủ. Truy cập nguồn phải theo quyền.

## D2 · Input người dùng → command → kiểm nhận

Tách chính xác huba nhận yêu cầu, CLI nhận chữ và model xử lý. Đây là đường ghi duy nhất cho thao tác từ các kênh được cấp quyền.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Desktop / Telegram
    participant P1 as Gate / core
    participant P2 as Sổ command
    participant P3 as Session driver
    participant P4 as CLI / journal
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Desktop qua IPC; Telegram qua owner-chat gate
    P0->>P1: Identity + input + command ID
    Note over P1: Bước 3 / Trước side effect; không dùng text hash duy nhất
    P1->>P2: Insert intent theo idempotency key
    opt Nếu retry cùng key, đúng principal/phạm vi
    P2->>P0: Trả trạng thái lệnh đã có
    end
    Note over P2: Bước 5 / Chỉ một writer/lease gửi trên mỗi phiên
    P2->>P3: Lệnh mới + target generation
    Note over P3: Bước 6 / Revalidate identity ngay trước side effect
    P3->>P4: Thực hiện một lần có kiểm nhận
    Note over P4: Bước 7 / Nguồn bằng chứng được adapter hỗ trợ
    P4->>P3: Input record / composer / lỗi
    opt Khi bằng chứng đủ
    P3->>P2: CLI-accepted / in-composer / failed
    end
    Note over P2: Bước 10 / Không lấy exit0 làm accepted
    P2->>P0: Trạng thái giao thật
    opt Sau restart: không tự replay hành động mơ hồ
    P1->>P4: Chỉ đọc để hòa giải lệnh treo
    end
    Note over P4: Bước 12 / CLI-accepted khác model-done
    P4->>P1: Turn events qua collector D1
    Note over P1: Bước 13 / Đúng kênh và đúng phiên được theo dõi
    P1->>P0: Cập nhật output + trạng thái
    else Nhánh lỗi tại bước 2 / Nếu sai quyền / sai phiên / revision cũ: dừng
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P1: Bước 2 / Nếu sai quyền / sai phiên / revision cũ: dừng
    P1->>P0: Từ chối + audit lý do
    end
    else Nhánh lỗi tại bước 9 / Nếu crash, mất ACK hoặc không đọc được nguồn
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P3: Bước 9 / Nếu crash, mất ACK hoặc không đọc được nguồn
    P3->>P2: Uncertain + attempt evidence
    end
    end
```

**Input:** Principal được transport xác minh, channel/source-message ID, session ref, client command ID, expected revision, nội dung.

**Output:** Sổ lệnh/attempt và kết quả giao có bằng chứng. Request model chỉ xuất hiện ở C2 khi CLI thực sự gọi.

**Ràng buộc:** Terminal nhập trực tiếp không có command huba thì không bịa command ID. Một command có nhiều delivery attempt được ghi riêng, không có nghĩa được thực thi nhiều lần.

## D3 · Xem raw · MCP facade · xuất dữ liệu

Đọc cùng nguồn từ desktop hoặc MCP mà không ngầm gọi model. Nếu model tự gọi MCP tool thì response có thể đi vào context của đúng lượt đó — xem C2.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy / client
    participant P1 as UI IPC / MCP
    participant P2 as Gate / service
    participant P3 as Kho raw / refs
    participant P4 as Bản hiển thị / export
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / UC-12: chọn record cụ thể
    P0->>P1: Xem bản gốc / đọc resource
    Note over P1: Bước 2 / Không tin principal client tự khai
    P1->>P2: Client identity + scope + record ref
    Note over P2: Bước 4 / Theo quyền, không lấy preview thay raw
    P2->>P3: Đọc record và manifest
    Note over P3: Bước 5 / UI escape nội dung, không chạy HTML/JS
    P3->>P4: Raw + nguồn + hash + completeness
    opt UC-16: không tự gửi raw ra kênh khác
    P0->>P2: Yêu cầu xuất phạm vi đã chọn
    end
    opt Nếu nguồn CLI chứa bí mật do tool in ra
    P2->>P0: Cảnh báo nhạy cảm + xin xác nhận
    end
    opt Raw nguyên vẹn hoặc derivative có nhãn
    P2->>P4: Xuất manifest + dữ liệu theo quyền
    end
    Note over P2: Bước 10 / Không chép raw/secret vào log audit
    P2->>P2: Audit ai đọc/xuất, ref, thời điểm
    else Nhánh lỗi tại bước 3 / Nếu thiếu scope / sai owner: dừng
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P2: Bước 3 / Nếu thiếu scope / sai owner: dừng
    P2->>P1: Từ chối có lý do
    end
    else Nhánh lỗi tại bước 6 / Nếu chỉ có journal hoặc ảnh terminal
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P3: Bước 6 / Nếu chỉ có journal hoặc ảnh terminal
    P3->>P4: Không có nguồn / payload chưa cung cấp
    end
    end
```

**Input:** Client/principal, scopes, record refs hoặc cursor; yêu cầu export riêng và xác nhận khi nhạy cảm.

**Output:** Nội dung đúng loại nguồn, manifest/provenance/coverage; derivative đã lọc có nhãn khác raw.

**Ràng buộc:** MCP là cửa đọc/điều khiển có quyền, không phải bộ bắt toàn bộ I/O. Secret transport cấu hình không được đưa vào response. Chủ máy vẫn xem raw nhạy cảm theo quyền.

## D4 · Usage CLI/plugin → thống kê token

Dữ liệu token đi cùng raw/provenance, không qua một lời gọi model phân tích hoặc một dịch vụ billing mới.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as CLI / plugin nguồn
    participant P1 as Normalizer
    participant P2 as Raw + usage index
    participant P3 as Query / bộ lọc
    participant P4 as Báo cáo UI / CSV
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Đi qua ingest D1
    P0->>P2: Lưu raw usage + source ref
    Note over P2: Bước 2 / Không suy usage từ độ dài text
    P2->>P1: Đọc raw/version/counter semantics
    Note over P1: Bước 3 / Chưa biết semantics → unknown
    P1->>P1: Xác định scope và thành phần
    opt Nếu counter tích lũy; reset mở thế hệ mới
    P1->>P1: Delta theo cùng counter + baseline
    end
    Note over P1: Bước 5 / Chunk/final/refork đã đối chiếu khử trùng
    P1->>P2: Canonical request usage + coverage
    Note over P4: Bước 7 / UC-13: một filter chung
    P4->>P3: Ngày/múi giờ/dự án/model/account
    Note over P3: Bước 8 / Không gọi provider/model khi đổi filter
    P3->>P2: Đọc cùng tập canonical records
    Note over P2: Bước 9 / Ngày không có nguồn không thành0 thật
    P2->>P4: Tổng biết + coverage + danh sách thiếu
    opt Cùng scope/revision với chart
    P4->>P3: CSV hoặc drill-down tới raw
    end
    opt Mất nguồn → dữ liệu cũ, không báo mới nhất
    P3->>P4: Kết quả khớp bảng và chart
    end
    else Nhánh lỗi tại bước 6 / Nếu thiếu ID/account hoặc thành phần
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P1: Bước 6 / Nếu thiếu ID/account hoặc thành phần
    P1->>P2: Thiếu / chưa quy thuộc + lý do
    end
    end
```

**Input:** Raw usage có semantics, account/session/model/request refs khi nguồn cho phép; counter từng request hoặc tích lũy.

**Output:** Chỉ mục usage chuẩn, KPI/chart/bảng/CSV cùng filter; coverage/unknown/unattributed luôn thấy được.

**Ràng buộc:** Cache đã nằm trong input thì không cộng lần hai. Context hiện tại/quota% không phải token đã dùng. Không cộng cả parent rollup và usage child nếu đã bao gồm nhau.

## D5 · Lifecycle — nguồn thật → registry → panel/list/Telegram

Trạng thái phiên, lượt model, tool và job không được gộp thành một cờ busy. Sự kiện đánh thức nhánh nhẹ, polling đối soát phần nguồn không phát sự kiện.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as CLI / runtime
    participant P1 as Adapter observer
    participant P2 as Registry / graph
    participant P3 as Event delivery
    participant P4 as UI / Telegram
    alt Đường chính khi các điều kiện hợp lệ
    Note over P0: Bước 1 / Nguồn mutable cũng phải được cập nhật
    P0->>P1: Busy/idle/question/exit hoặc revision mới
    Note over P1: Bước 2 / Polling định kỳ chỉ để đối soát
    P1->>P2: Wake nhánh nhẹ + observation có nguồn
    Note over P2: Bước 3 / Mốc cũ không ghi đè trạng thái mới
    P2->>P2: Kiểm generation và thứ tự bằng chứng
    Note over P2: Bước 4 / Đo t_source/t_ingest/t_fanout riêng
    P2->>P3: Event chuẩn + source health + watermark
    opt ACK khởi chạy không khép runtime job
    P2->>P3: Child/job còn chạy hoặc kết quả lỗi
    end
    Note over P3: Bước 6 / Một lần theo transition, không báo mỗi poll
    P3->>P4: Panel live + summary list + notice theo dõi
    opt Khôi phục không phát lại notice hàng loạt
    P1->>P2: Resync từ snapshot có generation
    end
    Note over P4: Bước 10 / ACK Telegram không chứng minh đã thấy trên điện thoại
    P4->>P4: Render/ACK transport và đo latency
    else Nhánh lỗi tại bước 7 / Giữ ảnh cũ, đánh dấu chưa xác minh
    break Dừng thao tác hiện tại sau khi báo đủ lỗi
    Note over P1: Bước 7 / Giữ ảnh cũ, đánh dấu chưa xác minh
    P1->>P2: Probe lỗi / disconnect / schema chưa biết
    Note over P2: Bước 8 / Không chuyển unknown thành idle/closed
    P2->>P4: Cũ/unknown và mốc cuối đã xác minh
    end
    end
```

**Input:** Runtime/source events có timestamp/revision/generation và kết quả probe; scope nguồn và mức độ tin cậy; graph child/job.

**Output:** Session lifecycle + turn state + job state + source health, các mốc quan sát và notification chuyển trạng thái có dedup.

**Ràng buộc:** Idle khác closed; ended turn khác nghiệp vụ được nghiệm thu; lỗi probe khác danh sách rỗng. Mất kết nối không tự giết phiên hay khép mọi job.

## D6 · Danh tính — SessionKey khác topic, cwd và dự án được giao

Dẹp việc một phiên hỏi chung bị đổi tên sang ứng dụng trong log. Cho phép đặt tên dễ đọc nhưng không để nhãn làm thay quyền, scope hay đích lệnh.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy
    participant P1 as UI / lời giao việc
    participant P2 as Identity service
    participant P3 as Registry / audit
    participant P4 as CLI / nguồn
    Note over P4: Bước 1 / Account attribution và binding riêng
    P4->>P2: Native ID + namespace + host + CLI
    Note over P2: Bước 2 / Không ép root phải thuộc một ứng dụng
    P2->>P3: Chưa giao dự án: assignment null
    Note over P4: Bước 3 / Lưu provenance; không dùng làm assignment
    P4->>P2: Path trong log / prefix assistant / cwd
    Note over P2: Bước 4 / Topic tự sinh có nhãn, không đổi scope
    P2->>P1: Hiện Phiên chung · ID + topic riêng
    opt Ngôn ngữ tự nhiên mơ hồ thì chưa nhận giao
    P0->>P1: Chọn dự án/module hoặc xác nhận gợi ý
    end
    opt Không đổi tên từ thao tác xem log
    P1->>P2: Assignment request + expected revision
    end
    opt Xung đột revision phải đối chiếu lại
    P2->>P3: Kiểm folder/quyền, lưu actor và nguồn
    end
    Note over P3: Bước 8 / Đổi scope/runtime là UC riêng
    P3->>P1: Nhãn đã xác nhận, khóa phiên giữ nguyên
    opt Có audit; không phục hồi nhãn đoán cũ
    P0->>P2: Sửa tên hoặc bỏ assignment
    end
```

**Input:** SessionKey từ host/CLI/namespace native; account attribution riêng; assignment user chọn/xác nhận, topic/cwd riêng, expected revision và actor.

**Output:** Phiên chung + ID hoặc dự án/module đã giao; provenance cho assignment và audit. Đổi tên không thay khóa phiên.

**Ràng buộc:** Không có assignment không có nghĩa là không có phiên. Path census/prefix model chỉ có thể là gợi ý, không tự xác lập danh tính. Giao dự án không tự cấp quyền ghi hay đổi luật/context.

## C1 · Khởi tạo phiên: ai đưa gì vào context?

Phân định trách nhiệm: huba chọn profile/model và cấp nguồn luật/cổng; CLI lắp request thực tế. Project rules được đọc đúng phạm vi, không giả mọi tệp tự nạp.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy
    participant P1 as huba / launcher
    participant P2 as CLI
    participant P3 as Luật / tools / nguồn
    participant P4 as Model / provider
    Note over P0: Bước 1 / Cấu hình không phải câu hỏi cho model
    P0->>P1: Chọn dự án, account, model/effort
    Note over P1: Bước 2 / Không gộp secret vào system prompt
    P1->>P2: Profile + nguồn luật + cổng quyền
    Note over P2: Bước 3 / Phạm vi project/subfolder phải kiểm thực tế
    P2->>P3: Nạp luật được CLI hỗ trợ
    Note over P3: Bước 4 / Đọc trực tiếp khi vào phạm vi theo quy định
    P3->>P2: Nội dung luật + provenance
    Note over P2: Bước 5 / Catalog/schema có thể nạp eager hoặc lazy
    P2->>P3: Liệt kê tools/MCP được cấp
    Note over P3: Bước 6 / Lấy dữ liệu khi tool/resource được đọc
    P3->>P2: Schema tools, không phải mọi dữ liệu
    opt Nếu mở phiên không việc thì chờ, không bịa tin
    P0->>P2: Tin đầu / brief được duyệt
    end
    Note over P2: Bước 8 / System + luật + lịch sử chọn + input
    P2->>P2: Lắp messages theo CLI/provider
    opt Chỉ khi CLI thực sự khởi một lượt
    P2->>P4: Request model theo nguồn xác thực
    end
    opt Không quan sát được → ghi chưa biết
    P2-->>P1: Manifest / usage nếu nguồn hỗ trợ
    end
    Note over P1: Bước 11 / huba không khẳng định thấy prompt ẩn
    P1->>P0: Hiện nguồn nạp và độ đầy đủ
```

**Input:** Profile/account config, model/effort, nguồn luật workspace/project, agent mode, tool/MCP catalog được cấp, việc đầu tiên tùy chọn.

**Output:** CLI session và context envelope theo capability; nếu quan sát được thì có manifest nguồn/thời điểm, không bịa payload đầy đủ.

**Ràng buộc:** Secret xác thực provider là header/kho secret, không phải nội dung prompt. Mô tả tool được expose có thể chiếm token; đăng ký MCP không có nghĩa toàn DB MCP đã vào context.

## C2 · Một lượt model: input → tool → input tiếp → output

Một lượt chat có thể phát sinh nhiều request model. Collector chỉ quan sát; không đứng giữa để tóm tắt mọi message rồi gọi thêm model.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Người dùng
    participant P1 as CLI / context builder
    participant P2 as Model / provider
    participant P3 as Tool / MCP / host
    participant P4 as Collector → huba
    Note over P0: Bước 1 / Từ D2 hoặc gõ terminal trực tiếp
    P0->>P1: Tin mới + tệp/tham chiếu
    Note over P1: Bước 2 / Lịch sử được chọn + luật + schemas + input
    P1->>P1: Lắp context cho request R1
    Note over P1: Bước 3 / Provider adapter có thể biến đổi payload
    P1->>P2: R1: model input
    Note over P2: Bước 4 / Chỉ trường nguồn thực sự xuất
    P2->>P1: Chunk text / tool call / usage
    Note over P1: Bước 5 / Không mặc định journal = payload R1
    P1->>P4: Journal/event quan sát được
    opt Nếu output yêu cầu tool và gate cho phép
    P1->>P3: Kiểm quyền rồi chạy tool
    end
    opt Giữ tool-call ID; không giả thành assistant text
    P3->>P1: Tool result / lỗi / ref tệp
    end
    opt Có thể trim/truncate; raw nguồn giữ riêng
    P1->>P1: Chọn tool result đưa vào context
    end
    opt Lặp nếu cần; không chỉ gửi riêng kết quả tool
    P1->>P2: R2: context cập nhật + tool result
    end
    opt Kết thúc request chưa chắc kết thúc toàn lượt
    P2->>P1: Phản hồi cuối + usage R2
    end
    Note over P1: Bước 11 / D1 chuyển tới UI và kênh theo dõi
    P1->>P4: Mốc turn-complete / abort / error
    opt Nếu dùng subagent; context con không mặc định y hệt cha
    P3-->>P4: Child session + parent relation
    end
    opt Nếu chủ máy ngắt: ghi abort, báo tool nền còn lại
    P0->>P1: Yêu cầu ngắt qua core/driver
    end
```

**Input:** Tin mới đã được CLI nhận, context CLI chọn, model/provider, tools được cấp, attachment theo capability.

**Output:** Assistant text/tool calls/usage và mốc kết thúc; mỗi request/attempt giữ quan hệ, reasoning chỉ có khi nguồn xuất.

**Ràng buộc:** Tool result trở thành input cho request kế nếu CLI đưa vào; vì vậy lệnh chạy ít tiền chưa chắc context nhỏ. Tool permission enforced ngoài model. Không coi mọi output stream là câu cuối.

## C3 · Context dài: compaction · resume · fork · handover

Bốn cơ chế khác nhau, không dùng chung nhãn tiếp tục phiên. Chỉ CLI/provider biết chính xác context request nếu không có hook quan sát.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Chủ máy / UI
    participant P1 as huba / service
    participant P2 as CLI / model runtime
    participant P3 as Raw / lịch sử / brief
    opt Nhánh compaction: không dùng ngưỡng UI đoán
    P2->>P2: Phát hiện context vượt policy CLI
    end
    opt Chỉ thu nếu CLI xuất; thiếu thì ghi chưa biết
    P2-->>P3: Nguồn compact summary / refs
    end
    opt Không còn đồng nghĩa toàn lịch sử gốc
    P2->>P2: Lắp request từ summary + phần giữ
    end
    Note over P3: Bước 4 / Xem trên UI không bơm lại vào prompt
    P3->>P0: Xem lịch sử cũ theo quyền
    opt Nhánh resume: chủ máy chọn
    P0->>P1: Resume cùng phiên/profile
    end
    opt Đọc nguồn; có thể re-nạp theo semantics CLI
    P1->>P2: Khôi phục bằng cơ chế CLI
    end
    opt Nhánh fork: capability phải chứng minh
    P0-->>P1: Fork hỏi chỉ đọc
    end
    opt Phiên gốc không thêm lượt hỏi này
    P1-->>P2: Nhánh ID mới + quyền chỉ đọc
    end
    opt Không gộp lượt fork vào thread cha
    P2-->>P3: Nhật ký/usage riêng cho nhánh
    end
    opt Nhánh handover: xem trước phạm vi
    P0->>P1: Bàn giao sang đích đã chọn
    end
    opt Tóm tắt có nhãn, không thay raw
    P1->>P3: Brief + refs + việc dở + nền
    end
    opt Account/CLI khác: không hứa token/context chuyển nguyên
    P1->>P2: Phiên mới + brief + luật đích
    end
    opt Chỉ đóng gốc khi chủ máy/phạm vi cho phép
    P2->>P0: Đích nhận việc, còn thiếu nói rõ
    end
```

**Input:** Lịch sử gốc, context CLI hiện chọn, giới hạn CLI khai nếu có, checkpoint/brief, capability theo phiên bản.

**Output:** Context được compact/resume hoặc nhánh/phiên mới có quan hệ; UI chỉ rõ cái nào giữ nguyên, cái nào bị tóm tắt/chưa biết.

**Ràng buộc:** UI vẫn có thể đọc lịch sử cũ từ raw dù model đã compact. Giữ raw không làm model tự nhớ. Tóm tắt bằng model/fork có usage riêng, không được gọi miễn token.

## C4 · Ranh giới nhìn thấy: UI · journal · payload · token

Chốt nghĩa input/output gốc và tránh suy từ những gì UI thấy sang những gì model thực nhận. Thiết kế giữ hai đường journal và model payload riêng.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as Desktop / MCP
    participant P1 as huba / service
    participant P2 as Collector
    participant P3 as CLI
    participant P4 as Provider
    Note over P0: Bước 1 / Đọc dữ liệu không phát sinh suy luận model
    P0->>P1: Xem hội thoại / lọc báo cáo
    Note over P3: Bước 2 / Mức J: những gì CLI ghi
    P3->>P2: Journal: user/assistant/tools
    Note over P2: Bước 3 / Không gọi đây là model payload
    P2->>P1: Raw journal + completeness
    opt Mức P: chỉ khi có capability/version đã đo
    P3-->>P2: Request/response từ hook được hỗ trợ
    end
    opt Giữ nhãn model-body source, không gọi packet nguyên byte
    P2-->>P1: Payload đã bỏ header secret + provenance
    end
    Note over P4: Bước 6 / Phần provider không xuất không được suy diễn
    P4->>P3: Text / reasoning / usage được phép xuất
    Note over P3: Bước 7 / Không dùng cùng một nhãn tổng token
    P3->>P2: Usage từng request / context metric
    Note over P1: Bước 8 / Không có nguồn → chưa cung cấp
    P1->>P0: Journal hoặc payload với nhãn riêng
    opt Chỉ khi chủ máy chủ động yêu cầu
    P0->>P1: Chọn nội dung để gửi lại cho CLI
    end
    opt Lúc này nội dung mới có thể vào context
    P1->>P3: Tin mới / attachment theo quyền
    end
    Note over P1: Bước 11 / Không lấy hash khớp1 record chứng minh thu đủ
    P1->>P0: Coverage lịch sử và nguồn bị thiếu
```

**Input:** Câu hỏi của chủ máy về nguồn, record refs, usage hoặc context metrics do CLI xuất.

**Output:** Nhãn nguồn/coverage, payload nếu có khả năng, hoặc chưa cung cấp; token usage và context metrics tách rõ.

**Ràng buộc:** Không hứa đầy đủ request model từ transcript hoặc MCP. Không MITM/chuyển sang API trả phí để lấy payload. Provider có thể giữ prompt/thinking không xuất.

## C5 · Cây lượt xử lý — vòng assistant, model, tool, child và job

Cùng graph phục vụ xem trực tiếp và xem lại. Các ID nguồn có đến đâu hiện đến đó; không lấy số hàng assistant làm số HTTP request khi thiếu ánh xạ.

```mermaid
sequenceDiagram
    autonumber
    participant P0 as CLI / provider
    participant P1 as Collector
    participant P2 as Graph / raw
    participant P3 as UI trace
    participant P4 as Child / job
    Note over P0: Bước 1 / Turn ref chỉ khi có liên hệ chắc chắn
    P0->>P1: Lượt bắt đầu hoặc assistant event
    Note over P1: Bước 2 / Không dựng graph từ preview cắt ngắn
    P1->>P2: Lưu event gốc + source revision
    Note over P0: Bước 3 / Đọc revision, không chỉ seq tăng
    P0->>P1: Cập nhật hàng cũ hoặc tool state
    Note over P2: Bước 4 / Không bịa request HTTP từ số hàng
    P2->>P3: Vòng assistant quan sát được + refs
    opt Có capability mới hiện payload/source P
    P0-->>P1: Model request/response nếu hook xuất
    end
    opt Giữ toolID; gắn raw input/output
    P0->>P2: Tool-call → permission → result/error
    end
    opt ACK khởi chạy không phải job kết thúc
    P0->>P4: Launch child/job + nhận ACK ID
    end
    opt Theo IDcon/job, không ghép bằng giờ gần
    P4->>P1: Running/exit/result/error theo nguồn
    end
    Note over P2: Bước 9 / Usage parent/child không cộng trùng
    P2->>P3: Cập nhật graph + coverage + usage
    Note over P0: Bước 10 / Kết thúc lượt không phải nghiệmthu côngviệc
    P0->>P1: Idle/abort/error + job còn mở
    Note over P2: Bước 11 / Chưa có ID thì hiện chưa ghép, không giấu
    P2->>P3: Live hoặc replay cùng dữ liệu có nguồn
```

**Input:** Command/delivery refs nếu có; turn/assistant/request/tool/child/job IDs, mutable revision, trạng thái và raw refs theo capability.

**Output:** Timeline/cây cha-con có input/output/tool result/errors/usage/source links; live và lịch sử chung semantics, có nhãn thiếu correlation.

**Ràng buộc:** Tool returned/completed cho launch chỉ chứng minh lời gọi khởi chạy trả về. Child/job outcome và runtime terminal là bằng chứng riêng; exit1 không thành thành công vì metadata completed.

## Sổ màn hình V2 — đầy đủ thao tác và trạng thái phải vẽ

Đây là yêu cầu thiết kế, không phải checklist đã kiểm sản phẩm. Header toàn ngang; logo/tên trái, tài khoản chủ máy phải; nút thu menu ở đầu menu trái. Bảng có STT xuyên trang, nền xen kẽ, tìm toàn tập và phân trang.

| Màn / vùng | Nội dung và thao tác | Empty/error/unknown bắt buộc | Điểm vướng |
| --- | --- | --- | --- |
| Bàn làm việc | A: nhiều chat/input riêng, luôn thấy việc đang làm; focus đổi cột phải. B: footer full ngang icon/status/việc/tin mới; lưu layout | Không focus vẫn cập nhật; draft riêng; mất nguồn giữ ảnh cũ và nhãn unknown; tổng chưa xác minh | P02/P04/P05 |
| Panel phiên | Danh tính/topic/account/model; hội thoại; gửi/tệp/ngắt; bảng hỏi/quyền; job; raw | Draft riêng; ACK chưa rõ; câu hỏi cũ; tệp mất; chỉ có preview; chưa hỗ trợ thao tác | P01/P03/P08/P09 |
| Trace một lượt | Cây lượt → vòng assistant/request nếu biết → tool → child/job; input/result/raw; live/xem lại | Chưa ghép được; thiếu một phần; bị từ chối/lỗi; còn job sau idle; không có payload | P03/P07/P08 |
| Phiên và lịch sử | Root mở, chưa xác minh và đã đóng tách nhóm; tìm toàn tập; mở panel/resume/fork/bàn giao | Mất nguồn khác 0 phiên; resume chưa hỗ trợ; job mồ côi; lịch sử có khoảng thiếu | P03/P06/P12 |
| Mở phiên | Chung/dự án rõ; account/model/effort; mode/capability; xem luật/quyền; việc tùy chọn | Thiếu CLI/login; đang chờ binding; thiếu luật; mode chưa đủ; không tự mở pane thay thế | P01/P06/P11 |
| Tài khoản & model | Thêm/sửa/khóa; lần đăng nhập; catalog model; quota có thời điểm và nguồn | Challenge hết hạn; account khóa; model chưa xác minh; quota chưa đo; revision xung đột | P09/P11 |
| Kết nối | Desktop IPC, Telegram, MCP pairing/scopes; kiểm nhận/gửi hai chiều; thu hồi client | Sai owner/secret; poller trùng; mất mạng; nguồn lỗi; getMe không chứng minh hai chiều | P05/P11/P12 |
| Dự án / danh tính | Folder thực; defaults; assignment có nguồn; topic riêng; xem luật; sửa tên nhưng giữ key | Chưa gán; folder mất; thiếu quyền; xung đột; không đổi cwd/quyền theo nhãn | P01 |
| Báo cáo token | ECharts từ usage CLI/plugin; lọc ngày/múi giờ/CLI/account/model; drill-down; CSV | Chưa đo khác 0; cache không cộng đôi; độ phủ; counter reset; child rollup; cũ/offline | P10 |
| Hoạt động & chẩn đoán | Command/attempt/audit, mốc latency và nguồn; export theo quyền | Không đưa raw nhạy cảm vào log; thiếu receipt; crash chưa rõ kết quả; schema lạ | P02/P09/P12 |
| Cài đặt / bản cài | Layout, thông báo, retention, cấu hình động có quyền/revision/audit; update/rollback | Mặc định an toàn có nhãn; hết đĩa; migration lỗi; bundle lệch version; không tự tải/chạy | P11/P12 |
| Terminal tiếp quản | Nhúng/gốc cùng binding; lease quyền nhập; chỉ xem khi không có quyền điều khiển | Attach lỗi; tty tái sử dụng; dialog OS; lease cũ; mode chưa đo; không tạo phiên khác | P05/P06/P09 |

## State model — không gộp mọi thứ thành đang chạy / xong

Các trục độc lập. Tên hiển thị theo bằng chứng nguồn; unknown có lý do và lần cuối xác minh. Kết thúc lượt không chứng minh nghiệp vụ hoàn thành đúng.

| Trục | Trạng thái thiết kế | Nguồn / điều kiện | Hành vi UI |
| --- | --- | --- | --- |
| Session lifecycle | starting / open / closing / closed / unverified | Process/native session binding; snapshot và generation | Idle vẫn open; mất nguồn giữ known row unverified, không loại nhưclosed |
| Turn | idle / queued nếu hỗ trợ / processing / waiting-input / waiting-permission / ended / aborted / error / unknown | Lifecycle CLI + turn refs khi có; không chỉ lấy text cuối | Hiện outcome nguồn; job con vẫn độc lập; phần thiếu phải thấy |
| Command delivery | received / dispatching / CLI-accepted / in-composer / rejected / uncertain | Sổ intent + attempt + bằng chứng giao; command ID không thay native ACK | Uncertain không tự gửi lại; giữ bản tin chờ đối chiếu |
| Tool call | requested / permission-pending / running / returned / denied / error / unknown | tool-callID, args, result hoặc error | returned của launch chưa phải child/job ended; status metadata≠exit outcome |
| Child/job | launching / running / ended(exit/outcome) / aborted / unknown | Native child/job ID; liên hệ parent; runtime và kết quả | Tách khỏi số root; parent đóng không tự khép job |
| Source health | fresh / stale / disconnected / parse-error / unsupported / gap | Tách content_changed/read_ok/inventory_complete/binding_verified; mỗi nguồn và child/job riêng | Nguồn im lặng nhưng kiểm thành công không tự stale/ended; inventory kỳ vọng có phần chưa đo |
| Control | structured / terminal-lease / transitioning / unavailable | Core cấp lease theo actor + target generation + expiry | Mọi view xem theo quyền; chỉ chủ lease gửi raw keys; không trộn composer |
| Assignment | unassigned / confirmed(project,module,actor,revision) / conflict | User chọn hoặc giao rõ được xác nhận; không đếm đường dẫn | Topic/cwd riêng; đổi nhãn không đổi session key/context/quyền |

## Contract bổ sung cho nhiều view và runtime

Tên contract dự kiến, không khẳng định là API đang có. Localhost không tự có quyền. HostID là khóa local ổn định; hỗ trợ máy xa là mở rộng riêng, không mở LAN trong thiết kế này.

| Đối tượng | Khóa / trường tối thiểu | Bất biến | Phục hồi / lỗi |
| --- | --- | --- | --- |
| SessionKey | host_id, cli_kind, source_namespace_ref, native_session_id | Namespace native đã xác minh; account attribution riêng. Alias nhiều account không nhân đôi logical session | Native ID chưa có: giữ pending_launch_id; nhiều runtime cùng session có binding riêng và phải giải mơ hồ |
| RuntimeBinding | SessionKey, process/pane/console handle, binding_generation, driver_version | Thay viewer/binding không tự đổi danh tính; kiểm generation trước tác động | Handle cũ/tty dùng lại: từ chối hoặc đối soát, không gửi gần đúng |
| View / Panel | client_instance_id, view_id, panel_id, selected_session, layout_revision | Draft theo panel/session; focus view không đổi focus Tele; đóng panel không đóng CLI | Khôi phục layout nhưng xác minh phiên còn sống; không tự launch |
| Subscription | subscription_id, view_id, scopes, SessionKeys/filter, cursor, snapshot_watermark | Thu nguồn một lần; kiểm quyền tại core; replay event không replay command | Queue có giới hạn cấu hình; vượt giới hạn báo gap/resnapshot |
| Assignment | SessionKey, project_ref?, module?, origin, actor, revision, confirmed_at | Không assignment vẫn có Phiên chung; gợi ý khác xác nhận | Nhãn đoán cũ giữ provenance/unconfirmed, không nâng thành lựa chọn user |
| ControlLease | lease_id, lease_epoch, SessionKey, binding_generation, owner_client, expiry, mode | Barrier chặn structured/raw mới; drain hoặc hòa giải đang bay; fence writer cũ tại side effect rồi attach | Uncertain thì chưa cấp nhập; trả mode cũng qua barrier/composer; không phát lại phím; terminal ngoài không fence được phải báo giới hạn |
| Command | command_id, actor đã xác minh, target, generation, expected_revision, payload_ref | Idempotency theo ý định/phạm vi; payload khác cùng key phải từ chối | Ghi sổ trước hiệu ứng; uncertain hòa giải theo bằng chứng |
| Capability | OS, CLI, version, runtime_mode, feature, evidence_ref, verified_at | supported / unsupported / unmeasured là ba trạng thái; không đoán parity | Version đổi thì kiểm lại bằng chứng phù hợp; tắt action có giải thích |

## Ma trận giảm terminal — điều kiện phải đo trước khi bật mode UI-first

Dấu chưa đo là trạng thái hiện tại của thiết kế mới, không phủ nhận thao tác cũ đã có. Không chọn bỏ terminal hiện hành chỉ vì vẽ được UI.

| Khả năng | Trong UI mong muốn | Đường runtime / fallback | Điều kiện mở mode mới |
| --- | --- | --- | --- |
| Mở/attach/resume | Một session key đúng, không cửa sổ riêng thường trực | CLI + PTY/console khi cần; Linux tmux tách viewer; driver macOS/Windows riêng | Attach/resume/restart/identity trên OS + CLI/version thật |
| Chat và giao nhận | Text/Unicode/đa dòng, draft, receipt đúng mức | API hỗ trợ hoặc PTY driver; không ACK chỉ theo chữ màn hình | Mất ACK/double-click/crash/nhiều view; nguồn không chắc thì uncertain |
| Hỏi / quyền / input | Đủ câu/multi-select/mô tả/free-text; revision; gate quyền | Nguồn cấu trúc + driver chính thức; terminal nếu thiếu | Không trả lời bảng cũ; giữ từ chối; không nới cổng |
| Tệp / ảnh / output | Manifest/ref/bytes, code/text an toàn, bản gốc có nguồn | Khả năng attachment CLI/provider; ref mất báo lỗi | Phân biệt byte/ref/receipt; không nói model nhận bytes khi chỉ có path |
| Ngắt/đóng/job | Năm thao tác: đóng view, ngắt lượt, dừng job, đóng CLI, tắt dịch vụ | Driver/process group theo hỗ trợ; không giết bừa mọi job | Đọc nguồn sau tác động; nêu job sống/unknown; xác nhận ảnh hưởng |
| Tiếp quản | Terminal nhúng hoặc mở viewer gốc cùng phiên | Control lease + cùng binding; dialog OS có thể cần tay người | UI → terminal → UI giữ session ID/nội dung/draft, không mở phiên thay thế |
| Login / khôi phục | Wizard đúng account/lần login; offline/reconnect | URL/OTP/dialog OS theo yêu cầu; không tự mở browser | Đo hết hạn/mã cũ/tắt app/tắt viewer/daemon crash, giữ đường cứu hộ |

## Một kết quả tương tác đầy đủ gồm những phần nào?

Đầy đủ nghĩa là phân loại được mọi phần theo nguồn và có cờ thiếu/không áp dụng; không có nghĩa bắt provider phải xuất thinking hoặc payload ẩn. UI có thể gom tab, nhưng không làm mất loại dữ liệu.

| Phần kết quả | Nguồn / liên kết | Hiển thị / lưu | Khi thiếu hoặc lỗi |
| --- | --- | --- | --- |
| 1. Đích và nguồn thao tác | Actor/channel/project/account/CLI/session/model refs | Header phiên, chi tiết nguồn và audit | Unknown attribution, không suy từ tên folder symlink |
| 2. Input người dùng đã gửi | Command ID → delivery attempt → CLI input evidence | Timeline phân biệt soạn/nhận tại huba/CLI-accepted | Còn draft/composer hoặc uncertain; không dấu tích đã gửi giả |
| 3. Context quan sát được | Request ref → manifest luật/history/tools/transform nếu có | Tab context/source; phân biệt journal với payload | Không có hook → chưa cung cấp, không suy text UI = full context |
| 4. Trạng thái lượt | Turn start/end/abort/error theo nguồn | Đang chạy/chờ hỏi/ngắt/lỗi/hoàn thành/chưa đo | Không kết thúc bằng HTTP200/process spawn0; partial giữ nhãn |
| 5. Nội dung assistant | Role + content blocks + stream/final IDs | Text, Markdown an toàn, code copy, liên kết/tệp theo quyền | Streaming partial, lỗi giữa chừng, phần bị cắt hiển thị rõ |
| 6. Tool/native/MCP trace | Tool-call ID/name/args → permission → result/error | Hoạt động có nhánh mở chi tiết và raw refs | Tool denied/timeout/isError không chuyển thành tool thành công |
| 7. Câu hỏi/quyền chờ chủ máy | Question/permission ID + revision + options | Nút đúng bảng hiện hành, xác nhận đủ câu | Stale/mù/không hỗ trợ: không bấm tự động |
| 8. Tệp/ảnh/artifact | Manifest/hash nếu nguồn có; bytes/ref/path riêng | Xem/tải/copy theo khả năng và quyền | Ref mất/không có bytes hoặc chỉ preview phải ghi rõ |
| 9. Subagent/tác vụ nền | Child session/task refs, parent relation, state | Hoạt động con và liên kết, nền còn chạy sau abort | Không đếm mọi child lịch sử là đang chạy; không gộp context con vào cha |
| 10. Usage / context metrics | Canonical request usage, counter semantics, CLI window metric | Báo cáo token/coverage; context hiện tại riêng | Unknown/không áp dụng/0 thật tách biệt; no billing |
| 11. Provenance / raw / độ đầy đủ | Source kind/version/key/range/hash, parser revision | Xem bản gốc + gap/parse error/retention status | Không nhận hash1 record là coverage100%; không giả payload |
| 12. Hành động tiếp theo và audit | Capability + current identity + command refs | Trả lời/tiếp tục/ngắt/đóng/fork/export theo quyền | Phiên chết hoặc identity đổi thì tắt action cũ, không bấm nhầm đích |

## Dữ liệu và khóa liên kết đề xuất

Tên dưới đây là contract thiết kế, KHÔNG phải khẳng định các bảng/API đã có. Raw chỉ bất biến trong thời hạn lưu; policy lưu/xóa và index phải cùng revision, công bố gap khi dữ liệu bị xóa.

| Đối tượng | Khóa / quan hệ | Nguồn thật / bên sở hữu | Quy tắc |
| --- | --- | --- | --- |
| Account / project config | account_ref, project_ref, config_revision | Chủ máy cấu hình qua core; provider auth riêng | Không suy account từ thư mục symlink; revision/audit; model mặc định chỉ áp dụng đúng phạm vi |
| CLI session | host_id + cli_kind + source_namespace_ref + native_session_id; binding_generation riêng | CLI tạo/khôi phục; huba ghi binding và account attribution có nguồn | Namespace chung nhiều account không nhân đôi; generation không nằm trong SessionKey; pending launch không phải native session đã tồn tại |
| Command intent | command_id + principal + scope; source_message_id nếu có | UI/Telegram → core | Idempotency key theo yêu cầu, không theo nội dung: hai tin cùng chữ vẫn có thể là hai ý định thật |
| Delivery attempt | attempt_id → command_id + target_generation | Driver thực thi | Ghi trước/sau side effect; ambiguous không được replay tự động; khóa session không khóa cả app |
| Turn / model request | turn_id → request_id → attempt_id nếu nguồn cấp | CLI/provider events | Một user turn có nhiều request và retry; không có native ID thì local observation ID phải mang nhãn chưa ghép |
| Tool call/result | tool_call_id → request/turn; child_session_ref nếu có | CLI/tool/MCP | Giữ role/type, args/result/errors; parent-child không đồng nghĩa share nguyên context |
| Raw observation | raw_id + source_ref + generation + source key/range + hash | Collector đọc nguồn chính thức | Lưu byte record file hoặc structured DB snapshot có provenance; hash snapshot không chứng minh byte file DB; không sửa raw để đẹp UI |
| Normalized event | event_id → raw_ref(s), parser_version, source_event_id? | Projection có thể dựng lại | Replay từ raw, khử trùng, giữ parse-error; không thay thế raw bằng preview đã trim |
| Usage canonical | request_ref + source version + counter scope + canonical revision | Normalizer từ raw usage | Lưu observations trước; snapshot mới có revision; tổng chỉ lấy canonical hiện hành, không cộng mọi snapshot |
| Read / ingest cursors | source_cursor, projection_checkpoint, client_cursor | Collector / worker / client riêng | Không dùng chung một cursor cho ba việc; ingest commit khác client đã đọc; retention gap là trạng thái có thể xảy ra |
| Context manifest | request_ref → input source refs / transform / visibility | Hook/runtime CLI nếu hỗ trợ | Manifest chỉ phần quan sát được; system ẩn/tokenizer ẩn không suy từ journal; không nhận self-report của model làm bằng chứng đủ |
| Retention / audit | policy_revision, actor, action, affected refs, timestamp | Chủ máy/core | Raw nhạy cảm theo quyền; thay policy phải thấy tác động, không xóa silent; audit không chứa raw hay secret |

## Context model chứa gì — và không tự chứa gì?

Đây là mô hình khái niệm, không cam kết thứ tự serialize của từng CLI/provider. Mọi phần chỉ được đánh dấu đã quan sát khi có nguồn chứng minh.

| Thành phần | Ai quyết định | Khi nào vào input model | huba biết đến đâu |
| --- | --- | --- | --- |
| System/platform instructions | CLI/provider | Theo runtime/provider | Có thể ẩn hoặc biến đổi; chưa thấy thì ghi không cung cấp |
| Luật workspace/project/subfolder | Chủ máy + quy tắc nạp CLI | Tự nạp theo khả năng CLI hoặc đọc trực tiếp khi vào phạm vi | Có thể biết file/hash đã đọc; chưa đủ chứng minh nguyên payload request |
| Lịch sử hội thoại | CLI context builder | CLI chọn lịch sử, compact hoặc summary | Kho raw có nhiều hơn context hiện tại; UI scroll không thay đổi context |
| Tin mới / attachment | Người dùng + CLI adapter | Sau khi CLI nhận và đưa nội dung vào request | Có thể chỉ có ref tệp; chưa đọc tệp ≠ đã gửi byte tệp tới model |
| Tools và schema MCP | Config quyền + CLI | Khi runtime expose cho request; eager/lazy tùy CLI | Schema có thể tiêu token; không đồng nghĩa dữ liệu cả server đã được nạp |
| Tool/MCP result | Tool thực thi; CLI chọn representation | Request tiếp theo nếu CLI đưa kết quả vào messages | Có raw result và có thể có bản trim; hai bản phải có nhãn, không nhận trim là toàn bộ |
| Memory/RAG/resource đọc thêm | CLI/tool được cấp quyền | Chỉ khi được đọc và đưa vào input | Không phải mọi file memory hay mọi trang UI tự vào model |
| Summary/compaction | CLI; có thể dùng model | Thay phần lịch sử theo semantics CLI | Raw cũ vẫn lưu theo policy; nếu có gọi model phải ghi usage riêng khi nguồn có |
| Auth header/token cấu hình | Secret store / transport | KHÔNG chủ động đưa vào prompt | Chỉ tên secret/config refs; body nguồn tự chứa bí mật thì xử lý raw nhạy cảm, không tự gửi tiếp |
| Draft UI, bộ lọc báo cáo, nhật ký UI | Ứng dụng/chủ máy | KHÔNG tự vào model | Chỉ vào nếu người dùng chọn gửi qua command rõ ràng |
| Output assistant / tool-call | Model | Trở thành lịch sử có thể được CLI chọn cho request sau | Output stream/final/hidden thinking phân biệt; phần không xuất không bịa |

## Ví dụ token — số minh họa, không phải số tài khoản thật

Giả sử nguồn đã xác nhận input_total gồm cache. Cache write không áp dụng trong ví dụ. Không dùng ví dụ này như semantics chung của mọi provider.

| Request | Input tổng | Cache read (nằm trong input) | Input chưa cache | Output | Usage cộng kỳ |
| --- | --- | --- | --- | --- | --- |
| R1 | 6 000 | 0 thật | 6 000 | 1 000 | 7 000 |
| R2 | 7 600 | 5 000 | 2 600 | 800 | 8 400 |
| Cộng hai request | 13 600 | 5 000 thành phần con | 8 600 | 1 800 | 15 400, KHÔNG phải20 400 |
| Context đầu R2 | 7 600 theo ví dụ input thực | Không cộng cache lần nữa | Không lấy tổng usage13 600 làm context | 800 là output phát sinh sau input | Giới hạn context phải lấy CLI/provider, chưa có thì unknown |
| Độ phủ ví dụ | 2/2 request đủ | 0 ≠ unknown ≠ không áp dụng | Request thiếu giữ riêng | Reasoning không cộng đôi nếu đã nằm output | Tổng request không phải số token duy nhất từng xuất hiện |

## Một trace xuyên ba lớp — hoàn toàn minh họa

Ví dụ cách truy vết UC-05. Không tạo dữ liệu nghiệp vụ thật và không khẳng định các ID này đang tồn tại.

| Bước | ID / dữ liệu | Nguồn → đích | Context / kết quả |
| --- | --- | --- | --- |
| 1. Chủ máy gửi | C17 → session S8, nội dung kiểm một tệp | Desktop → core | Mới là ý định; chưa biết model đã nhận |
| 2. Driver giao | A17 → C17, target_generation G3 | Core → CLI | CLI input record khớp → CLI-accepted; thiếu correlation thì chưa xác định |
| 3. CLI gọi model | T4 / R1 nếu nguồn cung cấp | CLI → provider | Luật + phần lịch sử CLI chọn + schemas + tin mới; payload chỉ thấy khi có capability |
| 4. Model yêu cầu tool | K3 → R1, loại Read | Model → CLI → tool | Gate đọc/ghi kiểm trước thực thi; nội dung tệp là tool result, không phải system instruction |
| 5. CLI gọi tiếp | R2 ← K3 result, cùng T4 | CLI → provider | Context mới có tool result representation; R1 và R2 đều có usage riêng |
| 6. Thu nguồn | RAW51/RAW52 → E80, U1/U2 | Collector → raw → projection | Raw giữ loại nguồn; canonical usage khử stream/final; không ghép chỉ bằng timestamp |
| 7. Chủ máy xem | E80 → timeline; source refs → raw | Core → Desktop / kênh theo dõi | Hiện output và turn status; xem raw không gọi model thêm |
| 8. Báo cáo | U1 + U2 trong filter F1 | Usage index → chart/CSV | Tổng request usage, không lấy context T4 hoặc quota% thay thế |

## Nguồn khảo sát bản15/11 — đối chiếu lịch sử

Các số dòng dưới đây giữ mốc15/11, không được coi là vị trí mã hiện tại. Bằng chứng đọc lại ở lượt V2 nằm trong P01–P12 và đặc tả thiet-ke-tong-the-v2.md. Không phải bản đồ phụ thuộc runtime đã đo.

| Nguồn mã / thiết kế | Điều quan sát được | Không suy rộng thành |
| --- | --- | --- |
| rust/src/accounts.rs:889–919,941–1084,1087–1107 | Nguồn luật gốc, cấu hình OpenCode, plugin quyền/MCP/model | Context payload đầy đủ hoặc collector usage đa CLI đã triển khai |
| rust/src/sessions.rs:7198–7227 | Stream đọc đuôi transcript và parse ra hiển thị | Kho raw đầy đủ, append-only hoặc nguyên byte model request |
| rust/src/sessions.rs:7255–7292 | Prompt bàn giao và khái niệm fork chỉ đọc | Fork mọi CLI đều có semantics/khả năng ngang nhau |
| rust/src/opencode.rs:72–145 | Đọc SQLite read-only, hai thế hệ session/session_v2 | API ổn định xuyên phiên bản hoặc tất cả block/payload đã được thu đủ |
| rust/src/codex_sessions.rs:352–409 | Parser rollout và last_token_usage/context window | Báo cáo tổng usage mọi request trong kỳ |
| rust/src/pipeline.rs:8242; rust/src/telegram.rs:1049,2612 | Cửa giao phiên, nhận diện sender và nhận tin Telegram | Desktop app service mới đã tách và chạy |
| docs/thiet-ke-desktop.md:48–80,126–179,237–290 | Đề xuất contract, provenance, gửi có kiểm nhận và token-only | Quyết định sản phẩm đã duyệt hoặc app đã nghiệm thu |

## Các nhánh lỗi phải hiện rõ trên thiết kế
- Chưa cấu hình / dữ liệu rỗng thật / probe hỏng / nguồn chưa hỗ trợ là bốn trạng thái riêng; giữ snapshot cũ có nhãn khi không đo được.
- Crash giữa ghi ý định và gửi: phục hồi theo evidence, không replay mù. Crash sau side effect mà chưa ghi ACK: giữ uncertain cho tới hòa giải; mọi auto retry cần chứng minh chưa thực thi.
- Rotation/truncate/file replacement/schema mới: source generation và gap; parser hỏng vẫn giữ raw. Ingest/projection/client cursor riêng; không nhầm nguồn đọc được với UI đã nhận.
- Question revision cũ / tty dùng lại / phiên đổi profile: kiểm lại ngay trước tác động; không chỉ kiểm ở lúc người dùng mở màn.
- Input/output/attachment quá lớn: preview có thể cắt nhưng nhãn rõ; raw theo policy. Không đặt limit cắt cứng âm thầm và nhận đầy đủ.
- Scope/raw nhạy cảm/export: chỉ chủ thể có quyền, audit không ghi nội dung bí mật. Model/tool output không được trở thành quyền quản trị hoặc chạy script trong UI.
- Nhiều nguồn usage cùng request: giữ observations, chọn canonical theo semantics/version; retry có thực phát sinh usage không được xóa chỉ vì cùng turn. Không gộp cache và reasoning lần hai.

## Các quyết định còn cần duyệt / spike, không tự chốt trong lượt vẽ
- Mức gốc cần nghiệm thu: J = journal CLI giữ nguyên theo nguồn; P = body request/response model nếu CLI có điểm mở rộng đã đo. Thiết kế chứa cả hai đường, không nhận P sẵn có.
- Ưu tiên Claude CLI + OpenCode theo thiết kế trước; không tự xóa/chuyển Codex đang tích hợp. Tích hợp khác model/provider không được mặc định dùng chung thuê bao hoặc mọi tính năng.
- Tauri là đề xuất vỏ đa OS; cần spike WebView/IPC/capability và native packaging trên máy thật. Chưa có app V2 để nghiệm thu UC01–22 xuyên ba hệ.
- Thời hạn lưu/raw nhạy cảm/dung lượng, quyền export, danh mục model và ngưỡng vận hành phải cấu hình bằng dữ liệu có quyền quản trị + audit; bí mật vẫn ở secret store/env.
- Hiện có thể duyệt luồng và nội dung bản vẽ. Browser kiểm tài liệu chỉ chứng minh tài liệu hiển thị, không thay cho UI→core→CLI→model→nguồn thật của sản phẩm.

---
SINH từ `docs/flows/design.json` bằng `python3 docs/flows/build.py`; không sửa tay bản sinh. Sơ đồ thiết kế không phải bản đồ phụ thuộc trích từ mã/vết chạy.
