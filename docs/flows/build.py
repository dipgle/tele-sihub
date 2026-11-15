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

HERE = Path(__file__).resolve().parent
WIDTH = 1200
ROW = 126
TOP = 126
COLORS = {"normal": "#f8fafc", "branch": "#fff7e6", "error": "#fff1f2", "optional": "#f3f0ff"}


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


def esc(text):
    return html.escape(str(text), quote=True)


def lines(text, width):
    return textwrap.wrap(text, width=max(18, width), break_long_words=False, break_on_hyphens=False)


def svg(flow):
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
        out.append(f'<rect x="8" y="{y-28}" width="{width-16}" height="{ROW-8}" rx="6" fill="{COLORS[kind]}" fill-opacity=".94"/>')
        out.append(f'<text x="20" y="{y-6}" font-size="12" fill="#64748b">{index+1:02}</text>')
        # Conditions have their own line across the row, not over an arrow.
        out.append(f'<text x="70" y="{y-7}" font-size="12" font-weight="600" fill="#854d0e">{esc(condition)}</text>')
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
    for start, end, text, condition, kind in flow["events"]:
        # Every conditional row is a guarded event, not an unconditional step.
        guarded = kind != "normal"
        if guarded:
            out.append(f"    opt {condition}")
        else:
            out.append(f"    Note over P{start}: {condition}")
        arrow = "-->>" if kind == "optional" else "->>"
        safe = text.replace(";", " / ").replace("\n", "<br/>")
        out.append(f"    P{start}{arrow}P{end}: {safe}")
        if guarded:
            out.append("    end")
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


def render(data):
    validate(data)
    title = data['title']
    uc_rows = [[u[0], u[1], u[2], u[3], ', '.join(u[4]), u[5]] for u in data['use_cases']]
    uc_headers = ['UC', 'Người dùng / mục tiêu', 'Input', 'Output nhìn thấy', 'Sơ đồ', 'Điều kiện lỗi / nghiệm thu dự kiến']
    md = [f'# {title}', '', data['status'], '', data['intro'], '', '## Quy tắc đọc', *[f'- {p}' for p in data['principles']], '', '## Danh mục UC', table_md(uc_headers, uc_rows)]
    links = [('components', 'Đủ thành phần'), ('routes', 'Dữ liệu đi đường nào?'), ('X0', 'Trọn một tương tác'), ('uc', '1 · UC'), ('data', '2 · Dữ liệu'), ('context', '3 · Context / I/O'), ('contracts', 'Contract & ví dụ'), ('sources', 'Nguồn & phần chưa chốt')]
    for flow in data['flows']:
        links.append((flow['id'], flow['id']))
    page = ['<!doctype html><html lang="vi"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">', f'<title>{esc(title)}</title><style>{CSS}</style></head><body>', '<header id="top"><span class="tag">THIẾT KẾ ĐỂ DUYỆT · 15/11/2026</span>', f'<h1>{esc(title)}</h1><p class="subtitle">{esc(data["intro"])}</p><p class="warning">{esc(data["status"])}</p>', '<nav aria-label="Mục lục">' + ''.join(f'<a href="#{key}">{esc(label)}</a>' for key, label in links) + '</nav></header><main>', '<section><h2>Đọc bản vẽ thế nào?</h2><ul>' + ''.join(f'<li>{esc(p)}</li>' for p in data['principles']) + '</ul><div class="legend"><span>Nền trắng/xám: đường chính</span><span style="background:#fff7e6">Vàng: nhánh có điều kiện</span><span style="background:#fff1f2">Hồng: lỗi / chưa xác định</span><span style="background:#f3f0ff">Tím, nét đứt: khả năng tuỳ nguồn</span></div></section>']
    for block in data['front_tables']:
        md.extend(['', '## ' + block['title'], '', block['note'], '', table_md(block['headers'], block['rows'])])
        page.append(f'<section id="{block["id"]}"><h2>{esc(block["title"])}</h2><p>{esc(block["note"])}</p>' + table_html(block['headers'], block['rows']) + '</section>')
    page.append('<section id="uc"><h2>1 · UC — người dùng làm gì?</h2>' + table_html(uc_headers, uc_rows) + '</section>')
    seen = {'UC'}
    for flow in data['flows']:
        if flow['layer'] not in seen:
            seen.add(flow['layer'])
            name = '2 · Dữ liệu — dữ liệu đi đâu?' if flow['layer'] == 'DATA' else '3 · Context / input / output — model nhận gì?'
            key = 'data' if flow['layer'] == 'DATA' else 'context'
            page.append(f'<section id="{key}"><h2>{esc(name)}</h2><p>{esc(data["layer_notes"][flow["layer"]])}</p></section>')
        md.extend(['', f'## {flow["id"]} · {flow["title"]}', '', flow['purpose'], '', '```mermaid', mermaid(flow), '```', '', '**Input:** ' + flow['input'], '', '**Output:** ' + flow['output'], '', '**Ràng buộc:** ' + flow['invariant']])
        page.extend([f'<section id="{flow["id"]}"><span class="tag">{flow["layer"]} · {flow["id"]}</span><h2>{esc(flow["title"])}</h2>', f'<p>{esc(flow["purpose"])}</p><p><strong>Input:</strong> {esc(flow["input"])}<br><strong>Output:</strong> {esc(flow["output"])}</p>', '<div class="diagram">' + svg(flow) + '</div>', f'<p class="warning"><strong>Ràng buộc:</strong> {esc(flow["invariant"])}</p>', '<details><summary>Đọc các bước bằng chữ</summary><ol>' + ''.join(f'<li><strong>{esc(e[3])}</strong> — {esc(flow["actors"][e[0]])} → {esc(flow["actors"][e[1]])}: {esc(e[2])}</li>' for e in flow['events']) + '</ol></details><a class="back" href="#top">↑ Mục lục</a></section>'])
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
    print(json.dumps({'flows': len(data['flows']), 'use_cases': len(data['use_cases']), 'html_bytes': len(page.encode()), 'markdown_bytes': len(markdown.encode())}))


if __name__ == '__main__':
    main()
