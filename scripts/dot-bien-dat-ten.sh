#!/bin/bash
# Đối chứng ngược cho phiếu zkey 23/11 (nhãn phiên + tên tệp `.inbox`):
# cấy từng lỗi, bài kiểm PHẢI ĐỎ; trả mã gốc, PHẢI XANH.
set -u
cd "$(dirname "$0")/../rust" || exit 2
F1=src/sessions.rs
F2=src/telegram.rs
cp "$F1" "$F1.goc" || exit 2
cp "$F2" "$F2.goc" || exit 2
sai=0
khoi_phuc() { cp "$F1.goc" "$F1"; cp "$F2.goc" "$F2"; }

cay() { # $1 tên · $2 tệp · $3 chuỗi cũ · $4 chuỗi mới · $5 lệnh kiểm
  khoi_phuc
  python3 -I - "$2" "$3" "$4" <<'PY'
import sys
f, cu, moi = sys.argv[1:4]
s = open(f, encoding="utf-8").read()
if s.count(cu) < 1:
    print(f"KHÔNG CẤY ĐƯỢC: «{cu[:40]}…» không xuất hiện")
    sys.exit(3)
open(f, "w", encoding="utf-8").write(s.replace(cu, moi))
PY
  if [ $? -ne 0 ]; then echo "DB $1 => KHÔNG CẤY ĐƯỢC"; sai=$((sai+1)); return; fi
  if eval "$5" >/dev/null 2>&1; then
    echo "DB $1 => XANH-OAN (lỗi cấy mà test vẫn xanh)"
    sai=$((sai+1))
  else
    echo "DB $1 => ĐỎ (đúng)"
  fi
}

cay dem_ban_ghi_thay_vi_luot_noi "$F1" \
  'let Some(text) = assistant_spoken_text(&record) else {' \
  'let Some(text) = text_of(&record) else {' \
  'cargo test --offline --lib phieu_zkey_20261123::luot_chi_goi_cong_cu_khong_tieu_o_cua_so'

cay bo_bang_chung_tin_nhan_dau "$F1" \
  'if bang_chung.as_deref() == Some(declared.as_str()) {' \
  'if false {' \
  'cargo test --offline --lib phieu_zkey_20261123::loi_khai_don_can_bang_chung_tu_tin_nhan_dau'

cay cho_phep_ghi_de "$F2" \
  '.create_new(true)' \
  '.create(true)' \
  'cargo test --offline --test phieu_zkey_dat_ten'

khoi_phuc
cmp -s "$F1" "$F1.goc" && cmp -s "$F2" "$F2.goc" \
  && echo "mã gốc trả về nguyên vẹn (cmp)" || { echo "⚠ mã gốc KHÔNG khớp bản lưu"; sai=$((sai+1)); }
echo "DOT_BIEN_EXIT=$sai"
exit $sai
