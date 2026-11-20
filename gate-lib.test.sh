#!/usr/bin/env bash
# Bài kiểm `gate-lib.sh` — ĐỐI CHỨNG NGƯỢC hai chiều cho nhánh LINUX native của gate.sh ②c.
#   bash gate-lib.test.sh      ⟹ in `GATE_LIB_TEST_EXIT=<số ca hỏng>`
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=gate-lib.sh
source "$HERE/gate-lib.sh"
LINUX=x86_64-unknown-linux-gnu
hong=0
kiem() { # <tên> <mong: 0|1> <mã thật>
  if [ "$2" -eq "$3" ]; then echo "ok   $1"; else echo "HỎNG $1 (mong $2, được $3)"; hong=$((hong + 1)); fi
}

gate_dich_la_host "$LINUX" "$LINUX";               kiem "host == đích ⟹ native"            0 $?
gate_dich_la_host "aarch64-apple-darwin" "$LINUX"; kiem "host khác đích ⟹ KHÔNG native"    1 $?
gate_dich_la_host "" "$LINUX";                     kiem "host đọc hỏng (rỗng) ⟹ KHÔNG"     1 $?
gate_dich_la_host "" "";                           kiem "rỗng == rỗng vẫn KHÔNG native"    1 $?

# Ép host bằng biến: giả ⟹ không khớp; thật ⟹ khớp.
GATE_HOST_TRIPLE="aarch64-apple-darwin" gate_dich_la_host "$(GATE_HOST_TRIPLE=aarch64-apple-darwin gate_host_triple)" "$LINUX"
kiem "GATE_HOST_TRIPLE giả ⟹ không khớp" 1 $?
gate_dich_la_host "$(GATE_HOST_TRIPLE=$LINUX gate_host_triple)" "$LINUX"
kiem "GATE_HOST_TRIPLE thật ⟹ khớp" 0 $?

# Không ép: phải bằng đúng `rustc -vV` (có rustc thì mới đo được).
if command -v rustc >/dev/null; then
  that=$(rustc -vV | sed -n 's/^host: //p')
  [ "$(env -u GATE_HOST_TRIPLE bash -c "source '$HERE/gate-lib.sh'; gate_host_triple")" = "$that" ]
  kiem "không ép ⟹ đúng host của rustc ($that)" 0 $?
fi
echo "GATE_LIB_TEST_EXIT=$hong"
exit "$hong"
