# Luồng chi tiết huba desktop

- `index.html`: bản vẽ SVG tự chứa, mở offline; đọc trên desktop, màn hẹp kéo ngang sơ đồ. Không gọi model/mạng.
- `index.md`: cùng nội dung ở Markdown + Mermaid, để review/version control.
- `design.json`: nguồn nội dung ba lớp **UC / dữ liệu / context–input–output**. Các hợp đồng mới là đề xuất, không phải tính năng đã chạy.
- `build.py`: sinh hai bản từ cùng nguồn: `python3 docs/flows/build.py`.
- `test_build.py`: kiểm cấu trúc, liên kết UC, SVG, khả năng tái sinh và đối chứng dữ liệu hỏng: `python3 -m unittest discover -s docs/flows -p 'test_*.py'`.
- `review.cjs`: kiểm tài liệu bằng Playwright đã cài trong môi trường, chạy qua cổng Node: `bash ~/projects/scripts/cong-node chay e2e -- node docs/flows/review.cjs`. Module `playwright` phải resolve được (có thể khai `NODE_PATH` trỏ kho gói đã cài); mặc định dùng Chromium của Playwright, hoặc khai `HUBA_CHROMIUM_EXECUTABLE`. Không cài gói npm riêng trong cây tài liệu.

Tài liệu cha: `../thiet-ke-desktop.md`. Chưa triển khai collector, kho raw hay Tauri. Không sửa mẫu tám màn hoặc bản HTML đã gửi1811 trong lượt vẽ luồng.

Theo dõi vòng model/tool từ một yêu cầu: `request-trace.md` — có đối chiếu metadata OpenCode thật, giới hạn nguồn và hai phương án A/B đang chờ chủ máy chọn; chưa phải tính năng đã triển khai.
