//! Phím SỐ vào bảng hỏi NHIỀU câu phải đi PHÍM RỜI — không kèm CR.
//!
//! 🔴 Quang 2026-11-04, ảnh Telegram 14:54: *"Cứ có nhiều tab lựa chọn, lựa xong
//! là bị treo không thể thao tác được gì"*. Ca sống `[bts/check-list]`: bấm `1`
//! ở câu cuối của bảng 3 câu ⟹ màn trống, không ô nhập, CLI đứng
//! `waiting · input needed` mãi.
//!
//! Gốc: `keys::press` đi `do script`, đẩy số + CR vào CÙNG một lượt ghi; CLI
//! 2.1.280 đọc cả hai trong một nhịp nên chốt câu HAI lần (`shouldAdvance` +1
//! hai lần) — câu giữa nhảy cóc, câu cuối vượt trang Submit rồi vẽ `null`. Mẫu
//! số trên sổ: 7/11 bảng nhiều câu đi qua huba kết thúc *"rejected"*.
//!
//! Fixture là màn THẬT của ca ấy lúc 07:52:41Z (đứng ở câu 3, ngay trước cú bấm
//! gây treo), rút gọn phần lời phiên phía trên và gột tên riêng — thanh tab, hộp
//! chọn và dòng chân giữ nguyên văn.

use huba::keys::{key_way, KeyWay};

const BANG_CAU_3: &str = include_str!("fixtures/ask-table-q3-2026-11-04.txt");
const TRANG_SUBMIT: &str = include_str!("fixtures/chooser-review-2026-08-18.txt");

/// Hộp chọn MỘT câu — không thanh tab. Số + CR ở đây vẫn đúng: CLI gửi ngay khi
/// bấm số, CR rơi vào ô nhập rỗng.
const MOT_CAU: &str = "\
⏺ Chọn một hướng:

❯ 1. Hướng A
  2. Hướng B
  3. Type something.

Enter to select · ↑/↓ to navigate · Esc to cancel
";

fn phim(ds: &[&str]) -> Vec<String> {
    ds.iter().map(|s| s.to_string()).collect()
}

/// Đúng cú bấm gây treo: số ở câu cuối của bảng nhiều câu.
#[test]
fn so_o_cau_cuoi_bang_nhieu_cau_di_phim_roi() {
    assert_eq!(
        key_way(BANG_CAU_3, &phim(&["1"])),
        KeyWay::Bare,
        "số + CR một lượt ghi ⟹ CLI chốt hai lần ⟹ vượt trang Submit ⟹ TREO"
    );
}

/// Mọi chữ số 1–9 đều thế, không riêng `1`.
#[test]
fn moi_chu_so_mot_toi_chin_deu_roi() {
    for n in 1..=9 {
        assert_eq!(
            key_way(BANG_CAU_3, &phim(&[&n.to_string()])),
            KeyWay::Bare,
            "số {n}"
        );
    }
}

/// Trang Submit cũng có thanh tab: `1. Submit answers` đi phím rời.
#[test]
fn trang_submit_cung_di_phim_roi() {
    assert_eq!(key_way(TRANG_SUBMIT, &phim(&["1"])), KeyWay::Bare);
}

/// Dãy `/pick`: mũi tên ngang + số.
#[test]
fn day_pick_tren_bang_di_phim_roi() {
    assert_eq!(
        key_way(BANG_CAU_3, &phim(&["left", "left", "2"])),
        KeyWay::Bare
    );
    assert_eq!(key_way(BANG_CAU_3, &phim(&["right", "3"])), KeyWay::Bare);
}

/// 🔵 Đối chứng: hộp MỘT câu giữ đường cũ — nó chạy đúng từ 13/08, đừng đổi.
#[test]
fn hop_mot_cau_giu_duong_cu() {
    assert_eq!(key_way(MOT_CAU, &phim(&["1"])), KeyWay::Written);
}

/// 🔵 Đối chứng: phím KHÁC trên bảng giữ đường cũ — `submit_plan`/`checkbox_plan`
/// được xếp dựa trên cái CR đi kèm. Mũi tên trần thì `/tab` đã lo bằng phím rời.
#[test]
fn phim_khac_tren_bang_giu_duong_cu() {
    for k in ["enter", "esc", "down", "up", "tab", "0", "10", "a"] {
        assert_eq!(
            key_way(BANG_CAU_3, &phim(&[k])),
            KeyWay::Written,
            "phím {k}"
        );
    }
    assert_eq!(
        key_way(BANG_CAU_3, &phim(&["left", "right"])),
        KeyWay::Written
    );
    assert_eq!(
        key_way(BANG_CAU_3, &phim(&["right", "enter"])),
        KeyWay::Written
    );
    assert_eq!(key_way(BANG_CAU_3, &[]), KeyWay::Written);
}

/// 🔵 Đối chứng: lời phiên NHẮC LẠI thanh tab (như chính sổ huba ghi về ca này)
/// mà màn không có hộp chọn nào ⟹ không đổi đường.
#[test]
fn thanh_tab_trong_loi_phien_khong_doi_duong() {
    let man = "\
⏺ Bảng `←  ☒ Gán TK  ☒ Mật khẩu  ☐ Đẩy B1  ✔ Submit  →` treo sau cú bấm thứ ba.

────────────────────────────────────────
❯ \u{a0}
────────────────────────────────────────
";
    assert_eq!(key_way(man, &phim(&["1"])), KeyWay::Written);
}

/// 🔵 Đối chứng: màn TREO (bảng vượt trang Submit, vẽ `null`) không có thanh tab
/// lẫn hộp chọn ⟹ không có gì để chọn bằng số; lối ra là `←` (`/key left`).
#[test]
fn man_treo_khong_co_bang() {
    let man = "\
✻ Sautéed for 44m 43s · done 2:44 PM

❯ Dùng tài khoản thử A, B và C

  Listed 1 directory, ran 3 shell commands






› Message from @projects-2f: [bts] Nhận B1 (chưa đẩy)… (ctrl+o to expand)
";
    assert_eq!(key_way(man, &phim(&["1"])), KeyWay::Written);
}
