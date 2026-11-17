# Xác minh lại bản chốt trên hai engine — 17/11/2026 09:55+07

Lượt 16/11 12:23 sinh lại `index.html` (nhúng lại W0 sau khi sửa footer) nhưng chỉ
chạy unit + `pin_review` trên bản riêng; **19 sơ đồ/12 issue link của `index.html`
hiện tại chưa từng được kiểm bằng browser**. Lượt này chạy đúng bộ đó trên tệp
đang có, không sửa tài liệu nào.

| Đo trên bản đang có | Kết quả |
|---|---|
| `index.html` 337843 B · `328d0c35cdecd9026276f85e89143cf6256abcb24913a120e6c4d75bc30d151f` | Chromium + WebKit cùng đọc hash này trước/sau lượt chạy |
| `workspace.html` 22948 B · `e5f5f5e8…1c315` | `standalone_sha256` khớp ở cả hai engine |
| Chromium (`.tmp/verify-index-chromium-20261117.log`) | `GATE_EXIT=0` · 19 flow/30 thành phần/22 UC/12 issue · console-page-request 0 · remote 0 |
| WebKit (`.tmp/verify-index-webkit-20261117.log`) | `GATE_EXIT=0`, cùng mẫu số và cùng hash |
| Unit | `python3 -m unittest discover -s docs/flows -p 'test_*.py'` 17/17, mã 0 |
| Màn hẹp | page390/390 cả hai engine; sơ đồ cuộn 1000 trong 326 |
| Đối chứng ngược (cổng ghim B) | Bản sao `workspace.html` cấy lại `min-height:740px` đúng gốc lỗi cũ ⟹ `pin_review` **ĐỎ**: `GATE_EXIT=1`, "*B must be visible without page scrolling*", B tại y720…843 ngoài viewport 720 (`.tmp/counterproof-pin-20261117/red.log:2,15`) |

Vẫn chỉ là tài liệu/bản mẫu: chưa app, chưa nối CLI/model, không sửa Rust/config.
Bản sao đối chứng nằm trong `.tmp/`, tệp thật không bị đụng (hash trên đo lại sau lượt).
Số `336969B/hash 7e2f3ce3…` ở mục dưới là **lịch sử trước lần sinh lại 12:23**.

# B luôn hiện, chỉ A cuộn — 16/11/2026 12:24+07

Sửa riêng bản mẫu `workspace.html`: khung app bám `100dvh`, body không cuộn cả
trang, phần giữa co lại để luôn dành chỗ footer B. A có scroll nội bộ. B không
di chuyển khi wheel trong A; header/menu/cột phải không bị kéo theo. Tài liệu
thuyết minh dài `index.html` không phải shell ứng dụng cố định viewport.

- RED trước sửa: `.tmp/pinned-B-red.log:2`, cửa sổ1280×720 nhưng trang856px,
  B tại y720…843 nằm ngoài viewport. Gốc là `min-height:740px` và chiều cao shell
  tính cứng thay vì flex theo phần không gian còn lại.
- So ảnh còn thấy nội dung A/chi tiết tràn lên B ở1440×600; bổ sung regression
  `.tmp/pinned-B-overlap-red.log:2` bắt A.bottom518.6 > B.top464. Sửa grid row
  `minmax(0,1fr)`, giữ overflow trong phần giữa; menu nén khoảng cách khi thấp.
- GREEN cuối: `.tmp/pinned-B-{chromium,webkit}-green.log:1–3`, mỗi engine3/3 cỡ
  1440×900,1280×720,1440×600; scrollY trang0, B luôn trong viewport và không đổi
  vị trí sau wheel, A thực sự đổi scrollTop. A/detail không lấn footer.
- Đối chứng: ép shell1400px phải làm phép đo nhận B ra ngoài màn; phục hồi layout
  lành B phải hiện. Cả hai engine console/page/request errors0, remote0, exit0.
- Unit17/17 mã0. Chỉ chạy bộ theo ảnh hưởng này; không khai đã chạy lại toàn bộ
  interaction/19flow hay QG Rust. Script giữ regression: `pin_review.cjs`,
  `workspace_review.cjs::reviewPinnedFooter`.
- Đã mở ảnh cuộn A ở600px Chromium và720px WebKit:
  `previews/pinned-B-20261116/{chromium,webkit}/`. Hash HTML riêng cuối
  `e5f5f5e848a40b063114be573313da2fa9eafdc3b7f33f27da84e382eab1c315`.

Chỉ bản mẫu/tài liệu, chưa app hay dữ liệu CLI thật. Không cài/restart/commit/push;
config/Rust giữ, không có job nền. Số đo/hash các mục dưới là lịch sử trước lượt này.

## Chỉnh bố cục ô chat + footer — 16/11/2026 10:42+07

Theo phản hồi mới: A không còn chỉ là thẻ tóm tắt; mỗi ô có hội thoại, input,
icon/trạng thái và việc đang làm riêng. Focus bằng click/bàn phím đổi cột chi tiết;
hover không giành focus. B là footer dưới toàn bộ menu+A+chi tiết, có icon trạng
thái/việc đang làm/tin gần nhất, không phụ thuộc focus.

Bản mẫu độc lập: `workspace.html`; cùng nguồn `workspace_mockup.py` được nhúng W0
trong `index.html`. **Chỉ fixture minh họa**: nút mô phỏng và gửi thử chạy trong
trang, không mạng/storage/CLI/model. Có script tin cậy của bản mẫu và JSON fixture;
không thực thi nội dung người dùng/model như mã.

| Kiểm áp dụng cho tài liệu | Bằng chứng mới |
|---|---|
| Cấu trúc/tái sinh | Unit17/17 exit0; HTML tổng/HTML riêng khớp nguồn;19flow/22UC/12Pxx giữ |
| Chromium + WebKit | `.tmp/review-layout-v3-{chromium,webkit}.log:1–3`, cả hai GATE_EXIT0; console/page/request0, remote0 |
| Ô chat thật sự trong mẫu | Mỗi engine kiểm cả bản nhúng và độc lập:4chat/4textarea, activity nhìn thấy ở cả4ô, focus đổi đúng chi tiết, hover không đổi |
| Không phụ thuộc focus | Đang chọn S03, bấm mô phỏng thấy S01 cập nhật activity+chat và footerS05 nhận tin mới/icon chạy; S03 vẫn được chọn, draftS02 giữ |
| Footer đầy ngang | Đo bounding rect cùng chiều ngang shell, dưới body; cấy B vào vùng giữa phải đỏ, trả về shell phải lành |
| Tương tác khác | Gửi thử giữ chữ `<img…>` như text, không tạo img/script và ghi chưa gửi CLI; mở footer lênA/thu xuốngB đúng số; draftS04 giữ qua thu/mở |
| Hình/tài liệu nền |19SVG/22UC/12issue links mỗi engine; bboxchồng0, cấy chồng đỏ/reload lành0; trang390px không tràn, khung desktop1120px cuộn trong326px |
| Ảnh tự xem | Đã mở `previews/v3-layout-20261116/chromium/workspace-standalone.png` và WebKit `workspace-standalone-focused-updated.png`; footer ở dưới cả ba cột, chat/input không bị che |
| Giới hạn | Desktop mockup, chưa responsive app hay nguồn realtime; menu là nhãn minh họa, không có service tích hợp; chưa chạy UC sản phẩm thật |

HTML tổng336969B/hash`7e2f3ce3…`; HTML riêng22422B/hash`827aa9ca…`; khớp hai engine
và đọc lại nguồn10:42. Không sửa Rust/config/cài/restart, chưa commit/push. Không có
job nền của lượt này. Các số V2 bên dưới là **lịch sử trước chỉnh bố cục**, không
phải hash/shape hiện tại.

## Lịch sử kiểm bản vẽ V2 — 16/11/2026

**Chỉ kiểm tài liệu thiết kế; chưa triển khai/nghiệm thu sản phẩm V2.**
Đặc tả: `../thiet-ke-tong-the-v2.md`; bản vẽ tự chứa: `index.html`.

| Chiều | Kết quả / phạm vi bằng chứng |
|---|---|
| Tái sinh/cấu trúc | `design.json` → `build.py` → HTML/Mermaid; unit15/15 mã0;19 sơ đồ,22UC,30 thành phần,12 mã P01–P12 |
| Điểm vấn đề | Validator kiểm state/evidence/flow/step; cấy sai state/thiếu evidence/trỏ bước không có/trùng ID đều đỏ, bản lành không đỏ; markup được escape |
| Nghĩa nhánh | Mermaid dùng alt/else/break; các hàng lỗi liền nhau cùng nhánh. Regression D5 bảo đảm probe lỗi vẫn báo unknown trước kết thúc, không rơi sang đường thành công |
| Chromium | `.tmp/review-flows-v2-chromium-final.log:1–3` GATE_EXIT0;19/19 SVG,22UC,12issue links hai chiều; console/page/request errors0; remote0 |
| WebKit | `.tmp/review-flows-v2-webkit-final.log:1–3` cùng mẫu số và GATE_EXIT0; không gọi đây là Tauri/WebView native đã chạy |
| Hình/bố cục | Hai engine: SVG bbox chồng0; cấy hai nhãn đè nhau được phát hiện, reload lành0. Đây là bbox trình duyệt, không phải kiểm pixel/màu/logo |
| Màn hẹp | RED thật page450px ở viewport390; cô lập do invariant U7 chứa chuỗi slash dài; sửa wrap. Hai engine GREEN page390/390; diagram cuộn1000 trong326, wireframe cuộn1040 trong326 |
| Ảnh tự xem | Đã mở ảnh tổng thể/W0/T0, U5/U6/U7/D5/D6/C5/P09 và màn hẹp ở các lượt soát; browser chụp đủ19flow,12issue,overview/workspace/terminal/hover/narrow. Không tuyên bố đã xem bằng mắt mọi pixel của mọi ảnh |
| Security/contract | Không thêm endpoint/dependency; HTML offline không script; nội dung nguồn escape. Source JSON và cả hai consumer sinh HTML/MD cùng thay đổi; prototype sản phẩm cũ được ghi rõ chưa đổi |
| Runtime/build/cài | N-A cho yêu cầu vẽ: không sửa Rust/config/runtime, không build/cài/restart hoặc chạy QG toàn kho. Không dùng kiểm tài liệu thay UI→core→CLI→model thật |
| Tình trạng | Còn duyệt P04/P06/P07 và spike capabilities/OS; chưa triển khai sản phẩm, chưa commit/push lượt vẽ |

HTML cuối:318004B, SHA256
`49236e25d90d94683681ca49ac00e32ebb0dc97d4338e455cfa4d240d52c6b8e`,
khớp đầu/cuối hai browser và đọc lại nguồn10:17+07. Ảnh ở
`previews/v2-20261116/{chromium,webkit}/`; log RED ở
`.tmp/review-flows-v2-chromium.log`, cô lập `.tmp/design-overflow-isolation.log`.
Lượt không có job nền còn chạy. Khung W0 là bản vẽ ngang, không phải responsive app
đã triển khai; mô tả tab/stack trên điện thoại vẫn là thiết kế.

## Lịch sử bản15/11 — số đo dưới đây không áp dụng thay cho V2

Đây là kiểm **tài liệu thiết kế**, không nghiệm thu ứng dụng hoặc dữ liệu CLI/model thật.

| Chiều kiểm | Kết quả / bằng chứng |
|---|---|
| Nguồn và tái sinh | `design.json` → `build.py` → `index.html`, `index.md`; không có runtime dependency mới |
| Cấu trúc | `python3 -m unittest discover -s docs/flows -p 'test_*.py' -v`: 9/9 qua, mã0 |
| Mẫu số | 30 thành phần logic; 16 UC; 13 sơ đồ; ma trận22 loại dữ liệu; kết quả12 nhóm thành phần |
| Đối chứng | Actor không tồn tại, UC trỏ sai flow, trùng ID, thiếu lớp, nhãn quá dài đều bị từ chối; markup trong nhãn được escape |
| Kiểm hình | RED chữ tiếng Việt ở line-height17 → GREEN line-height22/giãn hàng. Chromium kiểm 13/13 sơ đồ,0 bbox chồng; cấy hai nhãn trùng vị trí → bộ dò phát hiện, reload lành →0. Đây là số đo bbox trình duyệt, không gọi là phép đo màu/điểm ảnh |
| Tự xem ảnh | Đã mở ảnh13 sơ đồ, ảnh toàn cảnh `previews/X0-full.png`, hover và390px. Sửa thêm min-width để sơ đồ tối đa5 cột vừa desktop; sơ đồ tổng11 nhóm kéo ngang có chủ ý |
| Browser/asset | Script mang theo Git `review.cjs` (Playwright đã cài, browser path tùy env); log lịch sử `.tmp/flows-review.log`, `.tmp/flows-review-final.log`; lượt kiểm lại trước commit `.tmp/flows-review-commit.log` GATE0/hash khớp |
| Kết quả browser | Log cuối GATE_EXIT0, console/page/request errors0, remote requests0; bấm13 liên kết/13 bảng bước. Màn390px không tràn trang, sơ đồ tự cuộn1000px trong vùng326px |
| WebKit | Chưa đo13 sơ đồ bằng WebKit. Kế nhiệm21:14 xác nhận host đã có WebKit2311 và chạy3/3 bài prototype riêng; kết quả đó không thay kiểm sơ đồ |
| Mã sản phẩm / build / deploy | Không sửa core/CLI/provider hoặc dựng/cài app. Lượt gom theo lệnh commit đã chạy QG: fmt/clippy0, nhưng chủ động ngắt130 khi bước dựng test Rust còn chạy để chuyển phiên; KHÔNG có QG toàn kho xanh |
| Tích hợp thật | Chưa nghiệm thu UI→core→CLI→model→tool; các contract mới vẫn là đề xuất |

HTML bản chốt:207930B, SHA256 `c141693f8a8cf2061ac37e025f1acca8c17d902ea9085a9d24af86d1fca86a1a`, khớp hash đầu/cuối lượt browser. Không còn job nền.

**Kết luận:** tài liệu đã có kiểm cấu trúc và hình Chromium như trên; **chưa nghiệm thu đầy đủ đa engine hoặc bất kỳ UC sản phẩm thật nào**. Thiết kế vẫn chờ chủ máy duyệt trước triển khai.
