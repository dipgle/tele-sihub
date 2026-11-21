#!/bin/bash
# Đóng gói bản macOS (Apple Silicon, aarch64-apple-darwin): dựng huba + hubad + MCP codetrail
# `project-agent` → chép cùng cổng bảo mật, khâu mở đầu phiên, luật/skill, mẫu tài khoản, trình cài
# `install_mac.sh` → tar.gz → in sha256. Gói ra ở .tmp/huba-mac-<sha>.tar.gz.
#
#   bash scripts/dong-goi-mac.sh
#
# Quang 15/11: «dựng gói mac đi» — máy Mac MỚI cài từ gói, không cần cây mã/cây workspace, không sửa tay.
# Cùng khuôn `dong-goi-linux.sh`. hubad trong gói CHƯA ký: trình cài ký bằng định danh của MÁY ĐÍCH
# (quyền Trợ năng ghim theo định danh ấy).
# Thiếu đầu vào thì DỪNG và nói thiếu gì (mã 2), không dựng nửa chừng.
set -u
H="$(cd "$(dirname "$0")/.." && pwd)"
R="$H/rust"
W="$(cd "$H/.." && pwd)"
PA="$W/AI/codetrail/mcp/project-agent-rs"
KEM_SRC="$W/scripts"

[ "$(uname)" = Darwin ] || { echo "KHÔNG ĐO ĐƯỢC: dựng gói Mac phải chạy trên macOS"; exit 2; }
[ "$(uname -m)" = arm64 ] || { echo "KHÔNG ĐO ĐƯỢC: máy dựng không phải Apple Silicon (gói nhắm aarch64-apple-darwin)"; exit 2; }
for f in "$PA/Cargo.toml" "$KEM_SRC/cong_bao_mat.py" "$KEM_SRC/bao-mat-policy.json" "$KEM_SRC/cong-bao-mat-plugin/index.js" \
         "$KEM_SRC/mo-dau-phien.py" "$KEM_SRC/nhac-chuyen-phien.py" "$KEM_SRC/nhac-vao-phien.py" "$H/cli-ghim.env" "$H/install_mac.sh" \
         "$H/com.dipgle.hubd.plist" "$H/sign.sh" "$H/make-signing-cert.sh" "$H/DOC-TRUOC-MAC.txt"; do
  [ -f "$f" ] || { echo "KHÔNG ĐO ĐƯỢC: thiếu $f — gói thiếu phần bắt buộc thì không đóng"; exit 2; }
done
for k in skills agents; do
  [ -d "$W/.claude/$k" ] || { echo "KHÔNG ĐO ĐƯỢC: thiếu $W/.claude/$k — gói thiếu skill/tác tử thì không đóng"; exit 2; }
done

if [ -n "$(git -C "$H" status --porcelain -- rust install_mac.sh cli-ghim.env DOC-TRUOC-MAC.txt huba.env.example com.dipgle.hubd.plist sign.sh make-signing-cert.sh scripts/huba_cai_linux.py scripts/CLAUDE-goc-workspace.md)" ]; then
  echo "⚠ cây có thay đổi CHƯA commit trong phần đóng gói — tên gói mang sha HEAD nhưng nội dung là cây hiện tại"
fi
SHA=$(git -C "$H" rev-parse --short HEAD)

( cd "$R" && cargo build --offline --release --bin huba --bin hubad )
BUILD_EXIT=$?
echo "BUILD_EXIT=$BUILD_EXIT"
[ $BUILD_EXIT -eq 0 ] || exit 1
# Đích dựng của project-agent nằm trong cây huba: không ghi vào cây codetrail.
PA_TARGET="$H/.tmp/pa-mac-target"
CARGO_TARGET_DIR="$PA_TARGET" cargo build --offline --locked --release --manifest-path "$PA/Cargo.toml"
PA_EXIT=$?
echo "PA_EXIT=$PA_EXIT"
[ $PA_EXIT -eq 0 ] || exit 1

D="$H/.tmp/huba-mac-$SHA"
rm -rf "$D" "$D.tar.gz"
mkdir -p "$D/kem/bin" "$D/kem/scripts/cong-bao-mat-plugin"
cp "$R/target/release/huba" "$R/target/release/hubad" "$H/install_mac.sh" "$H/huba.env.example" \
   "$H/cli-ghim.env" "$H/com.dipgle.hubd.plist" "$H/sign.sh" "$H/make-signing-cert.sh" "$D/"
cp "$H/DOC-TRUOC-MAC.txt" "$D/DOC-TRUOC.txt"
cp "$PA_TARGET/release/project-agent" "$D/kem/bin/"
cp "$KEM_SRC/cong_bao_mat.py" "$KEM_SRC/bao-mat-policy.json" "$KEM_SRC/mo-dau-phien.py" "$KEM_SRC/nhac-chuyen-phien.py" "$KEM_SRC/nhac-vao-phien.py" "$D/kem/scripts/"
cp "$KEM_SRC/cong-bao-mat-plugin/index.js" "$KEM_SRC/cong-bao-mat-plugin/package.json" "$D/kem/scripts/cong-bao-mat-plugin/"
cp "$H/scripts/huba_cai_linux.py" "$D/kem/huba_cai_linux.py"
cp "$H/scripts/CLAUDE-goc-workspace.md" "$D/kem/CLAUDE.md"
for k in skills agents; do
  mkdir -p "$D/kem/claude"
  rsync -a --exclude '.DS_Store' "$W/.claude/$k/" "$D/kem/claude/$k/"
done
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); json.dump(d.get("claude_accounts",[]),open(sys.argv[2],"w"),ensure_ascii=False,indent=2)' \
  "$H/huba.config.json" "$D/kem/tai-khoan.mau.json" || { echo "❌ không trích được mẫu tài khoản"; exit 1; }
python3 -B "$D/kem/huba_cai_linux.py" tu-kiem >/dev/null || { echo "❌ huba_cai_linux.py tu-kiem đỏ"; exit 1; }
python3 -B "$D/kem/scripts/mo-dau-phien.py" --tu-kiem >/dev/null || { echo "❌ mo-dau-phien.py --tu-kiem đỏ"; exit 1; }
chmod 755 "$D/huba" "$D/hubad" "$D/install_mac.sh" "$D/sign.sh" "$D/make-signing-cert.sh" "$D/kem/bin/project-agent"
( cd "$H/.tmp" && COPYFILE_DISABLE=1 tar --no-mac-metadata --no-xattrs -czf "huba-mac-$SHA.tar.gz" "huba-mac-$SHA" )
TAR_EXIT=$?
echo "TAR_EXIT=$TAR_EXIT"
[ $TAR_EXIT -eq 0 ] || exit 1
tar -tzf "$D.tar.gz" | grep -v '/kem/claude/' | sed 's/^/  /'
echo "  … + $(tar -tzf "$D.tar.gz" | grep -c '/kem/claude/') mục skill/tác tử"
shasum -a 256 "$D.tar.gz"
