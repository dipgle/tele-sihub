//! `/effort` — phần THUẦN của `huba::effort`, chấm trên MÀN THẬT.
//!
//! Mọi màn dưới đây chép từ cửa sổ Terminal thật lúc đo 2026-11-10 (`claude`
//! 2.1.280, Sonnet 5, acc6, cwd `~/projects`): chỉ rút ngắn viền và dòng dài —
//! không sửa chữ nào nằm giữa. Mỗi phép chấm có VẾ NGƯỢC trong cùng bài: một
//! phép đo không bao giờ ra kết quả ngược lại thì không phải phép đo.

use huba::effort::{
    applied_marks, command_text, confirm_dialog, first_move, parse_level, probe_verdict, FirstMove,
    Outcome, ProbeSays, LEVELS,
};

const RULE: &str = "────────────────────────────────────────────────────────────";

fn screen(lines: &[&str]) -> String {
    lines.join("\n")
}

/// Rảnh, vừa gõ `/effort high` lúc phiên chưa có lịch sử — chạy ngay.
fn idle_applied_high() -> String {
    screen(&[
        "❯ /effort high",
        "  ⎿  Set effort level to high (saved as your default for new sessions): Comprehensive implementation with extensive testing and documentation",
        "                                                     ● high · /effort",
        RULE,
        "❯ ",
        RULE,
        "  ⏵⏵ auto mode on (shift+tab to cycle) · ← for agents",
    ])
}

/// ĐANG BẬN (đang in số 1123), gõ `/effort high` — dòng xác nhận hiện canh
/// phải ngay trên ô, lượt vẫn chạy (`esc to interrupt`).
fn busy_applied_high() -> String {
    screen(&[
        "  1122",
        "  1123",
        "                                                                    Set effort level to high (saved as your default for new sessions): Comprehensive implementation with e",
        RULE,
        "❯ ",
        RULE,
        "  ⏵⏵ auto mode on (shift+tab to cycle) · esc to interrupt · ← for agents",
    ])
}

/// Rảnh, phiên ĐÃ có lịch sử, gõ `/effort medium` — hộp xác nhận (cửa sổ 80 cột).
fn confirm_medium() -> String {
    screen(&[
        "✻ Cooked for 2s · done 8:17 AM",
        "▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔",
        "   Change effort level?",
        "   Your next response will be slower and use more tokens",
        "   This conversation is cached for the current effort level. Switching to",
        "   medium means the full history gets re-read on your next message.",
        "   ❯ 1. Yes, switch to medium",
        "     2. No, go back",
    ])
}

fn box_with(lines: &[&str], footer: &str) -> String {
    let mut v = vec!["⏺ ok", "✻ Cooked for 2s · done 8:17 AM", RULE];
    v.extend_from_slice(lines);
    v.push(RULE);
    v.push(footer);
    screen(&v)
}

const IDLE_FOOTER: &str = "  ⏵⏵ auto mode on (shift+tab to cycle)";
const BUSY_FOOTER: &str =
    "  ⏵⏵ auto mode on (shift+tab to cycle) · esc to interrupt · ← for agents";

#[test]
fn parse_level_nhan_du_nam_muc_va_tu_choi_muc_la() {
    for l in LEVELS {
        assert_eq!(parse_level(l), Ok(l));
        assert_eq!(parse_level(&format!("  {} ", l.to_uppercase())), Ok(l));
    }
    // Vế ngược: mức không có ⟹ lỗi, và lỗi kể đủ mức để người gõ chọn lại.
    for bad in ["ultra", "", "hi", "xhigh2", "auto"] {
        let e = parse_level(bad).expect_err(bad);
        for l in LEVELS {
            assert!(e.contains(l), "lỗi cho {bad:?} phải kể mức {l}: {e}");
        }
    }
    assert_eq!(command_text("xhigh"), "/effort xhigh");
}

#[test]
fn applied_marks_dem_dong_xac_nhan_ca_luc_ranh_lan_luc_ban() {
    assert_eq!(applied_marks(&idle_applied_high(), "high"), 1);
    assert_eq!(applied_marks(&busy_applied_high(), "high"), 1);
    // Vế ngược: cùng màn, mức khác ⟹ 0 — kể cả `xhigh`, chữ CHỨA `high`.
    for other in ["xhigh", "medium", "low", "max"] {
        assert_eq!(applied_marks(&idle_applied_high(), other), 0, "{other}");
        assert_eq!(applied_marks(&busy_applied_high(), other), 0, "{other}");
    }
    // Và chiều kia của cái bẫy chuỗi con: màn đổi sang `xhigh` không được đếm là `high`.
    let x = "  ⎿  Set effort level to xhigh (saved as your default for new sessions)";
    assert_eq!(applied_marks(x, "xhigh"), 1);
    assert_eq!(applied_marks(x, "high"), 0);
    // Màn trước khi gõ gì: 0 — đây là mẫu số của phép so trước/sau.
    assert_eq!(applied_marks(&box_with(&["❯ "], IDLE_FOOTER), "high"), 0);
}

#[test]
fn applied_marks_van_dem_khi_cua_so_hep_be_dong() {
    // Cửa sổ 80 cột bẻ câu xác nhận — đúng như màn thật lúc đo `medium`.
    let s = screen(&[
        "❯ /effort medium",
        "  ⎿  Set effort level to medium (saved as your default for new sessions):",
        "     Balanced approach with standard implementation and testing",
    ]);
    assert_eq!(applied_marks(&s, "medium"), 1);
    let be_giua = "  ⎿  Set effort level to\n     medium (saved as your default)";
    assert_eq!(applied_marks(be_giua, "medium"), 1);
    assert_eq!(applied_marks(be_giua, "high"), 0);
}

#[test]
fn confirm_dialog_chi_nhan_hop_xin_dung_muc_vua_ra_lenh() {
    assert!(confirm_dialog(&confirm_medium(), "medium"));
    // Vế ngược: hộp xin `medium` mà lệnh là mức khác ⟹ KHÔNG bấm.
    for other in ["low", "high", "xhigh", "max"] {
        assert!(!confirm_dialog(&confirm_medium(), other), "{other}");
    }
    // Không có tiêu đề hộp ⟹ không phải hộp xác nhận, dù có dòng giống.
    let gia = "   ❯ 1. Yes, switch to medium\n     2. No, go back";
    assert!(!confirm_dialog(gia, "medium"));
    assert!(!confirm_dialog(&idle_applied_high(), "high"));
}

#[test]
fn first_move_o_trong_thi_go_sach_ca_luc_ranh_lan_luc_ban() {
    assert_eq!(
        first_move(&box_with(&["❯ "], IDLE_FOOTER)),
        FirstMove::TypeClean
    );
    assert_eq!(
        first_move(&box_with(&["❯ "], BUSY_FOOTER)),
        FirstMove::TypeClean
    );
}

#[test]
fn first_move_o_co_chu_luc_ranh_thi_do_luc_ban_thi_tu_choi() {
    let nhap = ["❯ nháp của chủ máy"];
    assert_eq!(first_move(&box_with(&nhap, IDLE_FOOTER)), FirstMove::Probe);
    // Vế ngược: CÙNG ô ấy nhưng phiên đang chạy ⟹ không dò (ESC là phím cắt lượt).
    match first_move(&box_with(&nhap, BUSY_FOOTER)) {
        FirstMove::Refuse(why) => assert!(why.contains("nháp của chủ máy"), "{why}"),
        m => panic!("phiên đang chạy + ô có chữ phải TỪ CHỐI, được {m:?}"),
    }
}

#[test]
fn first_move_tu_choi_khi_dang_co_hop_chon_hoac_khong_co_o() {
    match first_move(&confirm_medium()) {
        FirstMove::Refuse(_) => {}
        m => panic!("màn đang có hộp chọn phải TỪ CHỐI, được {m:?}"),
    }
    let shell = "Last login: Tue Nov 10 07:58:24 on ttys012\nuser@host ~ % ";
    match first_move(shell) {
        FirstMove::Refuse(_) => {}
        m => panic!("cửa sổ shell trần phải TỪ CHỐI, được {m:?}"),
    }
}

#[test]
fn probe_verdict_tach_lenh_tron_khoi_lenh_lan_nhap() {
    let cmd = "/effort low";
    // Màn thật sau `nháp…`+ESC rồi `/effort low`+ESC: hai dòng trong ô.
    let lan = box_with(&["❯ nháp của chủ máy", "  /effort low"], IDLE_FOOTER);
    match probe_verdict(&lan, cmd) {
        ProbeSays::Mixed(t) => assert!(t.contains("nháp"), "{t}"),
        v => panic!("ô lẫn nháp phải là Mixed, được {v:?}"),
    }
    // Vế ngược: ô chỉ có đúng lệnh (thứ có trước là gợi ý mờ đã tắt).
    let tron = box_with(&["❯ /effort low"], IDLE_FOOTER);
    assert_eq!(probe_verdict(&tron, cmd), ProbeSays::OnlyOurs);
    assert_eq!(
        probe_verdict(&box_with(&["❯ "], IDLE_FOOTER), cmd),
        ProbeSays::Gone
    );
    assert_eq!(probe_verdict(&confirm_medium(), cmd), ProbeSays::NoBox);
}

#[test]
fn outcome_ba_nhanh_ba_ma_thoat_va_cau_noi_dung_muc() {
    let ok = Outcome::Applied { confirmed: false };
    let ok_xn = Outcome::Applied { confirmed: true };
    let no = Outcome::Refused("ô có nháp".into());
    let mu = Outcome::NotSeen("đuôi màn".into());
    assert_eq!(
        [
            ok.exit_code(),
            ok_xn.exit_code(),
            no.exit_code(),
            mu.exit_code()
        ],
        [0, 0, 1, 2]
    );
    assert!(ok.say("[huba]", "high").contains("high"));
    // Đo 10/11: đổi lúc đang chờ công cụ ⟹ lần gọi model KẾ (cùng lượt) đã theo mức mới.
    assert!(ok.say("[huba]", "high").contains("GỌI MODEL KẾ"));
    assert!(ok_xn.say("[huba]", "medium").contains("ĐỌC LẠI"));
    // "Không thấy xác nhận" KHÔNG được nói như đã đổi.
    assert!(!mu.say("[huba]", "high").contains("effort → "));
    assert!(no.say("[huba]", "high").contains("CHƯA"));
}

#[test]
fn route_effort_parse_ca_co_muc_lan_tran() {
    use huba::adapters::CommandKind;
    assert_eq!(
        huba::verbs::parse_command("/effort high"),
        Some((CommandKind::Effort, 0, "high".to_string()))
    );
    assert_eq!(
        huba::verbs::parse_command("/mucdo xhigh"),
        Some((CommandKind::Effort, 0, "xhigh".to_string()))
    );
    // Chạm từ menu ☰ chỉ gửi `/effort` trần — vẫn phải tới được handler.
    assert_eq!(
        huba::verbs::parse_command("/effort"),
        Some((CommandKind::Effort, 0, String::new()))
    );
    // Vế ngược: chữ thường không phải lệnh.
    assert_ne!(
        huba::verbs::parse_command("effort high").map(|c| c.0),
        Some(CommandKind::Effort)
    );
}
