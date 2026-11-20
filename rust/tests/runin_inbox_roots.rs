//! Hòm thư của phiên phải được đọc ở MỌI gốc tạm mà CLI dùng (Hà 2026-11-20).
//!
//! Ca thật: thư mục tạm chuyển sang `/mnt/data/tmp` (`CLAUDE_CODE_TMPDIR`), phiên
//! mới ghi hòm thư ở `/mnt/data/tmp/claude-1000/…` còn huba chỉ quét `/tmp` ⟹ lệnh
//! xếp vào hòm thư không bao giờ chạy, và không có lỗi nào được ghi.

use huba::pipeline::{scan_session_inboxes, scratch_roots_from, RUNIN_INBOX_NAME};
use std::fs;
use std::path::PathBuf;

const SID: &str = "f114d22a-341b-4d6f-bcd8-90d3ec4ec9b0";

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("huba-inbox-roots-{name}"));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_configured_tmpdir_is_scanned_besides_the_base() {
    let roots = scratch_roots_from("/tmp", Some("/mnt/data/tmp"), None);
    assert_eq!(
        roots,
        vec![PathBuf::from("/tmp"), PathBuf::from("/mnt/data/tmp")]
    );
}

#[test]
fn duplicates_and_blanks_are_dropped() {
    // ĐỐI CHỨNG NGƯỢC: không quét một gốc hai lần (sẽ nhận trùng hòm thư).
    let roots = scratch_roots_from("/tmp", Some("/tmp"), Some("  "));
    assert_eq!(roots, vec![PathBuf::from("/tmp")]);
    assert_eq!(
        scratch_roots_from("/tmp", None, None),
        vec![PathBuf::from("/tmp")],
        "không có biến ⟹ vẫn quét gốc mặc định"
    );
}

#[test]
fn a_mailbox_under_the_new_tmpdir_is_found_and_one_under_the_base_still_is() {
    let base = tmp("base");
    let moved = tmp("moved");
    for (root, sid, cmd) in [
        (&moved, SID, "git status --short"),
        (&base, "ses_e06f09837ffedkvJJi0ZBam657", "git fetch"),
    ] {
        let dir = root
            .join("claude-1000")
            .join("-mnt-data-projects")
            .join(sid)
            .join("scratchpad");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(RUNIN_INBOX_NAME), format!("{cmd}\n")).unwrap();
    }
    let roots = scratch_roots_from(base.to_str().unwrap(), Some(moved.to_str().unwrap()), None);
    let mut got: Vec<(String, String)> = roots
        .iter()
        .flat_map(|r| scan_session_inboxes(r))
        .map(|(s, c, _)| (s, c))
        .collect();
    got.sort();
    assert_eq!(got.len(), 2, "{got:#?}");
    assert!(got
        .iter()
        .any(|(s, c)| s == SID && c == "git status --short"));
    // Chỉ quét gốc cũ (như trước khi sửa) ⟹ mất hòm thư ở gốc mới: đây là chỗ RED.
    assert_eq!(scan_session_inboxes(&base).len(), 1);
}
