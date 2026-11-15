# Phạm vi kiểm bản vẽ — 15/11/2026

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
