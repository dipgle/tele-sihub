//! opencode as a SECOND CLI behind the same bridge.
//!
//! huba was written around `claude`: its session list comes from
//! `~/.claude-*/projects`, its fork calls use `-p --fork-session`, and its
//! `/usage` shell runs `claude -p /usage`. An account whose `cli` is
//! `opencode` must not borrow any of those paths — it has its own stores and
//! its own command shape.
//!
//! **Mỗi tài khoản MỘT phân vùng** (`~/.opencode-<tên>/{config,data,cache,state}`):
//! opencode's config, auth, SQLite store and session ids live there, so two
//! accounts cannot see each other. The account row is the only source of truth
//! for the key file and model; nothing here caches either.
//!
//! This module is the read/parse half. Command construction lives in
//! `accounts::launch_line` + `sessions::opencode_terminal_command`, so a
//! change in the wall or the partition shape has exactly one home.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::ClaudeAccountCfg;
use crate::exec::{self, RunOpts};

/// Bản ghi phiên đọc từ kho SQLite của opencode.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OcSession {
    pub id: String,
    pub title: String,
    pub directory: String,
    /// `provider/model`, đúng thứ `LiveSession.model` đang hiển thị.
    pub model: Option<String>,
    /// Token ngữ cảnh đã dùng: input + cache read + cache write.
    pub context_tokens: u64,
    pub cost_usd: f64,
    pub created_ms: i64,
    pub updated_ms: i64,
    /// Chữ của lượt ASSISTANT cuối — để `/session` có preview và `/ask` có
    /// bằng chứng "phiên này đã nói lượt nào".
    pub last_text: Option<String>,
    pub last_role: Option<String>,
    /// Tổng token CỘNG DỒN của cả phiên (để chẩn đoán). KHÔNG phải ngữ cảnh
    /// hiện tại — xem `context_tokens`.
    pub cum_tokens: u64,
    /// Thư mục DỰ ÁN mà chính opencode khai (`project.worktree`) — nguồn đúng
    /// cho nhãn hiển thị. `None` khi kho không có bảng `project`.
    pub project_dir: Option<String>,
}

/// Thư mục gốc phân vùng của một tài khoản.
pub fn profile_dir(acc: &ClaudeAccountCfg) -> PathBuf {
    crate::accounts::opencode_profile_dir(acc)
}

/// Đường dẫn kho SQLite — `$XDG_DATA_HOME/opencode/opencode.db`.
pub fn db_path(acc: &ClaudeAccountCfg) -> PathBuf {
    profile_dir(acc)
        .join("data")
        .join("opencode")
        .join("opencode.db")
}

/// Đọc danh sách phiên từ kho của tài khoản.
///
/// `Err` là "không đo được", không phải "không có phiên": chỗ gọi phải đẩy nó
/// vào `SessionsSnapshot::blind` (luật 11b). Kho chưa tồn tại (phiên đầu chưa
/// mở) trả `Ok(vec![])` — đó là sự thật "chưa có phiên nào", khác hẳn lỗi mở.
pub fn read_sessions(acc: &ClaudeAccountCfg) -> Result<Vec<OcSession>, String> {
    read_sessions_at(&db_path(acc))
}

/// Đọc MỘT kho SQLite cụ thể; `read_sessions` chỉ là lớp tìm ra đường dẫn ấy.
///
/// 🔴 Một tệp mang HAI lược đồ: opencode cũ ghi hàng vào `session`, bản mới ghi
/// vào `session_v2` và **không** sao chép hàng cũ sang. Chỉ đọc một bảng là
/// đúng hình dạng "phép đo mù" (đo 2026-11-12 00:1x, acc8): danh sách hiện 3
/// phiên đã tắt từ hôm trước và **giấu phiên đang chạy** ⇒ `/session` trên điện
/// thoại không có dòng nào để bấm; `/ask` bắn vào hàng giả `win-ttys001`, đi
/// đường Claude (gõ `/btw` vào TUI opencode), chờ 74 s rồi báo HỎNG; `/new` chờ
/// 45 s rồi báo `opencode_session_not_seen` dù phiên đã nằm trong kho. Bảng nào
/// có thì đọc; cùng một id thì hàng `session_v2` thắng (nó là bản đang ghi tiếp).
pub fn read_sessions_at(path: &Path) -> Result<Vec<OcSession>, String> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("mở {}: {e}", path.display()))?;
    let mut by_id: BTreeMap<String, OcSession> = BTreeMap::new();
    let co_project = has_table(&conn, "project")?;
    if has_table(&conn, "session")? {
        for s in query_sessions(&conn, path, "session", "", co_project)? {
            by_id.insert(s.id.clone(), s);
        }
    }
    if has_table(&conn, "session_v2")? {
        // `parent_id` khác NULL = phiên con của một subagent: không ai bấm vào
        // nó từ điện thoại, và liệt kê chúng chỉ đẩy phiên thật xuống dưới.
        for s in query_sessions(
            &conn,
            path,
            "session_v2",
            " AND s.parent_id IS NULL",
            co_project,
        )? {
            by_id.insert(s.id.clone(), s);
        }
    }
    let mut out: Vec<OcSession> = by_id.into_values().collect();
    out.sort_by_key(|b| std::cmp::Reverse(b.updated_ms));
    out.truncate(200);
    fill_last_turns(&conn, &mut out)?;
    fill_last_turns_v2(&conn, &mut out)?;
    Ok(out)
}

/// Có bảng ấy không. Kho cũ (chưa từng chạy opencode mới) không có
/// `session_v2`; hỏi `sqlite_master` thay vì để câu truy vấn nổ.
fn has_table(conn: &Connection, name: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n > 0)
    .map_err(|e| format!("hỏi sqlite_master ({name}): {e}"))
}

/// Một bảng phiên → danh sách. Hai bảng dùng CÙNG tên cột cho 12 cột này, nên
/// chỉ khác tên bảng và điều kiện lọc.
fn query_sessions(
    conn: &Connection,
    path: &Path,
    table: &str,
    extra_where: &str,
    co_project: bool,
) -> Result<Vec<OcSession>, String> {
    // `project.worktree` là nguồn đúng cho nhãn dự án (xem
    // `sessions::opencode_project_label`); kho chưa có bảng ấy thì thôi, không lỗi.
    let (cot, join) = if co_project {
        (
            ", p.worktree",
            " LEFT JOIN project p ON p.id = s.project_id".to_string(),
        )
    } else {
        ("", String::new())
    };
    let sql = format!(
        "SELECT s.id, COALESCE(s.title, ''), s.directory, s.model, \
                s.tokens_input, s.tokens_output, s.tokens_reasoning, \
                s.tokens_cache_read, s.tokens_cache_write, s.cost, \
                s.time_created, s.time_updated{cot} \
         FROM {table} s{join} WHERE s.time_archived IS NULL{extra_where} \
         ORDER BY s.time_updated DESC LIMIT 200"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("đọc schema {} ({table}): {e}", path.display()))?;
    let rows = stmt
        .query_map([], |r| {
            let id: String = r.get(0)?;
            let title: String = r.get(1)?;
            let directory: String = r.get(2)?;
            let model_raw: Option<String> = r.get(3)?;
            let input: i64 = r.get(4).unwrap_or(0);
            let _output: i64 = r.get(5).unwrap_or(0);
            let _reasoning: i64 = r.get(6).unwrap_or(0);
            let cache_read: i64 = r.get(7).unwrap_or(0);
            let cache_write: i64 = r.get(8).unwrap_or(0);
            let cost: f64 = r.get(9).unwrap_or(0.0);
            let created: i64 = r.get(10).unwrap_or(0);
            let updated: i64 = r.get(11).unwrap_or(0);
            let project_dir: Option<String> = if co_project {
                r.get::<_, Option<String>>(12).unwrap_or(None)
            } else {
                None
            };
            Ok(OcSession {
                id,
                title,
                directory,
                model: parse_model(&model_raw),
                // 🔴 KHÔNG dùng `tokens_*` của bảng session: đó là SỐ CỘNG DỒN
                // của cả phiên (đo 12/11: input 365 488 · cache.read 72 256 512
                // ⇒ 72,6 M "token ngữ cảnh" trong khi opencode hiện 35%). Ngữ
                // cảnh HIỆN TẠI đọc từ `tokens` của lượt assistant cuối
                // (`fill_last_turns_v2`); đây chỉ là giá trị dự phòng.
                context_tokens: 0,
                cum_tokens: input.max(0) as u64
                    + cache_read.max(0) as u64
                    + cache_write.max(0) as u64,
                cost_usd: cost,
                created_ms: created,
                updated_ms: updated,
                last_text: None,
                last_role: None,
                project_dir,
            })
        })
        .map_err(|e| format!("truy vấn {} ({table}): {e}", path.display()))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("đọc hàng {} ({table}): {e}", path.display()))?);
    }
    Ok(out)
}

/// Điền chữ của lượt assistant cuối — hai truy vấn cho CẢ tập, không N+1.
///
/// Không dùng JSON1 của SQLite: `message.data`/`part.data` là JSON nhưng ta
/// parse bằng serde, nên cổng không phụ thuộc cờ biên dịch của SQLite.
fn fill_last_turns(conn: &Connection, sessions: &mut [OcSession]) -> Result<(), String> {
    use rusqlite::params_from_iter;
    if sessions.is_empty() {
        return Ok(());
    }
    // Kho của lược đồ MỚI không có `message`/`part`; thiếu bảng là "không có gì
    // để đọc ở đây", không phải lỗi. (Đo 12/11: bài kiểm v2 đỏ vì câu này.)
    if !has_table(conn, "message")? {
        return Ok(());
    }
    if !has_table(conn, "part")? {
        return Ok(());
    }
    let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
    let ph = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT session_id, id, data FROM (              SELECT session_id, id, data,                     ROW_NUMBER() OVER (PARTITION BY session_id ORDER BY time_created DESC) rn              FROM message WHERE session_id IN ({ph})          ) WHERE rn <= 4"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("đọc message: {e}"))?;
    let rows = stmt
        .query_map(params_from_iter(ids.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("truy vấn message: {e}"))?;
    let mut last_assistant: BTreeMap<String, (String, String)> = BTreeMap::new();
    for row in rows {
        let (sid, mid, data) = row.map_err(|e| format!("đọc message: {e}"))?;
        if last_assistant.contains_key(&sid) {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        let role = v.get("role").and_then(Value::as_str).unwrap_or("");
        if role == "assistant" {
            last_assistant.insert(sid, (mid, role.to_string()));
        }
    }
    if last_assistant.is_empty() {
        return Ok(());
    }
    let mids: Vec<&str> = last_assistant.values().map(|(m, _)| m.as_str()).collect();
    let ph2 = vec!["?"; mids.len()].join(",");
    let sql2 = format!(
        "SELECT message_id, data FROM part WHERE message_id IN ({ph2}) ORDER BY time_created ASC"
    );
    let mut stmt2 = conn.prepare(&sql2).map_err(|e| format!("đọc part: {e}"))?;
    let rows2 = stmt2
        .query_map(params_from_iter(mids.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e| format!("truy vấn part: {e}"))?;
    let mut texts: BTreeMap<String, String> = BTreeMap::new();
    for row in rows2 {
        let (mid, data) = row.map_err(|e| format!("đọc part: {e}"))?;
        let Ok(v) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        if v.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let Some(t) = v.get("text").and_then(Value::as_str) else {
            continue;
        };
        let t = t.trim();
        if t.is_empty() {
            continue;
        }
        let e = texts.entry(mid).or_default();
        if !e.is_empty() {
            e.push('\n');
        }
        e.push_str(t);
    }
    for s in sessions.iter_mut() {
        if let Some((mid, role)) = last_assistant.get(&s.id) {
            s.last_text = texts.get(mid).cloned().filter(|t| !t.trim().is_empty());
            s.last_role = Some(role.clone());
        }
    }
    Ok(())
}

/// Điền chữ của lượt assistant cuối cho lược đồ MỚI (`session_v2`).
///
/// Ở đó nội dung KHÔNG nằm trong `part` như bảng cũ mà trong một cột `data` của
/// `session_message`, mang cả message: `content: [{type:"text"|"reasoning"|"tool"}]`.
/// Nên "lượt assistant cuối" phải là lượt CUỐI CÓ CHỮ: lượt cuối cùng thường là
/// toàn `reasoning`/`tool`, và lấy nó sẽ ra preview rỗng — rồi `/ask` lại tưởng
/// phiên chưa nói bao giờ.
///
/// Chỉ điền cho phiên còn trống: hàng `session` cũ (nếu có) đã có đường của nó.
fn fill_last_turns_v2(conn: &Connection, sessions: &mut [OcSession]) -> Result<(), String> {
    use rusqlite::params_from_iter;
    if !has_table(conn, "session_message")? {
        return Ok(());
    }
    let ids: Vec<&str> = sessions
        .iter()
        .filter(|s| s.last_text.is_none())
        .map(|s| s.id.as_str())
        .collect();
    if ids.is_empty() {
        return Ok(());
    }
    let ph = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT session_id, data FROM session_message \
         WHERE type = 'assistant' AND session_id IN ({ph}) ORDER BY seq DESC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("đọc session_message: {e}"))?;
    let rows = stmt
        .query_map(params_from_iter(ids.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e| format!("truy vấn session_message: {e}"))?;
    let mut xong: BTreeMap<String, String> = BTreeMap::new();
    // Ngữ cảnh HIỆN TẠI = `tokens` của lượt assistant mới nhất ĐÃ KHÉP (có
    // `finish`); cột `tokens_*` của bảng session là số CỘNG DỒN cả phiên.
    let mut ngu_canh: BTreeMap<String, u64> = BTreeMap::new();
    for row in rows {
        let (sid, data) = row.map_err(|e| format!("đọc session_message: {e}"))?;
        let Ok(v) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        if !ngu_canh.contains_key(&sid) {
            if let Some(t) = v.get("tokens") {
                let so = |k: &str| t.get(k).and_then(Value::as_u64).unwrap_or(0);
                let cache = t.get("cache");
                let doc = cache
                    .and_then(|c| c.get("read"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let ghi = cache
                    .and_then(|c| c.get("write"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let tong = so("input") + doc + ghi;
                if tong > 0 {
                    ngu_canh.insert(sid.clone(), tong);
                }
            }
        }
        if xong.contains_key(&sid) {
            continue;
        }
        let Some(items) = v.get("content").and_then(Value::as_array) else {
            continue;
        };
        let mut text = String::new();
        for it in items {
            if it.get("type").and_then(Value::as_str) != Some("text") {
                continue;
            }
            let Some(t) = it.get("text").and_then(Value::as_str) else {
                continue;
            };
            let t = t.trim();
            if t.is_empty() {
                continue;
            }
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(t);
        }
        if !text.is_empty() {
            xong.insert(sid, text);
        }
    }
    for s in sessions.iter_mut() {
        if let Some(n) = ngu_canh.get(&s.id) {
            s.context_tokens = *n;
        }
        if let Some(t) = xong.get(&s.id) {
            if s.last_text.is_none() {
                s.last_text = Some(t.clone());
                s.last_role = Some("assistant".into());
            }
        }
    }
    Ok(())
}

/// Đếm phiên trong sổ — dùng cho chẩn đoán, không phải "phiên đang sống".
pub fn count_sessions(acc: &ClaudeAccountCfg) -> Result<usize, String> {
    read_sessions(acc).map(|v| v.len())
}

/// `{"id":"deepseek-v4.1-flash","providerID":"opencode-go"}` → `opencode-go/deepseek-v4.1-flash`.
///
/// Trả `None` khi cột model trống hoặc không phải JSON — KHÔNG bịa tên model.
pub fn parse_model(raw: &Option<String>) -> Option<String> {
    let raw = raw.as_deref()?.trim();
    if raw.is_empty() {
        return None;
    }
    let v: Value = serde_json::from_str(raw).ok()?;
    let id = v.get("id").and_then(Value::as_str)?.trim();
    let provider = v
        .get("providerID")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if id.is_empty() {
        return None;
    }
    Some(if provider.is_empty() {
        id.to_string()
    } else {
        format!("{provider}/{id}")
    })
}

// ── Cửa sổ ↔ phiên ─────────────────────────────────────────────────────────

/// Một phiên đã được huba mở trong một cửa sổ Terminal nào.
///
/// opencode không ghi pid/tty vào sổ của nó, nên liên kết này do HUB A ghi lúc
/// mở cửa sổ. Nó nằm TRONG phân vùng của tài khoản (`huba/windows.json`), nên
/// hai tài khoản không bao giờ chung một tên phiên.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowBinding {
    pub window: i64,
    pub tty: String,
    pub at_ms: i64,
}

pub fn bindings_path(acc: &ClaudeAccountCfg) -> PathBuf {
    profile_dir(acc).join("huba").join("windows.json")
}

pub fn read_bindings(acc: &ClaudeAccountCfg) -> Result<BTreeMap<String, WindowBinding>, String> {
    let p = bindings_path(acc);
    let text = match std::fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(format!("đọc {}: {e}", p.display())),
    };
    serde_json::from_str(&text).map_err(|e| {
        format!(
            "{} không phải JSON hợp lệ: {e} — KHÔNG ghi đè, sửa/xoá tay trước",
            p.display()
        )
    })
}

/// Ghi một liên kết phiên → cửa sổ. Ghi tmp + rename, không để tệp nửa vời.
pub fn write_binding(
    acc: &ClaudeAccountCfg,
    session_id: &str,
    window: i64,
    tty: &str,
) -> Result<(), String> {
    let p = bindings_path(acc);
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("tạo {}: {e}", dir.display()))?;
    }
    let mut all = read_bindings(acc)?;
    all.insert(
        session_id.to_string(),
        WindowBinding {
            window,
            tty: tty.to_string(),
            at_ms: chrono::Utc::now().timestamp_millis(),
        },
    );
    let text =
        serde_json::to_string_pretty(&all).map_err(|e| format!("dựng JSON liên kết: {e}"))? + "\n";
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("ghi {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &p).map_err(|e| format!("đổi tên vào {}: {e}", p.display()))?;
    Ok(())
}

/// Phiên MỚI NHẤT sinh sau mốc `since_ms` trong đúng thư mục ấy.
///
/// Nhiều hơn một ứng viên ⇒ `None` và một dòng log: hai phiên cùng thư mục mở
/// trong cùng một khoảng thì không có phép đo nào tách được chúng, và ghép bừa
/// là gõ vào việc của phiên khác. Fail-closed.
pub fn newest_session_since(
    acc: &ClaudeAccountCfg,
    dir: &Path,
    since_ms: i64,
) -> Result<Option<OcSession>, String> {
    let wanted = dir.to_string_lossy();
    let mut hits: Vec<OcSession> = read_sessions(acc)?
        .into_iter()
        .filter(|s| s.created_ms >= since_ms && s.directory == wanted)
        .collect();
    hits.sort_by_key(|s| s.created_ms);
    match hits.len() {
        0 => Ok(None),
        1 => Ok(hits.pop()),
        n => {
            crate::logging::warn(
                "opencode_bind_ambiguous",
                json!({ "account": acc.name, "dir": wanted, "count": n,
                        "effect": "không gắn cửa sổ vào phiên nào — tránh gõ nhầm phiên" }),
            );
            Ok(None)
        }
    }
}

// ── /ask + /handover ───────────────────────────────────────────────────────

/// Kết quả một lượt fork opencode, đã đọc từ luồng JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct ForkAnswer {
    pub new_session_id: String,
    pub text: String,
    pub cost_usd: f64,
}

/// Đọc luồng `opencode run --format json` (mỗi dòng một object).
///
/// Bốn thứ phải đúng, và mỗi thứ là một cách hỏng đã gặp ở CLI Claude:
/// * chỉ lấy `type == "text"` — `step_start`/`step_finish` không phải câu trả lời;
/// * `sessionID` phải là id MỚI (fork), không phải id gốc;
/// * cộng `cost` của mọi `step_finish`;
/// * dòng hỏng thì BỎ QUA nhưng đếm — một luồng toàn dòng hỏng không được đọc
///   thành "câu trả lời rỗng".
pub fn parse_run_stream(stdout: &str, source_id: &str) -> Result<ForkAnswer, String> {
    let mut text = String::new();
    let mut new_id = String::new();
    let mut cost = 0.0f64;
    let mut lines = 0usize;
    let mut bad = 0usize;
    for line in stdout.lines().map(str::trim).filter(|l| !l.is_empty()) {
        lines += 1;
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            bad += 1;
            continue;
        };
        if let Some(sid) = v.get("sessionID").and_then(Value::as_str) {
            if sid != source_id {
                new_id = sid.to_string();
            }
        }
        match v.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(t) = v
                    .get("part")
                    .and_then(|p| p.get("text"))
                    .and_then(Value::as_str)
                {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(t);
                }
            }
            Some("step_finish") => {
                if let Some(c) = v
                    .get("part")
                    .and_then(|p| p.get("cost"))
                    .and_then(Value::as_f64)
                {
                    cost += c;
                }
            }
            _ => {}
        }
    }
    if lines > 0 && bad == lines {
        return Err(format!("luồng JSON toàn dòng hỏng ({bad}/{lines})"));
    }
    if new_id.is_empty() {
        return Err(format!(
            "không nhận được session id mới từ fork (nguồn {source_id}, {lines} dòng)"
        ));
    }
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("fork xong nhưng không có chữ trả lời".into());
    }
    Ok(ForkAnswer {
        new_session_id: new_id,
        text,
        cost_usd: cost,
    })
}

/// Chạy một lượt fork trên phiên opencode — nguồn KHÔNG bị chạm.
///
/// `account_call` là cửa duy nhất: nó mang theo dòng launch (phân vùng, khoá,
/// model) và chạy qua `/bin/zsh` nên không phải chép lại cách nạp env. Agent
/// `huba-fork` (khai trong `OPENCODE_CONFIG_CONTENT`) khoá quyền ghi.
pub fn fork_call(
    cfg: &crate::config::Config,
    acc: &ClaudeAccountCfg,
    source_id: &str,
    cwd: &Path,
    prompt: &str,
) -> Result<ForkAnswer, String> {
    let args: Vec<&str> = vec![
        "run",
        "--format",
        "json",
        "--session",
        source_id,
        "--fork",
        "--agent",
        "huba-fork",
        prompt,
    ];
    let (prog, argv) = crate::sessions::account_call(cfg, Some(&acc.name), &args);
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let out = exec::run(
        &prog,
        &argv,
        RunOpts {
            cwd: cwd.is_dir().then_some(cwd),
            timeout: Some(Duration::from_secs(cfg.call.timeout_sec)),
            ..Default::default()
        },
    )
    .map_err(|e| format!("chạy opencode fork: {e}"))?;
    if out.timed_out {
        return Err(format!("quá {}s", cfg.call.timeout_sec));
    }
    if out.code != Some(0) {
        return Err(format!(
            "opencode exit {:?}: {}",
            out.code,
            exec::truncate(out.stderr.trim(), 300)
        ));
    }
    parse_run_stream(&out.stdout, source_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_model_doc_dung_provider_va_id() {
        let raw = Some(r#"{"id":"deepseek-v4.1-flash","providerID":"opencode-go"}"#.to_string());
        assert_eq!(
            parse_model(&raw).as_deref(),
            Some("opencode-go/deepseek-v4.1-flash")
        );
        assert_eq!(parse_model(&Some("".into())), None);
        assert_eq!(parse_model(&Some("không phải json".into())), None);
    }

    #[test]
    fn parse_stream_lay_chu_va_id_moi_va_cost() {
        let stdout = concat!(
            r#"{"type":"step_start","sessionID":"ses_new","part":{}}"#,
            "\n",
            r#"{"type":"text","sessionID":"ses_new","part":{"type":"text","text":"Xin chào"}}"#,
            "\n",
            r#"{"type":"text","sessionID":"ses_new","part":{"type":"text","text":"Thế giới"}}"#,
            "\n",
            r#"{"type":"step_finish","sessionID":"ses_new","part":{"reason":"stop","cost":0.25}}"#,
            "\n",
        );
        let a = parse_run_stream(stdout, "ses_cu").expect("phải đọc được");
        assert_eq!(a.new_session_id, "ses_new");
        assert_eq!(a.text, "Xin chào\nThế giới");
        assert!((a.cost_usd - 0.25).abs() < 1e-9);
    }

    #[test]
    fn parse_stream_tu_choi_khi_id_khong_doi() {
        let stdout = r#"{"type":"text","sessionID":"ses_cu","part":{"type":"text","text":"x"}}"#;
        let err = parse_run_stream(stdout, "ses_cu").unwrap_err();
        assert!(err.contains("session id mới"), "{err}");
    }

    #[test]
    fn parse_stream_tu_choi_khi_toan_dong_hong() {
        let err = parse_run_stream("rác\nrác", "ses_cu").unwrap_err();
        assert!(err.contains("toàn dòng hỏng"), "{err}");
    }

    #[test]
    fn doc_bang_sqlite_that_va_anh_xa_token() {
        let tmp = std::env::temp_dir().join(format!(
            "huba-oc-test-{}-{}",
            std::process::id(),
            crate::logging::now_iso().replace([':', '.', 'T', 'Z'], "_")
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = tmp.join("opencode.db");
        {
            let c = Connection::open(&db).unwrap();
            c.execute_batch(
                "CREATE TABLE session (
                    id TEXT PRIMARY KEY, project_id TEXT NOT NULL DEFAULT '',
                    workspace_id TEXT, parent_id TEXT, slug TEXT NOT NULL DEFAULT '',
                    directory TEXT NOT NULL, path TEXT, title TEXT NOT NULL,
                    version TEXT NOT NULL DEFAULT '', share_url TEXT,
                    summary_additions INTEGER, summary_deletions INTEGER,
                    summary_files INTEGER, summary_diffs TEXT, metadata TEXT,
                    cost REAL DEFAULT 0 NOT NULL,
                    tokens_input INTEGER DEFAULT 0 NOT NULL,
                    tokens_output INTEGER DEFAULT 0 NOT NULL,
                    tokens_reasoning INTEGER DEFAULT 0 NOT NULL,
                    tokens_cache_read INTEGER DEFAULT 0 NOT NULL,
                    tokens_cache_write INTEGER DEFAULT 0 NOT NULL,
                    revert TEXT, permission TEXT, agent TEXT, model TEXT,
                    time_created INTEGER NOT NULL, time_updated INTEGER NOT NULL,
                    time_compacting INTEGER, time_archived INTEGER
                );
                INSERT INTO session (id,directory,title,model,cost,tokens_input,
                    tokens_cache_read,tokens_cache_write,time_created,time_updated)
                VALUES ('ses_1','/tmp/p','Một phiên',
                    '{\"id\":\"deepseek-v4.1-flash\",\"providerID\":\"opencode-go\"}',
                    0.5, 100, 20, 3, 1000, 2000);",
            )
            .unwrap();
        }
        let got = read_sessions_at(&db).expect("đọc được");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].id, "ses_1");
        assert_eq!(
            got[0].model.as_deref(),
            Some("opencode-go/deepseek-v4.1-flash")
        );
        // Cột `tokens_*` là CỘNG DỒN — không phải ngữ cảnh hiện tại.
        assert_eq!(got[0].cum_tokens, 123);
        assert_eq!(
            got[0].context_tokens, 0,
            "không có `tokens` của lượt ⇒ không bịa"
        );
        assert_eq!(got[0].created_ms, 1000);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Lược đồ MỚI: opencode ghi `session_v2` + `session_message`, và KHÔNG chép
    /// hàng cũ sang bảng `session`. Bài này là đối chứng ngược cho lỗi đo
    /// 2026-11-12: chỉ đọc `session` ⇒ thấy 3 phiên chết, giấu phiên đang chạy.
    #[test]
    fn doc_bang_session_v2_va_luot_co_chu_cuoi() {
        let tmp = std::env::temp_dir().join(format!(
            "huba-oc-v2-{}-{}",
            std::process::id(),
            crate::logging::now_iso().replace([':', '.', 'T', 'Z'], "_")
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = tmp.join("opencode.db");
        {
            let c = Connection::open(&db).unwrap();
            c.execute_batch(
                "CREATE TABLE session_v2 (
                    id TEXT PRIMARY KEY, project_id TEXT NOT NULL DEFAULT '',
                    workspace_id TEXT, parent_id TEXT, fork_session_id TEXT,
                    slug TEXT NOT NULL DEFAULT '', directory TEXT NOT NULL,
                    path TEXT, title TEXT, version TEXT NOT NULL DEFAULT '',
                    cost REAL DEFAULT 0 NOT NULL,
                    tokens_input INTEGER DEFAULT 0 NOT NULL,
                    tokens_output INTEGER DEFAULT 0 NOT NULL,
                    tokens_reasoning INTEGER DEFAULT 0 NOT NULL,
                    tokens_cache_read INTEGER DEFAULT 0 NOT NULL,
                    tokens_cache_write INTEGER DEFAULT 0 NOT NULL,
                    model TEXT, time_created INTEGER NOT NULL,
                    time_updated INTEGER NOT NULL, time_archived INTEGER
                );
                CREATE TABLE session_message (
                    id TEXT PRIMARY KEY, session_id TEXT NOT NULL,
                    type TEXT NOT NULL, seq INTEGER NOT NULL,
                    time_created INTEGER NOT NULL, time_updated INTEGER NOT NULL,
                    data TEXT NOT NULL
                );
                INSERT INTO session_v2 (id,directory,title,model,cost,tokens_input,
                    tokens_cache_read,time_created,time_updated) VALUES
                  ('ses_live','/Users/x/projects','Phiên đang chạy',
                   '{\"id\":\"deepseek-v4.1-flash\",\"providerID\":\"opencode-go\"}',
                   0.25, 400, 20, 5000, 9000),
                  ('ses_con','/Users/x/projects','Phiên con của subagent',
                   NULL, 0, 0, 0, 5000, 9500),
                  ('ses_cu','/Users/x/projects','Phiên cũ', NULL, 0, 0, 0, 1000, 2000);
                UPDATE session_v2 SET parent_id='ses_live' WHERE id='ses_con';
                INSERT INTO session_message (id,session_id,type,seq,time_created,
                    time_updated,data) VALUES
                  ('m1','ses_live','assistant',1,6000,6000,
                   '{\"content\":[{\"type\":\"text\",\"text\":\"Xong việc.\"}]}'),
                  ('m2','ses_live','assistant',2,7000,7000,
                   '{\"content\":[{\"type\":\"reasoning\",\"text\":\"nghĩ\"},
                                   {\"type\":\"tool\",\"tool\":\"bash\"}],
                     \"tokens\":{\"input\":10,\"output\":5,\"cache\":{\"read\":50,\"write\":2}}}'),
                  ('m3','ses_live','user',3,8000,8000,
                   '{\"content\":[{\"type\":\"text\",\"text\":\"câu của người\"}]}');",
            )
            .unwrap();
        }
        let got = read_sessions_at(&db).expect("đọc được");
        // Phiên con (parent_id) không lên danh sách; hai phiên gốc thì có.
        let ids: Vec<&str> = got.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["ses_live", "ses_cu"], "thứ tự + lọc phiên con");
        let live = &got[0];
        assert_eq!(live.title, "Phiên đang chạy");
        assert_eq!(
            live.model.as_deref(),
            Some("opencode-go/deepseek-v4.1-flash")
        );
        // Ngữ cảnh HIỆN TẠI = tokens của lượt mới nhất (10+50+2), không phải
        // cột cộng dồn 420 — hai số này lệch nhau 5 bậc trên phiên thật.
        assert_eq!(live.context_tokens, 62);
        assert_eq!(live.cum_tokens, 420);
        // Lượt assistant CUỐI CÓ CHỮ, không phải lượt cuối cùng (toàn reasoning).
        assert_eq!(live.last_text.as_deref(), Some("Xong việc."));
        assert_eq!(live.last_role.as_deref(), Some("assistant"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Kho CŨ không có `session_v2` phải đọc được như trước (fail-closed chỉ khi
    /// hỏng thật, không phải khi thiếu bảng của bản mới).
    #[test]
    fn kho_cu_khong_co_bang_v2_van_doc_duoc() {
        let tmp = std::env::temp_dir().join(format!(
            "huba-oc-{}-{}",
            std::process::id(),
            crate::logging::now_iso().replace([':', '.', 'T', 'Z'], "_")
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = tmp.join("opencode.db");
        {
            let c = Connection::open(&db).unwrap();
            c.execute_batch(
                "CREATE TABLE session (
                    id TEXT PRIMARY KEY, title TEXT NOT NULL,
                    directory TEXT NOT NULL, model TEXT, cost REAL DEFAULT 0,
                    tokens_input INTEGER DEFAULT 0, tokens_output INTEGER DEFAULT 0,
                    tokens_reasoning INTEGER DEFAULT 0,
                    tokens_cache_read INTEGER DEFAULT 0,
                    tokens_cache_write INTEGER DEFAULT 0,
                    parent_id TEXT, time_created INTEGER NOT NULL,
                    time_updated INTEGER NOT NULL, time_archived INTEGER
                );
                INSERT INTO session (id,title,directory,time_created,time_updated)
                VALUES ('ses_cu','Một phiên','/Users/x',10,20);",
            )
            .unwrap();
        }
        let got = read_sessions_at(&db).expect("đọc được");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].id, "ses_cu");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
