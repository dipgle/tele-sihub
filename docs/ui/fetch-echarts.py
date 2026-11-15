"""Lấy bản phân phối ECharts đã ghim, kiểm integrity npm, giữ license/notice.

Chỉ phục vụ prototype offline; không cài dependency hay chạy script từ gói.
"""
import base64
import hashlib
import io
import json
from pathlib import Path
import tarfile
import urllib.request

version = '6.0.0'
registry = 'https://registry.npmjs.org/echarts/' + version
vendor = Path(__file__).resolve().parent / 'vendor'
with urllib.request.urlopen(registry, timeout=30) as reply:
    metadata = json.load(reply)
distribution = metadata['dist']
url = distribution['tarball']
if not url.startswith('https://registry.npmjs.org/echarts/-/'):
    raise SystemExit('Nguồn tarball không đúng registry ECharts')
with urllib.request.urlopen(url, timeout=60) as reply:
    blob = reply.read(20 * 1024 * 1024 + 1)
if len(blob) > 20 * 1024 * 1024:
    raise SystemExit('Gói vượt giới hạn tải của prototype')
integrity = 'sha512-' + base64.b64encode(hashlib.sha512(blob).digest()).decode()
if integrity not in distribution['integrity'].split():
    raise SystemExit('Integrity không khớp; không ghi asset')
files = {}
with tarfile.open(fileobj=io.BytesIO(blob), mode='r:gz') as archive:
    for source, target in [('package/dist/echarts.min.js', 'echarts.min.js'),
                           ('package/LICENSE', 'ECHARTS-LICENSE.txt'),
                           ('package/NOTICE', 'ECHARTS-NOTICE.txt')]:
        member = archive.extractfile(source)
        if member is None:
            raise SystemExit('Thiếu tệp bắt buộc: ' + source)
        files[target] = member.read()
vendor.mkdir(exist_ok=True)
for name, content in files.items():
    (vendor / name).write_bytes(content)
manifest = {'name':'Apache ECharts','version':version,'source':url,'integrity':integrity,
            'files':{name:{'size':len(data),'sha256':hashlib.sha256(data).hexdigest()}
                     for name,data in files.items()}}
(vendor / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps(manifest))
print('GATE_EXIT=0')
