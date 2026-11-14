//! `/session` liệt kê phiên codex — phần THUẦN của `codex_sessions`.
//!
//! Khuôn lấy từ máy thật 14/11 (codex-cli 0.169.0, Ubuntu): hai TUI sống ở
//! pts/11 · pts/14 hiện trên `/session` thành «cửa sổ pts/11» trơn (loại `shell`,
//! không tên, không dự án) — bản cài a17b03a có 0 hàng codex.

use huba::codex_sessions::{is_codex_tui, parse_rollout, rollout_among, thread_name};
use std::path::{Path, PathBuf};

const TUI: &str = "/home/user1/.npm-global/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin/codex -s workspace-write -a never --dangerously-bypass-hook-trust";

#[test]
fn chi_tui_codex_moi_la_phien() {
    assert!(is_codex_tui(TUI));
    assert!(is_codex_tui("codex"));
    // Dịch vụ nền chung, con của TUI, lượt exec — đo cùng lúc trên máy, không phải phiên.
    assert!(!is_codex_tui("/home/user1/.codex/packages/app-server-daemon/releases/0.169.0-x86_64-unknown-linux-musl/bin/codex app-server --listen unix:// --analytics-default-enabled --managed-daemon"));
    assert!(!is_codex_tui(
        "/x/bin/codex-code-mode-host --listen grpc+stdio://"
    ));
    assert!(!is_codex_tui("codex exec -s workspace-write 'đo'"));
    assert!(!is_codex_tui("/usr/bin/claude --permission-mode auto"));
    assert!(!is_codex_tui(""));
}

#[test]
fn chon_dung_tep_rollout_dang_mo() {
    let s = Path::new("/home/u/.codex-acc10/sessions");
    let targets: Vec<PathBuf> = vec![
        "/dev/pts/11".into(),
        "/home/u/.codex-acc10/logs_2.sqlite".into(),
        "/home/u/.codex-acc10/sessions/2026/11/14/rollout-2026-11-14T14-35-11-01a1d8fb-5244-7a80-8a50-7442ef0e760d.jsonl".into(),
        // Rollout của TÀI KHOẢN KHÁC — không được nhận.
        "/home/u/.codex-acc11/sessions/2026/11/14/rollout-2026-11-14T15-00-00-ffffffff-0000-0000-0000-000000000000.jsonl".into(),
    ];
    assert_eq!(
        rollout_among(&targets, s).unwrap(),
        PathBuf::from("/home/u/.codex-acc10/sessions/2026/11/14/rollout-2026-11-14T14-35-11-01a1d8fb-5244-7a80-8a50-7442ef0e760d.jsonl")
    );
    assert_eq!(rollout_among(&targets[..2], s), None);
}

const HEAD: &str = r#"{"timestamp":"2026-11-14T07:55:35.610Z","type":"session_meta","payload":{"id":"01a1d90d-fea8-7140-8278-dea70be41e0e","timestamp":"2026-11-14T07:55:35.475Z","cwd":"/mnt/data/projects","originator":"codex-tui","source":"cli"}}"#;

fn dong(ts: &str, kind: &str, payload: &str) -> String {
    format!(r#"{{"timestamp":"{ts}","type":"{kind}","payload":{payload}}}"#)
}

#[test]
fn doc_rollout_luot_dang_chay_roi_xong() {
    let chay = [
        // Đuôi đọc theo byte có thể bắt đầu giữa một dòng.
        "\"t\":1}}".to_string(),
        dong("2026-11-14T07:55:35.613Z", "event_msg", r#"{"type":"task_started","turn_id":"t1"}"#),
        dong("2026-11-14T07:55:37.233Z", "turn_context", r#"{"model":"gpt-6.1-sol","effort":"high","cwd":"/mnt/data/projects"}"#),
        dong("2026-11-14T07:55:37.248Z", "event_msg", r#"{"type":"item_completed","item":{"type":"UserMessage","content":[{"type":"text","text":"aihub"}]}}"#),
        dong("2026-11-14T07:55:42.803Z", "event_msg", r#"{"type":"token_count","info":{"last_token_usage":{"input_tokens":14387}}}"#),
    ]
    .join("\n");
    let r = parse_rollout(HEAD, &chay);
    assert_eq!(r.id, "01a1d90d-fea8-7140-8278-dea70be41e0e");
    assert_eq!(r.cwd, "/mnt/data/projects");
    assert!(r.started_ms > 0);
    assert_eq!(
        r.busy,
        Some(true),
        "task_started chưa có task_complete ⟹ đang chạy"
    );
    assert_eq!(r.model.as_deref(), Some("gpt-6.1-sol"));
    assert_eq!(r.context_tokens, 14387);
    assert_eq!(r.last_role.as_deref(), Some("user"));
    assert_eq!(r.last_ts.as_deref(), Some("2026-11-14T07:55:42.803Z"));

    let xong = format!(
        "{chay}\n{}",
        dong(
            "2026-11-14T07:55:58.382Z",
            "event_msg",
            r#"{"type":"task_complete","last_agent_message":"Đã vào dự án aihub. Bạn muốn xử lý phần nào tiếp?"}"#
        )
    );
    let r = parse_rollout(HEAD, &xong);
    assert_eq!(r.busy, Some(false));
    assert_eq!(
        r.last_text.as_deref(),
        Some("Đã vào dự án aihub. Bạn muốn xử lý phần nào tiếp?")
    );
    assert_eq!(r.last_role.as_deref(), Some("assistant"));
}

#[test]
fn duoi_khong_co_moc_luot_thi_khong_doan() {
    let r = parse_rollout(HEAD, "rác\n{\"khong\":\"phai\"}");
    assert_eq!(r.busy, None);
    assert_eq!(r.last_text, None);
    // Dòng đầu hỏng ⟹ id rỗng (người gọi lấy id từ tên tệp), không panic.
    assert_eq!(
        parse_rollout("{\"type\":\"session_meta\",\"payl", "").id,
        ""
    );
}

#[test]
fn ten_luong_moi_nhat_thang() {
    let idx = r#"{"id":"a","thread_name":"Cũ","updated_at":"1"}
{"id":"b","thread_name":"Khác","updated_at":"2"}
{"id":"a","thread_name":"Mới","updated_at":"3"}"#;
    assert_eq!(thread_name(idx, "a").as_deref(), Some("Mới"));
    assert_eq!(thread_name(idx, "c"), None);
}
