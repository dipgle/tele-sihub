"""Render the proposed huba flows as offline SVG/HTML and Mermaid Markdown.

Only design documents are generated. This is not a runtime dependency map or
an implementation of the proposed contracts. Run from any directory.
"""
from __future__ import annotations

import html
import json
from pathlib import Path
import textwrap
import xml.etree.ElementTree as ET
from workspace_mockup import WORKSPACE_CSS, render_workspace, standalone

HERE = Path(__file__).resolve().parent
WIDTH = 1200
ROW = 126
TOP = 126
COLORS = {"normal": "#f8fafc", "branch": "#fff7e6", "error": "#fff1f2", "optional": "#f3f0ff"}
ISSUE_STATES = {
    "bug": ("LỖI ĐÃ XÁC NHẬN", "#b91c1c"),
    "gap": ("CHƯA ĐO / CHƯA ĐỦ", "#9a4a00"),
    "decision": ("CẦN BÀN / CHỐT", "#6d28d9"),
    "risk": ("RỦI RO CẦN KIỂM", "#7c5800"),
}


def validate(data):
    ids = set()
    for flow in data["flows"]:
        if flow["id"] in ids:
            raise ValueError("Duplicate flow ID: " + flow["id"])
        ids.add(flow["id"])
        if not 2 <= len(flow["actors"]) <= 12:
            raise ValueError("Expected 2–12 participants")
        if len(set(flow["actors"])) != len(flow["actors"]):
            raise ValueError("Duplicate participant")
        for event in flow["events"]:
            if len(event) != 5:
                raise ValueError("Event requires from/to/text/condition/kind")
            start, end, text, condition, kind = event
            if not (0 <= start < len(flow["actors"]) and 0 <= end < len(flow["actors"])):
                raise ValueError("Unknown participant in " + flow["id"])
            if not text or not condition or kind not in COLORS:
                raise ValueError("Unlabelled event or unknown state")
    uc_ids = set()
    for uc in data["use_cases"]:
        if uc[0] in uc_ids:
            raise ValueError("Duplicate UC ID")
        uc_ids.add(uc[0])
        if len(uc) != 6 or not uc[4] or any(ref not in ids for ref in uc[4]):
            raise ValueError("UC must link existing flows")
    if {f["layer"] for f in data["flows"]} != {"UC", "DATA", "CONTEXT"}:
        raise ValueError("All three layers are required")
    flows = {f["id"]: f for f in data["flows"]}
    issue_ids = set()
    for issue in data.get("issues", []):
        if issue["id"] in issue_ids:
            raise ValueError("Duplicate issue ID")
        issue_ids.add(issue["id"])
        if issue["state"] not in ISSUE_STATES:
            raise ValueError("Unknown issue state")
        if not all(issue.get(k) for k in ("title", "evidence", "proposal", "discuss", "points")):
            raise ValueError("Issue requires evidence, proposal, discussion and points")
        for flow_id, step in issue["points"]:
            if flow_id not in flows or not 1 <= step <= len(flows[flow_id]["events"]):
                raise ValueError("Issue points to an unknown flow or step")


def esc(text):
    return html.escape(str(text), quote=True)


def lines(text, width):
    return textwrap.wrap(text, width=max(18, width), break_long_words=False, break_on_hyphens=False)


def svg(flow, issues=()):
    """Sequence layout: one event per row; labels never share a row."""
    n = len(flow["actors"])
    width = max(WIDTH, n * 220 + 140)
    min_width = width if n > 5 else 1000
    col = (width - 140) / n
    xs = [100 + col * (i + .5) for i in range(n)]
    height = TOP + len(flow["events"]) * ROW + 30
    prefix = flow["id"]
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" style="min-width:{min_width}px" role="img" aria-labelledby="{prefix}-title {prefix}-desc">',
           f'<title id="{prefix}-title">{esc(flow["title"])}</title>',
           f'<desc id="{prefix}-desc">{esc(flow["purpose"])}. Đọc từ trên xuống; các hàng có điều kiện chỉ chạy khi điều kiện đúng.</desc>',
           f'<defs><marker id="{prefix}-arrow" markerWidth="9" markerHeight="9" refX="8" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8" fill="#334155"/></marker></defs>',
           f'<rect width="{width}" height="{height}" fill="white"/>']
    for i, actor in enumerate(flow["actors"]):
        x = xs[i]
        out.append(f'<line x1="{x}" y1="86" x2="{x}" y2="{height-15}" stroke="#cbd5e1" stroke-dasharray="5 5"/>')
        out.append(f'<rect x="{x-col/2+8}" y="18" width="{col-16}" height="68" rx="10" fill="#eaf2ff" stroke="#b8cef2"/>')
        for j, part in enumerate(lines(actor, int((col-28)/8))):
            out.append(f'<text x="{x}" y="{43+j*22}" text-anchor="middle" font-size="14" font-weight="600" fill="#0f172a">{esc(part)}</text>')
    for index, (start, end, text, condition, kind) in enumerate(flow["events"]):
        y = TOP + index * ROW
        marks = [issue for issue in issues if [prefix, index + 1] in issue['points']]
        out.append(f'<rect id="{prefix}-step-{index+1}" x="8" y="{y-28}" width="{width-16}" height="{ROW-8}" rx="6" fill="{COLORS[kind]}" fill-opacity=".94"/>')
        out.append(f'<text x="20" y="{y-6}" font-size="12" fill="#64748b">{index+1:02}</text>')
        for mark_index, issue in enumerate(marks):
            label, color = ISSUE_STATES[issue['state']]
            out.append(f'<a href="#{esc(issue["id"])}"><text x="20" y="{y+22+mark_index*18}" font-size="11" font-weight="700" fill="{color}">{esc(issue["id"])}</text></a>')
        # Conditions have their own line across the row, not over an arrow.
        row_condition = ('DỪNG NHÁNH: ' if kind == 'error' else '') + condition
        out.append(f'<text x="70" y="{y-7}" font-size="12" font-weight="600" fill="#854d0e">{esc(row_condition)}</text>')
        a, b = xs[start], xs[end]
        if start == end:
            wrapped = lines(text, int((col-44)/7.2))
            if len(wrapped) > 3:
                raise ValueError(f"Local label too long: {prefix}/{index+1}")
            out.append(f'<rect x="{a-col/2+15}" y="{y+4}" width="{col-30}" height="84" rx="8" fill="white" stroke="#94a3b8"/>')
            for j, part in enumerate(wrapped):
                out.append(f'<text x="{a}" y="{y+25+j*22}" text-anchor="middle" font-size="13" fill="#0f172a">{esc(part)}</text>')
        else:
            wrapped = lines(text, int((abs(b-a)-24)/7.1))
            if len(wrapped) > 3:
                raise ValueError(f"Arrow label too long: {prefix}/{index+1}")
            for j, part in enumerate(wrapped):
                out.append(f'<text x="{(a+b)/2}" y="{y+18+j*22}" text-anchor="middle" font-size="13" fill="#0f172a">{esc(part)}</text>')
            dash = ' stroke-dasharray="6 4"' if kind == "optional" else ""
            out.append(f'<line x1="{a}" y1="{y+88}" x2="{b}" y2="{y+88}" stroke="#334155" stroke-width="1.6"{dash} marker-end="url(#{prefix}-arrow)"/>')
    out.append('</svg>')
    result = ''.join(out)
    ET.fromstring(result)
    return result


def mermaid(flow):
    out = ["sequenceDiagram", "    autonumber"]
    for i, actor in enumerate(flow["actors"]):
        out.append(f"    participant P{i} as {actor}")
    errors = []
    for step, event in enumerate(flow['events'], 1):
        if event[4] == 'error':
            if errors and errors[-1][-1][0] == step - 1:
                errors[-1].append((step, event))
            else:
                errors.append([(step, event)])
    if errors:
        out.append('    alt Đường chính khi các điều kiện hợp lệ')
    for step, (start, end, text, condition, kind) in enumerate(flow['events'], 1):
        if kind == 'error':
            continue
        guarded = kind != 'normal'
        out.append(f"    opt {condition}" if guarded else f"    Note over P{start}: Bước {step} / {condition}")
        safe = text.replace(';', ' / ').replace('\n', '<br/>')
        arrow = '-->>' if kind == 'optional' else '->>'
        out.append(f'    P{start}{arrow}P{end}: {safe}')
        if guarded:
            out.append('    end')
    # Adjacent error rows are one alternative: e.g. source failure must still
    # reach the unknown/stale notification before this interaction terminates.
    for group in errors:
        first_step, first = group[0]
        out.append(f'    else Nhánh lỗi tại bước {first_step} / {first[3]}')
        out.append('    break Dừng thao tác hiện tại sau khi báo đủ lỗi')
        for step, (start, end, text, condition, _) in group:
            safe = text.replace(';', ' / ').replace('\n', '<br/>')
            out.append(f'    Note over P{start}: Bước {step} / {condition}')
            out.append(f'    P{start}->>P{end}: {safe}')
        out.append('    end')
    if errors:
        out.append('    end')
    return '\n'.join(out)


def table_html(headers, rows):
    return '<div class="table-wrap"><table><thead><tr>' + ''.join(f'<th>{esc(h)}</th>' for h in headers) + '</tr></thead><tbody>' + ''.join('<tr>' + ''.join(f'<td>{esc(c)}</td>' for c in row) + '</tr>' for row in rows) + '</tbody></table></div>'


def table_md(headers, rows):
    def cell(value):
        return str(value).replace('|', '\\|').replace('\n', '<br>')
    return '\n'.join(['| ' + ' | '.join(map(cell, headers)) + ' |', '| ' + ' | '.join(['---'] * len(headers)) + ' |'] + ['| ' + ' | '.join(map(cell, row)) + ' |' for row in rows])


CSS = """
:root{font-family:system-ui,sans-serif;color:#152238;background:#f0f4fa;font-size:16px;line-height:1.65}
*{box-sizing:border-box}body{margin:0}header,main{max-width:1280px;margin:auto;padding:28px 32px}header{padding-top:48px}h1{font-size:38px;line-height:1.2;max-width:1000px}h2{line-height:1.35;margin:0 0 14px}h3{margin-bottom:8px}a{color:#154c9d}nav{display:flex;flex-wrap:wrap;gap:10px;margin-top:24px}nav a,.tag{background:#e6efff;border:1px solid #b8cef2;border-radius:8px;padding:6px 12px;text-decoration:none}.tag{font-size:12px;display:inline-block}section{background:white;border:1px solid #d5dfed;border-radius:14px;margin-bottom:24px;padding:26px;scroll-margin-top:20px}.warning{border-left:5px solid #b45309;padding:16px 20px;background:#fff7e6}.subtitle{max-width:1000px;color:#475569}.legend{display:flex;flex-wrap:wrap;gap:18px;font-size:14px}.legend span{padding:4px 10px;border:1px solid #cbd5e1;border-radius:6px}.diagram{overflow-x:auto;border:1px solid #dce3ed;border-radius:10px;background:white;margin:16px 0}.diagram svg{display:block;width:100%;min-width:1000px;font-family:system-ui,sans-serif}.table-wrap{overflow:auto}table{width:100%;border-collapse:collapse;font-size:14px;min-width:800px}th,td{border:1px solid #d6dfeb;padding:10px 12px;text-align:left;vertical-align:top}th{background:#eaf2ff}tbody tr:nth-child(even){background:#f8fafc}details{border-top:1px solid #d5dfed;padding-top:14px}summary{cursor:pointer;color:#154c9d;font-weight:600}pre{white-space:pre-wrap;overflow-wrap:anywhere;font-size:13px;background:#f4f6fb;padding:18px}.back{font-size:13px}footer{max-width:1280px;margin:auto;padding:0 32px 40px;color:#64748b;font-size:14px}@media(max-width:700px){header,main{padding:18px 14px}h1{font-size:28px}section{padding:16px}.diagram svg{min-width:1000px}.subtitle{font-size:15px}}@media print{body{background:white}nav,.back,details{display:none}header,main{padding:0}.diagram svg{min-width:0}section{break-before:page;border:0;padding:0}header{break-after:page}}
"""

CSS += """
.subtitle,section p,.issue-links a{overflow-wrap:anywhere}.issue-links a{min-width:0;max-width:100%}
.issue{border-left:5px solid var(--issue);padding:18px;background:#fafaff;margin:14px 0;scroll-margin-top:20px;overflow-wrap:anywhere}
.issue h3{color:var(--issue);margin:0 0 8px}.issue p{margin:8px 0}.issue-links{display:flex;flex-wrap:wrap;gap:8px}.issue-links a{font-size:13px;padding:3px 8px;border:1px solid currentColor;border-radius:6px}
.architecture{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px}.architecture>div{border:1px solid #94a3b8;background:#f8fafc;border-radius:10px;padding:16px}.architecture .core{grid-column:1/-1;background:#eaf2ff;text-align:center}.architecture p{margin:7px 0;font-size:14px}.arrow-note{text-align:center;color:#475569;font-weight:600}
.wire-scroll{overflow:auto;border:1px solid #94a3b8;border-radius:10px}.wire{min-width:1040px;background:#f8fafc}.wire-head{background:#142b4d;color:white;display:flex;justify-content:space-between;padding:14px 18px;gap:24px}.wire-body{display:grid;grid-template-columns:145px 1fr 230px}.wire-menu{padding:14px;background:#eaf2ff;line-height:2.2}.wire-work{padding:14px;min-width:0}.wire-detail{padding:14px;background:#eff3f9;border-left:1px solid #cbd5e1}.wire-panels{display:grid;grid-template-columns:1fr 1fr;gap:10px}.wire-panel{padding:12px;border:1px solid #94a3b8;border-radius:8px;background:white;min-height:200px}.wire-panel p{font-size:13px}.wire-input{border:1px dashed #64748b;border-radius:6px;padding:8px;font-size:12px}.wire-list{border:1px solid #94a3b8;border-radius:8px;background:white;margin-top:14px;padding:12px}.wire-row{display:grid;grid-template-columns:34px 1fr 90px;gap:8px;padding:8px;font-size:12px}.wire-row:nth-child(even){background:#eaf2ff}.wire-caption{font-size:13px;color:#475569}.terminal-answer{border-left:5px solid #2563eb;padding:18px;background:#eaf2ff}.terminal-answer strong{color:#123768}
@media(max-width:700px){.architecture{grid-template-columns:1fr}.architecture .core{grid-column:auto}.issue{padding:12px}.wire-head{font-size:13px}}
"""
CSS += WORKSPACE_CSS


def overview_html():
    return '''<section id="overview"><h2>0 · Bức tranh tổng thể — huba là nơi làm việc, CLI là runtime</h2>
<p class="warning">Bản V2 bổ sung bàn hai vùng, nhiều view, identity và terminal takeover. <strong>Không phải app đã chạy.</strong> Các khối dưới là thành phần logic, không phải thêm từng ấy dịch vụ.</p>
<div class="architecture">
<div><strong>Desktop: nhiều bàn / cửa sổ</strong><p>Chat · Trace · Bảng hỏi/quyền · Tệp · Báo cáo · Cấu hình</p><a href="#U5">P04 · Bàn hai vùng</a></div>
<div><strong>Telegram / client được cấp quyền</strong><p>Cùng handler và dữ liệu; lựa chọn riêng từng kênh. MCP facade là cửa tùy chọn.</p><a href="#P05">P05 · Đích lệnh rõ</a></div>
<div><strong>Terminal nhúng / cửa sổ gốc</strong><p>Tiếp quản cùng CLI khi cần. Không mở một phiên khác thay thế.</p><a href="#P06">P06 · Cần chốt mode</a></div>
<div class="core"><strong>↓ Gate quyền → Command/query service → Dispatcher theo phiên ↑ Fanout theo view</strong><p>Session registry · Identity assignment · Runtime binding · Capabilities · Revision/audit · Command ledger</p><a href="#U6">U6 · Snapshot + delta + reconnect</a></div>
<div><strong>CLI adapter</strong><p>Claude / OpenCode ưu tiên; giữ Codex hiện có. CLI sở hữu context, model loop và tool/MCP.</p><a href="#C2">C2 · Model → tool → model</a></div>
<div><strong>Host / runtime adapter</strong><p>Process · PTY/console · attach/takeover · terminal viewer · OS permissions. Khác biệt theo hệ.</p><a href="#U7">U7 · Điều kiện giảm terminal</a></div>
<div><strong>Collector → raw → projection</strong><p>Ingest cursor, graph lượt/tool/job, source health, usage index. UI đọc không gọi thêm model.</p><a href="#P03">P03 · Nguồn chưa đủ</a></div>
</div><p class="arrow-note">Đường lệnh: UI → core → driver → CLI. Đường quan sát: CLI → collector → raw/projection → views/kênh theo dõi.</p></section>
''' + render_workspace() + '''
<section id="terminal"><h2>T0 · Có còn cần cửa sổ terminal riêng?</h2><div class="terminal-answer"><strong>Mục tiêu: không cần mở thường xuyên. Không phải bỏ CLI hay mọi terminal.</strong><p>Chat, theo dõi, bảng hỏi, quyền, tệp, ngắt và quản lý ở UI. CLI vẫn chạy, PTY/console vẫn có thể cần. Terminal nhúng hoặc cửa sổ gốc là đường tiếp quản/đăng nhập/cứu hộ.</p><p>Linux hiện đã tách tmux pane khỏi cửa sổ xem. macOS hiện còn phụ thuộc Terminal trong driver; Windows source ghi chưa đo console/ConPTY trên máy thật. Chưa thể hứa bỏ cửa sổ riêng đồng loạt.</p><p><a href="#P06">P06 cần bàn</a> · <a href="#U7">U7 đường takeover</a>. Không tự tắt cửa sổ hoặc chuyển runtime của phiên đang làm.</p></div></section>'''


def issue_html(issue):
    label, color = ISSUE_STATES[issue['state']]
    points = ''.join(f'<a href="#{esc(fid)}-step-{step}">{esc(fid)} · bước {step}</a>' for fid, step in issue['points'])
    return (f'<article class="issue" id="{esc(issue["id"])}" style="--issue:{color}">'
            f'<h3>{esc(issue["id"])} · {esc(issue["title"])}</h3><strong>{label}</strong>'
            f'<p><b>Bằng chứng / phạm vi:</b> {esc(issue["evidence"])}</p>'
            f'<p><b>Hướng thiết kế, chưa triển khai:</b> {esc(issue["proposal"])}</p>'
            f'<p><b>Điểm bàn tiếp:</b> {esc(issue["discuss"])}</p><div class="issue-links">{points}</div></article>')


def render(data):
    validate(data)
    title = data['title']
    uc_rows = [[u[0], u[1], u[2], u[3], ', '.join(u[4]), u[5]] for u in data['use_cases']]
    uc_headers = ['UC', 'Người dùng / mục tiêu', 'Input', 'Output nhìn thấy', 'Sơ đồ', 'Điều kiện lỗi / nghiệm thu dự kiến']
    md = [f'# {title}', '', data['status'], '', data['intro'], '', '## Quy tắc đọc', *[f'- {p}' for p in data['principles']], '', '## Danh mục UC', table_md(uc_headers, uc_rows)]
    links = [('overview', '0 · Tổng thể'), ('workspace', 'W0 · Bàn hai vùng'), ('terminal', 'T0 · Terminal?'), ('issues', 'P01–P12 · Điểm có vấn đề'), ('components', 'Đủ thành phần'), ('routes', 'Dữ liệu đi đường nào?'), ('X0', 'Trọn một tương tác'), ('uc', '1 · UC'), ('data', '2 · Dữ liệu'), ('context', '3 · Context / I/O'), ('contracts', 'Contract & ví dụ'), ('sources', 'Nguồn & phần chưa chốt')]
    for flow in data['flows']:
        links.append((flow['id'], flow['id']))
    page = ['<!doctype html><html lang="vi"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">', f'<title>{esc(title)}</title><style>{CSS}</style></head><body>', f'<header id="top"><span class="tag">THIẾT KẾ ĐỂ DUYỆT · {esc(data.get("date", "15/11/2026"))}</span>', f'<h1>{esc(title)}</h1><p class="subtitle">{esc(data["intro"])}</p><p class="warning">{esc(data["status"])}</p>', '<nav aria-label="Mục lục">' + ''.join(f'<a href="#{key}">{esc(label)}</a>' for key, label in links) + '</nav></header><main>', overview_html(), '<section><h2>Đọc bản vẽ thế nào?</h2><ul>' + ''.join(f'<li>{esc(p)}</li>' for p in data['principles']) + '</ul><div class="legend"><span>Nền trắng/xám: đường chính</span><span style="background:#fff7e6">Vàng: nhánh có điều kiện</span><span style="background:#fff1f2">Hồng: lỗi / chưa xác định</span><span style="background:#f3f0ff">Tím, nét đứt: khả năng tuỳ nguồn</span></div><p>Mã Pxx đặt ngay cạnh bước đang vướng; bấm để xem bằng chứng và điểm bàn. Màu mã Pxx thể hiện loại vấn đề, độc lập màu nền nhánh xử lý.</p></section>']
    page.append('<section id="issues"><h2>P01–P12 · Những chỗ cần bàn tiếp</h2>' + ''.join(issue_html(issue) for issue in data.get('issues', [])) + '</section>')
    md.extend(['', '## Tổng thể V2', '', 'Xem HTML: W0 bàn hai vùng, T0 terminal và các Pxx liên kết tới từng bước. Đặc tả tổng: ../thiet-ke-tong-the-v2.md. Khung W0 là minh họa, không nối dữ liệu thật.', '', '## Vấn đề đánh dấu'])
    for issue in data.get('issues', []):
        md.extend(['', f'### {issue["id"]} · {issue["title"]} — {ISSUE_STATES[issue["state"]][0]}', '', '**Bằng chứng/phạm vi:** ' + issue['evidence'], '', '**Đề xuất:** ' + issue['proposal'], '', '**Bàn tiếp:** ' + issue['discuss'], '', '**Tại:** ' + ', '.join(f'{fid}/bước{step}' for fid, step in issue['points'])])
    for block in data['front_tables']:
        md.extend(['', '## ' + block['title'], '', block['note'], '', table_md(block['headers'], block['rows'])])
        page.append(f'<section id="{block["id"]}"><h2>{esc(block["title"])}</h2><p>{esc(block["note"])}</p>' + table_html(block['headers'], block['rows']) + '</section>')
    page.append('<section id="uc"><h2>1 · UC — người dùng làm gì?</h2>' + table_html(uc_headers, uc_rows) + '</section>')
    seen = {'UC'}
    ordered = sorted(data['flows'], key=lambda f: {'UC': 0, 'DATA': 1, 'CONTEXT': 2}[f['layer']])
    for flow in ordered:
        if flow['layer'] not in seen:
            seen.add(flow['layer'])
            name = '2 · Dữ liệu — dữ liệu đi đâu?' if flow['layer'] == 'DATA' else '3 · Context / input / output — model nhận gì?'
            key = 'data' if flow['layer'] == 'DATA' else 'context'
            page.append(f'<section id="{key}"><h2>{esc(name)}</h2><p>{esc(data["layer_notes"][flow["layer"]])}</p></section>')
        md.extend(['', f'## {flow["id"]} · {flow["title"]}', '', flow['purpose'], '', '```mermaid', mermaid(flow), '```', '', '**Input:** ' + flow['input'], '', '**Output:** ' + flow['output'], '', '**Ràng buộc:** ' + flow['invariant']])
        associated = [issue for issue in data.get('issues', []) if any(fid == flow['id'] for fid, _ in issue['points'])]
        badges = '<div class="issue-links">' + ''.join(f'<a style="color:{ISSUE_STATES[i["state"]][1]}" href="#{i["id"]}">{i["id"]} · {esc(i["title"])}</a>' for i in associated) + '</div>'
        page.extend([f'<section id="{flow["id"]}"><span class="tag">{flow["layer"]} · {flow["id"]}</span><h2>{esc(flow["title"])}</h2>', badges, f'<p>{esc(flow["purpose"])}</p><p><strong>Input:</strong> {esc(flow["input"])}<br><strong>Output:</strong> {esc(flow["output"])}</p>', '<div class="diagram">' + svg(flow, data.get('issues', [])) + '</div>', f'<p class="warning"><strong>Ràng buộc:</strong> {esc(flow["invariant"])}</p>', '<details><summary>Đọc các bước bằng chữ</summary><ol>' + ''.join(f'<li><strong>{esc(e[3])}</strong> — {esc(flow["actors"][e[0]])} → {esc(flow["actors"][e[1]])}: {esc(e[2])}</li>' for e in flow['events']) + '</ol></details><a class="back" href="#top">↑ Mục lục</a></section>'])
    page.append('<section id="contracts"><h2>Contract, dữ liệu và ví dụ context</h2>')
    for block in data['tables']:
        md.extend(['', '## ' + block['title'], '', block['note'], '', table_md(block['headers'], block['rows'])])
        page.extend([f'<h3>{esc(block["title"])}</h3><p>{esc(block["note"])}</p>', table_html(block['headers'], block['rows'])])
    page.append('</section><section id="sources"><h2>Nguồn và phần chưa chốt</h2>')
    for block in data['notes']:
        md.extend(['', '## ' + block['title'], *['- ' + item for item in block['items']]])
        page.append(f'<h3>{esc(block["title"])}</h3><ul>' + ''.join(f'<li>{esc(item)}</li>' for item in block['items']) + '</ul>')
    page.append('</section></main><footer>SINH từ docs/flows/design.json bằng docs/flows/build.py — không sửa tay. Không gọi mạng, không chạy CLI/model, không cần thư viện ngoài.</footer></body></html>')
    md.extend(['', '---', 'SINH từ `docs/flows/design.json` bằng `python3 docs/flows/build.py`; không sửa tay bản sinh. Sơ đồ thiết kế không phải bản đồ phụ thuộc trích từ mã/vết chạy.', ''])
    return '\n'.join(md), ''.join(page)


def main():
    data = json.loads((HERE / 'design.json').read_text(encoding='utf-8'))
    markdown, page = render(data)
    (HERE / 'index.md').write_text(markdown, encoding='utf-8')
    (HERE / 'index.html').write_text(page, encoding='utf-8')
    (HERE / 'workspace.html').write_text(standalone(), encoding='utf-8')
    print(json.dumps({'flows': len(data['flows']), 'use_cases': len(data['use_cases']), 'html_bytes': len(page.encode()), 'markdown_bytes': len(markdown.encode())}))


if __name__ == '__main__':
    main()
