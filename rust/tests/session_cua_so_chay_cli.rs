//! Quang 13/11 (Ubuntu): hai `/new -a acc8` xong, `/session` đáp "Không có phiên nào
//! đang sống" — ảnh chụp có 3 hàng, cả ba là "cửa sổ trần" chạy opencode chưa ghép
//! được phiên, và bộ lọc `host != "shell"` bỏ hết.
use huba::sessions::{la_phien_chua_ghep, LiveSession};

fn shell(activity: Option<&str>) -> LiveSession {
    LiveSession {
        host: "shell".into(),
        kind: "shell".into(),
        activity: activity.map(str::to_string),
        ..Default::default()
    }
}

#[test]
fn cua_so_chay_opencode_hay_claude_la_phien() {
    assert!(la_phien_chua_ghep(&shell(Some("opencode"))));
    assert!(la_phien_chua_ghep(&shell(Some("claude"))));
    assert!(la_phien_chua_ghep(&shell(Some(
        "opencode · ĐANG HỎI MẬT KHẨU"
    ))));
}

#[test]
fn shell_tran_hay_chuong_trinh_khac_khong_phai() {
    assert!(!la_phien_chua_ghep(&shell(None)));
    assert!(!la_phien_chua_ghep(&shell(Some("nano"))));
    assert!(!la_phien_chua_ghep(&shell(Some("ĐANG HỎI MẬT KHẨU"))));
    assert!(!la_phien_chua_ghep(&shell(Some("opencode-helper"))));
    // Hàng phiên thật không qua cửa này (nó đã được giữ bằng `host`).
    let mut p = shell(Some("opencode"));
    p.host = "terminal".into();
    assert!(!la_phien_chua_ghep(&p));
}
