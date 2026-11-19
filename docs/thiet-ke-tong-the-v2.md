# huba V2 — thiết kế tổng thể và sổ điểm cần bàn

16/11/2026. **Bản đề xuất mới để bàn, chưa triển khai sản phẩm.**

Bản vẽ đầy đủ: [HTML offline](flows/index.html), [sơ đồ dạng chữ/Mermaid](flows/index.md).
Nguồn sơ đồ và điểm đánh dấu: `flows/design.json`; bộ sinh: `flows/build.py`.
Bản này bổ sung/thay các phần bàn làm việc, danh tính và runtime của
[thiết kế 15/11](thiet-ke-desktop.md); không hủy các nguyên tắc an toàn, provenance,
token-only hay các khả năng đã tích hợp. Prototype `ui/index.html` vẫn là mẫu cũ,
**chưa mô phỏng bàn hai vùng V2**.

## 1. Kết luận về terminal

**Nếu thực hiện đủ thiết kế: làm việc thường ngày trong huba, gần như không cần
mở cửa sổ terminal riêng. Không đồng nghĩa bỏ CLI hoặc mọi terminal.**

Phân biệt bốn lớp:
1. **CLI runtime:** vẫn sở hữu phiên, lịch sử/context, vòng model/tool và xác thực.
2. **PTY/console hoặc API điều khiển:** đường giao tiếp với runtime; phụ thuộc CLI/OS.
3. **Terminal viewer:** giao diện hiển thị/nhập thô, có thể nhúng hoặc là cửa sổ gốc.
4. **UI cấu trúc của huba:** chat, trace, câu hỏi/quyền, tệp, quản lý và báo cáo.

UI thay phần lớn công việc của lớp 3; không tự thay lớp 1 bằng agent mới của huba.
Nguồn/API không cung cấp đủ thì giữ terminal tiếp quản. Đăng nhập tương tác, dialog
OS, cứu hộ driver và ứng dụng toàn màn hình có thể vẫn cần terminal hoặc tay người.
Tauri chỉ là vỏ app, không tự giải quyết những khả năng này.

Hiện tại Linux đã tách pane tmux khỏi cửa sổ xem (`keys_tmux.rs:646–658`);
macOS còn điều khiển qua Terminal (`keys.rs:1641–1665,1700–1709`);
Windows source nói rõ console/ConPTY chưa chạy trên Windows thật (`keys_win.rs:1–37`).
Đó không phải bằng chứng cả ba hệ đã có terminal nhúng hoặc mode UI-first.

**P06 cần bàn:** mục tiêu mặc định tương lai là “chỉ mở terminal gốc khi cần”.
Luật hiện hành vẫn yêu cầu đường chủ máy tiếp quản phiên thật (`CLAUDE.md:24–44`).
Chưa duyệt thay luật/đo driver thì giữ cách mở hiện hành, không âm thầm chuyển
phiên đang chạy thành headless hoặc đóng cửa sổ của người dùng.

## 2. Khung sản phẩm: một core, nhiều mặt xem

```text
Desktop: nhiều bàn/view ─┐
Telegram: chọn đích riêng ├─ Gate quyền ─ Application service ─ Command dispatcher
MCP facade tùy chọn ────┘                    │                     │
                                  Query/subscriptions          CLI driver
                                            ↑                     │
                             Registry/graph/projection       Host/runtime
                                            ↑                     │
                                 Raw + ingest cursor ← Collector ← CLI
                                                           │       │
                                                    provider/model tools/MCP
Terminal nhúng/gốc ── lease + binding đã xác minh ──────────── CLI/PTY
```

- Một hubad vận hành nghiệp vụ, một nguồn command và event; không tạo thêm poller
  Telegram hoặc backend JavaScript cho cùng việc.
- Tách `ConversationSource`, `SessionDriver`, `Host`, `Channel`, `ApplicationService`.
  Thêm model/provider không mặc nhiên cần thêm driver CLI.
- Ưu tiên Claude CLI và OpenCode theo hướng đã trao đổi; giữ Codex hiện có,
  không tự xóa/chuyển tài khoản. Không hứa một thuê bao dùng được qua mọi CLI.
- Frontend dùng generated contract/client, không gọi shell hoặc tự đọc DB CLI.
- Tauri/frontend TypeScript là đề xuất cần spike; framework và vỏ chưa được coi là
  quyết định triển khai chỉ vì tài liệu đã ghi tên. Kiểm WebView2/WebKit thật.
- IPC local theo OS peer identity + pairing/scopes; core kiểm quyền. Không mở LAN
  mặc định, không dựng cloud login hoặc quản lý máy xa ngoài phạm vi đã yêu cầu.

## 3. Bàn làm việc hai vùng và nhiều cửa sổ

### Hai vùng trong cùng một bàn

**Chỉnh theo phản hồi tiếp của chủ máy:** mục tiêu là nhìn nhiều phiên đang làm việc
giống nhiều cửa sổ, không phải nhiều thẻ tóm tắt phải bấm vào mới biết đang làm gì.

**A — Các ô chat làm việc luôn nhìn thấy:** mỗi ô có tên/ID, account/CLI, icon và
chữ đang chạy/đang dừng/chờ hỏi/chờ quyền, dòng **đang làm gì**, lịch sử chat cuộn
riêng và ô soạn/gửi riêng. Hoạt động và tin mới cập nhật ngay cả khi ô không được
focus. Focus bằng click hoặc bàn phím chỉ chọn ô và cập nhật cột chi tiết bên phải;
hover không giành focus, không tráo draft. Cột phải mở tool/vòng xử lý/child/job,
nguồn và kiểm nhận của đúng ô đã chọn; không bắt các ô A khác mất hội thoại.
Số panel, tỉ lệ và layout lưu theo view; bản mẫu 2×2 chỉ để xem bố cục. Khi thêm
panel vượt diện tích đủ đọc thì cuộn vùng A/đổi layout, không ép mọi ô đến mức mất chat.

**B — Thanh footer đầy chiều ngang ở đáy ứng dụng:** nằm dưới cả menu trái, vùng A
và cột chi tiết, **luôn hiện trong khung nhìn khi cuộn A hoặc đổi chiều cao cửa sổ**.
Chỉ vùng A sở hữu cuộn danh sách ô chat; không kéo cả trang khiến footer biến mất.
Khung app bám chiều cao viewport, phần giữa co lại để giữ chỗ B; không đặt chiều
cao tối thiểu khiến app cao hơn cửa sổ. Đây không là bảng lớn bên trong A.
Mỗi phiên còn lại là một mục gọn:
tên/ID + icon/chữ trạng thái luôn thấy; đang làm gì; tin gần nhất nếu nguồn có.
Phiên chạy vẫn hiện icon chạy dù không focus. Không có nội dung/hoạt động từ nguồn
thì ghi chưa có/chưa đo, không bịa. Footer vẫn cập nhật khi đang đọc/gõ tại A;
tin mới không tự kéo focus hoặc đổi thứ tự mục. Nhiều mục thì cuộn ngang và giữ
tổng/phần chưa xác minh rõ. Chọn mục footer đưa phiên lên A, không mở CLI mới.

Bản mẫu tương tác riêng: [flows/workspace.html](flows/workspace.html), cũng nhúng
ở W0 trong bản tổng thể. Tất cả dữ liệu và nút gửi/mô phỏng chỉ chạy cục bộ trong
trang, không gửi CLI/model và không phải nguồn realtime đã tích hợp.

Với `O` là tập root session mở đã xác minh trong scope của bàn; `Panels(v)` là
tập khóa đang có panel trên view đó:
- `A(v) = O ∩ Panels(v)`, `B(v) = O \ A(v)`; vậy `A ∩ B = ∅`, `A ∪ B = O`.
- Đếm theo SessionKey duy nhất, không theo số panel hoặc process.
- Cùng phiên có thể ở nhiều bàn; mỗi bàn đếm đúng một lần trong scope của mình.
- Tổng nhiều bàn dùng hợp các khóa cùng scope, không cộng tổng từng bàn.
- Child/job có graph và bộ đếm riêng, không tăng tổng phiên người dùng.
- Bộ lọc có “đang hiện/tổng chưa lọc”. Không để filter tạo ảo giác đã phủ mọi phiên.
- Root đã đóng chuyển lịch sử. Root mất nguồn còn giữ nhóm **chưa xác minh** cùng
  ảnh cũ; nếu nguồn mù thì tổng toàn bộ chưa thể xác nhận, phải hiện thiếu phần đó.
- Panel mất nguồn hoặc ra ngoài bộ lọc scope có thể còn hiện để chủ máy không mất
  chỗ đọc, nhưng gắn nhãn riêng và không tính vào A/B của scope mới. Thu hồi quyền
  không chỉ là đổi bộ lọc: ngừng stream/action và xử lý cache theo policy quyền.

### Không giành focus của nhau

Mỗi app instance/cửa sổ có ViewID; panel, draft và read cursor riêng. Chọn phiên
trên desktop không đổi đích Telegram hay cửa khác. Xem không chiếm quyền nhập thô.
Mở panel không mở CLI mới; đóng panel cuối của phiên trong view mới trả phiên về B,
không ngắt/đóng CLI.
Mở lại app khôi phục layout sau xác minh, không tự launch phiên lịch sử.

Header và footer B toàn chiều ngang, logo+tên trái, tài khoản chủ máy/cài đặt phải;
nút thu menu ở hàng đầu menu trái. Màn hẹp là nhánh thiết kế tiếp; bản mẫu này ưu
tiên so bố cục desktop, khung rộng cuộn trong trang, chưa giả đã làm responsive app.
Bảng có STT xuyên trang, nền xen kẽ, tìm toàn bộ dữ liệu và phân trang.

## 4. Danh tính: sửa tận quy tắc, không sửa mỗi chuỗi nhãn

| Trường | Nghĩa / bên xác lập |
|---|---|
| SessionKey | `host_id + cli_kind + source_namespace_ref + native_session_id`; khóa ổn định |
| RuntimeBinding | PID/pane/console handle + generation; kiểm lại trước tác động |
| Assignment | Dự án/module được người dùng giao/chọn rõ, có nguồn và revision |
| Topic | Tên cuộc trao đổi; tự sinh thì ghi là gợi ý, không làm assignment |
| Working scope | Cwd/workspace quan sát được; không phải quyền ghi hay dự án đã giao |
| Display name | `Phiên chung · ID` hoặc dự án/module đã xác nhận + topic tùy hiển thị |

**P01 lỗi đã xác nhận:** phiên kiểm tra máy bị gán `[zkey]`, rồi CLI trả `[scripts]`
do đếm đường dẫn trong log; không phải user đã chọn các dự án đó. Bằng chứng:
[báo cáo ca thật](ra-soat-ten-phien-goc-20261116.md).

Không dùng path census, prefix assistant hoặc tên ứng dụng trong văn bản làm nguồn
giao dự án có thẩm quyền. Câu giao rõ hoặc lựa chọn có cấu trúc có thể tạo assignment;
ngôn ngữ mơ hồ chỉ gợi ý và xin xác nhận, chưa đổi danh tính. Đổi nhãn không tự
chuyển cwd, cấp quyền, nạp luật dự án khác hay thay context. Với dữ liệu cũ, nhãn
do suy đoán phải giữ provenance/unconfirmed, không migration thành “user đã chọn”.

Giao dự án lần đầu hoặc sửa metadata đoán sai không phải quyền chuyển việc sang
dự án khác trong cùng phiên. Giữ luật một phiên/một project/module; muốn làm
dự án khác thì mở phiên đúng phạm vi, không sửa assignment để lách hàng rào.
Chuyển working scope thực của một phiên sống sang dự án khác **ngoài phạm vi bản
thiết kế này**. UI assignment không sửa prefix assistant hoặc chứng minh CLI đã nạp luật.

`source_namespace_ref` định danh namespace nguồn native đã xác minh, không phải
account hiện gắn. Cùng transcript/DB được nhiều account nhìn thấy không tạo nhiều
logical session hoặc usage; giữ attribution quan sát/runtime riêng. Canonical path,
native namespace và provenance giúp đối chiếu alias, không chỉ ghép vì nội dung giống.
Nếu cùng native session mở ở nhiều runtime thì có nhiều binding, không tự chọn một
binding để gửi khi chưa xác minh được đích.

## 5. Luồng thao tác và kiểm nhận

### Mở phiên

`Chung/dự án đã giao → account/model/effort → mode hỗ trợ → luật/MCP/quyền → mở runtime
→ nhận native ID/binding → xác minh readiness → đưa vào panel`.

Đề bài có thể để trống. Pending launch có ID riêng, chưa bịa native session đã sẵn
sàng. Chưa có mode UI-first được đo thì giữ đường terminal gốc.

### Gửi chat / tệp / lựa chọn

`View → command ID + SessionKey + generation + payload → gate → ghi intent bền vững
→ dispatcher đúng phiên → driver → evidence → cập nhật delivery → collector nhận output`.

Các trạng thái khác nhau: đã nhận ở huba, đang chuyển, CLI đã nhận, còn trong composer,
bị từ chối, chưa xác định. Turn processing/ended là trục khác. Không dùng HTTP200,
spawn0 hoặc chữ biến mất khỏi màn hình làm chứng minh CLI đã nhận.

Một command ID retry cùng actor/phạm vi trả trạng thái cũ; cùng ID nhưng payload khác
phải từ chối. Hai tin cùng chữ với ID khác có thể là hai ý định hợp lệ. Mất ACK sau
side effect giữ uncertain, hòa giải theo nguồn, không tự gõ lại.

**P09 khoảng hở đã đọc từ mã:** `pipeline.rs:8214–8227` từ chối nhận đã gửi nếu Codex
thiếu nhật ký, nhưng non-Codex vẫn có đường `DaGui` dựa màn hình. Đây là bất nhất
semantic theo mã; lượt này chưa tái hiện lỗi giao thật. Representation bảng hỏi
`sessions.rs:4244–4288` chưa giữ question revision/tool-call ID, dù đã hỗ trợ nhiều
câu/multi-select. Không nói các tính năng hỏi hiện có là chưa từng làm.

Bản mới giữ đủ question/permission ID, revision, mô tả options, multi/free-text;
revalidate trước trả lời. Attachments tách bytes/ref/path, có manifest và receipt
theo capability; không nhận chỉ có path là model đã nhận bytes.

### Năm thao tác khác nhau

1. Đóng panel/view: chỉ ngừng xem.
2. Ngắt lượt: xin dừng lượt hiện tại, giữ phiên; job còn sống phải báo.
3. Dừng job: tác động đúng task/process group đã xác minh.
4. Đóng CLI session: nêu ảnh hưởng background task và hỏi xác nhận.
5. Tắt dịch vụ: nêu mất kênh điều khiển/quan sát và khả năng CLI tiếp tục theo runtime.

Hiện `/stop` với interactive còn dẫn sang xác nhận đóng CLI+terminal
(`pipeline.rs:15963–15972`). UI mới không được chỉ đổi nhãn thành “Ngắt lượt”
trong khi vẫn gọi thao tác đóng này.

## 6. Nhiều người xem, một đường tác động đúng phiên

Subscription gồm `client/view/subscription IDs + scope + SessionKeys + cursor`.
Core trả snapshot có watermark rồi delta theo revision; nguồn thu một lần, phân phối
cho các view có quyền. Summary cho B khác timeline cho A; raw chỉ đọc khi cần.
Bounded queue/backpressure dùng cấu hình; tràn thì báo gap/resnapshot, không bỏ im.

Command luôn mang target tường minh, không lấy focus toàn cục. Core serialize phần
tác động theo session; không khóa mọi phiên vì một phiên đang làm việc lâu.
Gửi thêm giữa lượt chỉ theo khả năng driver đã đo; nếu không, giữ draft.
Raw terminal input cần **control lease** gắn actor+generation+lease epoch+expiry.
Takeover qua barrier: đóng nhận tác động structured và raw mới, drain/đối soát lệnh
đang bay; nếu còn uncertain thì chưa cấp quyền nhập. Fence writer cũ bằng epoch tại
thời điểm thực thi, kiểm composer/binding rồi mới attach terminal và cho lease nhập.
Trả lại structured mode cũng qua barrier và đối chiếu composer, không phát lại keys.
Không hứa phối hợp đồng thời tay người ở terminal ngoài nếu adapter chưa quan sát được.

**P05 rủi ro phải kiểm:** `pipeline.rs:913–925` còn fallback `focus:session` khi thiếu
target; nguồn mã cũng có thao tác chụp explicit đổi focus. Không bê cơ chế đó vào
desktop nhiều cửa. Chưa khẳng định đã tái hiện race của desktop V2.

## 7. Trạng thái, độ trễ, graph tool/child/job

Tách `session lifecycle`, `turn state`, `delivery`, `tool call`, `child/job`,
`source health`, `control mode`, `assignment`. Idle không đồng nghĩa closed;
tool completed của launch không đồng nghĩa child/job ended; exit1 không thành
thành công chỉ vì metadata gọi completed. CLI succeeded không phải nghiệm thu nghiệp vụ.

**P02:** nguồn lifecycle đánh thức nhánh trạng thái nhẹ, poll định kỳ đối soát.
Đo riêng nguồn→ingest→fanout→UI, Telegram receipt; receipt không chứng minh đã thấy
trên điện thoại. Số poll120s trong thăm dò trước là nhịp vòng, không phải latency
đầu-cuối và không là cam kết nhịp bản mới.

**P03/P08:** preview OpenCode40hàng/6000ký tự không làm timeline đầy đủ.
Đọc revision cập nhật hàng cũ, không chỉ `seq > cursor`. Graph giữ turn/assistant/
request/tool/child/job IDs nguồn; thiếu ánh xạ thì hiện chưa ghép, không suy theo
thời gian gần. Không lấy số row assistant làm số HTTP request.

Freshness lưu riêng `content_changed_at`, `last_source_read_ok`,
`last_inventory_complete`, `last_binding_verified`; mỗi nguồn/child/job có mốc riêng.
Journal im lặng nhưng vẫn đọc/kiểm thành công không tự thành stale/idle/ended.
Inventory nguồn kỳ vọng quyết định phần nào chưa xác minh, không lấy cha fresh làm
chứng minh mọi job con fresh.

UI trace dùng cùng graph cho xem trực tiếp và xem lại; có thể phân kỳ triển khai,
nhưng chưa coi yêu cầu vẽ lại là chọn A/B trước đây hoặc cho phép viết sản phẩm.

## 8. Input/output/context và dữ liệu gốc

Giữ riêng: chữ user nhập; journal/event CLI; model request/response body nếu hook
hỗ trợ; màn hình/PTY. MCP facade là cửa truy cập dữ liệu có quyền, không tự bắt
toàn bộ lưu lượng model. CLI MCP client/server thực thi tool là một đường khác.

CLI chọn luật, history/summary, tools, input, tool results cho mỗi model request.
UI scroll/lọc/xem raw không gọi model và không tự đưa kho history vào context.
Tool result có thể bị CLI trim trước request tiếp; raw giữ theo nguồn, bản trim
có nhãn riêng. Phần provider không xuất thì “nguồn không cung cấp”, không bịa.

Ingest lưu raw/source kind/version/key/generation/hash/completeness và checkpoint
cùng transaction rồi mới nâng cursor. Projection dựng lại được; ingest cursor,
projection checkpoint và view cursor là ba thứ riêng. Snapshot DB có provenance,
không gọi là nguyên byte file SQLite. Record mutable giữ revision/source snapshot.
Parse lỗi vẫn giữ raw+lỗi, không biến thành dữ liệu rỗng hợp lệ.

Raw nhạy cảm theo quyền/policy; không chủ động thêm auth secrets của huba vào prompt,
audit hoặc diagnostics. Chủ máy xem raw theo quyền, không tự phát tán sang kênh khác.
Hash khớp một record không chứng minh đã thu đủ mọi record.

**P07 cần chốt:** mức J journal hay cả P model body. P chưa có bằng chứng thì không
hứa, không MITM hoặc chuyển thuê bao sang API trả phí để tạo khả năng này.

## 9. Các màn quản trị và cấu hình động

Ngoài bàn/panel/trace: Phiên & lịch sử; Tài khoản/model; Kết nối; Dự án/assignment;
Hoạt động/chẩn đoán; Cài đặt/bản cài; Báo cáo token. Bản HTML có bảng chi tiết từng
màn, input/output, empty/error/unknown, contract và ma trận terminal.

Mọi thay đổi vận hành có thể đổi lưu trong dữ liệu: layout/panel limits, retention,
ngưỡng freshness, queue/retry policy, thông báo, catalog/model defaults, lease expiry,
quyền và tích hợp. Core kiểm quyền quản trị, expected revision và audit. Giá trị
an toàn khi chưa cấu hình phải có nhãn; không tự nâng quyền vì thiếu capability.
Bí mật giữ env/secret store, không DOM/localStorage hoặc giá trị trong audit.

Token-only: canonical request usage, semantics/cache/child rollup/counter reset;
không lấy quota% hay context hiện tại làm tổng kỳ. ECharts, bảng và CSV dùng cùng
filter/revision/coverage; unknown khác0, không thêm billing/tracker/model tóm tắt.

## 10. Phục hồi, đóng gói và an toàn

- App đóng vẫn có thể để hubad/CLI chạy theo mode; restart app không restart CLI.
- Reconnect xác minh quyền/binding, đối chiếu command đang treo, rồi snapshot/delta.
  Không replay side effect từ cursor đọc; cursor hết retention phải báo gap.
- Migration có backup/transaction/rollback; không ghi trực tiếp DB CLI để giả event.
- Renderer escape/sanitize nội dung nguồn; không thực thi HTML/JS/lệnh từ model.
- Cùng core/contracts trên ba OS; bundle frontend/native/core đồng version.
  Native installer và update ký, đọc lại marker từ target, có rollback dữ liệu.
- Windows/macOS/Linux cần cài/chạy/đóng/mở/cập nhật thật, không coi cross-build
  hoặc browser tài liệu là nghiệm thu app/native transport.

## 11. Đánh dấu và thứ tự bàn

| Mã | Phân loại | Nội dung |
|---|---|---|
| P01 | Lỗi có ca thật | Path/log làm sai tên phiên |
| P02 | Chưa đo đủ | Wake lifecycle và độ trễ đầu-cuối |
| P03 | Chưa đủ nguồn | Preview/timeline/raw và bản ghi mutable |
| P04 | Cần bàn bố trí | Hai vùng, layout, nhiều cửa/view |
| P05 | Rủi ro | Focus/global target và tác động đồng thời |
| P06 | Cần chốt | Mode UI-first và terminal takeover |
| P07 | Cần chốt | Mức journal/model body bắt buộc |
| P08 | Chưa đo đủ | Child/job live, kết quả và recovery |
| P09 | Khoảng hở contract | Giao nhận, revision câu hỏi, permission/tệp |
| P10 | Chưa triển khai đầy đủ | Usage canonical/báo cáo nguồn thật |
| P11 | Cần spike/chốt | Vỏ, IPC, capabilities và ba OS |
| P12 | Rủi ro | Crash/idempotency/gap/retention/quyền |
| P13 | Cần chốt | Ai được XEM — principal/viewer, luật viewer≠owner |
| P14 | Chưa có nguồn | `host_id` · `generation` · command ID |
| P15 | Chưa có số | Ngưỡng độ trễ · retention · gap |
| P16 | Rủi ro | Raw nguyên văn và nội dung nhạy cảm |
| P17 | Rủi ro | Windows chưa chạy thật mà đã hứa ngang hai OS |
| P18 | Lỗi có ca thật | Số hiển thị đọc từ catalog CLI, không bảng cứng |

HTML gắn từng Pxx vào đúng bước sơ đồ và có bằng chứng, hướng đề xuất, điểm bàn;
đỏ/cam/tím/vàng không được hiểu tất cả là lỗi đã tái hiện.

Thứ tự bàn đề xuất: **P04 bố cục → P06 cách làm việc không cửa sổ riêng → P01
danh tính → P07 mức dữ liệu → P02/P03/P08/P09 đường dữ liệu và control → P11/P12**.
Không cần chốt hết ngay; chưa chọn thì vẫn ghi pending, không tự biến thành đã duyệt.

## 12. Phân kỳ và phép nghiệm thu dự kiến

1. Đo capability từng CLI/OS/version, source/attach/ACK; khóa scope UI-first an toàn.
2. Khóa contract identity/command/event/view; dựng registry/projection không phá route cũ.
3. Một phiên UI thật trọn vòng: mở→gửi→tool→hỏi/quyền→kết quả→ngắt/đóng đúng nghĩa.
4. Hai vùng/nhiều view và chéo Desktop↔Telegram; terminal takeover cùng session.
5. Child/job, raw/trace/history/recovery và token canonical có coverage.
6. Quản trị, bộ cài/update/rollback và kiểm đủ target thật.

Ca âm phải có RED/GREEN: root+câu chung+log ứng dụng không đổi assignment; hai view
đổi lựa chọn không đổi target; stale question/lease/generation bị từ chối; mất ACK
không gửi trùng; cha idle nhưng job chạy vẫn thấy; child exit1 vẫn lỗi; source mất
không báo0/closed; mutable row cập nhật được; retention gap không giả liên tục;
cache/parent rollup không cộng đôi; tắt viewer không tắt CLI; update lỗi rollback được.

Mỗi lượt đo nêu mẫu số/nguồn/phiên bản; UI qua thao tác thật, console0 lỗi, ảnh mọi
trạng thái. Sơ đồ thiết kế không thay dependency map trích mã/traces. **Lượt này
chỉ vẽ/kiểm tài liệu; chưa build/cài hoặc nghiệm thu bất kỳ UC sản phẩm V2 nào.**

## 13. Rà soát 19/11 — sáu lỗ hổng còn mở (P13–P18)

Rà lại toàn bộ §1–§12 trên bản đang có trong cây (`index.html` hash `328d0c35…`,
19 flow/30 thành phần/22 UC/12 issue — khớp mục REVIEW 17/11). Nội dung §1–§12
**không sai chỗ nào đã kiểm**; các trích dẫn `file:line` còn đúng chủ đề. Sáu việc
dưới đây là chỗ *chưa có quyết định hoặc chưa có nguồn*, và chúng đứng TRƯỚC
P04 trong thứ tự triển khai.

**P13 — ai được XEM (nặng nhất).** §6 nói «nhiều người xem, một đường tác động»
nhưng không định nghĩa principal. Trong mã hiện tại **không có** khái niệm này:
cổng duy nhất là `telegram::update_sender` theo chat id (`CLAUDE.md:160–173`);
`sessions::read_only_count` (`sessions.rs:5677`) là đếm PHIÊN, không phải quyền
người. Làm đúng chữ ⇒ desktop thành **cổng thứ hai mở quyền ra lệnh**, đúng thứ đã
xoá một lần (`trust.tfl5_user_tids`, `CLAUDE.md:164–172`). Cần: bảng `{owner,
viewer}` · mọi route GHI chỉ owner · pairing chỉ cấp quyền đọc · audit khi nâng
quyền · thu hồi quyền cắt stream + xử lý cache.

**P14 — ba trường xương sống chưa có nguồn.** `host_id` (§4 SessionKey),
`generation` (§4/§6 lease), command ID (§5): grep `rust/src` ngày 19/11 = **0** cả
ba. Telegram dedupe đang dùng `update_id` + bảng `tin_den`, riêng kênh ấy
(`CLAUDE.md:149–159`). P05/P09/P12 đều đứng trên chúng ⇒ phải chốt: một host (bỏ
`host_id`) hay nhiều host (thì bỏ câu «không quản lý máy xa» ở §2); `generation`
sinh ở driver hay DB; command ID do view sinh, lưu bảng nào, khoá theo
`(session, actor)`.

**P15 — không có ngưỡng nghiệm thu.** P02 đòi đo latency mà không có mục tiêu/trần;
P03/P12 đòi retention/gap mà không có hạn mức; §7 có ba mốc freshness mà không nói
bao lâu thì cũ. Không có số ⇒ không viết được ca đỏ. Cần một bảng: p95
nguồn→ingest→UI · retention raw (ngày · GB) · mất quá N% thì **cấm** gọi «liên
tục» · queue tràn ⇒ gap/resnapshot · hạn freshness theo loại nguồn.

**P16 — raw nguyên văn và nội dung nhạy cảm.** Luật 5 (không giấu chữ) + §8 raw
nguyên văn: bí mật lọt vào transcript (tiền lệ một phiên đọc `.env`, sổ 18/11) sẽ
nằm trong DB raw. §8 hiện chỉ nói «theo quyền/policy». Cần ghi rõ: mã hoá at-rest
hoặc thư mục 0700 + tên tệp không lộ nội dung · retention mặc định + xoá theo yêu
cầu có audit · đánh dấu record khớp mẫu bí mật · ai đọc raw qua MCP facade.

**P17 — Windows.** `keys_win.rs:1–12` tự khai «CHƯA CHẠY TRÊN WINDOWS THẬT», mà §10
hứa «cùng core/contracts trên ba OS» và §12 không có mốc Windows. Ghi rõ parity 3
OS là MỤC TIÊU; mỗi OS một cột nghiệm thu; Windows chỉ được khai «hỗ trợ» sau khi
chạy thật gõ/đọc/đóng/mở/cập nhật.

**P18 — số hiển thị phải đọc từ catalog CLI (ca thật 18/11).** Đầu nhắc chuyển
phiên khai cứng cửa sổ 200k cho `opencode-go/deepseek-v4.1-flash` trong khi catalog
`models-dev` của chính kho là 1.000.000 ⇒ 206k token bị tính 103% ⇒ «99% ≥ 50%,
chuyển phiên» khi ngữ cảnh thật 21%. Mẫu 11 phiên/3 ngày: 6 phiên acc8 (thật
16–22%) báo 80–112%; 5 phiên acc9 (thật 12–35%) báo 19–59%. Đã vá bằng cách đọc
`limit.input`→`limit.context` từ `kv models-dev:v2:catalog` (kho chung `a73e394`).
Luật cho §9: mọi % ngữ cảnh/quota hiển thị **kèm nguồn** + ca âm «cấy catalog sai
⇒ đỏ»; bảng cứng chỉ là dự phòng có hạn.

**Hai chỗ lệch nhãn (nhẹ).** (a) `design.json` để P04 `state: decision` trong khi
evidence của chính nó ghi «hướng bố cục không còn là câu hỏi» — đọc kèm §3, không
đọc thành chưa chọn. (b) Trích dẫn bằng **số dòng** vào `sessions.rs`/`pipeline.rs`/
`keys.rs` — các tệp này vừa nhận commit 18/11 ⇒ về sau nên ghim `commit:line`, nếu
không tài liệu tự mất khả năng kiểm. Ngoài ra dispatcher desktop phải đọc
`commands::ROUTES` (`CLAUDE.md:184–186` — «the real table»), không dựng bảng lệnh
thứ hai.

**Thứ tự triển khai sửa lại:** **P13 → P14 → P15** rồi mới **P04** (vì A/B,
subscription và lease đều đứng trên principal/định danh/ngưỡng), sau đó
P06/P01/P07, rồi P02/P03/P08/P09/P16/P18, cuối cùng P11/P12/P17.

