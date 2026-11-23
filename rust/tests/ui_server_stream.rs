//! `GET /api/sessions/<id>/stream` (K3.1): nội dung một phiên cho màn Chat.
//!
//! Hàng rào: cần cookie; `<id>` đi vào việc tìm TỆP nhật ký nên chỉ nhận [A-Za-z0-9_-] (không `..`, không `/`); id phải CÓ trong
//! danh sách phiên đang đệm (không cho dò id lạ); `limit` bị kẹp; nguồn lỗi trả JSON lỗi chứ không sập.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use huba::localserver::Req;
use huba::uiserver::{handle, UiState, COOKIE};
use serde_json::{json, Value};

const PORT: u16 = 45873;

fn state() -> (UiState, tempfile::TempDir) {
    let d = tempfile::tempdir().unwrap();
    let ui = d.path().join("ui");
    std::fs::create_dir_all(&ui).unwrap();
    std::fs::write(ui.join("index.html"), "x").unwrap();
    let s = UiState::new(PORT, ui);
    s.put_sessions(
        Ok(json!({"sessions":[
            {"session_id":"s-claude","cwd":"/tmp/a","kind":"interactive","account":"acc6"},
            {"session_id":"s-oc","cwd":"/tmp/b","kind":"opencode","account":"acc8"}
        ]})),
        1,
    );
    (s, d)
}

fn get(s: &UiState, target: &str, cookie: bool) -> (u16, Value, String) {
    let mut headers = BTreeMap::new();
    headers.insert("host".to_string(), format!("127.0.0.1:{PORT}"));
    if cookie {
        headers.insert("cookie".to_string(), format!("{COOKIE}={}", s.session()));
    }
    let r = handle(
        s,
        &Req {
            method: "GET".into(),
            target: target.into(),
            headers,
            body: vec![],
        },
    );
    let txt = String::from_utf8_lossy(&r.body).to_string();
    (
        r.status,
        serde_json::from_str(&txt).unwrap_or(Value::Null),
        txt,
    )
}

/// Nhà cung cấp giả: ghi lại (id, cwd, kind, limit) mà nó nhận.
/// Một lần gọi nhà cung cấp giả: (id, cwd, kind, limit).
type LanGoi = (String, String, String, usize);

fn gan(s: &UiState) -> Arc<Mutex<Vec<LanGoi>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let l2 = Arc::clone(&log);
    s.set_stream_provider(Box::new(move |sess, limit| {
        let id = sess["session_id"].as_str().unwrap_or("").to_string();
        l2.lock().unwrap().push((
            id.clone(),
            sess["cwd"].as_str().unwrap_or("").to_string(),
            sess["kind"].as_str().unwrap_or("").to_string(),
            limit,
        ));
        if id == "s-boom" {
            return Err("đọc nhật ký hỏng".to_string());
        }
        Ok(json!({"events":[{"kind":"say","role":"user","text":"chào"}], "older_hidden": 0}))
    }));
    log
}

#[test]
fn the_stream_needs_the_cookie() {
    let (s, _d) = state();
    let _ = gan(&s);
    let (code, _, txt) = get(&s, "/api/sessions/s-claude/stream", false);
    assert_eq!(code, 401);
    assert!(
        !txt.contains("chào"),
        "lộ nội dung phiên khi chưa đăng nhập"
    );
}

#[test]
fn a_known_session_returns_its_events_and_the_provider_sees_the_session() {
    let (s, _d) = state();
    let log = gan(&s);
    let (code, v, _) = get(&s, "/api/sessions/s-claude/stream", true);
    assert_eq!(code, 200);
    assert_eq!(v["data"]["events"][0]["text"], "chào");
    let l = log.lock().unwrap();
    assert_eq!(l.len(), 1);
    assert_eq!(
        (l[0].0.as_str(), l[0].1.as_str(), l[0].2.as_str()),
        ("s-claude", "/tmp/a", "interactive")
    );
}

#[test]
fn an_unknown_session_is_404_and_never_reaches_the_provider() {
    let (s, _d) = state();
    let log = gan(&s);
    let (code, _, _) = get(&s, "/api/sessions/khong-co/stream", true);
    assert_eq!(code, 404);
    assert!(
        log.lock().unwrap().is_empty(),
        "id lạ vẫn chạm tới nhà cung cấp"
    );
}

/// `<id>` đi vào việc tìm tệp: ký tự lạ ⟹ 400, KHÔNG phải 404 (không cho thử dò).
#[test]
fn a_malformed_id_is_400() {
    let (s, _d) = state();
    let log = gan(&s);
    let dai = "a".repeat(81);
    for id in ["..", "..%2Fetc", "a%2Fb", "a b", "a%00b", dai.as_str()] {
        let (code, _, _) = get(&s, &format!("/api/sessions/{id}/stream"), true);
        assert_eq!(code, 400, "id «{id}» không bị chặn");
    }
    assert!(log.lock().unwrap().is_empty());
    // đối chứng ngược: id hợp lệ có dấu gạch vẫn đi tiếp tới 404/200 (không bị 400 oan)
    let (code, _, _) = get(&s, "/api/sessions/s-claude/stream", true);
    assert_eq!(code, 200);
}

#[test]
fn limit_defaults_is_clamped_and_ignores_garbage() {
    let (s, _d) = state();
    let log = gan(&s);
    for (q, want) in [
        ("", 80usize),
        ("?limit=5", 5),
        ("?limit=99999", 300),
        ("?limit=0", 1),
        ("?limit=abc", 80),
        ("?limit=-3", 80),
    ] {
        let _ = get(&s, &format!("/api/sessions/s-claude/stream{q}"), true);
        let got = log.lock().unwrap().last().unwrap().3;
        assert_eq!(got, want, "«{q}»");
    }
}

#[test]
fn a_provider_error_is_a_json_error_not_a_crash() {
    let (s, _d) = state();
    let _ = gan(&s);
    s.put_sessions(
        Ok(json!({"sessions":[{"session_id":"s-boom","cwd":"/x","kind":"interactive"}]})),
        2,
    );
    let (code, v, _) = get(&s, "/api/sessions/s-boom/stream", true);
    assert_eq!(code, 500);
    assert_eq!(v["error"], "đọc nhật ký hỏng");
    assert!(v["data"].is_null());
}

#[test]
fn without_a_provider_the_api_says_it_is_not_configured() {
    let (s, _d) = state();
    let (code, v, _) = get(&s, "/api/sessions/s-claude/stream", true);
    assert_eq!(code, 503);
    assert!(v["error"].is_string());
}
