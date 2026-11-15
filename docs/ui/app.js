// Bản mẫu thiết kế độc lập: không gọi mạng, CLI, MCP, filesystem hay model.
// Mọi thay đổi chỉ nằm trong bộ nhớ và mất khi tải lại. Không nhập bí mật thật.
'use strict';
const $ = s => document.querySelector(s);
const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const navigation = [
  ['chat','◫','Chat','3'], ['sessions','▦','Phiên',''], ['reports','▥','Báo cáo token',''], ['accounts','♙','Tài khoản & model',''],
  ['connections','⌁','Kết nối',''], ['projects','▱','Dự án',''], ['activity','≋','Hoạt động',''], ['settings','⚙','Cài đặt','']
];
const sessions = [
  {id:'demo-ui',name:'Thiết kế ứng dụng desktop',project:'huba',lane:'ui',account:'Tài khoản C',cli:'Codex',model:'Theo cấu hình tài khoản',state:'Đang làm',preview:'Thiết kế một giao diện dùng chung trên mọi PC.',color:'',time:'Vừa xong'},
  {id:'demo-review',name:'Rà soát luồng đăng nhập',project:'Ví dụ dự án',lane:'review',account:'Tài khoản A',cli:'Claude',model:'Theo cấu hình tài khoản',state:'Chờ bạn',preview:'Cần bạn chọn cách xử lý phiên đăng nhập cũ.',color:'amber',time:'2 phút trước'},
  {id:'demo-docs',name:'Cập nhật tài liệu hướng dẫn',project:'Ví dụ dự án',lane:'docs',account:'Tài khoản B',cli:'OpenCode',model:'Theo cấu hình tài khoản',state:'Sẵn sàng',preview:'Đã tổng hợp các bước sử dụng.',color:'gray',time:'12 phút trước'}
];
let accounts = [
  {name:'Tài khoản A',cli:'Claude',status:'Đã đăng nhập',model:'Theo CLI',quota:'Chưa đo trong bản mẫu'},
  {name:'Tài khoản B',cli:'OpenCode',status:'Đã đăng nhập',model:'Theo cấu hình',quota:'Chưa đo trong bản mẫu'},
  {name:'Tài khoản C',cli:'Codex',status:'Đã đăng nhập',model:'Theo CLI',quota:'Chưa đo trong bản mẫu'},
  {name:'Tài khoản D',cli:'Claude',status:'Đang khoá',model:'Theo CLI',quota:'Chưa đo trong bản mẫu'}
];
const projects = [
  {name:'huba',desc:'Cầu kết nối với các phiên CLI',path:'Thư mục được chọn trên máy',sessions:'1 phiên',account:'Tự động'},
  {name:'Ví dụ dự án',desc:'Dự án minh họa',path:'Thư mục được chọn trên máy',sessions:'2 phiên',account:'Tự động'}
];
const activity = [
  ['09:42','Desktop','Đã chọn phiên','Thiết kế ứng dụng desktop','Đọc'],
  ['09:40','CLI','Lượt xử lý bắt đầu','Thiết kế ứng dụng desktop','Đang xử lý'],
  ['09:38','CLI','Có câu hỏi cần trả lời','Rà soát luồng đăng nhập','Chờ bạn'],
  ['09:32','Chủ máy','Đổi model mặc định','Tài khoản B','Cho phiên mới'],
  ['09:20','Telegram','Kiểm kết nối','Kênh Telegram minh họa','Chưa kiểm hai chiều'],
  ['09:15','Desktop','Mở ứng dụng','Trên máy này','Hoàn thành'],
  ['09:02','CLI','Lượt xử lý kết thúc','Cập nhật tài liệu hướng dẫn','Hoàn thành']
];
let page = 'chat', scenario = 'normal', current = sessions[0], detail = 'session', settingsTab = 'general';
let draft = '', drafts = {}, demoMessages = [], tableQuery = '', tablePage = 0, tableFilter = 'all', toastTimer, pendingSearch = null;
const pill = (text, color='') => `<span class="pill ${color}">${esc(text)}</span>`;
const button = (label, action, style='', extra='') => `<button class="${style}" data-action="${action}" ${extra}>${label}</button>`;
const heading = (category,title,description,actions='') => `<div class="page-top"><div><div class="eyebrow">${category}</div><h1>${title}</h1><p>${description}</p></div><div class="actions">${actions}</div></div>`;
const notice = () => scenario === 'offline' ? '<div class="notice error">Mất kết nối dịch vụ · Đang hiển thị dữ liệu cũ. Nội dung bạn soạn được giữ lại. <button class="link-button" data-action="reconnect">Thử kết nối lại</button></div>' : scenario === 'error' ? '<div class="notice error">Chưa đọc được nguồn CLI · Dữ liệu dưới đây là bản đã đọc trước. Chưa xác định trạng thái hiện tại. <button class="link-button" data-action="reconnect">Đọc lại</button></div>' : '';
const empty = (title,description,action='new-session',label='Mở phiên') => `<div class="empty"><div class="empty-icon">◫</div><h2>${title}</h2><p>${description}</p>${button(label,action,'primary')}</div>`;
function toast(text) { clearTimeout(toastTimer); $('#toast').textContent = text; $('#toast').classList.add('visible'); toastTimer = setTimeout(() => $('#toast').classList.remove('visible'), 4200); }
function navigate() {
  const target = location.hash.slice(1) || 'chat';
  page = navigation.some(n => n[0] === target) ? target : 'chat';
  tableQuery = pendingSearch ?? ''; pendingSearch = null; tablePage = 0; tableFilter = 'all'; render();
}
function render() {
  window.hubaReports?.dispose();
  $('#nav').innerHTML = navigation.map((n,i) => `${i===2?'<div class="nav-group nav-label">GÓC QUẢN LÝ</div>':''}<a href="#${n[0]}" title="${n[2]}" class="nav-item ${page===n[0]?'active':''}" ${page===n[0]?'aria-current="page"':''}><span class="nav-icon" aria-hidden="true">${n[1]}</span><span class="nav-label">${n[2]}</span>${n[3]?`<span class="nav-badge" aria-hidden="true">${n[3]}</span>`:''}</a>`).join('');
  $('#main').innerHTML = ({chat:chatPage,sessions:sessionsPage,reports:()=>window.hubaReports.page(scenario),accounts:accountsPage,connections:connectionsPage,projects:projectsPage,activity:activityPage,settings:settingsPage}[page])();
  if (page === 'reports') window.hubaReports.mount(scenario);
  if ($('#composer-text')) $('#composer-text').value = draft;
  if ($('#table-filter')) $('#table-filter').value = tableFilter;
  if (scenario === 'empty' && page === 'chat') {
    $('.chat-top').innerHTML = '<div><h1>Cuộc trò chuyện mới</h1><p>Chưa chọn phiên</p></div>';
    $('.details').innerHTML = '<h2>Thông tin phiên</h2><p>Chọn hoặc mở phiên để xem chi tiết.</p>';
    $('.composer-bottom').innerHTML = '<small>Chọn hoặc mở phiên để bắt đầu chat.</small><button type="submit" class="primary send" disabled>Gửi ↑</button>';
  }
}
function chooseSession(id) {
  drafts[current.id] = draft;
  current = sessions.find(s=>s.id===id) || current;
  draft = drafts[current.id] || '';
  if (page !== 'chat') location.hash = 'chat'; else render();
}
function chatPage() {
  const available = scenario !== 'offline' && scenario !== 'error' && scenario !== 'empty';
  return `<div class="chat-layout"><section class="session-list" aria-label="Chọn phiên"><div class="session-list-title"><h2>Cuộc trò chuyện</h2>${button('+','new-session','','aria-label="Mở phiên mới"')}</div><input id="session-search" placeholder="Tìm cuộc trò chuyện…" aria-label="Tìm cuộc trò chuyện"><div class="session-group">ĐANG MỞ · ${scenario==='empty'?0:3}</div><div id="session-items">${scenario==='empty'?'<p class="session-list-bottom">Các phiên của bạn sẽ xuất hiện ở đây.</p>':sessionCards(sessions)}</div><p class="session-list-bottom">Chọn phiên để tiếp tục công việc.<br>Mỗi cuộc trò chuyện gắn một phiên CLI.</p></section><section class="conversation" aria-label="Hội thoại"><div class="chat-top"><div><h1>${scenario==='empty'?'Cuộc trò chuyện mới':esc(current.name)}</h1><p>${esc(current.project)} / ${esc(current.lane)} · ${esc(current.account)} · ${esc(current.cli)}</p></div><div class="actions">${pill(scenario==='waiting'?'Chờ bạn':current.state,scenario==='waiting'?'amber':current.color)}${button('↗ Terminal','terminal')}${button('•••','session-menu','','aria-label="Thao tác với phiên"')}</div></div>${notice()}<div class="timeline" id="timeline">${scenario==='empty'?empty('Bắt đầu từ một phiên','Chọn dự án và tài khoản, rồi trò chuyện trực tiếp với phiên trên máy.'):timeline()}</div><div class="composer-area"><form id="composer" class="composer"><label class="sr-only" for="composer-text">Tin nhắn tới phiên đã chọn</label><textarea id="composer-text" placeholder="Tiếp tục công việc cùng phiên này…" ${available?'':'disabled'}></textarea><div class="composer-bottom">${button('＋','attach','ghost','type="button" aria-label="Đính kèm"')}<span class="tag">${esc(current.cli)}</span><span class="tag">${esc(current.account)}</span><button class="primary send" type="submit" ${available?'':'disabled'}>Gửi ↑</button></div></form><div class="composer-hint"><span>Ctrl / ⌘ + Enter để gửi · Enter xuống dòng</span><span>${available?'Bản mẫu · không gửi đến model':'Giữ bản nháp · chưa thể gửi'}</span></div></div></section><aside class="details" aria-label="Chi tiết hội thoại"><h2>Không gian phiên</h2><div class="detail-tabs">${[['session','Phiên'],['raw','Nội dung gốc'],['activity','Hoạt động']].map(([id,label])=>button(label,`detail-${id}`,detail===id?'selected':'')).join('')}</div>${detailPanel()}</aside></div>`;
}
function sessionCards(rows) { return rows.map(s => `<button class="session-card ${current.id===s.id?'active':''}" data-action="select-session" data-id="${s.id}"><span class="session-id">${s.cli}</span><strong>${esc(s.project)} / ${s.lane}</strong><p>${esc(s.name)}</p><div class="session-meta"><span>${pill(s.state,s.color)}</span><span>${s.time}</span></div></button>`).join(''); }
function timeline() {
  return `<div class="day">HÔM NAY · NỘI DUNG MINH HỌA</div><article class="message user-message"><span class="avatar">C</span><div class="message-body"><div class="message-label"><b>Bạn</b><span class="source">Desktop</span><time>09:40</time></div><p>Thiết kế giao diện và luồng xử lý trước. Tôi muốn dùng cùng một ứng dụng trên các nền tảng PC.</p></div></article><article class="message"><span class="avatar">h</span><div class="message-body"><div class="message-label"><b>${esc(current.cli)}</b><span class="source">Ví dụ hội thoại</span><time>09:41</time></div><p>Đề xuất một không gian làm việc chung: chat ở giữa, các phiên ở bên trái và thông tin cần đối chiếu ở bên phải.</p><p>Các màn quản lý dùng cùng thiết kế trên Windows, macOS và Linux. Bạn có thể tiếp tục một phiên ở desktop rồi theo dõi qua Telegram.</p><div class="tool-summary"><span class="check">✓</span><span>Đọc tài liệu thiết kế</span><small>Minh họa tool</small></div><p>Nội dung hiển thị luôn gắn với nguồn. Khi cần đối chiếu, bấm <b>Xem bản gốc</b> để mở đúng bản ghi của lượt đó.</p>${scenario==='waiting'||current.state==='Chờ bạn'?question():''}<div class="message-actions">${button('▣ Sao chép','copy')}${button('⌘ Xem bản gốc','raw')}${button('↗ Xem hoạt động','detail-activity')}</div></div></article>${demoMessages.filter(m=>m.session===current.id).map(m=>`<article class="message user-message"><span class="avatar">C</span><div class="message-body"><div class="message-label"><b>Bạn</b><span class="source">Chỉ trong bản mẫu</span></div><p>${esc(m.text).replace(/\n/g,'<br>')}</p><small>Minh họa tin đã soạn · chưa chuyển tới CLI.</small></div></article>`).join('')}`;
}
function question() { return `<div class="question"><b>Cần lựa chọn của bạn</b><p>Ví dụ hộp hỏi. Chọn một phương án rồi xác nhận.</p><label><input type="radio" name="answer" value="Giữ phiên hiện tại">Giữ phiên hiện tại và tiếp tục</label><label><input type="radio" name="answer" value="Bàn giao sang phiên mới">Bàn giao sang phiên mới</label>${button('Xác nhận lựa chọn','answer','primary')}</div>`; }
function detailPanel() {
  if (detail==='raw') return `<div class="detail-item"><small>Nguồn</small><b>Bản ghi CLI · ví dụ dữ liệu</b></div>${pill('Chưa kết nối nguồn thật','amber')}<pre>${esc(JSON.stringify({example:true,role:'user',content:'Thiết kế giao diện và luồng xử lý trước.',source:{kind:'cli_journal',cli:current.cli},completeness:'illustrative'},null,2))}</pre><div class="source-note">Bản ghi CLI và payload gọi model được phân biệt. Chỉ hiện payload khi nguồn thực sự cung cấp.</div>${button('Xem giải thích nguồn','source-info','detail-action')}`;
  if (detail==='activity') return `<div class="detail-item"><small>09:40 · Desktop</small><b>Bạn chọn phiên</b></div><div class="detail-item"><small>09:41 · CLI minh họa</small><b>Có nội dung hội thoại mới</b></div><div class="source-note">Khi triển khai, các mốc này lấy từ sự kiện đã lưu, có liên kết về nguồn.</div>${button('Mở nhật ký hoạt động','go-activity','detail-action')}`;
  return `<div class="detail-item"><small>Dự án / Làn</small><b>${esc(current.project)} / ${current.lane}</b></div><div class="detail-item"><small>CLI / Tài khoản</small><b>${current.cli} · ${esc(current.account)}</b></div><div class="detail-item"><small>Model</small><b>${esc(current.model)}</b></div><div class="detail-item"><small>Ngữ cảnh đã dùng</small><b>Chưa đo trong bản mẫu</b></div><div class="divider"></div><div class="detail-item"><small>Nguồn hội thoại</small><b>Nhật ký / sự kiện của CLI</b></div><div class="source-note">Bạn đang xem bản mẫu.<br>Trạng thái và hội thoại ở đây là ví dụ để duyệt thiết kế.</div><div class="divider"></div>${button('⇄ Bàn giao công việc','handover','detail-action')}${button('◻ Ngắt lượt hiện tại','interrupt','detail-action')}${button('Đóng phiên…','close-session','detail-action danger')}`;
}
function table(title,headers,rows,filters='') {
  const normalized = tableQuery.toLocaleLowerCase('vi');
  const filtered = rows.filter(r=>(tableFilter==='all'||r.filter===tableFilter)&&r.search.toLocaleLowerCase('vi').includes(normalized));
  const size=5, pages=Math.max(1,Math.ceil(filtered.length/size)); tablePage=Math.min(tablePage,pages-1);
  const visible=filtered.slice(tablePage*size,(tablePage+1)*size);
  return `<section class="card"><div class="card-top"><h2>${title}</h2><small>${filtered.length} mục</small></div><div class="toolbar"><input id="table-search" value="${esc(tableQuery)}" aria-label="Tìm trong toàn bộ danh sách" placeholder="Tìm trong toàn bộ danh sách…">${filters?`<select id="table-filter" aria-label="Lọc danh sách"><option value="all">Tất cả trạng thái</option>${filters}</select>`:''}</div><div class="table-wrap"><table><thead><tr><th>STT</th>${headers.map(h=>`<th>${h}</th>`).join('')}</tr></thead><tbody>${visible.map((r,i)=>`<tr><td>${tablePage*size+i+1}</td>${r.cells.map(c=>`<td>${c}</td>`).join('')}</tr>`).join('')||`<tr><td colspan="${headers.length+1}">Không có kết quả phù hợp.</td></tr>`}</tbody></table></div><div class="table-footer"><span>${filtered.length?tablePage*size+1:0}–${Math.min((tablePage+1)*size,filtered.length)} trên ${filtered.length} mục · Tìm trên toàn bộ dữ liệu</span><div class="actions">${button('‹','prev-page','',tablePage===0?'disabled':'')}<span>Trang ${tablePage+1} / ${pages}</span>${button('›','next-page','',tablePage>=pages-1?'disabled':'')}</div></div></section>`;
}
function sessionsPage() {
  return `<div class="page">${heading('KHÔNG GIAN LÀM VIỆC','Các phiên của bạn','Theo dõi công việc và tiếp tục đúng phiên trên máy.',button('+ Mở phiên','new-session','primary'))}${notice()}${scenario==='empty'?`<section class="card">${empty('Chưa có phiên đang mở','Mở một phiên mới hoặc kết nối lại với phiên đã có trên máy.')}</section>`:`<div class="summary-grid"><div class="card summary"><div><span>Phiên đang mở</span><strong style="display:block">3</strong></div><em>◫</em></div><div class="card summary"><div><span>Đang làm việc</span><strong style="display:block">1</strong></div><em>↗</em></div><div class="card summary"><div><span>Cần bạn trả lời</span><strong style="display:block">1</strong></div><em>◷</em></div></div>${table('Tất cả phiên',['Phiên / Dự án','CLI / Tài khoản','Trạng thái','Hoạt động gần nhất',''],sessions.map(s=>({search:Object.values(s).join(' '),filter:s.state,cells:[`<b>${esc(s.name)}</b><small>${esc(s.project)} / ${s.lane}</small>`,`${s.cli}<small>${esc(s.account)}</small>`,pill(s.state,s.color),s.time,button('Vào chat ↗','open-chat','','data-id="'+s.id+'"')]})),'<option value="Đang làm">Đang làm</option><option value="Chờ bạn">Chờ bạn</option><option value="Sẵn sàng">Sẵn sàng</option>')}`}</div>`;
}
function accountsPage() {
  return `<div class="page">${heading('QUẢN LÝ','Tài khoản & model','Tài khoản CLI dùng để mở phiên. Tách biệt với quyền chủ máy.',button('+ Thêm tài khoản','new-account','primary'))}${notice()}<div class="notice info">Model mặc định áp dụng cho phiên mới. Khoá tài khoản giữ nguyên lịch sử và không tự đóng phiên đang chạy.</div>${scenario==='empty'?`<section class="card">${empty('Thêm tài khoản đầu tiên','Chọn Claude, Codex hoặc OpenCode rồi đăng nhập bằng luồng của CLI.','new-account','Thêm tài khoản')}</section>`:table('Tài khoản trên máy',['Tên / CLI','Trạng thái','Model mặc định','Hạn mức',''],accounts.map((a,i)=>({search:Object.values(a).join(' '),filter:a.cli,cells:[`<b>${esc(a.name)}</b><small>${a.cli}</small>`,pill(a.status,a.status==='Đang khoá'?'gray':''),esc(a.model),`<small>${esc(a.quota)}</small>`,button('Quản lý','edit-account','','data-index="'+i+'"')]})),'<option value="Claude">Claude</option><option value="Codex">Codex</option><option value="OpenCode">OpenCode</option>')}</div>`;
}
function connectionsPage() {
  return `<div class="page">${heading('QUẢN LÝ','Kết nối','Làm việc trên máy. Tiếp tục cuộc trò chuyện ở nơi thuận tiện cho bạn.')}${notice()}<div class="grid-2"><section class="card channel-card"><div class="card-icon">▣</div><div class="card-top"><h2>Ứng dụng desktop</h2>${pill('Trên máy này')}</div><p>Mở phiên và chat trực tiếp. Đóng cửa sổ vẫn có thể giữ phiên và dịch vụ hoạt động.</p><div class="actions">${button('Xem dịch vụ','service-info')}</div></section><section class="card channel-card"><div class="card-icon blue">➤</div><div class="card-top"><h2>Telegram</h2>${pill(scenario==='empty'?'Chưa kết nối':'Chưa kiểm hai chiều','amber')}</div><p>Điều khiển cùng các phiên từ điện thoại. Chỉ chat đã xác minh được phép gửi lệnh.</p><div class="actions">${button('Thiết lập kết nối','telegram','primary')}${button('Xem luồng kiểm tra','telegram-check')}</div></section><section class="card channel-card"><div class="card-icon">⌘</div><div class="card-top"><h2>MCP</h2>${pill('Theo quyền được cấp','gray')}</div><p>Cung cấp nguồn hội thoại và thao tác cho client được cấp quyền. Độ đầy đủ của nội dung phụ thuộc nguồn CLI.</p><div class="actions">${button('Nguồn & quyền truy cập','mcp')}</div></section><section class="card channel-card"><div class="card-icon">⌁</div><div class="card-top"><h2>Kênh khác</h2>${pill('Chưa có adapter','gray')}</div><p>Giữ cùng các luồng chat và quản lý khi bổ sung kênh. Chỉ cho phép kết nối sau khi adapter được triển khai và kiểm chứng.</p><div class="actions">${button('Xem cách mở rộng','other-channels')}</div></section></div></div>`;
}
function projectsPage() {
  return `<div class="page">${heading('QUẢN LÝ','Dự án','Mỗi phiên gắn đúng thư mục làm việc và quy tắc của dự án.',button('+ Thêm dự án','new-project','primary'))}${notice()}${scenario==='empty'?`<section class="card">${empty('Chọn thư mục làm việc','Ứng dụng dùng hộp chọn thư mục của hệ điều hành.','new-project','Thêm dự án')}</section>`:table('Workspace & dự án',['Dự án','Thư mục','Phiên','Tài khoản mặc định',''],projects.map(p=>({search:Object.values(p).join(' '),cells:[`<b>${esc(p.name)}</b><small>${p.desc}</small>`,esc(p.path),p.sessions,p.account,button('Cấu hình','edit-project','','data-name="'+esc(p.name)+'"')]})))}</div>`;
}
function activityPage() {
  return `<div class="page">${heading('QUẢN LÝ','Hoạt động','Đối chiếu thao tác, nguồn gửi và kết quả thực hiện.',button('Xuất chẩn đoán','export-log'))}${notice()}${table('Nhật ký minh họa',['Thời điểm','Nguồn','Thao tác','Đối tượng','Kết quả'],scenario==='empty'?[]:activity.map(a=>({search:a.join(' '),cells:[a[0],pill(a[1],'gray'),a[2],a[3],a[4]]})))}</div>`;
}
function settingsPage() {
  const menu=[['general','Giao diện & thao tác'],['notifications','Thông báo'],['storage','Dữ liệu & nguồn gốc'],['security','Quyền truy cập'],['app','Ứng dụng & cập nhật']];
  const panels={
    general:`<h2>Không gian làm việc</h2><p>Thiết lập dùng chung trên các nền tảng PC.</p><div class="field"><label for="theme">Giao diện</label><select id="theme"><option>Theo hệ điều hành</option><option>Sáng</option><option>Tối</option></select></div><div class="field"><label for="send-key">Phím gửi tin nhắn</label><select id="send-key"><option>Ctrl / ⌘ + Enter</option><option>Enter</option></select><small>Phím còn lại dùng để xuống dòng.</small></div><div class="list-row"><div><b>Giữ dịch vụ khi đóng cửa sổ</b><small>Tiếp tục theo dõi các phiên qua Telegram.</small></div><input class="switch" type="checkbox" checked aria-label="Giữ dịch vụ khi đóng cửa sổ"></div>`,
    notifications:`<h2>Thông báo khi có thay đổi</h2><p>Chọn nơi nhận và các sự kiện cần biết.</p>${['Phiên cần bạn trả lời','Lượt xử lý kết thúc','Lỗi cần xử lý','Mất kết nối kênh'].map((t,i)=>`<div class="list-row"><b>${t}</b><input class="switch" type="checkbox" ${i!==1?'checked':''} aria-label="${t}"></div>`).join('')}<div class="field"><label for="notify-target">Nơi nhận</label><select id="notify-target"><option>Desktop</option><option>Desktop và Telegram đã kết nối</option></select></div>`,
    storage:`<h2>Dữ liệu & nguồn gốc</h2><p>Phân biệt nội dung hiển thị với bản ghi nguồn.</p><div class="field"><label for="retention">Lưu hội thoại gốc trong</label><input id="retention" type="number" min="1" value="30"><small>Số ngày minh họa, cần cấu hình khi dùng thật. Dọn dữ liệu có xác nhận.</small></div><div class="list-row"><div><b>Thu bản ghi hội thoại CLI</b><small>Theo nguồn và quyền của từng tài khoản.</small></div><input class="switch" type="checkbox" checked aria-label="Thu bản ghi CLI"></div><div class="notice">Payload model là nguồn riêng. Không nhận đã thu đủ nếu CLI không cung cấp.</div>`,
    security:`<h2>Chủ máy & client được cấp quyền</h2><p>Phân quyền ở dịch vụ dùng chung cho mọi cửa vào.</p><div class="list-row"><div><b>Chủ máy</b><small>Được quản lý cấu hình trên máy này.</small></div>${pill('Quản trị')}</div><div class="list-row"><div><b>Client MCP</b><small>Chưa có client minh họa được cấp quyền.</small></div>${button('Xem quyền','mcp')}</div><div class="notice info">Tài khoản model không cấp quyền quản trị ứng dụng. Bot token không hiện lại sau lưu.</div>`,
    app:`<h2>Ứng dụng & cập nhật</h2><p>Một thiết kế chung cho Windows, macOS và Linux.</p><div class="list-row"><div><b>Trạng thái bản này</b><small>Prototype thiết kế · chưa phải ứng dụng đã đóng gói.</small></div>${pill('Bản mẫu','amber')}</div><div class="list-row"><div><b>Cài đặt & cập nhật</b><small>Gói theo OS, dữ liệu và phiên được giữ qua cập nhật.</small></div>${button('Xem luồng cài đặt','install-flow')}</div><div class="list-row"><div><b>Đóng ứng dụng</b><small>Phân biệt đóng cửa sổ với tắt dịch vụ.</small></div>${button('Xem lựa chọn','exit-app')}</div>`
  };
  return `<div class="page">${heading('QUẢN LÝ','Cài đặt','Điều chỉnh ứng dụng theo cách bạn làm việc.')}${notice()}<div class="settings-layout"><aside class="settings-menu">${menu.map(([id,t])=>button(t,`settings-${id}`,settingsTab===id?'selected':'')).join('')}</aside><section class="card">${panels[settingsTab]}<div class="dialog-footer">${button('Khôi phục biểu mẫu','reset-settings')}${button('Lưu thay đổi','save-settings','primary')}</div></section></div></div>`;
}
function field(label,id,value='',type='text',hint='') {return `<div class="field"><label for="${id}">${label}</label><input id="${id}" name="${id}" type="${type}" value="${esc(value)}">${hint?`<small>${hint}</small>`:''}</div>`;}
function showModal(title,description,body,footer=button('Đã hiểu','dismiss','primary')) {
  $('#modal-content').innerHTML=`<div class="dialog-title"><h2>${title}</h2>${button('×','dismiss','ghost','type="button" aria-label="Đóng hộp thoại"')}</div><p class="dialog-description">${description}</p>${body}<div class="dialog-footer">${footer}</div>`;
  if (!$('#modal').open) $('#modal').showModal();
}
function accountModal(index) {
  const a=index===undefined?{name:'',cli:'Claude',model:'Theo CLI'}:accounts[index];
  showModal(index===undefined?'Thêm tài khoản':'Quản lý tài khoản','Các thay đổi trong bản mẫu chỉ nằm trong bộ nhớ.',`${field('Tên tài khoản','account-name',a.name)}<div class="field"><label for="account-cli">Loại CLI</label><select id="account-cli">${['Claude','Codex','OpenCode'].map(t=>`<option ${t===a.cli?'selected':''}>${t}</option>`).join('')}</select></div>${field('Model mặc định','account-model',a.model)}<div class="notice info">Đăng nhập qua luồng của CLI. Không nhập API key hay token thật vào bản mẫu.</div>${index!==undefined?`<div class="actions">${button('Đăng nhập','account-login')}${button(a.status==='Đang khoá'?'Mở khoá':'Khoá tài khoản','lock-account','','data-index="'+index+'"')}</div>`:''}`,`${button('Huỷ','dismiss')}${button('Lưu bản mẫu','save-account','primary',index===undefined?'':'data-index="'+index+'"')}`);
}
function telegramStep(step) {
  const text={
    1:`<div class="dialog-step">BƯỚC 1 / 3 · BOT CỦA BẠN</div><div class="notice">Không nhập token thật. Đây chỉ là minh họa các bước kết nối.</div>${field('Token bot Telegram','demo-token','','password','Bản thật lưu bí mật trên máy, không đưa vào cấu hình chia sẻ.')}<p>Nhập token từ BotFather; bước kiểm tra sẽ hiển thị danh tính bot.</p>`,
    2:`<div class="dialog-step">BƯỚC 2 / 3 · XÁC MINH CHỦ CHAT</div><div class="notice info">Ví dụ: danh tính bot đã được kiểm. Chưa xác minh chat nhận lệnh.</div><p>Nhắn một tin cho bot từ điện thoại. Ứng dụng sẽ nhận tin qua cùng dịch vụ đang chạy rồi hiển thị chat để bạn xác nhận.</p>${field('Chat được tìm thấy (minh họa)','demo-chat','Chat riêng của chủ máy')}<label><input type="checkbox" id="verify-chat"> Tôi xác nhận đây là chat được phép điều khiển máy.</label>`,
    3:`<div class="dialog-step">BƯỚC 3 / 3 · PHẠM VI KẾT NỐI</div><div class="list-row"><b>Khi phiên cần trả lời</b><input class="switch" type="checkbox" checked aria-label="Thông báo khi cần trả lời"></div><div class="list-row"><b>Khi lượt xử lý kết thúc</b><input class="switch" type="checkbox" aria-label="Thông báo khi lượt kết thúc"></div><div class="notice info">Sau khi lưu ở bản thật: kiểm tin đi → nhận tin trả lời. Chỉ lúc ấy mới ghi kết nối hai chiều hoạt động.</div>`
  };
  showModal('Kết nối Telegram','Cùng các phiên trên máy, thêm một cửa giao tiếp từ điện thoại.',text[step],`${button(step===1?'Huỷ':'Quay lại',step===1?'dismiss':`telegram-step-${step-1}`)}${button(step===3?'Lưu bản mẫu':'Tiếp tục',step===3?'telegram-save':`telegram-step-${step+1}`,'primary')}`);
}
document.addEventListener('click', e => {
  const b=e.target.closest('[data-action]'); if (!b) return; e.preventDefault();
  const a=b.dataset.action;
  if(a==='dismiss') {$('#modal').close();return;}
  if(a.startsWith('detail-')) {detail=a.slice(7);render();return;}
  if(a.startsWith('settings-')) {settingsTab=a.slice(9);render();return;}
  if(a.startsWith('telegram-step-')) {if(a==='telegram-step-3'&&!$('#verify-chat')?.checked){toast('Hãy xác nhận đúng chat trước khi tiếp tục.');return;}telegramStep(Number(a.slice(14)));return;}
  switch(a){
    case 'select-session': case 'open-chat': chooseSession(b.dataset.id); break;
    case 'raw': detail='raw';render();if(window.innerWidth<=1240)showModal('Nội dung gốc','Minh họa cách đối chiếu nguồn.',detailPanel());break;
    case 'go-activity': location.hash='activity'; break;
    case 'search': showModal('Tìm nhanh','Tìm theo tên phiên hoặc dự án.',field('Từ khoá','quick-query'),button('Tìm phiên','quick-find','primary'));break;
    case 'quick-find': pendingSearch=$('#quick-query').value;$('#modal').close();if(page==='sessions'){tableQuery=pendingSearch;pendingSearch=null;tablePage=0;render();}else location.hash='sessions';break;
    case 'notifications': showModal('Thông báo','Ví dụ những thay đổi cần bạn quan tâm.','<div class="list-row"><div><b>Rà soát luồng đăng nhập</b><small>Có câu hỏi chờ bạn lựa chọn.</small></div></div>',button('Mở phiên','notification-open','primary'));break;
    case 'notification-open': $('#modal').close();chooseSession(sessions[1].id);break;
    case 'owner': showModal('Chủ máy','Quyền quản lý ứng dụng trên máy này.','<div class="notice info">Đây là danh tính điều khiển ứng dụng. Tài khoản Claude, Codex và OpenCode được quản lý riêng.</div>',`${button('Cài đặt quyền','owner-settings','primary')}${button('Đóng','dismiss')}`);break;
    case 'owner-settings': settingsTab='security';$('#modal').close();location.hash='settings';render();break;
    case 'new-session': showModal('Mở phiên mới','Chọn nơi làm việc trước khi bắt đầu.',`<div class="field"><label for="new-project">Dự án</label><select id="new-project"><option>huba</option><option>Ví dụ dự án</option></select></div><div class="field"><label for="new-acc">Tài khoản</label><select id="new-acc"><option>Tự động chọn theo khả năng và hạn mức</option>${accounts.filter(a=>a.status!=='Đang khoá').map(a=>`<option>${esc(a.name)} · ${a.cli}</option>`).join('')}</select></div><div class="field"><label for="new-task">Việc đầu tiên (tuỳ chọn)</label><textarea id="new-task" placeholder="Bạn muốn làm gì?"></textarea></div><small>Bản thật kiểm quyền và luật → mở terminal → nhận diện phiên → sẵn sàng chat.</small>`,`${button('Huỷ','dismiss')}${button('Xem bước tiếp theo','new-session-next','primary')}`);break;
    case 'new-session-next': showModal('Phiên sẵn sàng khi nào?','Luồng mở phiên dự kiến.','<ol><li>Kiểm dự án, tài khoản, quyền và luật.</li><li>Mở terminal thật bằng adapter hệ điều hành.</li><li>Nhận diện đúng ID phiên mới.</li><li>Kiểm nguồn hội thoại và khả năng gửi.</li><li>Chuyển bạn vào màn chat.</li></ol><div class="notice">Bản mẫu chưa mở terminal hay khởi chạy CLI.</div>');break;
    case 'new-account':accountModal();break;
    case 'edit-account':accountModal(Number(b.dataset.index));break;
    case 'save-account': {const name=$('#account-name').value.trim();if(!name){toast('Cần tên tài khoản.');return;}const row={name,cli:$('#account-cli').value,model:$('#account-model').value||'Theo CLI'};if(b.dataset.index!==undefined)accounts[Number(b.dataset.index)]={...accounts[Number(b.dataset.index)],...row};else accounts.push({...row,status:'Chưa đăng nhập',quota:'Chưa đo trong bản mẫu'});$('#modal').close();render();toast('Đã thay đổi dữ liệu minh họa; chưa ghi cấu hình thật.');break;}
    case 'lock-account':{const row=accounts[Number(b.dataset.index)];row.status=row.status==='Đang khoá'?'Chưa đăng nhập':'Đang khoá';$('#modal').close();render();toast('Đã đổi trạng thái tài khoản minh họa.');break;}
    case 'account-login': showModal('Đăng nhập CLI','Bản thật mở đúng luồng của loại tài khoản.','<ol><li>Mở cửa sổ đăng nhập gắn với tài khoản và lần thử.</li><li>Bạn hoàn tất đăng nhập trên trang của nhà cung cấp.</li><li>Ứng dụng kiểm lại kết quả từ CLI.</li></ol><div class="notice">Nếu mã hết hạn, bắt đầu lần đăng nhập mới. Không nhận cửa sổ đã mở là đăng nhập xong.</div>');break;
    case 'telegram':telegramStep(1);break;
    case 'telegram-save':$('#modal').close();toast('Đã đi hết wizard minh họa. Chưa lưu token, gọi Telegram hay khởi động lại dịch vụ.');break;
    case 'telegram-check':showModal('Kiểm Telegram hai chiều','Token hợp lệ chưa đủ để xác nhận kênh hoạt động.','<ol><li>Kiểm danh tính bot.</li><li>Nhận tin từ đúng chat qua daemon.</li><li>Chủ máy bấm gửi tin kiểm tra.</li><li>Nhận phản hồi và đối chiếu đúng đích.</li></ol><div class="notice">Bản mẫu không gửi tin thật.</div>');break;
    case 'mcp':showModal('MCP · Nguồn & quyền','MCP truy xuất dữ liệu qua dịch vụ huba dùng chung.','<div class="list-row"><div><b>Đọc hội thoại</b><small>Đọc sự kiện có cursor và nguồn đối chiếu.</small></div></div><div class="list-row"><div><b>Đọc bản ghi gốc</b><small>Quyền riêng; phân biệt nhật ký CLI và payload model.</small></div></div><div class="list-row"><div><b>Điều khiển phiên</b><small>Chỉ client có quyền; cùng hàng lệnh và kiểm nhận.</small></div></div><div class="notice">MCP không tự thu mọi input/output. Nội dung có được phải do nguồn CLI/SDK cung cấp.</div>');break;
    case 'source-info':showModal('Độ đầy đủ của nguồn','Ba nguồn khác nhau được ghi nhãn riêng.','<ul><li><b>Nhật ký CLI:</b> bản ghi mà CLI cung cấp.</li><li><b>Payload model:</b> request/response nếu điểm thu được hỗ trợ.</li><li><b>Màn terminal:</b> chữ nhìn thấy, có thể thiếu nội dung cuộn khỏi màn.</li></ul>');break;
    case 'other-channels':showModal('Mở rộng kênh giao tiếp','Mỗi kênh mới dùng cùng nghiệp vụ hiện có.','<p>Thêm adapter xác thực/nhận/gửi cho kênh, rồi kiểm chéo với desktop. Hiện huba có Telegram; chưa có adapter Slack hay Discord đang hoạt động.</p>');break;
    case 'service-info':showModal('Dịch vụ trên máy','Giao diện là một cửa vào dịch vụ dùng chung.','<p>Desktop và Telegram cùng quản lý một tập phiên. Không khởi động thêm bộ nhận Telegram khi mở ứng dụng. Khi dịch vụ mất kết nối, giữ bản nháp và hiển thị dữ liệu cũ.</p>');break;
    case 'new-project':case 'edit-project':showModal(a==='new-project'?'Thêm dự án':'Cấu hình dự án','Bản thật dùng hộp chọn thư mục của hệ điều hành.',`${field('Tên dự án','project-name',b.dataset.name||'')}${field('Thư mục làm việc','project-folder','','text','Đường dẫn từ hộp chọn native; không ghim đường dẫn máy dựng.')}<div class="field"><label for="project-default">Tài khoản mặc định</label><select id="project-default"><option>Tự động</option>${accounts.map(a=>`<option>${esc(a.name)}</option>`).join('')}</select></div><small>Kiểm thư mục thực, quyền và luật trước khi mở phiên.</small>`,`${button('Huỷ','dismiss')}${button('Xem kết quả minh họa','save-project','primary')}`);break;
    case 'save-project':$('#modal').close();toast('Đã xem luồng thêm dự án; chưa đọc hay ghi thư mục thật.');break;
    case 'terminal':showModal('Mở terminal của phiên','Bản thật đưa bạn tới đúng terminal đang chứa phiên.','<p>Giữ nguyên phiên, kích thước và nội dung đang chạy. Nếu không xác định được đúng terminal, hiển thị lỗi thay vì mở một phiên khác.</p>');break;
    case 'session-menu':showModal('Thao tác với phiên',esc(current.name),`<div class="actions">${button('Ngắt lượt','interrupt')}${button('Bàn giao','handover')}${button('Đóng phiên','close-session','danger')}</div>`);break;
    case 'interrupt':case 'close-session':case 'handover': {const content={interrupt:['Ngắt lượt hiện tại?','Dừng công việc đang xử lý; giữ phiên để tiếp tục.'], 'close-session':['Đóng phiên này?','Kết thúc đúng phiên đã chọn. Việc đang chạy có thể bị dừng.'],handover:['Bàn giao công việc','Xem bản bàn giao và phiên nhận trước khi chuyển. Chỉ cho phép khi CLI hỗ trợ và đã được kiểm.']}[a];showModal(content[0],esc(current.name),`<p>${content[1]}</p><div class="notice">Bản mẫu không tác động tới phiên thật.</div>`,`${button('Huỷ','dismiss')}${button('Xác nhận trong bản mẫu','confirm-demo',a==='close-session'?'danger':'primary')}`);break;}
    case 'confirm-demo':$('#modal').close();toast('Đã xác nhận luồng minh họa; không thực thi thao tác thật.');break;
    case 'answer':{const answer=$('input[name="answer"]:checked');if(!answer){toast('Chọn phương án trước khi xác nhận.');return;}showModal('Đối chiếu câu hỏi trước khi gửi','Ví dụ lựa chọn: '+esc(answer.value),'<p>Bản thật kiểm ID/revision câu hỏi, gửi đúng phiên, đọc lại trạng thái để xác nhận đã nhận lựa chọn.</p>');break;}
    case 'attach':showModal('Đính kèm vào hội thoại','Chỉ hiện thao tác nếu CLI hỗ trợ.','<p>Bản thật dùng hộp chọn tệp. Hiển thị tên, loại và kích thước; phân biệt đã chọn, đã chuyển tới CLI và nguồn đã ghi nhận. Không coi một đường dẫn là đã thu được nội dung tệp.</p>');break;
    case 'copy':toast('Bản mẫu: thao tác sao chép sẽ lấy nguyên nội dung hiển thị của tin được chọn.');break;
    case 'export-log':showModal('Xuất chẩn đoán','Chủ máy xem nội dung trước khi xuất.','<p>Phiên bản, lỗi và trạng thái lệnh có nguồn. Loại bot token và bí mật xác thực huba. Xuất hội thoại/raw là thao tác riêng có quyền và phạm vi rõ.</p>');break;
    case 'save-settings':toast('Đã xem luồng lưu: kiểm hợp lệ → kiểm revision → ghi và audit → áp dụng. Bản mẫu chưa lưu thật.');break;
    case 'reset-settings':render();toast('Đã đặt lại biểu mẫu minh họa.');break;
    case 'reconnect':showModal('Phục hồi kết nối','Luồng phục hồi không tự gửi lại lệnh chưa rõ kết quả.','<ol><li>Nối lại dịch vụ và kiểm version.</li><li>Đối chiếu lệnh đang treo bằng ID.</li><li>Đọc tiếp sự kiện từ cursor.</li><li>Kiểm lại phiên đích trước thao tác.</li></ol>',button('Xem trạng thái bình thường','normal-state','primary'));break;
    case 'normal-state':scenario='normal';$('#scenario').value=scenario;$('#modal').close();render();break;
    case 'install-flow':showModal('Cài đặt & nâng cấp','Cùng một ứng dụng; gói cài theo hệ điều hành.','<ol><li>Cài và mở từ launcher.</li><li>Nhận diện cấu hình/dịch vụ có sẵn.</li><li>Kiểm phiên bản và quyền OS.</li><li>Chọn workspace, đăng nhập CLI, bắt đầu chat.</li></ol><p>Cập nhật giữ dữ liệu; backup/migration có rollback. Mỗi gói phải được kiểm trên OS thật.</p>');break;
    case 'exit-app':showModal('Đóng ứng dụng','Các phiên có thể tiếp tục chạy khi đóng cửa sổ.','<p>Chỉ đóng cửa sổ để tiếp tục qua Telegram. Tắt dịch vụ ngừng kết nối huba; không tự coi là đã đóng các phiên CLI.</p>',`${button('Huỷ','dismiss')}${button('Đóng cửa sổ — minh họa','confirm-demo','primary')}`);break;
    case 'prev-page':tablePage=Math.max(0,tablePage-1);render();break;
    case 'next-page':tablePage++;render();break;
  }
});
document.addEventListener('input',e=>{
  if(e.target.id==='composer-text') {draft=e.target.value;drafts[current.id]=draft;}
  if(e.target.id==='session-search') $('#session-items').innerHTML=sessionCards(sessions.filter(s=>Object.values(s).join(' ').toLocaleLowerCase('vi').includes(e.target.value.toLocaleLowerCase('vi'))));
  if(e.target.id==='table-search'){const at=e.target.selectionStart;tableQuery=e.target.value;tablePage=0;render();const input=$('#table-search');input.focus();input.setSelectionRange(at,at);}
});
document.addEventListener('change',e=>{if(e.target.id==='table-filter'){tableFilter=e.target.value;tablePage=0;render();$('#table-filter').value=tableFilter;}});
document.addEventListener('submit',e=>{
  e.preventDefault();if(e.target.id!=='composer')return;
  const text=$('#composer-text').value.trim();if(!text)return;
  demoMessages.push({session:current.id,text});draft='';drafts[current.id]='';render();$('#timeline').scrollTop=$('#timeline').scrollHeight;$('#composer-text').focus();toast('Tin chỉ được thêm vào bản mẫu, chưa gửi tới CLI hay model.');
});
document.addEventListener('keydown',e=>{if((e.ctrlKey||e.metaKey)&&e.key==='Enter'&&e.target.id==='composer-text'){e.preventDefault();$('#composer').requestSubmit();}if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'){e.preventDefault();document.querySelector('[data-action="search"]').click();}});
$('#collapse').addEventListener('click',()=>{const collapsed=document.body.classList.toggle('collapsed');$('#collapse').setAttribute('aria-expanded',String(!collapsed));$('#collapse').setAttribute('aria-label',collapsed?'Mở rộng menu':'Thu gọn menu');});
$('#scenario').addEventListener('change',e=>{scenario=e.target.value;render();});
$('#reset-demo').addEventListener('click',()=>location.reload());
window.addEventListener('hashchange',navigate);
window.addEventListener('huba-report-change',render);
navigate();
