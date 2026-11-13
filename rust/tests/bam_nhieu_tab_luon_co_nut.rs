//! Quang 13/11 12:06 (ảnh Telegram, phiên `[tfl5]·a2edb96d`, bảng 2 câu): bấm `3`
//! ở câu 1 ⟹ huba trả *"⚠ đã bấm '3' — nhưng KHÔNG đọc ra đang đứng ở câu nào
//! (bảng 2 câu) … Nên tôi KHÔNG gắn nút"*. Quang: *"Lựa chọn nhiều tab vẫn có vấn
//! đề: bấm lần nào xong cũng phải hiện lại luôn danh sách mới bấm được chứ"*.
//!
//! Màn ở đây là bản `/shot` THẬT chụp 05:06:58Z (sau `/tab 2`), câu hỏi là nguyên
//! văn `AskUserQuestion` trong nhật ký phiên. Câu 2 in sạch dưới thanh tab.

use huba::pipeline::{press_ack_from_screen, press_screen_blind};

/// Khung CŨ (câu 1, chưa bấm) còn vệt phía trên khung MỚI — bản đọc 500 ms sau
/// cú bấm. Nhãn và chữ câu 1 lấy nguyên văn bảng thật.
fn man_vet_khung_cu() -> String {
    let cu = "\
←  ☐ Xoá TK – tin  ☐ Deploy PDPD  ✔ Submit  →
│ Tin nhắn P1: khi một người XOÁ TÀI KHOẢN (PDPD, erasure.rs), tin họ đã gửi xử lý thế nào? (Hiện tombstone đổi tên họ thành deleted-<tid>; vỏ sẽ hiện «[đã xoá]».)
❯ 1. Như C8 (Khuyến nghị)
  2. Thu hồi TẤT CẢ
  3. Giữ tất cả
";
    format!("{cu}{MAN}")
}

const MAN: &str = include_str!("fixtures/ask-table-tab2-cau-dai-2026-11-13.txt");

fn hai_cau() -> Vec<String> {
    vec![
        "Tin nhắn P1: khi một người XOÁ TÀI KHOẢN (PDPD, erasure.rs), tin họ đã gửi xử lý thế nào? (Hiện tombstone đổi tên họ thành deleted-<tid>; vỏ sẽ hiện «[đã xoá]».)".into(),
        "PDPD danh tính (fix/erasure-identity-1113: xoá facet/grant/nhật ký khi xoá TK + grant theo username): khi kiểm hoangha cd9eb59 xanh và đã lên main, deploy prod luôn không?".into(),
    ]
}

#[test]
fn bam_cau_1_sang_cau_2_van_co_nut_cua_cau_2() {
    let (ack, nut, _) = press_ack_from_screen(MAN, "3", "[tfl5]", &hai_cau());
    assert!(
        !ack.contains("KHÔNG đọc ra đang đứng ở câu nào"),
        "màn in rõ câu 2 dưới thanh tab: {ack}"
    );
    let ma: Vec<&str> = nut.iter().map(|(m, _)| m.as_str()).collect();
    assert_eq!(ma, vec!["2.1", "2.2", "2.3", "2.4"], "{nut:?}");
}

#[test]
fn vet_khung_cu_phia_tren_khong_lam_mu_cau_dang_mo() {
    let man = man_vet_khung_cu();
    // MẪU SỐ: màn phải chứa HAI thanh tab và chữ cả hai câu, không thì bài này
    // không chặn gì.
    assert_eq!(
        man.lines()
            .filter(|l| l.contains('←') && l.contains('→'))
            .count(),
        2
    );
    assert_eq!(huba::keys::cursor_on(&man, &hai_cau()), Some(1));
    let (ack, nut, _) = press_ack_from_screen(&man, "3", "[tfl5]", &hai_cau());
    let ma: Vec<&str> = nut.iter().map(|(m, _)| m.as_str()).collect();
    assert_eq!(ma, vec!["2.1", "2.2", "2.3", "2.4"], "{ack}");
    // Thanh tab đọc từ khung HIỆN HÀNH: câu 1 đã chốt (☒), còn 1 câu trống.
    let t = huba::keys::ask_table(&man).expect("bảng");
    assert_eq!(t.answered, vec![true, false]);
    assert!(!press_screen_blind(&man, &hai_cau()));
}

/// Mù thật (câu đang mở bị cắt khỏi khung) ⟹ vẫn không đoán câu, nhưng tin PHẢI
/// mang thanh tab để mỗi câu mọc neo ↪ — một chạm là ra danh sách có nút.
#[test]
fn mu_cau_thi_tin_mang_thanh_tab_de_bam_sang_cau() {
    let cut: String = MAN
        .lines()
        .filter(|l| !l.starts_with("│ PDPD"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        press_screen_blind(&cut, &hai_cau()),
        "màn cắt câu phải là MÙ"
    );
    let (ack, nut, _) = press_ack_from_screen(&cut, "3", "[tfl5]", &hai_cau());
    assert!(
        nut.is_empty(),
        "không biết câu thì không gắn nút lựa chọn: {nut:?}"
    );
    let t = huba::keys::ask_table(&ack).expect("tin phải mang thanh tab:\n{ack}");
    assert_eq!(t.answered.len(), 2, "{ack}");
    assert!(ack.contains("Bấm ↪"), "{ack}");
}
