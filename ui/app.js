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
const LIVE = new Set(['sessions']); // màn đã làm thật; còn lại hiện «Sắp có»
const PAGE_SIZE = 10;
const REFRESH_MS = 5000;
const STALE_MS = 30000; // bản đệm cũ hơn mức này ⟹ gọi là «dữ liệu cũ» dù không có lỗi

const S = { resp: null, netError: null, fetchedAt: 0, q: '', status: 'all', page: 1 };

function route() {
  const h = (location.hash || '#sessions').slice(1);
  return NAV.some((n) => n[0] === h) ? h : 'sessions';
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
  const cur = route();
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
        `<td><b>${esc(s.label || s.name)}</b><br><small>${esc(s.name)}</small></td>` +
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

function render() {
  renderNav();
  renderHeader();
  const id = route();
  $('#main').innerHTML = LIVE.has(id) ? renderSessions() : renderComing(id);
}

// ── sự kiện (không có trình xử lý nội dòng: CSP chặn) ───────────────────────────────────────────────────────────────────
$('#main').addEventListener('click', (e) => {
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
window.addEventListener('hashchange', render);

render();
load();
setInterval(load, REFRESH_MS);
setInterval(() => { renderHeader(); }, 1000); // đồng hồ «cũ» chạy cả khi chưa có lần tải mới
