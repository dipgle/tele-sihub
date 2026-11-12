#!/bin/bash
# Cài / cập nhật hubd trên Linux (Ubuntu) — vai trò của install_update.sh (launchd) trên macOS
# và install_update.ps1 (Task Scheduler) trên Windows. Quang 12/11: «Làm bản Ubuntu dùng tmux đi».
#
#   bash install_update_linux.sh            # chép hubd → ~/.local/share/hub/bin, dựng + bật dịch vụ systemd --user
#   bash install_update_linux.sh --verify   # bản cài có KHỚP bản trong thư mục này không + dịch vụ có chạy không
#
# Chạy TRONG thư mục giải nén (có huba, hubad, huba.config.json). Không cần sudo.
# Mã thoát: 0 xong · 1 hỏng · 2 KHÔNG ĐO ĐƯỢC (thiếu systemd --user / thiếu tệp).
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hub/bin"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
UNIT=huba-hubd.service
CFG="${HUB_CONFIG:-$DIR/huba.config.json}"

[ -x "$DIR/hubad" ] || { echo "KHÔNG ĐO ĐƯỢC: không có $DIR/hubad"; exit 2; }
command -v systemctl >/dev/null || { echo "KHÔNG ĐO ĐƯỢC: máy không có systemctl"; exit 2; }
systemctl --user show-environment >/dev/null 2>&1 \
  || { echo "KHÔNG ĐO ĐƯỢC: systemd --user không chạy cho người dùng này (đăng nhập phiên đồ hoạ/ssh rồi chạy lại)"; exit 2; }

if [ "${1:-}" = "--verify" ]; then
  if [ ! -x "$BIN_DIR/hubd" ]; then echo "❌ chưa cài: không có $BIN_DIR/hubd"; exit 1; fi
  A=$(sha256sum "$DIR/hubad" | cut -d' ' -f1); B=$(sha256sum "$BIN_DIR/hubd" | cut -d' ' -f1)
  [ "$A" = "$B" ] && echo "✅ KHỚP — bản cài là bản trong thư mục này ($A)" || { echo "❌ LỆCH — thư mục $A · bản cài $B"; exit 1; }
  if systemctl --user is-active --quiet "$UNIT"; then
    echo "✅ $UNIT đang chạy (pid $(systemctl --user show -p MainPID --value "$UNIT"))"
  else
    echo "❌ $UNIT KHÔNG chạy — journalctl --user -u $UNIT -n 50"; exit 1
  fi
  exit 0
fi

[ -f "$CFG" ] || { echo "KHÔNG ĐO ĐƯỢC: chưa có $CFG — chạy ./huba init trước"; exit 2; }
command -v tmux >/dev/null || echo "⚠ chưa có tmux — /new sẽ hỏng: sudo apt install -y tmux"

mkdir -p "$BIN_DIR" "$UNIT_DIR" "$DIR/logs" || exit 1
# Chép ra tệp mới rồi đổi tên: đè thẳng lên tệp đang chạy thì `cp` báo "Text file busy".
cp "$DIR/hubad" "$BIN_DIR/hubd.new" && chmod 755 "$BIN_DIR/hubd.new" && mv -f "$BIN_DIR/hubd.new" "$BIN_DIR/hubd" || exit 1

# PATH của dịch vụ: systemd --user KHÔNG đọc .bashrc ⇒ khai tay chỗ `claude`/`node`/`tmux` thường nằm
# (cài bằng trình cài gốc ⇒ ~/.local/bin; npm toàn cục ⇒ ~/.npm-global/bin; apt ⇒ /usr/bin).
cat > "$UNIT_DIR/$UNIT" <<EOF
[Unit]
Description=huba hubd — cầu nối phiên claude (tmux) ra Telegram
After=network-online.target

[Service]
ExecStart=$BIN_DIR/hubd
WorkingDirectory=$DIR
Environment=HUB_CONFIG=$CFG
Environment=HUB_LOG_LEVEL=info
Environment=PATH=$HOME/.local/bin:$HOME/.npm-global/bin:$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
Restart=always
RestartSec=30
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
  echo "install_update_linux.sh: $UNIT chạy (pid $(systemctl --user show -p MainPID --value "$UNIT")) · $BIN_DIR/hubd"
else
  echo "❌ $UNIT không lên — journalctl --user -u $UNIT -n 50 · $DIR/logs/hubd.err"; exit 1
fi
# Không có linger thì dịch vụ chết khi người dùng đăng xuất. Bật cần quyền: báo, không tự sudo.
if [ "$(loginctl show-user "$USER" -p Linger --value 2>/dev/null)" != "yes" ]; then
  echo "⚠ linger TẮT ⇒ hubd dừng khi đăng xuất. Bật một lần: sudo loginctl enable-linger $USER"
fi
