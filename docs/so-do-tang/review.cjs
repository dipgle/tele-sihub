// Kiểm tài liệu sơ đồ tầng: không gọi CLI/model, không dữ liệu nghiệp vụ.
// Chạy: bash ~/projects/scripts/cong-node chay e2e -- node docs/so-do-tang/review.cjs
// Ra: ảnh từng sơ đồ ở .tmp/so-do-tang-shots/<engine>/ + một dòng JSON; mã ≠ 0 khi có lỗi.
const {chromium, webkit} = require('playwright');
const {pathToFileURL} = require('node:url');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');

const root = path.resolve(__dirname, '../..');
const engine = process.env.HUBA_REVIEW_BROWSER || 'chromium';
assert(['chromium', 'webkit'].includes(engine), 'trình duyệt không hỗ trợ');
const out = path.join(root, '.tmp/so-do-tang-shots', engine);
const SO_SVG = 10; // 3 sơ đồ chung + 7 UC — đổi nguon.json thì đổi số này

async function inspect(page) {
  return page.locator('svg').evaluateAll(els => {
    const loi = [];
    els.forEach((svg, si) => {
      const v = svg.viewBox.baseVal;
      const t = [...svg.querySelectorAll('text')].filter(x => x.textContent.trim())
        .map(x => ({s: x.textContent, b: x.getBBox()}));
      for (const {s, b} of t)
        if (b.x < 0 || b.y < 0 || b.x + b.width > v.width || b.y + b.height > v.height)
          loi.push({svg: si, loai: 'tran-svg', s});
      for (let i = 0; i < t.length; i++) for (let j = i + 1; j < t.length; j++) {
        const a = t[i].b, b = t[j].b;
        if (Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x) > 0.5 &&
            Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y) > 0.5)
          loi.push({svg: si, loai: 'chu-chong', a: t[i].s, b: t[j].s});
      }
    });
    return loi;
  });
}

(async () => {
  fs.mkdirSync(out, {recursive: true});
  const browser = await (engine === 'webkit' ? webkit : chromium).launch({headless: true});
  const page = await browser.newPage({viewport: {width: 1440, height: 1000}});
  const loi = [], mang = [];
  page.on('console', m => { if (m.type() === 'error') loi.push(m.text()); });
  page.on('pageerror', e => loi.push(e.message));
  page.on('request', r => { if (/^https?:/.test(r.url())) mang.push(r.url()); });
  let ma = 0, ket = {};
  try {
    await page.goto(pathToFileURL(path.join(root, 'docs/so-do-tang/index.html')).href);
    await page.evaluate(() => document.fonts.ready);
    const n = await page.locator('svg').count();
    assert.equal(n, SO_SVG, `số SVG ${n} ≠ ${SO_SVG}`);
    for (const a of await page.locator('nav a').all()) {
      const href = await a.getAttribute('href');
      await a.click();
      assert.equal(await page.evaluate(() => location.hash), href);
    }
    const hong = await inspect(page);
    const svgs = await page.locator('svg').all();
    for (let i = 0; i < svgs.length; i++)
      await svgs[i].screenshot({path: path.join(out, `${String(i + 1).padStart(2, '0')}.png`)});
    await page.screenshot({path: path.join(out, 'trang.png'), fullPage: true});
    ket = {engine, svg: n, chu_chong_tran: hong.length, console: loi.length, mang: mang.length, hong: hong.slice(0, 12), loi};
    if (hong.length || loi.length || mang.length) ma = 1;
  } catch (e) {
    ket = {engine, loi: String(e && e.message || e)};
    ma = 1;
  } finally {
    await browser.close();
  }
  console.log(JSON.stringify(ket));
  process.exit(ma);
})();
