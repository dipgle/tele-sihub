//! Lõi bảo mật của giao diện web cục bộ (`huba ui`) — HÀM THUẦN: nhận `Req`, trả `Resp`, không đụng ổ cắm.
//!
//! W1 của `docs/ui/ke-hoach-cong-viec.md`. Cổng 127.0.0.1 KHÔNG phải hàng rào: mọi tiến trình cùng máy gọi được, và một
//! trang web lạ trong trình duyệt của chủ máy cũng bắn được yêu cầu vào đó (DNS-rebinding, CSRF). Giao diện này sẽ GÕ được
//! vào phiên CLI, nên hở là hở lệnh. Thứ tự kiểm — mỗi tầng chặn trước tầng sau, và KHÔNG tầng nào đốt vé:
//!
//! 1. `Host` phải là `127.0.0.1:<cổng>` hoặc `localhost:<cổng>` (chống DNS-rebinding: trang `evil.com` trỏ DNS về
//!    127.0.0.1 vẫn gửi `Host: evil.com`).
//! 2. Không phải GET ⟹ `Origin` phải là chính nó (chống CSRF; `SameSite=Strict` là lớp thứ hai, không thay thế).
//! 3. Thân > [`MAX_BODY`] ⟹ 413.
//! 4. `GET /?t=<vé>`: vé MỘT LẦN, đúng ⟹ cookie phiên `HttpOnly; SameSite=Strict`, rồi 303 về `/` (vé rời thanh địa chỉ).
//! 5. Mọi thứ còn lại cần cookie phiên đúng (so sánh thời gian không đổi) ⟹ không thì 401.
//! 6. Định tuyến: `/api/*` và tệp tĩnh từ `ui_dir`. Tệp tĩnh KHÔNG thoát khỏi `ui_dir` (phân giải thật + so tiền tố, nên cả
//!    liên kết mềm dẫn ra ngoài cũng bị chặn).
//!
//! Token: 128 bit từ hệ điều hành (`getrandom`); không lấy được ⟹ KHÔNG dựng (không bao giờ rơi về khoá yếu).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

pub use crate::localserver::MAX_BODY;
use crate::localserver::{parse_form, percent_decode, Req};

/// Tên cookie phiên.
pub const COOKIE: &str = "huba_ui";

#[derive(Debug, Clone)]
pub struct Resp {
    pub status: u16,
    pub ctype: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Bộ nhớ đệm danh sách phiên. `snapshot` tốn ~4,7 giây (đo 23/11: 11 phiên) nên KHÔNG tính theo từng yêu cầu — một thread
/// riêng làm mới, giao diện đọc bản đệm. Lỗi làm mới KHÔNG xoá bản cũ: giao diện phải thấy «dữ liệu cũ + lỗi», không thấy «0 phiên».
#[derive(Default)]
struct SessionsCache {
    data: Option<serde_json::Value>,
    ok_at_ms: Option<i64>,
    tried_at_ms: Option<i64>,
    error: Option<String>,
}

/// Nhà cung cấp nội dung một phiên: nhận hàng phiên (JSON trong bản đệm) + giới hạn, trả JSON hoặc lời lỗi.
pub type StreamFn =
    Box<dyn Fn(&serde_json::Value, usize) -> Result<serde_json::Value, String> + Send + Sync>;

/// Trần/mặc định số sự kiện của `/api/sessions/<id>/stream` (kẹp phía máy chủ, không tin tham số).
const STREAM_DEFAULT: usize = 80;
const STREAM_MAX: usize = 300;

/// Nhà cung cấp việc GỬI tin vào phiên (K4): nhận hàng phiên (JSON trong bản đệm) + chữ, trả kết cục. Mã chạy thật gắn
/// `ui_send::send`; test gắn bản giả — nhờ vậy hàng rào của đường POST kiểm được mà không đụng Terminal nào.
pub type SendFn = Box<dyn Fn(&serde_json::Value, &str) -> crate::ui_send::SendResult + Send + Sync>;

/// Số kết quả gửi gần nhất được nhớ theo `client_id` (để bấm lặp/mạng retry không gõ hai lần). Trần cố định: bộ nhớ không phình.
const SEND_REMEMBER: usize = 200;

/// Sổ chống gõ hai lần: `client_id` đã có kết quả (nhớ SEND_REMEMBER cái gần nhất), `client_id` và phiên đang được gửi.
#[derive(Default)]
struct SendBook {
    done: std::collections::VecDeque<(String, serde_json::Value)>,
    in_flight: std::collections::HashSet<String>,
    busy_sessions: std::collections::HashSet<String>,
}

pub struct UiState {
    pub port: u16,
    pub ui_dir: PathBuf,
    session: String,
    ticket: String,
    ticket_used: AtomicBool,
    sessions: RwLock<SessionsCache>,
    stream_fn: RwLock<Option<StreamFn>>,
    send_fn: RwLock<Option<SendFn>>,
    sends: std::sync::Mutex<SendBook>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 16 byte ngẫu nhiên của hệ điều hành ⟹ 32 chữ số hex.
fn random_hex() -> Result<String, String> {
    let mut b = [0u8; 16];
    getrandom::getrandom(&mut b)
        .map_err(|e| format!("không lấy được số ngẫu nhiên từ hệ điều hành: {e}"))?;
    Ok(b.iter().map(|x| format!("{x:02x}")).collect())
}

impl UiState {
    /// Dựng trạng thái. Lỗi nếu hệ điều hành không cấp được số ngẫu nhiên — KHÔNG rơi về khoá yếu.
    pub fn try_new(port: u16, ui_dir: PathBuf) -> Result<Self, String> {
        let session = random_hex()?;
        let mut ticket = random_hex()?;
        // Xác suất 2^-128, nhưng «vé trùng khoá phiên» là lỗ hổng thật nên không để nó xảy ra.
        while ticket == session {
            ticket = random_hex()?;
        }
        Ok(UiState {
            port,
            ui_dir,
            session,
            ticket,
            ticket_used: AtomicBool::new(false),
            sessions: RwLock::new(SessionsCache::default()),
            stream_fn: RwLock::new(None),
            send_fn: RwLock::new(None),
            sends: std::sync::Mutex::new(SendBook::default()),
        })
    }

    /// Gắn nhà cung cấp việc gửi tin (mã chạy thật gắn `ui_send::send`; test gắn bản giả).
    pub fn set_send_provider(&self, f: SendFn) {
        *self.send_fn.write().unwrap_or_else(|e| e.into_inner()) = Some(f);
    }

    /// Ghi kết quả một lần làm mới danh sách phiên. `Ok` thay bản đệm; `Err` giữ bản cũ và ghi lỗi.
    pub fn put_sessions(&self, r: Result<serde_json::Value, String>, now: i64) {
        let mut c = self.sessions.write().unwrap_or_else(|e| e.into_inner());
        c.tried_at_ms = Some(now);
        match r {
            Ok(v) => {
                c.data = Some(v);
                c.ok_at_ms = Some(now);
                c.error = None;
            }
            Err(e) => c.error = Some(e),
        }
    }

    /// Gắn nhà cung cấp nội dung phiên (mã chạy thật gắn `sessions::stream`; test gắn bản giả).
    pub fn set_stream_provider(&self, f: StreamFn) {
        *self.stream_fn.write().unwrap_or_else(|e| e.into_inner()) = Some(f);
    }

    /// Hàng phiên (JSON) theo id trong bản đệm; `None` nếu id không có trong danh sách đang đệm.
    fn session_row(&self, id: &str) -> Option<serde_json::Value> {
        let c = self.sessions.read().unwrap_or_else(|e| e.into_inner());
        c.data
            .as_ref()?
            .get("sessions")?
            .as_array()?
            .iter()
            .find(|s| s.get("session_id").and_then(|v| v.as_str()) == Some(id))
            .cloned()
    }

    fn sessions_json(&self) -> serde_json::Value {
        let c = self.sessions.read().unwrap_or_else(|e| e.into_inner());
        serde_json::json!({
            "loading": c.tried_at_ms.is_none(),
            "error": c.error,
            "ok_at_ms": c.ok_at_ms,
            "now_ms": now_ms(),
            "data": c.data,
        })
    }

    /// Như [`try_new`] nhưng dừng chương trình khi không có số ngẫu nhiên (dùng trong test).
    pub fn new(port: u16, ui_dir: PathBuf) -> Self {
        Self::try_new(port, ui_dir)
            .expect("uiserver: không có số ngẫu nhiên ⟹ không dựng giao diện")
    }

    /// Vé một lần để ghép vào URL đầu tiên.
    pub fn ticket(&self) -> &str {
        &self.ticket
    }

    /// Giá trị cookie phiên (test dùng; mã chạy thật chỉ đưa nó qua `Set-Cookie`).
    pub fn session(&self) -> &str {
        &self.session
    }

    fn hosts(&self) -> [String; 2] {
        [
            format!("127.0.0.1:{}", self.port),
            format!("localhost:{}", self.port),
        ]
    }

    fn origins(&self) -> [String; 2] {
        [
            format!("http://127.0.0.1:{}", self.port),
            format!("http://localhost:{}", self.port),
        ]
    }
}

/// So sánh không rò độ dài-khớp qua thời gian (độ dài khác ⟹ sai ngay; chỉ độ dài là lộ, mà nó cố định 32).
fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

const CSP: &str = "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; \
                   frame-ancestors 'none'; base-uri 'none'; form-action 'self'";

fn mk(status: u16, ctype: &str, body: impl Into<Vec<u8>>) -> Resp {
    Resp {
        status,
        ctype: ctype.to_string(),
        headers: vec![],
        body: body.into(),
    }
}

fn text(status: u16, msg: &str) -> Resp {
    mk(status, "text/plain; charset=utf-8", msg.as_bytes().to_vec())
}

/// Gắn tiêu đề an toàn lên MỌI phản hồi (kể cả lỗi).
fn finish(mut r: Resp) -> Resp {
    for (k, v) in [
        ("X-Content-Type-Options", "nosniff"),
        ("Cache-Control", "no-store"),
        ("Referrer-Policy", "no-referrer"),
        ("X-Frame-Options", "DENY"),
        ("Content-Security-Policy", CSP),
    ] {
        r.headers.push((k.to_string(), v.to_string()));
    }
    r
}

fn cookie_value<'a>(req: &'a Req, name: &str) -> Option<&'a str> {
    let h = req.headers.get("cookie")?;
    h.split(';').find_map(|kv| {
        let (k, v) = kv.trim().split_once('=')?;
        (k == name).then_some(v)
    })
}

pub fn handle(state: &UiState, req: &Req) -> Resp {
    finish(route(state, req))
}

fn route(state: &UiState, req: &Req) -> Resp {
    // 1. Host
    let host = req.headers.get("host").map(|h| h.to_ascii_lowercase());
    match host {
        Some(h) if state.hosts().contains(&h) => {}
        _ => return text(403, "Host không hợp lệ"),
    }
    // 2. Origin khi không phải GET
    let is_get = req.method.eq_ignore_ascii_case("GET");
    if !is_get {
        match req.headers.get("origin") {
            Some(o) if state.origins().iter().any(|ok| ok == o) => {}
            _ => return text(403, "Origin không hợp lệ"),
        }
    }
    // 3. Thân
    if req.body.len() > MAX_BODY {
        return text(413, "Thân yêu cầu quá lớn");
    }
    let (path, query) = match req.target.split_once('?') {
        Some((p, q)) => (p, q),
        None => (req.target.as_str(), ""),
    };
    // 4. Đổi vé lấy cookie
    if is_get && path == "/" && query.split('&').any(|kv| kv.starts_with("t=")) {
        let t = parse_form(query).get("t").cloned().unwrap_or_default();
        if ct_eq(&t, &state.ticket)
            && state
                .ticket_used
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        {
            let mut r = mk(303, "text/plain; charset=utf-8", Vec::new());
            r.headers.push(("Location".into(), "/".into()));
            r.headers.push((
                "Set-Cookie".into(),
                format!(
                    "{COOKIE}={}; HttpOnly; SameSite=Strict; Path=/",
                    state.session
                ),
            ));
            return r;
        }
        return text(401, "Vé không đúng hoặc đã dùng");
    }
    // 5. Cookie phiên
    match cookie_value(req, COOKIE) {
        Some(v) if ct_eq(v, &state.session) => {}
        _ => return text(401, "Cần đăng nhập bằng vé"),
    }
    // 6. Định tuyến
    if !is_get {
        if req.method.eq_ignore_ascii_case("POST") {
            if let Some(id) = path
                .strip_prefix("/api/sessions/")
                .and_then(|r| r.strip_suffix("/send"))
            {
                return send_route(state, id, req);
            }
        }
        return text(405, "Không có thao tác ghi này");
    }
    if path == "/api/health" {
        return mk(200, "application/json", br#"{"ok":true}"#.to_vec());
    }
    if path == "/api/sessions" {
        return mk(
            200,
            "application/json",
            state.sessions_json().to_string().into_bytes(),
        );
    }
    if let Some(rest) = path.strip_prefix("/api/sessions/") {
        if let Some(id) = rest.strip_suffix("/stream") {
            return stream_route(state, id, query);
        }
    }
    if path.starts_with("/api/") {
        return text(404, "Không có đường API này");
    }
    serve_static(&state.ui_dir, path)
}

fn json_resp(status: u16, v: serde_json::Value) -> Resp {
    mk(status, "application/json", v.to_string().into_bytes())
}

/// Id phiên đi vào việc tìm TỆP nhật ký / khoá sổ ⟹ chỉ nhận [A-Za-z0-9_-]{1,80}.
fn valid_session_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn err_json(status: u16, msg: &str) -> Resp {
    json_resp(status, serde_json::json!({ "error": msg, "data": null }))
}

/// `POST /api/sessions/<id>/send` (K4) — gõ một tin vào phiên. Thứ tự: kiểu nội dung JSON (chặn biểu mẫu `text/plain`
/// không cần preflight) → thân → id phiên → phiên có trong bản đệm → có nhà cung cấp → CHỐNG GÕ HAI LẦN → gửi.
///
/// Chống gõ hai lần: `client_id` do trang sinh MỖI lần bấm. Đã có kết quả ⟹ trả lại đúng kết quả ấy (`replayed`), KHÔNG
/// gõ nữa — kể cả kết quả mơ hồ; đang gửi ⟹ 409; cùng phiên đang được gửi tin khác ⟹ 409 (hai tin không gõ chồng nhau).
/// Mọi kết cục tới được nhà cung cấp đều trả 200 kèm `result` để trang hiển thị; chỉ lỗi đầu vào/hàng rào mới là mã khác.
fn send_route(state: &UiState, raw_id: &str, req: &Req) -> Resp {
    let id = percent_decode(raw_id);
    if !valid_session_id(&id) {
        return err_json(400, "id phiên không hợp lệ");
    }
    let is_json = req
        .headers
        .get("content-type")
        .map(|c| {
            c.split(';')
                .next()
                .unwrap_or("")
                .trim()
                .eq_ignore_ascii_case("application/json")
        })
        .unwrap_or(false);
    if !is_json {
        return err_json(415, "Chỉ nhận application/json");
    }
    let Ok(body) = serde_json::from_slice::<serde_json::Value>(&req.body) else {
        return err_json(400, "Thân không phải JSON");
    };
    let client_id = body.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
    let valid_client = (8..=64).contains(&client_id.len())
        && client_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid_client {
        return err_json(400, "client_id phải là 8–64 ký tự [A-Za-z0-9_-]");
    }
    let Some(text) = body.get("text").and_then(|v| v.as_str()) else {
        return err_json(400, "thiếu trường text (chuỗi)");
    };
    let Some(row) = state.session_row(&id) else {
        return err_json(404, "không có phiên này trong danh sách");
    };
    let guard = state.send_fn.read().unwrap_or_else(|e| e.into_inner());
    let Some(f) = guard.as_ref() else {
        return err_json(503, "chưa cấu hình đường gửi tin vào phiên");
    };
    {
        let mut b = state.sends.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, v)) = b.done.iter().find(|(c, _)| c == client_id) {
            let mut v = v.clone();
            v["data"]["replayed"] = serde_json::json!(true);
            return json_resp(200, v);
        }
        if b.in_flight.contains(client_id) {
            return err_json(409, "tin này đang được gửi");
        }
        if b.busy_sessions.contains(&id) {
            return err_json(409, "phiên đang nhận một tin khác — chờ xong rồi gửi tiếp");
        }
        b.in_flight.insert(client_id.to_string());
        b.busy_sessions.insert(id.clone());
    }
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&row, text)))
        .unwrap_or_else(|_| crate::ui_send::SendResult {
            outcome: crate::ui_send::Outcome::Failed,
            message: "việc gửi bị lỗi nội bộ (panic) — xem phiên trước khi gửi lại".into(),
        });
    let resp = serde_json::json!({
        "data": { "result": r.outcome.as_str(), "message": r.message, "replayed": false },
        "now_ms": now_ms(),
        "error": null,
    });
    {
        let mut b = state.sends.lock().unwrap_or_else(|e| e.into_inner());
        b.in_flight.remove(client_id);
        b.busy_sessions.remove(&id);
        b.done.push_back((client_id.to_string(), resp.clone()));
        while b.done.len() > SEND_REMEMBER {
            b.done.pop_front();
        }
    }
    json_resp(200, resp)
}

/// `GET /api/sessions/<id>/stream[?limit=N]`. `<id>` đi vào việc tìm TỆP nhật ký ⟹ chỉ nhận [A-Za-z0-9_-]{1,80} (sai ⟹ 400,
/// không phải 404: không cho thử dò); id phải CÓ trong bản đệm (lạ ⟹ 404, không chạm nhà cung cấp); `limit` kẹp trong [1, 300].
fn stream_route(state: &UiState, raw_id: &str, query: &str) -> Resp {
    let id = percent_decode(raw_id);
    if !valid_session_id(&id) {
        return json_resp(
            400,
            serde_json::json!({ "error": "id phiên không hợp lệ", "data": null }),
        );
    }
    let limit = parse_form(query)
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|n| *n >= 0)
        .map(|n| (n as usize).clamp(1, STREAM_MAX))
        .unwrap_or(STREAM_DEFAULT);
    let Some(row) = state.session_row(&id) else {
        return json_resp(
            404,
            serde_json::json!({ "error": "không có phiên này trong danh sách", "data": null }),
        );
    };
    let guard = state.stream_fn.read().unwrap_or_else(|e| e.into_inner());
    let Some(f) = guard.as_ref() else {
        return json_resp(
            503,
            serde_json::json!({ "error": "chưa cấu hình nguồn nội dung phiên", "data": null }),
        );
    };
    match f(&row, limit) {
        Ok(v) => json_resp(
            200,
            serde_json::json!({ "data": v, "now_ms": now_ms(), "error": null }),
        ),
        Err(e) => {
            crate::logging::warn(
                "ui_stream_failed",
                serde_json::json!({ "session": id, "err": e }),
            );
            json_resp(500, serde_json::json!({ "error": e, "data": null }))
        }
    }
}

/// Tệp tĩnh dưới `ui_dir`. `400` cho đường dẫn dị dạng, `404` cho không thấy HOẶC thoát khỏi `ui_dir`.
fn serve_static(ui_dir: &Path, path: &str) -> Resp {
    let decoded = percent_decode(path);
    if decoded.contains('\0') || decoded.contains('\\') || !decoded.starts_with('/') {
        return text(400, "Đường dẫn không hợp lệ");
    }
    let rel = decoded.trim_start_matches('/');
    let segs: Vec<&str> = if rel.is_empty() {
        vec!["index.html"]
    } else {
        rel.split('/').collect()
    };
    // `trim_start_matches('/')` đã nuốt MỌI dấu `/` đầu ⟹ `//etc/passwd` thành `etc/passwd`; vì vậy kiểm lại bản gốc.
    if decoded.starts_with("//") || segs.iter().any(|s| s.is_empty() || *s == "." || *s == "..") {
        return text(400, "Đường dẫn không hợp lệ");
    }
    let mut target = ui_dir.to_path_buf();
    for s in &segs {
        target.push(s);
    }
    let (Ok(root), Ok(file)) = (ui_dir.canonicalize(), target.canonicalize()) else {
        return text(404, "Không thấy");
    };
    if !file.starts_with(&root) || !file.is_file() {
        return text(404, "Không thấy");
    }
    match std::fs::read(&file) {
        Ok(bytes) => mk(200, ctype_of(&file), bytes),
        Err(_) => text(404, "Không thấy"),
    }
}

fn ctype_of(p: &Path) -> &'static str {
    match p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

// ── Vòng lắng nghe thật (W1.8) ─────────────────────────────────────────────────────────────────

/// Số kết nối xử lý cùng lúc tối đa: một máy một chủ nhân, 32 là thừa; trần này chỉ để một vòng lặp lỗi/kẻ lạ không đẻ
/// thread vô hạn.
const MAX_CONN: usize = 32;
/// Quá hạn đọc/ghi một kết nối: không để một ổ cắm treo giữ thread mãi.
const IO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Thư mục giao diện mặc định: env `HUBA_UI_DIR` → `<thư mục chứa huba>/ui` (gói cài) → `<hub_home>/ui` → `<hub_home>/docs/ui`
/// (cây mã). Chỉ nhận thư mục CÓ `index.html`.
pub fn default_ui_dir(hub_home: &Path) -> Option<PathBuf> {
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Ok(d) = std::env::var("HUBA_UI_DIR") {
        if !d.trim().is_empty() {
            cands.push(PathBuf::from(d));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            cands.push(dir.join("ui"));
        }
    }
    cands.push(hub_home.join("ui"));
    cands.push(hub_home.join("docs").join("ui"));
    cands.into_iter().find(|d| d.join("index.html").is_file())
}

/// Chạy vòng nhận kết nối trên `listener` (chặn mãi mãi). Mỗi kết nối một thread, có hạn đọc/ghi và trần số kết nối.
pub fn serve_loop(listener: std::net::TcpListener, state: std::sync::Arc<UiState>) {
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    let live = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                crate::logging::warn(
                    "ui_accept_failed",
                    serde_json::json!({ "err": e.to_string() }),
                );
                continue;
            }
        };
        if live.fetch_add(1, Ordering::SeqCst) >= MAX_CONN {
            live.fetch_sub(1, Ordering::SeqCst);
            let _ = write_out(&mut stream, finish(text(503, "Quá nhiều kết nối")));
            continue;
        }
        let (state, live) = (Arc::clone(&state), Arc::clone(&live));
        std::thread::spawn(move || {
            let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
            let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
            let resp = match crate::localserver::read_request(&mut stream) {
                Ok(req) => handle(&state, &req),
                Err(e) => {
                    let m = e.to_string();
                    let code = if m.contains("body_too_large") {
                        413
                    } else if m.contains("headers_too_large") {
                        431
                    } else {
                        400
                    };
                    // Lỗi đọc thường là kẻ lạ gửi rác hoặc hết hạn — ghi để truy, không ồn.
                    crate::logging::info(
                        "ui_bad_request",
                        serde_json::json!({ "why": m, "status": code }),
                    );
                    finish(text(code, "Yêu cầu không hợp lệ"))
                }
            };
            if let Err(e) = write_out(&mut stream, resp) {
                crate::logging::info(
                    "ui_write_failed",
                    serde_json::json!({ "err": e.to_string() }),
                );
            }
            live.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

fn write_out(stream: &mut std::net::TcpStream, r: Resp) -> anyhow::Result<()> {
    let status = match r.status {
        200 => "200 OK",
        303 => "303 See Other",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        405 => "405 Method Not Allowed",
        409 => "409 Conflict",
        413 => "413 Payload Too Large",
        415 => "415 Unsupported Media Type",
        431 => "431 Request Header Fields Too Large",
        503 => "503 Service Unavailable",
        _ => "500 Internal Server Error",
    };
    let hs: Vec<(&str, String)> = r
        .headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();
    crate::localserver::write_resp(stream, status, &r.ctype, &hs, &r.body)
}

/// Thread làm mới danh sách phiên: gọi `produce` mỗi `every`, ghi vào bản đệm. `produce` panic cũng chỉ thành một lỗi được ghi,
/// không làm chết thread (không thì bản đệm đứng im mà không ai biết).
pub fn spawn_refresher<F>(state: std::sync::Arc<UiState>, every: std::time::Duration, produce: F)
where
    F: Fn() -> Result<serde_json::Value, String> + Send + 'static,
{
    std::thread::spawn(move || loop {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(&produce))
            .unwrap_or_else(|_| Err("việc đọc danh sách phiên bị panic".to_string()));
        if let Err(e) = &r {
            crate::logging::warn(
                "ui_sessions_refresh_failed",
                serde_json::json!({ "err": e }),
            );
        }
        state.put_sessions(r, now_ms());
        std::thread::sleep(every);
    });
}

/// Danh sách phiên THẬT: cùng nguồn với `huba sessions --json` (snapshot + đánh dấu phiên do huba mở).
fn produce_sessions(cfg: &crate::config::Config) -> Result<serde_json::Value, String> {
    let mut snap = crate::sessions::snapshot(cfg);
    let db =
        crate::db::Db::open(&cfg.db).map_err(|e| format!("không mở được cơ sở dữ liệu: {e}"))?;
    crate::pipeline::mark_started_by_hub(&db, &mut snap);
    serde_json::to_value(&snap).map_err(|e| format!("không mã hoá được snapshot: {e}"))
}

/// Nội dung MỘT phiên từ nhật ký của CLI. Hiện chỉ Claude có nguồn (`sessions::stream`); phiên khác trả ghi chú thật thà thay vì
/// một khung chat rỗng (rỗng = nói dối rằng phiên chưa nói gì).
fn produce_stream(
    cfg: &crate::config::Config,
    row: &serde_json::Value,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let id = row.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
    let cwd = row.get("cwd").and_then(|v| v.as_str()).unwrap_or("");
    let kind = row.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    if kind != "interactive" {
        return Ok(serde_json::json!({
            "session_id": id,
            "events": [],
            "older_hidden": 0,
            "note": format!("Giao diện chưa đọc được nội dung phiên loại «{kind}» (hiện chỉ Claude). Xem bằng Telegram /shot hoặc terminal.")
        }));
    }
    let st = crate::sessions::stream(cfg, id, cwd, limit);
    serde_json::to_value(&st).map_err(|e| format!("không mã hoá được nội dung phiên: {e}"))
}

/// Gửi MỘT tin vào phiên THẬT (K4). Hàng phiên trong bản đệm là chính `LiveSession` đã mã hoá — đọc ngược lại, không dựng
/// lại ảnh chụp (4,7 s). Không đọc ngược được ⟹ từ chối, không đoán.
fn produce_send(
    cfg: &crate::config::Config,
    row: &serde_json::Value,
    text: &str,
) -> crate::ui_send::SendResult {
    match serde_json::from_value::<crate::sessions::LiveSession>(row.clone()) {
        Ok(s) => crate::ui_send::send(cfg, &s, text),
        Err(e) => {
            crate::logging::warn(
                "ui_send_row_unreadable",
                serde_json::json!({ "err": e.to_string() }),
            );
            crate::ui_send::SendResult {
                outcome: crate::ui_send::Outcome::Refused,
                message: format!("Không đọc được hàng phiên trong bản đệm ({e}) nên không gõ."),
            }
        }
    }
}

/// `huba ui`: mở cổng 127.0.0.1, in URL (có vé), tuỳ chọn mở cửa sổ, rồi phục vụ mãi.
pub fn serve(cfg: crate::config::Config, ui_dir: PathBuf, open: bool) -> anyhow::Result<()> {
    if !ui_dir.join("index.html").is_file() {
        anyhow::bail!(
            "thư mục giao diện «{}» không có index.html — đặt HUBA_UI_DIR hoặc --dir",
            ui_dir.display()
        );
    }
    let (listener, port) = crate::localserver::bind_loopback()?;
    let state = UiState::try_new(port, ui_dir).map_err(|e| anyhow::anyhow!(e))?;
    let url = format!("http://127.0.0.1:{port}/?t={}", state.ticket());
    crate::logging::info("ui_started", serde_json::json!({ "port": port }));
    println!("\n  Giao diện huba:\n\n    {url}\n");
    println!("  (chỉ máy này vào được; vé trên URL dùng MỘT lần — Ctrl-C để đóng)\n");
    if open {
        crate::localserver::open_window(&url, "ui");
    }
    let cfg_stream = cfg.clone();
    state.set_stream_provider(Box::new(move |row, limit| {
        produce_stream(&cfg_stream, row, limit)
    }));
    let cfg_send = cfg.clone();
    state.set_send_provider(Box::new(move |row, text| {
        produce_send(&cfg_send, row, text)
    }));
    let state = std::sync::Arc::new(state);
    let cfg_dem = cfg.clone();
    spawn_refresher(
        std::sync::Arc::clone(&state),
        std::time::Duration::from_secs(10),
        move || produce_sessions(&cfg_dem),
    );
    serve_loop(listener, state);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_compare_agrees_with_equality() {
        assert!(ct_eq("abc", "abc"));
        assert!(!ct_eq("abc", "abd"));
        assert!(!ct_eq("abc", "ab"));
        assert!(!ct_eq("", "a"));
        assert!(ct_eq("", ""));
    }
}
