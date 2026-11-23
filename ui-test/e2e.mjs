// Nghiệm thu giao diện `huba ui` bằng TRÌNH DUYỆT THẬT (Chromium + WebKit) trên `huba ui` THẬT với danh sách phiên THẬT.
//
// Chạy:  cd ui-test && node e2e.mjs [đường tới huba]      (mặc định ../rust/target/debug/huba)
// Mã thoát: 0 = mọi ca đạt · 1 = có ca hỏng · 2 = KHÔNG ĐO ĐƯỢC (không dựng được máy chủ / không mở được engine nào)
//
// Phân loại khai báo (CLAUDE.md §12): ca 1–6 đi qua máy chủ thật + dữ liệu thật. Ca «giả lỗi» (7–11) CHẶN `/api/sessions` bằng
// `page.route` để ép trạng thái lỗi/rỗng/chờ — đó là ĐỐI CHỨNG GIẢ LỖI theo thiết kế §3 (không thể bắt máy thật lỗi theo lệnh),
// KHÔNG phải nghiệm thu dữ liệu; chúng chỉ chứng minh giao diện không nói dối («0 phiên») khi đọc lỗi.
import { chromium, webkit } from 'playwright-core';
import { spawn, execFileSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const AQUI = dirname(fileURLToPath(import.meta.url));
const HUBA = resolve(process.argv[2] || resolve(AQUI, '../rust/target/debug/huba'));
const UI = resolve(AQUI, '../ui');
const SHOTS = resolve(AQUI, '../.tmp/ui-shots');
mkdirSync(SHOTS, { recursive: true });

let sai = 0;
let khongDo = 0;
function ca(ten, ok, chiTiet = '') {
  if (!ok) sai++;
  console.log(`${ok ? 'ĐẠT ' : 'HỎNG'} ${ten}${chiTiet ? ' — ' + chiTiet : ''}`);
}

// ── máy chủ thật (MỖI engine một máy chủ + một vé riêng: vé chỉ dùng MỘT lần — dùng chung thì engine sau bị 401 đúng như thiết kế) ──
const dsSrv = [];
async function khoiDong() {
  const srv = spawn(HUBA, ['ui', '--no-open', '--dir', UI], { stdio: ['ignore', 'pipe', 'pipe'] });
  let out = '';
  srv.stdout.on('data', (d) => (out += d));
  srv.stderr.on('data', (d) => (out += d));
  dsSrv.push(srv);
  for (let i = 0; i < 100; i++) {
    const m = /http:\/\/127\.0\.0\.1:\d+\/\?t=[0-9a-f]+/.exec(out);
    if (m) return { url: m[0], goc: m[0].split('?')[0] };
    await new Promise((r) => setTimeout(r, 100));
  }
  console.log('KHÔNG ĐO ĐƯỢC: không thấy URL của `huba ui` sau 10 giây\n' + out.slice(0, 400));
  dung();
  process.exit(2);
}
const dung = () => { for (const s of dsSrv) { try { s.kill(); } catch {} } };
process.on('exit', dung);

// Số phiên theo CLI độc lập (đo song song để so; phiên có thể đổi giữa hai lần đo ⟹ chỉ so lỏng).
let soCli = null;
try {
  const j = JSON.parse(execFileSync(HUBA, ['sessions', '--json'], { encoding: 'utf8', timeout: 60000, maxBuffer: 1 << 26 }));
  soCli = j.sessions.length;
} catch (e) {
  console.log('(không đọc được `huba sessions --json` để so: ' + String(e.message).slice(0, 80) + ')');
}

function theoDoi(page) {
  const loi = [];
  page.on('console', (m) => { if (m.type() === 'error') loi.push('console: ' + m.text()); });
  page.on('pageerror', (e) => loi.push('pageerror: ' + e.message));
  page.on('requestfailed', (r) => loi.push('requestfailed: ' + r.url() + ' ' + (r.failure()?.errorText || '')));
  return loi;
}

async function chayEngine(ten, tc) {
  const { url, goc } = await khoiDong();
  let browser;
  try {
    browser = await tc.launch();
  } catch (e) {
    khongDo++;
    console.log(`KHÔNG ĐO ĐƯỢC [${ten}]: không mở được engine — ${String(e.message).split('\n')[0].slice(0, 160)}`);
    return null;
  }
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();
  const loi = theoDoi(page);
  const P = (s) => `[${ten}] ${s}`;

  // 1. vào bằng vé ⟹ trang thật hiện, danh sách phiên thật
  await page.goto(url);
  await page.waitForSelector('[data-testid=row]', { timeout: 40000 }).catch(() => {});
  const hang = await page.locator('[data-testid=row]').count();
  ca(P('1 vào bằng vé ⟹ bảng có phiên thật'), hang > 0, `${hang} hàng ở trang 1`);
  const tong = Number((await page.locator('[data-testid=n-total]').textContent().catch(() => '0')) || 0);
  const api = await page.evaluate(async () => (await (await fetch('/api/sessions')).json()).data?.sessions?.length ?? -1);
  ca(P('2 tổng trên màn = số phiên của API'), tong === api && tong > 0, `màn ${tong} · API ${api}` + (soCli != null ? ` · CLI ${soCli}` : ''));
  if (soCli != null) ca(P('2b số phiên gần với `huba sessions` (±2, đo lệch giờ)'), Math.abs(tong - soCli) <= 2, `${tong} vs ${soCli}`);

  // 3. STT liên tục qua trang
  const stt1 = await page.locator('[data-testid=row] td:first-child').allTextContents();
  ca(P('3 STT trang 1 bắt đầu từ 1, liên tục'), stt1.length > 0 && stt1.every((v, i) => Number(v) === i + 1), stt1.join(','));
  if (tong > 10) {
    await page.locator('button[data-act=next]').click();
    const stt2 = await page.locator('[data-testid=row] td:first-child').allTextContents();
    ca(P('3b STT trang 2 nối tiếp (bắt đầu từ 11)'), Number(stt2[0]) === 11, stt2.join(','));
    await page.locator('button[data-act=prev]').click();
  }
  await page.screenshot({ path: `${SHOTS}/${ten}-1-phien.png` });

  // 4. tìm kiếm trên TOÀN tập, rồi trả lại
  const ten1 = (await page.locator('[data-testid=row] td:nth-child(2) b').first().textContent()) || '';
  const tu = ten1.trim().split(/\s+/)[0];
  await page.fill('#q', tu);
  const khop = await page.locator('[data-testid=row]').count();
  ca(P('4 tìm «' + tu + '» lọc được và vẫn còn ≥ 1 hàng'), khop >= 1 && khop <= hang + 10, `${khop} hàng`);
  await page.fill('#q', 'zzz-khong-co-phien-nao-ten-vay');
  ca(P('4b từ khoá vô nghĩa ⟹ «không có phiên khớp», KHÔNG phải rỗng'), (await page.locator('[data-testid=no-match]').count()) === 1);
  await page.fill('#q', '');

  // 5. lọc trạng thái — SO VỚI API, không so với chính giao diện (`every` trên mảng rỗng luôn đúng ⟹ từng là phép đo mù)
  const dangApi = await page.evaluate(async () => (await (await fetch('/api/sessions')).json()).data.sessions
    .filter((s) => s.working && s.host !== 'shell' && s.kind !== 'shell' && !s.asking_password).length);
  await page.selectOption('#status', 'working');
  const dang = await page.locator('[data-testid=row] .pill').allTextContents();
  const khongKhop = await page.locator('[data-testid=no-match]').count();
  ca(P('5 lọc «Đang làm»: số hàng = số phiên working của API, mọi hàng đúng nhãn'),
    dang.length === Math.min(dangApi, 10) && dang.every((t) => t.includes('Đang làm')) && (dangApi > 0 || khongKhop === 1),
    `${dang.length} hàng · API ${dangApi}` + (dangApi === 0 ? ' · (API không có phiên đang làm ⟹ phải hiện «không khớp»)' : ''));
  await page.selectOption('#status', 'all');

  // 6. menu: thu gọn không làm header nhảy; mục chưa làm báo «sắp có» thay vì giả vờ chạy
  const w0 = await page.locator('.app-header').evaluate((e) => e.getBoundingClientRect().width);
  await page.click('#collapse');
  const w1 = await page.locator('.app-header').evaluate((e) => e.getBoundingClientRect().width);
  ca(P('6 thu gọn menu: header giữ nguyên chiều rộng'), Math.abs(w0 - w1) < 1, `${w0} → ${w1}`);
  await page.click('#collapse');
  await page.getByRole('navigation').getByRole('link', { name: 'Báo cáo token' }).click();
  await page.locator('[data-testid=coming]').waitFor({ timeout: 3000 }).catch(() => {});
  ca(P('6b màn chưa làm hiện «sắp có»'), (await page.locator('[data-testid=coming]').count()) === 1);
  await page.getByRole('navigation').getByRole('link', { name: 'Phiên', exact: true }).click();

  // ── giả lỗi (khai báo ở đầu tệp) ──
  const thuc = await page.evaluate(async () => await (await fetch('/api/sessions')).json());
  const ap = async (body, status = 200) => {
    await page.route('**/api/sessions', (r) => r.fulfill({ status, contentType: 'application/json', body: JSON.stringify(body) }));
    await page.evaluate(() => window.dispatchEvent(new Event('hashchange')));
  };
  const lam = async () => { await page.unroute('**/api/sessions'); };
  const doiTai = async () => { await page.waitForTimeout(5600); }; // chờ nhịp làm mới 5 giây của giao diện

  // 7. đọc lỗi SAU lần tốt ⟹ vẫn hiện bản cũ + cảnh báo, KHÔNG hiện «0 phiên»
  await ap({ ...thuc, error: 'claude agents treo', loading: false });
  await doiTai();
  const nt = (await page.locator('[data-testid=notice]').textContent().catch(() => '')) || '';
  ca(P('7 lỗi sau lần tốt ⟹ cảnh báo «dữ liệu CŨ» + vẫn còn hàng'), nt.includes('CŨ') && nt.includes('claude agents treo') && (await page.locator('[data-testid=row]').count()) > 0, nt.slice(0, 90));
  await page.screenshot({ path: `${SHOTS}/${ten}-2-loi-du-lieu-cu.png` });
  ca(P('7b đối chứng ngược: lỗi KHÔNG làm hiện trạng thái rỗng'), (await page.locator('[data-testid=empty]').count()) === 0);
  // 8. mất kết nối mạng
  await lam();
  await page.route('**/api/sessions', (r) => r.abort());
  await doiTai();
  const nt2 = (await page.locator('[data-testid=notice]').textContent().catch(() => '')) || '';
  ca(P('8 mất kết nối ⟹ «Mất kết nối… dữ liệu CŨ», vẫn còn hàng'), nt2.includes('Mất kết nối') && (await page.locator('[data-testid=row]').count()) > 0, nt2.slice(0, 80));
  ca(P('8b chỉ báo trên header chuyển «Mất kết nối»'), ((await page.locator('#conn').getAttribute('class')) || '').includes('off'));
  await lam();
  // 9. hết phiên đăng nhập (401)
  await ap({}, 401);
  await doiTai();
  ca(P('9 401 ⟹ báo phiên hết hạn'), ((await page.locator('[data-testid=notice]').textContent().catch(() => '')) || '').includes('hết hạn'));
  await lam();
  // 10. rỗng thật ⟹ trạng thái «Chưa có phiên»
  await ap({ loading: false, error: null, ok_at_ms: Date.now(), now_ms: Date.now(), data: { sessions: [], notes: [] } });
  await doiTai();
  ca(P('10 danh sách rỗng thật ⟹ «Chưa có phiên»'), (await page.locator('[data-testid=empty]').count()) === 1);
  await page.screenshot({ path: `${SHOTS}/${ten}-3-rong.png` });
  await lam();
  // 11. XSS: tên phiên chứa HTML không được thành phần tử
  const doc = JSON.parse(JSON.stringify(thuc));
  doc.data.sessions[0].name = '<img src=x onerror="window.__xss=1">';
  doc.data.sessions[0].label = '<script>window.__xss=2</script>';
  doc.data.sessions[0].last_text = '"><b id=bom>x</b>';
  await ap(doc);
  await doiTai();
  const xss = await page.evaluate(() => ({ bien: window.__xss ?? null, img: document.querySelectorAll('img[src="x"]').length, bom: document.querySelectorAll('#bom').length }));
  ca(P('11 nội dung phiên có HTML/script ⟹ hiện nguyên chữ, không thực thi'), xss.bien === null && xss.img === 0 && xss.bom === 0, JSON.stringify(xss));
  ca(P('11b chữ nguy hiểm hiện đúng như văn bản'), ((await page.locator('[data-testid=row]').first().textContent()) || '').includes('<script>'));
  await lam();

  // 12. không có lỗi console / trang / request suốt cả phiên (trừ chính các ca cố ý abort/401 ở trên)
  const that = loi.filter((x) => !/net::ERR_FAILED|NetworkError|Load failed|status of 401|Failed to load resource|Blocked by Web Inspector/.test(x));
  ca(P('12 0 lỗi console/trang/yêu cầu ngoài các ca cố ý'), that.length === 0, that.slice(0, 3).join(' | '));

  await browser.close();
  return { url, goc };
}

const dau = await chayEngine('chromium', chromium);
await chayEngine('webkit', webkit);

// 13. vé đã dùng: mở lại URL có vé ở ngữ cảnh MỚI ⟹ bị từ chối (hàng rào thật, qua trình duyệt)
try {
  if (!dau) throw new Error('không có lần chạy chromium để dùng lại vé');
  const { url, goc } = dau;
  const b = await chromium.launch();
  const p = await (await b.newContext()).newPage();
  const r = await p.goto(url);
  ca('13 dùng lại vé ở trình duyệt mới ⟹ 401', r.status() === 401, `HTTP ${r.status()}`);
  const r2 = await p.goto(goc);
  ca('13b không cookie ⟹ 401', r2.status() === 401);
  await b.close();
} catch (e) {
  khongDo++;
  console.log('KHÔNG ĐO ĐƯỢC ca 13: ' + e.message.split('\n')[0]);
}

dung();
console.log(`\nKẾT QUẢ: ${sai} ca hỏng · ${khongDo} ca không đo được · ảnh: ${SHOTS}`);
process.exit(sai ? 1 : khongDo ? 2 : 0);
