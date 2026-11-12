//! Quang 12/11: *"Sao giờ gửi tin vào ô chat nhưng lại không gửi được vào phiên,
//! phải gửi 2 kênh enter mới vào"*. Ca 05:20Z `[tfl5]·39682ce1`: huba gõ 84 ký
//! tự, `still_in_box` trả FALSE ⟹ không bấm Enter bù, báo "đã gửi"; Quang phải
//! tự bấm `enter` hai lần.
//!
//! Gốc (đo trên màn THẬT tab `ttys004` 12/11): Claude Code vẽ viền TRÊN của ô
//! nhập kèm một NHÃN — `─…─ ultracode ─` (171 ký tự kẻ, dòng 182 cột) — nên
//! `is_rule_line` cũ (đòi cả dòng là ký tự kẻ) không nhận ⟹ `box_start` = None.
//! Khung dưới đây chép ĐÚNG byte phần giao diện của màn ấy (viền có nhãn, dấu
//! nhắc `❯` + NBSP, viền dưới 182 ký tự, dòng chân); nội dung hội thoại thay
//! bằng chữ trung tính (repo công khai).

use huba::keys::{box_start, is_rule_line, still_in_box};

fn man(trong_o: &str) -> String {
    format!(
        "● Đã xong lượt trước.\n\n✻ Cogitated for 16s · done 12:26 PM\n\n{} ultracode ─\n❯\u{a0}{trong_o}\n{}\n  ⏵⏵ auto mode on (shift+tab to cycle) · ← for agents\n",
        "─".repeat(170),
        "─".repeat(182)
    )
}

const GOI_Y: &str = "D1–D7 theo đề xuất, làm đi";
const GO: &str =
    "Kiểm lại D3 trên máy A30 rồi chụp ảnh màn hình gửi tôi, xong thì báo kết quả đo nhé";

#[test]
fn vien_co_nhan_van_la_vien() {
    assert!(box_start(&man(GOI_Y)).is_some(), "phải tìm ra ô nhập");
    assert!(is_rule_line(&format!("{} ultracode ─", "─".repeat(170))));
    assert!(is_rule_line(&"─".repeat(20)));
    assert!(!is_rule_line(
        "── chỗ này là một câu văn dài có gạch ngang ở hai đầu ──"
    ));
    assert!(!is_rule_line("Kết quả ────────── xong"));
    assert!(!is_rule_line("───"));
}

#[test]
fn chu_that_trong_o_phai_doc_ra_con_nam_trong_o() {
    assert!(
        still_in_box(&man(GO), GO),
        "chữ THẬT nằm trong ô mà đọc ra 'đã đi'"
    );
}

#[test]
fn o_chi_co_goi_y_thi_chu_da_di() {
    assert!(
        !still_in_box(&man(GOI_Y), GO),
        "ô chỉ còn gợi ý ⟹ chữ đã đi"
    );
}
