# huba/ui — kế nhiệm sau nhắc 50%

Chỉ làm module UI trong /mnt/data/projects/huba. Chủ máy: thiết kế giao diện/luồng TRƯỚC viết sản phẩm; PC đa nền tảng ít sửa; MCP/input-output gốc; vừa đề nghị chỉ Claude CLI + OpenCode để phủ nhiều model/giảm chi phí nạp.

Đã viết docs/thiet-ke-desktop.md và docs/ui/{index.html,style.css,app.js,export.py,README.md}. Bản mẫu7màn fixture, không nối backend. Đề xuất Tauri+Rust chung, OS/CLI/kênh độc lập; MCP facade không tự thu payload model. Mới thêm mục ưu tiên2CLI; chưa có phép đo đủ để nói phủ hết/rẻ nhất. Không xoá Codex sản phẩm.

Đã gửi thật qua Tele: Markdown1736(27212B), HTML tự chứa1742(63927B). Biên nhận .tmp/gui-thiet-ke-receipt.json + gui-html-thiet-ke-receipt.json. Đừng gửi trùng. Xuất lại HTML: python3 docs/ui/export.py.

Đo: JSsyntax qua cong-node0; Playwright sandbox1 trước page vì socket. CHƯA nghiệm thu UI/ảnh. Hòm thư host review cũ sai cấp nên chưa chạy; đúng là /tmp/claude-1000/-mnt-data-projects/<ID>/scratchpad/huba-run.txt (pipeline.rs7608). Lệnh chạy lại review: python3 /mnt/data/projects/huba/.tmp/ui-review/run-on-host.py; log .tmp/ui-review/host-review.log.

Việc kế: trả lời/gom phản hồi thiết kế; đo nguồn raw/plugin và chi phí so công bằng nếu được triển khai tiếp. Không tự làm app khi chủ máy đang duyệt thiết kế. Config M2836B/mtime14-11 13:41 có trước, không commit. Không chạm công việc account/core của phiên khác. Việc nền: không có; không có job gửi còn treo. Sổ memory/lanes/ui.md.
