#!/bin/bash
# Cài / cập nhật huba trên Linux (Ubuntu) — MỘT lệnh, không sửa tệp nào bằng tay. Vai trò của
# install_update.sh (launchd) trên macOS và install_update.ps1 (Task Scheduler) trên Windows.
# Quang 12/11 «Làm bản Ubuntu dùng tmux đi» · 13/11 «chỉnh nốt để gói cài đặt không phải chỉnh
# sửa gì mà chạy được ngay» · «thêm giao diện quản lý acc» · «thêm để cài đặt bot tele luôn».
#
#   bash install_update_linux.sh            # cài lần đầu HOẶC cập nhật (giải nén gói mới rồi chạy)
#   bash install_update_linux.sh --verify   # bản cài KHỚP gói không + dịch vụ có chạy không
#
# Làm theo thứ tự, việc nào đã xong thì bỏ qua (chạy lại bao nhiêu lần cũng được):
#   ① chép gói vào $HUBA_HOME (mặc định ~/huba) — KHÔNG BAO GIỜ đè huba.config.json · huba.env · data/
#   ② tmux + linger (cần sudo — mật khẩu do chính người cài gõ; máy không có tty thì chỉ báo)
#   ③ cấu hình: chưa có ⟹ huba init; gốc dự án ~/projects; thêm các tài khoản CÒN THIẾU từ mẫu
#   ④ thư mục tài khoản + liên kết projects · ⑤ claude / opencode 2.2.7 nếu thiếu
#   ⑥ dịch vụ systemd --user huba-hubd · ⑦ lối vào «huba — quản lý» trong menu ứng dụng
#   ⑧ tự kiểm (linux-tu-kiem) · ⑨ chưa có bot Telegram ⟹ mở trang quản lý (bot + tài khoản)
# Đè: HUBA_HOME · HUBA_KHONG_CAI_CLI=1 (không tự cài claude/opencode) · HUBA_KHONG_MO_TRANG=1.
# Mã thoát: 0 xong · 1 hỏng · 2 KHÔNG ĐO ĐƯỢC (thiếu systemd --user / thiếu tệp gói).
set -u
SRC="$(cd "$(dirname "$0")" && pwd)"
DIR="${HUBA_HOME:-$HOME/huba}"
BIN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hub/bin"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
UNIT=huba-hubd.service
OPENCODE_VER=2.2.7
CODEX_VER=0.169.0
CO_TTY=0; [ -t 0 ] && [ -t 1 ] && CO_TTY=1
buoc() { echo; echo "── $*"; }

[ -x "$SRC/hubad" ] || { echo "KHÔNG ĐO ĐƯỢC: không có $SRC/hubad — chạy trong thư mục giải nén của gói"; exit 2; }
command -v systemctl >/dev/null || { echo "KHÔNG ĐO ĐƯỢC: máy không có systemctl"; exit 2; }
systemctl --user show-environment >/dev/null 2>&1 \
  || { echo "KHÔNG ĐO ĐƯỢC: systemd --user không chạy cho người dùng này (đăng nhập phiên đồ hoạ/ssh rồi chạy lại)"; exit 2; }

if [ "${1:-}" = "--verify" ]; then
  if [ ! -x "$BIN_DIR/hubd" ]; then echo "❌ chưa cài: không có $BIN_DIR/hubd"; exit 1; fi
  A=$(sha256sum "$SRC/hubad" | cut -d' ' -f1); B=$(sha256sum "$BIN_DIR/hubd" | cut -d' ' -f1)
  [ "$A" = "$B" ] && echo "✅ KHỚP — bản cài là bản trong thư mục này ($A)" || { echo "❌ LỆCH — thư mục $A · bản cài $B"; exit 1; }
  if systemctl --user is-active --quiet "$UNIT"; then
    echo "✅ $UNIT đang chạy (pid $(systemctl --user show -p MainPID --value "$UNIT"))"
  else
    echo "❌ $UNIT KHÔNG chạy — journalctl --user -u $UNIT -n 50"; exit 1
  fi
  exit 0
fi

# Gói mang `kem/` (cổng bảo mật + MCP codetrail cho Linux + mẫu tài khoản + phần cấu hình). Thiếu
# ⟹ opencode chạy KHÔNG có tường bảo mật và báo «1 plugin failed / 1 MCP failed» — dừng.
for f in kem/scripts/cong_bao_mat.py kem/scripts/bao-mat-policy.json kem/scripts/cong-bao-mat-plugin/index.js \
         kem/bin/project-agent kem/huba_cai_linux.py kem/CLAUDE.md; do
  [ -f "$SRC/$f" ] || { echo "❌ gói thiếu $f — dựng lại bằng scripts/dong-goi-linux.sh"; exit 2; }
done
for f in kem/claude/skills kem/claude/agents; do
  [ -d "$SRC/$f" ] || { echo "❌ gói thiếu $f — dựng lại bằng scripts/dong-goi-linux.sh"; exit 2; }
done
command -v python3 >/dev/null || { echo "KHÔNG ĐO ĐƯỢC: thiếu python3 (sudo apt install -y python3)"; exit 2; }

# ① Chép gói vào chỗ CỐ ĐỊNH rồi chạy tiếp từ đó — giải nén bản mới ở đâu cũng được.
if [ "$SRC" != "$(mkdir -p "$DIR" && cd "$DIR" && pwd)" ]; then
  buoc "① chép gói vào $DIR (giữ nguyên huba.config.json · huba.env · data/)"
  for f in huba hubad install_update_linux.sh huba.env.example DOC-TRUOC.txt; do
    [ -e "$SRC/$f" ] || continue
    # tệp mới rồi đổi tên: đè thẳng lên tệp đang chạy thì `cp` báo "Text file busy".
    cp -p "$SRC/$f" "$DIR/$f.moi" && mv -f "$DIR/$f.moi" "$DIR/$f" || exit 1
  done
  rm -rf "$DIR/kem.moi" && cp -rp "$SRC/kem" "$DIR/kem.moi" && rm -rf "$DIR/kem" && mv "$DIR/kem.moi" "$DIR/kem" || exit 1
  echo "  ✓ $(ls "$DIR" | tr '\n' ' ')"
  exec bash "$DIR/install_update_linux.sh" "$@"
fi
CFG="${HUB_CONFIG:-$DIR/huba.config.json}"
export HUB_CONFIG="$CFG"
PY="$DIR/kem/huba_cai_linux.py"

# ② tmux + linger. sudo chỉ khi có người ngồi gõ mật khẩu; không thì nói đúng lệnh.
buoc "② tmux + linger"
if ! command -v tmux >/dev/null; then
  if [ $CO_TTY = 1 ] && command -v apt-get >/dev/null; then
    echo "  cài tmux (sudo sẽ hỏi mật khẩu của bạn)…"
    sudo apt-get install -y tmux || { echo "❌ không cài được tmux — /new sẽ hỏng"; exit 1; }
  else
    echo "❌ chưa có tmux và không có ai gõ sudo — chạy: sudo apt install -y tmux, rồi chạy lại"; exit 2
  fi
fi
echo "  ✓ $(tmux -V)"
if [ "$(loginctl show-user "$USER" -p Linger --value 2>/dev/null)" != "yes" ]; then
  if [ $CO_TTY = 1 ]; then
    echo "  bật linger để hubd chạy cả khi đăng xuất (sudo)…"
    sudo loginctl enable-linger "$USER" || echo "  ⚠ chưa bật được linger — hubd dừng khi đăng xuất"
  else
    echo "  ⚠ linger TẮT ⇒ hubd dừng khi đăng xuất. Bật: sudo loginctl enable-linger $USER"
  fi
fi
[ "$(loginctl show-user "$USER" -p Linger --value 2>/dev/null)" = "yes" ] && echo "  ✓ linger bật"

# ③ Cấu hình — chưa có thì tạo; mọi lượt: gốc dự án + tài khoản còn thiếu (hàng đã có KHÔNG đụng).
buoc "③ cấu hình $CFG"
if [ ! -f "$CFG" ]; then
  (cd "$DIR" && ./huba init) || { echo "❌ huba init hỏng"; exit 1; }
fi
python3 "$PY" cau-hinh "$CFG" "$DIR/kem/tai-khoan.mau.json" || exit 1
# Luật chung của máy = `CLAUDE.md` ở GỐC workspace, đúng cấu trúc chủ máy đang
# dùng (`~/projects/CLAUDE.md`). Chưa có ⟹ chép bản SẠCH của gói; đã có (kể cả
# bản chủ máy đã sửa) ⟹ GIỮ. claude tự nạp tệp này (cwd + mọi thư mục cha);
# opencode thì không — nên unit khai HUBA_LUAT_GOC để huba trỏ `instructions`
# của opencode tới đúng tệp ấy (đo 13/11: opencode 2.2.7 chỉ tự đọc AGENTS.md,
# và chỉ tới gốc git của cwd).
GOC=$(python3 "$PY" goc-workspace "$CFG")
python3 "$PY" luat-goc "$CFG" "$DIR/kem/CLAUDE.md" || exit 1
# Skill + tác tử của workspace: `<gốc>/.claude/{skills,agents}` — cùng luật «chưa có mới đặt».
python3 "$PY" skill-goc "$CFG" "$DIR/kem/claude" || exit 1
LUAT=""
[ -n "$GOC" ] && [ -f "$GOC/CLAUDE.md" ] && LUAT="$GOC/CLAUDE.md"
LUAT_ENV=""
[ -n "$LUAT" ] && LUAT_ENV="Environment=HUBA_LUAT_GOC=$LUAT"

# ④ Thư mục tài khoản + liên kết projects (sổ phiên chung — /session thấy phiên của mọi acc).
buoc "④ thư mục tài khoản"
python3 "$PY" thu-muc-acc "$CFG" || exit 1
# MCP codetrail cho mọi acc claude (opencode được chèn lúc mở — `accounts::chen_mcp_codetrail`).
python3 "$PY" mcp-claude "$CFG" "$DIR/kem/bin/project-agent" || exit 1

# ⑤ CLI mà các tài khoản cần. Trình cài GỐC của nhà sản xuất; opencode ghim bản đã đo.
buoc "⑤ CLI"
export PATH="$HOME/.local/bin:$HOME/.opencode/bin:$PATH"
for cli in $(python3 "$PY" can-cli "$CFG"); do
  case "$cli" in
    claude)
      if ! command -v claude >/dev/null; then
        if [ "${HUBA_KHONG_CAI_CLI:-0}" = 1 ]; then echo "  ⚠ thiếu claude (HUBA_KHONG_CAI_CLI=1 — không tự cài)"
        else echo "  cài claude (claude.ai/install.sh)…"; curl -fsSL https://claude.ai/install.sh | bash || echo "  ❌ cài claude hỏng"; fi
      fi
      command -v claude >/dev/null && echo "  ✓ claude $(claude --version 2>/dev/null | head -1)" ;;
    opencode)
      if [ ! -x "$HOME/.opencode/bin/opencode" ]; then
        if [ "${HUBA_KHONG_CAI_CLI:-0}" = 1 ]; then echo "  ⚠ thiếu opencode (HUBA_KHONG_CAI_CLI=1 — không tự cài)"
        else echo "  cài opencode ${OPENCODE_VER}…"; curl -fsSL https://opencode.ai/v2/install | bash -s -- --version "$OPENCODE_VER" --no-modify-path || echo "  ❌ cài opencode hỏng"; fi
      fi
      [ -x "$HOME/.opencode/bin/opencode" ] && echo "  ✓ opencode $("$HOME/.opencode/bin/opencode" --version 2>/dev/null | head -1)" ;;
    codex)
      # OpenAI Codex CLI (acc ChatGPT). Gói npm `@openai/codex` vào ~/.npm-global (có trong PATH
      # của unit bên dưới) — cùng chỗ `codex_cli` mặc định của huba. Ghim bản đã đo (14/11).
      if [ ! -x "$HOME/.npm-global/bin/codex" ]; then
        if [ "${HUBA_KHONG_CAI_CLI:-0}" = 1 ]; then echo "  ⚠ thiếu codex (HUBA_KHONG_CAI_CLI=1 — không tự cài)"
        elif ! command -v npm >/dev/null; then echo "  ❌ thiếu npm — không cài được codex (sudo apt install -y nodejs npm)"
        else echo "  cài codex ${CODEX_VER}…"; NODE_OPTIONS=--dns-result-order=ipv4first npm install -g --prefix "$HOME/.npm-global" "@openai/codex@${CODEX_VER}" || echo "  ❌ cài codex hỏng"; fi
      fi
      [ -x "$HOME/.npm-global/bin/codex" ] && echo "  ✓ codex $("$HOME/.npm-global/bin/codex" --version 2>/dev/null | head -1)" ;;
  esac
done

# ⑥ Dịch vụ. PATH: systemd --user KHÔNG đọc .bashrc ⇒ khai tay chỗ claude/opencode/tmux nằm.
buoc "⑥ dịch vụ $UNIT"
python3 "$DIR/kem/scripts/cong_bao_mat.py" --tu-kiem >/dev/null 2>&1 \
  || echo "  ⚠ cong_bao_mat.py --tu-kiem KHÔNG xanh trên máy này — xem: python3 $DIR/kem/scripts/cong_bao_mat.py --tu-kiem"
mkdir -p "$BIN_DIR" "$UNIT_DIR" "$DIR/logs" || exit 1
cp "$DIR/hubad" "$BIN_DIR/hubd.new" && chmod 755 "$BIN_DIR/hubd.new" && mv -f "$BIN_DIR/hubd.new" "$BIN_DIR/hubd" || exit 1
cat > "$UNIT_DIR/$UNIT" <<EOF
[Unit]
Description=huba hubd — cầu nối phiên claude (tmux) ra Telegram
After=network-online.target

[Service]
ExecStart=$BIN_DIR/hubd
WorkingDirectory=$DIR
Environment=HUB_CONFIG=$CFG
Environment=HUB_LOG_LEVEL=info
Environment=HUBA_OPENCODE_PLUGIN=$DIR/kem/scripts/cong-bao-mat-plugin
Environment=BAO_MAT_POLICY=$DIR/kem/scripts/bao-mat-policy.json
Environment=HUBA_MCP_PROJECT_AGENT=$DIR/kem/bin/project-agent
$LUAT_ENV
Environment=PATH=$HOME/.local/bin:$HOME/.opencode/bin:$HOME/.npm-global/bin:$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
Restart=always
RestartSec=30
# Máy chủ tmux (mọi phiên claude/opencode) và cửa sổ xem do hubd đẻ ra nằm trong cgroup
# của dịch vụ. Mặc định control-group ⟹ mỗi restart/upgrade/sập GIẾT HẾT phiên (đo 13/11:
# tmux pid 13018 trong huba-hubd.service). Trên Mac cửa sổ Terminal sống độc lập launchd.
KillMode=process
StandardOutput=append:$DIR/logs/hubd.out
StandardError=append:$DIR/logs/hubd.err

[Install]
WantedBy=default.target
EOF
systemctl --user daemon-reload || exit 1
systemctl --user enable "$UNIT" >/dev/null 2>&1 || exit 1
systemctl --user restart "$UNIT" || exit 1
sleep 2
if systemctl --user is-active --quiet "$UNIT"; then
  echo "  ✓ $UNIT chạy (pid $(systemctl --user show -p MainPID --value "$UNIT")) · $BIN_DIR/hubd"
else
  echo "❌ $UNIT không lên — journalctl --user -u $UNIT -n 50 · $DIR/logs/hubd.err"; exit 1
fi

# ⑦ Lối vào trong menu ứng dụng: trang quản lý (bot Telegram + tài khoản). Terminal=true: trang
# chạy tới khi đóng cửa sổ — cửa sổ ấy là chỗ thấy URL và là nút tắt.
buoc "⑦ lối vào «huba — quản lý»"
mkdir -p "$APP_DIR"
cat > "$APP_DIR/huba-quan-ly.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=huba — quản lý
Comment=Bot Telegram và tài khoản Claude/opencode của huba trên máy này
Exec=env HUB_CONFIG=$CFG $DIR/huba accounts
Path=$DIR
Terminal=true
Icon=utilities-terminal
Categories=Utility;
EOF
echo "  ✓ $APP_DIR/huba-quan-ly.desktop (mở bằng: tìm «huba» trong menu ứng dụng, hoặc $DIR/huba accounts)"

# ⑧ Tự kiểm lớp tmux trên chính máy này.
buoc "⑧ tự kiểm"
(cd "$DIR" && ./huba linux-tu-kiem 2>/dev/null)
TU_KIEM=$?
[ $TU_KIEM = 0 ] && echo "  ✓ linux-tu-kiem đạt hết" || echo "  ⚠ linux-tu-kiem mã $TU_KIEM — xem các dòng ❌/⚠ ở trên"

# ⑨ Bot Telegram: chưa có khoá ⟹ huba chưa nghe được điện thoại ⟹ mở trang quản lý.
buoc "⑨ bot Telegram"
if grep -q '^HUB_TELEGRAM_BOT_TOKEN=..*' "$DIR/huba.env" 2>/dev/null \
   && grep -q '^HUB_TELEGRAM_CHAT_ID=..*' "$DIR/huba.env" 2>/dev/null; then
  echo "  ✓ đã có bot (huba.env) — đổi bot/tài khoản: menu ứng dụng «huba — quản lý»"
elif [ $CO_TTY = 1 ] && [ "${HUBA_KHONG_MO_TRANG:-0}" != 1 ] && { [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]; }; then
  echo "  chưa có bot ⟹ mở trang quản lý: dán token (@BotFather), nhắn một câu cho bot, bấm «Lưu»."
  echo "  Trang tự dò chat id, ghi huba.env rồi khởi động lại huba. Xong thì Ctrl-C ở đây."
  cd "$DIR" && exec ./huba accounts
else
  echo "  ⚠ chưa có bot — mở «huba — quản lý» trong menu ứng dụng (hoặc chạy: $DIR/huba accounts)"
fi
echo
echo "Xong. Kiểm: bash $DIR/install_update_linux.sh --verify · nhật ký: $DIR/logs/hubd.err"
