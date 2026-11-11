//! Nhiều phiên chung MỘT cửa sổ Terminal (tab) — huba phải nhắm đúng TAB.
//!
//! 🔴 Quang 2026-11-11: *"tại sao giờ mở phiên mới lại ra cửa sổ terminal mới
//! thế, mở chung 1 cửa sổ thôi"*. Trước đó mọi lệnh AppleScript nhắm `selected
//! tab of window id W` — đúng khi mỗi cửa sổ một tab, SAI khi nhiều tab: gõ,
//! đọc, đóng vào tab đang chọn, tức việc của phiên khác (bẫy ghi từ 12/08).

use huba::keys::{handle_tty, parse_all_tabs, tab_handle, tab_ref, window_id};

#[test]
fn tay_cam_tab_mang_ca_cua_so_lan_tty() {
    let h = tab_handle(185, "/dev/ttys001");
    assert_ne!(h, 185, "tay cầm tab phải khác id cửa sổ trần");
    assert_eq!(window_id(h), 185);
    assert_eq!(handle_tty(h).as_deref(), Some("/dev/ttys001"));
    // `ps` khai `ttys001` không có /dev/ — cùng một tab.
    assert_eq!(tab_handle(185, "ttys001"), h);
    // Hai tab cùng cửa sổ là hai tay cầm khác nhau.
    assert_ne!(tab_handle(185, "/dev/ttys002"), h);
    // Gói lại một tay cầm đã gói = như cũ (lấy id cửa sổ của nó).
    assert_eq!(tab_handle(h, "/dev/ttys001"), h);
    // Id thật đo 07/11 vẫn gói được.
    assert_eq!(window_id(tab_handle(79292, "ttys012")), 79292);
}

#[test]
fn tty_khong_doc_duoc_thi_giu_id_cua_so_tran() {
    assert_eq!(tab_handle(185, ""), 185);
    assert_eq!(tab_handle(185, "??"), 185);
    assert_eq!(handle_tty(185), None);
    assert_eq!(window_id(185), 185);
}

#[test]
fn applescript_nham_dung_tab_theo_tty() {
    let h = tab_handle(185, "/dev/ttys001");
    assert_eq!(
        tab_ref(h),
        "(first tab of window id 185 whose tty is \"/dev/ttys001\")"
    );
    // Tay cầm cũ giữ nghĩa cũ.
    assert_eq!(tab_ref(185), "(selected tab of window id 185)");
}

#[test]
fn doc_danh_sach_tab_cua_terminal() {
    let out = "184 /dev/ttys000\n185 /dev/ttys001\n185 /dev/ttys007\nrác\n\n";
    assert_eq!(
        parse_all_tabs(out),
        vec![
            (184, "/dev/ttys000".to_string()),
            (185, "/dev/ttys001".to_string()),
            (185, "/dev/ttys007".to_string()),
        ]
    );
    assert!(parse_all_tabs("").is_empty());
}

/// Khoá hồi quy: không lệnh AppleScript nào trong `keys.rs` được nhắm lại
/// `selected tab of window id {window}` — chỗ duy nhất còn chữ ấy là `tab_ref`
/// (tay cầm cũ) và `front_window` (hỏi tab đang chọn của cửa sổ trước mặt).
#[test]
fn khong_lenh_nao_go_vao_tab_dang_chon_nua() {
    let src = include_str!("../src/keys.rs");
    let bad: Vec<(usize, &str)> = src
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim_start().starts_with("//"))
        .filter(|(_, l)| l.contains("selected tab of window id {window}"))
        .collect();
    assert!(
        bad.is_empty(),
        "còn lệnh nhắm tab ĐANG CHỌN thay vì tab của phiên: {bad:?}"
    );
    // Đối chứng: bài này đang đọc đúng tệp (tab_ref còn đó).
    assert!(src.contains("whose tty is {}"));
}

/// Đóng một tab KHÔNG được đóng cả cửa sổ đang còn tab của phiên khác.
#[test]
fn dong_tab_khong_dong_ca_cua_so_con_tab_khac() {
    let src = include_str!("../src/keys.rs");
    let start = src.find("fn close_tab(").expect("không thấy close_tab");
    let body = &src[start..start + src[start..].find("\n}\n").unwrap()];
    let dong_cua_so = body
        .find("close window id")
        .expect("close_tab không còn nhánh tab-cuối");
    let dieu_kien = body[..dong_cua_so]
        .rfind("(count of tabs of window id {wid}) is 1")
        .expect("đóng cửa sổ phải nằm SAU điều kiện 'tab cuối'");
    assert!(dieu_kien < dong_cua_so);
    assert!(
        !body.contains("set visible of window"),
        "close_tab không được ẩn cả cửa sổ"
    );
    // `rfind`: bản macOS nằm sau bản shim Windows ở đầu tệp.
    let cw = src.rfind("pub fn close_window(window: i64)").unwrap();
    let cw_body = &src[cw..cw + 600];
    assert!(
        cw_body.contains("return close_tab(window)"),
        "close_window phải rẽ sang close_tab với tay cầm tab"
    );
}
