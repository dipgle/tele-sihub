#!/bin/bash
# Đóng gói bản Windows: dựng .exe (x86_64-pc-windows-gnu) → chép → zip → in sha256. Chạy từ đâu cũng
# được; gói ra ở .tmp/huba-windows-<sha>.zip.
#
#   bash scripts/dong-goi-windows.sh
#
# Gói mang CÙNG phần dùng chung với gói Linux (`kem/`): luật chung CLAUDE.md, skill + tác tử, cổng bảo
# mật (policy + cổng + plugin), hook mở đầu phiên, MCP codetrail `project-agent.exe`, mẫu tài khoản,
# bộ cấu hình `huba_cai.py` (Quang 13/11 «huba luôn đi kèm mcp codetrail», 21/11 «đồng bộ hết tính năng
# để dựng bản cài window»). Chỉ mang hàng tài khoản `claude` (opencode/codex chưa có bản Windows).
#
# Bộ biên dịch C cho đích Windows (crate `ring`): có mingw ⟹ `cargo build`; không có (máy Ubuntu, sudo
# cần mật khẩu) ⟹ `cargo-zigbuild` + zig lấy từ venv `GATE_ZIG_VENV` (mặc định ~/.local/zig-venv).
# Thiếu cả hai thì DỪNG và nói thiếu gì (mã 2), không dựng nửa chừng.
set -u
H="$(cd "$(dirname "$0")/.." && pwd)"
R="$H/rust"
TARGET=x86_64-pc-windows-gnu

GATE_ZIG_VENV="${GATE_ZIG_VENV:-$HOME/.local/zig-venv}"
[ -x "$GATE_ZIG_VENV/bin/python" ] && PATH="$GATE_ZIG_VENV/bin:$PATH"

rustup target list --installed 2>/dev/null | grep -qx "$TARGET" \
  || { echo "KHÔNG ĐO ĐƯỢC: thiếu target $TARGET (rustup target add $TARGET)"; exit 2; }
if command -v x86_64-w64-mingw32-gcc >/dev/null; then
  DUNG="build"
elif command -v cargo-zigbuild >/dev/null && python3 -m ziglang version >/dev/null 2>&1; then
  DUNG="zigbuild"
else
  echo "KHÔNG ĐO ĐƯỢC: thiếu linker mingw (x86_64-w64-mingw32-gcc) lẫn cargo-zigbuild+ziglang"
  echo "  (Mac: brew install mingw-w64 · Linux: pip install ziglang trong venv + cargo install cargo-zigbuild)"
  exit 2
fi
echo "dựng bằng: cargo $DUNG ($TARGET)"

if [ -n "$(git -C "$H" status --porcelain -- rust install_update.ps1 scripts/do-windows.ps1 DOC-TRUOC-WINDOWS.txt scripts/huba_cai_linux.py scripts/CLAUDE-goc-workspace.md huba.config.json)" ]; then
  echo "⚠ cây có thay đổi CHƯA commit trong phần đóng gói — tên gói mang sha HEAD nhưng nội dung là cây hiện tại"
fi
SHA=$(git -C "$H" rev-parse --short HEAD)

W="$(cd "$H/.." && pwd)"
PA="$W/AI/codetrail/mcp/project-agent-rs"
KEM_SRC="$W/scripts"
for f in "$PA/Cargo.toml" "$KEM_SRC/cong_bao_mat.py" "$KEM_SRC/bao-mat-policy.json" "$KEM_SRC/cong-bao-mat-plugin/index.js" \
         "$KEM_SRC/mo-dau-phien.py" "$KEM_SRC/nhac-chuyen-phien.py" "$H/scripts/huba_cai_linux.py" "$H/scripts/CLAUDE-goc-workspace.md"; do
  [ -f "$f" ] || { echo "KHÔNG ĐO ĐƯỢC: thiếu $f — gói thiếu MCP/cổng bảo mật/luật thì không đóng"; exit 2; }
done
for k in skills agents; do
  [ -d "$W/.claude/$k" ] || { echo "KHÔNG ĐO ĐƯỢC: thiếu $W/.claude/$k — gói thiếu skill/tác tử thì không đóng"; exit 2; }
done

( cd "$R" && cargo "$DUNG" --offline --release --target "$TARGET" --bin huba --bin hubad )
BUILD_EXIT=$?
echo "BUILD_EXIT=$BUILD_EXIT"
[ $BUILD_EXIT -eq 0 ] || exit 1
# Đích dựng của project-agent nằm trong cây huba: không ghi vào cây codetrail.
PA_TARGET="$H/.tmp/pa-windows-target"
CARGO_TARGET_DIR="$PA_TARGET" cargo "$DUNG" --offline --locked --release --target "$TARGET" --manifest-path "$PA/Cargo.toml"
PA_EXIT=$?
echo "PA_EXIT=$PA_EXIT"
[ $PA_EXIT -eq 0 ] || exit 1

D="$H/.tmp/huba-windows-$SHA"
rm -rf "$D" "$D.zip"
mkdir -p "$D/scripts" "$D/kem/bin" "$D/kem/scripts/cong-bao-mat-plugin" "$D/kem/claude"
cp "$R/target/$TARGET/release/huba.exe" "$R/target/$TARGET/release/hubad.exe" "$H/install_update.ps1" "$D/"
cp "$H/scripts/do-windows.ps1" "$D/scripts/"
cp "$H/DOC-TRUOC-WINDOWS.txt" "$D/DOC-TRUOC.txt"
cp "$PA_TARGET/$TARGET/release/project-agent.exe" "$D/kem/bin/"
cp "$KEM_SRC/cong_bao_mat.py" "$KEM_SRC/bao-mat-policy.json" "$KEM_SRC/mo-dau-phien.py" "$KEM_SRC/nhac-chuyen-phien.py" "$D/kem/scripts/"
cp "$KEM_SRC/cong-bao-mat-plugin/index.js" "$KEM_SRC/cong-bao-mat-plugin/package.json" "$D/kem/scripts/cong-bao-mat-plugin/"
# Bộ cấu hình dùng chung với Linux (cùng tệp nguồn; trên Windows nó tạo JUNCTION thay symlink).
cp "$H/scripts/huba_cai_linux.py" "$D/kem/huba_cai.py"
cp "$H/scripts/CLAUDE-goc-workspace.md" "$D/kem/CLAUDE.md"
for k in skills agents; do
  rsync -a --exclude '.DS_Store' "$W/.claude/$k/" "$D/kem/claude/$k/"
done
# Mẫu tài khoản: CHỈ hàng `claude` (không có bí mật — khoá nằm ở huba.env của từng máy).
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); a=[x for x in d.get("claude_accounts",[]) if (x.get("cli") or "claude").lower()=="claude"]; json.dump(a,open(sys.argv[2],"w"),ensure_ascii=False,indent=2)' \
  "$H/huba.config.json" "$D/kem/tai-khoan.mau.json" || { echo "❌ không trích được mẫu tài khoản"; exit 1; }
python3 "$D/kem/huba_cai.py" tu-kiem >/dev/null || { echo "❌ huba_cai.py tu-kiem đỏ"; exit 1; }
( cd "$H/.tmp" && zip -qr "huba-windows-$SHA.zip" "huba-windows-$SHA" )
ZIP_EXIT=$?
echo "ZIP_EXIT=$ZIP_EXIT"
[ $ZIP_EXIT -eq 0 ] || exit 1
unzip -l "$D.zip" | tail -n +4 | sed '$d' | sed '$d' | awk '{print $1, $4}' | head -60
sha256sum "$D.zip" 2>/dev/null || shasum -a 256 "$D.zip"
