//! `huba accounts` — trang quản lý tài khoản Claude, chạy NGAY TRÊN MÁY cài huba.
//!
//! Quang 2026-11-09: *"cần có 1 giao diện quản lý để thêm sửa xóa acc, ví dụ như
//! vụ acc9 không có đầu vào cố định dẫn đến không biết lỗi do đâu, rồi muốn cấu
//! hình models cho acc thì làm thế nào"*. Trang này là câu trả lời cho phần
//! "thêm · sửa · khoá" và "cấu hình model"; Telegram giữ nguyên đường xem/khoá
//! đã có (`/accounts`).
//!
//! Nó dùng lại đúng khung của `setup.rs` (đã tách sang `localserver.rs`), vì hai
//! trang có CÙNG bốn ranh giới:
//! * **Chỉ 127.0.0.1** — trang không có việc gì ngoài máy.
//! * **Vé một lần** — cổng loopback thì tiến trình nào trên máy cũng gọi được,
//!   nên cái vé mới là hàng rào. Vé chỉ nằm trong RAM, log chỉ ghi `{port}`.
//! * **Không bao giờ hiện lại giá trị bí mật** — chỉ TÊN biến + "✓ đã đặt
//!   trong <env_file>". Đọc tệp 600 rồi bơm ngược ra HTTP là tự dựng đường rò.
//! * **Ghi rồi mới đổi tên** — `settings.json.tmp` → tệp, kèm `.bak` (việc của
//!   `accounts::apply_settings`).
//!
//! ## Ngoại lệ luật 12 (chép nguyên lý do vào đây)
//!
//! Luật 12 nói *"trang điện thoại là UI duy nhất"* và nó sinh ra để chặn một
//! bảng điều khiển vận hành THỨ HAI. Trang này cùng hình dạng với `setup.rs`:
//! trình soạn ĐĂNG KÝ TÀI KHOẢN mà chính kênh phụ thuộc vào — không lệnh phiên,
//! không `/ask`, không chi tiêu, không đọc ngược bí mật. Nó ghi đúng hai thứ:
//! `claude_accounts[]` trong `huba.config.json` và `settings.json` của một tài
//! khoản. Mọi thao tác nghiệp vụ khác vẫn đi qua buồng chat.
//!
//! ## `route` là hàm THUẦN đủ để test
//!
//! [`route`] nhận một [`Req`] đã đọc xong và một [`PageState`]; nó KHÔNG mở
//! cổng, KHÔNG mở cửa sổ. Việc đăng nhập đi qua trait [`LoginHook`], nên bài
//! kiểm thay bằng một hook giả — không cửa sổ Terminal nào mở ra lúc chạy test.

use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::json;

use crate::accounts;
use crate::config::{self, ClaudeAccountCfg, Config};
use crate::db::Db;
use crate::localserver::{
    bind_loopback, new_ticket, open_window, parse_form, percent_decode, read_request, write_resp,
    Req,
};

// ── Response ───────────────────────────────────────────────────────────────

/// Một response trọn vẹn. `route` dựng nó; `serve` ghi nó ra socket.
pub struct Resp {
    pub status: &'static str,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
    pub body: String,
}

impl Resp {
    fn html(body: String) -> Resp {
        Resp {
            status: "200 OK",
            content_type: "text/html; charset=utf-8",
            headers: Vec::new(),
            body,
        }
    }

    /// 303 + `Location` — mẫu POST/Redirect/GET: bấm Lưu xong thì thanh địa chỉ
    /// về một trang GET, nên F5 không gửi lại biểu mẫu.
    fn redirect(to: &str) -> Resp {
        Resp {
            status: "303 See Other",
            content_type: "text/plain; charset=utf-8",
            headers: vec![("Location", to.to_string())],
            body: String::new(),
        }
    }

    fn text(status: &'static str, body: impl Into<String>) -> Resp {
        Resp {
            status,
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: body.into(),
        }
    }

    fn forbidden(why: &str) -> Resp {
        Resp::html(page_shell(
            "Bị chặn",
            None,
            &format!(
                "<div class=\"err\">{}</div><p>Chỉ trang mở từ đúng đường dẫn terminal vừa in mới \
                 dùng được. Đóng tab này, chạy lại <code>huba accounts</code> nếu cần.</p>",
                esc(why)
            ),
        ))
        .with_status("403 Forbidden")
    }

    fn not_found(what: &str) -> Resp {
        Resp::html(page_shell(
            "Không thấy",
            None,
            &format!("<div class=\"err\">{}</div>", esc(what)),
        ))
        .with_status("404 Not Found")
    }

    fn with_status(mut self, status: &'static str) -> Resp {
        self.status = status;
        self
    }
}

// ── Đăng nhập: trait để test không mở cửa sổ ───────────────────────────────

/// Hai thao tác đăng nhập mà trang gọi. Bản thật uỷ quyền cho
/// [`crate::account_login`]; bản giả trong bài kiểm trả chuỗi cố định và KHÔNG
/// mở cửa sổ Terminal nào.
pub trait LoginHook {
    /// Mở cửa sổ đăng nhập, trả hướng dẫn (có thể chứa URL OAuth).
    fn start(&mut self, name: &str) -> String;
    /// Gõ mã chủ máy dán vào đúng cửa sổ đang chờ.
    fn finish(&mut self, name: &str, code: &str) -> String;
}

/// [`LoginHook`] thật — gọi đúng hai hàm mà route Telegram `/accounts <tên> -l`
/// đã dùng. Không có đường đăng nhập thứ hai.
struct RealLogin<'a> {
    cfg: &'a Config,
    db: &'a Db,
}

impl LoginHook for RealLogin<'_> {
    fn start(&mut self, name: &str) -> String {
        crate::account_login::start(self.cfg, self.db, name)
    }
    fn finish(&mut self, name: &str, code: &str) -> String {
        crate::account_login::finish(self.cfg, self.db, name, code)
    }
}

// ── Trạng thái trang (sống qua các request) ────────────────────────────────

/// Trạng thái sống suốt một lần chạy `huba accounts`.
///
/// `flash` sống Ở ĐÂY chứ không ở từng request: một lượt POST đặt nó rồi 303,
/// lượt GET kế mới đọc ra. Nếu nó nằm trong `Req` thì thông báo "đã lưu" sẽ
/// không bao giờ tới được trang đích.
pub struct PageState<'a> {
    pub cfg: &'a Config,
    pub ticket: String,
    /// Gốc same-origin để kiểm `Origin`/`Referer` của POST.
    pub origin: String,
    pub login: &'a mut dyn LoginHook,
    flash: Option<(String, bool)>,
}

impl<'a> PageState<'a> {
    pub fn new(
        cfg: &'a Config,
        ticket: String,
        origin: String,
        login: &'a mut dyn LoginHook,
    ) -> PageState<'a> {
        PageState {
            cfg,
            ticket,
            origin,
            login,
            flash: None,
        }
    }
}

/// `route` — toàn bộ luật của trang, không đụng socket. Test gọi thẳng hàm này.
pub fn route(req: &Req, st: &mut PageState) -> Resp {
    let (path, query) = split_target(&req.target);
    let form: BTreeMap<String, String> = if req.method == "POST" {
        parse_form(&String::from_utf8_lossy(&req.body))
    } else {
        BTreeMap::new()
    };

    let submitted_t = query.get("t").cloned().or_else(|| form.get("t").cloned());
    let cookie_t = cookie_value(&req.headers, "huba_ticket");
    let authed = cookie_t.as_deref() == Some(st.ticket.as_str())
        || submitted_t.as_deref() == Some(st.ticket.as_str());

    // Lượt ĐẦU TIÊN: `GET /?t=<vé>` hợp lệ, chưa có cookie ⇒ đặt cookie
    // `HttpOnly; SameSite=Strict` rồi 303 về `/`. Vé không nằm lại thanh địa chỉ.
    if req.method == "GET"
        && path == "/"
        && submitted_t.as_deref() == Some(st.ticket.as_str())
        && cookie_t.as_deref() != Some(st.ticket.as_str())
    {
        let mut r = Resp::redirect("/");
        r.headers.push((
            "Set-Cookie",
            format!(
                "huba_ticket={}; HttpOnly; SameSite=Strict; Path=/",
                st.ticket
            ),
        ));
        return r;
    }

    if !authed {
        return Resp::forbidden("vé không đúng");
    }

    // POST phải cùng gốc. Trình duyệt LUÔN gửi `Origin` cho form POST, nên
    // "không có Origin lẫn Referer" là một client lạ ⇒ từ chối (fail-closed).
    if req.method == "POST" && !origin_ok(req, &st.origin) {
        return Resp::forbidden("POST từ nguồn lạ (Origin/Referer không khớp)");
    }

    // Cấu hình phải đọc LẠI từ đĩa mỗi request: lượt trước vừa ghi thì bản
    // trong bộ nhớ đã cũ, và một trang luôn hiện trạng thái cũ là trang nói dối.
    let cfg = match config::load(Some(&st.cfg.config_file)) {
        Ok(c) => c,
        Err(e) => {
            crate::logging::error(
                "accounts_config_reload_failed",
                json!({ "err": crate::logging::err_chain(&e) }),
            );
            return Resp::html(page_shell(
                "Lỗi đọc cấu hình",
                None,
                &format!(
                    "<div class=\"err\">Không đọc lại được {}: {}</div>",
                    esc(&st.cfg.config_file.display().to_string()),
                    esc(&crate::logging::err_chain(&e))
                ),
            ))
            .with_status("500 Internal Server Error");
        }
    };

    let segs: Vec<&str> = path.trim_matches('/').split('/').collect();
    match (req.method.as_str(), segs.as_slice()) {
        ("GET", [""]) | ("GET", ["index.html"]) => index(&cfg, st),
        ("GET", ["new"]) => new_form(None, None),
        ("GET", ["telegram"]) => telegram_form(&cfg, None),
        ("POST", ["telegram"]) => telegram_save(&cfg, &form),
        ("GET", ["acc", name]) => {
            let name = percent_decode(name);
            detail(&cfg, &name, st, None)
        }
        ("POST", ["acc", "new"]) => add(&cfg, &form, st),
        ("POST", ["acc", name]) => {
            let name = percent_decode(name);
            update(&cfg, &name, &form, st)
        }
        ("POST", ["acc", name, "lock"]) => {
            let name = percent_decode(name);
            lock(&cfg, &name, &form, st)
        }
        ("POST", ["acc", name, "login"]) => {
            let name = percent_decode(name);
            login_start(&cfg, &name, st)
        }
        ("POST", ["acc", name, "login-code"]) => {
            let name = percent_decode(name);
            login_finish(&cfg, &name, &form, st)
        }
        ("POST", ["acc", name, "settings-sync"]) => {
            let name = percent_decode(name);
            settings_sync(&cfg, &name, st)
        }
        (m, _) => {
            Resp::not_found(&format!("không có đường `{m} {path}`")).with_status("404 Not Found")
        }
    }
}

// ── Bot Telegram ───────────────────────────────────────────────────────────
//
// Quang 13/11: *"thêm giao diện quản lý acc"* · *"thêm để cài đặt bot tele luôn"*.
// MỘT trang quản lý: tài khoản + bot. Lõi dùng lại của `setup.rs` (ô nhập, tự dò
// chat id, ghi `huba.env` 600 rồi đổi tên) — không chép luật bí mật ra chỗ thứ hai.

/// `✓ đã có token · chat` / `✗ chưa có token` — chỉ TÊN khoá, không bao giờ giá trị.
fn telegram_trang_thai(cfg: &Config) -> String {
    let have = crate::setup::existing_keys(&cfg.hub_home.join("huba.env"));
    let co = |k: &str| have.iter().any(|h| h == k);
    match (co("HUB_TELEGRAM_BOT_TOKEN"), co("HUB_TELEGRAM_CHAT_ID")) {
        (true, true) => "<span class=\"ok\">✓ bot đã cài</span>".to_string(),
        (true, false) => "<span class=\"bad\">⚠ có token, thiếu chat id</span>".to_string(),
        _ => {
            "<span class=\"bad\">✗ chưa cài bot — huba chưa nghe được điện thoại</span>".to_string()
        }
    }
}

fn telegram_form(cfg: &Config, ket_qua: Option<String>) -> Resp {
    let have = crate::setup::existing_keys(&cfg.hub_home.join("huba.env"));
    let body = format!(
        "{}{}<form method=\"POST\" action=\"/telegram\">{}\
         <button>Lưu &amp; khởi động lại huba</button></form>\
         <p><a href=\"/\">← Các tài khoản</a></p>",
        ket_qua.unwrap_or_default(),
        crate::setup::BOT_RIENG_NOTE,
        crate::setup::telegram_inputs(&have)
    );
    Resp::html(page_shell("Bot Telegram của máy này", None, &body))
}

fn telegram_save(cfg: &Config, form: &BTreeMap<String, String>) -> Resp {
    let env_path = cfg.hub_home.join("huba.env");
    let mut form = form.clone();
    form.remove("t");
    // Form trống ⟹ không đụng tệp (save_env viết lại cả tệp, kể cả khi không đổi gì).
    if form.values().all(|v| v.trim().is_empty()) {
        return telegram_form(
            cfg,
            Some("<p>Không ô nào được điền — huba.env giữ nguyên, không khởi động lại.</p>".into()),
        );
    }
    let do_chat = crate::setup::auto_chat_id(&mut form);
    let written = match crate::setup::save_env(&env_path, &form) {
        Ok(w) => w,
        Err(e) => {
            crate::logging::error(
                "accounts_telegram_save_failed",
                json!({ "err": crate::logging::err_chain(&e) }),
            );
            return telegram_form(
                cfg,
                Some(format!(
                    "<div class=\"err\">Không ghi được huba.env: {}</div>",
                    esc(&crate::logging::err_chain(&e))
                )),
            );
        }
    };
    crate::logging::info("accounts_telegram_saved", json!({ "keys": written }));
    let khoi_dong = if written.is_empty() {
        "<p>Không ô nào được điền — huba.env giữ nguyên, không khởi động lại.</p>".to_string()
    } else {
        match khoi_dong_lai_hubd() {
            Ok(cau) => format!("<p>✅ {}</p>", esc(&cau)),
            Err(cau) => format!("<div class=\"err\">⚠ {}</div>", esc(&cau)),
        }
    };
    let ghi = if written.is_empty() {
        String::new()
    } else {
        format!(
            "<p>Đã ghi: {}</p>",
            written
                .iter()
                .map(|k| format!("<code>{}</code>", esc(k)))
                .collect::<Vec<_>>()
                .join(" · ")
        )
    };
    telegram_form(cfg, Some(format!("{ghi}{do_chat}{khoi_dong}")))
}

/// huba.env chỉ được đọc lúc hubd khởi động ⟹ lưu khoá xong phải khởi động lại,
/// nếu không "đã lưu" mà bot vẫn câm. Đo lại sau khi gọi, không tin mã thoát.
fn khoi_dong_lai_hubd() -> Result<String, String> {
    let chay = |prog: &str, args: &[&str]| {
        crate::exec::run(prog, args, crate::exec::RunOpts::default())
            .map_err(|e| format!("{prog}: {e}"))
    };
    let kq = if cfg!(target_os = "linux") {
        chay("systemctl", &["--user", "restart", "huba-hubd"])
            .and_then(|_| {
                std::thread::sleep(std::time::Duration::from_secs(2));
                chay("systemctl", &["--user", "is-active", "huba-hubd"])
            })
            .and_then(|r| {
                if r.stdout.trim() == "active" {
                    Ok("đã khởi động lại huba-hubd (systemd) — bot dùng khoá mới".to_string())
                } else {
                    Err(format!(
                        "huba-hubd không lên sau khi khởi động lại: {}",
                        r.stdout.trim()
                    ))
                }
            })
    } else if cfg!(target_os = "macos") {
        let uid = chay("id", &["-u"]).map(|r| r.stdout.trim().to_string())?;
        chay(
            "launchctl",
            &["kickstart", "-k", &format!("gui/{uid}/com.dipgle.hubd")],
        )
        .and_then(|r| {
            if r.code == Some(0) {
                Ok("đã khởi động lại com.dipgle.hubd (launchd) — bot dùng khoá mới".to_string())
            } else {
                Err(format!(
                    "launchctl kickstart thoát {:?}: {}",
                    r.code,
                    r.stderr.trim()
                ))
            }
        })
    } else {
        Err("máy này không tự khởi động lại được hubd — khởi động lại tay để nạp khoá".into())
    };
    match &kq {
        Ok(c) => crate::logging::info("accounts_hubd_restarted", json!({ "say": c })),
        Err(c) => crate::logging::warn("accounts_hubd_restart_failed", json!({ "err": c })),
    }
    kq.map_err(|e| format!("đã lưu khoá nhưng {e}"))
}

// ── Các handler ────────────────────────────────────────────────────────────

fn index(cfg: &Config, st: &mut PageState) -> Resp {
    let flash = st.flash.take();
    let mut rows = String::new();
    let accs = cfg.claude_accounts_or_ambient();
    for acc in &accs {
        let in_config = cfg.claude_accounts.iter().any(|a| a.name == acc.name);
        let (ok, total, hong) = check_summary(acc);
        let dir = match acc.config_dir.as_deref().filter(|d| !d.trim().is_empty()) {
            Some(d) => esc(d),
            None => "<i>mặc định (ambient)</i>".to_string(),
        };
        let khoa = if acc.locked { "🔒" } else { "" };
        let mut bad = String::new();
        for c in hong.iter().take(2) {
            bad.push_str(&format!("<li>{}</li>", esc(&c.line())));
        }
        if hong.len() > 2 {
            bad.push_str(&format!("<li>… +{} mục</li>", hong.len() - 2));
        }
        rows.push_str(&format!(
            "<tr><td><a href=\"/acc/{}\">{}</a> {khoa}</td><td><code>{dir}</code></td>\
             <td class=\"{cls}\">{ok}/{total}</td><td><ul class=\"tight\">{bad}</ul></td></tr>",
            url_encode(&acc.name),
            esc(&acc.name),
            cls = if hong.is_empty() { "ok" } else { "bad" },
        ));
        let _ = in_config;
    }
    let tg = telegram_trang_thai(cfg);
    let body = format!(
        "<p><a class=\"btn\" href=\"/new\">+ Thêm tài khoản</a> \
         <a class=\"btn\" href=\"/telegram\">🤖 Bot Telegram</a> {tg}</p>\
         <table><thead><tr><th>Tài khoản</th><th>Thư mục cấu hình</th><th>Chẩn đoán</th>\
         <th>Việc cần làm</th></tr></thead><tbody>{rows}</tbody></table>\
         <div class=\"note\"><b>Không có nút xoá — cố ý.</b> huba không bao giờ bỏ một hàng \
         khỏi cấu hình hay đụng vào <code>~/.claude-accN/</code>: “xoá tài khoản” nghĩa là \
         <b>khoá</b> nó. Thư mục và nhật ký phiên vẫn còn nguyên; phiên đang chạy vẫn hiện \
         trong danh sách.</div>"
    );
    Resp::html(page_shell(
        "Các tài khoản Claude trên máy này",
        flash,
        &body,
    ))
}

fn detail(cfg: &Config, name: &str, st: &mut PageState, err: Option<String>) -> Resp {
    let flash = st.flash.take();
    let accs = cfg.claude_accounts_or_ambient();
    let Some(acc) = accs.iter().find(|a| a.name == name) else {
        return Resp::not_found(&format!("không có tài khoản '{name}'"));
    };
    let in_config = cfg.claude_accounts.iter().any(|a| a.name == name);

    let mut checks = String::new();
    let (_, _, hong) = check_summary(acc);
    let p = accounts::probe(
        acc,
        if !acc.is_claude() {
            None
        } else {
            Some(crate::quota::read(
                &acc.name,
                Some(crate::api_key::thu_muc(acc).as_path()),
            ))
        },
    );
    for c in accounts::checks(acc, &p) {
        checks.push_str(&format!(
            "<li class=\"{}\">{} <code>{}</code>{}</li>",
            match c.state {
                accounts::CheckState::Ok => "ok",
                accounts::CheckState::Fail => "bad",
                accounts::CheckState::Unknown => "unk",
            },
            c.icon(),
            esc(&c.text),
            if c.path.is_empty() {
                String::new()
            } else {
                format!(" <small>{}</small>", esc(&c.path))
            }
        ));
    }

    let khoa = if acc.locked {
        "🔓 Mở khoá"
    } else {
        "🔒 Khoá"
    };
    let khoa_to = if acc.locked { "0" } else { "1" };
    let dong_khoa = if acc.locked {
        "<p class=\"bad\">Đang KHOÁ — huba không tự chọn, không tự mở phiên, không dò /usage trên tài khoản này. Phiên đang chạy vẫn hiện.</p>"
    } else {
        ""
    };

    let form_html = if in_config {
        format!(
            "<form class=\"grid\" method=\"POST\" action=\"/acc/{}\">\
             <input type=\"hidden\" name=\"t\" value=\"{}\"><h2>Sửa</h2>{}\
             <p><button>Lưu</button></p></form>",
            url_encode(name),
            esc(&st.ticket),
            edit_fields(acc),
        )
    } else {
        "<div class=\"note\">Đây là tài khoản <b>mặc định (ambient)</b>: nó không có hàng riêng \
         trong <code>huba.config.json</code>, nên trang chỉ xem được chẩn đoán. Muốn quản lý \
         nhiều tài khoản, hãy bấm <a href=\"/new\">+ Thêm tài khoản</a>.</div>"
            .to_string()
    };

    let actions = if in_config && acc.is_opencode() {
        format!(
            "<div class=\"row\">\
             <form method=\"POST\" action=\"/acc/{n}/lock\"><input type=\"hidden\" name=\"t\" value=\"{tk}\">\
             <input type=\"hidden\" name=\"locked\" value=\"{to}\"><button class=\"sec\">{khoa}</button></form></div>\
             <div class=\"note\">opencode dùng khoá trong <code>env_file</code>; không có OAuth Claude \
             và không dùng <code>settings.json</code>.</div>",
            n = url_encode(name),
            tk = esc(&st.ticket),
            to = khoa_to,
            khoa = khoa,
        )
    } else if in_config {
        format!(
            "<div class=\"row\">\
             <form method=\"POST\" action=\"/acc/{n}/lock\"><input type=\"hidden\" name=\"t\" value=\"{tk}\">\
             <input type=\"hidden\" name=\"locked\" value=\"{to}\"><button class=\"sec\">{khoa}</button></form>\
             <form method=\"POST\" action=\"/acc/{n}/login\"><input type=\"hidden\" name=\"t\" value=\"{tk}\">\
             <button class=\"sec\">Đăng nhập / đăng nhập lại</button></form>\
             <form method=\"POST\" action=\"/acc/{n}/settings-sync\"><input type=\"hidden\" name=\"t\" value=\"{tk}\">\
             <button class=\"sec\">Đồng bộ settings.json</button></form></div>",
            n = url_encode(name),
            tk = esc(&st.ticket),
            to = khoa_to,
            khoa = khoa,
        )
    } else {
        String::new()
    };

    let dir = if acc.is_opencode() {
        accounts::opencode_profile_dir(acc).display().to_string()
    } else if acc.is_codex() {
        accounts::codex_home(acc).display().to_string()
    } else {
        acc.config_dir
            .clone()
            .unwrap_or_else(|| "(mặc định)".into())
    };
    let body = format!(
        "{err}\
         <p><code>{ten}</code> · <code>{cli}</code> · <code>{dir}</code></p>{dong_khoa}{actions}\
         <h2>Chẩn đoán ({}/{})</h2><ul class=\"checks\">{checks}</ul>\
         <p class=\"muted\">Hạn mức và email: gõ <code>/accounts {ten}</code> trên Telegram \
         (trang này chỉ đọc đĩa, không gọi mạng).</p>{form_html}",
        hong.len(),
        accounts::checks(acc, &p).len(),
        ten = esc(name),
        cli = esc(acc.cli_name()),
        dir = esc(&dir),
        err = err
            .map(|e| format!("<div class=\"err\">{}</div>", esc(&e)))
            .unwrap_or_default(),
    );
    Resp::html(page_shell("Chi tiết tài khoản", flash, &body))
}

fn new_form(err: Option<String>, form: Option<&BTreeMap<String, String>>) -> Resp {
    let body = format!(
        "{err}<form class=\"grid\" method=\"POST\" action=\"/acc/new\">\
         <h2>Tài khoản mới</h2>{}\
         <p><button>Thêm</button> · hàng mới vào trạng thái <b>KHOÁ</b> cho tới khi đăng nhập xong</p>\
         </form>",
        edit_fields_form(form),
        err = err
            .map(|e| format!("<div class=\"err\">{}</div>", esc(&e)))
            .unwrap_or_default(),
    );
    Resp::html(page_shell("Thêm tài khoản", None, &body))
}

fn add(cfg: &Config, form: &BTreeMap<String, String>, st: &mut PageState) -> Resp {
    let name = form
        .get("name")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if let Err(e) = accounts::validate_row(&name) {
        return new_form(Some(e), Some(form)).with_status("400 Bad Request");
    }
    let mut row = match row_from_form(form, None) {
        Ok(r) => r,
        Err(e) => return new_form(Some(e), Some(form)).with_status("400 Bad Request"),
    };
    row.name = name.clone();
    if row.config_dir.as_deref().unwrap_or("").trim().is_empty() {
        row.config_dir = Some(derived_config_dir(&name));
    }
    if let Some(k) = row.extra_env.keys().find(|k| accounts::looks_secret(k)) {
        return new_form(
            Some(format!(
                "biến `{k}` trông như một BÍ MẬT — luật 4 cấm đưa giá trị bí mật vào cấu hình. \
                 Đặt khoá trong `env_file` (chmod 600) rồi khai `api_key_env` = TÊN biến ấy."
            )),
            Some(form),
        )
        .with_status("400 Bad Request");
    }
    // Thư mục + liên kết `projects` TRƯỚC khi ghi hàng: nếu bước này hỏng thì
    // config chưa bị chạm (đúng thứ tự và lý do của `them-tai-khoan.sh`).
    let notes = match accounts::ensure_dirs(&row) {
        Ok(n) => n,
        Err(e) => return new_form(Some(e), Some(form)).with_status("400 Bad Request"),
    };
    // 🔴 Hàng mới LUÔN khoá: hạng `Unknown` đứng TRƯỚC `Full` trong thứ tự chọn,
    // nên một tài khoản chưa đăng nhập mà đã "dùng được" sẽ bị chọn đúng lúc mọi
    // tài khoản khác kịch trần. Mở khoá là bước riêng, sau khi đăng nhập xác minh.
    row.locked = true;
    match accounts::upsert_account(cfg, row) {
        Ok(w) => {
            let head = match w {
                accounts::AccountWrite::Added => "đã THÊM (đang khoá, chờ đăng nhập)",
                accounts::AccountWrite::Updated => "đã CẬP NHẬT",
            };
            st.flash = Some((format!("{head}: {name}\n{}", notes.join("\n")), false));
            Resp::redirect(&format!("/acc/{}", url_encode(&name)))
        }
        Err(e) => new_form(Some(e), Some(form)).with_status("400 Bad Request"),
    }
}

fn update(cfg: &Config, name: &str, form: &BTreeMap<String, String>, st: &mut PageState) -> Resp {
    let accs = cfg.claude_accounts_or_ambient();
    let Some(existing) = accs.iter().find(|a| a.name == name) else {
        return Resp::not_found(&format!("không có tài khoản '{name}'"));
    };
    if !cfg.claude_accounts.iter().any(|a| a.name == name) {
        return detail(
            cfg,
            name,
            st,
            Some("tài khoản mặc định không sửa được từ đây".into()),
        )
        .with_status("400 Bad Request");
    }
    let mut row = match row_from_form(form, Some(existing)) {
        Ok(r) => r,
        Err(e) => return detail(cfg, name, st, Some(e)).with_status("400 Bad Request"),
    };
    row.name = name.to_string();
    // Tên không đổi được qua biểu mẫu (nó là `~/.claude-<tên>`); giữ `config_dir`
    // cũ khi ô để trống — đổi thư mục là chuyện của người sửa tay.
    if row.config_dir.as_deref().unwrap_or("").trim().is_empty() {
        row.config_dir = existing.config_dir.clone();
    }
    if let Some(k) = row.extra_env.keys().find(|k| accounts::looks_secret(k)) {
        return detail(
            cfg,
            name,
            st,
            Some(format!(
                "biến `{k}` trông như BÍ MẬT — dùng `env_file` + `api_key_env` (luật 4)"
            )),
        )
        .with_status("400 Bad Request");
    }
    if let Err(e) = accounts::validate_row(name) {
        return detail(cfg, name, st, Some(e)).with_status("400 Bad Request");
    }

    let msg = match accounts::upsert_account(cfg, row.clone()) {
        Ok(_) => {
            // Cấu hình là NGUỒN SỰ THẬT, nên ghi nó trước; `settings.json` theo sau.
            // Lỗi settings KHÔNG lùi thay đổi cấu hình (xem `accounts::apply_settings`).
            match accounts::apply_settings(&row) {
                Ok(a) if a.set.is_empty() => {
                    "đã lưu cấu hình (không có khoá nào cần ghi vào settings.json)".to_string()
                }
                Ok(a) => format!(
                    "đã lưu cấu hình + đồng bộ settings.json ({} khoá){}",
                    a.set.len(),
                    if a.unsupported.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "\n{} trường chỉ đi qua dòng lệnh: {}",
                            a.unsupported.len(),
                            a.unsupported.join("; ")
                        )
                    }
                ),
                Err(e) => format!("đã lưu CẤU HÌNH, nhưng settings.json CHƯA đồng bộ: {e}"),
            }
        }
        Err(e) => return detail(cfg, name, st, Some(e)).with_status("400 Bad Request"),
    };
    st.flash = Some((format!("{name}: {msg}"), false));
    Resp::redirect(&format!("/acc/{}", url_encode(name)))
}

fn lock(cfg: &Config, name: &str, form: &BTreeMap<String, String>, st: &mut PageState) -> Resp {
    let locked = matches!(
        form.get("locked").map(String::as_str),
        Some("1" | "true" | "on")
    );
    match accounts::set_locked(cfg, name, locked, "page") {
        Ok(msg) => st.flash = Some((msg, false)),
        Err(e) => st.flash = Some((format!("⚠ không đổi được khoá: {e}"), true)),
    }
    Resp::redirect(&format!("/acc/{}", url_encode(name)))
}

fn mo_khoa_opencode(cfg: &Config, name: &str) -> Option<Resp> {
    let acc = cfg
        .claude_accounts_or_ambient()
        .into_iter()
        .find(|a| a.name == name)?;
    if !acc.is_opencode() {
        return None;
    }
    Some(Resp::html(page_shell(
        "opencode",
        Some(("opencode dùng khoá trong env_file — không có OAuth Claude".to_string(), false)),
        &format!(
            "<p><code>{}</code> đọc khoá từ <code>{}</code> (biến <code>{}</code>).</p>             <p><a class=\"btn\" href=\"/acc/{}\">← về {}</a></p>",
            esc(name),
            esc(acc.env_file.as_deref().unwrap_or("(chưa khai env_file)")),
            esc(acc.api_key_env.as_deref().unwrap_or("(chưa khai api_key_env)")),
            url_encode(name),
            esc(name)
        ),
    )))
}

fn login_start(cfg: &Config, name: &str, st: &mut PageState) -> Resp {
    if let Some(r) = mo_khoa_opencode(cfg, name) {
        return r;
    }
    let msg = st.login.start(name);
    Resp::html(login_page(name, &msg, st.ticket.as_str(), None))
}

fn login_finish(
    cfg: &Config,
    name: &str,
    form: &BTreeMap<String, String>,
    st: &mut PageState,
) -> Resp {
    if let Some(r) = mo_khoa_opencode(cfg, name) {
        return r;
    }
    let code = form.get("code").cloned().unwrap_or_default();
    if code.trim().is_empty() {
        return Resp::html(login_page(
            name,
            "",
            st.ticket.as_str(),
            Some("chưa dán mã — dán mã OAuth mà trang đăng nhập của Claude hiện ra"),
        ));
    }
    let msg = st.login.finish(name, &code);
    Resp::html(page_shell(
        "Kết quả đăng nhập",
        Some((msg, false)),
        &format!(
            "<p><a class=\"btn\" href=\"/acc/{}\">← về {}</a></p>",
            url_encode(name),
            esc(name)
        ),
    ))
}

fn settings_sync(cfg: &Config, name: &str, st: &mut PageState) -> Resp {
    let accs = cfg.claude_accounts_or_ambient();
    let Some(acc) = accs.iter().find(|a| a.name == name) else {
        return Resp::not_found(&format!("không có tài khoản '{name}'"));
    };
    if !acc.is_claude() {
        st.flash = Some((
            format!(
                "{} không dùng settings.json của Claude — cấu hình nằm ở chỗ riêng của CLI ấy",
                acc.cli_name()
            ),
            true,
        ));
        return Resp::redirect(&format!("/acc/{}", url_encode(name)));
    }
    match accounts::apply_settings(acc) {
        Ok(a) if a.set.is_empty() => {
            st.flash = Some((format!("không có khoá nào cần ghi vào {}", a.path), false));
        }
        Ok(a) => {
            st.flash = Some((
                format!(
                    "đã ghi {} khoá vào {} · bản lưu: {}",
                    a.set.len(),
                    a.path,
                    a.backup.unwrap_or_else(|| "(chưa có tệp cũ)".into())
                ),
                false,
            ));
        }
        Err(e) => st.flash = Some((format!("⚠ settings.json chưa đồng bộ: {e}"), true)),
    }
    Resp::redirect(&format!("/acc/{}", url_encode(name)))
}

// ── Đọc dữ liệu ────────────────────────────────────────────────────────────

/// `(số mục đạt, tổng, danh sách mục KHÔNG đạt)` cho một tài khoản — dùng
/// chung index và detail để hai chỗ không nói hai con số khác nhau.
fn check_summary(acc: &ClaudeAccountCfg) -> (usize, usize, Vec<accounts::Check>) {
    let p = accounts::probe(
        acc,
        if !acc.is_claude() {
            None
        } else {
            Some(crate::quota::read(
                &acc.name,
                Some(crate::api_key::thu_muc(acc).as_path()),
            ))
        },
    );
    let c = accounts::checks(acc, &p);
    let hong: Vec<accounts::Check> = accounts::problems(&c).into_iter().cloned().collect();
    (c.len() - hong.len(), c.len(), hong)
}

/// Biểu mẫu sửa, dựng từ một hàng đang có. KHÔNG bao giờ đọc/ghi giá trị bí mật.
fn edit_fields(acc: &ClaudeAccountCfg) -> String {
    let env_lines = acc
        .extra_env
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    edit_fields_values(
        &acc.name,
        acc.cli.as_deref().unwrap_or("claude"),
        acc.config_dir.as_deref().unwrap_or(""),
        acc.base_url.as_deref().unwrap_or(""),
        acc.env_file.as_deref().unwrap_or(""),
        acc.api_key_env.as_deref().unwrap_or(""),
        acc.api_key_as.as_deref().unwrap_or(""),
        acc.model.as_deref().unwrap_or(""),
        acc.effort.as_deref().unwrap_or(""),
        &env_lines,
        &acc.extra_args.join("\n"),
        &acc.opencode_config
            .as_ref()
            .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
            .unwrap_or_default(),
        acc.launch.as_deref(),
    )
}

/// Biểu mẫu THÊM — giữ lại thứ người dùng vừa gõ khi có lỗi (không bắt nhập lại).
fn edit_fields_form(form: Option<&BTreeMap<String, String>>) -> String {
    let g = |k: &str| form.and_then(|f| f.get(k)).cloned().unwrap_or_default();
    edit_fields_values(
        &g("name"),
        form.and_then(|f| f.get("cli"))
            .map(String::as_str)
            .unwrap_or("claude"),
        &g("config_dir"),
        &g("base_url"),
        &g("env_file"),
        &g("api_key_env"),
        &g("api_key_as"),
        &g("model"),
        &g("effort"),
        &g("extra_env"),
        &g("extra_args"),
        &g("opencode_config"),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn edit_fields_values(
    name: &str,
    cli: &str,
    config_dir: &str,
    base_url: &str,
    env_file: &str,
    api_key_env: &str,
    api_key_as: &str,
    model: &str,
    effort: &str,
    extra_env: &str,
    extra_args: &str,
    opencode_config: &str,
    launch: Option<&str>,
) -> String {
    let name_ro = if name.is_empty() {
        text_input(
            "name",
            "Tên tài khoản",
            "",
            "chữ, số, `_` và `-`; thành `~/.claude-<tên>` (hoặc `~/.opencode-<tên>` với opencode)",
        )
    } else {
        // Sửa thì tên cố định (nó là thư mục); hiện để đọc, không cho gõ.
        format!(
            "<label>Tên tài khoản</label><small>tên không đổi được — nó là <code>~/.claude-{}</code> / <code>~/.opencode-{}</code></small>\
             <input type=\"text\" value=\"{}\" disabled>",
            esc(name),
            esc(name),
            esc(name)
        )
    };
    let oc = cli.eq_ignore_ascii_case("opencode");
    let cx = cli.eq_ignore_ascii_case("codex");
    let cli_select = format!(
        "<label>CLI</label><small>opencode: phân vùng riêng <code>~/.opencode-&lt;tên&gt;/{{config,data,cache,state}}</code> — nhiều acc, nhiều API/model không trộn nhau · codex: <code>CODEX_HOME</code> = thư mục cấu hình (mặc định <code>~/.codex-&lt;tên&gt;</code>), đăng nhập ChatGPT, chưa mở phiên</small>\
         <select name=\"cli\">\
         <option value=\"claude\"{c}>claude</option>\
         <option value=\"opencode\"{o}>opencode</option>\
         <option value=\"codex\"{x}>codex</option>\
         </select>",
        c = if oc || cx { "" } else { " selected" },
        o = if oc { " selected" } else { "" },
        x = if cx { " selected" } else { "" },
    );
    let launch_note = match launch {
        Some(l) => format!(
            "<div class=\"note\">⚠ Hàng này có <code>launch</code> viết tay đang ĐÈ trường cấu trúc: \
             <code>{}</code>. Sửa các ô dưới đây sẽ ghi vào cấu hình nhưng CHƯA đổi dòng lệnh cho tới \
             khi bỏ <code>launch</code> (việc di trú, GĐ 5).</div>",
            esc(l)
        ),
        None => String::new(),
    };
    format!(
        "{name_ro}{launch_note}{cli_select}\
         {config_dir}\
         {base_url}\
         {env_file}\
         {api_key_env}\
         {api_key_as}\
         {model}\
         {effort}\
         <label>Biến môi trường thêm (KEY=VALUE, mỗi dòng một biến)</label>\
         <small>KHÔNG đặt bí mật ở đây — bí mật thuộc <code>env_file</code>. Dòng chứa tên kết thúc \
         <code>_KEY</code>/<code>_TOKEN</code>/<code>_SECRET</code>/<code>_PASSWORD</code> bị từ chối.</small>\
         <textarea name=\"extra_env\" rows=\"3\" spellcheck=\"false\">{extra_env}</textarea>\
         <label>Cấu hình opencode riêng (JSON object, để trống nếu không cần)</label>\
         <small>Merge vào <code>OPENCODE_CONFIG_CONTENT</code> — provider/baseURL/agent riêng. KHÔNG chứa bí mật; \
         bộ chặn <code>git push</code>/<code>sudo</code>/<code>rm</code>/WebFetch… của huba luôn được áp sau.</small>\
         <textarea name=\"opencode_config\" rows=\"6\" spellcheck=\"false\">{opencode_config}</textarea>\
         <label>Đối số thêm cho CLI (mỗi dòng một đối số)</label>\
         <small>ví dụ <code>--debug</code>. Không đặt <code>--model</code> ở đây — có ô riêng.</small>\
         <textarea name=\"extra_args\" rows=\"2\" spellcheck=\"false\">{extra_args}</textarea>",
        config_dir = text_input(
            "config_dir",
            "Thư mục cấu hình (Claude)",
            config_dir,
            "để trống khi THÊM = `~/.claude-<tên>`; opencode dùng phân vùng riêng, không dùng ô này",
        ),
        base_url = text_input("base_url", "Base URL (ANTHROPIC_BASE_URL)", base_url, "để trống = API của Claude; opencode khai provider trong ô JSON bên dưới"),
        env_file = text_input("env_file", "Tệp env (chmod 600)", env_file, "ĐƯỜNG DẪN tới tệp giữ khoá — không phải giá trị"),
        api_key_env = text_input("api_key_env", "Tên biến giữ khoá", api_key_env, "TÊN biến trong tệp env, ví dụ OPENCODE_API_KEY / OPENROUTER_API_KEY — không phải giá trị"),
        api_key_as = text_input("api_key_as", "Đổi tên biến khi nạp (tuỳ chọn)", api_key_as, "cơ chế acc8 cũ: OPENCODE_API_KEY → ANTHROPIC_API_KEY"),
        model = text_input("model", "Model (provider/model với opencode)", model, "ví dụ opencode-go/deepseek-v4.1-flash; với claude cũng ghi `model` vào settings.json"),
        effort = text_input("effort", "Effort (chỉ Claude)", effort, "ghi `modelSettings.<model>.effortLevel`"),
    )
}

fn text_input(name: &str, label: &str, value: &str, hint: &str) -> String {
    format!(
        "<label>{label}</label><small>{}</small>\
         <input type=\"text\" name=\"{name}\" value=\"{}\" autocomplete=\"off\" spellcheck=\"false\">",
        esc(hint),
        esc(value)
    )
}

fn login_page(name: &str, msg: &str, ticket: &str, err: Option<&str>) -> String {
    format!(
        "<h2>Đăng nhập <code>{}</code></h2>{}<pre class=\"guide\">{}</pre>\
         <form method=\"POST\" action=\"/acc/{}/login-code\">\
         <input type=\"hidden\" name=\"t\" value=\"{}\">\
         <label>Mã OAuth</label><small>dán mã mà trang đăng nhập của Claude hiện ra</small>\
         <input type=\"text\" name=\"code\" autocomplete=\"off\" spellcheck=\"false\">\
         <p><button>Gửi mã</button></p></form>\
         <p><a class=\"btn sec\" href=\"/acc/{}\">← về {}</a></p>",
        esc(name),
        err.map(|e| format!("<div class=\"err\">{}</div>", esc(e)))
            .unwrap_or_default(),
        esc(msg),
        url_encode(name),
        esc(ticket),
        url_encode(name),
        esc(name),
    )
}

// ── Biểu mẫu → hàng cấu hình ───────────────────────────────────────────────

fn row_from_form(
    form: &BTreeMap<String, String>,
    existing: Option<&ClaudeAccountCfg>,
) -> Result<ClaudeAccountCfg, String> {
    let cli = form
        .get("cli")
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let cli = match cli.as_str() {
        "" | "claude" => None,
        "opencode" => Some("opencode".to_string()),
        "codex" => Some("codex".to_string()),
        other => {
            return Err(format!(
                "CLI `{other}` không hợp lệ (chỉ claude/opencode/codex)"
            ))
        }
    };
    let oc_text = form
        .get("opencode_config")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let opencode_config = if oc_text.is_empty() {
        None
    } else {
        let v: serde_json::Value = serde_json::from_str(&oc_text)
            .map_err(|e| format!("opencode_config không phải JSON: {e}"))?;
        if !v.is_object() {
            return Err("opencode_config phải là một object JSON".into());
        }
        Some(v)
    };
    let mut r = ClaudeAccountCfg {
        name: form
            .get("name")
            .map(|s| s.trim().to_string())
            .unwrap_or_default(),
        cli,
        config_dir: opt(form, "config_dir"),
        base_url: opt(form, "base_url"),
        env_file: opt(form, "env_file"),
        api_key_env: opt(form, "api_key_env"),
        api_key_as: opt(form, "api_key_as"),
        model: opt(form, "model"),
        effort: opt(form, "effort"),
        opencode_config,
        extra_env: parse_env_lines(form.get("extra_env").map(String::as_str).unwrap_or("")),
        extra_args: parse_arg_lines(form.get("extra_args").map(String::as_str).unwrap_or("")),
        ..Default::default()
    };
    // `launch` và `locked` KHÔNG đến từ biểu mẫu: di trú (GĐ 5) và quyết định
    // khoá là việc riêng. Không chép lại `launch` ở đây là biểu mẫu LẶNG LẼ xoá
    // dòng shell đang chạy thật của chủ máy.
    if let Some(e) = existing {
        r.launch = e.launch.clone();
        r.locked = e.locked;
    }
    Ok(r)
}

fn opt(form: &BTreeMap<String, String>, key: &str) -> Option<String> {
    form.get(key)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn parse_env_lines(s: &str) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = t.split_once('=') {
            let k = k.trim();
            if !k.is_empty() {
                m.insert(k.to_string(), v.trim().to_string());
            }
        }
    }
    m
}

fn parse_arg_lines(s: &str) -> Vec<String> {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// Đường cấu hình mặc định khi THÊM — đúng quy ước `scripts/them-tai-khoan.sh`
/// và đúng thứ `~/.zshrc` của chủ máy dùng (`claude3` → `~/.claude-acc3`).
/// Giữ dạng `~/...` chứ không tuyệt đối hoá: cấu hình không nên gõ cứng tên
/// người dùng.
fn derived_config_dir(name: &str) -> String {
    format!("~/.claude-{name}")
}

// ── HTTP nhỏ ───────────────────────────────────────────────────────────────

fn split_target(target: &str) -> (String, BTreeMap<String, String>) {
    match target.split_once('?') {
        Some((p, q)) => (p.to_string(), parse_form(q)),
        None => (target.to_string(), BTreeMap::new()),
    }
}

fn cookie_value(headers: &BTreeMap<String, String>, key: &str) -> Option<String> {
    let raw = headers.get("cookie")?;
    for part in raw.split(';') {
        if let Some((k, v)) = part.trim().split_once('=') {
            if k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn origin_ok(req: &Req, expected: &str) -> bool {
    if let Some(o) = req.headers.get("origin") {
        return o == expected;
    }
    if let Some(r) = req.headers.get("referer") {
        return r == expected || r.starts_with(&format!("{expected}/"));
    }
    false
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(*b, b'-' | b'_' | b'.' | b'~') {
            out.push(*b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Vỏ trang ───────────────────────────────────────────────────────────────

const STYLE: &str = "body{font:16px/1.55 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;\
max-width:60rem;margin:2.5rem auto;padding:0 1.25rem;color:#1a1a1a;background:#fbfbfa}\
h1{font-size:1.35rem;margin:0 0 .2rem}h2{font-size:1.05rem;margin:2rem 0 .4rem}\
p.lead{color:#555;margin:0 0 1.4rem}\
label{display:block;font-weight:600;margin:1rem 0 .2rem}\
small{display:block;color:#666;font-weight:400;margin-bottom:.35rem}\
input,textarea{width:100%;padding:.5rem .6rem;font:inherit;border:1px solid #ccc;border-radius:8px;\
background:#fff;box-sizing:border-box}textarea{font-family:ui-monospace,Menlo,monospace;font-size:.9rem}\
button{padding:.55rem 1.1rem;font:inherit;font-weight:600;border:0;border-radius:8px;\
background:#1a6dd4;color:#fff;cursor:pointer}button.sec{background:#eceef1;color:#1a1a1a}\
a.btn{display:inline-block;padding:.5rem 1rem;border-radius:8px;background:#1a6dd4;color:#fff;\
text-decoration:none;font-weight:600}a.btn.sec{background:#eceef1;color:#1a1a1a}\
table{width:100%;border-collapse:collapse;margin:1rem 0}th,td{text-align:left;padding:.5rem .55rem;\
border-bottom:1px solid #e4e4e0;vertical-align:top;overflow-wrap:anywhere}\
tbody tr:nth-child(even){background:#f4f4f1}\
code{background:#eceef1;padding:.08rem .3rem;border-radius:4px;font-size:.9em;overflow-wrap:anywhere}\
ul.tight{margin:0;padding-left:1.1rem}ul.checks{list-style:none;padding:0;margin:.4rem 0}\
ul.checks li{margin:.35rem 0}.ok{color:#0a7d33}.bad{color:#c0392b}.unk{color:#a06a00}\
.err{background:#fdecea;border-left:3px solid #c0392b;padding:.7rem 1rem;margin:.8rem 0;\
border-radius:0 8px 8px 0;white-space:pre-wrap}.note{background:#fff8e1;border-left:3px solid #e0a800;\
padding:.7rem 1rem;margin:1.2rem 0;border-radius:0 8px 8px 0}\
.flash{background:#eaf6ee;border-left:3px solid #0a7d33;padding:.7rem 1rem;margin:.8rem 0;\
border-radius:0 8px 8px 0;white-space:pre-wrap}.flash.err{background:#fdecea;border-color:#c0392b}\
.row{display:flex;gap:.6rem;flex-wrap:wrap;margin:1rem 0}.row form{margin:0}.muted{color:#666;font-size:.9rem}\
pre.guide{background:#111;color:#eee;padding:1rem;border-radius:8px;overflow-x:auto;white-space:pre-wrap}\
.grid{display:block}\
@media (max-width:600px){thead{display:none}table,tbody,tr,td{display:block}\
tr{border:1px solid #e4e4e0;border-radius:8px;padding:.6rem .75rem;margin:0 0 .8rem;background:#fff}\
tbody tr:nth-child(even){background:#f4f4f1}td{border:0;padding:.1rem 0}\
td:first-child{font-weight:700;font-size:1.05rem}\
td:nth-child(2)::before{content:\"Thư mục: \";color:#666}\
td:nth-child(3)::before{content:\"Chẩn đoán: \";color:#666}}";

fn page_shell(title: &str, flash: Option<(String, bool)>, body: &str) -> String {
    let flash_html = match flash {
        Some((msg, is_err)) => format!(
            "<div class=\"flash{}\">{}</div>",
            if is_err { " err" } else { "" },
            esc(&msg)
        ),
        None => String::new(),
    };
    format!(
        "<!doctype html><html lang=\"vi\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>huba · tài khoản</title><style>{STYLE}</style></head><body>\
         <h1>huba · tài khoản</h1><p class=\"lead\">{}</p>{flash_html}{body}\
         <p class=\"muted\" style=\"margin-top:2rem\"><a href=\"/\">← danh sách</a> · \
         trang chỉ chạy trên máy này · Ctrl-C để đóng</p></body></html>",
        esc(title)
    )
}

// ── Chạy thật ───────────────────────────────────────────────────────────────

/// `huba accounts` — mở trang trên 127.0.0.1 rồi phục vụ tới khi Ctrl-C.
///
/// `open_window` = true thì tự mở cửa sổ trình duyệt (người dùng gõ tay);
/// `--no-open` truyền false — nghiệm thu headless đọc URL in ra, không bật cửa sổ.
pub fn serve(cfg: &Config, db: &Db, mo_cua_so: bool) -> Result<()> {
    let (listener, port) = bind_loopback()?;
    let ticket = new_ticket(&listener);
    let url = format!("http://127.0.0.1:{port}/?t={ticket}");
    crate::logging::info("accounts_started", json!({ "port": port }));
    println!("\n  Mở trang quản lý tài khoản:\n\n    {url}\n");
    println!("  (chỉ máy này vào được — Ctrl-C để đóng)\n");
    if mo_cua_so {
        open_window(&url, "accounts");
    }

    let mut login = RealLogin { cfg, db };
    let mut st = PageState::new(cfg, ticket, format!("http://127.0.0.1:{port}"), &mut login);
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                crate::logging::warn(
                    "accounts_accept_failed",
                    json!({ "port": port, "err": e.to_string() }),
                );
                continue;
            }
        };
        let resp = match read_request(&mut stream) {
            Ok(req) => route(&req, &mut st),
            Err(e) => Resp::text("400 Bad Request", format!("request hỏng: {e}")),
        };
        if let Err(e) = write_resp(
            &mut stream,
            resp.status,
            resp.content_type,
            &resp.headers,
            resp.body.as_bytes(),
        ) {
            crate::logging::warn(
                "accounts_write_failed",
                json!({ "port": port, "err": e.to_string() }),
            );
        }
    }
    Ok(())
}

/// Bảng cho `huba accounts --print` — cho script và cho một cái nhìn nhanh ở
/// terminal. Thuần ĐỌC: không mở cổng, không mở cửa sổ.
pub fn print_table(cfg: &Config) -> String {
    let accs = cfg.claude_accounts_or_ambient();
    let mut out = format!("accounts: {}\n", accs.len());
    for acc in &accs {
        let (ok, total, hong) = check_summary(acc);
        let dir = if acc.is_opencode() {
            accounts::opencode_profile_dir(acc).display().to_string()
        } else if acc.is_codex() {
            accounts::codex_home(acc).display().to_string()
        } else {
            acc.config_dir
                .as_deref()
                .filter(|d| !d.trim().is_empty())
                .unwrap_or("(mặc định)")
                .to_string()
        };
        out.push_str(&format!(
            "  {:<10} {:<9} {}/{} OK{:<2}  {}{}\n",
            acc.name,
            acc.cli_name(),
            ok,
            total,
            if acc.locked { "🔒" } else { "" },
            dir,
            if hong.is_empty() {
                String::new()
            } else {
                format!("  ({} mục cần xem)", hong.len())
            }
        ));
        for c in hong.iter().take(3) {
            out.push_str(&format!("             {}\n", c.line()));
        }
        if hong.len() > 3 {
            out.push_str(&format!("             … +{} mục\n", hong.len() - 3));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_va_url_encode_di_ve_hai_chieu() {
        for name in ["acc9", "mặc định", "a b", "acc-1_x.y"] {
            assert_eq!(percent_decode(&url_encode(name)), name, "vòng {name:?}");
        }
    }

    #[test]
    fn split_target_tach_query_va_giu_path() {
        let (p, q) = split_target("/acc/acc9?t=abc&x=1");
        assert_eq!(p, "/acc/acc9");
        assert_eq!(q.get("t").unwrap(), "abc");
        assert_eq!(q.get("x").unwrap(), "1");
        let (p, q) = split_target("/");
        assert_eq!(p, "/");
        assert!(q.is_empty());
    }

    #[test]
    fn cookie_chim_trong_chuoi_nhieu_cookie() {
        let mut h = BTreeMap::new();
        h.insert(
            "cookie".to_string(),
            "khac=1; huba_ticket=ve-that; cuoi=2".to_string(),
        );
        assert_eq!(cookie_value(&h, "huba_ticket").as_deref(), Some("ve-that"));
    }

    #[test]
    fn parse_env_lines_bo_dong_trong_va_chu_thich() {
        let m = parse_env_lines("A=1\n\n# bỏ\nB = hai = ba \nKHONGCODAU");
        assert_eq!(m.get("A").unwrap(), "1");
        assert_eq!(m.get("B").unwrap(), "hai = ba");
        assert!(!m.contains_key("KHONGCODAU"));
    }

    #[test]
    fn config_dir_mac_dinh_theo_quy_uoc_zshrc() {
        assert_eq!(derived_config_dir("acc9"), "~/.claude-acc9");
    }
}
