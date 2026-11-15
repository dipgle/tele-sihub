"""Xuất bản mẫu thành một HTML tự chứa, không cần CDN hoặc file đi kèm."""
from pathlib import Path
import hashlib
import json
import html as markup

source = Path(__file__).resolve().parent
target = source.parent.parent / '.tmp/huba-desktop-design.html'
html = (source / 'index.html').read_text()
css = (source / 'style.css').read_text()
css_tag = '<link rel="stylesheet" href="style.css">'
doc_link = '<a href="../thiet-ke-desktop.md">Đọc thiết kế & luồng ↗</a>'
script_files = ['vendor/echarts.min.js', 'reports.js', 'app.js']
script_tags = [f'<script src="{name}"></script>' for name in script_files]
for item in (css_tag, doc_link, *script_tags):
    if html.count(item) != 1:
        raise SystemExit('Đầu vào đã đổi; kiểm lại các thẻ trước khi xuất.')
html = html.replace(css_tag, '<style>\n' + css + '\n</style>')
notices = '\n\n'.join((source / 'vendor' / name).read_text()
                       for name in ['ECHARTS-LICENSE.txt', 'ECHARTS-NOTICE.txt'])
if html.count('</body>') != 1:
    raise SystemExit('Thẻ body của template không duy nhất; không ghép mù')
# Sửa template TRƯỚC khi nhúng mã. Library có chuỗi "</body>" của chính nó;
# thay toàn văn sau khi nhúng từng làm hỏng JavaScript và mất ECharts.
html = html.replace('</body>', '<template id="third-party-notices"><pre>' + markup.escape(notices) + '</pre></template>\n</body>')
html = html.replace(doc_link, '<span>Tám màn bấm thử · ECharts nhúng sẵn · Thiết kế kiến trúc gửi riêng</span>')
for name, tag in zip(script_files, script_tags):
    js = (source / name).read_text()
    html = html.replace(tag, '<script>\n' + js.replace('</script', '<\\/script') + '\n</script>', 1)
target.parent.mkdir(exist_ok=True)
target.write_text(html)
print(json.dumps({'file':str(target),'size':target.stat().st_size,
                  'sha256':hashlib.sha256(target.read_bytes()).hexdigest()}))
