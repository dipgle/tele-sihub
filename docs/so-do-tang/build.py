"""Sinh docs/so-do-tang/index.html (SVG tự chứa) từ nguon.json + mã ở HEAD.

Phần HIỆN TẠI không vẽ tay: số dòng của mỗi bằng chứng và bảng đếm theo module
được TRÍCH từ `git show HEAD:<tệp>` mỗi lần dựng. Mẫu không còn trong mã ⇒ ĐỎ.

    python3 docs/so-do-tang/build.py          # dựng
    python3 docs/so-do-tang/build.py --kiem   # chỉ kiểm bản đã dựng còn khớp không

Mã thoát: 0 xong · 1 ĐỎ (bằng chứng mất / bản dựng cũ) · 2 KHÔNG ĐO ĐƯỢC (không đọc được mã).
"""
from __future__ import annotations

import html
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
WS = REPO.parent
OUT = HERE / "index.html"

RE_NHAC = re.compile(r"codex|opencode|claude", re.I)
RE_RE = re.compile(r"\.is_(codex|opencode|claude)\(\)")
RE_DINH_DANH = re.compile(
    r"^\s*(pub(\([a-z]+\))? )?(fn|struct|enum|const|static|mod|type) "
    r"[A-Za-z0-9_]*(claude|codex|opencode|model|cli|chatgpt)",
    re.I,
)

MAU = {
    "gt": ("#fff7ed", "#c2410c"),
    "ql": ("#eff6ff", "#1d4ed8"),
    "kho": ("#f0fdf4", "#15803d"),
    "ngoai": ("#f8fafc", "#475569"),
    "khac": ("#f8fafc", "#64748b"),
    "loi": ("#fef2f2", "#b91c1c"),
}
LANE_MAU = {"Chủ": "ngoai", "GT": "gt", "QL": "ql", "CV": "ql", "CLI": "ngoai"}


class KhongDoDuoc(Exception):
    """Không đọc được nguồn — trạng thái RIÊNG, không phải 'không có lỗi'."""


# ── Nguồn ──────────────────────────────────────────────────────────────────
def doc_head(tep: str) -> str:
    """`ws:` ⇒ đọc đĩa của kho chung; còn lại ⇒ `git show HEAD:` của huba."""
    if tep.startswith("ws:"):
        p = WS / tep[3:]
        try:
            return p.read_text()
        except OSError as e:
            raise KhongDoDuoc(f"{p}: {e}") from e
    r = subprocess.run(
        ["git", "-C", str(REPO), "show", f"HEAD:{tep}"], capture_output=True, text=True
    )
    if r.returncode != 0:
        raise KhongDoDuoc(f"git show HEAD:{tep}: {r.stderr.strip()}")
    return r.stdout


def head_sha() -> str:
    """Commit CUỐI chạm `rust/src` tới HEAD — thứ bản đồ trích từ đó. Không dùng HEAD trần:
    commit chính index.html làm HEAD đổi ⟹ bản đồ tự thành «cũ» mãi (đo 17/11)."""
    r = subprocess.run(
        ["git", "-C", str(REPO), "log", "-1", "--format=%h", "HEAD", "--", "rust/src"],
        capture_output=True,
        text=True,
    )
    if r.returncode != 0 or not r.stdout.strip():
        raise KhongDoDuoc("git log HEAD -- rust/src")
    return r.stdout.strip()


def tep_rust_head() -> list[str]:
    r = subprocess.run(
        ["git", "-C", str(REPO), "ls-tree", "-r", "--name-only", "HEAD", "rust/src"],
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        raise KhongDoDuoc("git ls-tree HEAD rust/src")
    return [t for t in r.stdout.split() if t.endswith(".rs")]


def giai_bang_chung(bang: dict, doc=doc_head) -> tuple[dict, list[str]]:
    """{khoá: {..., 'dong': [số dòng]}} + danh sách khoá KHÔNG tìm thấy."""
    ra, mat = {}, []
    for k, bc in bang.items():
        dong = [i for i, l in enumerate(doc(bc["tep"]).splitlines(), 1) if bc["mau"] in l]
        if not dong:
            mat.append(k)
        ra[k] = {**bc, "dong": dong}
    return ra, mat


def dem_module(lop: dict, doc=doc_head, tep_list=None) -> list[dict]:
    tep_list = tep_list if tep_list is not None else tep_rust_head()
    gt, ql = set(lop["giao_tiep"]), set(lop["quan_ly_model"])
    ra = []
    for t in tep_list:
        ten = Path(t).stem if not t.startswith("rust/src/bin/") else "bin/" + Path(t).stem
        if t.startswith("rust/src/adapters/"):
            ten = "adapters/" + Path(t).stem
        dong = doc(t).splitlines()
        nhac = sum(1 for l in dong if RE_NHAC.search(l))
        if not nhac:
            continue
        ra.append(
            {
                "module": ten,
                "tep": t,
                "lop": "gt" if ten in gt else "ql" if ten in ql else "khac",
                "nhac": nhac,
                "re": sum(len(RE_RE.findall(l)) for l in dong),
                "dinh_danh": sum(1 for l in dong if RE_DINH_DANH.search(l)),
            }
        )
    return sorted(ra, key=lambda m: (-m["nhac"], m["module"]))


# ── SVG ────────────────────────────────────────────────────────────────────
def e(s) -> str:
    return html.escape(str(s), quote=True)


def gói(chu: str, rong: int, co: int = 13) -> list[str]:
    """Ngắt dòng theo số ký tự ước lượng (≈ 0.58·cỡ chữ mỗi ký tự)."""
    toi_da = max(8, int(rong / (co * 0.58)))
    dong, hien = [], ""
    for tu in chu.split():
        if hien and len(hien) + 1 + len(tu) > toi_da:
            dong.append(hien)
            hien = tu
        else:
            hien = f"{hien} {tu}".strip()
    if hien:
        dong.append(hien)
    return dong or [""]


def hop(x, y, w, h, loai, dong: list[str], dam=False, co=13, vien=1.5) -> str:
    nen, net = MAU[loai]
    out = [f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" fill="{nen}" stroke="{net}" stroke-width="{vien}"/>']
    lh = co + 5
    y0 = y + h / 2 - (len(dong) - 1) * lh / 2 + co * 0.35
    for i, d in enumerate(dong):
        w8 = ' font-weight="600"' if dam and i == 0 else ""
        out.append(
            f'<text x="{x + w / 2}" y="{y0 + i * lh:.1f}" font-size="{co}" text-anchor="middle"{w8}>{e(d)}</text>'
        )
    return "\n".join(out)


def mui_ten(x1, y1, x2, y2, nhan="", lech=0, mau="#334155") -> str:
    out = [f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" stroke="{mau}" stroke-width="1.4" marker-end="url(#mui)"/>']
    if nhan:
        mx, my = (x1 + x2) / 2, (y1 + y2) / 2 + lech
        rong = len(nhan) * 6.4 + 10
        out.append(f'<rect x="{mx - rong / 2:.1f}" y="{my - 10:.1f}" width="{rong:.1f}" height="18" rx="4" fill="#ffffff" opacity="0.95"/>')
        out.append(f'<text x="{mx:.1f}" y="{my + 3:.1f}" font-size="11" text-anchor="middle" fill="{mau}">{e(nhan)}</text>')
    return "\n".join(out)


DEFS = (
    '<defs><marker id="mui" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" '
    'orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" fill="#334155"/></marker></defs>'
)


def svg(w, h, than: str, nhan: str) -> str:
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" '
        f'role="img" aria-label="{e(nhan)}" font-family="system-ui, sans-serif">{DEFS}{than}</svg>'
    )


def so_khoi_hien_tai(mods: list[dict]) -> str:
    cot = {"gt": 0, "ql": 1, "khac": 2}
    tieu = ["Tầng giao tiếp (theo tên module)", "Quản lý model (theo tên module)", "Khác / hạ tầng"]
    W, bw, bh, gap = 1080, 320, 58, 12
    hang = {k: 0 for k in cot}
    than = []
    for k, i in cot.items():
        x = 30 + i * 350
        than.append(f'<text x="{x + bw / 2}" y="34" font-size="15" font-weight="700" text-anchor="middle">{e(tieu[i])}</text>')
    for m in mods:
        i = cot[m["lop"]]
        x, y = 30 + i * 350, 52 + hang[m["lop"]] * (bh + gap)
        hang[m["lop"]] += 1
        loai = "loi" if m["lop"] == "gt" and m["re"] else m["lop"]
        than.append(
            hop(x, y, bw, bh, loai, [f'{m["module"]}.rs', f'{m["nhac"]} dòng nhắc CLI · {m["re"]} chỗ rẽ · {m["dinh_danh"]} định danh'], dam=True)
        )
    H = 52 + max(hang.values()) * (bh + gap) + 20
    return svg(W, H, "\n".join(than), "Sơ đồ khối hiện tại — module theo lớp")


def so_khoi_dich(kd: dict) -> str:
    W, x, w = 900, 150, 600
    than, y = [], 20
    than.append(hop(x + 150, y, 300, 44, "ngoai", ["Chủ máy (Telegram)"], dam=True))
    y += 44
    than.append(mui_ten(450, y, 450, y + 30, "lệnh / nút"))
    y += 30
    h = 40 + 22 * len(kd["giao_tiep"])
    than.append(hop(x, y, w, h, "gt", ["TẦNG GIAO TIẾP", *kd["giao_tiep"]], dam=True))
    y += h
    than.append(mui_ten(450, y, 450, y + 30))
    y += 30
    h = 40 + 22 * len(kd["giao_dien"])
    than.append(hop(x + 60, y, w - 120, h, "khac", ["GIAO DIỆN CHUNG (GT chỉ hỏi những câu này)", *kd["giao_dien"]], dam=True, co=12))
    y += h
    than.append(mui_ten(450, y, 450, y + 30))
    y += 30
    h = 40 + 22 * len(kd["quan_ly_model"]) + 90
    than.append(hop(x, y, w, h, "ql", ["TẦNG QUẢN LÝ MODEL", *kd["quan_ly_model"], "", "", ""], dam=True))
    bx, bw = x + 30, (w - 60 - 2 * 12) / 3
    by = y + h - 70
    than.append(f'<text x="450" y="{by - 8}" font-size="12" text-anchor="middle" font-weight="600">BỘ CHUYỂN CLI — nơi DUY NHẤT biết CLI</text>')
    for i, c in enumerate(kd["bo_chuyen"]):
        than.append(hop(bx + i * (bw + 12), by, bw, 40, "khac", [c]))
    y += h
    than.append(mui_ten(450, y, 450, y + 30))
    y += 30
    than.append(hop(x + 150, y, 300, 44, "ngoai", ["CLI + model (thay được)"], dam=True))
    return svg(W, y + 64, "\n".join(than), "Sơ đồ khối đích — hai tầng")


def luong_du_lieu(dl: dict) -> str:
    W, H, bw, bh = 1120, 720, 230, 56
    nut = {n["id"]: n for n in dl["nut"]}
    than = []
    cap = {tuple(sorted((a, b))) for a, b, _ in dl["canh"]}
    for a, b, nhan in dl["canh"]:
        na, nb = nut[a], nut[b]
        ax, ay = na["x"] + bw / 2, na["y"] + bh / 2
        bx, by = nb["x"] + bw / 2, nb["y"] + bh / 2
        hai_chieu = any(c == a2 and d == b2 for c, d, _ in dl["canh"] for a2, b2 in [(b, a)])
        dx, dy = bx - ax, by - ay
        do_dai = max((dx * dx + dy * dy) ** 0.5, 1)
        ox, oy = (-dy / do_dai * 9, dx / do_dai * 9) if hai_chieu else (0, 0)

        def cat(cx, cy, tx, ty):
            # điểm trên viền hộp tâm (cx,cy) theo hướng tới (tx,ty)
            vx, vy = tx - cx, ty - cy
            k = min(abs((bw / 2 + 4) / vx) if vx else 1e9, abs((bh / 2 + 4) / vy) if vy else 1e9)
            return cx + vx * k, cy + vy * k

        x1, y1 = cat(ax, ay, bx, by)
        x2, y2 = cat(bx, by, ax, ay)
        than.append(mui_ten(x1 + ox, y1 + oy, x2 + ox, y2 + oy, nhan, lech=oy * 1.6 if hai_chieu else 0))
    del cap
    for n in dl["nut"]:
        than.append(hop(n["x"], n["y"], bw, bh, n["loai"], gói(n["ten"], bw - 16), dam=True))
    return svg(W, H, "\n".join(than), "Luồng dữ liệu đích")


def luong_uc(uc: dict, lan: list[str], ten_lan: dict, bang: dict) -> str:
    cw, bw, x0, top = 214, 190, 20, 70
    W = x0 * 2 + cw * len(lan)
    than = []
    hang = []
    for b in uc["buoc"]:
        hang.append(gói(b[1], bw - 16, 12))
    y, toa = top, []
    for i, b in enumerate(uc["buoc"]):
        h = max(44, 16 + 17 * len(hang[i]))
        toa.append((y, h))
        y += h + 26
    H = y + 10
    for j, l in enumerate(lan):
        x = x0 + j * cw
        nen = "#f8fafc" if j % 2 == 0 else "#ffffff"
        than.append(f'<rect x="{x}" y="10" width="{cw}" height="{H - 20}" fill="{nen}" stroke="#e2e8f0"/>')
        than.append(f'<text x="{x + cw / 2}" y="34" font-size="13" font-weight="700" text-anchor="middle">{e(l)}</text>')
        than.append(f'<text x="{x + cw / 2}" y="52" font-size="11" text-anchor="middle" fill="#64748b">{e(ten_lan[l])}</text>')
    tam = []
    for i, b in enumerate(uc["buoc"]):
        j = lan.index(b[0])
        x = x0 + j * cw + (cw - bw) / 2
        y, h = toa[i]
        canh_bao = len(b) > 2
        dong = [f"{i + 1}. {d}" if k == 0 else d for k, d in enumerate(hang[i])]
        if canh_bao:
            dong[0] = "⚠ " + dong[0]
        than.append(hop(x, y, bw, h, "loi" if canh_bao else LANE_MAU[b[0]], dong, co=12, vien=2.2 if canh_bao else 1.5))
        tam.append((x + bw / 2, y, y + h))
    for i in range(1, len(tam)):
        (xa, _, ya), (xb, yb, _) = tam[i - 1], tam[i]
        if abs(xa - xb) < 1:
            than.append(mui_ten(xa, ya, xb, yb))
        else:
            ym = ya + 13
            than.append(f'<polyline points="{xa},{ya} {xa},{ym} {xb},{ym} {xb},{yb}" fill="none" stroke="#334155" stroke-width="1.4" marker-end="url(#mui)"/>')
    return svg(W, H, "\n".join(than), f"Luồng {uc['id']}")


# ── HTML ───────────────────────────────────────────────────────────────────
CSS = """
body{font-family:system-ui,sans-serif;margin:0;color:#0f172a;background:#fff}
header{padding:16px 24px;border-bottom:1px solid #e2e8f0}
h1{font-size:20px;margin:0 0 6px} h2{font-size:17px;margin:28px 0 8px}
main{padding:0 24px 40px} nav a{margin-right:12px;font-size:14px}
.note{color:#475569;font-size:13px}
.svgwrap{overflow-x:auto;border:1px solid #e2e8f0;border-radius:8px;padding:8px;background:#fff}
table{border-collapse:collapse;font-size:13px;margin-top:8px}
th,td{border:1px solid #e2e8f0;padding:4px 8px;text-align:left;vertical-align:top}
th{background:#f1f5f9} tbody tr:nth-child(odd){background:#ffffff} tbody tr:nth-child(even){background:#f8fafc}
tbody tr:hover{background:#e0f2fe} code{font-size:12px}
.chu{display:flex;gap:14px;flex-wrap:wrap;font-size:12px;margin:8px 0}
.chu span{display:inline-flex;align-items:center;gap:6px}
.o{width:14px;height:14px;border-radius:3px;display:inline-block;border:1.5px solid}
"""


def bang_bang_chung(khoa: list[str], bang: dict) -> str:
    if not khoa:
        return ""
    rows = []
    for i, k in enumerate(khoa, 1):
        bc = bang[k]
        tep = bc["tep"].replace("ws:", "~/projects/")
        noi = ", ".join(f"<code>{e(tep)}:{d}</code>" for d in bc["dong"])
        rows.append(f"<tr><td>{i}</td><td>{e(bc['noi'])}</td><td>{noi}</td></tr>")
    return (
        "<table><thead><tr><th>STT</th><th>⚠ Hiện tại lệch</th><th>Bằng chứng (HEAD)</th></tr></thead><tbody>"
        + "".join(rows)
        + "</tbody></table>"
    )


def chu_giai() -> str:
    muc = [("gt", "Tầng giao tiếp"), ("ql", "Quản lý model / bộ chuyển"), ("kho", "Kho dữ liệu"), ("ngoai", "Bên ngoài"), ("loi", "⚠ CLI chen vào luồng (hiện tại)")]
    return '<div class="chu">' + "".join(
        f'<span><i class="o" style="background:{MAU[k][0]};border-color:{MAU[k][1]}"></i>{e(t)}</span>' for k, t in muc
    ) + "</div>"


def dung(nguon: dict, bang: dict, mods: list[dict], sha: str) -> str:
    phan = []
    nav = ['<a href="#khoi-hien-tai">Khối hiện tại</a>', '<a href="#khoi-dich">Khối đích</a>', '<a href="#du-lieu">Luồng dữ liệu</a>']
    nav += [f'<a href="#{u["id"]}">{u["id"]}</a>' for u in nguon["uc"]]
    rows = "".join(
        f"<tr><td>{i}</td><td><code>{e(m['tep'])}</code></td><td>{ {'gt': 'giao tiếp', 'ql': 'quản lý model', 'khac': 'khác'}[m['lop']] }</td>"
        f"<td>{m['nhac']}</td><td>{m['re']}</td><td>{m['dinh_danh']}</td></tr>"
        for i, m in enumerate(mods, 1)
    )
    tong_re_gt = sum(m["re"] for m in mods if m["lop"] == "gt")
    phan.append(
        f'<section id="khoi-hien-tai"><h2>1. Sơ đồ khối — HIỆN TẠI (trích từ mã HEAD {e(sha)})</h2>'
        f'<p class="note">Mỗi hộp = một module có nhắc tên CLI. Đỏ = module tầng giao tiếp có chỗ rẽ theo CLI '
        f'(tổng {tong_re_gt} chỗ rẽ <code>.is_codex()/.is_opencode()/.is_claude()</code> trong tầng giao tiếp). '
        f'Đếm thô theo dòng — gồm cả chú thích và bài kiểm.</p>{chu_giai()}'
        f'<div class="svgwrap">{so_khoi_hien_tai(mods)}</div>'
        f'<table id="bang-module"><thead><tr><th>STT</th><th>Tệp</th><th>Lớp</th><th>Dòng nhắc CLI</th><th>Chỗ rẽ</th><th>Định danh mang tên CLI/model</th></tr></thead><tbody>{rows}</tbody></table></section>'
    )
    phan.append(
        f'<section id="khoi-dich"><h2>2. Sơ đồ khối — ĐÍCH (đề xuất)</h2>'
        f'<p class="note">Tầng giao tiếp chỉ biết PHIÊN và CỬA SỔ; CLI + model chỉ có ở bộ chuyển.</p>'
        f'<div class="svgwrap">{so_khoi_dich(nguon["khoi_dich"])}</div></section>'
    )
    phan.append(
        f'<section id="du-lieu"><h2>3. Luồng dữ liệu — ĐÍCH</h2>{chu_giai()}'
        f'<div class="svgwrap">{luong_du_lieu(nguon["du_lieu"])}</div></section>'
    )
    for u in nguon["uc"]:
        khoa = [b[2] for b in u["buoc"] if len(b) > 2]
        phan.append(
            f'<section id="{e(u["id"])}"><h2>{e(u["id"])} · {e(u["ten"])}</h2>'
            f'<div class="svgwrap">{luong_uc(u, nguon["lan"], nguon["ten_lan"], bang)}</div>{bang_bang_chung(khoa, bang)}</section>'
        )
    return (
        '<!doctype html><html lang="vi"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        f"<title>{e(nguon['tieu_de'])}</title><style>{CSS}</style></head><body>"
        f"<header><h1>{e(nguon['tieu_de'])}</h1><p class=\"note\">{e(nguon['ghi_chu'])} Nguồn: "
        f"<code>docs/so-do-tang/nguon.json</code> · dựng: <code>python3 docs/so-do-tang/build.py</code>.</p>"
        f"<nav>{''.join(nav)}</nav></header><main>{''.join(phan)}</main></body></html>\n"
    )


def tao(doc=doc_head) -> tuple[str, list[str]]:
    nguon = json.loads((HERE / "nguon.json").read_text())
    bang, mat = giai_bang_chung(nguon["bang_chung"], doc)
    for u in nguon["uc"]:
        for b in u["buoc"]:
            if len(b) > 2 and b[2] not in bang:
                mat.append(f"{u['id']}: khoá bằng chứng lạ {b[2]}")
    mods = dem_module(nguon["lop_module"], doc)
    return dung(nguon, bang, mods, head_sha()), mat


def main(argv: list[str]) -> int:
    try:
        noi_dung, mat = tao()
    except KhongDoDuoc as ex:
        print(f"KHÔNG ĐO ĐƯỢC: {ex}")
        return 2
    if mat:
        print("ĐỎ — bằng chứng không còn trong mã HEAD:", ", ".join(mat))
        return 1
    if "--kiem" in argv:
        if not OUT.exists() or OUT.read_text() != noi_dung:
            print("ĐỎ — index.html cũ so với nguon.json + HEAD; chạy lại build.py")
            return 1
        print("khớp")
        return 0
    OUT.write_text(noi_dung)
    print(f"đã dựng {OUT} ({len(noi_dung)} byte)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
