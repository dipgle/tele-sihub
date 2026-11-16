# Mac 15/11 — đọc gộp tab, chưa cài/nghiệm thu Terminal

Phiên acc10 kế nhiệm `45dac16e`; gốc mã `994791a`.

- `rust/src/keys.rs`: `window_script`, `window_any_script`, `tabs_script` lấy danh
  sách thuộc tính theo cửa sổ, vòng tab chỉ đọc danh sách cục bộ. Giữ `try` theo
  cửa sổ để inspector không làm mất toàn bộ kết quả. Chưa đo số event/thời gian
  trên Terminal thật. Ký tự phân cột dùng `character id 9`, không cần OSAX.
- Giữ ưu tiên busy → alive → dead (dead chỉ đường đóng), tty tái dùng, cửa sổ
  ẩn, nhan đề nhiều dòng/thiếu, khung màn theo số dòng, danh sách tiến trình và
  `#skipped`. Hai unit chạy chính script sinh ra với dữ liệu mô phỏng, không
  gọi Terminal. Hai kiểm nguồn có đối chứng ngược ở `rust/tests/doc_gop_tab.rs`.
- RED trước: kiểm đọc gộp trên mã cũ thoát 101 (1 đỏ/2). GREEN sau: 229/229
  trong 7 binary, chạy bản chép ngoài deps; fmt/clippy `-D warnings`/release
  build mã 0. Log cục bộ: `.tmp/ke-nhiem-test-*.log`, bộ chạy
  `.tmp/chay-kiem-ke-nhiem.py`. Không chạy full gate/QG, không gom/đẩy.
- Sandbox từ chối `ps`; AppleScript tới Terminal trả `-1728`, XPC connection
  invalid. Vì vậy chưa đo tab thật, chưa đóng phiên cũ, chưa cài bản sửa.
  `install_update.sh --verify` trả 1: build `d99dc52e…`, bản cài `4a44a3e7…`.

Bàn giao được đối chiếu thêm bằng log thật: `logs/huba.log:115796` chọn
acc6 → acc10 (31%); `:116717` nhận `/accounts`, `:116734` ack gửi thành công,
`:116736` nội dung 4.397 ký tự. Không kết luận đã nhìn tin Telegram.
Việc iOS 3550/3551/3555 bị bỏ trước sửa nhận diện; đã chuyển tình trạng tới
inbox tfl5 (tin 3). Chưa có kết quả chạy lại.
