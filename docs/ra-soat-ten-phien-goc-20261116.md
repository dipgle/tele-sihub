# Rà tên phiên mở ở gốc workspace — 16/11/2026

## Kết luận điều tra

Huba đang nhầm **đường dẫn được nhắc tới** với **dự án của phiên**. Đây là lỗi
quy tắc suy danh tính, không chỉ lỗi hiển thị. Lượt này chỉ điều tra/ghi sổ;
chưa sửa Rust, chưa build/cài/restart, chưa nghiệm thu UI.

## Ca thật đã đối chiếu

Phiên acc9 `ses_e1dcda77affe1La7beMZARYWTb`, tiêu đề “Kiểm tra máy bị tắt đột
ngột”, có directory và project.worktree đều `/mnt/data/projects` (SQLite đọc
`mode=ro`). User seq4 hỏi kiểm tra máy, không giao dự án; các text assistant
seq14/49/84 dùng `[projects]`. Seq25 có kết quả công cụ chứa đường dẫn zkey.

Nhật ký daemon `/home/user1/huba/logs/huba.log`:
- dòng72633: mở phiên với `project:""`, cwd gốc, 02:31:53Z.
- dòng72701: phản hồi `/session` hiện `[projects]`, 02:32:22Z.
- dòng72820: phản hồi `/session` đã hiện `[zkey]`, 02:33:29Z, nhưng preview vẫn
  `[projects]` và nội dung kiểm tra máy.
- dòng72853: thông báo dừng cũng gắn `[zkey]`, 02:34:57Z.

Thăm dò CLI bản cài lúc khoảng09:37+07, exit0, mẫu số5 phiên: chính ID đó trả
`folder:"scripts"`, `lane:""`, `label:"[scripts]"`, cwd vẫn gốc. Đây là thăm dò
chỉ đọc, KHÔNG thay nghiệm thu bấm Telegram. Nhãn đã trôi theo dữ liệu được đọc.

## Đường gây lỗi trong mã

1. `rust/src/opencode.rs:274–276,325–329`: ghép dữ liệu thô của mọi loại bản ghi
   vào preview40 hàng, tối đa6000 ký tự/hàng, trước khi lọc assistant text.
2. `rust/src/sessions.rs:2104–2159`: quét chuỗi đường dẫn toàn bộ preview,
   ứng viên ≥2 lần, chọn số lần lớn nhất; không phân biệt nội dung log/công cụ
   với hành động chọn dự án. Không yêu cầu ứng viên có marker dự án.
3. `rust/src/sessions.rs:2418–2449`: `[projects]` chỉ được chấp nhận nếu giải ra
   thư mục con thật; đây không phải một giá trị riêng chỉ “đang ở gốc”.
4. `rust/src/sessions.rs:2508–2529,5921–5928`: thiếu lời khai hợp lệ thì nhận
   kết quả đếm; fallback tên `projects` không ngăn bộ đếm gán zkey/scripts.
5. `rust/src/sessions.rs:2468–2476`: một tiền tố assistant cũng đủ gieo tên lần
   đầu; chống nhắc một lần chỉ bảo vệ lúc đổi tên đã khai, không xác thực việc
   người dùng đã giao dự án.

Test hiện hữu còn khóa hành vi đếm đường dẫn khi không khai:
`rust/tests/session_label_declared.rs:81–105`. Test root→`projects` ở
`rust/src/sessions.rs:10898–10918` chỉ kiểm fallback, không kiểm ca nhiễm đường
dẫn. Lượt này không chạy lại test; không báo test xanh.

## Hướng sửa đề xuất, chưa triển khai

- Chưa chọn dự án thì hiển thị nhãn trung tính, kèm ID để phân biệt phiên.
- Tách scope cwd, chủ đề hội thoại và danh tính được giao; đường dẫn trong log,
  tài liệu, kết quả công cụ không tự xác lập/chuyển danh tính.
- Có nguồn giao/chọn dự án rõ ràng thì mới nâng thành nhãn dự án/module;
  assistant tự viết một prefix không phải bằng chứng độc lập về ý định user.
- Giữ cùng contract cho Claude/OpenCode/Codex và khai nguồn của nhãn.
- Regression cần: root+câu chung; chỉ log chứa nhiều đường dẫn ứng dụng;
  prefix đầu đoán sai; giao dự án thật; nhắc dự án khác; warm/cold/restart.

Không tự triển khai trong lượt rà soát. HEAD địa phương9d97da5, origin/main có
5 commit mới theo refs đã đọc; cần đồng bộ/kiểm xung đột trước sửa. Cấu hình
3158B mtime15/11 11:34 giữ nguyên. Không có job nền của lượt này.

## Kết quả sửa — 20/11/2026 (`c1bb2ad`)

Số dòng ở §"Đường gây lỗi" là của 16/11, đã cũ sau P21. Tra lại ngày 20/11:
`sessions.rs` `folder_for_session` · `folder_from_tail` · `declared_in_tail_counted`; `session_identity_reducer.rs`.

**Đã đổi**
- `folder_for_session(id, tail, ws, scope)` — không còn đếm đường dẫn; `folder_from_tail` hết người gọi
  (giữ làm phép đo chủ đề, có cảnh báo trong doc).
- **scope** = dự án của `cwd` (`scope_label`, đo trên đĩa, mở ngăn kéo bằng `looks_like_project`).
- **danh tính** = chỉ lời tự khai. Một tiền tố assistant đơn lẻ KHÔNG gieo tên; cần 2 lượt nhất quán
  trong cửa sổ 12 lượt, hoặc 1 lượt khớp scope. Lật tên vẫn cần 2 lượt (luật 16/09 giữ).
- Cold (lịch sử OpenCode) và warm (đuôi) dùng cùng `IdentityReducer`; bằng chứng cold đã qua chính sách
  được ghi thẳng vào sổ nhớ (lần đầu đưa lại qua `folder_for_session` thì bị cổng 2-lượt từ chối — bắt được
  bởi `warm_and_cold_accept_a_repeated_project_transition`).
- Từ chối nào cũng có log: `folder_label_seed_refused`, `folder_label_flip_refused`.

**Đo**: RED `.tmp/red-nhan-20261120.log` (2/5 đỏ, `Some("zkey")`); gate `.tmp/gate-nhan-20261120.log`.
Test mới `rust/tests/nhan_phien_khong_suy_tu_duong_dan.rs` (11) · `sessions_opencode_identity_tests.rs` (+3) ·
`session_identity_history.rs` (+2). Đảo (không xoá): `with_no_declaration_the_census_does_not_name_the_project`,
`phien_chua_co_nhan_thi_mot_lan_khai_chua_du_de_gieo`, ca `dem` trong `duoi_v2_cho_nhan_du_an_nhu_phien_claude`.

**Cái giá, nói thẳng**: phiên mở ở GỐC mà chưa tự khai 2 lượt (vd lượt cuối chỉ `[dùng Bash]`) nay hiện tên
trung tính (`projects-NN` = tên phiên + id), trước đây hiện `[dự-án]` do đếm. Đúng yêu cầu «chưa giao ⇒ trung tính».
**Chưa đo**: bấm `/session` trong Telegram thật; ca ngăn kéo nhiều bậc ngoài `AI/tcc/amm` dựng tạm.
**Nợ nhỏ**: `opencode.rs:274–329` vẫn ghép dữ liệu thô vào preview — vô hại cho nhãn nay, nhưng là nguồn nhiễm nếu ai
đếm lại; `remember_folder_ranked(..., declared=false)` chỉ còn ghi scope.
