//! Hòm thư / `/runin`: thư mục chạy phải tìm được cho phiên mở ở GỐC workspace
//! ngay cả khi hàng tra từ sổ theo dõi không mang `cwd` — `pipeline::goc_chay`.
//!
//! Ca thật 23/11 (Mac): sau một lượt khởi động lại hubd, hòm thư của hai phiên
//! (`f28b8137` huba, `17d1d2ff` aihub) chết ở `runin_no_root` — lệnh không chạy,
//! câu trả lời bị tắt tiếng.

use huba::pipeline::goc_chay;
use std::cell::Cell;

#[test]
fn hang_tu_so_khong_co_cwd_thi_doc_anh_chup() {
    let goc = tempfile_dir();
    let hoi = Cell::new(0);
    let r = goc_chay(None, "", || {
        hoi.set(hoi.get() + 1);
        Some(goc.display().to_string())
    });
    assert_eq!(r.as_deref(), Some(goc.as_path()));
    assert_eq!(hoi.get(), 1);
}

#[test]
fn so_hoac_cwd_cua_hang_tra_loi_duoc_thi_khong_chup() {
    let goc = tempfile_dir();
    let r = goc_chay(Some(goc.clone()), "", || panic!("không được chụp"));
    assert_eq!(r.as_deref(), Some(goc.as_path()));
    let r = goc_chay(None, &goc.display().to_string(), || {
        panic!("không được chụp")
    });
    assert_eq!(r.as_deref(), Some(goc.as_path()));
}

#[test]
fn hang_co_cwd_nhung_mat_thu_muc_thi_tu_choi_khong_chup_lai() {
    // Hàng đã đến từ ảnh chụp (có cwd) — chụp lại cũng ra đúng cwd ấy.
    let r = goc_chay(None, "/khong/co/thu-muc/nay", || panic!("không được chụp"));
    assert!(r.is_none());
}

#[test]
fn anh_chup_khong_co_thu_muc_that_thi_tu_choi() {
    assert!(goc_chay(None, "", || None).is_none());
    assert!(goc_chay(None, "", || Some(String::new())).is_none());
    assert!(goc_chay(None, "", || Some("/khong/co/thu-muc/nay".into())).is_none());
}

fn tempfile_dir() -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "huba-runin-goc-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&d).expect("tạo thư mục tạm");
    d
}
