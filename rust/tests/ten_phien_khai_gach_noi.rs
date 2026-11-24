//! Lời khai `[dự-án-làn]` (gạch NỐI thay gạch chéo) phải ra đúng dự án.
//!
//! 🔴 Hà 24/11 12:13 (ảnh `/session`): phiên `85924d1b` hiện `projects-1f·user1-ubuntu` dù mọi
//! lượt nói đều mở bằng `[dwork-audit]`. ĐO trên nhật ký ấy: 33 lượt nói, cả 33 đều khai
//! `[dwork-audit]`; `dwork-audit` KHÔNG là thư mục và không có `/` ⇒ `declared_parts` trả `None`
//! ⇒ không lời khai ⇒ nhãn trung tính, ghim luôn.
//!
//! Luật workspace viết `[<dự-án>/<làn>]`, nhưng phiên viết gạch nối là lỗi quen — huba phải đọc
//! được, vì người nhìn danh sách chỉ thấy một phiên «không có tên».

fn workspace() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("huba-khai-gach-noi-{}", std::process::id()));
    for d in ["alpha", "beta-gamma", "AI/tfl5"] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }
    dir
}

fn khai(text: &str) -> Option<(String, String)> {
    let ws = workspace();
    huba::sessions::declared_parts(text, ws.to_str().unwrap())
}

fn cap(a: &str, b: &str) -> Option<(String, String)> {
    Some((a.to_string(), b.to_string()))
}

/// ① CA CỦA HÀ — `[dwork-audit]` ⇒ dự án `alpha`, làn `audit`.
#[test]
fn gach_noi_sau_ten_du_an_that_la_lan() {
    assert_eq!(khai("[alpha-audit] Đang đọc brief."), cap("alpha", "audit"));
}

/// ② Thư mục THẬT có gạch nối vẫn thắng — không bị cắt nhầm thành `beta` + `gamma`.
#[test]
fn thu_muc_that_co_gach_noi_van_nguyen() {
    assert_eq!(khai("[beta-gamma] xong."), cap("beta-gamma", ""));
    assert_eq!(khai("[beta-gamma-doc] xong."), cap("beta-gamma", "doc"));
}

/// ③ ĐỐI CHỨNG NGƯỢC — phần trước gạch nối KHÔNG phải thư mục ⇒ không đoán.
#[test]
fn gach_noi_sau_ten_la_khong_phai_loi_khai() {
    assert_eq!(khai("[zeta-audit] xong."), None);
    assert_eq!(khai("[alpha-] xong."), None);
    assert_eq!(khai("[-audit] xong."), None);
}

/// ④ Dạng chuẩn không đổi.
#[test]
fn dang_chuan_khong_doi() {
    assert_eq!(khai("[alpha/audit] x"), cap("alpha", "audit"));
    assert_eq!(khai("[alpha] x"), cap("alpha", ""));
    assert_eq!(khai("[tfl5] x"), cap("AI/tfl5", ""));
}
