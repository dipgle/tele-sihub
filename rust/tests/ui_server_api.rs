//! `/api/sessions` của giao diện web (W2.1/W2.3): bản đệm danh sách phiên và ba trạng thái bắt buộc của thiết kế §3
//! («Giữ ảnh trước + "Dữ liệu cũ"; KHÔNG báo 0 phiên khi probe lỗi»).

use std::collections::BTreeMap;

use huba::localserver::Req;
use huba::uiserver::{handle, UiState, COOKIE};
use serde_json::{json, Value};

const PORT: u16 = 45872;

fn state() -> (UiState, tempfile::TempDir) {
    let d = tempfile::tempdir().unwrap();
    let ui = d.path().join("ui");
    std::fs::create_dir_all(&ui).unwrap();
    std::fs::write(ui.join("index.html"), "x").unwrap();
    (UiState::new(PORT, ui), d)
}

fn api(s: &UiState) -> (u16, Value) {
    let mut headers = BTreeMap::new();
    headers.insert("host".to_string(), format!("127.0.0.1:{PORT}"));
    headers.insert("cookie".to_string(), format!("{COOKIE}={}", s.session()));
    let r = handle(
        s,
        &Req {
            method: "GET".into(),
            target: "/api/sessions".into(),
            headers,
            body: vec![],
        },
    );
    (
        r.status,
        serde_json::from_slice(&r.body).unwrap_or(Value::Null),
    )
}

fn snap(n: usize) -> Value {
    json!({
        "sessions": (0..n).map(|i| json!({"session_id": format!("s{i}"), "name": format!("phien {i}")})).collect::<Vec<_>>(),
        "notes": [], "hidden_editor": 0, "hidden_dead": 0
    })
}

#[test]
fn before_the_first_refresh_the_api_says_loading_not_empty() {
    let (s, _d) = state();
    let (code, v) = api(&s);
    assert_eq!(code, 200);
    assert_eq!(v["loading"], true, "{v}");
    assert!(v["data"].is_null(), "chưa đọc xong mà đã có dữ liệu: {v}");
    assert!(v["error"].is_null());
}

#[test]
fn a_good_refresh_serves_the_snapshot() {
    let (s, _d) = state();
    s.put_sessions(Ok(snap(3)), 1_000);
    let (_, v) = api(&s);
    assert_eq!(v["loading"], false);
    assert!(v["error"].is_null());
    assert_eq!(v["ok_at_ms"], 1_000);
    assert_eq!(v["data"]["sessions"].as_array().unwrap().len(), 3);
    assert!(v["now_ms"].as_i64().unwrap() > 0);
}

/// 🔴 Ca chính: đọc lỗi SAU một lần đọc tốt ⟹ vẫn thấy bản cũ + lỗi, KHÔNG phải «0 phiên».
#[test]
fn a_failed_refresh_keeps_the_old_data_and_reports_the_error() {
    let (s, _d) = state();
    s.put_sessions(Ok(snap(5)), 1_000);
    s.put_sessions(Err("claude agents treo".into()), 2_000);
    let (_, v) = api(&s);
    assert_eq!(
        v["data"]["sessions"].as_array().unwrap().len(),
        5,
        "mất bản cũ khi lỗi: {v}"
    );
    assert_eq!(v["error"], "claude agents treo");
    assert_eq!(v["ok_at_ms"], 1_000, "mốc đọc tốt cuối phải giữ nguyên");
}

/// Đối chứng ngược: lần đọc tốt kế tiếp xoá lỗi và đổi mốc.
#[test]
fn the_next_good_refresh_clears_the_error() {
    let (s, _d) = state();
    s.put_sessions(Ok(snap(5)), 1_000);
    s.put_sessions(Err("x".into()), 2_000);
    s.put_sessions(Ok(snap(2)), 3_000);
    let (_, v) = api(&s);
    assert!(v["error"].is_null(), "{v}");
    assert_eq!(v["ok_at_ms"], 3_000);
    assert_eq!(v["data"]["sessions"].as_array().unwrap().len(), 2);
}

#[test]
fn an_error_with_no_prior_data_is_an_error_not_an_empty_list() {
    let (s, _d) = state();
    s.put_sessions(Err("không mở được cơ sở dữ liệu".into()), 500);
    let (_, v) = api(&s);
    assert_eq!(v["loading"], false, "đã thử rồi thì không còn «đang tải»");
    assert!(v["data"].is_null(), "lỗi mà vẫn trả danh sách: {v}");
    assert_eq!(v["error"], "không mở được cơ sở dữ liệu");
}

#[test]
fn the_sessions_api_needs_the_cookie() {
    let (s, _d) = state();
    s.put_sessions(Ok(snap(1)), 1);
    let mut headers = BTreeMap::new();
    headers.insert("host".to_string(), format!("127.0.0.1:{PORT}"));
    let r = handle(
        &s,
        &Req {
            method: "GET".into(),
            target: "/api/sessions".into(),
            headers,
            body: vec![],
        },
    );
    assert_eq!(r.status, 401);
    assert!(
        !String::from_utf8_lossy(&r.body).contains("phien 0"),
        "lộ dữ liệu phiên khi chưa đăng nhập"
    );
}

/// Thread làm mới: panic trong `produce` không làm chết nó (bản đệm vẫn cập nhật ở vòng sau).
#[test]
fn the_refresher_survives_a_panic_and_recovers() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    let (s, _d) = state();
    let s = Arc::new(s);
    let n = Arc::new(AtomicUsize::new(0));
    let n2 = Arc::clone(&n);
    huba::uiserver::spawn_refresher(
        Arc::clone(&s),
        std::time::Duration::from_millis(30),
        move || {
            if n2.fetch_add(1, Ordering::SeqCst) == 0 {
                panic!("giả lỗi");
            }
            Ok(snap(4))
        },
    );
    let t0 = std::time::Instant::now();
    loop {
        let (_, v) = api(&s);
        if v["data"]["sessions"].as_array().map(|a| a.len()) == Some(4) {
            assert!(v["error"].is_null());
            break;
        }
        assert!(
            t0.elapsed() < std::time::Duration::from_secs(3),
            "refresher chết sau panic: {v}"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
