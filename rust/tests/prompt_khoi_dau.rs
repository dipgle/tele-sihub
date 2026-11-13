//! Quang 13/11: *"bản đóng gói cần một file prompt sạch khi cài đặt để chạy là luôn
//! đọc file đó đầu tiên"*. Trình cài chép `kem/prompt-khoi-dau.md` ra
//! `~/huba/prompt-khoi-dau.md` và khai `HUBA_PROMPT_KHOI_DAU`; mọi phiên huba mở
//! nạp nó: claude qua `--append-system-prompt-file` (đo 13/11: 2.1.280 Mac và
//! 2.1.322 Ubuntu đều nhận cờ — đường tệp sai báo «Append system prompt file not
//! found» ngay lúc đọc đối số), opencode qua `instructions` (đo: `opencode debug
//! config` 2.2.7 hiện đúng đường tệp).

use std::path::Path;

use huba::accounts::chen_prompt_khoi_dau;
use huba::sessions::{terminal_command_full, terminal_command_resuming};
use serde_json::json;

#[test]
fn claude_nap_tep_truoc_disallowed_tools() {
    let p = Path::new("/home/u/huba/prompt-khoi-dau.md");
    let co = terminal_command_full("claude", Path::new("/w"), Some("việc"), None, Some(p));
    let i_co = co
        .find("--append-system-prompt-file '/home/u/huba/prompt-khoi-dau.md'")
        .unwrap_or_else(|| panic!("thiếu cờ nạp tệp: {co}"));
    let i_chan = co.find("--disallowedTools").expect("tường chặn");
    // `--disallowedTools` là variadic — cờ đứng sau nó bị nuốt thành tên công cụ.
    assert!(i_co < i_chan, "cờ phải đứng TRƯỚC --disallowedTools: {co}");
    assert!(co.contains("'việc'"), "đề bài vẫn còn: {co}");

    // Không khai tệp ⟹ dòng lệnh Y NHƯ trước (máy Mac không đổi gì).
    let khong = terminal_command_full("claude", Path::new("/w"), Some("việc"), None, None);
    assert_eq!(
        khong,
        terminal_command_resuming("claude", Path::new("/w"), Some("việc"), None)
    );
    assert!(!khong.contains("append-system-prompt"), "{khong}");
}

#[test]
fn opencode_dua_tep_len_dau_instructions_khong_lap() {
    let mut cfg = json!({ "instructions": ["/a.md", "/p.md"] });
    chen_prompt_khoi_dau(&mut cfg, "/p.md");
    assert_eq!(cfg["instructions"], json!(["/p.md", "/a.md"]));

    let mut rong = json!({});
    chen_prompt_khoi_dau(&mut rong, "/p.md");
    assert_eq!(rong["instructions"], json!(["/p.md"]));
}
