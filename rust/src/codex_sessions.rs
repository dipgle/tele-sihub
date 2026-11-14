//! Phiên codex (OpenAI Codex CLI) đang mở — nguồn phiên THỨ BA của `/session`.
//!
//! Quang 14/11: *«mọi tài khoản phải có đầu vào như nhau cùng trải nghiệm»*. Phiên
//! Claude lên danh sách qua sổ `claude agents`, phiên opencode qua kho SQLite; codex
//! không có sổ "ai đang sống" nào — chỉ có nhật ký `CODEX_HOME/sessions/**/
//! rollout-<mốc>-<id>.jsonl` cho MỌI phiên từng mở.
//!
//! 🔴 Ghép SỐNG ↔ NHẬT KÝ bằng tệp đang mở, không đoán. Đo 14/11 (codex-cli
//! 0.169.0, Ubuntu): TUI `codex` giữ MỞ đúng tệp rollout của phiên nó
//! (`/proc/<pid>/fd`) — hai TUI sống (pts/11, pts/14) trỏ đúng hai tệp, một-một.
//! Nên: tiến trình TUI còn sống ⟹ có hàng; tty của hàng = tty của tiến trình (từ
//! cùng bảng `ps` của ảnh chụp); thư mục `CODEX_HOME` chứa tệp ⟹ tài khoản.
//! Nhật ký không có tiến trình giữ = phiên đã tắt ⟹ KHÔNG lên (cùng luật «phiên
//! ma» của opencode, Quang 12/11).
//!
//! Không đọc được `/proc` (macOS/Windows, hoặc lỗi) là **KHÔNG ĐO ĐƯỢC** — trả
//! `Err`, người gọi đưa tài khoản vào `blind` (luật 11b), không đọc thành "0 phiên".

use serde_json::Value;
use std::path::{Path, PathBuf};

/// Dòng lệnh `ps` này có phải một TUI codex đang mở không.
///
/// Đo 14/11 trên máy: TUI = `…/vendor/…/bin/codex -s workspace-write -a never …`
/// (argv[0] tên `codex`, đối số đầu là CỜ). KHÔNG tính: `codex app-server …`
/// (dịch vụ nền chung, không cửa sổ), `codex exec …` (chạy không tty — lượt đo /
/// fork), `codex-code-mode-host` (con của TUI, tên khác), và các lệnh con quản trị.
pub fn is_codex_tui(command: &str) -> bool {
    let mut it = command.split_whitespace();
    let Some(prog) = it.next() else {
        return false;
    };
    if Path::new(prog).file_name().and_then(|n| n.to_str()) != Some("codex") {
        return false;
    }
    !matches!(
        it.next(),
        Some(
            "exec"
                | "e"
                | "app-server"
                | "mcp"
                | "mcp-server"
                | "login"
                | "logout"
                | "features"
                | "completion"
                | "sandbox"
                | "debug"
                | "apply"
                | "cloud"
        )
    )
}

/// Trong các đích fd của một tiến trình, tệp rollout nằm dưới `sessions_dir`
/// (`CODEX_HOME/sessions`). Nhiều tệp (hiếm — chưa thấy) ⟹ lấy tên lớn nhất: tên
/// mang mốc mở phiên, nên đó là phiên mới nhất của tiến trình.
pub fn rollout_among(targets: &[PathBuf], sessions_dir: &Path) -> Option<PathBuf> {
    targets
        .iter()
        .filter(|p| p.starts_with(sessions_dir))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
        })
        .max()
        .cloned()
}

/// Những gì đọc được từ một tệp rollout.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rollout {
    pub id: String,
    pub cwd: String,
    pub started_ms: i64,
    pub model: Option<String>,
    /// Câu cuối của assistant (lời kết lượt, hoặc tin nhắn assistant mới nhất).
    pub last_text: Option<String>,
    /// `"user"` · `"assistant"` — ai nói câu cuối.
    pub last_role: Option<String>,
    /// `Some(true)` = lượt đang chạy (`task_started` chưa có `task_complete`);
    /// `None` = đuôi không chứa mốc lượt nào — không biết, đừng đoán.
    pub busy: Option<bool>,
    /// RFC 3339 của dòng cuối có mốc.
    pub last_ts: Option<String>,
    pub context_tokens: u64,
}

/// Đọc dòng ĐẦU (`session_meta`) + ĐUÔI của một rollout. Hàm THUẦN.
///
/// `tail` có thể bắt đầu giữa một dòng (đọc theo byte) — dòng không phải JSON bị
/// bỏ qua. Khuôn đo 14/11: `session_meta.payload.{id,cwd,timestamp}`;
/// `turn_context.payload.model`; `event_msg` `task_started` / `task_complete`
/// (`last_agent_message`) / `turn_aborted` / `token_count`
/// (`info.last_token_usage.input_tokens`) / `item_completed` (`item.type =
/// "UserMessage"`); `response_item` `message` (role `assistant` · `user`).
pub fn parse_rollout(head_line: &str, tail: &str) -> Rollout {
    let mut r = Rollout::default();
    if let Ok(v) = serde_json::from_str::<Value>(head_line.trim()) {
        if v.get("type").and_then(Value::as_str) == Some("session_meta") {
            let p = &v["payload"];
            r.id = p["id"].as_str().unwrap_or_default().to_string();
            r.cwd = p["cwd"].as_str().unwrap_or_default().to_string();
            r.started_ms = p["timestamp"]
                .as_str()
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|d| d.timestamp_millis())
                .unwrap_or(0);
        }
    }
    for line in tail.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(ts) = v.get("timestamp").and_then(Value::as_str) {
            r.last_ts = Some(ts.to_string());
        }
        let p = &v["payload"];
        match (v["type"].as_str(), p["type"].as_str()) {
            (Some("turn_context"), _) => {
                if let Some(m) = p["model"].as_str().filter(|m| !m.is_empty()) {
                    r.model = Some(m.to_string());
                }
            }
            (Some("event_msg"), Some("task_started")) => r.busy = Some(true),
            (Some("event_msg"), Some("turn_aborted")) => r.busy = Some(false),
            (Some("event_msg"), Some("task_complete")) => {
                r.busy = Some(false);
                if let Some(t) = p["last_agent_message"]
                    .as_str()
                    .filter(|t| !t.trim().is_empty())
                {
                    r.last_text = Some(t.to_string());
                    r.last_role = Some("assistant".into());
                }
            }
            (Some("event_msg"), Some("token_count")) => {
                if let Some(n) = p["info"]["last_token_usage"]["input_tokens"].as_u64() {
                    r.context_tokens = n;
                }
            }
            (Some("event_msg"), Some("item_completed")) => {
                if p["item"]["type"].as_str() == Some("UserMessage") {
                    r.last_role = Some("user".into());
                }
            }
            (Some("response_item"), Some("message")) if p["role"].as_str() == Some("assistant") => {
                let text: String = p["content"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|c| c["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default();
                if !text.trim().is_empty() {
                    r.last_text = Some(text);
                    r.last_role = Some("assistant".into());
                }
            }
            _ => {}
        }
    }
    r
}

/// Tên luồng (`thread_name`) mới nhất của `id` trong `CODEX_HOME/session_index.jsonl`
/// (dòng `{"id","thread_name","updated_at"}`, đo 14/11). Hàm THUẦN.
pub fn thread_name(index: &str, id: &str) -> Option<String> {
    index
        .lines()
        .rev()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["id"].as_str() == Some(id))
        .filter_map(|v| v["thread_name"].as_str().map(str::to_string))
        .find(|n| !n.trim().is_empty())
}

/// Mọi đích fd của một tiến trình. Chỉ Linux (`/proc`) — nơi khác là KHÔNG ĐO
/// ĐƯỢC, trả `Err` nói rõ, không trả danh sách rỗng.
pub fn fd_targets(pid: i64) -> Result<Vec<PathBuf>, String> {
    let dir = PathBuf::from(format!("/proc/{pid}/fd"));
    let rd =
        std::fs::read_dir(&dir).map_err(|e| format!("không đọc được {}: {e}", dir.display()))?;
    Ok(rd
        .filter_map(|e| e.ok())
        .filter_map(|e| std::fs::read_link(e.path()).ok())
        .collect())
}

/// Đọc dòng đầu (≤ `max` byte) của tệp.
pub fn first_line(path: &Path, max: usize) -> std::io::Result<String> {
    use std::io::{BufRead, Read};
    let f = std::fs::File::open(path)?;
    let mut s = String::new();
    std::io::BufReader::new(f.take(max as u64)).read_line(&mut s)?;
    Ok(s)
}
