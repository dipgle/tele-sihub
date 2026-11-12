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
TARGET=x86_64-unknown-linux-gnu

rustup target list --installed 2>/dev/null | grep -qx "$TARGET" \
  || { echo "KHÔNG ĐO ĐƯỢC: thiếu target $TARGET (rustup target add $TARGET)"; exit 2; }
command -v cargo-zigbuild >/dev/null \
  || { echo "KHÔNG ĐO ĐƯỢC: thiếu cargo-zigbuild (cargo install cargo-zigbuild; brew install zig)"; exit 2; }

if [ -n "$(git -C "$H" status --porcelain -- rust install_update_linux.sh DOC-TRUOC-LINUX.txt huba.env.example)" ]; then
  echo "⚠ cây có thay đổi CHƯA commit trong phần đóng gói — tên gói mang sha HEAD nhưng nội dung là cây hiện tại"
fi
SHA=$(git -C "$H" rev-parse --short HEAD)

( cd "$R" && cargo zigbuild --offline --release --target "$TARGET" --bin huba --bin hubad )
BUILD_EXIT=$?
echo "BUILD_EXIT=$BUILD_EXIT"
[ $BUILD_EXIT -eq 0 ] || exit 1

D="$H/.tmp/huba-linux-$SHA"
rm -rf "$D" "$D.tar.gz"
mkdir -p "$D"
cp "$R/target/$TARGET/release/huba" "$R/target/$TARGET/release/hubad" "$H/install_update_linux.sh" "$H/huba.env.example" "$D/"
cp "$H/DOC-TRUOC-LINUX.txt" "$D/DOC-TRUOC.txt"
chmod 755 "$D/huba" "$D/hubad" "$D/install_update_linux.sh"
( cd "$H/.tmp" && COPYFILE_DISABLE=1 tar --no-mac-metadata --no-xattrs -czf "huba-linux-$SHA.tar.gz" "huba-linux-$SHA" )
TAR_EXIT=$?
echo "TAR_EXIT=$TAR_EXIT"
[ $TAR_EXIT -eq 0 ] || exit 1
tar -tzvf "$D.tar.gz"
shasum -a 256 "$D.tar.gz"
