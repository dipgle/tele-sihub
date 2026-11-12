//! Quang 12/11, ảnh Telegram `[aihub-dev]` bảng 2 câu: *"bấm chọn xong thì ko đc
//! nhưng xem màn thì lại có submit"* · *"Sửa quanh mãi chỗ lựa chọn từ đầu dự án
//! đến giờ chưa xong"*.
//!
//! Bấm câu CUỐI ⟹ CLI sang bước «Review your answers / Ready to submit». Con trỏ
//! không còn nằm trên câu nào nên `cursor_on` = None, và `press_ack_from_screen`
//! đọc None thành "không đo được" ⟹ ⚠ + KHÔNG gắn nút — trong khi `/shot` (qua
//! `multi_question_screen`: hết ô trống ⟹ không còn là bảng nhiều câu) gắn nút
//! đúng. HAI LUẬT cho một màn ⟹ lỗi tái mãi. Bài này khoá MỘT luật.

use huba::pipeline::{multi_question_screen, press_ack_from_screen};

const MAN: &str = include_str!("fixtures/chooser-review-2026-08-18.txt");

fn hai_cau() -> Vec<String> {
    vec!["Đi tiếp thế nào?".into(), "Xử lý sao?".into()]
}

#[test]
fn bam_xong_cau_cuoi_sang_review_van_co_nut_gui() {
    let (ack, nut, _submit) = press_ack_from_screen(MAN, "2", "[x]", &hai_cau());
    assert!(
        !ack.contains("KHÔNG đọc ra đang đứng ở câu nào"),
        "bước Review là trạng thái ĐÃ BIẾT, không phải mù: {ack}"
    );
    let ma: Vec<&str> = nut.iter().map(|(m, _)| m.as_str()).collect();
    assert_eq!(
        ma,
        vec!["1", "2"],
        "nút gửi/huỷ là số trần (đường /key): {nut:?}"
    );
    assert!(nut[0].1.contains("Submit"), "{nut:?}");
    assert!(ack.contains("2/2"), "phải nói đã đủ câu: {ack}");
}

/// Cùng một màn, hai cửa (sau cú bấm và `/shot`) phải ra CÙNG kết luận.
#[test]
fn hai_cua_cung_mot_luat_o_buoc_review() {
    assert!(
        !multi_question_screen(true, MAN),
        "/shot: Review ⟹ không còn là bảng nhiều câu"
    );
    let (_, nut, _) = press_ack_from_screen(MAN, "2", "[x]", &hai_cau());
    assert!(!nut.is_empty(), "sau cú bấm cũng phải có nút như /shot");
}
