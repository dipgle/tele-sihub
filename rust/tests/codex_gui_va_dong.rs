//! Quang 15/11: chữ nằm ô Codex, bàn giao sinh thêm phiên mà không đóng cũ.
use huba::keys::{box_inner, input_box_text, landed, still_in_box, Landed};

const MAN: &str = include_str!("fixtures/codex-input-2026-11-15.txt");
const CHU: &str = "tại sao sô lượng phiên càng ngày càng tăng lên thế, mớ phiên kế nhiệm không đóng phiên cũ à\nchỉnh cho tôi tài khoản acc9 giống acc8 đi\nsao chạy huba xong dán vào ô chat đứng ở đó luôn không enter để gửi\ngửi từ tele xong cũng chỉ vào ô chat không gửi đi\nđang có chuyện gì vậy?";

#[test]
fn man_that_codex_chu_chua_gui_thi_phai_bam_enter() {
    assert!(still_in_box(MAN, CHU));
    assert_eq!(landed(MAN, CHU), Landed::InBox);
    let input = input_box_text(MAN).expect("ô nhập Codex thật");
    assert!(input.starts_with("tại sao"));
    assert!(input.ends_with("đang có chuyện gì vậy?"));
    assert!(!input.contains("GPT-6.1"));
    assert!(!input.contains("229 kiểm tra"));
}

#[test]
fn codex_gui_roi_khong_gui_lai_hoi_thoai_hay_placeholder() {
    let sent = format!("› {CHU}\n• Đã nhận\n› Ask Codex to do anything\n  GPT-6.1-Sol high · ~/projects/huba · Main [default]\n  ? for shortcuts\n");
    assert!(!still_in_box(&sent, CHU));
    assert!(input_box_text(&sent).is_none());
    let shell = "host % echo '› đây là chữ đầu ra'\n› đây là chữ đầu ra\nhost % ";
    assert!(box_inner(shell).is_none());
    // Dấu › trong tin của phiên khác không biến bảng Claude thành ô Codex.
    let quoted = "› Message from @projects-1e: [tfl5] bàn giao\n❯ 1. Đồng ý\n  2. Từ chối";
    assert!(box_inner(quoted).is_none());
    let rule = "─".repeat(60);
    let claude =
        format!("{sent}\nđó là ảnh Codex\n{rule}\n❯ tin mới của Claude\n{rule}\n⏵⏵ auto mode on\n");
    assert!(
        !still_in_box(&claude, CHU),
        "không lấy ô Codex trong ảnh được trích lại"
    );
    assert!(still_in_box(&claude, "tin mới của Claude"));
}

#[test]
fn codex_tin_ngan_va_nhan_dan_nhieu_dong_deu_chua_gui() {
    for typed in ["ok", "ừ", "/quit", "nội dung nhiều dòng\nkết quả"] {
        let shown = if typed.contains('\n') {
            "[Pasted Content 1234 chars]"
        } else {
            typed
        };
        let screen = format!(
            "• Đang chờ\n› {shown}\n  GPT-6.1-Sol high · ~/projects/huba · Main [default]\n"
        );
        assert!(still_in_box(&screen, typed), "{typed}");
    }
}

#[test]
fn ca_hai_duong_gui_dung_cung_cong_chon_enter_theo_cli() {
    let src = include_str!("../src/keys.rs");
    // `a834fbf` tách thân `type_and_send` sang `finish_send` — đường gửi nằm ở đó.
    let typed = &src[src.find("pub fn type_and_send(").unwrap()..];
    let typed = &typed[..typed.find("\n}\n").unwrap()];
    assert!(typed.contains("finish_send(window, text)"));
    assert!(!typed.contains("press(window, \"enter\")"));
    let body = &src[src.find("pub fn finish_send(").unwrap()..];
    let body = &body[..body.find("\n}\n").unwrap()];
    assert!(body.contains("enter_after_text(window, &screen)"));
    assert!(!body.contains("press(window, \"enter\")"));
    let src = include_str!("../src/pipeline.rs");
    let start = src.find("let mut extra_enter = 0u32;").unwrap();
    let end = start + src[start..].find("if extra_enter > 0").unwrap();
    assert!(src[start..end].contains("enter_after_text(w, body)"));
    assert!(!src[start..end].contains("keys::press(w, \"enter\")"));
}

#[test]
fn chi_codex_doi_sang_enter_roi_claude_giu_nguyen() {
    use huba::keys::{text_enter_way, KeyWay};
    assert_eq!(text_enter_way(MAN), KeyWay::Bare);
    let rule = "─".repeat(60);
    let claude = format!("{rule}\n❯ cùng một câu gửi đi\n{rule}\n⏵⏵ auto mode on\n");
    assert_eq!(text_enter_way(&claude), KeyWay::Written);
    assert!(still_in_box(&claude, "cùng một câu gửi đi"));
    assert!(!still_in_box(
        &claude.replace("cùng một câu gửi đi", "ok"),
        "ok"
    ));
    assert_eq!(
        text_enter_way("host % echo hello\nhost % "),
        KeyWay::Written
    );
    assert_eq!(text_enter_way(""), KeyWay::Written);
    let quoted = format!("{MAN}\nđó là ảnh được trích lại\n{claude}");
    assert_eq!(text_enter_way(&quoted), KeyWay::Written);
}

#[test]
fn ban_giao_codex_khong_return_giu_cu_truoc_khi_do_id() {
    let src = include_str!("../src/sessions.rs");
    let start = src.find("pub fn start_fresh_after_handover(").unwrap();
    let end = start + src[start..].find("\n}\n").unwrap();
    let body = &src[start..end];
    let codex = body.find("open_codex_window(").unwrap();
    let guard = body.find("if new_id.is_none()").unwrap();
    assert!(!body[codex..guard].contains("return Ok(FreshWindow"));
    assert!(body[codex..guard].contains("wait_for_codex_session_id("));
    assert!(body[guard..].contains("quit_and_close"));
}

#[test]
fn id_ke_nhiem_phai_moi_dung_tty_dung_acc_va_duy_nhat() {
    use huba::sessions::{codex_successor_on_tty, LiveSession};
    let new = LiveSession {
        session_id: "new".into(),
        account: "acc10".into(),
        tty: "ttys016".into(),
        kind: "codex".into(),
        started_at_ms: 2000,
        ..Default::default()
    };
    let select = |rows: &[LiveSession]| {
        codex_successor_on_tty(rows, "/dev/ttys016", "acc10", "old", 1500).map(str::to_string)
    };
    assert_eq!(select(std::slice::from_ref(&new)).as_deref(), Some("new"));
    for bad in [
        LiveSession {
            session_id: "old".into(),
            ..new.clone()
        },
        LiveSession {
            session_id: "".into(),
            ..new.clone()
        },
        LiveSession {
            tty: "ttys005".into(),
            ..new.clone()
        },
        LiveSession {
            tty: "??".into(),
            ..new.clone()
        },
        LiveSession {
            account: "acc8".into(),
            ..new.clone()
        },
        LiveSession {
            kind: "foreground".into(),
            ..new.clone()
        },
        LiveSession {
            started_at_ms: 1000,
            ..new.clone()
        },
    ] {
        assert_eq!(select(&[bad]), None);
    }
    assert_eq!(select(&[]), None);
    let second = LiveSession {
        session_id: "another".into(),
        ..new.clone()
    };
    assert_eq!(select(&[new, second]), None, "mơ hồ thì giữ phiên cũ");
}
