//! `/new` KHÔNG được gõ đề bài khi màn đang mở một HỘP CHỌN — kể cả hộp không số.
//!
//! Ca thật 15/11 (Ubuntu, `/new acc3 "1+1"`): claude 2.1.326 vẽ hộp tin-thư-mục
//! bằng hai dòng trơn (`❯ No, exit` / `Yes, I trust this folder`). `parse_choices`
//! chỉ đọc `"<số>."` nên trả rỗng, cổng "ô nhập đã sẵn sàng" trả `Yes`, huba gõ
//! đề bài + một CR ⟹ CR chốt «No, exit» ⟹ phiên chết, còn log ghi
//! `session_started`. Màn chụp nguyên văn nằm ở
//! `tests/fixtures/man-hop-trust-khong-so-20261115.txt`.
//!
//! Ba điều phải đo được ở đây: (1) hộp không số chặn việc gõ; (2) hộp CÓ số vẫn
//! chặn như cũ và được báo đúng hình dạng; (3) dòng đỡ `win-<tty>` của
//! `claude agents` KHÔNG được nhận làm `sessionId`.

use huba::sessions::{hop_le_tu_agents, ready_from_screen, BoxShape, Ready};

const MAN_HOP_TRUST: &str = include_str!("fixtures/man-hop-trust-khong-so-20261115.txt");

#[test]
fn hop_khong_so_chan_viec_go_de_bai() {
    assert_eq!(
        ready_from_screen(MAN_HOP_TRUST),
        Ready::Blocked {
            options: 2,
            shape: BoxShape::Unnumbered,
        },
        "hộp không số mà đọc ra `Yes` là đúng con bug 15/11"
    );
}

#[test]
fn hop_co_so_van_chan_va_bao_dung_hinh_dang() {
    let co_so = "Chọn cách đi tiếp?\n❯ 1. Sửa tại chỗ\n  2. Mở phiên mới\n\n Enter to confirm · Esc to cancel";
    assert_eq!(
        ready_from_screen(co_so),
        Ready::Blocked {
            options: 2,
            shape: BoxShape::Numbered,
        }
    );
}

#[test]
fn man_khong_hop_thi_go_duoc() {
    // Văn xuôi + ô nhập — trạng thái bình thường của một phiên vừa dựng xong.
    let man = "✻ Chào bạn\n\n❯ \n? for shortcuts";
    assert_eq!(ready_from_screen(man), Ready::Yes);
}

/// Dòng `win-ttysNNN` là CỬA SỔ, không phải phiên.
#[test]
fn dong_do_win_tty_khong_phai_id_phien() {
    assert!(
        !hop_le_tu_agents("ttys005", "win-ttys005", "ttys005"),
        "dòng đỡ là cửa sổ, không phải phiên"
    );
    assert!(hop_le_tu_agents(
        "ttys005",
        "6f5e4c0a-1111-2222-3333-444455556666",
        "ttys005"
    ));
    // tty khác / rỗng: dòng ấy không nói gì về cửa sổ vừa mở.
    assert!(!hop_le_tu_agents("ttys006", "6f5e4c0a-1111", "ttys005"));
    assert!(!hop_le_tu_agents("", "6f5e4c0a-1111", "ttys005"));
}
