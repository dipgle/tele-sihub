//! Khung HTTP loopback dùng chung cho những trang chạy NGAY TRÊN MÁY cài huba.
//!
//! Tách khỏi `setup.rs` ngày 2026-11-09 khi có trang thứ hai (`huba accounts`).
//! Không phải để "cho đẹp kiến trúc": hai trang mà tự chép lấy phần đọc request
//! và dựng vé thì hàng rào sẽ trôi ở một trong hai — và cái trôi thường là cái
//! không ai nhìn.
//!
//! Ba ranh giới giữ nguyên từ `setup.rs`:
//! * **Chỉ 127.0.0.1**, không bao giờ `0.0.0.0` — trang này không có việc gì ở
//!   ngoài máy.
//! * **Vé một lần trong URL**: một tiến trình khác trên cùng máy vẫn gọi được
//!   cổng loopback, nên cổng không phải là hàng rào; cái vé mới là.
//! * **Không bao giờ hiện lại giá trị đã lưu** — trang chỉ nói khoá ấy *đã có*
//!   hay *chưa có*.
//!
//! Tệp này KHÔNG biết gì về `huba.env`, về tài khoản, về HTML của từng trang:
//! nó chỉ đọc một request, trả một response, giải mã form. Ai gọi thì tự quyết
//! nội dung.

use anyhow::Result;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};

use crate::logging;

/// Mở cổng trên 127.0.0.1, để hệ điều hành tự chọn số cổng.
///
/// Trả cả listener lẫn cổng: chỗ gọi cần cổng để in URL, và cần listener để
/// nhận request — đọc lại `local_addr()` ở nơi khác là mở đường cho hai con số
/// lệch nhau.
pub fn bind_loopback() -> Result<(TcpListener, u16)> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
    let port = listener.local_addr()?.port();
    Ok((listener, port))
}

/// Vé một lần. `SystemTime` + địa chỉ con trỏ là đủ cho một cái vé sống vài phút
/// trên máy của chính mình; đây không phải khoá phiên đăng nhập.
pub fn new_ticket(listener: &TcpListener) -> String {
    format!(
        "{:x}{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        listener as *const _ as usize
    )
}

/// Trần thân yêu cầu (1 MiB) — mọi trang loopback của huba chỉ nhận form nhỏ.
pub const MAX_BODY: usize = 1 << 20;
/// Trần phần đầu (dòng yêu cầu + mọi header): 64 KiB.
pub const MAX_HEADERS: usize = 64 * 1024;

/// Một request đã đọc TRỌN: dòng đầu, mọi header (tên viết thường), và body
/// theo `Content-Length`.
#[derive(Debug, Clone, Default)]
pub struct Req {
    pub method: String,
    pub target: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

/// Đọc một request từ kết nối. KHÔNG kiểm vé, KHÔNG định tuyến — đó là việc của
/// trang gọi nó, vì mỗi trang có một cách đặt vé khác nhau.
pub fn read_request(stream: &mut TcpStream) -> Result<Req> {
    let mut reader = BufReader::new(stream.try_clone()?);
    // Trần cho cả dòng đầu lẫn header: `read_line` trần đọc tới khi gặp `\n`, nên một dòng 4 GB không có xuống dòng
    // nuốt hết bộ nhớ. `take` chặn MỖI lần đọc dòng ở phần còn lại của ngân sách (W1.6, 23/11).
    let mut used = 0usize;
    let mut read_capped = |reader: &mut BufReader<TcpStream>, line: &mut String| -> Result<usize> {
        let left = MAX_HEADERS.saturating_sub(used) as u64 + 1;
        let n = Read::by_ref(reader).take(left).read_line(line)?;
        used += n;
        if used > MAX_HEADERS {
            anyhow::bail!("headers_too_large");
        }
        Ok(n)
    };
    let mut request = String::new();
    read_capped(&mut reader, &mut request)?;
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut headers = BTreeMap::new();
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        if read_capped(&mut reader, &mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim().to_ascii_lowercase();
        let val = v.trim().to_string();
        if key == "content-length" {
            len = val.parse().unwrap_or(0);
        }
        headers.insert(key, val);
    }
    // Từ chối TRƯỚC khi cấp phát: `vec![0u8; len]` với `len` do bên gọi tự khai là cửa tấn công từ chối dịch vụ.
    if len > MAX_BODY {
        anyhow::bail!("body_too_large");
    }
    let mut body = vec![0u8; len];
    if len > 0 {
        reader.read_exact(&mut body)?;
    }
    Ok(Req {
        method,
        target,
        headers,
        body,
    })
}

/// Trả một response trọn vẹn rồi đóng kết nối.
pub fn respond(stream: &mut TcpStream, status: &str, ctype: &str, body: &[u8]) -> Result<()> {
    write_resp(stream, status, ctype, &[], body)
}

/// Như [`respond`] nhưng kèm HEADER phụ — chỗ dùng là `Set-Cookie` và
/// `Location` của `303 See Other`, thứ biến "vé nằm trong thanh địa chỉ" thành
/// "vé nằm trong cookie".
///
/// `headers` nhận `(tên, giá trị)` chứ không phải một chuỗi thô: nơi gọi không
/// tự nối chuỗi HTTP, nên không có đường nào chèn `\r\n` vào giữa header.
pub fn write_resp(
    stream: &mut TcpStream,
    status: &str,
    ctype: &str,
    headers: &[(&str, String)],
    body: &[u8],
) -> Result<()> {
    let mut head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (k, v) in headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}

/// Trình duyệt kiểu Chrome biết `--app=<url>` trên Linux, THEO THỨ TỰ ƯU TIÊN (tên tệp chạy được trong `PATH`).
/// Firefox không có trong danh sách vì không có `--app` ⟹ máy chỉ Firefox rơi về `xdg-open`.
pub const LINUX_APP_BROWSERS: &[&str] = &[
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
    "brave-browser",
    "microsoft-edge",
];

/// Tệp thường VÀ có quyền chạy (một tệp tên `chromium` không chạy được không phải trình duyệt).
fn la_tep_chay_duoc(p: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p)
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

/// Trình duyệt `--app` đầu tiên (theo [`LINUX_APP_BROWSERS`], rồi theo thứ tự thư mục trong `path_var`).
/// Chỉ ĐỌC đĩa. Mục rỗng trong `PATH` (= thư mục hiện tại) bị bỏ qua: không chạy chương trình nằm cạnh cwd.
pub fn find_linux_app_browser(path_var: &std::ffi::OsStr) -> Option<std::path::PathBuf> {
    let dirs: Vec<std::path::PathBuf> = std::env::split_paths(path_var)
        .filter(|d| !d.as_os_str().is_empty())
        .collect();
    for name in LINUX_APP_BROWSERS {
        for d in &dirs {
            let p = d.join(name);
            if la_tep_chay_duoc(&p) {
                return Some(p);
            }
        }
    }
    None
}

/// Mở một URL thành CỬA SỔ ỨNG DỤNG, không phải một tab lẫn trong bầy.
///
/// Tách từ `setup.rs` ngày 2026-11-09 khi có trang thứ hai (`huba accounts`):
/// cùng một quyết định "mở bằng `--app` của Chrome/Edge nếu có, rơi về `open`",
/// và cùng một tính chất — `open` hỏng thì KHÔNG làm hỏng cả lệnh, vì đường
/// dẫn đã in ra màn hình rồi.
///
/// `tag` đặt tiền tố tên sự kiện log (`setup_window_opened`,
/// `accounts_open_failed`…), để log vẫn nói được trang NÀO mở.
pub fn open_window(url: &str, tag: &str) {
    let opened = format!("{tag}_window_opened");
    let failed = format!("{tag}_open_failed");
    // Windows không có `open`: `start` (lệnh trong `cmd`) mở bằng trình duyệt
    // mặc định. Chuỗi rỗng `""` là tiêu đề cửa sổ — thiếu nó thì `start` đọc
    // chính URL (trong ngoặc kép) làm tiêu đề và không mở gì.
    if cfg!(windows) {
        match std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
        {
            Ok(s) if s.success() => logging::info(&opened, json!({ "as": "windows-start" })),
            other => logging::warn(
                &failed,
                json!({ "detail": format!("{other:?}"), "url_printed": true }),
            ),
        }
        return;
    }
    // Linux: có trình duyệt kiểu Chrome ⟹ `--app` (cửa sổ riêng, như macOS); không có (máy chỉ Firefox)
    // ⟹ `xdg-open` (trình duyệt mặc định của phiên đồ hoạ, tab thường). Đo 13/11 trên Ubuntu: `xdg-open`
    // chạy (HTTP 200) mà không tự mở ⟹ hai nhánh đều phải LOG, không im lặng (W3.1, tests/open_window_linux.rs).
    if cfg!(target_os = "linux") {
        if let Some(bin) = std::env::var_os("PATH").and_then(|p| find_linux_app_browser(&p)) {
            let ten = bin.display().to_string();
            match std::process::Command::new(&bin)
                .arg(format!("--app={url}"))
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                // `spawn`, KHÔNG `status`: trình duyệt sống cho tới khi người dùng đóng cửa sổ — đợi nó
                // là treo `huba ui` ngay sau khi mở trang.
                .spawn()
            {
                Ok(mut child) => {
                    logging::info(&opened, json!({ "as": "app", "browser": ten }));
                    // Thu dọn tiến trình con (không để zombie) và NÓI RA nếu nó thoát lỗi — trình duyệt chết
                    // ngay lúc mở (không có màn hình, hồ sơ khoá…) không được thành im lặng.
                    let failed = failed.clone();
                    std::thread::spawn(move || match child.wait() {
                        Ok(s) if s.success() => {}
                        other => logging::warn(
                            &failed,
                            json!({ "detail": format!("{other:?}"), "browser": ten,
                                    "stage": "browser_exited", "url_printed": true }),
                        ),
                    });
                    return;
                }
                Err(e) => logging::warn(
                    &failed,
                    json!({ "detail": e.to_string(), "browser": ten,
                            "stage": "browser_spawn", "fallback": "xdg-open" }),
                ),
            }
        }
        match std::process::Command::new("xdg-open")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            Ok(s) if s.success() => logging::info(&opened, json!({ "as": "xdg-open" })),
            other => logging::warn(
                &failed,
                json!({ "detail": format!("{other:?}"), "url_printed": true }),
            ),
        }
        return;
    }
    const APP_BROWSERS: &[&str] = &[
        "Google Chrome",
        "Microsoft Edge",
        "Brave Browser",
        "Chromium",
    ];
    for b in APP_BROWSERS {
        let ok = std::process::Command::new("open")
            .args(["-na", b, "--args", &format!("--app={url}")])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            logging::info(&opened, json!({ "as": "app", "browser": b }));
            return;
        }
    }
    match std::process::Command::new("open").arg(url).status() {
        Ok(s) if s.success() => {
            logging::info(&opened, json!({ "as": "browser-tab" }));
        }
        other => logging::warn(
            &failed,
            json!({ "detail": format!("{other:?}"), "url_printed": true }),
        ),
    }
}

/// `a=1&b=2` với mã hoá phần trăm → cặp khoá–giá trị.
pub fn parse_form(body: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for pair in body.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        out.insert(percent_decode(k), percent_decode(v));
    }
    out
}

pub fn percent_decode(s: &str) -> String {
    let bytes = s.replace('+', " ").into_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(b) = u8::from_str_radix(hex, 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Đối chứng ngược của chính phép giải mã: chuỗi có `%` KHÔNG hợp lệ phải
    /// đi qua nguyên văn, không được nuốt ký tự.
    #[test]
    fn a_broken_percent_escape_survives_verbatim() {
        assert_eq!(percent_decode("a%zzb"), "a%zzb");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("a%2"), "a%2");
    }

    #[test]
    fn form_parsing_keeps_equals_and_plus_inside_values() {
        let form = parse_form("k=a%3Db%2Bc&k2=x+y&rong");
        assert_eq!(form.get("k").unwrap(), "a=b+c");
        assert_eq!(form.get("k2").unwrap(), "x y");
        assert!(!form.contains_key("rong"), "cặp không có `=` phải bị bỏ");
    }

    /// Gửi `raw` tới một ổ cắm loopback và đọc request ở phía máy chủ.
    fn doc(raw: &'static [u8]) -> Result<Req> {
        let (listener, port) = bind_loopback().unwrap();
        let t = std::thread::spawn(move || {
            let mut c = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
            c.write_all(raw).unwrap();
            c.flush().unwrap();
            // giữ kết nối mở một lúc để phía đọc không gặp EOF sớm
            std::thread::sleep(std::time::Duration::from_millis(300));
        });
        let (mut s, _) = listener.accept().unwrap();
        s.set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        let r = read_request(&mut s);
        t.join().unwrap();
        r
    }

    /// 🔴 W1.6: `Content-Length` khai 4 GB mà không có thân — bản cũ `vec![0u8; len]` cấp phát 4 GB TRƯỚC khi đọc.
    #[test]
    fn a_huge_content_length_is_refused_before_allocating() {
        let e = doc(b"POST / HTTP/1.1\r\nHost: x\r\nContent-Length: 4294967296\r\n\r\n")
            .expect_err("phải từ chối");
        assert!(e.to_string().contains("body_too_large"), "{e}");
    }

    /// Đối chứng ngược: thân nhỏ vẫn đọc được trọn.
    #[test]
    fn a_small_body_is_still_read_whole() {
        let r = doc(b"POST /x HTTP/1.1\r\nHost: x\r\nContent-Length: 5\r\n\r\nabcde").unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.body, b"abcde");
    }

    /// Phần đầu (dòng yêu cầu + header) cũng có trần: một dòng header 100 KB không được nuốt bộ nhớ vô hạn.
    #[test]
    fn an_endless_header_is_refused() {
        let mut raw = b"GET / HTTP/1.1\r\nX-Rac: ".to_vec();
        raw.extend(std::iter::repeat_n(b'a', 100_000));
        raw.extend_from_slice(b"\r\n\r\n");
        let e = doc(Box::leak(raw.into_boxed_slice())).expect_err("phải từ chối");
        assert!(e.to_string().contains("headers_too_large"), "{e}");
    }
}
