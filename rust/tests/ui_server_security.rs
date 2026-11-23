//! Hàng rào của giao diện web cục bộ `huba ui` (W1 của docs/ui/ke-hoach-cong-viec.md).
//!
//! Cổng 127.0.0.1 KHÔNG phải hàng rào: mọi tiến trình cùng máy gọi được, và một trang web lạ trong trình duyệt của
//! chủ máy cũng có thể bắn yêu cầu vào đó (DNS-rebinding, CSRF). Giao diện này sẽ GÕ được vào phiên CLI nên hở là hở lệnh.
//! Mỗi hàng rào có một ca ĐỎ-khi-gỡ và một ca XANH-khi-đúng (đối chứng ngược): ca chỉ kiểm vế chặn thì cổng chặn tất cả
//! cũng qua.

use std::collections::BTreeMap;

use huba::localserver::Req;
use huba::uiserver::{handle, Resp, UiState, COOKIE, MAX_BODY};

const PORT: u16 = 45871;

fn state() -> (UiState, tempfile::TempDir) {
    let d = tempfile::tempdir().expect("tmp");
    let ui = d.path().join("ui");
    std::fs::create_dir_all(&ui).unwrap();
    std::fs::write(ui.join("index.html"), "<h1>huba</h1>").unwrap();
    std::fs::write(ui.join("app.js"), "console.log(1)").unwrap();
    std::fs::write(d.path().join("bi-mat.txt"), "KHONG-DUOC-LO").unwrap();
    (UiState::new(PORT, ui), d)
}

fn req(method: &str, target: &str, hs: &[(&str, &str)], body: &[u8]) -> Req {
    let mut headers = BTreeMap::new();
    for (k, v) in hs {
        headers.insert(k.to_string(), v.to_string());
    }
    Req {
        method: method.into(),
        target: target.into(),
        headers,
        body: body.to_vec(),
    }
}

fn host() -> String {
    format!("127.0.0.1:{PORT}")
}

fn origin() -> String {
    format!("http://127.0.0.1:{PORT}")
}

/// GET có Host đúng và cookie đúng.
fn get_auth(s: &UiState, target: &str) -> Resp {
    let ck = format!("{COOKIE}={}", s.session());
    handle(
        s,
        &req("GET", target, &[("host", &host()), ("cookie", &ck)], b""),
    )
}

fn header<'a>(r: &'a Resp, name: &str) -> Option<&'a str> {
    r.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

// ── W1.1 · không cookie ⟹ 401; có cookie ⟹ 200 ─────────────────────────────────────────────

#[test]
fn health_without_cookie_is_401() {
    let (s, _d) = state();
    let r = handle(&s, &req("GET", "/api/health", &[("host", &host())], b""));
    assert_eq!(r.status, 401, "không cookie mà vào được");
}

#[test]
fn health_with_the_right_cookie_is_200() {
    let (s, _d) = state();
    let r = get_auth(&s, "/api/health");
    assert_eq!(r.status, 200);
    assert!(String::from_utf8_lossy(&r.body).contains("\"ok\":true"));
}

#[test]
fn a_wrong_cookie_is_401() {
    let (s, _d) = state();
    let ck = format!("{COOKIE}=sai-roi");
    let r = handle(
        &s,
        &req(
            "GET",
            "/api/health",
            &[("host", &host()), ("cookie", &ck)],
            b"",
        ),
    );
    assert_eq!(r.status, 401);
}

#[test]
fn the_cookie_is_found_among_other_cookies() {
    let (s, _d) = state();
    let ck = format!("a=1; {COOKIE}={}; b=2", s.session());
    let r = handle(
        &s,
        &req(
            "GET",
            "/api/health",
            &[("host", &host()), ("cookie", &ck)],
            b"",
        ),
    );
    assert_eq!(r.status, 200);
}

// ── W1.2 · token ngẫu nhiên ────────────────────────────────────────────────────────────────

#[test]
fn tokens_are_random_and_long() {
    let (a, _da) = state();
    let (b, _db) = state();
    for t in [a.session(), a.ticket(), b.session(), b.ticket()] {
        assert_eq!(t.len(), 32, "token không đủ 128 bit hex: {t}");
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()), "{t}");
    }
    assert_ne!(a.session(), b.session(), "hai phiên cùng một token");
    assert_ne!(a.ticket(), b.ticket());
    assert_ne!(a.session(), a.ticket(), "vé và khoá phiên không được trùng");
}

// ── W1.3 · vé một lần → cookie ───────────────────────────────────────────────────────────────

#[test]
fn the_ticket_becomes_a_httponly_samesite_cookie_once() {
    let (s, _d) = state();
    let url = format!("/?t={}", s.ticket());
    let r = handle(&s, &req("GET", &url, &[("host", &host())], b""));
    assert_eq!(r.status, 303, "đổi vé phải chuyển hướng");
    assert_eq!(header(&r, "location"), Some("/"), "vé còn nằm trên URL");
    let ck = header(&r, "set-cookie").expect("thiếu Set-Cookie");
    assert!(ck.contains(&format!("{COOKIE}={}", s.session())), "{ck}");
    for can in ["HttpOnly", "SameSite=Strict", "Path=/"] {
        assert!(ck.contains(can), "cookie thiếu {can}: {ck}");
    }
    // Đối chứng ngược: dùng LẠI vé ⟹ 401, không cấp cookie nữa.
    let r2 = handle(&s, &req("GET", &url, &[("host", &host())], b""));
    assert_eq!(r2.status, 401, "vé dùng được hai lần");
    assert!(header(&r2, "set-cookie").is_none());
}

#[test]
fn a_wrong_ticket_gets_no_cookie() {
    let (s, _d) = state();
    let r = handle(&s, &req("GET", "/?t=khong-dung", &[("host", &host())], b""));
    assert_eq!(r.status, 401);
    assert!(header(&r, "set-cookie").is_none());
    // vé đúng vẫn dùng được sau một lần thử sai (thử sai không đốt vé)
    let ok = handle(
        &s,
        &req(
            "GET",
            &format!("/?t={}", s.ticket()),
            &[("host", &host())],
            b"",
        ),
    );
    assert_eq!(ok.status, 303);
}

// ── W1.4 · Host ──────────────────────────────────────────────────────────────────────────────

#[test]
fn a_foreign_host_is_refused_even_with_a_valid_cookie() {
    let (s, _d) = state();
    let ck = format!("{COOKIE}={}", s.session());
    for h in ["evil.com", "evil.com:45871", "127.0.0.1:1", "127.0.0.1"] {
        let r = handle(
            &s,
            &req("GET", "/api/health", &[("host", h), ("cookie", &ck)], b""),
        );
        assert_eq!(r.status, 403, "Host «{h}» lọt qua");
    }
    let r = handle(&s, &req("GET", "/api/health", &[("cookie", &ck)], b""));
    assert_eq!(r.status, 403, "thiếu Host mà lọt");
}

#[test]
fn both_loopback_names_with_the_right_port_are_accepted() {
    let (s, _d) = state();
    let ck = format!("{COOKIE}={}", s.session());
    for h in [format!("127.0.0.1:{PORT}"), format!("localhost:{PORT}")] {
        let r = handle(
            &s,
            &req("GET", "/api/health", &[("host", &h), ("cookie", &ck)], b""),
        );
        assert_eq!(r.status, 200, "Host hợp lệ «{h}» bị chặn");
    }
}

#[test]
fn the_ticket_exchange_also_checks_the_host() {
    let (s, _d) = state();
    let r = handle(
        &s,
        &req(
            "GET",
            &format!("/?t={}", s.ticket()),
            &[("host", "evil.com")],
            b"",
        ),
    );
    assert_eq!(r.status, 403);
    // và vé KHÔNG bị đốt bởi một yêu cầu bị chặn
    let ok = handle(
        &s,
        &req(
            "GET",
            &format!("/?t={}", s.ticket()),
            &[("host", &host())],
            b"",
        ),
    );
    assert_eq!(ok.status, 303);
}

// ── W1.5 · Origin khi gửi dữ liệu ──────────────────────────────────────────────────────────────

fn post(s: &UiState, origin_hdr: Option<&str>, body: &[u8]) -> Resp {
    let ck = format!("{COOKIE}={}", s.session());
    let h = host();
    let mut hs = vec![("host", h.as_str()), ("cookie", ck.as_str())];
    if let Some(o) = origin_hdr {
        hs.push(("origin", o));
    }
    handle(s, &req("POST", "/api/health", &hs, body))
}

#[test]
fn a_post_needs_the_right_origin() {
    let (s, _d) = state();
    assert_eq!(post(&s, Some("http://evil.com"), b"").status, 403);
    assert_eq!(post(&s, None, b"").status, 403, "POST không Origin mà lọt");
    assert_eq!(post(&s, Some("null"), b"").status, 403);
    // đối chứng ngược: Origin đúng ⟹ qua hàng rào (đường chưa có POST ⟹ 405, KHÔNG phải 403)
    assert_eq!(post(&s, Some(&origin()), b"").status, 405);
}

// ── W1.6 · giới hạn thân ────────────────────────────────────────────────────────────────────

#[test]
fn an_oversized_body_is_413_and_a_small_one_passes() {
    let (s, _d) = state();
    let big = vec![b'x'; MAX_BODY + 1];
    assert_eq!(post(&s, Some(&origin()), &big).status, 413);
    let ok = vec![b'x'; 1024];
    assert_eq!(post(&s, Some(&origin()), &ok).status, 405);
}

// ── W1.7 · tệp tĩnh, không thoát khỏi thư mục ui/ ─────────────────────────────────────────────

#[test]
fn static_files_are_served_with_the_right_type() {
    let (s, _d) = state();
    let r = get_auth(&s, "/");
    assert_eq!(r.status, 200);
    assert_eq!(String::from_utf8_lossy(&r.body), "<h1>huba</h1>");
    assert!(r.ctype.starts_with("text/html"), "{}", r.ctype);
    let j = get_auth(&s, "/app.js");
    assert_eq!(j.status, 200);
    assert!(j.ctype.contains("javascript"), "{}", j.ctype);
}

#[test]
fn static_files_need_the_cookie_too() {
    let (s, _d) = state();
    let r = handle(&s, &req("GET", "/app.js", &[("host", &host())], b""));
    assert_eq!(r.status, 401);
}

#[test]
fn path_traversal_never_reaches_a_file_outside_ui() {
    let (s, _d) = state();
    for t in [
        "/../bi-mat.txt",
        "/%2e%2e/bi-mat.txt",
        "/%2E%2E%2Fbi-mat.txt",
        "/a/../../bi-mat.txt",
        "/..%5Cbi-mat.txt",
        "/%00",
        "//etc/passwd",
    ] {
        let r = get_auth(&s, t);
        // Đúng 400 (lớp ĐẦU: từ chối đường dị dạng), không chấp nhận 404: lớp thứ hai (phân giải thật + so tiền tố)
        // cũng ra 404, nên nếu test cho phép 404 thì gỡ lớp đầu đi test vẫn xanh (đột biến sống sót 23/11).
        assert_eq!(r.status, 400, "«{t}» ⟹ {} (lớp đầu phải trả 400)", r.status);
        assert!(
            !String::from_utf8_lossy(&r.body).contains("KHONG-DUOC-LO"),
            "«{t}» LỘ tệp ngoài ui/"
        );
    }
    // đối chứng ngược: đường lành vẫn đọc được
    assert_eq!(get_auth(&s, "/app.js").status, 200);
}

#[cfg(unix)]
#[test]
fn a_symlink_inside_ui_cannot_escape() {
    let (s, d) = state();
    std::os::unix::fs::symlink(d.path().join("bi-mat.txt"), s.ui_dir.join("lien-ket.txt")).unwrap();
    let r = get_auth(&s, "/lien-ket.txt");
    assert_eq!(r.status, 404, "liên kết mềm dẫn ra ngoài ui/ mà đọc được");
    assert!(!String::from_utf8_lossy(&r.body).contains("KHONG-DUOC-LO"));
}

#[test]
fn a_missing_file_is_404_not_a_listing() {
    let (s, _d) = state();
    assert_eq!(get_auth(&s, "/khong-co.js").status, 404);
}

// ── tiêu đề an toàn trên MỌI phản hồi ────────────────────────────────────────────────────────

#[test]
fn every_response_carries_the_safety_headers() {
    let (s, _d) = state();
    let sach = get_auth(&s, "/");
    let tu_choi = handle(&s, &req("GET", "/api/health", &[("host", &host())], b""));
    for (ten, r) in [("200", &sach), ("401", &tu_choi)] {
        assert_eq!(
            header(r, "x-content-type-options"),
            Some("nosniff"),
            "{ten}"
        );
        assert_eq!(header(r, "cache-control"), Some("no-store"), "{ten}");
        assert_eq!(header(r, "referrer-policy"), Some("no-referrer"), "{ten}");
        let csp = header(r, "content-security-policy").unwrap_or("");
        assert!(csp.contains("default-src 'self'"), "{ten}: {csp}");
        assert!(csp.contains("frame-ancestors 'none'"), "{ten}: {csp}");
    }
}
