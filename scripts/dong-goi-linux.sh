#!/bin/bash
# Đóng gói bản Linux (Ubuntu/tmux): dựng (x86_64-unknown-linux-gnu, zig làm linker — crate `ring` cần bộ
# biên dịch C cho Linux mà máy Mac không có) → chép → tar.gz → in sha256. Gói ra ở .tmp/huba-linux-<sha>.tar.gz.
#
#   bash scripts/dong-goi-linux.sh
#
# Thiếu target/zig thì DỪNG và nói thiếu gì (mã 2), không dựng nửa chừng.
set -u
H="$(cd "$(dirname "$0")/.." && pwd)"
R="$H/rust"
# Đích: mặc định x86_64 (Ubuntu thật). `HUBA_LINUX_TARGET=aarch64-unknown-linux-gnu` cho máy ARM
# — và để đo trình cài trong container arm64 chạy TỰ NHIÊN trên Mac (không cần giả lập amd64).
TARGET="${HUBA_LINUX_TARGET:-x86_64-unknown-linux-gnu}"
DUOI=""; [ "$TARGET" = x86_64-unknown-linux-gnu ] || DUOI="-${TARGET%%-*}"

rustup target list --installed 2>/dev/null | grep -qx "$TARGET" \
  || { echo "KHÔNG ĐO ĐƯỢC: thiếu target $TARGET (rustup target add $TARGET)"; exit 2; }
command -v cargo-zigbuild >/dev/null \
  || { echo "KHÔNG ĐO ĐƯỢC: thiếu cargo-zigbuild (cargo install cargo-zigbuild; brew install zig)"; exit 2; }

if [ -n "$(git -C "$H" status --porcelain -- rust install_update_linux.sh DOC-TRUOC-LINUX.txt huba.env.example scripts/huba_cai_linux.py huba.config.json)" ]; then
  echo "⚠ cây có thay đổi CHƯA commit trong phần đóng gói — tên gói mang sha HEAD nhưng nội dung là cây hiện tại"
fi
SHA=$(git -C "$H" rev-parse --short HEAD)

# huba LUÔN đi kèm (Quang 13/11 «cài cả mcp nữa, huba luôn đi kèm mcp codetrail»): MCP codetrail
# `project-agent` dựng cho Linux + cụm cổng bảo mật mà plugin opencode tìm theo đường TƯƠNG ĐỐI
# (`cong-bao-mat-plugin/index.js` gọi `../cong_bao_mat.py`, policy nằm cạnh). Chép cả `~/projects` từ
# Mac sang thì `project-agent` là binary macOS ⟹ opencode báo «1 MCP failed» — nên gói mang bản của nó.
W="$(cd "$H/.." && pwd)"
PA="$W/AI/codetrail/mcp/project-agent-rs"
KEM_SRC="$W/scripts"
for f in "$PA/Cargo.toml" "$KEM_SRC/cong_bao_mat.py" "$KEM_SRC/bao-mat-policy.json" "$KEM_SRC/cong-bao-mat-plugin/index.js"; do
  [ -f "$f" ] || { echo "KHÔNG ĐO ĐƯỢC: thiếu $f — gói thiếu MCP/cổng bảo mật thì không đóng"; exit 2; }
done

( cd "$R" && cargo zigbuild --offline --release --target "$TARGET" --bin huba --bin hubad )
BUILD_EXIT=$?
echo "BUILD_EXIT=$BUILD_EXIT"
[ $BUILD_EXIT -eq 0 ] || exit 1
# Đích dựng của project-agent nằm trong cây huba: không ghi vào cây codetrail.
PA_TARGET="$H/.tmp/pa-linux-target"  # cargo tự tách theo --target bên trong
CARGO_TARGET_DIR="$PA_TARGET" cargo zigbuild --offline --locked --release --target "$TARGET" --manifest-path "$PA/Cargo.toml"
PA_EXIT=$?
echo "PA_EXIT=$PA_EXIT"
[ $PA_EXIT -eq 0 ] || exit 1

D="$H/.tmp/huba-linux-$SHA$DUOI"
rm -rf "$D" "$D.tar.gz"
mkdir -p "$D"
cp "$R/target/$TARGET/release/huba" "$R/target/$TARGET/release/hubad" "$H/install_update_linux.sh" "$H/huba.env.example" "$D/"
cp "$H/DOC-TRUOC-LINUX.txt" "$D/DOC-TRUOC.txt"
mkdir -p "$D/kem/bin" "$D/kem/scripts/cong-bao-mat-plugin"
cp "$PA_TARGET/$TARGET/release/project-agent" "$D/kem/bin/"
cp "$KEM_SRC/cong_bao_mat.py" "$KEM_SRC/bao-mat-policy.json" "$D/kem/scripts/"
cp "$KEM_SRC/cong-bao-mat-plugin/index.js" "$KEM_SRC/cong-bao-mat-plugin/package.json" "$D/kem/scripts/cong-bao-mat-plugin/"
# Phần cấu hình của trình cài + mẫu tài khoản (CHỈ các hàng `claude_accounts` của kho — không
# có bí mật nào trong đó: khoá nằm ở env_file/huba.env của từng máy). Trình cài chỉ THÊM hàng thiếu.
cp "$H/scripts/huba_cai_linux.py" "$D/kem/huba_cai_linux.py"
cp "$H/scripts/prompt-khoi-dau.md" "$D/kem/prompt-khoi-dau.md"
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); json.dump(d.get("claude_accounts",[]),open(sys.argv[2],"w"),ensure_ascii=False,indent=2)' \
  "$H/huba.config.json" "$D/kem/tai-khoan.mau.json" || { echo "❌ không trích được mẫu tài khoản"; exit 1; }
python3 "$D/kem/huba_cai_linux.py" tu-kiem >/dev/null || { echo "❌ huba_cai_linux.py tu-kiem đỏ"; exit 1; }
chmod 755 "$D/huba" "$D/hubad" "$D/install_update_linux.sh" "$D/kem/bin/project-agent"
( cd "$H/.tmp" && COPYFILE_DISABLE=1 tar --no-mac-metadata --no-xattrs -czf "huba-linux-$SHA$DUOI.tar.gz" "huba-linux-$SHA$DUOI" )
TAR_EXIT=$?
echo "TAR_EXIT=$TAR_EXIT"
[ $TAR_EXIT -eq 0 ] || exit 1
tar -tzvf "$D.tar.gz"
shasum -a 256 "$D.tar.gz"
