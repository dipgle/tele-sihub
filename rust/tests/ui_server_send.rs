//! `POST /api/sessions/<id>/send` (K4): gửi một tin từ giao diện vào phiên.
//!
//! Đường này GÕ được vào phiên CLI nên hàng rào quan trọng hơn mọi đường khác. Ở đây dùng nhà cung cấp GIẢ (không đụng
//! Terminal): kiểm Host/Origin/cookie, kiểu nội dung, trần thân, id phiên, `client_id`, chống gửi hai lần, và CHỈ nhà cung
//! cấp mới được chạm vào phiên. Luật từ chối theo từng loại phiên nằm ở `tests/ui_send_rules.rs`.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use huba::localserver::Req;
use huba::ui_send::{Outcome, SendResult};
use huba::uiserver::{handle, UiState, COOKIE};
use serde_json::{json, Value};

const PORT: u16 = 45874;

fn state() -> (UiState, tempfile::TempDir) {
    let d = tempfile::tempdir().unwrap();
    let ui = d.path().join("ui");
    std::fs::create_dir_all(&ui).unwrap();
    std::fs::write(ui.join("index.html"), "x").unwrap();
    let s = UiState::new(PORT, ui);
    s.put_sessions(
        Ok(json!({"sessions":[
            {"session_id":"s-claude","cwd":"/tmp/a","kind":"interactive","account":"acc6"},
            {"session_id":"s-khac","cwd":"/tmp/b","kind":"interactive","account":"acc6"}
        ]})),
        1,
    );
    (s, d)
}

struct Gui<'a> {
    s: &'a UiState,
    target: &'a str,
    cookie: bool,
    origin: Option<&'a str>,
    host: Option<&'a str>,
    ctype: Option<&'a str>,
    body: Vec<u8>,
}

fn gui<'a>(s: &'a UiState, target: &'a str, body: Value) -> Gui<'a> {
    Gui {
        s,
        target,
        cookie: true,
        origin: Some("http://127.0.0.1:45874"),
        host: Some("127.0.0.1:45874"),
        ctype: Some("application/json"),
        body: body.to_string().into_bytes(),
    }
}

impl Gui<'_> {
    fn chay(&self) -> (u16, Value) {
        let mut h = BTreeMap::new();
        if let Some(x) = self.host {
            h.insert("host".to_string(), x.to_string());
        }
        if let Some(x) = self.origin {
            h.insert("origin".to_string(), x.to_string());
        }
        if let Some(x) = self.ctype {
            h.insert("content-type".to_string(), x.to_string());
        }
        if self.cookie {
            h.insert(
                "cookie".to_string(),
                format!("{COOKIE}={}", self.s.session()),
            );
        }
        let r = handle(
            self.s,
            &Req {
                method: "POST".into(),
                target: self.target.into(),
                headers: h,
                body: self.body.clone(),
            },
        );
        let txt = String::from_utf8_lossy(&r.body).to_string();
        (r.status, serde_json::from_str(&txt).unwrap_or(Value::Null))
    }
}

type LanGoi = (String, String);

/// Nhà cung cấp giả: ghi lại (id phiên, chữ) rồi trả `ket_qua`.
fn gan(s: &UiState, ket_qua: Outcome) -> Arc<Mutex<Vec<LanGoi>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let l2 = Arc::clone(&log);
    s.set_send_provider(Box::new(move |row, text| {
        l2.lock().unwrap().push((
            row["session_id"].as_str().unwrap_or("").to_string(),
            text.to_string(),
        ));
        SendResult {
            outcome: ket_qua,
            message: "giả".into(),
        }
    }));
    log
}

const ID_TOT: &str = "abcdef0123456789";

#[test]
fn a_good_send_reaches_the_provider_once_and_says_sent() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    let (code, v) = gui(
        &s,
        "/api/sessions/s-claude/send",
        json!({"client_id": ID_TOT, "text": "chào phiên"}),
    )
    .chay();
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["data"]["result"], "sent");
    assert_eq!(
        log.lock().unwrap().as_slice(),
        &[("s-claude".to_string(), "chào phiên".to_string())]
    );
}

/// Không cookie ⟹ 401 và nhà cung cấp KHÔNG được chạm.
#[test]
fn no_cookie_means_no_typing() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    let mut g = gui(
        &s,
        "/api/sessions/s-claude/send",
        json!({"client_id": ID_TOT, "text": "x"}),
    );
    g.cookie = false;
    assert_eq!(g.chay().0, 401);
    assert!(log.lock().unwrap().is_empty());
}

/// Origin lạ hoặc thiếu ⟹ 403 (CSRF); Host lạ ⟹ 403 (DNS-rebinding).
#[test]
fn a_foreign_origin_or_host_is_refused_before_anything_else() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    for (origin, host) in [
        (Some("http://evil.example"), Some("127.0.0.1:45874")),
        (None, Some("127.0.0.1:45874")),
        (Some("http://127.0.0.1:45874"), Some("evil.example")),
    ] {
        let mut g = gui(
            &s,
            "/api/sessions/s-claude/send",
            json!({"client_id": ID_TOT, "text": "x"}),
        );
        g.origin = origin;
        g.host = host;
        assert_eq!(g.chay().0, 403, "origin={origin:?} host={host:?}");
    }
    assert!(log.lock().unwrap().is_empty());
}

/// Chỉ nhận JSON: biểu mẫu `text/plain` (CSRF không cần preflight) bị từ chối.
#[test]
fn only_json_bodies_are_accepted() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    for ct in [
        Some("text/plain"),
        Some("application/x-www-form-urlencoded"),
        None,
    ] {
        let mut g = gui(
            &s,
            "/api/sessions/s-claude/send",
            json!({"client_id": ID_TOT, "text": "x"}),
        );
        g.ctype = ct;
        assert_eq!(g.chay().0, 415, "content-type={ct:?}");
    }
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn a_bad_body_is_400_and_never_typed() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    for body in [
        json!({"text": "thiếu client_id"}),
        json!({"client_id": ID_TOT}),
        json!({"client_id": "ngắn", "text": "x"}),
        json!({"client_id": "có khoảng trắng và dấu!", "text": "x"}),
        json!({"client_id": ID_TOT, "text": 5}),
        json!([1, 2]),
    ] {
        let (code, _) = gui(&s, "/api/sessions/s-claude/send", body.clone()).chay();
        assert_eq!(code, 400, "{body}");
    }
    let mut g = gui(&s, "/api/sessions/s-claude/send", json!({}));
    g.body = b"{khong phai json".to_vec();
    assert_eq!(g.chay().0, 400);
    assert!(log.lock().unwrap().is_empty());
}

/// Id phiên dị dạng ⟹ 400; id không có trong danh sách ⟹ 404 — cả hai đều không chạm nhà cung cấp.
#[test]
fn the_session_id_is_checked_like_the_stream_route() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    let b = json!({"client_id": ID_TOT, "text": "x"});
    assert_eq!(
        gui(&s, "/api/sessions/..%2Fetc/send", b.clone()).chay().0,
        400
    );
    assert_eq!(gui(&s, "/api/sessions/a b/send", b.clone()).chay().0, 400);
    assert_eq!(
        gui(&s, "/api/sessions/khong-co/send", b.clone()).chay().0,
        404
    );
    assert!(log.lock().unwrap().is_empty());
}

/// Thân quá trần ⟹ 413 (trần chung của máy chủ), không tới nhà cung cấp.
#[test]
fn an_oversized_body_is_413() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    let to = "x".repeat(huba::uiserver::MAX_BODY + 10);
    let (code, _) = gui(
        &s,
        "/api/sessions/s-claude/send",
        json!({"client_id": ID_TOT, "text": to}),
    )
    .chay();
    assert_eq!(code, 413);
    assert!(log.lock().unwrap().is_empty());
}

/// Chưa gắn nhà cung cấp ⟹ 503 thật thà, không im lặng nhận rồi bỏ.
#[test]
fn without_a_provider_it_says_503() {
    let (s, _d) = state();
    let (code, v) = gui(
        &s,
        "/api/sessions/s-claude/send",
        json!({"client_id": ID_TOT, "text": "x"}),
    )
    .chay();
    assert_eq!(code, 503);
    assert!(v["error"].is_string());
}

/// CHỐNG GỬI HAI LẦN: cùng `client_id` lặp lại ⟹ KHÔNG gõ lần nữa, trả lại đúng kết quả cũ.
#[test]
fn the_same_client_id_is_never_typed_twice() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    let b = json!({"client_id": ID_TOT, "text": "một lần thôi"});
    let (c1, v1) = gui(&s, "/api/sessions/s-claude/send", b.clone()).chay();
    let (c2, v2) = gui(&s, "/api/sessions/s-claude/send", b.clone()).chay();
    assert_eq!((c1, c2), (200, 200));
    assert_eq!(v1["data"]["result"], "sent");
    assert_eq!(v2["data"]["result"], "sent");
    assert_eq!(v2["data"]["replayed"], true);
    assert_eq!(
        log.lock().unwrap().len(),
        1,
        "lần lặp lại đã gõ thêm vào phiên"
    );
}

/// ĐỐI CHỨNG NGƯỢC: `client_id` KHÁC thì là một lần bấm khác — phải gõ thật.
#[test]
fn a_different_client_id_is_a_new_send() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    for id in ["aaaaaaaa11111111", "bbbbbbbb22222222"] {
        let (c, _) = gui(
            &s,
            "/api/sessions/s-claude/send",
            json!({"client_id": id, "text": "x"}),
        )
        .chay();
        assert_eq!(c, 200);
    }
    assert_eq!(log.lock().unwrap().len(), 2);
}

/// Kết quả MƠ HỒ cũng được nhớ theo `client_id`: bấm lặp lại không được đẻ thêm một lần gõ.
#[test]
fn an_ambiguous_result_is_not_retried_by_replay() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Unverified);
    let b = json!({"client_id": ID_TOT, "text": "x"});
    let (_, v1) = gui(&s, "/api/sessions/s-claude/send", b.clone()).chay();
    let (_, v2) = gui(&s, "/api/sessions/s-claude/send", b.clone()).chay();
    assert_eq!(v1["data"]["result"], "unverified");
    assert_eq!(v2["data"]["result"], "unverified");
    assert_eq!(log.lock().unwrap().len(), 1);
}

/// Hai lần gửi CÙNG LÚC vào cùng một phiên (client_id khác nhau) ⟹ lần sau bị 409, không gõ chồng lên nhau.
/// (Dùng kênh có hạn chờ: nếu hàng rào hỏng thì bài ĐỎ chứ không treo.)
#[test]
fn one_send_at_a_time_per_session() {
    use std::sync::mpsc;
    use std::time::Duration;
    let (s, _d) = state();
    let s = Arc::new(s);
    let (vao_tx, vao_rx) = mpsc::channel::<()>();
    let (tha_tx, tha_rx) = mpsc::channel::<()>();
    let tha_rx = Mutex::new(tha_rx);
    let vao_tx = Mutex::new(vao_tx);
    let so_lan = Arc::new(Mutex::new(0usize));
    let sl = Arc::clone(&so_lan);
    s.set_send_provider(Box::new(move |_row, _t| {
        *sl.lock().unwrap() += 1;
        let _ = vao_tx.lock().unwrap().send(()); // báo: đang ở trong nhà cung cấp
        let _ = tha_rx.lock().unwrap().recv_timeout(Duration::from_secs(5)); // giữ tới khi kiểm xong
        SendResult {
            outcome: Outcome::Sent,
            message: "".into(),
        }
    }));
    let s1 = Arc::clone(&s);
    let t = std::thread::spawn(move || {
        gui(
            &s1,
            "/api/sessions/s-claude/send",
            json!({"client_id": "aaaaaaaa11111111", "text": "một"}),
        )
        .chay()
        .0
    });
    vao_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("lần gửi đầu không tới nhà cung cấp");
    let (c, _) = gui(
        &s,
        "/api/sessions/s-claude/send",
        json!({"client_id": "bbbbbbbb22222222", "text": "hai"}),
    )
    .chay();
    let _ = tha_tx.send(());
    let _ = tha_tx.send(());
    assert_eq!(c, 409, "gõ chồng vào phiên đang được gõ");
    assert_eq!(t.join().unwrap(), 200);
    assert_eq!(*so_lan.lock().unwrap(), 1, "nhà cung cấp bị gọi chồng");
}

/// GET vào đường gửi ⟹ không phải đường đọc: 405, không gõ gì.
#[test]
fn get_on_the_send_route_is_not_allowed() {
    let (s, _d) = state();
    let log = gan(&s, Outcome::Sent);
    let mut h = BTreeMap::new();
    h.insert("host".to_string(), format!("127.0.0.1:{PORT}"));
    h.insert("cookie".to_string(), format!("{COOKIE}={}", s.session()));
    let r = handle(
        &s,
        &Req {
            method: "GET".into(),
            target: "/api/sessions/s-claude/send".into(),
            headers: h,
            body: vec![],
        },
    );
    assert!(r.status == 404 || r.status == 405, "status={}", r.status);
    assert!(log.lock().unwrap().is_empty());
}
