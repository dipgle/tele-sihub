#!/usr/bin/env python3
"""Phần CẤU HÌNH của trình cài Linux (`install_update_linux.sh`) — đi trong gói ở `kem/`.

Quang 13/11: «chỉnh nốt để gói cài đặt không phải chỉnh sửa gì mà chạy được ngay».
Mọi thứ hôm ấy phải làm TAY trên Ubuntu — đặt gốc dự án, chép 9 hàng tài khoản (bỏ
alias zsh `claude2`/`claude3`), tạo thư mục acc + liên kết `projects` — làm ở đây.

  python3 huba_cai_linux.py cau-hinh <huba.config.json> <tai-khoan.mau.json>
  python3 huba_cai_linux.py thu-muc-acc <huba.config.json>
  python3 huba_cai_linux.py can-cli <huba.config.json>      # in: claude / opencode
  python3 huba_cai_linux.py tu-kiem                          # bài kiểm của chính tệp này

Luật: KHÔNG sửa hàng tài khoản đã có (chỉ thêm hàng còn thiếu) · ghi tệp mới rồi đổi
tên, giữ `.bak-truoc-cai` · không đọc/ghi bí mật nào.
Mã thoát: 0 xong · 1 hỏng · 2 không đo được (thiếu tệp).
"""
import json
import os
import re
import shutil
import sys
import tempfile

HOME = os.path.expanduser("~")


def mo_rong(p):
    return os.path.expanduser(p) if p else p


def doc_json(p):
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def ghi_json(p, d):
    """Ghi tệp mới cạnh `p` rồi đổi tên — không bao giờ để tệp nửa chừng."""
    tmp = p + ".huba-cai-tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(d, f, ensure_ascii=False, indent=2)
        f.write("\n")
    os.replace(tmp, p)


def lam_sach_hang(a):
    """Hàng mẫu chép từ Mac: `launch` là alias zsh trần (`claude2`) thì bash trên
    Linux không có ⟹ bỏ, để huba tự dựng lệnh từ `config_dir`. Thuần."""
    a = dict(a)
    l = (a.get("launch") or "").strip()
    if re.fullmatch(r"claude\d*", l):
        a.pop("launch", None)
    return a


def gop_tai_khoan(cfg, mau):
    """Thêm hàng còn thiếu (theo tên), giữ nguyên hàng đã có. Trả danh sách tên đã thêm. Thuần."""
    co = {a.get("name") for a in cfg.get("claude_accounts", [])}
    them = []
    for a in mau:
        if a.get("name") and a["name"] not in co:
            cfg.setdefault("claude_accounts", []).append(lam_sach_hang(a))
            them.append(a["name"])
    cfg["claude_accounts"] = sorted(cfg.get("claude_accounts", []), key=lambda a: a.get("name", ""))
    return them


def dat_goc_du_an(cfg, home=HOME):
    """`workspace_root` trống và có `~/projects` ⟹ đặt nó. Trả câu nói (hoặc None). Thuần + đọc đĩa."""
    if (cfg.get("workspace_root") or "").strip():
        return None
    goc = os.path.join(home, "projects")
    if os.path.isdir(goc):
        cfg["workspace_root"] = goc
        return f"gốc dự án = {goc}"
    return f"⚠ chưa có {goc} — /new mở ở thư mục nhà; tạo/đồng bộ ~/projects rồi chạy lại trình cài"


def cmd_cau_hinh(cfg_p, mau_p):
    if not os.path.isfile(cfg_p):
        print(f"KHÔNG ĐO ĐƯỢC: chưa có {cfg_p} (huba init chưa chạy)")
        return 2
    cfg = doc_json(cfg_p)
    mau = doc_json(mau_p) if mau_p and os.path.isfile(mau_p) else []
    them = gop_tai_khoan(cfg, mau)
    goc = dat_goc_du_an(cfg)
    if them or goc and not goc.startswith("⚠"):
        bak = cfg_p + ".bak-truoc-cai"
        if not os.path.exists(bak):
            shutil.copy2(cfg_p, bak)
        ghi_json(cfg_p, cfg)
    print(f"  tài khoản: {len(cfg.get('claude_accounts', []))}"
          + (f" (thêm {', '.join(them)})" if them else " (không thêm)"))
    if goc:
        print(f"  {goc}")
    return 0


def thu_muc_acc(cfg, home=HOME):
    """Mỗi acc `claude`: thư mục cấu hình + `projects` → `~/.claude/projects` (sổ phiên
    chung). Không đụng thứ đã có. Trả số mục đã tạo."""
    chung = os.path.join(home, ".claude", "projects")
    os.makedirs(chung, exist_ok=True)
    tao = 0
    for a in cfg.get("claude_accounts", []):
        if (a.get("cli") or "claude").strip().lower() == "opencode":
            continue
        d = mo_rong(a.get("config_dir") or "~/.claude")
        d = d if os.path.isabs(d) else os.path.join(home, d)
        if not os.path.isdir(d):
            os.makedirs(d)
            tao += 1
        p = os.path.join(d, "projects")
        if os.path.realpath(d) != os.path.realpath(os.path.join(home, ".claude")) and not os.path.lexists(p):
            os.symlink(chung, p)
            tao += 1
    return tao


def can_cli(cfg):
    can = set()
    for a in cfg.get("claude_accounts", []) or [{}]:
        can.add("opencode" if (a.get("cli") or "").strip().lower() == "opencode" else "claude")
    return sorted(can)


def tu_kiem():
    hong = 0
    tong = 0

    def ok(dk, ten):
        nonlocal hong, tong
        print(("  ok   " if dk else "  HỎNG ") + ten)
        tong += 1
        hong += 0 if dk else 1

    ok("launch" not in lam_sach_hang({"name": "a", "launch": "claude2"}), "bỏ alias claude2")
    ok("launch" not in lam_sach_hang({"name": "a", "launch": "claude"}), "bỏ launch claude trần")
    giu = {"name": "a", "launch": "env X=1 claude --debug"}
    ok(lam_sach_hang(giu) == giu, "giữ launch thật")
    cfg = {"claude_accounts": [{"name": "acc8", "cli": "opencode", "model": "rieng"}]}
    them = gop_tai_khoan(cfg, [{"name": "acc8", "model": "mau"}, {"name": "acc3", "launch": "claude3"}])
    ok(them == ["acc3"], "chỉ thêm hàng thiếu")
    ok(cfg["claude_accounts"][1]["model"] == "rieng", "không sửa hàng đã có")
    ok("launch" not in cfg["claude_accounts"][0], "hàng thêm đã làm sạch")
    with tempfile.TemporaryDirectory() as h:
        c = {}
        ok(dat_goc_du_an(c, h).startswith("⚠") and "workspace_root" not in c, "thiếu ~/projects ⟹ không đặt")
        os.mkdir(os.path.join(h, "projects"))
        ok(dat_goc_du_an(c, h) and c["workspace_root"] == os.path.join(h, "projects"), "có ~/projects ⟹ đặt")
        c2 = {"workspace_root": "/khac"}
        ok(dat_goc_du_an(c2, h) is None and c2["workspace_root"] == "/khac", "đã đặt ⟹ giữ")
        cfg = {"claude_accounts": [{"name": "acc1"}, {"name": "acc3", "config_dir": os.path.join(h, ".claude-acc3")},
                                   {"name": "acc8", "cli": "opencode", "config_dir": os.path.join(h, ".x8")}]}
        ok(thu_muc_acc(cfg, h) == 2, "tạo thư mục + liên kết acc3")
        ok(os.path.islink(os.path.join(h, ".claude-acc3", "projects")), "liên kết projects có thật")
        ok(not os.path.exists(os.path.join(h, ".x8")), "không đụng acc opencode")
        ok(thu_muc_acc(cfg, h) == 0, "chạy lại ⟹ không tạo gì (bất biến)")
    ok(can_cli({"claude_accounts": [{"name": "a"}, {"name": "b", "cli": "opencode"}]}) == ["claude", "opencode"], "cần cả hai CLI")
    ok(can_cli({}) == ["claude"], "không có acc ⟹ acc mặc định cần claude")
    print(f"== {tong - hong}/{tong} ca đúng ==")
    return 1 if hong else 0


def main(argv):
    if len(argv) >= 2 and argv[1] == "tu-kiem":
        return tu_kiem()
    if len(argv) >= 3 and argv[1] == "cau-hinh":
        return cmd_cau_hinh(argv[2], argv[3] if len(argv) > 3 else None)
    if len(argv) == 3 and argv[1] in ("thu-muc-acc", "can-cli"):
        if not os.path.isfile(argv[2]):
            print(f"KHÔNG ĐO ĐƯỢC: chưa có {argv[2]}")
            return 2
        cfg = doc_json(argv[2])
        if argv[1] == "thu-muc-acc":
            print(f"  thư mục/liên kết acc mới: {thu_muc_acc(cfg)}")
        else:
            print(" ".join(can_cli(cfg)))
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
