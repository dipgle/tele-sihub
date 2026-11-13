#!/usr/bin/env python3
"""Phần CẤU HÌNH của trình cài Linux (`install_update_linux.sh`) — đi trong gói ở `kem/`.

Quang 13/11: «chỉnh nốt để gói cài đặt không phải chỉnh sửa gì mà chạy được ngay».
Mọi thứ hôm ấy phải làm TAY trên Ubuntu — đặt gốc dự án, chép 9 hàng tài khoản (bỏ
alias zsh `claude2`/`claude3`), tạo thư mục acc + liên kết `projects` — làm ở đây.

  python3 huba_cai_linux.py cau-hinh <huba.config.json> <tai-khoan.mau.json>
  python3 huba_cai_linux.py thu-muc-acc <huba.config.json>
  python3 huba_cai_linux.py can-cli <huba.config.json>      # in: claude / opencode
  python3 huba_cai_linux.py goc-workspace <huba.config.json>  # in: đường tuyệt đối (hoặc rỗng)
  python3 huba_cai_linux.py luat-goc <huba.config.json> <kem/CLAUDE.md>
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


def tep_cau_hinh_claude(a, home=HOME):
    """Tệp cấu hình NGƯỜI DÙNG của một acc claude: có `config_dir` ⟹ `<dir>/.claude.json`,
    acc mặc định ⟹ `~/.claude.json` (CLI đặt nó cạnh, không trong, `~/.claude`). Thuần."""
    d = (a.get("config_dir") or "").strip()
    if not d:
        return os.path.join(home, ".claude.json")
    d = mo_rong(d)
    d = d if os.path.isabs(d) else os.path.join(home, d)
    return os.path.join(d, ".claude.json")


def dat_mcp_claude(cfg, mcp_bin, home=HOME):
    """MCP codetrail cho MỌI acc claude — Quang 13/11 «trên ubuntu tài khoản claude sao
    không nhận mcp codetrail». Đo Ubuntu: không có `~/projects/.mcp.json` (Mac có, trỏ
    binary macOS), `.claude.json` của acc3/acc6 `mcpServers: []`, `claude mcp list` không
    có `project-agent` — gói chỉ chèn nó cho opencode (`accounts::chen_mcp_codetrail`).
    Ghi `mcpServers.project-agent` phạm vi NGƯỜI DÙNG trỏ `kem/bin/project-agent`: hàng
    đã có thì CHỈ thay `command` (giữ `env`/`args` khác), chưa có thì thêm. Trả danh sách
    tệp đã ghi."""
    root = os.path.join(home, "projects")
    ghi = []
    for a in cfg.get("claude_accounts", []) or [{}]:
        if (a.get("cli") or "claude").strip().lower() == "opencode":
            continue
        p = tep_cau_hinh_claude(a, home)
        d = doc_json(p) if os.path.isfile(p) else {}
        if not isinstance(d, dict):
            print(f"  ⚠ {p} không phải object JSON — KHÔNG ghi")
            continue
        srv = d.setdefault("mcpServers", {})
        cu = srv.get("project-agent")
        moi = dict(cu) if isinstance(cu, dict) else {}
        moi["type"] = "stdio"
        moi["command"] = mcp_bin
        env = dict(moi.get("env") or {})
        env.setdefault("PROJECTS_ROOT", root)
        moi["env"] = env
        if moi == cu:
            continue
        srv["project-agent"] = moi
        os.makedirs(os.path.dirname(p), exist_ok=True)
        ghi_json(p, d)
        ghi.append(p)
    return ghi


def can_cli(cfg):
    can = set()
    for a in cfg.get("claude_accounts", []) or [{}]:
        can.add("opencode" if (a.get("cli") or "").strip().lower() == "opencode" else "claude")
    return sorted(can)


def goc_workspace(cfg, cfg_dir, home=HOME):
    """Gốc workspace mà cấu hình đã khai, dạng ĐƯỜNG TUYỆT ĐỐI và có thật.

    Cùng phép `config::absolutize`: `~/…` mở theo nhà, đường tương đối tính từ
    thư mục chứa `huba.config.json` (đó là `hub_home` trong config.rs). Không
    khai / không phải thư mục ⟹ "" — người gọi nói ra, không đoán. KHÔNG tự dò
    `~/projects`: một bản sao thứ hai của `find_workspace_root` là chỗ để lệch.
    """
    g = (cfg.get("workspace_root") or "").strip()
    if not g:
        return ""
    g = mo_rong(g)
    if not os.path.isabs(g):
        g = os.path.join(cfg_dir, g)
    if not os.path.isdir(g):
        return ""
    return os.path.abspath(g)


def dat_luat_goc(goc, mau_p):
    """Đặt `<gốc>/CLAUDE.md` = bản sạch của gói — CHỈ khi chưa có.

    Quang 13/11: *"sao không giữ cấu trúc file md như hiện tại trong thư mục
    /projects?"*. Nên tệp cài ra đúng tên/tình trạng ấy: `CLAUDE.md` ở gốc
    workspace, claude tự nạp. Chủ máy đã có tệp (dù khác bản sạch) ⟹ GIỮ, chỉ
    nói bản sạch nằm đâu: ghi đè luật của người ta là việc huba không được làm.
    Chạy lại nhiều lần ⟹ không đổi gì. Trả (mã, câu nói); mã: moi · giong ·
    giu · khong-co-goc · khong-co-ban-sach.
    """
    if not goc:
        return "khong-co-goc", ("⚠ chưa xác định được gốc workspace (chưa khai `workspace_root`, hoặc thư mục "
                                "chưa gắn — /run/media chỉ có sau khi đăng nhập) — KHÔNG đặt CLAUDE.md; "
                                "gắn đĩa/khai gốc rồi chạy lại trình cài")
    if not os.path.isfile(mau_p):
        return "khong-co-ban-sach", f"❌ gói thiếu bản sạch {mau_p}"
    dich = os.path.join(goc, "CLAUDE.md")
    with open(mau_p, "rb") as f:
        sach = f.read()
    if not os.path.exists(dich):
        tmp = dich + ".huba-cai-tmp"
        with open(tmp, "wb") as f:
            f.write(sach)
        os.replace(tmp, dich)
        return "moi", f"tạo {dich} (bản sạch của gói)"
    with open(dich, "rb") as f:
        dang_co = f.read()
    if dang_co == sach:
        return "giong", f"{dich} = bản sạch của gói"
    return "giu", (f"giữ {dich} (bản của chủ máy — KHÔNG ghi đè) · "
                   f"bản sạch của gói: {mau_p}")


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
    with tempfile.TemporaryDirectory() as h:
        bin_ = os.path.join(h, "kem", "bin", "project-agent")
        cfg = {"claude_accounts": [{"name": "acc1"}, {"name": "acc3", "config_dir": os.path.join(h, ".claude-acc3")},
                                   {"name": "acc8", "cli": "opencode", "config_dir": os.path.join(h, ".x8")}]}
        os.makedirs(os.path.join(h, ".claude-acc3"))
        # acc3 đã có tệp, mang hàng project-agent trỏ binary macOS + một máy chủ khác.
        ghi_json(os.path.join(h, ".claude-acc3", ".claude.json"),
                 {"userID": "x", "mcpServers": {"project-agent": {"type": "stdio", "command": "/Users/mac/project-agent",
                                                                  "env": {"PROJECTS_ROOT": "/khac", "X": "1"}},
                                                "vault": {"command": "node"}}})
        da = dat_mcp_claude(cfg, bin_, h)
        ok(sorted(da) == sorted([os.path.join(h, ".claude.json"), os.path.join(h, ".claude-acc3", ".claude.json")]),
           "ghi acc mặc định (~/.claude.json) + acc3, bỏ acc opencode")
        a3 = doc_json(os.path.join(h, ".claude-acc3", ".claude.json"))
        pa = a3["mcpServers"]["project-agent"]
        ok(pa["command"] == bin_ and pa["env"] == {"PROJECTS_ROOT": "/khac", "X": "1"},
           "hàng cũ: chỉ thay command, giữ env")
        ok(a3["userID"] == "x" and "vault" in a3["mcpServers"], "không đụng khoá/máy chủ khác")
        a1 = doc_json(os.path.join(h, ".claude.json"))
        ok(a1["mcpServers"]["project-agent"] == {"type": "stdio", "command": bin_,
                                                 "env": {"PROJECTS_ROOT": os.path.join(h, "projects")}},
           "chưa có tệp ⟹ tạo hàng mới")
        ok(dat_mcp_claude(cfg, bin_, h) == [], "chạy lại ⟹ không ghi gì (bất biến)")
        ok(not os.path.exists(os.path.join(h, ".x8")), "không đụng acc opencode (mcp)")
    ok(can_cli({"claude_accounts": [{"name": "a"}, {"name": "b", "cli": "opencode"}]}) == ["claude", "opencode"], "cần cả hai CLI")
    ok(can_cli({}) == ["claude"], "không có acc ⟹ acc mặc định cần claude")
    with tempfile.TemporaryDirectory() as h:
        g = os.path.join(h, "ws")
        os.mkdir(g)
        ok(goc_workspace({"workspace_root": g}, h) == g, "gốc tuyệt đối")
        ok(goc_workspace({"workspace_root": "ws"}, h) == g, "gốc tương đối tính từ hub_home")
        ok(goc_workspace({"workspace_root": os.path.join(h, "khong-co")}, h) == "", "không có thư mục ⟹ rỗng")
        ok(goc_workspace({}, h) == "", "chưa khai ⟹ rỗng")
        mau = os.path.join(h, "kem-CLAUDE.md")
        with open(mau, "w", encoding="utf-8") as f:
            f.write("# luat sach\n")
        ok(dat_luat_goc(g, mau)[0] == "moi"
           and open(os.path.join(g, "CLAUDE.md"), encoding="utf-8").read() == "# luat sach\n",
           "chưa có ⟹ đặt bản sạch")
        ok(dat_luat_goc(g, mau)[0] == "giong", "chạy lại ⟹ không đổi (bất biến)")
        with open(os.path.join(g, "CLAUDE.md"), "w", encoding="utf-8") as f:
            f.write("# cua toi\n")
        ok(dat_luat_goc(g, mau)[0] == "giu"
           and open(os.path.join(g, "CLAUDE.md"), encoding="utf-8").read() == "# cua toi\n",
           "đã sửa ⟹ GIỮ, không ghi đè")
        ok(dat_luat_goc("", mau)[0] == "khong-co-goc", "không gốc ⟹ không đặt")
        ok(dat_luat_goc(g, os.path.join(h, "thieu.md"))[0] == "khong-co-ban-sach", "thiếu bản sạch ⟹ báo")
    print(f"== {tong - hong}/{tong} ca đúng ==")
    return 1 if hong else 0


def main(argv):
    if len(argv) >= 2 and argv[1] == "tu-kiem":
        return tu_kiem()
    if len(argv) >= 3 and argv[1] == "cau-hinh":
        return cmd_cau_hinh(argv[2], argv[3] if len(argv) > 3 else None)
    if len(argv) == 3 and argv[1] in ("thu-muc-acc", "can-cli", "goc-workspace"):
        if not os.path.isfile(argv[2]):
            print(f"KHÔNG ĐO ĐƯỢC: chưa có {argv[2]}")
            return 2
        cfg = doc_json(argv[2])
        if argv[1] == "thu-muc-acc":
            print(f"  thư mục/liên kết acc mới: {thu_muc_acc(cfg)}")
        elif argv[1] == "can-cli":
            print(" ".join(can_cli(cfg)))
        else:
            print(goc_workspace(cfg, os.path.dirname(os.path.abspath(argv[2]))))
        return 0
    if len(argv) == 4 and argv[1] == "mcp-claude":
        if not os.path.isfile(argv[2]) or not os.path.isfile(argv[3]):
            print(f"KHÔNG ĐO ĐƯỢC: thiếu {argv[2]} hoặc {argv[3]}")
            return 2
        da = dat_mcp_claude(doc_json(argv[2]), argv[3])
        print(f"  MCP codetrail cho claude: {len(da)} tệp ghi" + "".join(f"\n    · {p}" for p in da))
        return 0
    if len(argv) == 4 and argv[1] == "luat-goc":
        if not os.path.isfile(argv[2]):
            print(f"KHÔNG ĐO ĐƯỢC: chưa có {argv[2]}")
            return 2
        goc = goc_workspace(doc_json(argv[2]), os.path.dirname(os.path.abspath(argv[2])))
        ma, cau = dat_luat_goc(goc, argv[3])
        print(f"  {cau}")
        return 1 if ma == "khong-co-ban-sach" else 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
