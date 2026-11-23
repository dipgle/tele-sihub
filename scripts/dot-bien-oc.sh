#!/bin/bash
# Đối chứng ngược cho nhận diện hộp câu hỏi opencode (23/11): cấy từng lỗi vào rust/src/keys.rs, test opencode_hop_chon
# PHẢI ĐỎ; trả mã gốc, PHẢI XANH. Thoát 0 = mọi lỗi cấy đều bị bắt.
set -u
cd "$(dirname "$0")/../rust" || exit 2
F=src/keys.rs
TESTS="--test opencode_hop_chon --test opencode_o_nhap_dan"
mkdir -p ../.tmp || exit 2
cp "$F" "$F.goc" || exit 2   # bản lưu; cuối kịch bản trả lại và `cmp`. Chạy dở ⟹ khôi phục tay từ $F.goc.
sai=0

cay() { # $1 = tên, $2 = chuỗi cũ (phải xuất hiện đúng 1 lần), $3 = chuỗi mới
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
  cargo test --offline $TESTS --no-fail-fast > "../.tmp/dot-bien-oc-$1.log" 2>&1
  if [ $? -ne 0 ]; then echo "DB $1 => ĐỎ (đúng)  · $(grep -m1 -E '^test .*FAILED' ../.tmp/dot-bien-oc-$1.log | cut -c1-100)"; else echo "DB $1 => XANH-OAN (lỗi cấy mà test vẫn xanh)"; sai=$((sai+1)); fi
}

cay khong_nhan_chan_opencode '                || la_chan_opencode(line)
' ''
cay khong_bo_le '    t.strip_prefix('"'"'┃'"'"').map(str::trim_start).unwrap_or(t)' '    t'
cay bo_cua_bat_dau_bang_phim '    if !(t.starts_with("↑↓") || t.starts_with('"'"'⇆'"'"')) {
        return false;
    }
' ''
cay chi_nhan_mui_ten_doc '(t.starts_with("↑↓") || t.starts_with('"'"'⇆'"'"'))' 't.starts_with("↑↓")'
cay bo_nua_enter 'l.contains("esc dismiss") && (l.contains("enter submit") || l.contains("enter confirm"))' 'l.contains("esc dismiss")'
# ── ô nhập opencode giữ khối dán (opencode_o_nhap_dan) ──
cay khong_dung_o_nhap_opencode 'let screen = match opencode_input_text(screen) {' 'let screen = match None::<String> {'
cay bo_nhan_dan_opencode '(seen.contains("[Pastedtext") || seen.contains("[Pasted~"))' 'seen.contains("[Pastedtext")'
cay nhan_dan_khong_can_nhieu_dong 'if typed.contains('"'"'\n'"'"') && (seen.contains("[Pastedtext")' 'if (seen.contains("[Pastedtext")'
# Ô nhập = khối giữa hai dòng đệm `┃` trống (tin vừa gửi sát ô nhập — 23/11 16:18):
cay nuot_ca_dong_trong_vao_o_nhap '            Some(t) if !t.is_empty() => trong.push(t),' '            Some(t) => trong.push(t),'
cay khong_bo_dem_duoi '    if sau_le(j) == Some("") {' '    if false {'
cay bo_kiem_dong_agent_model '    if !model.starts_with('"'"'┃'"'"') || !model.contains(" · ") {
        return None;
    }
' ''

cp "$F.goc" "$F"
cargo test --offline $TESTS --no-fail-fast > ../.tmp/dot-bien-oc-goc.log 2>&1
if [ $? -eq 0 ]; then echo "GOC => XANH (đúng)"; else echo "GOC => ĐỎ-OAN"; sai=$((sai+1)); fi
cmp -s "$F" "$F.goc" && echo "mã gốc đã trả về nguyên vẹn (cmp)" || { echo "⚠ mã gốc KHÔNG khớp bản lưu"; sai=$((sai+1)); }
echo "DOT_BIEN_EXIT=$sai"
exit $sai
