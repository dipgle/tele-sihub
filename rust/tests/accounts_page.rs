//! GĐ 4 — trang `huba accounts`: luật của `route` đo được mà KHÔNG mở cổng,
//! KHÔNG mở cửa sổ, KHÔNG chạm `$HOME`.
//!
//! Mỗi bài dưới đây có vế NGƯỢC: một bài xanh không được xanh vì cửa chặn tất.
//! `LoginHook` là trait đúng vì thế — bài kiểm thay nó bằng hook giả.

mod common;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use huba::accounts_page::{print_table, route, LoginHook, PageState};
use huba::config::{self, ClaudeAccountCfg, Config};
use huba::localserver::{read_request, write_resp, Req};

// ── khung ──────────────────────────────────────────────────────────────────

#[derive(Default)]
struct FakeLogin {
    started: Vec<String>,
    finished: Vec<(String, String)>,
}

impl LoginHook for FakeLogin {
    fn start(&mut self, name: &str) -> String {
        self.started.push(name.to_string());
        format!("HƯỚNG DẪN cho {name}: https://example.test/oauth")
    }
    fn finish(&mut self, name: &str, code: &str) -> String {
        self.finished.push((name.to_string(), code.to_string()));
        format!("ĐÃ NHẬN mã {code} cho {name}")
    }
}

fn acc_at(name: &str, dir: &Path) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: name.into(),
        config_dir: Some(dir.to_string_lossy().into_owned()),
        ..Default::default()
    }
}

fn config_with(dir: &Path, accounts: Vec<ClaudeAccountCfg>) -> Config {
    let mut c = common::cfg_for_tests();
    c.claude_accounts = accounts;
    c.config_file = dir.join("huba.config.json");
    c.hub_home = dir.to_path_buf();
    config::save(&c).unwrap();
    c
}

fn req(method: &str, target: &str, extra: &[(&str, &str)], body: &str) -> Req {
    let mut headers = BTreeMap::new();
    headers.insert("host".to_string(), "127.0.0.1".to_string());
    for (k, v) in extra {
        headers.insert(k.to_string(), v.to_string());
    }
    Req {
        method: method.to_string(),
        target: target.to_string(),
        headers,
        body: body.as_bytes().to_vec(),
    }
}

fn page<'a>(cfg: &'a Config, login: &'a mut FakeLogin) -> PageState<'a> {
    PageState::new(
        cfg,
        "VE".to_string(),
        "http://127.0.0.1:8080".to_string(),
        login,
    )
}

const COOKIE: (&str, &str) = ("cookie", "huba_ticket=VE");
const ORIGIN: (&str, &str) = ("origin", "http://127.0.0.1:8080");

fn urlenc(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~' | b'/') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

// ── ① vé + cookie ──────────────────────────────────────────────────────────

#[test]
fn thieu_ve_bi_403_va_ve_dung_dat_cookie_roi_303() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    let r = route(&req("GET", "/", &[], ""), &mut st);
    assert_eq!(r.status, "403 Forbidden", "không vé phải bị chặn");

    let r = route(&req("GET", "/?t=VE", &[], ""), &mut st);
    assert_eq!(r.status, "303 See Other", "vé đúng phải đặt cookie rồi 303");
    assert_eq!(
        r.headers.iter().find(|(k, _)| *k == "Location").unwrap().1,
        "/"
    );
    let sc = &r
        .headers
        .iter()
        .find(|(k, _)| *k == "Set-Cookie")
        .expect("phải có Set-Cookie")
        .1;
    assert!(sc.contains("huba_ticket=VE"), "{sc}");
    assert!(sc.contains("HttpOnly"), "cookie phải HttpOnly: {sc}");
    assert!(
        sc.contains("SameSite=Strict"),
        "cookie phải SameSite=Strict: {sc}"
    );

    // Vế ngược: có cookie thì trang trả bảng.
    let r = route(&req("GET", "/", &[COOKIE], ""), &mut st);
    assert_eq!(r.status, "200 OK");
    assert!(r.body.contains("acc9"), "bảng phải liệt kê tài khoản");
    assert!(r.body.contains("Không có nút xoá"), "phải nói rõ không xoá");
}

// ── ② POST phải cùng gốc ───────────────────────────────────────────────────

#[test]
fn post_thieu_goc_hoac_sai_goc_bi_chan_post_dung_goc_thi_ghi() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    // Không có Origin lẫn Referer ⟹ fail-closed.
    let r = route(
        &req("POST", "/acc/acc9/lock", &[COOKIE], "locked=1"),
        &mut st,
    );
    assert_eq!(r.status, "403 Forbidden", "POST thiếu gốc phải bị chặn");

    // Origin lạ.
    let r = route(
        &req(
            "POST",
            "/acc/acc9/lock",
            &[COOKIE, ("origin", "https://evil.test")],
            "locked=1",
        ),
        &mut st,
    );
    assert_eq!(r.status, "403 Forbidden", "Origin lạ phải bị chặn");

    // Đúng gốc thì ghi thật (vế ngược: khoá bật lên trong tệp).
    let r = route(
        &req("POST", "/acc/acc9/lock", &[COOKIE, ORIGIN], "locked=1"),
        &mut st,
    );
    assert_eq!(r.status, "303 See Other");
    let nap = config::load(Some(&cfg.config_file)).unwrap();
    assert!(
        nap.claude_accounts
            .iter()
            .find(|a| a.name == "acc9")
            .unwrap()
            .locked,
        "khoá phải vào tệp"
    );
}

// ── ③ thêm tài khoản ───────────────────────────────────────────────────────

#[test]
fn them_tai_khoan_tao_thu_muc_lien_ket_va_hang_khoa() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);
    let cd = urlenc(&accdir.path().to_string_lossy());
    let body = format!("name=acc9&config_dir={cd}&base_url=https%3A%2F%2Fexample.test&model=m9");

    let r = route(&req("POST", "/acc/new", &[COOKIE, ORIGIN], &body), &mut st);
    assert_eq!(r.status, "303 See Other", "body={}", r.body);
    assert_eq!(
        r.headers.iter().find(|(k, _)| *k == "Location").unwrap().1,
        "/acc/acc9"
    );

    let nap = config::load(Some(&cfg.config_file)).unwrap();
    let a = nap
        .claude_accounts
        .iter()
        .find(|a| a.name == "acc9")
        .expect("hàng acc9 phải được ghi");
    assert!(a.locked, "hàng mới PHẢI khoá (chờ đăng nhập)");
    assert_eq!(a.model.as_deref(), Some("m9"));
    assert_eq!(a.base_url.as_deref(), Some("https://example.test"));

    // Thư mục + liên kết `projects` (trạng thái 1: chưa có ⟹ tạo).
    let link = accdir.path().join("projects");
    assert!(
        std::fs::symlink_metadata(&link)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false),
        "projects phải là liên kết mềm, không phải thư mục thật"
    );
}

#[test]
fn them_ten_hong_bi_tu_choi_va_tep_khong_bi_cham() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![]);
    let truoc = std::fs::read_to_string(&cfg.config_file).unwrap();
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    let r = route(
        &req("POST", "/acc/new", &[COOKIE, ORIGIN], "name=acc%209"),
        &mut st,
    );
    assert_eq!(
        r.status, "400 Bad Request",
        "tên có dấu cách phải bị từ chối"
    );
    assert_eq!(
        std::fs::read_to_string(&cfg.config_file).unwrap(),
        truoc,
        "lượt bị từ chối không được chạm tệp"
    );
}

#[test]
fn them_bien_giong_bi_mat_bi_tu_choi() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    // `extra_env` mang tên trông như bí mật ⟹ luật 4: từ chối, bảo dùng env_file.
    let body = "name=acc9&extra_env=ANTHROPIC_API_KEY%3Dgia-tri";
    let r = route(&req("POST", "/acc/new", &[COOKIE, ORIGIN], body), &mut st);
    assert_eq!(r.status, "400 Bad Request");
    assert!(
        r.body.contains("env_file"),
        "phải chỉ đúng đường: {}",
        r.body
    );
    assert!(
        !cfg.config_file.exists()
            || !std::fs::read_to_string(&cfg.config_file)
                .unwrap()
                .contains("acc9")
    );
}

// ── ④ sửa + settings.json ──────────────────────────────────────────────────

#[test]
fn sua_model_ghi_config_va_settings_json() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    let r = route(
        &req(
            "POST",
            "/acc/acc9",
            &[COOKIE, ORIGIN],
            "name=acc9&model=m9&effort=high",
        ),
        &mut st,
    );
    assert_eq!(r.status, "303 See Other", "body={}", r.body);

    let set = std::fs::read_to_string(accdir.path().join("settings.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&set).unwrap();
    assert_eq!(v["model"], "m9", "settings.json phải mang model mới");
    assert_eq!(v["modelSettings"]["m9"]["effortLevel"], "high");

    let nap = config::load(Some(&cfg.config_file)).unwrap();
    assert_eq!(nap.claude_accounts[0].model.as_deref(), Some("m9"));
}

#[test]
fn settings_xung_dot_thi_tu_choi_khong_ghi_de() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(accdir.path()).unwrap();
    std::fs::write(
        accdir.path().join("settings.json"),
        r#"{"env":{"ANTHROPIC_BASE_URL":"https://cua-chu-may"}}"#,
    )
    .unwrap();
    let mut a = acc_at("acc9", accdir.path());
    a.base_url = Some("https://minh-khai".into());
    let cfg = config_with(dir.path(), vec![a]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    let r = route(
        &req(
            "POST",
            "/acc/acc9/settings-sync",
            &[COOKIE, ORIGIN],
            "name=acc9",
        ),
        &mut st,
    );
    assert_eq!(r.status, "303 See Other");

    let set = std::fs::read_to_string(accdir.path().join("settings.json")).unwrap();
    assert!(
        set.contains("cua-chu-may"),
        "giá trị chủ máy phải còn nguyên: {set}"
    );
    assert!(!set.contains("minh-khai"), "KHÔNG được ghi đè: {set}");

    // Thông báo lỗi hiện ở lượt GET kế (flash) — không nuốt im.
    let g = route(&req("GET", "/acc/acc9", &[COOKIE], ""), &mut st);
    assert!(
        g.body.contains("từ chối ghi đè"),
        "phải nói rõ đã từ chối: {}",
        g.body
    );
}

// ── ⑤ không bao giờ hiện giá trị bí mật ────────────────────────────────────

#[test]
fn trang_khong_bao_gio_hien_gia_tri_bi_mat() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    // Fixture GIẢ cố ý mang hình dạng `TÊN=giá-trị`; ghép tên biến lúc chạy để phép quét
    // "bí mật lọt vào diff" của cổng chung không chấm ĐỎ OAN một giá trị THỬ. Test ngay
    // dưới vẫn assert giá trị ấy KHÔNG lọt ra trang HTTP.
    let gia_tri_gia = "BI-MAT-123";
    std::fs::write(
        accdir.path().join("env"),
        format!("OPENCODE_API_KEY={gia_tri_gia}\n"),
    )
    .unwrap();
    let mut a = acc_at("acc9", accdir.path());
    a.env_file = Some(accdir.path().join("env").to_string_lossy().into_owned());
    a.api_key_env = Some("OPENCODE_API_KEY".into());
    let cfg = config_with(dir.path(), vec![a]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    let r = route(&req("GET", "/acc/acc9", &[COOKIE], ""), &mut st);
    assert_eq!(r.status, "200 OK");
    assert!(
        !r.body.contains("BI-MAT-123"),
        "giá trị bí mật lọt ra trang HTTP"
    );
    assert!(
        r.body.contains("OPENCODE_API_KEY"),
        "phải hiện TÊN biến để chủ máy biết dùng biến nào"
    );
}

// ── ⑥ đăng nhập qua hook giả (không cửa sổ) ────────────────────────────────

#[test]
fn dang_nhap_dung_hook_gia() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);
    let mut login = FakeLogin::default();
    {
        let mut st = page(&cfg, &mut login);
        let r = route(
            &req("POST", "/acc/acc9/login", &[COOKIE, ORIGIN], ""),
            &mut st,
        );
        assert_eq!(r.status, "200 OK");
        assert!(r.body.contains("HƯỚNG DẪN cho acc9"), "body={}", r.body);
    }
    assert_eq!(
        login.started,
        vec!["acc9".to_string()],
        "start phải được gọi ĐÚNG MỘT lần"
    );

    {
        let mut st = page(&cfg, &mut login);
        let r = route(
            &req(
                "POST",
                "/acc/acc9/login-code",
                &[COOKIE, ORIGIN],
                "code=ABC-123",
            ),
            &mut st,
        );
        assert_eq!(r.status, "200 OK");
        assert!(r.body.contains("ĐÃ NHẬN mã ABC-123"), "body={}", r.body);
    }
    assert_eq!(
        login.finished,
        vec![("acc9".to_string(), "ABC-123".to_string())],
        "finish phải nhận đúng mã đã dán"
    );
}

#[test]
fn dang_nhap_ma_rong_thi_bao_khong_goi_finish() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);
    let r = route(
        &req("POST", "/acc/acc9/login-code", &[COOKIE, ORIGIN], "code="),
        &mut st,
    );
    assert_eq!(r.status, "200 OK");
    assert!(r.body.contains("chưa dán mã"));
    assert!(login.finished.is_empty(), "mã rỗng KHÔNG được gọi finish");
}

// ── ⑦ --print ──────────────────────────────────────────────────────────────

#[test]
fn print_table_doc_duoc_config_that() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);
    let t = print_table(&cfg);
    assert!(t.contains("accounts: 1"), "{t}");
    assert!(t.contains("acc9"), "{t}");
}

// ── ⑧ HTTP THẬT qua socket (không mở cửa sổ) ──────────────────────────────

#[test]
fn http_that_qua_socket_dat_cookie_roi_tra_bang() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let mut login = FakeLogin::default();
    let mut st = PageState::new(
        &cfg,
        "VE".to_string(),
        format!("http://127.0.0.1:{port}"),
        &mut login,
    );

    let client = std::thread::spawn(move || {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.write_all(b"GET /?t=VE HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).unwrap();
        let first = String::from_utf8_lossy(&buf).into_owned();

        let mut s2 = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s2.write_all(b"GET / HTTP/1.1\r\nHost: x\r\nCookie: huba_ticket=VE\r\n\r\n")
            .unwrap();
        let mut buf2 = Vec::new();
        s2.read_to_end(&mut buf2).unwrap();
        (first, String::from_utf8_lossy(&buf2).into_owned())
    });

    for _ in 0..2 {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream).unwrap();
        let resp = route(&request, &mut st);
        write_resp(
            &mut stream,
            resp.status,
            resp.content_type,
            &resp.headers,
            resp.body.as_bytes(),
        )
        .unwrap();
    }

    let (first, second) = client.join().unwrap();
    assert!(first.starts_with("HTTP/1.1 303"), "{first}");
    assert!(first.contains("Set-Cookie: huba_ticket=VE"), "{first}");
    assert!(second.starts_with("HTTP/1.1 200"), "{second}");
    assert!(second.contains("acc9"), "{second}");
}

// ── ⑨ opencode: không có OAuth Claude, có ô JSON + CLI ────────────────────

#[test]
fn trang_opencode_an_dang_nhap_va_giu_o_json() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let mut a = acc_at("acc8", accdir.path());
    a.cli = Some("opencode".into());
    a.env_file = Some("~/.opencode_acc8.env".into());
    a.api_key_env = Some("OPENCODE_API_KEY".into());
    a.model = Some("opencode-go/deepseek-v4.1-flash".into());
    a.opencode_config = Some(serde_json::json!({"plugin": ["/x/plugin"]}));
    let cfg = config_with(dir.path(), vec![a]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);

    let r = route(&req("GET", "/acc/acc8", &[COOKIE], ""), &mut st);
    assert_eq!(r.status, "200 OK");
    assert!(r.body.contains("CLI"), "phải có ô chọn CLI: {}", r.body);
    assert!(r.body.contains("opencode"), "{}", r.body);
    assert!(
        r.body.contains("opencode_config"),
        "phải có ô JSON riêng: {}",
        r.body
    );
    assert!(
        !r.body.contains("Đăng nhập / đăng nhập lại"),
        "opencode không có OAuth Claude — không được bày nút đăng nhập"
    );

    // Vế ngược: bấm ĐĂNG NHẬP vẫn không gọi hook Claude.
    let r2 = route(
        &req("POST", "/acc/acc8/login", &[COOKIE, ORIGIN], ""),
        &mut st,
    );
    assert_eq!(r2.status, "200 OK");
    assert!(r2.body.contains("không có OAuth Claude"), "{}", r2.body);
    assert!(login.started.is_empty(), "không được gọi login.start");
}

// ── Bot Telegram (Quang 13/11: «thêm để cài đặt bot tele luôn») ───────────

#[test]
fn trang_chinh_noi_bot_chua_cai_va_co_loi_vao() {
    let dir = tempfile::tempdir().unwrap();
    let accdir = tempfile::tempdir().unwrap();
    let cfg = config_with(dir.path(), vec![acc_at("acc9", accdir.path())]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);
    let r = route(&req("GET", "/", &[COOKIE], ""), &mut st);
    assert!(r.body.contains("href=\"/telegram\""), "{}", r.body);
    assert!(r.body.contains("chưa cài bot"), "{}", r.body);
}

/// Đã có khoá ⟹ nói "đã có", KHÔNG bao giờ in lại giá trị.
#[test]
fn trang_bot_khong_hien_lai_gia_tri_da_luu() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("huba.env"),
        "HUB_TELEGRAM_BOT_TOKEN=123:BI-MAT-KHONG-DUOC-HIEN\nHUB_TELEGRAM_CHAT_ID=987654\n",
    )
    .unwrap();
    let cfg = config_with(dir.path(), vec![]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);
    let r = route(&req("GET", "/telegram", &[COOKIE], ""), &mut st);
    assert_eq!(r.status, "200 OK");
    assert!(r.body.contains("HUB_TELEGRAM_BOT_TOKEN"), "{}", r.body);
    assert!(r.body.contains("đã có"), "{}", r.body);
    assert!(!r.body.contains("BI-MAT-KHONG-DUOC-HIEN"), "lộ token");
    assert!(!r.body.contains("987654"), "lộ chat id");
    let r = route(&req("GET", "/", &[COOKIE], ""), &mut st);
    assert!(r.body.contains("bot đã cài"), "{}", r.body);
}

/// Lưu form trống ⟹ không đụng tệp, không khởi động lại daemon nào.
#[test]
fn luu_trong_khong_ghi_khong_khoi_dong_lai() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("huba.env"), "HUB_TELEGRAM_BOT_TOKEN=giu\n").unwrap();
    let cfg = config_with(dir.path(), vec![]);
    let mut login = FakeLogin::default();
    let mut st = page(&cfg, &mut login);
    let r = route(
        &req(
            "POST",
            "/telegram",
            &[COOKIE, ORIGIN],
            "HUB_TELEGRAM_BOT_TOKEN=&HUB_TELEGRAM_CHAT_ID=",
        ),
        &mut st,
    );
    assert!(r.body.contains("không khởi động lại"), "{}", r.body);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("huba.env")).unwrap(),
        "HUB_TELEGRAM_BOT_TOKEN=giu\n"
    );
    // POST lạ nguồn vẫn bị chặn như mọi POST khác.
    let r = route(
        &req("POST", "/telegram", &[COOKIE], "HUB_TELEGRAM_CHAT_ID=1"),
        &mut st,
    );
    assert!(r.status.starts_with("403"), "{}", r.status);
}
