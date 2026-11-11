//! Tin gim "đang theo phiên nào" bị GỠ GIM thì vòng nền phải gim lại.
//!
//! 🔴 Quang 2026-11-11: *"mất gim tin nhắn thông tin phiên đang theo rồi ko tự
//! tạo lại?"*. Log: tin 77628 vẫn sửa được (`telegram_message_edited` 07:20–
//! 07:21) ⟹ tin còn sống ⟹ `pin_is_gone` không bao giờ đúng ⟹ không cửa dựng
//! lại nào chạy. Chỉ `getChat` nói được tin còn nằm trên đỉnh hay không.

#[test]
fn tin_trong_so_van_tren_dinh_thi_khong_phai_bi_go() {
    assert!(!huba::pipeline::pin_bi_go(77628, Some(77628)));
}

#[test]
fn dinh_trong_hoac_dinh_la_tin_khac_thi_la_bi_go() {
    assert!(huba::pipeline::pin_bi_go(77628, None));
    assert!(huba::pipeline::pin_bi_go(77628, Some(77001)));
}

#[test]
fn vong_nen_hoi_dinh_buong_va_gim_lai() {
    let src = include_str!("../src/pipeline.rs");
    let start = src
        .find("pub fn refresh_pin(")
        .expect("không thấy refresh_pin");
    let rest = &src[start..];
    let end = rest[1..]
        .find("\npub fn ")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    let body = &rest[..end];
    // Đối chứng: đang nhìn đúng thân hàm (nó vẫn gọi pin_apply ở cuối).
    assert!(
        body.contains("pin_apply("),
        "không đọc đúng thân refresh_pin"
    );
    assert!(
        body.contains("pinned_message_id()"),
        "refresh_pin KHÔNG hỏi Telegram tin nào đang trên đỉnh — gỡ gim thì không ai gim lại"
    );
    assert!(
        body.contains("pin_bi_go(") && body.contains("tg.pin("),
        "refresh_pin phải so sổ với đỉnh buồng (pin_bi_go) rồi gim lại (tg.pin)"
    );
}
