//! `/<lệnh> -h` — mỗi lệnh tự nói nó hỗ trợ gì.
//!
//! Quang 21/11: *«Sao không gắn cờ -h vào từng lệnh để xem nó hỗ trợ gì làm ntn»* — và: chưa có cách xem
//! `/set` đặt được trường nào, giá trị gì. Bài này khoá ba điều: (1) `-h` ĐỨNG MỘT MÌNH thì ra trợ giúp
//! cho MỌI route trong bảng, không để lọt cái nào chạy thành một đề bài/chữ gõ; (2) đối chứng ngược — một
//! đề bài kết thúc bằng `-h` vẫn là đề bài; (3) nội dung `/new -h`, `/set -h` có đúng thứ chủ máy cần.

use huba::adapters::CommandKind;
use huba::commands::{chi_tiet, hoi_chi_tiet, huong_dan_set, ROUTES};
use huba::verbs::parse_command;
use serde_json::json;

#[test]
fn every_route_answers_dash_h_with_its_own_help() {
    for r in ROUTES {
        for cach in ["-h", "--help"] {
            let got = parse_command(&format!("/{} {cach}", r.name));
            assert_eq!(
                got,
                Some((CommandKind::Help, 0, r.name.to_string())),
                "/{} {cach} phải ra trợ giúp của chính nó",
                r.name
            );
        }
        assert!(chi_tiet(r.name).is_some(), "/{} phải có chi tiết", r.name);
    }
}

/// Đối chứng ngược: `-h` ở cuối một ĐỀ BÀI không bị nuốt.
#[test]
fn a_task_that_ends_with_dash_h_is_still_a_task() {
    let (kind, _, arg) = parse_command("/new sửa lỗi hiển thị -h").expect("parsed");
    assert_eq!(kind, CommandKind::New);
    assert!(arg.contains("-h"), "{arg}");
    let (kind, _, _) = parse_command("/ask tại sao -h").expect("parsed");
    assert_ne!(kind, CommandKind::Help);
}

#[test]
fn dash_h_alone_never_opens_a_window_or_types_into_a_session() {
    for lenh in ["/new -h", "/type -h", "/ask -h", "/handover -h", "/stop -h"] {
        let (kind, _, _) = parse_command(lenh).expect("parsed");
        assert_eq!(kind, CommandKind::Help, "{lenh}");
    }
}

#[test]
fn only_set_takes_a_section_before_dash_h() {
    assert_eq!(
        hoi_chi_tiet("set", "auto_handover -h"),
        Some("set auto_handover".to_string())
    );
    assert_eq!(hoi_chi_tiet("new", "auto_handover -h"), None);
    // Hai chữ trước -h ⟹ có thể là khoá + giá trị: không phải lời xin trợ giúp.
    assert_eq!(hoi_chi_tiet("set", "a b -h"), None);
}

#[test]
fn the_new_help_names_every_flag_the_parser_reads() {
    let t = chi_tiet("new").expect("có");
    for co in ["-a", "-s", "-y"] {
        assert!(t.contains(co), "thiếu cờ {co}: {t}");
    }
    assert!(t.contains("/new "), "{t}");
}

#[test]
fn the_set_help_lists_sections_then_fields_with_type_and_default() {
    let root = json!({
        "auto_handover": { "enabled": true, "at_percent": 80, "idle_sec": 120 },
        "log_file": "x.log",
        "claude_accounts": [{ "name": "acc1" }]
    });
    let mac = json!({
        "auto_handover": { "enabled": true, "at_percent": 60, "idle_sec": 120 },
        "log_file": "y.log"
    });
    let muc = huong_dan_set(&root, &mac, None);
    assert!(muc.contains("auto_handover — 3 trường"), "{muc}");
    assert!(muc.contains("log_file"), "trường lẻ phải được kể: {muc}");

    let tr = huong_dan_set(&root, &mac, Some("auto_handover"));
    assert!(tr.contains("auto_handover.at_percent = 80"), "{tr}");
    assert!(tr.contains("số") && tr.contains("mặc định 60"), "{tr}");
    assert!(
        tr.contains("auto_handover.enabled = true — true/false · đang là mặc định"),
        "{tr}"
    );

    let le = huong_dan_set(&root, &mac, Some("log_file"));
    assert!(le.contains("chữ"), "{le}");

    let acc = huong_dan_set(&root, &mac, Some("claude_accounts"));
    assert!(acc.contains("không đặt bằng /set"), "{acc}");

    let sai = huong_dan_set(&root, &mac, Some("khong_co"));
    assert!(sai.contains("Không có mục"), "{sai}");
}

/// Mặc định in ra là mặc định THẬT trong mã — gồm cả 60 vừa đổi.
#[test]
fn the_default_shown_for_at_percent_is_the_one_in_code() {
    let mac = serde_json::to_value(huba::config::Config::default()).expect("ghi được");
    let root = json!({ "auto_handover": { "at_percent": 80 } });
    let tr = huong_dan_set(&root, &mac, Some("auto_handover.at_percent"));
    assert!(tr.contains("mặc định 60"), "{tr}");
}

#[test]
fn a_huge_listing_still_fits_a_telegram_message() {
    let mut m = serde_json::Map::new();
    for i in 0..400 {
        m.insert(format!("truong_{i}"), json!("x".repeat(70)));
    }
    let root = json!({ "to": m });
    let t = huong_dan_set(&root, &json!({}), Some("to"));
    assert!(t.chars().count() <= 3900, "{}", t.chars().count());
}
