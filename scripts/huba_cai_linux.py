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
    # Xếp tên TỰ NHIÊN (acc2 < acc10) — xếp theo chữ đặt acc10 ngay sau acc1 (đo 14/11).
    cfg["claude_accounts"] = sorted(cfg.get("claude_accounts", []), key=lambda a: ten_tu_nhien(a.get("name", "")))
    return them


def dat_goc_du_an(cfg, home=HOME, chon=""):
    """Đặt `workspace_root`. Trả câu nói (hoặc None). Đọc/ghi đĩa (tạo thư mục được chọn).

    `chon` (người cài CHỌN thư mục làm việc — env `HUBA_GOC`; Quang 21/11 «khi cài cần chọn luôn thư
    mục làm việc») THẮNG mọi thứ: tạo thư mục nếu chưa có rồi đặt. Không tạo được ⟹ KHÔNG đặt, báo ⚠.
    Không chọn: `workspace_root` đã khai ⟹ giữ; trống và có `~/projects` ⟹ đặt nó.
    """
    chon = (chon or "").strip()
    if chon:
        g = os.path.abspath(os.path.expanduser(chon))
        try:
            os.makedirs(g, exist_ok=True)
        except OSError as e:
            return f"⚠ không tạo được thư mục làm việc {g} ({e}) — giữ nguyên cấu hình"
        if cfg.get("workspace_root") == g:
            return None
        cfg["workspace_root"] = g
        return f"gốc dự án = {g} (do người cài chọn)"
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
    goc = dat_goc_du_an(cfg, chon=os.environ.get("HUBA_GOC", ""))
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


def ten_tu_nhien(ten):
    """Khoá xếp tên tự nhiên: khúc số so theo giá trị. Thuần."""
    return [(0, int(t), "") if t.isdigit() else (1, 0, t) for t in re.split(r"(\d+)", ten) if t != ""]


def cli_cua(a):
    """CLI của một hàng: `claude` (mặc định) · `opencode` · `codex`. MỘT chỗ đọc, để
    CLI thứ ba không lọt vào bước chỉ dành cho claude (thư mục + `projects` + MCP). Thuần."""
    return (a.get("cli") or "claude").strip().lower() or "claude"


def lenh_lien_ket_nt(dich, nguon):
    """Lệnh tạo JUNCTION trên Windows: `mklink /J <dich> <nguon>`. Không cần quyền admin hay
    Developer Mode (symlink thì cần) — gói Windows dùng tệp này cho `projects` của từng acc. Thuần."""
    return ["cmd", "/c", "mklink", "/J", dich, nguon]


def lien_ket(dich, nguon):
    """`dich` → thư mục `nguon`: symlink (Linux/macOS) · junction (Windows)."""
    if os.name == "nt":
        import subprocess
        r = subprocess.run(lenh_lien_ket_nt(dich, nguon), capture_output=True, text=True)
        if r.returncode != 0:
            raise OSError(f"mklink /J hỏng (mã {r.returncode}): {r.stdout.strip()} {r.stderr.strip()}")
    else:
        os.symlink(nguon, dich)


def thu_muc_acc(cfg, home=HOME, tao_lien_ket=None):
    """Mỗi acc `claude`: thư mục cấu hình + `projects` → `~/.claude/projects` (sổ phiên
    chung). Không đụng thứ đã có. Trả số mục đã tạo. `tao_lien_ket` chỉ để bài kiểm thay."""
    tao_lien_ket = tao_lien_ket or lien_ket
    chung = os.path.join(home, ".claude", "projects")
    os.makedirs(chung, exist_ok=True)
    tao = 0
    for a in cfg.get("claude_accounts", []):
        if cli_cua(a) != "claude":
            continue
        d = mo_rong(a.get("config_dir") or "~/.claude")
        d = d if os.path.isabs(d) else os.path.join(home, d)
        if not os.path.isdir(d):
            os.makedirs(d)
            tao += 1
        p = os.path.join(d, "projects")
        if os.path.realpath(d) != os.path.realpath(os.path.join(home, ".claude")) and not os.path.lexists(p):
            tao_lien_ket(p, chung)
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


def dat_mcp_claude(cfg, mcp_bin, home=HOME, goc=""):
    """MCP codetrail cho MỌI acc claude — Quang 13/11 «trên ubuntu tài khoản claude sao
    không nhận mcp codetrail». Đo Ubuntu: không có `~/projects/.mcp.json` (Mac có, trỏ
    binary macOS), `.claude.json` của acc3/acc6 `mcpServers: []`, `claude mcp list` không
    có `project-agent` — gói chỉ chèn nó cho opencode (`accounts::chen_mcp_codetrail`).
    Ghi `mcpServers.project-agent` phạm vi NGƯỜI DÙNG trỏ `kem/bin/project-agent`: hàng
    đã có thì CHỈ thay `command` (giữ `env`/`args` khác), chưa có thì thêm. Trả danh sách
    tệp đã ghi.

    `PROJECTS_ROOT` THEO THƯ MỤC LÀM VIỆC ĐÃ CHỌN (`goc` = `workspace_root` của cấu hình; Quang 21/11
    «các thành phần là liên kết động theo thư mục chọn») — không ghi cứng `~/projects`. Hàng cũ còn
    đúng giá trị mặc định cũ (`~/projects`) mà gốc đã chọn khác ⟹ chuyển theo gốc; giá trị chủ máy tự
    đặt (khác mặc định) thì GIỮ."""
    mac_dinh = os.path.join(home, "projects")
    root = goc or mac_dinh
    ghi = []
    for a in cfg.get("claude_accounts", []) or [{}]:
        if cli_cua(a) != "claude":
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
        if env.get("PROJECTS_ROOT") in (None, "", mac_dinh):
            env["PROJECTS_ROOT"] = root
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
        can.add(cli_cua(a))
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


def _cay(p):
    """{đường tương đối: bytes} của một tệp hoặc cả cây — để so «giống hệt». Thuần (đọc)."""
    if os.path.isfile(p):
        with open(p, "rb") as f:
            return {"": f.read()}
    out = {}
    for goc, _, tep in os.walk(p):
        for t in tep:
            if t == ".DS_Store":
                continue
            q = os.path.join(goc, t)
            with open(q, "rb") as f:
                out[os.path.relpath(q, p)] = f.read()
    return out


def dat_skill_goc(goc, nguon):
    """Đặt `<gốc>/.claude/{skills,agents}/<mục>` từ `kem/claude` — cùng luật với
    [`dat_luat_goc`]: chưa có ⟹ chép; giống hệt ⟹ để yên; KHÁC ⟹ GIỮ bản chủ máy, nói ra.

    Quang 13/11: *"Tại sao skill rất nhiều trong projects không đóng gói cùng? Lại đúng
    mỗi file claude.md"* — luật gọi tên `quality-gate`, `tra-cuu-haiku`… mà máy Linux
    không có. Trả danh sách (mã, câu); mã: moi · giong · giu · khong-co-goc · khong-co-ban-sach.
    """
    if not goc:
        return [("khong-co-goc", "⚠ chưa xác định được gốc workspace — KHÔNG đặt skill/tác tử")]
    if not os.path.isdir(nguon):
        return [("khong-co-ban-sach", f"❌ gói thiếu {nguon}")]
    kq = []
    for loai in ("skills", "agents"):
        src = os.path.join(nguon, loai)
        if not os.path.isdir(src):
            kq.append(("khong-co-ban-sach", f"❌ gói thiếu {src}"))
            continue
        for ten in sorted(os.listdir(src)):
            if ten == ".DS_Store":
                continue
            s = os.path.join(src, ten)
            d = os.path.join(goc, ".claude", loai, ten)
            if not os.path.exists(d):
                os.makedirs(os.path.dirname(d), exist_ok=True)
                tmp = d + ".huba-cai-tmp"
                if os.path.isdir(s):
                    shutil.rmtree(tmp, ignore_errors=True)
                    shutil.copytree(s, tmp, ignore=shutil.ignore_patterns(".DS_Store"))
                else:
                    shutil.copy2(s, tmp)
                os.replace(tmp, d)
                kq.append(("moi", f"tạo {d}"))
            elif _cay(d) == _cay(s):
                kq.append(("giong", f"{d} = bản của gói"))
            else:
                kq.append(("giu", f"giữ {d} (bản của chủ máy — KHÔNG ghi đè) · bản của gói: {s}"))
    return kq


SU_KIEN_NHAC = ("Stop", "UserPromptSubmit")


def _lenh_nhac(script):
    """Dòng lệnh hook: trình thông dịch + đường script (dấu ngoặc kép — đường có thể có dấu cách).

    Windows không có `python3` chắc chắn trên PATH ⟹ dùng chính Python đang chạy trình cài.
    """
    py = f'"{sys.executable}"' if os.name == "nt" else "python3"
    return f'{py} "{script}"'


def _co_hook_nhac(muc):
    return any("nhac-chuyen-phien.py" in str(h.get("command", ""))
               for e in muc if isinstance(e, dict) for h in (e.get("hooks") or []) if isinstance(h, dict))


def dat_hook_nhac(goc, script):
    """Đăng ký hook nhắc chuyển phiên (≥ 50 % ngữ cảnh) vào `<gốc>/.claude/settings.json`.

    Gói mang `kem/scripts/nhac-chuyen-phien.py` nhưng claude chỉ gọi hook khi nó được KHAI trong
    settings — thiếu khai thì script nằm đó mà không ai chạy, và luật «chuyển phiên khi quá 50 %»
    trong CLAUDE.md chỉ còn trông vào trí nhớ của phiên (Quang 21/11 «làm cả hai đi»).
    GỘP, không đè: giữ mọi khoá khác; sự kiện đã có hook `nhac-chuyen-phien.py` (kể cả đường khác) thì
    để yên, không thêm lần hai. Tệp settings hỏng/không phải JSON object ⟹ KHÔNG ghi, báo `hong`.
    Trả (mã, câu); mã: moi · them · giong · hong · khong-co-goc · khong-co-ban-sach.
    """
    if not goc:
        return "khong-co-goc", "⚠ chưa xác định được gốc workspace — KHÔNG đăng ký hook nhắc chuyển phiên"
    if not os.path.isfile(script):
        return "khong-co-ban-sach", f"❌ gói thiếu {script}"
    dich = os.path.join(goc, ".claude", "settings.json")
    co_tep = os.path.exists(dich)
    if co_tep:
        try:
            d = doc_json(dich)
        except (OSError, ValueError) as e:
            return "hong", f"❌ {dich} không đọc được ({e}) — KHÔNG sửa; đăng ký tay hook `{_lenh_nhac(script)}` cho {'/'.join(SU_KIEN_NHAC)}"
        if not isinstance(d, dict):
            return "hong", f"❌ {dich} không phải JSON object — KHÔNG sửa"
    else:
        d = {}
    hooks = d.setdefault("hooks", {})
    if not isinstance(hooks, dict):
        return "hong", f"❌ {dich}: khoá `hooks` không phải object — KHÔNG sửa"
    them = []
    for sk in SU_KIEN_NHAC:
        muc = hooks.setdefault(sk, [])
        if not isinstance(muc, list):
            return "hong", f"❌ {dich}: `hooks.{sk}` không phải danh sách — KHÔNG sửa"
        if not _co_hook_nhac(muc):
            muc.append({"hooks": [{"type": "command", "command": _lenh_nhac(script), "timeout": 10}]})
            them.append(sk)
    if not them:
        return "giong", f"{dich} đã khai hook nhắc chuyển phiên ({'/'.join(SU_KIEN_NHAC)})"
    os.makedirs(os.path.dirname(dich), exist_ok=True)
    ghi_json(dich, d)
    return ("them" if co_tep else "moi"), (f"{'thêm vào' if co_tep else 'tạo'} {dich}: hook nhắc chuyển phiên cho "
                                           + "/".join(them) + " (≥ 50 % ngữ cảnh)")


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
        # Người cài CHỌN thư mục: tạo nếu chưa có, đặt, và THẮNG cả khi cấu hình đã khai chỗ khác.
        chon = os.path.join(h, "lam-viec", "du-an")
        c3 = {}
        ok(dat_goc_du_an(c3, h, chon).startswith("gốc dự án =") and c3["workspace_root"] == chon
           and os.path.isdir(chon), "chọn thư mục chưa có ⟹ tạo + đặt")
        ok(dat_goc_du_an(c3, h, chon) is None, "chọn lại đúng thư mục ấy ⟹ không đổi (bất biến)")
        c4 = {"workspace_root": "/khac"}
        dat_goc_du_an(c4, h, chon)
        ok(c4["workspace_root"] == chon, "chọn thắng cấu hình đã khai (đối chứng ngược của «đã đặt ⟹ giữ»)")
        tep = os.path.join(h, "la-tep")
        with open(tep, "w", encoding="utf-8") as f:
            f.write("x")
        c5 = {"workspace_root": "/khac"}
        ok(dat_goc_du_an(c5, h, os.path.join(tep, "con")).startswith("⚠") and c5["workspace_root"] == "/khac",
           "chọn đường không tạo được ⟹ báo ⚠, KHÔNG đổi cấu hình")
        cfg = {"claude_accounts": [{"name": "acc1"}, {"name": "acc3", "config_dir": os.path.join(h, ".claude-acc3")},
                                   {"name": "acc8", "cli": "opencode", "config_dir": os.path.join(h, ".x8")},
                                   {"name": "acc10", "cli": "codex", "config_dir": os.path.join(h, ".x10")}]}
        ok(thu_muc_acc(cfg, h) == 2, "tạo thư mục + liên kết acc3")
        ok(os.path.islink(os.path.join(h, ".claude-acc3", "projects")), "liên kết projects có thật")
        ok(not os.path.exists(os.path.join(h, ".x8")), "không đụng acc opencode")
        ok(not os.path.exists(os.path.join(h, ".x10")), "không đụng acc codex (không `projects` của Claude)")
        ok(thu_muc_acc(cfg, h) == 0, "chạy lại ⟹ không tạo gì (bất biến)")
    ok(lenh_lien_ket_nt("D", "S") == ["cmd", "/c", "mklink", "/J", "D", "S"],
       "Windows: junction (mklink /J), không phải symlink")
    with tempfile.TemporaryDirectory() as h:
        goi = []
        cfg = {"claude_accounts": [{"name": "acc3", "config_dir": os.path.join(h, ".claude-acc3")}]}
        thu_muc_acc(cfg, h, tao_lien_ket=lambda d, n: goi.append((d, n)))
        ok(goi == [(os.path.join(h, ".claude-acc3", "projects"), os.path.join(h, ".claude", "projects"))],
           "liên kết đi qua MỘT hàm thay được (Windows dùng junction)")
    with tempfile.TemporaryDirectory() as h:
        bin_ = os.path.join(h, "kem", "bin", "project-agent")
        cfg = {"claude_accounts": [{"name": "acc1"}, {"name": "acc3", "config_dir": os.path.join(h, ".claude-acc3")},
                                   {"name": "acc8", "cli": "opencode", "config_dir": os.path.join(h, ".x8")},
                                   {"name": "acc10", "cli": "codex", "config_dir": os.path.join(h, ".x10")}]}
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
        # Gốc đã chọn: PROJECTS_ROOT theo gốc; hàng còn mặc định cũ chuyển theo, giá trị chủ máy đặt thì giữ.
        goc_chon = os.path.join(h, "lam-viec")
        dat_mcp_claude(cfg, bin_, h, goc=goc_chon)
        ok(doc_json(os.path.join(h, ".claude.json"))["mcpServers"]["project-agent"]["env"]["PROJECTS_ROOT"] == goc_chon,
           "MCP: hàng còn mặc định ~/projects ⟹ chuyển theo gốc đã chọn")
        ok(doc_json(os.path.join(h, ".claude-acc3", ".claude.json"))["mcpServers"]["project-agent"]["env"]["PROJECTS_ROOT"] == "/khac",
           "MCP: PROJECTS_ROOT chủ máy tự đặt ⟹ GIỮ (đối chứng ngược)")
        ok(dat_mcp_claude(cfg, bin_, h, goc=goc_chon) == [], "MCP: chạy lại với gốc đã chọn ⟹ không ghi gì")
        ok(not os.path.exists(os.path.join(h, ".x8")), "không đụng acc opencode (mcp)")
        ok(not os.path.exists(os.path.join(h, ".x10")), "không đụng acc codex (mcp)")
    with tempfile.TemporaryDirectory() as h:
        kem = os.path.join(h, "kem", "claude")
        os.makedirs(os.path.join(kem, "skills", "quality-gate"))
        os.makedirs(os.path.join(kem, "agents"))
        with open(os.path.join(kem, "skills", "quality-gate", "SKILL.md"), "w", encoding="utf-8") as f:
            f.write("# qg\n")
        with open(os.path.join(kem, "agents", "tra-cuu-haiku.md"), "w", encoding="utf-8") as f:
            f.write("# haiku\n")
        g = os.path.join(h, "ws")
        os.mkdir(g)
        ma = [m for m, _ in dat_skill_goc(g, kem)]
        ok(ma == ["moi", "moi"]
           and open(os.path.join(g, ".claude", "skills", "quality-gate", "SKILL.md"), encoding="utf-8").read() == "# qg\n"
           and os.path.isfile(os.path.join(g, ".claude", "agents", "tra-cuu-haiku.md")),
           "skill + tác tử: chưa có ⟹ chép vào <gốc>/.claude")
        ok([m for m, _ in dat_skill_goc(g, kem)] == ["giong", "giong"], "skill: chạy lại ⟹ không đổi (bất biến)")
        with open(os.path.join(g, ".claude", "skills", "quality-gate", "SKILL.md"), "w", encoding="utf-8") as f:
            f.write("# cua toi\n")
        ok(dat_skill_goc(g, kem)[0][0] == "giu"
           and open(os.path.join(g, ".claude", "skills", "quality-gate", "SKILL.md"), encoding="utf-8").read() == "# cua toi\n",
           "skill chủ máy đã sửa ⟹ GIỮ, không ghi đè")
        ok(dat_skill_goc("", kem)[0][0] == "khong-co-goc", "skill: không gốc ⟹ không đặt")
    ok(can_cli({"claude_accounts": [{"name": "a"}, {"name": "b", "cli": "opencode"}]}) == ["claude", "opencode"], "cần cả hai CLI")
    ok(can_cli({"claude_accounts": [{"name": "a"}, {"name": "c", "cli": "codex"}]}) == ["claude", "codex"],
       "acc codex ⟹ cần codex, không đọc thành claude")
    ok(can_cli({}) == ["claude"], "không có acc ⟹ acc mặc định cần claude")
    c = {"claude_accounts": [{"name": "acc1"}, {"name": "acc2"}, {"name": "acc9"}]}
    gop_tai_khoan(c, [{"name": "acc10", "cli": "codex"}])
    ok([a["name"] for a in c["claude_accounts"]] == ["acc1", "acc2", "acc9", "acc10"],
       "thêm acc10 ⟹ đứng SAU acc9 (xếp tự nhiên, không theo chữ)")
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
    with tempfile.TemporaryDirectory() as h:
        g = os.path.join(h, "ws")
        os.mkdir(g)
        sc = os.path.join(h, "kem", "scripts", "nhac-chuyen-phien.py")
        os.makedirs(os.path.dirname(sc))
        with open(sc, "w", encoding="utf-8") as f:
            f.write("# hook\n")
        st = os.path.join(g, ".claude", "settings.json")
        ma, _ = dat_hook_nhac(g, sc)
        d = doc_json(st)
        ok(ma == "moi" and all(_co_hook_nhac(d["hooks"][s]) for s in SU_KIEN_NHAC),
           "hook nhắc: chưa có settings ⟹ tạo, đủ Stop + UserPromptSubmit")
        ok(sc in d["hooks"]["Stop"][0]["hooks"][0]["command"] and d["hooks"]["Stop"][0]["hooks"][0]["timeout"] == 10,
           "hook nhắc: lệnh trỏ đúng script của gói")
        ok(dat_hook_nhac(g, sc)[0] == "giong", "hook nhắc: chạy lại ⟹ không đổi (bất biến)")
        ok(len(doc_json(st)["hooks"]["Stop"]) == 1, "hook nhắc: chạy lại ⟹ không nhân đôi")
        # Đối chứng ngược: gỡ một sự kiện ⟹ phải thêm lại đúng sự kiện ấy (phép đo đổi được trạng thái).
        d = doc_json(st)
        del d["hooks"]["Stop"]
        ghi_json(st, d)
        ma, cau = dat_hook_nhac(g, sc)
        ok(ma == "them" and "Stop" in cau and _co_hook_nhac(doc_json(st)["hooks"]["Stop"]),
           "hook nhắc: thiếu Stop ⟹ thêm lại (đối chứng ngược)")
        # Settings của chủ máy có khoá khác + hook khác: gộp, không đè.
        ghi_json(st, {"permissions": {"defaultMode": "auto"},
                      "hooks": {"Stop": [{"hooks": [{"type": "command", "command": "echo cua-toi"}]}]}})
        ma, _ = dat_hook_nhac(g, sc)
        d = doc_json(st)
        ok(ma == "them" and d["permissions"] == {"defaultMode": "auto"}
           and d["hooks"]["Stop"][0]["hooks"][0]["command"] == "echo cua-toi" and _co_hook_nhac(d["hooks"]["Stop"]),
           "hook nhắc: gộp vào settings của chủ máy, giữ khoá + hook khác")
        # Đã khai bằng đường khác ⟹ GIỮ, không thêm lần hai.
        ghi_json(st, {"hooks": {s: [{"hooks": [{"type": "command", "command": "python3 /dung-khac/nhac-chuyen-phien.py"}]}]
                                for s in SU_KIEN_NHAC}})
        ok(dat_hook_nhac(g, sc)[0] == "giong" and len(doc_json(st)["hooks"]["Stop"]) == 1,
           "hook nhắc: đã khai đường khác ⟹ GIỮ, không nhân đôi")
        # Settings hỏng ⟹ KHÔNG ghi (tệp giữ nguyên từng byte), báo `hong`.
        with open(st, "w", encoding="utf-8") as f:
            f.write("{ day khong phai json")
        ma, _ = dat_hook_nhac(g, sc)
        ok(ma == "hong" and open(st, encoding="utf-8").read() == "{ day khong phai json",
           "hook nhắc: settings hỏng ⟹ không ghi, báo hỏng")
        ok(dat_hook_nhac("", sc)[0] == "khong-co-goc", "hook nhắc: không gốc ⟹ không đặt")
        ok(dat_hook_nhac(g, os.path.join(h, "thieu.py"))[0] == "khong-co-ban-sach", "hook nhắc: thiếu script ⟹ báo")
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
    if len(argv) == 4 and argv[1] == "skill-goc":
        if not os.path.isfile(argv[2]):
            print(f"KHÔNG ĐO ĐƯỢC: chưa có {argv[2]}")
            return 2
        goc = goc_workspace(doc_json(argv[2]), os.path.dirname(os.path.abspath(argv[2])))
        kq = dat_skill_goc(goc, argv[3])
        for _, cau in kq:
            print(f"  {cau}")
        return 1 if any(m == "khong-co-ban-sach" for m, _ in kq) else 0
    if len(argv) == 4 and argv[1] == "mcp-claude":
        if not os.path.isfile(argv[2]) or not os.path.isfile(argv[3]):
            print(f"KHÔNG ĐO ĐƯỢC: thiếu {argv[2]} hoặc {argv[3]}")
            return 2
        cfg = doc_json(argv[2])
        da = dat_mcp_claude(cfg, argv[3], goc=goc_workspace(cfg, os.path.dirname(os.path.abspath(argv[2]))))
        print(f"  MCP codetrail cho claude: {len(da)} tệp ghi" + "".join(f"\n    · {p}" for p in da))
        return 0
    if len(argv) == 4 and argv[1] == "hook-nhac":
        if not os.path.isfile(argv[2]):
            print(f"KHÔNG ĐO ĐƯỢC: chưa có {argv[2]}")
            return 2
        goc = goc_workspace(doc_json(argv[2]), os.path.dirname(os.path.abspath(argv[2])))
        ma, cau = dat_hook_nhac(goc, argv[3])
        print(f"  {cau}")
        return 1 if ma in ("khong-co-ban-sach", "hong") else 0
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
