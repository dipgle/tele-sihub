//! Quang 13/11 gõ trần `/type` trên bot Ubuntu ⟹ huba đáp "Chưa hiểu lệnh này" —
//! sai nghĩa: lệnh có thật, chỉ thiếu chữ.
use huba::commands::thieu_tham_so;

#[test]
fn type_tran_noi_dung_thu_thieu() {
    let c = thieu_tham_so("/type").expect("phải nhận ra /type thiếu chữ");
    assert!(c.starts_with("/type cần kèm <chữ>"), "{c}");
    // Bí danh và đuôi @tên_bot cũng vậy.
    assert!(thieu_tham_so("/go").is_some());
    assert!(thieu_tham_so("/type@ai_hubha_bot").is_some());
    assert!(thieu_tham_so("  /TYPE  ").is_some());
}

#[test]
fn khong_dung_ca_khac() {
    // Có chữ ⟹ không phải ca thiếu.
    assert!(thieu_tham_so("/type xin chào").is_none());
    // Lệnh không bắt buộc tham số.
    assert!(thieu_tham_so("/shot").is_none());
    assert!(thieu_tham_so("/session").is_none());
    // Lệnh không có ⟹ vẫn là "chưa hiểu", không bịa cú pháp.
    assert!(thieu_tham_so("/sesion").is_none());
    assert!(thieu_tham_so("type").is_none());
}
