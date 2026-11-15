# huba — VIỆC CHO PHIÊN UBUNTU: cập nhật lên `7bf441f`+ (Mac, 15/11 ~08:30 +07)

Quang 15/11: «commit ghi chú luôn thì tôi đỡ phải nhắc» · «trên ubuntu cũng model ai làm». ⟹ Việc
này ĐƯỢC GIAO cho phiên huba trên Ubuntu, làm ngay khi mở, không chờ ai nhắc.

🔴 Kéo kho chung KHÔNG phải «ghi ngoài cây»: là một phần việc Quang giao (brief 0019 dừng ở đây —
«phụ thuộc script chung thuộc phiên hạ tầng» ⟹ 2 bài đỏ treo). Chỉ `pull --ff-only`, không sửa tệp.

Làm ĐỦ, theo thứ tự, mỗi bước đọc mã thoát:
1. `git -C <huba> pull --ff-only` và `git -C /mnt/data/projects pull --ff-only` (kho chung phải
   ≥ `4997f88`: có `scripts/mo-dau-phien.py`). ff hỏng ⟹ dừng, ghi lý do vào TODO.md, báo.
2. `cd rust && cargo test --offline --test tuong_codex --test codex_duoc_chon` ⟹ phải XANH.
   (Trước đây 2 bài đỏ chỉ vì thiếu `mo-dau-phien.py` — gốc: kho chung chưa kéo.)
3. Cài lại hubd: `bash scripts/dong-goi-linux.sh` → giải nén gói `.tmp/huba-linux-<sha>.tar.gz`
   → `bash huba-linux-<sha>/install_update_linux.sh` → `bash ~/huba/install_update_linux.sh --verify`.
   Lần ĐẦU gói/trình cài Linux mới chạy thật (Mac chỉ `bash -n`) — lỗi thì log đầy đủ vào TODO.md.
4. Đo sau cài: `huba handover --dry-run <id phiên bất kỳ>` phải chọn được acc10 khi acc Claude khoá;
   mở `/new -a acc10` thấy trên màn dòng «TÀI KHOẢN PHIÊN: acc10 · codex» + «MCP …: ✓».
5. Ghi kết quả (số đo + mã thoát) vào TODO.md mục «Ubuntu: cập nhật lên bc3b516», commit, đẩy.

Mới trong bản: acc codex được CHỌN bằng hạn mức thật (trước bị ép «khoá» ở `quota::rank_all`),
bàn giao sang codex, `/accounts` dài chia nhiều tin, đoạn mở đầu phiên bắt buộc (luật + MCP),
gói Mac, trình cài kiểm hệ điều hành. Chi tiết: `docs/kiem-tra-them-tai-khoan-va-nguon-model.md`.
Từ bản `7bf441f`: bước 3 bật `core.hooksPath=.githooks` ⟹ các lần `git pull` SAU tự dựng gói + cài ở nền
(`.tmp/tu-cai-*.log`, Quang «tự cài luôn đi»). Codex trên Mac thấy phiên qua `lsof`; Linux vẫn `/proc`.
Việc nền: không có.
