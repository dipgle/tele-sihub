//! Ubuntu thật 13/11 (user1-ubuntu, acc8 = opencode): mỗi vòng `/session` log
//!
//! ```text
//! {"account":"acc8","dir":"/home/user1/.claude-acc8","level":"warn","msg":"claude_books_missing",
//!  "why":"không thấy <config>/sessions — rơi về `claude agents --json` (chậm hơn ~30 lần)"}
//! ```
//!
//! — 957/2721 dòng log là câu ấy. acc opencode KHÔNG có sổ Claude; hai vòng
//! Claude (`snapshot`, `listed_in_books`) quên phép rẽ `is_opencode()` nên spawn
//! `claude agents --json` với thư mục cấu hình của acc8 mỗi vòng. Chỗ thứ sáu cùng
//! lớp với doctor (`tests/doctor_opencode_khong_doc_so_claude.rs`).
//!
//! Đo bằng một `claude` GIẢ ghi dấu mỗi lần bị gọi — không đọc log, không đọc mã.

mod common;

use huba::config::ClaudeAccountCfg;
use huba::sessions::{listed_in_books, Listed};

fn hoi_so(cli_acc: Option<&str>) -> (Listed, String) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dau = tmp.path().join("da-goi.txt");
    let gia = tmp.path().join("claude-gia.sh");
    std::fs::write(
        &gia,
        format!(
            "#!/bin/sh\necho \"$CLAUDE_CONFIG_DIR $*\" >> '{}'\necho '[]'\n",
            dau.display()
        ),
    )
    .expect("ghi claude giả");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&gia, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    // Đúng ca máy thật: thư mục cấu hình có, KHÔNG có `sessions/`.
    let cfg_dir = tmp.path().join("cfg-accx");
    std::fs::create_dir_all(&cfg_dir).expect("tạo cfg_dir");
    let mut cfg = common::cfg_for_tests();
    cfg.claude_cli = gia.display().to_string();
    cfg.claude_accounts = vec![ClaudeAccountCfg {
        name: "accx".into(),
        config_dir: Some(cfg_dir.display().to_string()),
        cli: cli_acc.map(str::to_string),
        ..Default::default()
    }];
    let kq = listed_in_books(&cfg, "0123abcd-0000-4000-8000-000000000000");
    let goi = std::fs::read_to_string(&dau).unwrap_or_default();
    (kq, goi)
}

#[test]
fn acc_opencode_khong_spawn_claude_agents() {
    let (kq, goi) = hoi_so(Some("opencode"));
    assert!(
        goi.is_empty(),
        "acc opencode làm `claude agents` chạy:\n{goi}"
    );
    assert_eq!(kq, Listed::No);
}

/// Đối chứng ngược: cùng dựng nhưng acc claude ⇒ `claude` giả PHẢI bị gọi — bài
/// trên không xanh nhờ đường rơi về CLI đã chết hay dấu ghi hỏng.
#[test]
fn doi_chung_acc_claude_van_hoi_cli() {
    let (_, goi) = hoi_so(None);
    assert!(
        goi.contains("agents --json"),
        "claude giả không được gọi: {goi:?}"
    );
}
