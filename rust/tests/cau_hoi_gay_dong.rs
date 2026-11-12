//! Câu hỏi DÀI trên bảng nhiều câu: dòng gãy cũng mang khung `│ `.
//!
//! 🔴 Quang 12/11 (ảnh `[aihub-dev]`): *"Xử lý nốt vụ chọn option nhiều tab đi, bấm
//! hết lượt thì báo ko đọc được, nhưng xem màn thì lại bấm được submit"*. Log 12:19:38Z:
//! bấm `1` ở câu 1 ⟹ *"KHÔNG đọc ra đang đứng ở câu nào (bảng 2 câu)"*, 0 nút. Bước
//! Review đã vá ở `cea273b`; ca này là bước GIỮA.
//!
//! Gốc, đo trên màn THẬT (phiên thử 12/11, CLI 2.1.280, cùng hai câu nguyên văn của phiên
//! `8bcfebfb`): câu ~250 ký tự gãy thành hai dòng, và dòng gãy cũng bắt đầu bằng `│ `:
//!
//! ```text
//! │ Rào «giả danh» (…) Đặc tả và 9 ca thử đã có ở
//! │ aihub-dev/.tmp/brief-mailler-rao-gia-danh.md. Làm theo cách nào?
//! ```
//!
//! `keys::cursor_on` chỉ bỏ KHOẢNG TRẮNG trước khi ghép ⟹ chữ trên màn thành
//! `…đãcóở│aihub-dev…`, không khớp câu trong nhật ký ⟹ `None` ⟹ không nút. Câu ngắn
//! (một dòng, mọi màn mẫu cũ) không bao giờ vấp — nên mọi bài cũ xanh.

use huba::keys::cursor_on;
use huba::pipeline::press_ack_from_screen;

const MAN: &str = include_str!("fixtures/ask-table-cau-dai-gay-dong-2026-11-12.txt");

const Q1: &str = "Rào «giả danh» (so tên hiển thị + thư trong nội dung với tên miền đã xác thực) phải viết vào mã mailler (AI/mailler, crates/smtp-in), là dự án khác, phiên này không được ghi. Đặc tả và 9 ca thử đã có ở aihub-dev/.tmp/brief-mailler-rao-gia-danh.md. Làm theo cách nào?";
const Q2: &str = "Tài khoản ssh không đọc được CSDL thư trên vps-a (/opt/mailler 0750). Để truy ngược cần tiêu đề gốc của thư (chặng Aliyun, chữ ký DKIM, link thật của nút «Thanh Toán Ngay»). Lấy bằng cách nào?";

#[test]
fn man_that_co_dong_gay_mang_khung() {
    // Neo hình dạng của màn mẫu — đổi màn mà mất dòng gãy thì bài dưới hết nghĩa.
    assert!(
        MAN.lines()
            .any(|l| l.starts_with("│ aihub-dev/.tmp/brief-mailler")),
        "màn mẫu không còn dòng gãy mang khung"
    );
}

#[test]
fn cau_dai_gay_dong_van_doc_ra_cau_dang_dung() {
    assert_eq!(cursor_on(MAN, &[Q1.into(), Q2.into()]), Some(0));
}

#[test]
fn nut_mang_ma_cau_khi_cau_dai() {
    let (ack, nut, _) = press_ack_from_screen(MAN, "1", "[thu]", &[Q1.into(), Q2.into()]);
    assert!(!ack.contains("KHÔNG đọc ra"), "{ack}");
    let ma: Vec<&str> = nut.iter().map(|(m, _)| m.as_str()).collect();
    assert!(ma.contains(&"1.1") && ma.contains(&"1.2"), "{ma:?} · {ack}");
}

#[test]
fn doi_chung_cau_khong_co_tren_man_thi_khong_doan() {
    assert_eq!(cursor_on(MAN, &[Q2.into()]), None);
    assert_eq!(cursor_on(MAN, &["Một câu khác hẳn".into()]), None);
}
