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
import { mkdirSync, readdirSync, existsSync, statSync, openSync, readSync, closeSync } from 'node:fs';
import { homedir } from 'node:os';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const AQUI = dirname(fileURLToPath(import.meta.url));
const HUBA = resolve(process.argv[2] || resolve(AQUI, '../rust/target/debug/huba'));
const UI = resolve(AQUI, '../ui');
const SHOTS = resolve(AQUI, '../.tmp/ui-shots');
mkdirSync(SHOTS, { recursive: true });


// Nhật ký GỐC của một phiên Claude (độc lập với mã huba): ~/.claude-acc*/projects/*/<id>.jsonl; lời cuối của model trong 2 MB cuối.
function tepNhatKy(id) {
  const h = homedir();
  for (const a of readdirSync(h).filter((x) => x.startsWith('.claude-acc'))) {
    const pj = `${h}/${a}/projects`;
    if (!existsSync(pj)) continue;
    for (const d of readdirSync(pj)) {
      const f = `${pj}/${d}/${id}.jsonl`;
      if (existsSync(f)) return f;
    }
  }
  return null;
}
function loiCuoiCuaModel(f) {
  const size = statSync(f).size;
  const n = Math.min(size, 2 * 1024 * 1024);
  const buf = Buffer.alloc(n);
  const fd = openSync(f, 'r');
  readSync(fd, buf, 0, n, size - n);
  closeSync(fd);
  const dong = buf.toString('utf8').split('\n');
  for (let i = dong.length - 1; i >= 0; i--) {
    try {
      const d = JSON.parse(dong[i]);
      if (d.type !== 'assistant') continue;
      const c = d.message?.content;
      const t = Array.isArray(c) ? c.filter((x) => x.type === 'text').map((x) => x.text).join('\n') : '';
      if (t.trim()) return t;
    } catch {}
  }
  return null;
}
const chuanHoa = (t) => String(t).replace(/\s+/g, ' ').trim();
// So SÁNH nhật ký gốc (markdown thô) với chữ ĐÃ DỰNG trên màn: dấu `…` và *…* bị trình dựng bỏ (đo 25/11: «ở `%135` ch» trên nhật ký
// ⟹ «ở %135 ch» trên màn, ca K3.3 HỎNG oan ở cả hai engine). Bỏ dấu ở CẢ HAI phía; vẫn so 40 ký tự chữ thật.
const boDauMd = (t) => chuanHoa(t).replace(/[`*]/g, '');

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

  // ── K3: Chat (đọc nội dung MỘT phiên) ──
  const dsPhien = await page.evaluate(async () => (await (await fetch('/api/sessions')).json()).data.sessions);
  const claude = dsPhien.find((x) => x.kind === 'interactive' && x.host !== 'shell' && !x.working) || dsPhien.find((x) => x.kind === 'interactive' && x.host !== 'shell');
  const oc = dsPhien.find((x) => x.kind === 'opencode');
  if (claude) {
    await page.fill('#q', claude.name);
    await page.locator('[data-testid=open-chat]').first().click();
    await page.locator('[data-testid=ev]').first().waitFor({ timeout: 20000 }).catch(() => {});
    const nEv = await page.locator('[data-testid=ev]').count();
    const apiEv = await page.evaluate(async (id) => ((await (await fetch('/api/sessions/' + id + '/stream?limit=120')).json()).data?.events ?? []).filter((e) => e.kind !== 'think').length || -1, claude.session_id);
    ca(P('K3.1 bấm tên phiên ⟹ chat có sự kiện'), nEv > 0 && (await page.locator('[data-testid=chat-title]').count()) === 1, `${nEv} sự kiện · phiên ${claude.name}`);
    ca(P('K3.2 số sự kiện trên màn ≈ API (±3: phiên đang chạy có thể thêm)'), nEv > 0 && Math.abs(nEv - apiEv) <= 3, `màn ${nEv} · API ${apiEv}`);
    const f = tepNhatKy(claude.session_id);
    const cuoi = f ? loiCuoiCuaModel(f) : null;
    if (cuoi) {
      const tren = (await page.locator('[data-testid=ev][data-kind=say][data-role=assistant]').allTextContents()).map(boDauMd);
      const mau = boDauMd(cuoi).slice(0, 40);
      ca(P('K3.3 ĐỐI CHIẾU NHẬT KÝ GỐC: lời cuối của model trong tệp .jsonl có trên màn'), tren.some((t) => t.includes(mau)), `«${mau}…»`);
      // ĐỐI CHỨNG NGƯỢC: bỏ dấu markdown KHÔNG được làm phép so khớp dễ dãi tới mức câu bịa cũng trùng.
      ca(P('K3.3b đối chứng: câu bịa KHÔNG có trên màn'), !tren.some((t) => t.includes(mau.slice(0, 20) + ' ĐIỀU-KHÔNG-AI-NÓI')));
    } else {
      khongDo++;
      console.log(P('KHÔNG ĐO ĐƯỢC K3.3: không tìm thấy/đọc được nhật ký gốc của phiên ') + claude.session_id.slice(0, 8));
    }
    await page.locator('button[data-tab=goc]').click();
    const goc = (await page.locator('[data-testid=tab-goc]').textContent()) || '';
    ca(P('K3.4 tab «Nội dung gốc» nêu đúng bốn thứ chủ máy chốt (25/11)'), ['Tin nhắn', 'thu gọn', 'Bước đang chạy', 'Trả lời cuối'].every((k) => goc.includes(k)) && !goc.includes('Chưa làm'), goc.slice(0, 80));
    await page.locator('button[data-tab=phien]').click();
    ca(P('K3.4b tab «Phiên» hiện thư mục làm việc thật'), ((await page.locator('[data-testid=tab-phien]').textContent()) || '').includes(claude.cwd));
    {
      const duocGui = claude.kind === 'interactive' && claude.host === 'terminal';
      const khoa = await page.locator('#msg').isDisabled();
      ca(P(`K3.5 ô nhập (K4): phiên thật host=${claude.host} ⟹ ${duocGui ? 'BẬT' : 'KHOÁ kèm lý do'}`), duocGui ? !khoa : khoa && ((await page.locator('#msg').getAttribute('placeholder')) || '').includes('không mở trong cửa sổ Terminal'));
    }
    await page.screenshot({ path: `${SHOTS}/${ten}-4-chat.png` });
    await page.getByRole('link', { name: 'Danh sách phiên' }).click();
    await page.locator('[data-testid=row]').first().waitFor({ timeout: 5000 }).catch(() => {});
    ca(P('K3.6 «Danh sách phiên» quay về bảng'), (await page.locator('[data-testid=row]').count()) > 0);
    await page.fill('#q', '');
  } else { khongDo++; console.log(P('KHÔNG ĐO ĐƯỢC K3: không có phiên Claude nào để mở chat')); }
  if (oc) {
    await page.fill('#q', oc.name);
    await page.locator('[data-testid=open-chat]').first().click();
    await page.locator('[data-testid=chat-empty]').waitFor({ timeout: 20000 }).catch(() => {});
    const t = (await page.locator('[data-testid=chat-empty]').textContent().catch(() => '')) || '';
    ca(P('K3.7 phiên opencode: nói «chưa đọc được nội dung», KHÔNG phải khung chat rỗng'), t.includes('chưa đọc được nội dung phiên loại') && (await page.locator('[data-testid=ev]').count()) === 0, t.slice(0, 70));
    await page.evaluate(() => { location.hash = '#sessions'; });
    await page.fill('#q', '');
  }
  // 404: id không có trong danh sách
  await page.evaluate(() => { location.hash = '#chat/khong-co-phien'; });
  await page.locator('[data-testid=notice]').waitFor({ timeout: 8000 }).catch(() => {});
  ca(P('K3.8 phiên không có trong danh sách ⟹ báo «Không có phiên này»'), ((await page.locator('[data-testid=notice]').textContent().catch(() => '')) || '').includes('Không có phiên này'));
  // XSS + markdown: nội dung model có HTML/script/khối mã (ĐỐI CHỨNG GIẢ LỖI qua page.route — khai báo ở đầu tệp)
  if (claude) {
    await page.route('**/api/sessions/*/stream*', (r) => r.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ now_ms: Date.now(), error: null, data: { session_id: 'x', older_hidden: 7, events: [
      { kind: 'say', role: 'assistant', ts: null, name: '', withheld: false, text: 'x **đậm** <img src=x onerror="window.__x3=1"> `a<b>` ```js\nconst z = "<script>window.__x3=2</script>";\n```' },
      { kind: 'tool', role: 'assistant', ts: null, name: '<b>Bash</b>', text: 'ls', withheld: false },
      { kind: 'say', role: 'user', ts: null, name: '', withheld: true, text: '[đã gác]' },
    ] } }) }));
    await page.evaluate((id) => { location.hash = '#sessions'; setTimeout(() => { location.hash = '#chat/' + id; }, 50); }, claude.session_id);
    await page.locator('[data-testid=ev]').first().waitFor({ timeout: 8000 }).catch(() => {});
    const x3 = await page.evaluate(() => ({ bien: window.__x3 ?? null, img: document.querySelectorAll('img[src="x"]').length, scriptTrongBang: document.querySelectorAll('#timeline script').length }));
    ca(P('K3.9 HTML/script trong nội dung model KHÔNG thực thi'), x3.bien === null && x3.img === 0 && x3.scriptTrongBang === 0, JSON.stringify(x3));
    const code = (await page.locator('#timeline pre code').first().textContent().catch(() => '')) || '';
    ca(P('K3.9b khối mã hiện nguyên văn + có nút sao chép'), code.includes('<script>window.__x3=2</script>') && (await page.locator('#timeline button[data-copy]').count()) === 1);
    ca(P('K3.9b2 **đậm** thành thẻ strong, không còn dấu sao'), ((await page.locator('#timeline .bubble strong').first().textContent().catch(() => '')) || '') === 'đậm');
    ca(P('K3.9c tên công cụ chứa HTML hiện nguyên chữ'), ((await page.locator('.ev.tool b').first().textContent()) || '') === '<b>Bash</b>');
    ca(P('K3.9d «sự kiện cũ hơn» và «nội dung bị giữ lại» đều được nói ra'), (await page.locator('[data-testid=older]').count()) === 1 && ((await page.locator('#timeline').textContent()) || '').includes('Nội dung bị giữ lại'));
    await page.unroute('**/api/sessions/*/stream*');
    // lỗi nguồn: 500 ⟹ báo lỗi, KHÔNG hiện «chưa có hội thoại»
    await page.route('**/api/sessions/*/stream*', (r) => r.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ error: 'đọc nhật ký hỏng', data: null }) }));
    await page.evaluate(() => { location.hash = '#sessions'; });
    await page.evaluate((id) => { setTimeout(() => { location.hash = '#chat/' + id + '-khac'; }, 50); }, 'ghost');
    await page.locator('[data-testid=notice]').waitFor({ timeout: 8000 }).catch(() => {});
    ca(P('K3.10 nguồn lỗi ⟹ báo lỗi, không giả vờ «chưa có hội thoại»'), (await page.locator('[data-testid=chat-empty]').count()) === 0 && (await page.locator('[data-testid=notice]').count()) >= 1);
    await page.unroute('**/api/sessions/*/stream*');
    await page.evaluate(() => { location.hash = '#sessions'; });
  }

  // ── K3.11 bốn thứ chủ máy chốt (K3.5, 25/11): tin nhắn · bước THU GỌN · bước ĐANG CHẠY + trạng thái · trả lời CUỐI ──
  // ĐỐI CHỨNG GIẢ DỮ LIỆU qua page.route (khai báo như ca K3.9): nhật ký thật không cho dựng trước «đang chạy» / «đã xong» theo lệnh.
  if (claude) {
    const thuc0 = await page.evaluate(async () => await (await fetch('/api/sessions')).json());
    const sev = (role, text) => ({ kind: 'say', role, ts: null, name: '', withheld: false, text });
    const tool = (name, text) => ({ kind: 'tool', role: 'assistant', ts: null, name, withheld: false, text });
    const kq = (text) => ({ kind: 'result', role: 'user', ts: null, name: '', withheld: false, text });
    const nghi = { kind: 'think', role: 'assistant', ts: null, name: '', withheld: false, text: 'BI-MAT-SUY-NGHI' };
    const dung = async (events, working, extra = {}) => {
      await page.unroute('**/api/sessions/*/stream*').catch(() => {});
      await page.unroute('**/api/sessions').catch(() => {});
      const ds = JSON.parse(JSON.stringify(thuc0));
      ds.data.sessions = ds.data.sessions.map((x) => (x.session_id === claude.session_id ? { ...x, working, asking_password: false, ...extra } : x));
      await page.route('**/api/sessions', (r) => r.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(ds) }));
      await page.route('**/api/sessions/*/stream*', (r) => r.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ now_ms: Date.now(), error: null, data: { session_id: claude.session_id, older_hidden: 0, events } }) }));
      await page.evaluate(() => { location.hash = '#sessions'; });
      await page.evaluate((id) => { setTimeout(() => { location.hash = '#chat/' + id; }, 50); }, claude.session_id);
      await page.waitForTimeout(400);
      await page.evaluate(() => window.dispatchEvent(new Event('hashchange')));
      await page.waitForTimeout(600);
      await page.evaluate(() => load()); // ép tải lại danh sách phiên NGAY (không chờ nhịp 5 giây) để hàng phiên giả có hiệu lực
      await page.waitForTimeout(400);
    };
    const xong = [sev('user', 'làm đi'), nghi, tool('Read', 'a.rs'), kq('nội dung a'), tool('Bash', 'ls -la'), kq('total 0'), sev('assistant', 'XONG-RỒI-NÈ')];
    await dung(xong, false);
    const tt = (await page.locator('#timeline').textContent()) || '';
    ca(P('K3.11a phiên rảnh: các bước gộp THU GỌN (1 ô, đóng sẵn, ghi «2 bước · Read ×1 · Bash ×1»)'),
      (await page.locator('details[data-testid=steps]').count()) === 1 && (await page.locator('details[data-testid=steps][open]').count()) === 0 && tt.includes('2 bước') && tt.includes('Read ×1') && tt.includes('Bash ×1'), tt.slice(0, 120));
    ca(P('K3.11b trả lời cuối được đánh dấu — đúng MỘT, là câu của model'), (await page.locator('[data-final]').count()) === 1 && ((await page.locator('[data-final]').textContent()) || '').includes('Trả lời cuối') && ((await page.locator('[data-final]').textContent()) || '').includes('XONG-RỒI-NÈ'));
    ca(P('K3.11c không có «bước đang chạy» khi phiên rảnh · «suy nghĩ» KHÔNG hiện'), (await page.locator('[data-testid=running]').count()) === 0 && !tt.includes('BI-MAT-SUY-NGHI'));
    const dangChay = [sev('user', 'chạy thử'), tool('Read', 'a.rs'), kq('nội dung a'), tool('Bash', 'cargo test --release')];
    await dung(dangChay, true);
    const rt = (await page.locator('[data-testid=running]').textContent().catch(() => '')) || '';
    ca(P('K3.11d phiên đang làm: bước ĐANG CHẠY hiện thẳng (tên + lệnh) kèm trạng thái «Đang làm»'),
      (await page.locator('[data-testid=running]').count()) === 1 && rt.includes('Đang chạy') && rt.includes('Bash') && rt.includes('cargo test --release') && ((await page.locator('[data-testid=running-state]').textContent()) || '').includes('Đang làm'), rt.slice(0, 120));
    ca(P('K3.11e bước TRƯỚC đó vẫn thu gọn (Read ×1) và chưa có «trả lời cuối»'),
      (await page.locator('[data-testid=running] details[data-testid=steps]:not([open])').count()) === 1 && rt.includes('Read ×1') && (await page.locator('[data-final]').count()) === 0);
    const dangNoi = [sev('user', 'viết giúp'), tool('Read', 'a.rs'), kq('nội dung a'), sev('assistant', 'ĐANG-VIẾT-DỞ')];
    await dung(dangNoi, true);
    ca(P('K3.11f ĐỐI CHỨNG: đang làm mà model vừa nói ⟹ KHÔNG gắn «Trả lời cuối» (chưa xong lượt)'), (await page.locator('[data-final]').count()) === 0 && ((await page.locator('#timeline').textContent()) || '').includes('ĐANG-VIẾT-DỞ'));
    await page.screenshot({ path: `${SHOTS}/${ten}-5-chat-buoc.png` });

    // ── K4 gửi tin từ giao diện. ĐỐI CHỨNG GIẢ qua page.route (khai báo như K3.9/K3.11): bấm «Gửi» thật trên giao diện thật, nhưng
    // máy chủ trả lời GIẢ — để dựng đủ bốn kết cục (đã nhận · còn trong ô · chưa xác nhận · bị từ chối) và lỗi mạng mà không
    // gõ vào phiên Claude thật nào. Đường gõ THẬT vào Terminal có bài riêng (rust/tests + nghiệm thu trên phiên bỏ đi).
    {
      const goi = [];
      let tra = { status: 200, data: { result: 'sent', message: 'Phiên đã nhận tin.' }, tre: 0, huy: false };
      await page.route('**/api/sessions/*/send', async (r) => {
        const req = r.request();
        let body = {};
        try { body = JSON.parse(req.postData() || '{}'); } catch {}
        goi.push({ body, ct: req.headers()['content-type'], method: req.method() });
        if (tra.tre) await new Promise((ok) => setTimeout(ok, tra.tre));
        if (tra.huy) return r.abort('failed');
        return r.fulfill({ status: tra.status, contentType: 'application/json', body: JSON.stringify({ now_ms: Date.now(), error: null, data: { replayed: false, ...tra.data } }) });
      });
      const nhap = async (chu) => { await page.locator('#msg').click(); await page.locator('#msg').fill(chu); };
      const nhanGui = () => page.locator('[data-testid=send]').click();
      const ket = async () => (await page.locator('[data-testid=send-result]').getAttribute('data-result').catch(() => null));
      const rong = [sev('user', 'xin chào'), sev('assistant', 'chào anh')];

      await dung(rong, false, { host: 'terminal', kind: 'interactive' });
      ca(P('K4.1 phiên Terminal: ô nhập + nút Gửi BẬT'), !(await page.locator('#msg').isDisabled()) && !(await page.locator('[data-testid=send]').isDisabled()));

      await nhap('làm tiếp việc đang dở nhé');
      await nhanGui();
      await page.locator('[data-testid=send-result]').waitFor({ timeout: 5000 }).catch(() => {});
      ca(P('K4.2 «đã nhận»: thân yêu cầu đúng (JSON, text nguyên văn, client_id) + nháp được XOÁ'),
        goi.length === 1 && goi[0].method === 'POST' && (goi[0].ct || '').includes('application/json') && goi[0].body.text === 'làm tiếp việc đang dở nhé' && /^[A-Za-z0-9_-]{8,64}$/.test(goi[0].body.client_id || '') && (await ket()) === 'sent' && (await page.locator('#msg').inputValue()) === '', JSON.stringify(goi[0] && goi[0].body));
      const id1 = goi[0] && goi[0].body.client_id;

      tra = { status: 200, data: { result: 'unverified', message: 'Màn khai tin đã đi, nhưng nhật ký phiên CHƯA thấy.' }, tre: 0, huy: false };
      await nhap('tin mơ hồ');
      await nhanGui();
      await page.locator('[data-result=unverified]').waitFor({ timeout: 5000 }).catch(() => {});
      ca(P('K4.3 «chưa xác nhận»: nháp GIỮ NGUYÊN + câu nói đúng kết cục'), (await ket()) === 'unverified' && (await page.locator('#msg').inputValue()) === 'tin mơ hồ' && ((await page.locator('[data-testid=send-result]').textContent()) || '').includes('Chưa xác nhận'));
      await page.screenshot({ path: `${SHOTS}/${ten}-6a-gui-tin-chua-xac-nhan.png` });
      await page.waitForTimeout(5800); // qua ≥1 nhịp làm mới 5 giây
      ca(P('K4.4 KHÔNG TỰ GỬI LẠI: sau một nhịp làm mới vẫn chỉ 2 yêu cầu, nháp vẫn còn'), goi.length === 2 && (await page.locator('#msg').inputValue()) === 'tin mơ hồ', `yêu cầu=${goi.length}`);
      await nhanGui();
      await page.waitForTimeout(500);
      ca(P('K4.5 bấm lại là MỘT quyết định mới: client_id KHÁC lần trước'), goi.length === 3 && goi[2].body.client_id !== goi[1].body.client_id && goi[2].body.client_id !== id1);

      tra = { status: 200, data: { result: 'in_box', message: 'Chữ vẫn nằm trong ô nhập của phiên.' }, tre: 0, huy: false };
      await nhanGui();
      await page.locator('[data-result=in_box]').waitFor({ timeout: 5000 }).catch(() => {});
      ca(P('K4.6 «còn trong ô nhập»: nháp giữ, nói rõ'), (await ket()) === 'in_box' && ((await page.locator('[data-testid=send-result]').textContent()) || '').includes('Còn trong ô nhập') && (await page.locator('#msg').inputValue()) === 'tin mơ hồ');

      tra = { status: 200, data: { result: 'refused', message: 'Phiên đang hỏi / mở hộp chọn.' }, tre: 0, huy: false };
      await nhanGui();
      await page.locator('[data-result=refused]').waitFor({ timeout: 5000 }).catch(() => {});
      ca(P('K4.7 «bị từ chối»: hiện LÝ DO của máy chủ, nháp giữ'), ((await page.locator('[data-testid=send-result]').textContent()) || '').includes('Phiên đang hỏi') && (await page.locator('#msg').inputValue()) === 'tin mơ hồ');

      tra = { status: 200, data: { result: 'sent', message: 'ok' }, tre: 0, huy: true };
      const t8 = goi.length;
      await nhanGui();
      await page.locator('[data-result=error]').waitFor({ timeout: 5000 }).catch(() => {});
      ca(P('K4.8 mất mạng giữa chừng: nói «có thể ĐÃ đi», nháp giữ, KHÔNG tự gửi lại'), ((await page.locator('[data-testid=send-result]').textContent()) || '').includes('có thể ĐÃ đi') && (await page.locator('#msg').inputValue()) === 'tin mơ hồ' && goi.length === t8 + 1, `yêu cầu thêm=${goi.length - t8}`);

      // bấm đúp: nút khoá trong lúc đang gửi ⟹ đúng MỘT yêu cầu
      tra = { status: 200, data: { result: 'sent', message: 'ok' }, tre: 900, huy: false };
      const truoc = goi.length;
      await page.locator('[data-testid=send]').dblclick();
      await page.waitForTimeout(1500);
      ca(P('K4.9 bấm đúp «Gửi» chỉ đi MỘT yêu cầu'), goi.length === truoc + 1, `thêm=${goi.length - truoc}`);

      // nháp sống qua việc dựng lại trang (mỗi vài giây) — gõ dở không bị mất, con trỏ không nhảy
      tra = { status: 200, data: { result: 'sent', message: 'ok' }, tre: 0, huy: false };
      await nhap('gõ dở dang');
      await page.locator('#msg').press('End');
      await page.waitForTimeout(5800);
      const giu = await page.evaluate(() => ({ v: document.getElementById('msg').value, f: document.activeElement && document.activeElement.id }));
      ca(P('K4.10 gõ dở qua một nhịp làm mới: chữ còn, ô vẫn đang focus'), giu.v === 'gõ dở dang' && giu.f === 'msg', JSON.stringify(giu));

      // Ctrl+Enter gửi
      const t2 = goi.length;
      await page.locator('#msg').press('Control+Enter');
      await page.waitForTimeout(600);
      ca(P('K4.11 Ctrl+Enter gửi'), goi.length === t2 + 1 && goi[goi.length - 1].body.text === 'gõ dở dang');

      // phiên KHÔNG ở Terminal ⟹ khoá + lý do (không có đường gửi)
      await dung(rong, false, { host: 'editor', kind: 'interactive' });
      const phk = (await page.locator('#msg').getAttribute('placeholder')) || '';
      ca(P('K4.12 phiên chạy ở editor: ô KHOÁ + nêu lý do'), (await page.locator('#msg').isDisabled()) && (await page.locator('[data-testid=send]').isDisabled()) && phk.includes('không mở trong cửa sổ Terminal'), phk.slice(0, 60));
      await page.screenshot({ path: `${SHOTS}/${ten}-6-gui-tin.png` });
      await page.unroute('**/api/sessions/*/send').catch(() => {});
    }
    await page.unroute('**/api/sessions/*/stream*').catch(() => {});
    await page.unroute('**/api/sessions').catch(() => {});
    await page.evaluate(() => { location.hash = '#sessions'; });
  }

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
