#!/usr/bin/env bash
# Hàm thuần của gate.sh — tách ra để bài kiểm `gate-lib.test.sh` gọi được mà không
# phải chạy cả cổng. Chỉ `source`, không chạy trực tiếp.

# Triple của MÁY ĐANG CHẠY. `GATE_HOST_TRIPLE` ép giá trị (dùng cho ĐỐI CHỨNG
# NGƯỢC: host giả ⟹ nhánh native không được bật). Đọc hỏng ⟹ chuỗi rỗng.
gate_host_triple() {
  if [ -n "${GATE_HOST_TRIPLE:-}" ]; then
    printf '%s' "$GATE_HOST_TRIPLE"
  else
    rustc -vV 2>/dev/null | sed -n 's/^host: //p'
  fi
}

# `gate_dich_la_host <host> <đích>` — mã 0 khi đích chính là máy đang chạy.
# Host rỗng (đọc hỏng) KHÔNG bao giờ khớp: không đo được ≠ khớp.
gate_dich_la_host() {
  [ -n "$1" ] && [ "$1" = "$2" ]
}
