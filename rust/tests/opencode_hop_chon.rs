//! Hộp câu hỏi (công cụ `question`) của OPENCODE — Quang 23/11: *«Tk acc8 vẫn chưa chọn được»*.
//!
//! LỖI: 14:02:35 chủ máy bấm `/key 1` cho phiên `[projects]·df9df371` (acc8), huba TỪ CHỐI (`keys_choice_refused`:
//! «màn không có hộp chọn») trong khi `/shot` 14:04:09 cho thấy hộp ngay trên màn. GỐC: `keys::chooser_footer_line` chỉ biết
//! dòng chân của `claude` (`to select … to navigate/cancel`); dòng chân opencode là `↑↓ select  enter submit  esc dismiss`
//! (hộp nhiều câu: `⇆ tab  ↑↓ select  enter confirm  esc dismiss`, trang Submit: `⇆ tab  enter submit  esc dismiss`) — không
//! mảnh nào khớp ⟹ không có dòng chân ⟹ `parse_choices` đòi con trỏ `❯` + dòng liền nhau ⟹ rỗng ⟹ cổng từ chối. Ngoài ra lề
//! `┃` của opencode đứng TRƯỚC số (`┃  1. A`) nên dòng đánh số không đọc được.
//!
//! ĐO TRÊN PHIÊN THỬ (23/11, acc8): phím SỐ một mình chọn VÀ nộp ở hộp một câu; ở hộp nhiều câu số chọn rồi tự sang câu kế,
//! hết câu thì tới trang Submit và cần một Enter. Nên chỉ cần NHẬN RA hộp — đường gửi số của `/key` đã đúng.
//!
//! Màn mẫu: `fixtures/man-hop-cau-hoi-opencode-*-20261123.txt` (một câu = màn THẬT của chủ máy trích từ nhật ký hubd lúc
//! 14:04:09; nhiều câu = chụp phiên thử, đã bỏ dòng trống và cắt ở cột 120).

use huba::keys::{has_chooser_footer, parse_choices};

const MOT_CAU: &str = include_str!("fixtures/man-hop-cau-hoi-opencode-mot-cau-20261123.txt");
const NHIEU_Q1: &str = include_str!("fixtures/man-hop-cau-hoi-opencode-nhieu-cau-q1-20261123.txt");
const NHIEU_Q2: &str = include_str!("fixtures/man-hop-cau-hoi-opencode-nhieu-cau-q2-20261123.txt");
const NHIEU_SUBMIT: &str =
    include_str!("fixtures/man-hop-cau-hoi-opencode-nhieu-cau-submit-20261123.txt");

fn nhan(v: &[(usize, String)]) -> Vec<(usize, &str)> {
    v.iter().map(|(n, l)| (*n, l.as_str())).collect()
}

#[test]
fn the_owners_real_one_question_box_is_recognised_with_its_three_choices() {
    assert!(
        has_chooser_footer(MOT_CAU),
        "dòng chân `↑↓ select  enter submit  esc dismiss` phải được nhận ra"
    );
    assert_eq!(
        nhan(&parse_choices(MOT_CAU)),
        vec![
            (1, "Chờ S9, đưa ngay khi có (Recommended)"),
            (2, "Đưa bản ngay, ghi rõ lệch"),
            (3, "Type your own answer"),
        ],
        "hai lựa chọn có mô tả xen giữa + mục gõ tự do; lề `┃` không được làm hỏng số"
    );
}

#[test]
fn a_multi_question_box_is_recognised_on_each_question_tab() {
    assert!(has_chooser_footer(NHIEU_Q1));
    assert_eq!(
        nhan(&parse_choices(NHIEU_Q1)),
        vec![(1, "A"), (2, "B"), (3, "Type your own answer")]
    );
    assert!(has_chooser_footer(NHIEU_Q2));
    assert_eq!(
        nhan(&parse_choices(NHIEU_Q2)),
        vec![(1, "X"), (2, "Y"), (3, "Type your own answer")]
    );
}

#[test]
fn the_submit_page_has_a_footer_but_no_numbered_choice() {
    // Trang tóm tắt `Chọn A hay B: A` không có mục đánh số ⟹ KHÔNG được bịa lựa chọn để gửi số vào; chỉ cần Enter.
    assert!(has_chooser_footer(NHIEU_SUBMIT));
    assert!(
        parse_choices(NHIEU_SUBMIT).is_empty(),
        "trang Submit không có lựa chọn đánh số: {:?}",
        parse_choices(NHIEU_SUBMIT)
    );
}

#[test]
fn prose_that_merely_mentions_the_key_names_is_not_a_choice_box() {
    // Đối chứng ÂM: câu văn có đủ chữ `enter submit` và `esc dismiss` + một danh sách đánh số liền nhau, nhưng KHÔNG phải dòng chân
    // (không bắt đầu bằng ký hiệu phím `↑↓`/`⇆`). Nhận nhầm = gắn nút vào văn xuôi, và một con số rơi vào ô chat của phiên.
    let man = "  ┃  Các bước:\n  ┃  1. mở tệp\n  ┃  2. sửa dòng\n  ┃  3. lưu lại\n  ┃  Mẹo: enter submit rồi esc dismiss để đóng\n";
    assert!(!has_chooser_footer(man));
    assert!(parse_choices(man).is_empty());
}

#[test]
fn half_a_footer_is_not_a_footer() {
    // Dòng bắt đầu bằng `↑↓` và có `esc dismiss` nhưng THIẾU nửa `enter submit|confirm` ⟹ không phải dòng chân của hộp
    // câu hỏi (dòng chân thật luôn mang cả hai). Đối chứng cho cửa «đủ hai vế».
    let man = "  ┃  1. mở tệp\n  ┃  2. sửa dòng\n  ┃  ↑↓ select  esc dismiss\n";
    assert!(!has_chooser_footer(man));
    assert!(parse_choices(man).is_empty());
}

#[test]
fn a_numbered_list_without_any_footer_stays_prose() {
    // Chống lùi cho luật cũ: không dòng chân + không con trỏ `❯` ⟹ văn xuôi, dù lề là `┃`.
    let man = "  ┃  1. Chờ S9\n  ┃  2. Đưa bản ngay\n";
    assert!(!has_chooser_footer(man));
    assert!(parse_choices(man).is_empty());
}
