//! W3.1 — Linux: `localserver::open_window` ưu tiên trình duyệt kiểu Chrome với `--app` (cửa sổ riêng, không thanh địa chỉ),
//! máy chỉ có Firefox thì rơi về `xdg-open` (tab thường). Đo CẢ HAI nhánh bằng trình duyệt GIẢ đặt trong một `PATH` riêng:
//! mỗi bản giả ghi các đối số nó nhận vào một tệp, nên "có được gọi không, với gì" là bằng chứng, không phải suy luận.
//!
//! `PATH` là trạng thái của cả tiến trình ⟹ MỘT `#[test]` duy nhất chạy các ca nối tiếp (hai `#[test]` song song sẽ giẫm
//! lên nhau và cho kết quả lúc xanh lúc đỏ).
#![cfg(target_os = "linux")]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use huba::localserver::open_window;

const URL: &str = "http://127.0.0.1:43210/?v=abc";

/// Tạo trình duyệt giả `name` trong `dir`: ghi đối số vào `<dir>/<name>.argv`, rồi (tuỳ chọn) ngủ `ngu_giay` giây như một
/// trình duyệt thật còn sống sau khi mở cửa sổ. `thuc_thi=false` ⟹ tệp có mặt nhưng KHÔNG có quyền chạy.
fn gia(dir: &Path, name: &str, ngu_giay: u32, thuc_thi: bool) {
    let out = dir.join(format!("{name}.argv"));
    let body = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n{}",
        out.display(),
        if ngu_giay > 0 {
            format!("/bin/sleep {ngu_giay}\n")
        } else {
            String::new()
        }
    );
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    let mode = if thuc_thi { 0o755 } else { 0o644 };
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode)).unwrap();
}

/// Đối số bản giả `name` đã nhận (đợi tối đa 3 s vì trình duyệt được chạy NỀN). `None` = không bao giờ được gọi.
fn da_nhan(dir: &Path, name: &str) -> Option<Vec<String>> {
    let f = dir.join(format!("{name}.argv"));
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(3) {
        if let Ok(s) = std::fs::read_to_string(&f) {
            if s.ends_with('\n') {
                return Some(s.lines().map(str::to_string).collect());
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    None
}

/// Khẳng định `name` KHÔNG được gọi: chờ một quãng ngắn (trình duyệt chạy nền, gọi muộn một chút vẫn phải bị bắt).
fn khong_duoc_goi(dir: &Path, name: &str) {
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        !dir.join(format!("{name}.argv")).exists(),
        "`{name}` không được gọi ở ca này nhưng đã nhận đối số"
    );
}

fn chay_voi_path(dir: &Path) -> Duration {
    chay_voi_path_tho(dir.as_os_str())
}

fn chay_voi_path_tho(path: &std::ffi::OsStr) -> Duration {
    let cu: Option<std::ffi::OsString> = std::env::var_os("PATH");
    std::env::set_var("PATH", path);
    let t = Instant::now();
    open_window(URL, "w31test");
    let dt = t.elapsed();
    match cu {
        Some(v) => std::env::set_var("PATH", v),
        None => std::env::remove_var("PATH"),
    }
    dt
}

fn thu_muc(ten: &str) -> (tempfile::TempDir, PathBuf) {
    let d = tempfile::Builder::new().prefix(ten).tempdir().unwrap();
    let p = d.path().to_path_buf();
    (d, p)
}

#[test]
fn linux_opens_an_app_window_when_a_chrome_family_browser_exists_else_xdg_open() {
    // Ca 1 — có chromium + xdg-open: PHẢI mở `chromium --app=<url>`, KHÔNG đụng xdg-open.
    let (_k1, d1) = thu_muc("w31-c1-");
    gia(&d1, "chromium", 0, true);
    gia(&d1, "xdg-open", 0, true);
    chay_voi_path(&d1);
    assert_eq!(
        da_nhan(&d1, "chromium"),
        Some(vec![format!("--app={URL}")]),
        "máy có chromium ⟹ phải mở cửa sổ riêng bằng --app"
    );
    khong_duoc_goi(&d1, "xdg-open");

    // Ca 2 — CHỈ có xdg-open (máy chỉ Firefox): rơi về xdg-open với đúng URL, không --app.
    let (_k2, d2) = thu_muc("w31-c2-");
    gia(&d2, "xdg-open", 0, true);
    chay_voi_path(&d2);
    assert_eq!(
        da_nhan(&d2, "xdg-open"),
        Some(vec![URL.to_string()]),
        "máy không có trình duyệt kiểu Chrome ⟹ xdg-open nhận nguyên URL"
    );

    // Ca 3 — thứ tự ưu tiên: có cả google-chrome lẫn chromium ⟹ google-chrome thắng.
    let (_k3, d3) = thu_muc("w31-c3-");
    gia(&d3, "chromium", 0, true);
    gia(&d3, "google-chrome", 0, true);
    gia(&d3, "xdg-open", 0, true);
    chay_voi_path(&d3);
    assert_eq!(
        da_nhan(&d3, "google-chrome"),
        Some(vec![format!("--app={URL}")])
    );
    khong_duoc_goi(&d3, "chromium");
    khong_duoc_goi(&d3, "xdg-open");

    // Ca 4 — tệp tên `chromium` có mặt nhưng KHÔNG có quyền chạy ⟹ bị bỏ qua, rơi về xdg-open (đối chứng ngược của ca 1).
    let (_k4, d4) = thu_muc("w31-c4-");
    gia(&d4, "chromium", 0, false);
    gia(&d4, "xdg-open", 0, true);
    chay_voi_path(&d4);
    assert_eq!(da_nhan(&d4, "xdg-open"), Some(vec![URL.to_string()]));
    khong_duoc_goi(&d4, "chromium");

    // Ca 4b — ưu tiên đứng đầu (`google-chrome`) là tệp KHÔNG chạy được, `chromium` phía sau chạy được ⟹ phải chọn
    // `chromium --app`, không được chọn nhầm tệp hỏng rồi bỏ cuộc sang xdg-open. (Ca 4 một mình không phân biệt được
    // "bỏ qua tệp hỏng" với "chọn tệp hỏng, spawn lỗi, rơi về xdg-open" — kết quả nhìn ngoài giống nhau.)
    let (_k4b, d4b) = thu_muc("w31-c4b-");
    gia(&d4b, "google-chrome", 0, false);
    gia(&d4b, "chromium", 0, true);
    gia(&d4b, "xdg-open", 0, true);
    chay_voi_path(&d4b);
    assert_eq!(
        da_nhan(&d4b, "chromium"),
        Some(vec![format!("--app={URL}")]),
        "tệp google-chrome không chạy được phải bị bỏ qua để chọn chromium"
    );
    khong_duoc_goi(&d4b, "google-chrome");
    khong_duoc_goi(&d4b, "xdg-open");

    // Ca 5 — trình duyệt thật còn sống sau khi mở cửa sổ: `open_window` PHẢI trả về ngay (spawn), không đợi tới khi đóng
    // cửa sổ — nếu đợi, `huba ui` treo cho tới lúc người dùng tắt cửa sổ.
    let (_k5, d5) = thu_muc("w31-c5-");
    gia(&d5, "chromium", 3, true);
    gia(&d5, "xdg-open", 0, true);
    let dt = chay_voi_path(&d5);
    assert!(
        dt < Duration::from_millis(1500),
        "open_window chặn {dt:?} — phải chạy trình duyệt NỀN"
    );
    assert!(
        da_nhan(&d5, "chromium").is_some(),
        "chromium phải được gọi ở ca 5"
    );

    // Ca 6 — không có trình duyệt nào, cũng không có xdg-open: KHÔNG panic, KHÔNG treo (đường dẫn đã in ra màn hình).
    let (_k6, d6) = thu_muc("w31-c6-");
    let dt = chay_voi_path(&d6);
    assert!(dt < Duration::from_millis(1500), "ca trống treo {dt:?}");

    // Ca 7 — mục RỖNG trong PATH nghĩa là thư mục hiện tại: một `chromium` nằm cạnh cwd KHÔNG được chạy chỉ vì cwd
    // tình cờ ở đó (huba chạy từ thư mục người dùng tự chọn). Chỉ có xdg-open thật trong PATH ⟹ xdg-open.
    let (_k7, d7) = thu_muc("w31-c7-"); // sẽ là cwd, chứa chromium giả
    let (_k7b, d7b) = thu_muc("w31-c7b-"); // nằm trong PATH, chỉ có xdg-open
    gia(&d7, "chromium", 0, true);
    gia(&d7b, "xdg-open", 0, true);
    let cu_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&d7).unwrap();
    let mut p = std::ffi::OsString::from(":");
    p.push(d7b.as_os_str());
    chay_voi_path_tho(&p);
    std::env::set_current_dir(&cu_cwd).unwrap();
    assert_eq!(da_nhan(&d7b, "xdg-open"), Some(vec![URL.to_string()]));
    khong_duoc_goi(&d7, "chromium");
}
