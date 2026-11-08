//! GỬI một bảng hỏi NHIỀU CÂU: phím rời, mũi tên tới trang `Submit`, rồi ĐÚNG
//! MỘT cú Enter — không CR nào đi kèm.
//!
//! 🔴 Quang 08/11: *"chọn xong là bị treo, phải Ctrl+C"* trên bảng nhiều tab.
//! Đường cũ đi `submit_plan` → `press_writes`, mà mỗi lượt `do script` mang sẵn
//! một CR không tắt được: mỗi cú Enter của kế hoạch hoá ra HAI cú, câu đang mở
//! bị chốt thêm một lần rồi bảng vượt trang `Submit` và đứng trắng.
//!
//! Bảng THẬT đo được (`tests/fixtures/ask-table-q3-2026-11-04.txt`, phiên
//! `[bts/check-list]`) là câu chọn-MỘT — không có ô `[ ]` nào — nên `has_submit`
//! không nhận nó và `submit_plan` trả `None`, tức trước bản vá đường thật rơi
//! xuống `press(enter)`: cũng `do script`, cũng thừa một CR. Bài kiểm đầu tiên
//! dưới đây giữ đúng con số ấy làm mẫu số.

use huba::keys::{submit_bare_keys, submit_plan};

/// Màn chụp THẬT: ba câu, thanh tab `← ☒ Gán TK ☒ Mật khẩu ☐ Đẩy B1 ✔ Submit →`.
const BANG_3_CAU: &str = include_str!("fixtures/ask-table-q3-2026-11-04.txt");

/// Mẫu số: vì sao bảng thật phải đi phím rời ở CẢ đường gửi.
#[test]
fn the_real_table_is_not_a_checkbox_list_so_the_old_plan_returns_nothing() {
    assert_eq!(
        submit_plan(BANG_3_CAU, 0),
        None,
        "câu chọn-MỘT: `has_submit` đòi ô `[ ]`, nên kế hoạch ba lượt ghi không dựng được \
         — đường cũ rơi về `press(enter)`, đúng cú thừa CR gây treo"
    );
}

#[test]
fn a_multi_question_table_walks_to_submit_and_presses_enter_once() {
    let keys = submit_bare_keys(BANG_3_CAU, 0).expect("bảng 3 câu phải có kế hoạch");
    assert_eq!(
        keys,
        vec!["right", "right", "right", "enter"],
        "đứng ở câu 1 của bảng 3 câu: ba bước ngang mới tới trang Submit"
    );
    // ĐÚNG MỘT Enter, và nó đứng CUỐI: Enter trước khi đi tới `Submit` là chốt
    // lại lựa chọn đang trỏ ở câu đang mở.
    assert_eq!(keys.iter().filter(|k| *k == "enter").count(), 1);
    assert_eq!(keys.last().map(String::as_str), Some("enter"));
    // Không phím DỌC nào: `✔ Submit` là một TAB bên phải (đo 17/08 — đi `↓`
    // tới nó thì con trỏ quấn về mục 1 và cú Enter lật mất một ô).
    assert!(!keys.iter().any(|k| k == "down" || k == "up"));
}

#[test]
fn standing_on_the_last_question_it_takes_a_single_step() {
    assert_eq!(
        submit_bare_keys(BANG_3_CAU, 2).unwrap(),
        vec!["right", "enter"],
        "câu cuối → Submit là một bước"
    );
}

/// 🔵 ĐỐI CHỨNG NGƯỢC: bảng MỘT câu (không thanh tab) giữ nguyên đường cũ.
///
/// Kế hoạch ba lượt ghi đã đo trên hộp thật (`tests/multi_choice_submit.rs`), và
/// đổi nó chỉ để "nhất quán" là bắt một đường đang chạy đúng gánh rủi ro của
/// đường mới.
#[test]
fn a_single_question_box_keeps_the_written_plan() {
    let mot = "\u{276f} 1. [\u{2713}] Không xoá gì\n  2. [ ] Rác build\n     Submit\n\
               Enter to select · ↑/↓ to navigate · Esc to cancel";
    assert_eq!(
        submit_bare_keys(mot, 0),
        None,
        "không có thanh tab ⟹ không phải bảng nhiều câu"
    );
    assert!(
        submit_plan(mot, 0).is_some(),
        "đường cũ vẫn phải dựng được kế hoạch — nó không bị bản vá này đụng tới"
    );
}

/// Dòng chân KHÔNG khai `to navigate` ⟹ không gửi mũi tên nào: mũi tên vừa DI
/// vừa CHỐT ở mọi màn khác, và cú chốt ấy không lùi lại được.
#[test]
fn without_the_navigate_footer_it_refuses_to_send_arrows() {
    let man = "\
←  ☒ Vá ACL  ☐ Đăng nhập  ✔ Submit  →
\u{276f} 1. [ ] A
  2. [ ] B
(không có dòng chân nào)";
    assert_eq!(submit_bare_keys(man, 0), None);
}

/// Thanh tab có hai câu mà mép phải KHÔNG có nút gửi ⟹ đừng bấm Enter: cú ấy
/// sẽ chốt bừa lựa chọn đang trỏ ở tab cuối, thay vì gửi bảng đi.
#[test]
fn without_a_submit_tab_it_does_not_press_enter() {
    let man = "\
←  ☒ Vá ACL  ☐ Đăng nhập  →
\u{276f} 1. A
  2. B
Enter to select · ↑/↓ to navigate · Esc to cancel";
    assert_eq!(submit_bare_keys(man, 0), None);
}
