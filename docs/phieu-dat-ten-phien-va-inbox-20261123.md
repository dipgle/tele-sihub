# Phiếu 23/11 — đặt tên phiên + tên tệp trong `.inbox` (chép từ `zkey`)

Nguồn: `~/projects/zkey/.tmp/phieu-huba-dat-ten-20261123.md` (phiên `zkey` đo, Hà chuyển
việc sang phiên `huba`). Chép vào đây để giữ lâu; **chuỗi đo gốc giữ nguyên** trong tệp nguồn.

## Đo được (nguyên văn phiếu nguồn)

- **Nhãn phiên**: log `folder_label_seed_refused` — `dong_y=1`, `quet_luot=12`, `vua_thay=zkey`,
  `scope=null`. `sessions.rs:2295` `KHAI_QUET_LUOT=12` đếm **bản ghi**, `sessions.rs:2336`
  đếm cả bản ghi chỉ-gọi-công-cụ (qua `text_of` → `[dùng X]`). Phiên opencode ghi 10–20
  bản ghi/lượt ⟹ cửa sổ 12 chỉ chứa 1 lời khai ⟹ từ chối ⟹ rơi về `opencode_project_label`
  → `[projects]·dfef9718`.
- **Tệp `.inbox`**: `telegram.rs:4180` đặt tên `anh-<file_unique_id>.jpg` (không ngày, không
  phiên, không dự án); `telegram.rs:1768` cắt id ở dấu `-` nên id opencode ra nguyên 30 ký tự;
  `std::fs::write` ghi đè im lặng khi trùng tên.
- **Đo lại ở phiên huba (23/11 16:40, kho opencode acc8)**: phiên `ses_dfef9718fffIFowlf8bxwsx4V`
  — **12 bản ghi assistant gần nhất có 0 khối `text`** (toàn `reasoning` + `tool`);
  tin nhắn đầu của người dùng = `cd /mnt/data/projects/zkey && đọc CLAUDE.md …` ⟹ bằng chứng
  độc lập cho đề xuất A3.

## Đề xuất (mỗi mục phải ĐỎ được)

**A. Nhãn phiên**
1. Thứ tự nguồn: tự khai + bằng chứng prompt → `project_dir` → **tiêu đề phiên** → `[projects]·id8`.
2. Cửa sổ đếm theo **LƯỢT NÓI**, không theo bản ghi.
3. Nhận **một** lời khai khi tin nhắn ĐẦU của người dùng `cd <workspace>/<dự án> &&` và thư mục có thật.
4. CLI (`huba sessions`) và Telegram dùng **cùng một chuỗi**.

**B. Tên tệp `.inbox`**
```
20261123-0845_zkey_dfef9718_anh-AQADnxlrG.jpg
20261123-0845_zkey_dfef9718_anh-AQADnxlrG.jpg.json   ← {msg_id, gui_luc, nguoi_gui, caption, bytes, sha256}
```
- `create_new`, trùng tên thì `-2`, `-3`, ghi log `inbox_name_collision`.
- Thư mục luôn `short_id(session)`; thêm `index.tsv` mỗi thư mục (giờ · tên · caption 80 ký tự).

## Trạng thái thực hiện (phiên huba, 23/11 — nhánh `feat/dat-ten` @ `604e048`)

- **A1–A4 + B: đã viết + đo.**
  - `assistant_spoken_text` — cửa sổ 12 đếm LƯỢT NÓI (bản ghi chỉ-gọi-công-cụ không tiêu ô).
  - `prompt_project` / `folder_for_session_ev` — lời khai đơn + `cd … &&` ở tin nhắn đầu;
    hai bằng chứng nói hai dự án khác nhau ⟹ trung tính + `folder_label_seed_ambiguous`.
  - `base_label` — gốc workspace không còn `[projects]·id8`, nhãn = tiêu đề phiên.
  - `huba sessions` in cùng chuỗi `label` với Telegram.
  - `.inbox`: `inbox_file_name` + `ghi_khong_de` (`create_new`, `-2`/`-3`) + sidecar `.json`
    (sha256 tự viết, khoá bằng véc-tơ FIPS) + `index.tsv`.
- **Bài kiểm**: 4/4 unit (`mod phieu_zkey_20261123`) + 3/3 tích hợp (`tests/phieu_zkey_dat_ten.rs`).
- **Đối chứng ngược**: `scripts/dot-bien-dat-ten.sh` — 3/3 ca cấy ĐỎ, mã gốc trả nguyên vẹn.
- **fmt + clippy `-D warnings`** (all targets): sạch.
- **Chạy thật** `huba sessions` trên 13 phiên: phiên `ses_dfef9718…` ra **`[zkey]`**
  (trước: `[projects]·dfef9718`); phiên huba này ra `[huba]·df863a01` nhờ chính đường
  bằng chứng `cd …/huba &&`.
- **Còn**: gate đầy đủ + cài + nghiệm thu bằng mắt trên Telegram (xem `memory/active-context.md`).

