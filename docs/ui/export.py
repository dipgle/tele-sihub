"""Xuất bản mẫu thành một HTML tự chứa, không cần CDN hoặc file đi kèm."""
from pathlib import Path
import hashlib
import json

source = Path(__file__).resolve().parent
target = source.parent.parent / '.tmp/huba-desktop-design.html'
html = (source / 'index.html').read_text()
css = (source / 'style.css').read_text()
js = (source / 'app.js').read_text()
css_tag = '<link rel="stylesheet" href="style.css">'
js_tag = '<script src="app.js"></script>'
doc_link = '<a href="../thiet-ke-desktop.md">Đọc thiết kế & luồng ↗</a>'
for item in (css_tag, js_tag, doc_link):
    if html.count(item) != 1:
        raise SystemExit('Đầu vào đã đổi; kiểm lại các thẻ trước khi xuất.')
html = html.replace(css_tag, '<style>\n' + css + '\n</style>')
html = html.replace(js_tag, '<script>\n' + js.replace('</script', '<\\/script') + '\n</script>')
html = html.replace(doc_link, '<span>Bảy màn bấm thử · Tài liệu kiến trúc gửi riêng</span>')
target.parent.mkdir(exist_ok=True)
target.write_text(html)
print(json.dumps({'file':str(target),'size':target.stat().st_size,
                  'sha256':hashlib.sha256(target.read_bytes()).hexdigest()}))
