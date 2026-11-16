# Sửa tên module trong danh sách phiên — 15/11/2026

## Kết quả đã đo

Phiên OpenCode `ses_e2073a7c9fff1DuRlTY6oE0VOg` trước sửa có nhãn
`[huba]·e2073a7c`, `lane=""`, dù assistant khai `[huba/ui]`.
Sau cập nhật bản cài, CLI và sổ của chính daemon đều ghi **`[huba/ui]`**.
Đây là đối chiếu kỹ thuật chỉ đọc trên dữ liệu thật, **chưa thay nghiệm thu bấm
`/session`/nút phiên trong Telegram**.

## Sửa đúng lớp lỗi

- `rust/src/sessions.rs:5870`: OpenCode đi qua constructor thực dùng bởi snapshot;
  lấy cả dự án/module qua cửa chung, không suy từ tiêu đề, user hoặc tên `ui` cố định.
- `rust/src/sessions.rs:2462` và `session_identity_reducer.rs`: xử lý cặp dự án/module
  theo thứ tự. Giữ module khi chỉ nhắc lại cùng dự án; một lần nhắc dự án khác không
  đổi danh tính. Chuyển dự án được xác nhận phải xóa module cũ, kể cả đổi đi rồi về
  giữa hai lượt chụp.
- `rust/src/sessions.rs:2228`: module từ đuôi/đầu phải thuộc dự án đã được chấp nhận.
- `session_identity_history.rs`: khôi phục lời khai giữa lịch sử sau khi trôi khỏi
  40 hàng preview; dùng cùng reducer khi khởi động lạnh. SQLite chỉ đọc, bind session
  ID, không lấy danh tính từ user/tool. Chỉ chuyển tối đa42 ký tự mở đầu mỗi lượt
  text; JSON hỏng/lỗi đọc có log và note, không coi là lịch sử rỗng hợp lệ.
- Cache theo account-profile/session/workspace/revision nguồn, gồm cả kết quả không
  có module; lỗi không cache để còn thử lại. Unicode whitespace có kiểm parity.

## Bằng chứng kiểm

| Chiều kiểm | Trạng thái | Bằng chứng |
|---|---|---|
| Tái hiện | PASS | `.tmp/ui-identity-red-expanded.log`: 7 đỏ/10; `.tmp/ui-identity-red-review.log`: thêm2 lỗi warm/cold và chuyển dự án |
| Regression mới | PASS | `.tmp/ui-identity-green-review.log`:21/21, mã0; SQLite → constructor thật → renderer, đối chứng user/tool, memo không có nguồn lịch sử cứu hộ, cold có census dự án khác |
| Kiểm dùng chung | PASS theo các bài qua | 7 binary:178 qua,1 ignored,1 đỏ nền tảng; `.tmp/ui-identity-regression.log` |
| Bài đỏ còn lại | FAIL | `telegram.rs:1080–1086` gọi `sips` không có trên Ubuntu; binary trước sửa cũng đỏ101 trong `.tmp/ui-identity-existing-sips-failure.log`. Hai tệp Telegram không đổi so HEAD |
| Fmt/clippy/build | PASS | `.tmp/ui-identity-final-batch.log`:FMT0/CLIPPY0 với `-D warnings`; `.tmp/ui-identity-release.log`:RELEASE0 |
| QG toàn kho | Chưa xanh | Không nhận batch mã1 ở bài `sips` là xanh; không bỏ/skip bài đỏ để đổi verdict. Không có tuyên bố full repository gate xanh |
| Nguồn thật | PASS thăm dò | `.tmp/ui-identity-history-probe.log`:145 hàng nguồn,12 lượt text,7.349ms, tối đa42 ký tự/lượt; session không tồn tại trả rỗng. Không phải số đo hiệu năng tổng quát |
| Bản cài | PASS kỹ thuật | `.tmp/cai-ui-module.log`:GATE0,3/3 binary khớp, `/proc` daemon khớp release, PID510547→528544, giữ12/12 pane, hai config không đổi |
| Daemon thật | PASS thăm dò | `.tmp/ui-module-daemon-book-probe.json`:nhãn `[huba/ui]`; đọc `Mark.l` (`watch.rs:159`), không nhầm `d` là nhãn |
| Telegram/UI thật | CHƯA ĐO | Chưa bấm route/nút và nhìn phản hồi trong chat thật; không dùng đọc DB/CLI thay nghiệm thu này |
| Security/contract/data | PASS trên diff | Không endpoint/credential mới, SQL bind+read-only; không đổi schema/shape `LiveSession`. Cài thay nguyên tử+fsync, có backup/rollback, không đổi unit/plugin/cấu hình |

## Bản cài và giới hạn

Chủ máy đã duyệt ngoại lệ đúng3 binary và restart riêng daemon. Không đóng TUI,
không sửa mã đang dở của phiên khác. HEAD nền lúc dựng là `f366361` của phiên lõi;
thay đổi nhận diện tại mốc cài23:15 **chưa commit/push**.

- CLI SHA256 `d2d252d698f20af57faa841e68dcf507e69b519684c08979aaa55f68d250287f`.
- Daemon SHA256 `a99992a2294deea5bec295c7ebe539f96a159ff78b34c46385123b913a9ab8ec`.
- Manifest: `.tmp/ui-module-install-manifest.json` (hash4 tệp nguồn và binary).
- Backup: `.tmp/ui-module-backup-20261115-231548/`.
- Nền của phiên: không có job còn chạy.
- Đã gửi thông báo để chủ máy đối chiếu `/session`: Telegram2332, mã0 trong
  `.tmp/ui-module-telegram-notice.log`. Helper gửi dùng hub_home bản cài thay đường
  `~/projects/hub` cũ trong script chung; không sửa script ngoài repo. Giao tin
  thành công không có nghĩa thao tác UI đã nghiệm thu.

**Chưa nghiệm thu toàn bộ:** còn bấm `/session`/chọn phiên trong Telegram, bài `sips`
trên Linux và QG toàn kho. Thiết kế desktop/A-B/raw vẫn chờ duyệt,
không triển khai app/collector trong lượt sửa này.

## Checkpoint theo lệnh Hà — 23:43 +07

Kiểm lại đúng nguồn hiện tại: `.tmp/ui-identity-precommit.log` **21/21 mã0**, gồm
13 bài `sessions::opencode_identity_tests` và8 bài history. Ba bài đỏ trong bản
trước là bằng chứng RED cũ, không phải kết quả của bản nguồn hiện tại. Hash4 tệp
Rust vẫn khớp manifest bản đã cài; config3158B/mtime11:34/hash267b436b… giữ nguyên.

Đã khởi QG với `QG_LY_DO=gom`, log `.tmp/qg-ui-precommit-20261115.log`; tại lúc ghi
khối này chưa có verdict, không khai xanh. Checkpoint chỉ9 tệp UI/Rust được giao,
không chứa cấu hình. Push non-force qua hòm thư huba và xác nhận remote sau đó;
log dự kiến `.tmp/push-ui-module-20261115.log`, sổ sống ghi kết quả khi có readback.
