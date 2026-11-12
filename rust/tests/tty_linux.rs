//! tty của bản Linux (tmux, Quang 12/11 «Làm bản Ubuntu dùng tmux đi»): `pts/4`.
//!
//! Hai chỗ hỏng mà tác tử soát đọc ra từ mã trước khi có máy Ubuntu: (1) id cửa sổ
//! `win-<tty>` cắt tty bằng `rsplit('/')` ⟹ `pts/4` thành `"4"`, mất tiền tố `tty`
//! mà `verbs`/`split_target` đòi; (2) `ps` procps in `?` (không phải `??`) cho
//! tiến trình không có tty ⟹ một phép dò `claude -p` không cửa sổ bị coi là có.

use huba::sessions::{is_real_tty, shell_id, tty_from_token, tty_of_window_id, tty_token};

#[test]
fn macos_giu_nguyen() {
    assert_eq!(tty_token("/dev/ttys004"), "ttys004");
    assert_eq!(tty_token("ttys004"), "ttys004");
    assert_eq!(shell_id("ttys004"), "win-ttys004");
    assert_eq!(tty_of_window_id("win-ttys004").as_deref(), Some("ttys004"));
}

#[test]
fn linux_pts_khu_hoi() {
    assert_eq!(tty_token("/dev/pts/4"), "ttypts4");
    assert_eq!(tty_token("pts/12"), "ttypts12");
    assert_eq!(tty_from_token("ttypts12"), "pts/12");
    assert_eq!(
        tty_of_window_id(&shell_id("pts/7")).as_deref(),
        Some("pts/7")
    );
}

#[test]
fn token_qua_duoc_lien_ket_telegram() {
    // `telegram::deep_link` chỉ nhận [A-Za-z0-9_-].
    let t = tty_token("pts/31");
    assert!(t.chars().all(|c| c.is_ascii_alphanumeric()), "{t}");
    assert!(t.starts_with("tty"), "{t}");
}

#[test]
fn token_la_khong_doan() {
    assert_eq!(tty_from_token("ttyptsx"), "ttyptsx");
    assert_eq!(tty_from_token("ttypts"), "ttypts");
}

#[test]
fn dau_hoi_cua_procps_khong_phai_tty() {
    assert!(!is_real_tty("?"));
    assert!(!is_real_tty("??"));
    assert!(is_real_tty("pts/4"));
}

#[test]
fn verbs_nhan_nut_cua_so_linux() {
    let v = huba::verbs::parse_command("/start w_ttypts4");
    let s = format!("{v:?}");
    assert!(s.contains("win-ttypts4"), "{s}");
}

#[test]
fn phien_khong_tty_cua_linux_la_detached() {
    use huba::sessions::classify_host;
    assert_eq!(classify_host("claude -p /usage", "", "?"), "detached");
    assert_eq!(classify_host("claude", "", "pts/4"), "terminal");
}
