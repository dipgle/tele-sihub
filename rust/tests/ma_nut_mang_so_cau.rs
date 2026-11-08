//! Mã của ☑ trong chữ phải mang ĐÚNG số câu đang mở — bấm ở tab 2 thì trả lời
//! câu 2, không được lùi về tab 1.
//!
//! 🔴 Quang 08/11: *"rõ ràng là bấm 2 ở tab 2 sao lại nhảy về tab 1"*.
//!
//! Đo được trong `logs/huba.log` (phiên `74451d4a`, cửa sổ 85760):
//!
//! ```text
//! 02:52:23  head=/start pick_74451d4a_1_1   → pick_sent keys:["1"]          (đúng)
//! 02:53:00  head=/start tab_74451d4a_2      → tin dựng lại từ màn của CÂU 2
//! 02:53:34  head=/start pick_74451d4a_1_2   → pick_sent {"from":1,"to":0,"keys":["left","2"]}
//!           …rồi ack: "gửi phím nhưng bảng KHÔNG đổi (vẫn 2 câu trống)"
//! ```
//!
//! Cú bấm thứ ba nằm dưới lựa chọn của **câu 2** trên màn, mà mã mang số **1**:
//! `pick_answer` đọc con trỏ (đang ở câu 2), thấy đích là câu 1, nên bấm `←`
//! trước rồi mới gõ `2` — đúng cái mắt anh thấy. Gốc: nhánh `/shot` và `/tab`
//! dựng mã bằng `format!("1.{n}")` GÕ CỨNG, dù ngay trên dòng ấy đã đo được câu
//! đang mở (`dang_o`) và chỉ dùng nó để in dấu *"◀ ĐANG MỞ"*.
//!
//! Bài này chấm ĐẦU RA của `pipeline::choice_codes` — đúng chuỗi đi vào mã nút
//! `pick_<sid>_<câu>_<n>`. Chấm "có gọi `cursor_on` không" là chấm cách viết.

use huba::pipeline::{choice_codes, current_question};
use huba::sessions::{Asking, Question};

/// Bốn lựa chọn đúng như `keys::parse_choices` đọc ra từ màn thật của bảng
/// `AskUserQuestion` (kể cả hai mục do CLI tự thêm).
fn bon_lua_chon() -> Vec<(usize, String)> {
    vec![
        (1, "Lựa chọn 1 (câu 2)".to_string()),
        (2, "Lựa chọn 2 (câu 2)".to_string()),
        (3, "Type something.".to_string()),
        (4, "Chat about this".to_string()),
    ]
}

/// Màn THẬT lúc con trỏ đứng ở **câu 2** của bảng ba câu (chép từ ảnh Quang gửi
/// 08/11, giữ nguyên thanh tab và dấu `❯`).
const MAN_CAU_2: &str = "\
────────────────────────────────────────────────
←  ☒ Thử ①  ☐ Thử ②  ☐ Thử ③  ✔ Submit  →
Lượt thử 2 — câu 2: chọn lựa chọn nào?
❯ 1. Lựa chọn 1 (câu 2)
     Bấm nút này cho câu 2.
  2. Lựa chọn 2 (câu 2)
     Hoặc nút này.
  3. Type something.
  4. Chat about this
Enter to select · Tab/Arrow keys to navigate · Esc to cancel";

/// Cùng bảng ấy, màn lúc còn ở **câu 1** — vế đối chứng ngược của phép đo.
const MAN_CAU_1: &str = "\
────────────────────────────────────────────────
←  ☐ Thử ①  ☐ Thử ②  ☐ Thử ③  ✔ Submit  →
Lượt thử 2 — câu 1: chọn lựa chọn nào?
❯ 1. Lựa chọn 1 (câu 1)
     Bấm nút này cho câu 1.
  2. Lựa chọn 2 (câu 1)
     Hoặc nút này.
  3. Type something.
  4. Chat about this
Enter to select · Tab/Arrow keys to navigate · Esc to cancel";

/// Bảng ba câu, đọc từ nhật ký — nguồn của `cursor_on`.
fn bang_ba_cau() -> Asking {
    Asking {
        header: "Thử ①".to_string(),
        question: "Lượt thử 2 — câu 1: chọn lựa chọn nào?".to_string(),
        options: vec![
            "Lựa chọn 1 (câu 1)".to_string(),
            "Lựa chọn 2 (câu 1)".to_string(),
        ],
        multi: false,
        rest: vec![
            Question {
                header: "Thử ②".to_string(),
                question: "Lượt thử 2 — câu 2: chọn lựa chọn nào?".to_string(),
                options: vec![
                    "Lựa chọn 1 (câu 2)".to_string(),
                    "Lựa chọn 2 (câu 2)".to_string(),
                ],
                multi: false,
            },
            Question {
                header: "Thử ③".to_string(),
                question: "Lượt thử 2 — câu 3: chọn lựa chọn nào?".to_string(),
                options: vec![
                    "Lựa chọn 1 (câu 3)".to_string(),
                    "Lựa chọn 2 (câu 3)".to_string(),
                ],
                multi: false,
            },
        ],
    }
}

fn ma(v: &[(String, String)]) -> Vec<String> {
    v.iter().map(|(m, _)| m.clone()).collect()
}

/// 🎯 Bài chính: đứng ở câu 2 ⟹ mã phải là `2.<n>`.
#[test]
fn bam_o_tab_2_thi_ma_mang_cau_2() {
    let a = bang_ba_cau();
    let dang_o = current_question(Some(&a), MAN_CAU_2).expect("màn câu 2 phải đo được");
    assert_eq!(
        dang_o, 1,
        "con trỏ đang ở câu 2 (0-based), không phải câu 1"
    );
    let got = choice_codes(&bon_lua_chon(), true, Some(dang_o));
    assert_eq!(
        ma(&got),
        vec!["2.1", "2.2", "2.3", "2.4"],
        "mã phải mang số câu ĐANG MỞ; mã `1.*` là cú lùi về tab 1 mà Quang đã thấy: {got:?}"
    );
    // Và nhãn phải đi theo đúng lựa chọn của nó — mã đúng mà nhãn lệch thì cú
    // bấm vẫn rơi vào chỗ khác.
    assert_eq!(got[1].1, "Lựa chọn 2 (câu 2)");
}

/// 🔁 ĐỐI CHỨNG NGƯỢC (một): cùng hàm, màn câu 1 ⟹ mã `1.<n>`.
///
/// Không có vế này thì một hàm luôn trả `2.` cũng làm bài trên xanh.
#[test]
fn man_cau_1_thi_ma_mang_cau_1() {
    let a = bang_ba_cau();
    let dang_o = current_question(Some(&a), MAN_CAU_1).expect("màn câu 1 phải đo được");
    assert_eq!(dang_o, 0);
    let got = choice_codes(&bon_lua_chon(), true, Some(dang_o));
    assert_eq!(ma(&got), vec!["1.1", "1.2", "1.3", "1.4"], "{got:?}");
}

/// 🔴 Bảng nhiều câu mà KHÔNG đo được câu đang mở ⟹ KHÔNG mã nào.
///
/// Đây là vế fail-closed, cùng luật với `press_ack_from_screen`: một mã đoán sai
/// ghi đè lên câu ĐÃ CHỐT, và việc đó không lùi lại được. Đường bấm vẫn còn —
/// `ask_command_lines` liệt kê `/pick_<câu>_<n>` ở cuối tin.
///
/// Đối chứng ngược nằm ngay trong bài: cùng `None`, bảng MỘT câu vẫn phải ra số
/// trần — nếu không thì luật này ăn mất hộp một câu, thứ chiếm phần lớn các ca.
#[test]
fn khong_do_duoc_cau_thi_khong_gan_ma() {
    assert!(
        choice_codes(&bon_lua_chon(), true, None).is_empty(),
        "không biết đang ở câu nào mà vẫn dựng mã ⟹ cú bấm sẽ ghi đè câu đã chốt"
    );
    let mot_cau = choice_codes(&bon_lua_chon(), false, None);
    assert_eq!(
        ma(&mot_cau),
        vec!["1", "2", "3", "4"],
        "hộp MỘT câu đi bằng `k_`, mã là số trần: {mot_cau:?}"
    );
}

/// Màn không có thanh tab ⟹ `current_question` nói CHƯA ĐO ĐƯỢC, không đoán.
#[test]
fn man_khong_thanh_tab_thi_chua_do_duoc() {
    let mot_cau = "\
Do you want to make this edit to billing.rs?
❯ 1. Yes
  2. No, tell Claude what to do differently

Enter to confirm · Esc to cancel";
    assert_eq!(
        current_question(Some(&bang_ba_cau()), mot_cau),
        None,
        "không thấy câu nào của bảng trên màn ⟹ KHÔNG phải `Some(0)`"
    );
    assert_eq!(
        current_question(None, MAN_CAU_2),
        None,
        "không có bảng ⟹ None"
    );
}
