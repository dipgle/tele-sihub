//! Cùng các hàng rào của `ui_server_security.rs`, nhưng qua Ổ CẮM THẬT: kiểm vòng `serve_loop` (đọc request thật, trần
//! thân, kết nối treo không chặn kết nối khác), thứ mà bài kiểm hàm thuần không chạm tới.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use huba::localserver::bind_loopback;
use huba::uiserver::{serve_loop, UiState};

struct Chay {
    port: u16,
    ticket: String,
    _d: tempfile::TempDir,
}

fn khoi_dong() -> Chay {
    let d = tempfile::tempdir().unwrap();
    let ui = d.path().join("ui");
    std::fs::create_dir_all(&ui).unwrap();
    std::fs::write(ui.join("index.html"), "<h1>huba</h1>").unwrap();
    let (listener, port): (TcpListener, u16) = bind_loopback().unwrap();
    let state = UiState::new(port, ui);
    let ticket = state.ticket().to_string();
    let state = Arc::new(state);
    std::thread::spawn(move || serve_loop(listener, state));
    Chay {
        port,
        ticket,
        _d: d,
    }
}

/// Gửi `raw`, đọc tới khi máy chủ đóng; trả (mã trạng thái, phần đầu, thân).
fn http(port: u16, raw: &str) -> (u16, String, String) {
    let mut c = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    c.write_all(raw.as_bytes()).unwrap();
    let mut buf = Vec::new();
    let _ = c.read_to_end(&mut buf);
    let s = String::from_utf8_lossy(&buf).to_string();
    let (head, body) = s.split_once("\r\n\r\n").unwrap_or((&s, ""));
    let code = head
        .split_whitespace()
        .nth(1)
        .and_then(|x| x.parse().ok())
        .unwrap_or(0);
    (code, head.to_string(), body.to_string())
}

fn get(port: u16, target: &str, extra: &str) -> (u16, String, String) {
    http(
        port,
        &format!("GET {target} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{extra}\r\n"),
    )
}

#[test]
fn the_whole_ticket_to_page_flow_works_over_a_real_socket() {
    let c = khoi_dong();
    // không cookie ⟹ 401
    assert_eq!(get(c.port, "/", "").0, 401);
    // đổi vé ⟹ 303 + cookie
    let (code, head, _) = get(c.port, &format!("/?t={}", c.ticket), "");
    assert_eq!(code, 303, "{head}");
    let ck = head
        .lines()
        .find_map(|l| l.strip_prefix("Set-Cookie: "))
        .expect("thiếu Set-Cookie");
    let cookie = ck.split(';').next().unwrap().to_string();
    // có cookie ⟹ 200 và đúng nội dung
    let (code, head, body) = get(c.port, "/", &format!("Cookie: {cookie}\r\n"));
    assert_eq!(code, 200, "{head}");
    assert_eq!(body, "<h1>huba</h1>");
    assert!(head.contains("Content-Security-Policy"), "{head}");
    // dùng lại vé ⟹ 401
    assert_eq!(get(c.port, &format!("/?t={}", c.ticket), "").0, 401);
}

#[test]
fn a_foreign_host_over_the_wire_is_403() {
    let c = khoi_dong();
    let (code, _, _) = http(c.port, "GET / HTTP/1.1\r\nHost: evil.com\r\n\r\n");
    assert_eq!(code, 403);
}

/// 🔴 Khai thân 4 GB: phải ra 413 NGAY, không cấp phát và không treo.
#[test]
fn a_four_gigabyte_content_length_is_413_fast() {
    let c = khoi_dong();
    let t0 = Instant::now();
    let (code, _, _) = http(
        c.port,
        &format!(
            "POST /api/x HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: 4294967296\r\n\r\n",
            c.port
        ),
    );
    assert_eq!(code, 413);
    assert!(
        t0.elapsed() < Duration::from_secs(3),
        "treo {:?}",
        t0.elapsed()
    );
}

/// Một kết nối mở rồi im lặng KHÔNG được chặn người khác (bản trang cũ xử lý tuần tự: một ổ cắm treo là cả trang treo).
#[test]
fn an_idle_connection_does_not_block_a_real_request() {
    let c = khoi_dong();
    let _treo = TcpStream::connect((Ipv4Addr::LOCALHOST, c.port)).unwrap();
    let t0 = Instant::now();
    let (code, _, _) = get(c.port, "/", "");
    assert_eq!(code, 401);
    assert!(
        t0.elapsed() < Duration::from_secs(2),
        "yêu cầu thật phải chờ kết nối treo: {:?}",
        t0.elapsed()
    );
}

#[test]
fn garbage_does_not_kill_the_server() {
    let c = khoi_dong();
    let _ = http(c.port, "\u{1}\u{2}\u{3} không phải HTTP\r\n\r\n");
    // đối chứng ngược: sau rác, yêu cầu hợp lệ vẫn được phục vụ
    assert_eq!(get(c.port, "/", "").0, 401);
}
