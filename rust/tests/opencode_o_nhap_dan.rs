//! Ô nhập của OPENCODE giữ khối dán chưa gửi — Quang 23/11: *«Lại lỗi dán vào ô chát không gửi đi»*.
//!
//! LỖI (đo trên nhật ký + màn thật): 14:43:10 huba chạy hộ `git push` của phiên `[dwork/review]` (acc8) rồi dán kết quả vào ô
//! chat; log ghi `runin_paste_khong_do_duoc_nhat_ky … coi là đã gửi`. Màn lúc 14:45:54 (tệp mẫu
//! `fixtures/man-o-nhap-opencode-khoi-dan-chua-gui-20261123.txt`) vẫn giữ `[Pasted ~5 lines]` trong ô, và phiên chỉ trả lời lúc
//! 14:46:54 sau một cú Enter tay.
//!
//! GỐC: `keys::still_in_box` dựa vào `box_start`, mà ô nhập opencode (viền `┃` … `╹▀▀▀`) không có `╭` lẫn vạch ngang + `❯` ⟹
//! `box_start = None` ⟹ `still_in_box` LUÔN nói «chữ đã đi» ⟹ `finish_send` không bao giờ bấm Enter bù. Cú Enter duy nhất nằm
//! trong `type_into` (nghỉ 150 ms sau khi dán) và có thể tới trước lúc opencode xử lý xong khối dán (máy đang tải nặng).
//!
//! ĐO TRÊN PHIÊN THỬ acc8: dán 5 dòng bằng `paste-buffer -p` mà không Enter ⟹ ô hiện đúng `[Pasted ~5 lines]`; MỘT Enter rời ⟹
//! gửi đi, ô sạch. Nên chỉ cần NHẬN RA ô còn giữ chữ để `finish_send` bấm Enter bù.

use huba::keys::{delivery_on_screen, opencode_input_text, still_in_box, Delivered};

const MAN_KET: &str = include_str!("fixtures/man-o-nhap-opencode-khoi-dan-chua-gui-20261123.txt");

/// Khối huba dán vào phiên (5 dòng, như ca thật): `[huba chạy hộ]` · lệnh · trạng thái · đầu ra.
const KHOI_DAN: &str = "[huba chạy hộ]\n$ git -C /mnt/data/projects/dwork/DW-phieu push origin main\n✅ xong (3.0s)\n[stderr] To https://example.invalid/DW-phieu.git\n   a3a274042f..d4ae9c32da  main -> main";

/// Cùng màn nhưng ô nhập TRỐNG — bản đã gửi đi.
fn man_trong() -> String {
    MAN_KET.replace("  ┃  [Pasted ~5 lines]", "  ┃")
}

#[test]
fn the_real_unsent_paste_block_is_seen_as_still_in_the_box() {
    assert!(
        still_in_box(MAN_KET, KHOI_DAN),
        "ô giữ `[Pasted ~5 lines]` ⟹ chữ CHƯA đi"
    );
    assert_eq!(delivery_on_screen(MAN_KET, KHOI_DAN), Delivered::StillInBox);
}

#[test]
fn an_empty_input_box_means_the_text_is_gone() {
    // Đối chứng ngược: cùng màn, ô trống ⟹ đã đi. Không có nó thì «luôn StillInBox» cũng qua được bài trên.
    let man = man_trong();
    assert!(!still_in_box(&man, KHOI_DAN));
    assert_eq!(delivery_on_screen(&man, KHOI_DAN), Delivered::Gone);
}

#[test]
fn a_placeholder_in_the_history_above_does_not_count() {
    // Nhãn nằm trong LỊCH SỬ (dòng không lề `┃`, phía trên chân lượt trả lời), ô nhập trống ⟹ đã đi. Chỉ ô nhập mới tính.
    let man = man_trong().replacen(
        "     Build · DeepSeek V4.1 Flash 1017 · 25.5s",
        "     [Pasted ~5 lines]\n     Build · DeepSeek V4.1 Flash 1017 · 25.5s",
        1,
    );
    assert!(
        man.contains("[Pasted ~5 lines]"),
        "bài kiểm phải thật sự cấy nhãn vào lịch sử"
    );
    assert!(!still_in_box(&man, KHOI_DAN));
}

#[test]
fn a_one_line_text_left_in_the_opencode_box_is_seen_by_its_own_words() {
    let cau = "Kiểm tra giúp tôi trạng thái nhánh đẩy lúc nãy nhé";
    let man = MAN_KET.replace("  ┃  [Pasted ~5 lines]", &format!("  ┃  {cau}"));
    assert!(still_in_box(&man, cau));
    // Chữ KHÁC trong ô ⟹ không phải chữ huba vừa gõ ⟹ không bấm Enter hộ.
    assert!(!still_in_box(
        &man,
        "một câu hoàn toàn khác với câu đang nằm trong ô"
    ));
}

#[test]
fn a_very_short_text_never_counts_as_left_in_the_box() {
    // Luật cũ giữ nguyên: dưới 6 ký tự khớp tình cờ ở khắp nơi.
    let man = MAN_KET.replace("  ┃  [Pasted ~5 lines]", "  ┃  ok");
    assert!(!still_in_box(&man, "ok"));
}

#[test]
fn the_placeholder_only_counts_for_a_multi_line_paste() {
    // Câu MỘT dòng mà ô lại giữ nhãn dán là của lượt trước, không phải của câu này ⟹ không ăn nhầm.
    assert!(!still_in_box(MAN_KET, "một câu một dòng dài hơn sáu ký tự"));
}

#[test]
fn input_text_is_read_from_the_opencode_box_only() {
    let trong = opencode_input_text(MAN_KET).expect("màn phiên opencode có ô nhập");
    assert!(trong.contains("[Pasted ~5 lines]"), "ô nhập: {trong:?}");
    assert!(
        !trong.contains("Build · DeepSeek V4.1 Flash 1017 OpenCode Go"),
        "dòng agent·model không thuộc chữ trong ô: {trong:?}"
    );
    assert!(
        !trong.contains("Hai mục bắt buộc"),
        "lịch sử hội thoại không được lọt vào ô: {trong:?}"
    );
    // Không phải màn opencode ⟹ None (để đường cũ của `claude` xử lý).
    assert_eq!(
        opencode_input_text("╭──────╮\n│ > xin chào │\n╰──────╯\n"),
        None
    );
    assert_eq!(opencode_input_text("$ ls\nfile\n"), None);
    // Trang chào (chưa có hội thoại): viền đáy có nhưng không có dòng `agent · model` ngay trên ⟹ None.
    assert_eq!(
        opencode_input_text("   ┃\n   ┃  Ask anything… \"Fix broken tests\"\n   ┃\n   ╹▀▀▀▀▀▀▀▀\n"),
        None
    );
}
