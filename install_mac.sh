#!/bin/bash
# Cài / nâng huba trên macOS TỪ GÓI (`scripts/dong-goi-mac.sh` ⟹ huba-mac-<sha>.tar.gz) — máy KHÔNG
# cần cây mã huba lẫn cây workspace. Quang 15/11: «dựng gói mac đi» — cùng khuôn gói Linux
# (`install_update_linux.sh`), để cài máy mới không phải sửa tay gì.
#
#   tar xzf huba-mac-<sha>.tar.gz && bash huba-mac-<sha>/install_mac.sh
#   bash ~/huba/install_mac.sh --verify      CHỈ ĐỌC: bản cài có phải bản trong gói không
#
# Các bước: ① chép gói vào ~/huba · ② ~/projects · ③ cấu hình + luật gốc + skill · ④ thư mục tài
#   khoản + MCP claude · ⑤ CLI claude/opencode/codex (bản ghim `cli-ghim.env`) · ⑥ ký + cài hubd +
#   plist launchd (khai đường cổng/MCP/luật trong gói) · ⑦ quyền macOS (NGƯỜI bấm) · ⑧ bot Telegram.
# Đè: HUBA_HOME · HUBA_KHONG_CAI_CLI=1 · HUBA_KHONG_MO_TRANG=1 ·
#     HUBA_THU=1 (lượt THỬ trong HOME giả: KHÔNG ký, KHÔNG launchctl — chỉ dựng tệp; chỉ để đo trình cài).
# Mã thoát: 0 xong · 1 hỏng · 2 KHÔNG ĐO ĐƯỢC (thiếu tệp gói / không phải macOS).
set -u
# 🔴 Hệ điều hành TRƯỚC mọi việc (Quang 15/11: «đây là máy win mà trình cài không kiểm tra hệ điều
# hành trước khi làm à»). Sai máy ⟹ mã 2, chỉ đúng trình cài — không build/ký/ghi gì nửa chừng.
case "$(uname -s 2>/dev/null || echo khong-ro)" in
  Darwin) ;;
  Linux) echo "SAI MÁY: install_mac.sh dành cho macOS (gói) — máy này Linux: giải nén huba-linux-*.tar.gz, chạy install_update_linux.sh"; exit 2 ;;
  MINGW*|MSYS*|CYGWIN*) echo "SAI MÁY: install_mac.sh dành cho macOS (gói) — máy này Windows: mở PowerShell, chạy install_update.ps1 (gói huba-windows-*.zip)"; exit 2 ;;
  *) echo "SAI MÁY: install_mac.sh dành cho macOS (gói) — hệ «$(uname -s 2>/dev/null)» chưa hỗ trợ"; exit 2 ;;
esac
SRC="$(cd "$(dirname "$0")" && pwd)"
DIR="${HUBA_HOME:-$HOME/huba}"
SUP="$HOME/Library/Application Support/hub"
BIN_DIR="$SUP/bin"
LABEL=com.dipgle.hubd
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
THU="${HUBA_THU:-0}"
CO_TTY=0; [ -t 0 ] && [ -t 1 ] && CO_TTY=1
buoc() { echo; echo "── $*"; }

[ -x "$SRC/hubad" ] || { echo "KHÔNG ĐO ĐƯỢC: không có $SRC/hubad — chạy trong thư mục giải nén của gói"; exit 2; }

if [ "${1:-}" = "--verify" ]; then
  [ -x "$BIN_DIR/hubd" ] || { echo "❌ chưa cài: không có $BIN_DIR/hubd"; exit 1; }
  # Bản cài đã KÝ ⟹ byte khác gói; so phần mã sau khi bỏ chữ ký.
  t=$(mktemp -d); cp "$BIN_DIR/hubd" "$t/a"; cp "$SRC/hubad" "$t/b"
  codesign --remove-signature "$t/a" 2>/dev/null; codesign --remove-signature "$t/b" 2>/dev/null
  if cmp -s "$t/a" "$t/b"; then echo "✅ KHỚP — bản cài là hubad của thư mục này"; else echo "❌ LỆCH — bản cài khác hubad của thư mục này"; exit 1; fi
  launchctl print "gui/$(id -u)/$LABEL" >/dev/null 2>&1 \
    && echo "✅ $LABEL đang nạp" || { echo "❌ $LABEL KHÔNG nạp — ~/Library/Logs/hubd.err"; exit 1; }
  exit 0
fi

for f in kem/scripts/cong_bao_mat.py kem/scripts/bao-mat-policy.json kem/scripts/cong-bao-mat-plugin/index.js \
         kem/scripts/mo-dau-phien.py kem/scripts/nhac-chuyen-phien.py \
         kem/bin/project-agent kem/huba_cai_linux.py kem/CLAUDE.md kem/tai-khoan.mau.json \
         cli-ghim.env com.dipgle.hubd.plist sign.sh make-signing-cert.sh; do
  [ -f "$SRC/$f" ] || { echo "❌ gói thiếu $f — dựng lại bằng scripts/dong-goi-mac.sh"; exit 2; }
done
for f in kem/claude/skills kem/claude/agents; do
  [ -d "$SRC/$f" ] || { echo "❌ gói thiếu $f — dựng lại bằng scripts/dong-goi-mac.sh"; exit 2; }
done
command -v python3 >/dev/null || { echo "KHÔNG ĐO ĐƯỢC: thiếu python3 (xcode-select --install)"; exit 2; }
. "$SRC/cli-ghim.env"

# ① Chép gói vào chỗ CỐ ĐỊNH rồi chạy tiếp từ đó.
if [ "$SRC" != "$(mkdir -p "$DIR" && cd "$DIR" && pwd)" ]; then
  buoc "① chép gói vào $DIR (giữ nguyên huba.config.json · huba.env · data/)"
  for f in huba hubad install_mac.sh huba.env.example DOC-TRUOC.txt cli-ghim.env com.dipgle.hubd.plist sign.sh make-signing-cert.sh; do
    [ -e "$SRC/$f" ] || continue
    cp -p "$SRC/$f" "$DIR/$f.moi" && mv -f "$DIR/$f.moi" "$DIR/$f" || exit 1
  done
  rm -rf "$DIR/kem.moi" && cp -Rp "$SRC/kem" "$DIR/kem.moi" && rm -rf "$DIR/kem" && mv "$DIR/kem.moi" "$DIR/kem" || exit 1
  echo "  ✓ $(ls "$DIR" | tr '\n' ' ')"
  exec bash "$DIR/install_mac.sh" "$@"
fi
CFG="${HUB_CONFIG:-$DIR/huba.config.json}"
export HUB_CONFIG="$CFG"
PY="$DIR/kem/huba_cai_linux.py"   # trình trợ giúp chung (tên cũ, không riêng Linux)

# ② Gốc workspace. Mac không có ổ dữ liệu gắn sau đăng nhập ⟹ chưa có thì TẠO — thiếu nó thì
# luật gốc không có chỗ đặt và huba từ chối mở phiên codex (luật là khâu bắt buộc).
buoc "② gốc workspace"
mkdir -p "$HOME/projects" || exit 1
echo "  ✓ $HOME/projects"

# ③ Cấu hình + luật gốc + skill/tác tử — chưa có mới đặt; đã có thì GIỮ.
buoc "③ cấu hình $CFG"
if [ ! -f "$CFG" ]; then
  (cd "$DIR" && ./huba init) || { echo "❌ huba init hỏng"; exit 1; }
fi
python3 "$PY" cau-hinh "$CFG" "$DIR/kem/tai-khoan.mau.json" || exit 1
GOC=$(python3 "$PY" goc-workspace "$CFG")
python3 "$PY" luat-goc "$CFG" "$DIR/kem/CLAUDE.md" || exit 1
python3 "$PY" skill-goc "$CFG" "$DIR/kem/claude" || exit 1
LUAT=""
[ -n "$GOC" ] && [ -f "$GOC/CLAUDE.md" ] && LUAT="$GOC/CLAUDE.md"

# ④ Thư mục tài khoản + MCP codetrail cho mọi acc claude.
buoc "④ thư mục tài khoản"
python3 "$PY" thu-muc-acc "$CFG" || exit 1
python3 "$PY" mcp-claude "$CFG" "$DIR/kem/bin/project-agent" || exit 1

# ⑤ CLI — đúng bản ghim (`cli-ghim.env`); lệch ⟹ cài lại.
buoc "⑤ CLI"
export PATH="$HOME/.local/bin:$HOME/.opencode/bin:$HOME/.npm-global/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
for cli in $(python3 "$PY" can-cli "$CFG"); do
  if [ "${HUBA_KHONG_CAI_CLI:-0}" = 1 ] || [ "$THU" = 1 ]; then echo "  ⚠ bỏ qua cài $cli (HUBA_KHONG_CAI_CLI/HUBA_THU)"; continue; fi
  case "$cli" in
    claude)
      command -v claude >/dev/null || { echo "  cài claude…"; curl -fsSL https://claude.ai/install.sh | bash || echo "  ❌ cài claude hỏng"; }
      command -v claude >/dev/null && echo "  ✓ claude $(claude --version 2>/dev/null | head -1)" ;;
    opencode)
      [ -x "$HOME/.opencode/bin/opencode" ] || { echo "  cài opencode ${OPENCODE_VER}…"; curl -fsSL https://opencode.ai/v2/install | bash -s -- --version "$OPENCODE_VER" --no-modify-path || echo "  ❌ cài opencode hỏng"; }
      [ -x "$HOME/.opencode/bin/opencode" ] && echo "  ✓ opencode $("$HOME/.opencode/bin/opencode" --version 2>/dev/null | head -1)" ;;
    codex)
      co="$("$HOME/.npm-global/bin/codex" --version 2>/dev/null | awk '{print $NF}')"
      if [ "$co" != "$CODEX_VER" ]; then
        if command -v npm >/dev/null; then
          echo "  cài codex ${CODEX_VER} (đang có: ${co:-chưa có})…"
          npm install -g --prefix "$HOME/.npm-global" "@openai/codex@${CODEX_VER}" >/dev/null || echo "  ❌ cài codex hỏng"
        else
          echo "  ❌ thiếu npm — không cài được codex (brew install node)"
        fi
      fi
      [ -x "$HOME/.npm-global/bin/codex" ] && echo "  ✓ codex $("$HOME/.npm-global/bin/codex" --version 2>/dev/null)" ;;
  esac
done

# ⑥ Ký + cài hubd + plist. Ký bằng định danh CỐ ĐỊNH của máy (`Hub Local Signing`) — quyền Trợ năng
# ghim theo định danh + chứng chỉ, ký tạm (ad-hoc) thì mỗi lần nâng phải cấp lại quyền.
buoc "⑥ hubd + launchd"
python3 -B "$DIR/kem/scripts/cong_bao_mat.py" --tu-kiem >/dev/null 2>&1 \
  || echo "  ⚠ cong_bao_mat.py --tu-kiem KHÔNG xanh trên máy này — xem: python3 $DIR/kem/scripts/cong_bao_mat.py --tu-kiem"
python3 -B "$DIR/kem/scripts/mo-dau-phien.py" --tu-kiem >/dev/null 2>&1 \
  || echo "  ⚠ mo-dau-phien.py --tu-kiem KHÔNG xanh trên máy này — xem: python3 $DIR/kem/scripts/mo-dau-phien.py --tu-kiem"
mkdir -p "$BIN_DIR" "$HOME/Library/LaunchAgents" "$HOME/Library/Logs" || exit 1
cp "$DIR/hubad" "$BIN_DIR/hubd.new" && chmod 755 "$BIN_DIR/hubd.new" || exit 1
if [ "$THU" = 1 ]; then
  echo "  ⚠ HUBA_THU=1 — KHÔNG ký"
else
  if ! security find-identity -p codesigning 2>/dev/null | grep -qF '"Hub Local Signing"' \
     && [ ! -f "$SUP/signing/hub-codesign.p12" ]; then
    echo "  chưa có định danh ký — tạo một lần (make-signing-cert.sh)"
    bash "$DIR/make-signing-cert.sh" || { echo "❌ không tạo được định danh ký"; exit 1; }
  fi
  bash "$DIR/sign.sh" "$BIN_DIR/hubd.new" || { echo "❌ ký hubd hỏng"; exit 1; }
fi
mv -f "$BIN_DIR/hubd.new" "$BIN_DIR/hubd" || exit 1

MOI="$(mktemp)"
sed -e "s|__HOME__|$HOME|g" -e "s|__INSTALL_DIR__|$SUP|g" -e "s|__HUB_CONFIG__|$CFG|g" \
    "$DIR/com.dipgle.hubd.plist" > "$MOI" || exit 1
PB=/usr/libexec/PlistBuddy
dat_env() { "$PB" -c "Delete :EnvironmentVariables:$1" "$MOI" >/dev/null 2>&1; "$PB" -c "Add :EnvironmentVariables:$1 string $2" "$MOI" || exit 1; }
# Gói KHÔNG có cây workspace ⟹ khai đường cổng / MCP / luật TRONG gói (như unit systemd của Linux).
dat_env HUBA_OPENCODE_PLUGIN "$DIR/kem/scripts/cong-bao-mat-plugin"
dat_env BAO_MAT_POLICY "$DIR/kem/scripts/bao-mat-policy.json"
dat_env HUBA_MCP_PROJECT_AGENT "$DIR/kem/bin/project-agent"
[ -n "$LUAT" ] && dat_env HUBA_LUAT_GOC "$LUAT"
# PATH của launchd không đọc shell đăng nhập: khai đủ chỗ claude (~/.local/bin, trình cài chính thức),
# opencode, codex (npm), cargo, homebrew nằm.
dat_env PATH "$HOME/.local/bin:$HOME/.opencode/bin:$HOME/.npm-global/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
plutil -lint "$MOI" >/dev/null || { echo "❌ plist dựng ra hỏng: $MOI"; exit 1; }
DOI=1; [ -f "$PLIST" ] && cmp -s "$MOI" "$PLIST" && DOI=0
[ $DOI = 1 ] && { cp "$MOI" "$PLIST" || exit 1; }
rm -f "$MOI"
if [ "$THU" = 1 ]; then
  echo "  ⚠ HUBA_THU=1 — KHÔNG launchctl · plist: $PLIST"
else
  if launchctl print "gui/$(id -u)/$LABEL" >/dev/null 2>&1; then
    if [ $DOI = 1 ]; then
      launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null
      launchctl bootstrap "gui/$(id -u)" "$PLIST" || { echo "❌ bootstrap $LABEL hỏng"; exit 1; }
    else
      launchctl kickstart -k "gui/$(id -u)/$LABEL" || exit 1
    fi
  else
    launchctl bootstrap "gui/$(id -u)" "$PLIST" || { echo "❌ bootstrap $LABEL hỏng"; exit 1; }
  fi
  sleep 2
  launchctl print "gui/$(id -u)/$LABEL" 2>/dev/null | grep -q 'state = running' \
    && echo "  ✓ $LABEL chạy · $BIN_DIR/hubd" \
    || { echo "❌ $LABEL không chạy — ~/Library/Logs/hubd.err"; exit 1; }
fi

# ⑦ Quyền macOS — thứ DUY NHẤT phải có người bấm (hộp thoại hệ thống không tự động được).
buoc "⑦ quyền macOS (người bấm, một lần)"
echo "  · Cài đặt hệ thống › Quyền riêng tư & Bảo mật › Trợ năng: thêm $BIN_DIR/hubd (mở tab ⌘T, gõ phím)"
echo "  · Lần đầu huba điều khiển Terminal, macOS hỏi «hubd muốn điều khiển Terminal» — bấm Cho phép"

# ⑧ Bot Telegram.
buoc "⑧ bot Telegram"
if grep -q '^HUB_TELEGRAM_BOT_TOKEN=..*' "$DIR/huba.env" 2>/dev/null \
   && grep -q '^HUB_TELEGRAM_CHAT_ID=..*' "$DIR/huba.env" 2>/dev/null; then
  echo "  ✓ đã có bot (huba.env)"
elif [ $CO_TTY = 1 ] && [ "${HUBA_KHONG_MO_TRANG:-0}" != 1 ] && [ "$THU" != 1 ]; then
  echo "  chưa có bot ⟹ mở trang cấu hình: dán token (@BotFather), nhắn một câu cho bot, bấm «Lưu»."
  cd "$DIR" && exec ./huba setup
else
  echo "  ⚠ chưa có bot — chạy: $DIR/huba setup"
fi
echo
echo "Xong. Kiểm: bash $DIR/install_mac.sh --verify · nhật ký: ~/Library/Logs/hubd.err"
