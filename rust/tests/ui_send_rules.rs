//! K4 — luật TỪ CHỐI của đường gửi từ giao diện (`ui_send::refuse_reason`, hàm thuần).
//!
//! Gõ vào phiên là thao tác không lùi lại được, nên mặc định là TỪ CHỐI và nói vì sao; chỉ phiên Claude có cửa sổ
//! Terminal thật, đang ở dấu nhắc bình thường, mới được nhận. Mỗi luật có một ca ĐỐI CHỨNG: phiên lành + tin lành ⟹ nhận.

use huba::sessions::{Asking, LiveSession};
use huba::ui_send::{refuse_reason, MAX_CHARS};

fn lanh() -> LiveSession {
    LiveSession {
        session_id: "0123abcd-0000-4000-8000-000000000001".into(),
        kind: "interactive".into(),
        host: "terminal".into(),
        tty: "ttys003".into(),
        ..Default::default()
    }
}

#[test]
fn a_healthy_claude_terminal_session_takes_a_normal_message() {
    assert_eq!(refuse_reason(&lanh(), "chào phiên, làm tiếp việc đang dở"), None);
    // nhiều dòng + tab vẫn là tin thường
    assert_eq!(refuse_reason(&lanh(), "dòng 1\n\tdòng 2"), None);
}

#[test]
fn an_empty_or_blank_message_is_refused() {
    for t in ["", "   ", "\n\n\t "] {
        assert!(refuse_reason(&lanh(), t).is_some(), "{t:?}");
    }
}

#[test]
fn a_too_long_message_is_refused_but_the_limit_itself_passes() {
    assert!(refuse_reason(&lanh(), &"a".repeat(MAX_CHARS)).is_none());
    assert!(refuse_reason(&lanh(), &"a".repeat(MAX_CHARS + 1)).is_some());
    // đếm KÝ TỰ, không đếm byte: chữ có dấu không bị tính gấp
    assert!(refuse_reason(&lanh(), &"ế".repeat(MAX_CHARS)).is_none());
}

#[test]
fn control_characters_are_refused() {
    for t in ["a\u{0}b", "a\u{1b}[31mb", "a\rb", "a\u{7}b"] {
        assert!(refuse_reason(&lanh(), t).is_some(), "{t:?}");
    }
}

#[test]
fn a_leading_slash_would_run_as_a_claude_command_so_it_is_refused() {
    assert!(refuse_reason(&lanh(), "/exit").is_some());
    assert!(refuse_reason(&lanh(), "  /clear").is_some());
    // đối chứng: dấu «/» ở GIỮA câu (đường dẫn) là chữ thường
    assert!(refuse_reason(&lanh(), "xem tệp /tmp/a.txt giúp").is_none());
}

#[test]
fn only_interactive_terminal_sessions() {
    for (kind, host) in [
        ("background", "background"),
        ("interactive", "editor"),
        ("interactive", "detached"),
        ("interactive", "dead"),
        ("interactive", "shell"),
    ] {
        let mut s = lanh();
        s.kind = kind.into();
        s.host = host.into();
        assert!(
            refuse_reason(&s, "x").is_some(),
            "kind={kind} host={host} lẽ ra bị từ chối"
        );
    }
}

#[test]
fn a_session_without_a_real_tty_is_refused() {
    for tty in ["", "??", "?"] {
        let mut s = lanh();
        s.tty = tty.into();
        assert!(refuse_reason(&s, "x").is_some(), "tty={tty:?}");
    }
}

#[test]
fn opencode_sessions_are_not_served_yet() {
    let mut s = lanh();
    s.session_id = "ses_e1dc8e2b".into();
    let why = refuse_reason(&s, "x").expect("opencode lẽ ra bị từ chối");
    assert!(why.to_lowercase().contains("opencode"), "{why}");
}

#[test]
fn a_session_that_is_asking_something_is_refused() {
    // câu hỏi trong nhật ký
    let mut s = lanh();
    s.asking = Some(Asking {
        header: "H".into(),
        question: "Q?".into(),
        options: vec!["a".into(), "b".into()],
        ..Default::default()
    });
    assert!(refuse_reason(&s, "x").is_some());
    // hộp chọn đang hiện trên màn
    let mut s = lanh();
    s.screen_choices = 3;
    assert!(refuse_reason(&s, "x").is_some());
    // ô mật khẩu
    let mut s = lanh();
    s.asking_password = true;
    assert!(refuse_reason(&s, "x").is_some());
}

#[test]
fn a_blocked_account_is_refused() {
    let mut s = lanh();
    s.limited = Some("hết hạn mức".into());
    assert!(refuse_reason(&s, "x").is_some());
    let mut s = lanh();
    s.account_dead = Some("đăng nhập hết hạn".into());
    assert!(refuse_reason(&s, "x").is_some());
}

/// Câu từ chối phải NÓI RA lý do bằng chữ người đọc được (không rỗng, không mã).
#[test]
fn every_refusal_says_why() {
    let mut s = lanh();
    s.host = "editor".into();
    let why = refuse_reason(&s, "x").unwrap();
    assert!(why.chars().count() > 15, "{why}");
}
