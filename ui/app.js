'use strict';
// Giao diện THẬT của huba (không phải bản mẫu docs/ui). Không bước build, không phụ thuộc; chạy dưới CSP `default-src 'self'`.
// Dữ liệu từ `/api/sessions` (huba ui đệm snapshot ~10 giây). Mọi chuỗi động đi qua esc(): tên phiên/nội dung cuối đến từ
// model và người dùng, KHÔNG được dựng thành HTML.

const $ = (s, r = document) => r.querySelector(s);
const esc = (v) =>
  String(v ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

const NAV = [
  ['chat', '◫', 'Chat'], ['sessions', '▦', 'Phiên'], ['reports', '▥', 'Báo cáo token'], ['accounts', '♙', 'Tài khoản & model'],
  ['connections', '⌁', 'Kết nối'], ['projects', '▱', 'Dự án'], ['activity', '≋', 'Hoạt động'], ['settings', '⚙', 'Cài đặt'],
];
const LIVE = new Set(['sessions', 'chat']); // màn đã làm thật; còn lại hiện «Sắp có»
const PAGE_SIZE = 10;
const REFRESH_MS = 5000;
const STALE_MS = 30000; // bản đệm cũ hơn mức này ⟹ gọi là «dữ liệu cũ» dù không có lỗi

const S = { resp: null, netError: null, fetchedAt: 0, q: '', status: 'all', page: 1 };

function route() {
  const [h, arg] = (location.hash || '#sessions').slice(1).split('/');
  let a = '';
  try { a = arg ? decodeURIComponent(arg) : ''; } catch { a = ''; }
  return { name: NAV.some((n) => n[0] === h) ? h : 'sessions', arg: a };
}

// ── dữ liệu ────────────────────────────────────────────────────────────────────────────────────────────────────────
async function load() {
  try {
    const r = await fetch('/api/sessions', { credentials: 'same-origin', cache: 'no-store' });
    if (r.status === 401) S.netError = 'Phiên đăng nhập hết hạn — chạy lại `huba ui` để lấy liên kết mới';
    else if (!r.ok) S.netError = 'Máy chủ huba trả mã ' + r.status;
    else {
      S.resp = await r.json();
      S.netError = null;
      S.fetchedAt = Date.now();
    }
  } catch (e) {
    S.netError = 'Mất kết nối tới huba (' + e.message + ')';
  }
  render();
}

function sessions() {
  return (S.resp && S.resp.data && Array.isArray(S.resp.data.sessions)) ? S.resp.data.sessions : null;
}

/** Tuổi bản đệm (ms), tính cả thời gian từ lần tải cuối; null nếu chưa từng đọc tốt. */
function ageMs() {
  if (!S.resp || S.resp.ok_at_ms == null) return null;
  return S.resp.now_ms - S.resp.ok_at_ms + (Date.now() - S.fetchedAt);
}

function health() {
  const age = ageMs();
  if (S.netError) return { cls: 'off', text: 'Mất kết nối', stale: true };
  if (S.resp && S.resp.error) return { cls: 'warn', text: 'Dữ liệu cũ', stale: true };
  if (age != null && age > STALE_MS) return { cls: 'warn', text: 'Dữ liệu cũ', stale: true };
  if (S.resp && S.resp.loading) return { cls: 'warn', text: 'Đang đọc…', stale: false };
  return { cls: '', text: 'Đã kết nối', stale: false };
}

// ── hiển thị một phiên ───────────────────────────────────────────────────────────────────────────────────────────────
function stateOf(s) {
  if (s.host === 'shell' || s.kind === 'shell') return { key: 'shell', label: 'Cửa sổ trống', cls: 'gray' };
  if (s.asking_password) return { key: 'asking', label: 'Cần mật khẩu', cls: 'red' };
  if (s.working) return { key: 'working', label: 'Đang làm', cls: '' };
  return { key: 'idle', label: 'Đang chờ bạn', cls: 'amber' };
}
const base = (p) => String(p || '').split('/').filter(Boolean).pop() || '';
function relTime(iso) {
  const t = Date.parse(iso || '');
  if (!t) return '—';
  const s = Math.max(0, Math.round((Date.now() - t) / 1000));
  if (s < 60) return s + ' giây trước';
  if (s < 3600) return Math.floor(s / 60) + ' phút trước';
  if (s < 86400) return Math.floor(s / 3600) + ' giờ trước';
  return Math.floor(s / 86400) + ' ngày trước';
}
const clock = (ms) => (ms ? new Date(ms).toLocaleTimeString('vi-VN') : '—');
const cut = (t, n) => { const s = String(t ?? '').replace(/\s+/g, ' ').trim(); return s.length > n ? s.slice(0, n - 1) + '…' : s; };

function filtered(all) {
  const q = S.q.trim().toLowerCase();
  return all.filter((s) => {
    if (S.status !== 'all' && stateOf(s).key !== S.status) return false;
    if (!q) return true;
    return [s.label, s.name, s.folder, s.cwd, s.account, s.last_text].some((f) => String(f ?? '').toLowerCase().includes(q));
  });
}

// ── dựng trang ───────────────────────────────────────────────────────────────────────────────────────────────────────
function renderNav() {
  const cur = route().name;
  $('#nav').innerHTML = NAV.map(([id, icon, label]) => {
    const live = LIVE.has(id);
    return `<a href="#${id}" title="${esc(label)}" class="nav-item ${cur === id ? 'active' : ''}${live ? '' : ' soon'}"` +
      `${cur === id ? ' aria-current="page"' : ''}>` +
      `<span class="nav-icon" aria-hidden="true">${icon}</span><span class="nav-label">${esc(label)}</span>` +
      `${live ? '' : '<span class="nav-badge" aria-hidden="true">Sắp có</span>'}</a>`;
  }).join('');
}

function renderHeader() {
  const h = health();
  const el = $('#conn');
  el.className = 'local-status ' + h.cls;
  const at = S.resp && S.resp.ok_at_ms ? ' · đọc lúc ' + clock(S.resp.ok_at_ms) : '';
  $('#conn-text').textContent = h.text + at;
}

function notice() {
  if (S.netError) return `<div class="notice error" role="alert" data-testid="notice">⚠ ${esc(S.netError)}${sessions() ? ' — đang hiện dữ liệu CŨ.' : ''}</div>`;
  if (S.resp && S.resp.error) {
    return `<div class="notice error" role="alert" data-testid="notice">⚠ Lần đọc gần nhất lỗi: ${esc(S.resp.error)}.` +
      `${sessions() ? ` Đang hiện dữ liệu CŨ (đọc tốt lúc ${esc(clock(S.resp.ok_at_ms))}).` : ''}</div>`;
  }
  const age = ageMs();
  if (age != null && age > STALE_MS) {
    return `<div class="notice" role="status" data-testid="notice">Dữ liệu cũ: lần đọc tốt cuối cách đây ${Math.round(age / 1000)} giây.</div>`;
  }
  return '';
}

function renderSessions() {
  const all = sessions();
  let body;
  if (S.resp === null && !S.netError) {
    body = '<div class="card" data-testid="loading"><div class="skeleton"></div><div class="skeleton"></div><div class="skeleton"></div></div>';
  } else if (S.resp && S.resp.loading && all === null) {
    body = '<div class="card" data-testid="loading"><p>Đang đọc danh sách phiên lần đầu (mất vài giây)…</p><div class="skeleton"></div><div class="skeleton"></div></div>';
  } else if (all === null) {
    body = '<div class="card empty" data-testid="no-data"><div class="empty-icon">⚠</div><h2>Chưa có dữ liệu phiên</h2><p>Chưa đọc được danh sách phiên. Xem thông báo phía trên.</p></div>';
  } else if (all.length === 0) {
    body = '<div class="card empty" data-testid="empty"><div class="empty-icon">▦</div><h2>Chưa có phiên</h2><p>Mở phiên mới từ Telegram bằng <b>/new</b>.</p></div>';
  } else {
    const rows = filtered(all);
    const pages = Math.max(1, Math.ceil(rows.length / PAGE_SIZE));
    if (S.page > pages) S.page = pages;
    const from = (S.page - 1) * PAGE_SIZE;
    const slice = rows.slice(from, from + PAGE_SIZE);
    const cnt = (k) => all.filter((s) => stateOf(s).key === k).length;
    const tr = slice.map((s, i) => {
      const st = stateOf(s);
      return `<tr data-testid="row" data-id="${esc(s.session_id)}"><td>${from + i + 1}</td>` +
        `<td><a class="sess-link" href="#chat/${encodeURIComponent(s.session_id)}" data-testid="open-chat"><b>${esc(s.label || s.name)}</b></a><br><small>${esc(s.name)}</small></td>` +
        `<td>${esc(s.folder || base(s.cwd) || '—')}</td><td>${esc(s.account || '—')}</td>` +
        `<td><span class="pill ${st.cls}"><span class="dot"></span>${esc(st.label)}</span></td>` +
        `<td title="${esc(s.last_activity)}">${esc(relTime(s.last_activity))}</td>` +
        `<td class="last-text">${esc(cut(s.last_text, 90))}</td></tr>`;
    }).join('');
    body = `<div class="summary-grid">` +
      `<div class="card summary"><span>Tổng số phiên</span><strong data-testid="n-total">${all.length}</strong></div>` +
      `<div class="card summary"><span>Đang làm</span><strong data-testid="n-working">${cnt('working')}</strong></div>` +
      `<div class="card summary"><span>Đang chờ bạn</span><strong data-testid="n-idle">${cnt('idle')}</strong></div></div>` +
      `<div class="card"><div class="toolbar"><div class="actions"><label for="status" class="sr-only">Lọc trạng thái</label>` +
      `<select id="status">${[['all', 'Mọi trạng thái'], ['working', 'Đang làm'], ['idle', 'Đang chờ bạn'], ['asking', 'Cần mật khẩu'], ['shell', 'Cửa sổ trống']]
        .map(([v, t]) => `<option value="${v}"${S.status === v ? ' selected' : ''}>${t}</option>`).join('')}</select></div></div>` +
      (rows.length === 0
        ? '<div class="empty" data-testid="no-match"><h2>Không có phiên khớp bộ lọc</h2><p>Đổi từ khoá hoặc trạng thái.</p></div>'
        : `<div class="table-wrap"><table><thead><tr><th>STT</th><th>Phiên</th><th>Dự án</th><th>Tài khoản</th><th>Trạng thái</th><th>Hoạt động cuối</th><th>Nội dung cuối</th></tr></thead><tbody>${tr}</tbody></table></div>` +
          `<div class="table-footer"><span data-testid="range">Hiện ${from + 1}–${from + slice.length} / ${rows.length} phiên${rows.length !== all.length ? ` (lọc từ ${all.length})` : ''}</span>` +
          `<div class="pager"><button data-act="prev"${S.page <= 1 ? ' disabled' : ''}>‹ Trước</button><span class="current" data-testid="page">Trang ${S.page}/${pages}</span><button data-act="next"${S.page >= pages ? ' disabled' : ''}>Sau ›</button></div></div>`) +
      `</div>`;
  }
  return `<div class="page"><div class="page-top"><div><div class="eyebrow">Phiên</div><h1>Phiên trên máy này</h1>` +
    `<p>Đọc từ huba trên máy này, làm mới mỗi ${REFRESH_MS / 1000} giây.</p></div></div>${notice()}${body}</div>`;
}

function renderComing(id) {
  const n = NAV.find((x) => x[0] === id);
  return `<div class="page"><div class="card empty" data-testid="coming"><div class="empty-icon">${n[1]}</div><h2>${esc(n[2])} — sắp có</h2>` +
    `<p>Màn này chưa được làm. Kế hoạch: <code>docs/ui/ke-hoach-cong-viec.md</code>.</p></div></div>`;
}

// ── Chat: xem nội dung MỘT phiên (K3). Chỉ đọc — gửi từ ứng dụng là K4. ────────────────────────────────────────────────
const C = { id: '', data: null, error: null, netError: null, fetchedAt: 0, tab: 'phien', loading: false };

async function loadChat() {
  const { name, arg } = route();
  if (name !== 'chat' || !arg) return;
  if (C.id !== arg) { C.id = arg; C.data = null; C.error = null; C.netError = null; C.tab = 'phien'; }
  C.loading = C.data === null;
  try {
    const r = await fetch('/api/sessions/' + encodeURIComponent(arg) + '/stream?limit=120', { credentials: 'same-origin', cache: 'no-store' });
    const j = await r.json().catch(() => ({}));
    if (r.status === 401) C.netError = 'Phiên đăng nhập hết hạn — chạy lại `huba ui` để lấy liên kết mới';
    else if (r.status === 404) { C.error = 'Không có phiên này trong danh sách (có thể đã tắt).'; C.data = null; C.netError = null; }
    else if (!r.ok) { C.error = j.error || 'Máy chủ huba trả mã ' + r.status; C.netError = null; }
    else { C.data = j.data; C.error = null; C.netError = null; C.fetchedAt = Date.now(); }
  } catch (e) {
    C.netError = 'Mất kết nối tới huba (' + e.message + ')';
  }
  C.loading = false;
  if (route().name === 'chat' && route().arg === arg) render();
}

/** Markdown AN TOÀN tối thiểu: khối ```, `mã` và **đậm** (xử lý SAU khi escape) — mọi thứ khác là chữ thường đã escape; không liên kết, không HTML. */
function md(text) {
  const parts = String(text ?? '').split('```');
  return parts.map((seg, i) => {
    if (i % 2 === 1) {
      const code = seg.replace(/^[^\n]*\n/, (m) => (/^[A-Za-z0-9_+-]*\n$/.test(m) ? '' : m)); // bỏ dòng tên ngôn ngữ
      return `<div class="codeblock"><button type="button" class="copy" data-copy>Sao chép</button><pre><code>${esc(code.replace(/\n$/, ''))}</code></pre></div>`;
    }
    return esc(seg).replace(/`([^`\n]+)`/g, '<code>$1</code>').replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>').replace(/\n/g, '<br>');
  }).join('');
}

function eventHtml(ev) {
  const who = ev.role === 'user' ? 'Bạn' : 'Claude';
  const when = ev.ts ? ` · ${esc(new Date(ev.ts).toLocaleTimeString('vi-VN'))}` : '';
  const held = ev.withheld ? '<span class="pill red">Nội dung bị giữ lại</span> ' : '';
  if (ev.kind === 'say') {
    return `<div class="ev say ${esc(ev.role)}" data-testid="ev" data-kind="say" data-role="${esc(ev.role)}"><div class="who">${who}${when}</div>` +
      `<div class="bubble">${held}${md(ev.text)}</div></div>`;
  }
  if (ev.kind === 'tool') {
    return `<div class="ev tool" data-testid="ev" data-kind="tool">▸ <b>${esc(ev.name || 'công cụ')}</b> <code>${esc(cut(ev.text, 160))}</code></div>`;
  }
  const label = ev.kind === 'think' ? 'Suy nghĩ' : 'Kết quả công cụ';
  return `<details class="ev ${esc(ev.kind)}" data-testid="ev" data-kind="${esc(ev.kind)}"><summary>${label}${ev.name ? ' · ' + esc(ev.name) : ''}</summary>` +
    `<pre>${esc(cut(ev.text, 4000))}</pre></details>`;
}

function chatMeta(id) {
  const all = sessions() || [];
  return all.find((x) => x.session_id === id) || null;
}

function renderChat() {
  const { arg } = route();
  if (!arg) {
    return `<div class="page"><div class="page-top"><div><div class="eyebrow">Chat</div><h1>Chọn một phiên</h1></div></div>` +
      `<div class="card empty" data-testid="chat-pick"><div class="empty-icon">◫</div><h2>Chưa chọn phiên</h2>` +
      `<p>Mở màn <a href="#sessions">Phiên</a> rồi bấm tên một phiên để xem nội dung.</p></div></div>`;
  }
  if (C.id !== arg) { C.id = arg; C.data = null; C.error = null; C.netError = null; C.tab = 'phien'; loadChat(); }
  const m = chatMeta(arg);
  const evs = C.data && Array.isArray(C.data.events) ? C.data.events : null;
  let tl;
  if (C.netError && evs === null) tl = `<div class="notice error" role="alert" data-testid="notice">⚠ ${esc(C.netError)}</div>`;
  else if (C.error && evs === null) tl = `<div class="notice error" role="alert" data-testid="notice">⚠ ${esc(C.error)}</div>`;
  else if (evs === null) tl = '<div data-testid="loading"><div class="skeleton"></div><div class="skeleton"></div><div class="skeleton"></div></div>';
  else if (evs.length === 0) {
    tl = `<div class="empty" data-testid="chat-empty"><h2>${C.data.note ? 'Chưa đọc được nội dung' : 'Chưa có hội thoại'}</h2>` +
      `<p>${esc(C.data.note || 'Phiên này chưa có lượt nào đọc được trong nhật ký.')}</p></div>`;
  } else {
    tl = (C.data.older_hidden > 0 ? `<div class="notice info" data-testid="older">${C.data.older_hidden} sự kiện cũ hơn chưa hiện.</div>` : '') + evs.map(eventHtml).join('');
  }
  const warn = (C.netError && evs !== null ? `<div class="notice error" role="alert" data-testid="notice">⚠ ${esc(C.netError)} — đang hiện nội dung CŨ.</div>` : '') +
    (C.error && evs !== null ? `<div class="notice error" role="alert" data-testid="notice">⚠ ${esc(C.error)} — đang hiện nội dung CŨ.</div>` : '');
  const st = m ? stateOf(m) : null;
  const tabs = [['phien', 'Phiên'], ['goc', 'Nội dung gốc'], ['hoat-dong', 'Hoạt động']];
  let panel;
  if (C.tab === 'goc') panel = '<p data-testid="tab-goc">Chưa làm. Cần chủ máy chốt mức dữ liệu «gốc» cần xem (kế hoạch K3.5) — không tự đoán.</p>';
  else if (C.tab === 'hoat-dong') panel = '<p data-testid="tab-hd">Sắp có.</p>';
  else if (!m) panel = '<p data-testid="tab-phien">Phiên này không còn trong danh sách phiên đang chạy.</p>';
  else panel = `<dl class="meta" data-testid="tab-phien"><dt>Tài khoản</dt><dd>${esc(m.account || '—')}</dd><dt>Loại</dt><dd>${esc(m.kind)}</dd>` +
    `<dt>Thư mục</dt><dd>${esc(m.cwd || '—')}</dd><dt>Trạng thái</dt><dd>${esc(st.label)}</dd><dt>Hoạt động cuối</dt><dd>${esc(relTime(m.last_activity))}</dd>` +
    `<dt>Mã phiên</dt><dd><code>${esc(m.session_id)}</code></dd></dl>`;
  return `<div class="page chat-page"><div class="page-top"><div><div class="eyebrow">Chat</div>` +
    `<h1 data-testid="chat-title">${esc(m ? (m.label || m.name) : arg)}</h1>` +
    `<p>${st ? `<span class="pill ${st.cls}"><span class="dot"></span>${esc(st.label)}</span> · ` : ''}Chỉ xem — làm mới mỗi ${REFRESH_MS / 1000} giây.</p></div>` +
    `<div class="actions"><a class="link-button" href="#sessions">‹ Danh sách phiên</a></div></div>${warn}` +
    `<div class="chat-wrap"><section class="timeline" id="timeline" aria-label="Nội dung phiên">${tl}</section>` +
    `<aside class="details"><div class="tabs" role="tablist">${tabs.map(([k, t]) => `<button type="button" role="tab" class="tab${C.tab === k ? ' on' : ''}" aria-selected="${C.tab === k}" data-tab="${k}">${t}</button>`).join('')}</div>` +
    `<div class="tabpanel">${panel}</div></aside></div>` +
    `<div class="composer" data-testid="composer"><label for="msg" class="sr-only">Tin nhắn</label>` +
    `<textarea id="msg" disabled rows="2" placeholder="Gửi từ ứng dụng chưa có (bước K4). Hiện chỉ xem; gõ vào phiên bằng Telegram hoặc terminal."></textarea>` +
    `<button type="button" disabled>Gửi</button></div></div>`;
}

/** Giữ vị trí cuộn của khung hội thoại qua mỗi lần dựng lại; đang ở đáy thì bám đáy. */
function chatScroll() {
  const t = document.getElementById('timeline');
  if (!t) return null;
  return { top: t.scrollTop, bottom: t.scrollHeight - t.scrollTop - t.clientHeight < 40 };
}
function chatRestore(k) {
  const t = document.getElementById('timeline');
  if (!t) return;
  if (k === null) { t.scrollTop = t.scrollHeight; return; } // lần đầu vào: xuống cuối (tin mới nhất)
  t.scrollTop = k.bottom ? t.scrollHeight : k.top;
}

function render() {
  renderNav();
  renderHeader();
  const { name: id } = route();
  const keep = chatScroll();
  $('#main').innerHTML = id === 'chat' ? renderChat() : LIVE.has(id) ? renderSessions() : renderComing(id);
  chatRestore(keep);
}

// ── sự kiện (không có trình xử lý nội dòng: CSP chặn) ───────────────────────────────────────────────────────────────────
$('#main').addEventListener('click', (e) => {
  const cp = e.target.closest('button[data-copy]');
  if (cp) {
    const code = cp.parentElement.querySelector('code');
    navigator.clipboard?.writeText(code ? code.textContent : '').then(() => { cp.textContent = 'Đã chép'; setTimeout(() => (cp.textContent = 'Sao chép'), 1200); }).catch(() => { cp.textContent = 'Không chép được'; });
    return;
  }
  const tb = e.target.closest('button[data-tab]');
  if (tb) { C.tab = tb.dataset.tab; render(); return; }
  const b = e.target.closest('button[data-act]');
  if (!b || b.disabled) return;
  S.page += b.dataset.act === 'next' ? 1 : -1;
  render();
});
$('#main').addEventListener('change', (e) => {
  if (e.target.id === 'status') { S.status = e.target.value; S.page = 1; render(); }
});
$('#q').addEventListener('input', (e) => { S.q = e.target.value; S.page = 1; render(); });
$('#collapse').addEventListener('click', () => {
  const c = document.body.classList.toggle('collapsed');
  $('#collapse').setAttribute('aria-expanded', String(!c));
  $('#collapse').setAttribute('aria-label', c ? 'Mở rộng menu' : 'Thu gọn menu');
});
window.addEventListener('hashchange', () => { render(); loadChat(); });

render();
load();
setInterval(load, REFRESH_MS);
setInterval(loadChat, REFRESH_MS);
loadChat();
setInterval(() => { renderHeader(); }, 1000); // đồng hồ «cũ» chạy cả khi chưa có lần tải mới
