//! Luật chung của máy = `CLAUDE.md` ở GỐC workspace, không phải một tệp lời
//! nhắc riêng.
//!
//! Quang 13/11: *"bản đóng gói cần một file prompt sạch khi cài đặt để chạy là
//! luôn đọc file đó đầu tiên"* rồi *"sao không giữ cấu trúc file md như hiện tại
//! trong thư mục /projects?"*. Hai vế ấy chốt hình dạng:
//!
//! - **claude**: KHÔNG cờ nào. Nó tự nạp `CLAUDE.md` của cwd + mọi thư mục cha,
//!   đúng cấu trúc `~/projects/CLAUDE.md` đang dùng trên Mac. Bài đầu tiên khoá
//!   điều ấy — ai thêm lại `--append-system-prompt-file` sẽ thấy nó đỏ.
//! - **opencode**: đo 13/11 trên 2.2.7 — nó chỉ tự tìm `AGENTS.md`, và chỉ đi
//!   ngược tới **gốc git của cwd** (`opencode run` trong worktree lồng nhau
//!   KHÔNG thấy `AGENTS.md` ở thư mục cha; không đọc `CLAUDE.md` ở đâu cả). Nên
//!   trình cài khai `HUBA_LUAT_GOC=<gốc>/CLAUDE.md` và huba đưa tệp ấy lên đầu
//!   `instructions` của `OPENCODE_CONFIG_CONTENT`.

use std::path::Path;

use huba::accounts::chen_luat_goc;
use huba::sessions::terminal_command_resuming;
use serde_json::json;

#[test]
fn claude_khong_nhan_co_nao_cho_luat_goc() {
    // claude tự đọc CLAUDE.md của cây — một cờ ở đây là bản sao thứ hai, lệch im lặng.
    let co = terminal_command_resuming("claude", Path::new("/w"), Some("việc"), None);
    assert!(!co.contains("append-system-prompt"), "{co}");
    assert!(co.contains("'việc'"), "đề bài vẫn còn: {co}");
    assert!(co.contains("--disallowedTools"), "tường chặn vẫn còn: {co}");
}

#[test]
fn opencode_luat_goc_len_dau_instructions_khong_lap() {
    let mut cfg = json!({ "instructions": ["/a.md", "/p.md"] });
    chen_luat_goc(&mut cfg, "/p.md");
    assert_eq!(cfg["instructions"], json!(["/p.md", "/a.md"]));

    let mut rong = json!({});
    chen_luat_goc(&mut rong, "/p.md");
    assert_eq!(rong["instructions"], json!(["/p.md"]));

    // Giữ nguyên mọi thứ khác của cấu hình tài khoản.
    let mut day = json!({ "model": "m", "agent": { "build": { "variant": "high" } } });
    chen_luat_goc(&mut day, "/p.md");
    assert_eq!(day["model"], "m");
    assert_eq!(day["agent"]["build"]["variant"], "high");
}

#[test]
fn env_luat_goc_tro_dung_tep() {
    let dir = std::env::temp_dir().join(format!("huba-luat-goc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmp");
    let tep = dir.join("CLAUDE.md");
    std::fs::write(&tep, "# luật\n").expect("ghi");

    std::env::remove_var("HUBA_LUAT_GOC");
    assert!(huba::accounts::luat_goc().is_none(), "không khai ⟹ None");

    std::env::set_var("HUBA_LUAT_GOC", &tep);
    assert_eq!(huba::accounts::luat_goc().as_deref(), Some(tep.as_path()));

    // Khai mà tệp mất ⟹ None (log `luat_goc_missing`), phiên vẫn mở được.
    std::env::set_var("HUBA_LUAT_GOC", dir.join("khong-co.md"));
    assert!(huba::accounts::luat_goc().is_none(), "tệp mất ⟹ None");

    std::env::remove_var("HUBA_LUAT_GOC");
}
