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

/// Phiên CHÍNH đang mở trong TUI. TUI giữ cả fd của các agent con; thứ tự tên
/// tệp không nói được ai sở hữu cửa sổ. Chỉ `session_meta.source = "cli"` là
/// phiên chính. Metadata hỏng/mơ hồ phải trả lỗi để người gọi giữ sổ (`blind`).
pub fn rollout_among(targets: &[PathBuf], sessions_dir: &Path) -> Result<Option<PathBuf>, String> {
    select_root_rollout(targets, sessions_dir, |p| {
        first_line(p, 1 << 20).map_err(|e| format!("{}: {e}", p.display()))
    })
}

/// Cùng bộ chọn với fd thật; tách nguồn đọc để kiểm cả lỗi I/O và đối chứng.
pub fn select_root_rollout(
    targets: &[PathBuf],
    sessions_dir: &Path,
    mut read_head: impl FnMut(&Path) -> Result<String, String>,
) -> Result<Option<PathBuf>, String> {
    let paths: std::collections::BTreeSet<&PathBuf> = targets
        .iter()
        .filter(|p| p.starts_with(sessions_dir))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
        })
        .collect();
    if paths.is_empty() {
        return Ok(None);
    }
    let mut roots = Vec::new();
    for path in paths {
        let raw = read_head(path)?;
        let v: Value = serde_json::from_str(raw.trim())
            .map_err(|e| format!("{}: session_meta không đọc được: {e}", path.display()))?;
        if v["type"] != "session_meta" || v["payload"]["id"].as_str().is_none_or(str::is_empty) {
            return Err(format!("{}: thiếu session_meta/id", path.display()));
        }
        let source = &v["payload"]["source"];
        if source.as_str() == Some("cli") {
            roots.push(path.clone());
        } else if source.get("subagent").is_some() || source.as_str() == Some("exec") {
            // Đây là công việc bên trong phiên, không phải một cửa sổ khác.
        } else {
            return Err(format!("{}: nguồn phiên chưa nhận diện", path.display()));
        }
    }
    match roots.len() {
        1 => Ok(roots.pop()),
        n => Err(format!(
            "có {n} rollout cli trong cùng TUI — chưa xác định được phiên chính"
        )),
    }
}

/// Chỉ lời assistant thật, không lấy item_completed (bản sao của cùng lời nói),
/// tool result hay user trích tên dự án vào phép đếm lời tự khai.
pub fn assistant_text(record: &Value) -> Option<String> {
    let p = &record["payload"];
    if record["type"] != "response_item" || p["type"] != "message" || p["role"] != "assistant" {
        return None;
    }
    let texts: Vec<&str> = p["content"]
        .as_array()?
        .iter()
        .filter_map(|c| c["text"].as_str())
        .collect();
    (!texts.is_empty()).then(|| texts.join("\n"))
}

/// Lượt người dùng CLI thực sự nhận. Thư giữa agent/tool output không phải
/// bằng chứng gửi; dùng cùng adapter cho tên việc và xác nhận kết quả huba.
pub fn user_message(record: &Value) -> Option<String> {
    let p = &record["payload"];
    let content = match (record["type"].as_str(), p["type"].as_str()) {
        (Some("response_item"), Some("message")) if p["role"] == "user" => &p["content"],
        (Some("event_msg"), Some("item_completed")) if p["item"]["type"] == "UserMessage" => {
            &p["item"]["content"]
        }
        (Some("event_msg"), Some("user_message")) => {
            return p["message"].as_str().map(str::to_string)
        }
        _ => return None,
    };
    let text = content
        .as_array()?
        .iter()
        .filter_map(|c| c["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    (!text.is_empty()).then_some(text)
}

/// Ô nhập Codex 0.169 đo trên terminal thật: dấu ›, dòng model/workspace,
/// rồi footer phím. Chỉ một dấu › trong hội thoại không đủ làm neo.
/// Some("") = ô trống/gợi ý mặc định; None = không đo được kiểu màn này.
pub fn composer_text(screen: &str) -> Option<&str> {
    let mut offset = 0;
    let lines: Vec<(usize, &str)> = screen
        .split_inclusive('\n')
        .map(|l| {
            let at = offset;
            offset += l.len();
            (at, l)
        })
        .collect();
    let footer = lines.iter().rposition(|(_, l)| !l.trim().is_empty())?;
    let hint = lines[footer].1.trim();
    if !hint.contains("? for shortcuts") && hint != "tab to queue message" {
        return None;
    }
    let model = lines[..footer]
        .iter()
        .rposition(|(_, l)| !l.trim().is_empty())?;
    let metadata = lines[model].1.trim();
    if !metadata.contains(" · ") || !metadata.ends_with(']') {
        return None;
    }
    let prompt = lines[..model]
        .iter()
        .rposition(|(_, l)| l.trim_start().starts_with('›'))?;
    let column = lines[prompt].1.find('›')?;
    let start = lines[prompt].0 + column + '›'.len_utf8();
    let text = screen[start..lines[model].0].trim();
    // Placeholder quan sát được, không phải bản nháp. Không gửi gợi ý mờ hộ chủ máy.
    Some(if text == "Ask Codex to do anything" {
        ""
    } else {
        text
    })
}

/// Đề bài đầu tiên của CHỦ MÁY, đọc từ đầu rollout khi chưa có thread_name.
/// Các khối luật/môi trường được CLI chèn cũng có role=user nhưng không phải
/// tên việc. Giữ tên này ổn định, không lấy lời assistant đang đổi mỗi vòng.
pub fn user_task(prefix: &str) -> Option<String> {
    for line in prefix.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(text) = user_message(&v) else {
            continue;
        };
        let text = text.trim();
        if text.is_empty()
            || [
                "# AGENTS.md instructions",
                "<environment_context>",
                "<INSTRUCTIONS>",
                "<turn_aborted>",
            ]
            .iter()
            .any(|s| text.starts_with(s))
        {
            continue;
        }
        return text
            .lines()
            .find(|l| !l.trim().is_empty())
            .map(|s| crate::exec::truncate(s.trim(), 160));
    }
    None
}

/// Bỏ phần tên dự án đã có trên nhãn; chỉ bỏ khi khớp dự án/làn đã đo được.
pub fn task_caption(name: &str, folder: &str, lane: &str) -> String {
    if let Some((project, task)) = name.split_once(':') {
        let declared = if lane.is_empty() {
            folder.to_string()
        } else {
            format!("{folder}/{lane}")
        };
        if !folder.is_empty()
            && (project.trim() == folder || project.trim() == declared)
            && !task.trim().is_empty()
        {
            return task.trim().to_string();
        }
    }
    name.to_string()
}

/// Đọc đầu rollout có giới hạn, bỏ dòng bị cắt giữa UTF-8/JSON ở mép cuối.
pub fn read_prefix(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(256 * 1024)
        .read_to_end(&mut bytes)?;
    if let Some(end) = bytes.iter().rposition(|b| *b == b'\n') {
        bytes.truncate(end + 1);
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Đọc mốc lượt trên TOÀN nhật ký. Đuôi 256KB không chứa task_started của một
/// lượt dài; mtime đứng im khi chạy lệnh không có nghĩa là lượt đã kết thúc.
/// Quét dòng và chỉ parse các mốc, không parse những khối tool output lớn.
pub fn turn_busy(path: &Path) -> Result<Option<bool>, String> {
    use std::io::BufRead;
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();
    let mut busy = None;
    loop {
        line.clear();
        if reader
            .read_line(&mut line)
            .map_err(|e| format!("{}: {e}", path.display()))?
            == 0
        {
            break;
        }
        // Dòng cuối còn đang ghi chưa phải bản ghi hoàn chỉnh.
        if !line.ends_with('\n') {
            break;
        }
        if !["task_started", "task_complete", "turn_aborted"]
            .iter()
            .any(|s| line.contains(s))
        {
            continue;
        }
        let v: Value = serde_json::from_str(&line)
            .map_err(|e| format!("{}: mốc lượt hỏng: {e}", path.display()))?;
        if v["type"] != "event_msg" {
            continue;
        }
        match v["payload"]["type"].as_str() {
            Some("task_started") => busy = Some(true),
            Some("task_complete" | "turn_aborted") => busy = Some(false),
            _ => {}
        }
    }
    Ok(busy)
}

/// Số agent con TRỰC TIẾP còn lượt đang chạy, cùng nghĩa pending_subagents của
/// Claude. fd mở / thread_spawn_edges=open chỉ nói "còn giữ", không nói "bận".
pub fn pending_children(
    targets: &[PathBuf],
    sessions_dir: &Path,
    root_id: &str,
) -> Result<usize, String> {
    let paths: std::collections::BTreeSet<&PathBuf> = targets
        .iter()
        .filter(|p| p.starts_with(sessions_dir))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
        })
        .collect();
    let mut count = 0;
    for path in paths {
        let raw = first_line(path, 1 << 20).map_err(|e| format!("{}: {e}", path.display()))?;
        let v: Value =
            serde_json::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
        if v["payload"]["source"]["subagent"]["thread_spawn"]["parent_thread_id"].as_str()
            != Some(root_id)
        {
            continue;
        }
        match turn_busy(path)? {
            Some(true) => count += 1,
            Some(false) => {}
            None => {
                return Err(format!(
                    "{}: chưa đọc được mốc lượt agent con",
                    path.display()
                ))
            }
        }
    }
    Ok(count)
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
    /// `token_count.info.model_context_window` — cửa sổ codex TỰ KHAI (đo 14/11:
    /// 475 000). `None` = đuôi chưa có `token_count` nào khai số ấy.
    pub context_window: Option<u64>,
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
                if let Some(w) = p["info"]["model_context_window"]
                    .as_u64()
                    .filter(|w| *w > 0)
                {
                    r.context_window = Some(w);
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
