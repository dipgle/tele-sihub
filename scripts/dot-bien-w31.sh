#!/bin/bash
# Đối chứng ngược W3.1: cấy từng lỗi vào rust/src/localserver.rs, test open_window_linux PHẢI ĐỎ; trả mã gốc, PHẢI XANH.
# Kết quả: dòng "DB<n> <tên> => ĐỎ|XANH-OAN" và "GOC => XANH|ĐỎ-OAN"; cuối cùng DOT_BIEN_EXIT (0 = mọi lỗi đều bị bắt).
set -u
cd "$(dirname "$0")/../rust" || exit 2
F=src/localserver.rs
mkdir -p ../.tmp || exit 2
cp "$F" "$F.goc" || exit 2   # bản lưu; cuối kịch bản trả lại và `cmp`. Chạy dở ⟹ khôi phục tay từ $F.goc rồi xoá nó.
sai=0

cay() { # $1 = tên, $2 = chuỗi cũ, $3 = chuỗi mới
  cp "$F.goc" "$F"
  python3 -I - "$F" "$2" "$3" <<'PY'
import sys
f, cu, moi = sys.argv[1:4]
s = open(f, encoding="utf-8").read()
if s.count(cu) != 1:
    print(f"KHÔNG CẤY ĐƯỢC: «{cu}» xuất hiện {s.count(cu)} lần (cần đúng 1)")
    sys.exit(3)
open(f, "w", encoding="utf-8").write(s.replace(cu, moi))
PY
  if [ $? -ne 0 ]; then echo "DB $1 => KHÔNG CẤY ĐƯỢC"; sai=$((sai+1)); return; fi
  cargo test --offline --test open_window_linux > "../.tmp/dot-bien-w31-$1.log" 2>&1
  if [ $? -ne 0 ]; then echo "DB $1 => ĐỎ (đúng)  · $(grep -m1 -E 'panicked|assertion' ../.tmp/dot-bien-w31-$1.log | cut -c1-110)"; else echo "DB $1 => XANH-OAN (lỗi cấy mà test vẫn xanh)"; sai=$((sai+1)); fi
}

cay chan_cho_trinh_duyet 'logging::info(&opened, json!({ "as": "app", "browser": ten }));' 'let _ = child.wait(); logging::info(&opened, json!({ "as": "app", "browser": ten }));'
cay bo_kiem_quyen_chay '.map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)' '.map(|m| m.is_file())'
cay dao_thu_tu_uu_tien 'for name in LINUX_APP_BROWSERS {' 'for name in LINUX_APP_BROWSERS.iter().rev() {'
cay quen_co_app '.arg(format!("--app={url}"))' '.arg(url)'
cay chay_chuong_trinh_canh_cwd '        .filter(|d| !d.as_os_str().is_empty())
' ''

cp "$F.goc" "$F"
cargo test --offline --test open_window_linux > ../.tmp/dot-bien-w31-goc.log 2>&1
if [ $? -eq 0 ]; then echo "GOC => XANH (đúng)"; else echo "GOC => ĐỎ-OAN"; sai=$((sai+1)); fi
cmp -s "$F" "$F.goc" && echo "mã gốc đã trả về nguyên vẹn (cmp)" || { echo "⚠ mã gốc KHÔNG khớp bản lưu"; sai=$((sai+1)); }
echo "DOT_BIEN_EXIT=$sai"
exit $sai
