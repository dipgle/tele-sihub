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
    let mut request = String::new();
    reader.read_line(&mut request)?;
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut headers = BTreeMap::new();
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
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
}
