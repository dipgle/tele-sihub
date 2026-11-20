//! Gợi ý MỜ trong ô nhập phải NHÌN RA là gợi ý, vẫn giữ nút gửi (Hà 2026-11-20).
//!
//! Ca thật: phiên hiện một câu gợi ý mờ trong ô nhập, `/shot` vẽ nó như một dòng
//! chữ thường kèm `⏎`, và cả huba lẫn người đọc tưởng chủ máy đã gõ câu ấy mà
//! chưa gửi. Nút ⏎ KHÔNG được bỏ (Hà 16/08 + 25/08: gửi nhanh gợi ý mờ —
//! `/key <phiên> enter` nhận gợi ý rồi mới Enter); chỉ biểu tượng đổi `💡⏎`.
//!
//! Hai chiều: mờ ⟹ `💡⏎`; thật hoặc KHÔNG ĐO ĐƯỢC ⟹ `⏎` như cũ.

use huba::pipeline::{render_session_data, SessionData};

const SCREEN: &str = "⏺ [huba] Xong, đã cài.\n\
                      \n\
                      ────────────────────────────────────────\n\
                      ❯ chạy gate đi\n\
                      ────────────────────────────────────────\n\
                      \x20 ⏵⏵ auto mode on (shift+tab to cycle)";

fn data(ghost: Option<bool>) -> SessionData {
    huba::telegram::set_bot_username("hub_test_bot");
    SessionData {
        sid: "26d92e09-fa3c-4c7c-9c98-b93ef5c3d4ae".into(),
        box_text: Some("chạy gate đi".into()),
        box_ghost: ghost,
        ..Default::default()
    }
}

fn send_link_label(html: &str) -> Option<String> {
    let at = html.find("send_26d92e09")?;
    let rest = &html[at..];
    let from = rest.find('>')? + 1;
    let to = rest.find("</a>")?;
    Some(rest[from..to].to_string())
}

#[test]
fn a_ghost_suggestion_keeps_its_send_link_and_is_marked() {
    let html = render_session_data(SCREEN, &data(Some(true)));
    let label = send_link_label(&html).unwrap_or_else(|| panic!("mất nút gửi:\n{html}"));
    assert!(
        label.starts_with("💡⏎"),
        "gợi ý mờ không được đánh dấu: {label:?}\n{html}"
    );
}

#[test]
fn typed_text_is_not_marked_as_a_suggestion() {
    // ĐỐI CHỨNG NGƯỢC: chữ thật ⟹ biểu tượng cũ, không 💡.
    let html = render_session_data(SCREEN, &data(Some(false)));
    let label = send_link_label(&html).unwrap_or_else(|| panic!("mất nút gửi:\n{html}"));
    assert!(
        label.starts_with('⏎') && !label.contains('💡'),
        "{label:?}\n{html}"
    );
}

#[test]
fn unmeasurable_box_keeps_the_old_icon() {
    // `None` = không đo được (macOS/Windows, ô rỗng, đọc hỏng): KHÔNG đoán là mờ.
    let html = render_session_data(SCREEN, &data(None));
    let label = send_link_label(&html).unwrap_or_else(|| panic!("mất nút gửi:\n{html}"));
    assert!(
        label.starts_with('⏎') && !label.contains('💡'),
        "{label:?}\n{html}"
    );
}
