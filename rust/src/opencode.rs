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
    /// Đuôi phiên dựng lại theo HÌNH DẠNG nhật ký Claude (mỗi dòng một bản
    /// ghi), để nhãn dự án đi CHUNG cửa `sessions::folder_for_session` với phiên
    /// Claude — xem [`fill_tails_v2`]. Rỗng khi kho không có `session_message`.
    #[serde(skip)]
    pub tail: String,
    /// `true` bận · `false` rảnh · `None` kho không khai — từ `time_busy` /
    /// `time_idle` của `session_v2` ([`fill_state_v2`]).
    #[serde(skip)]
    pub busy: Option<bool>,
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
    let fork_cua_huba = fill_state_v2(&conn, &mut out)?;
    if !fork_cua_huba.is_empty() {
        // Luật 11b: máy của huba không bao giờ được hiện thành một phiên. Phiên
        // fork của `/ask` (agent `huba-fork`) là phép dò, không phải việc của ai.
        crate::logging::info(
            "opencode_huba_forks_hidden",
            serde_json::json!({ "count": fork_cua_huba.len() }),
        );
        out.retain(|s| !fork_cua_huba.contains(&s.id));
    }
    fill_tails_v2(&conn, &mut out)?;
    Ok(out)
}

/// Trạng thái bận/rảnh + nhận ra fork của chính huba, từ cột `session_v2`.
///
/// Quang 12/11 *"phải cùng trải nghiệm"*: hàng Claude có `busy`/`idle`, hàng
/// opencode `status=null` ⟹ danh sách không nói phiên nào đang làm. Kho mới có
/// `time_busy`/`time_idle` (đo acc8: phiên đang chạy có `time_busy`, `time_idle`
/// rỗng; phiên đã xong có `time_idle` > `time_busy`) và `agent` (fork của `/ask`
/// mang `huba-fork`). Kho thiếu cột ⟹ để `None`, không đoán.
/// Trả về id các phiên là fork của huba.
fn fill_state_v2(
    conn: &Connection,
    sessions: &mut [OcSession],
) -> Result<std::collections::BTreeSet<String>, String> {
    let mut forks = std::collections::BTreeSet::new();
    if !has_table(conn, "session_v2")? {
        return Ok(forks);
    }
    let mut cols = conn
        .prepare("SELECT name FROM pragma_table_info('session_v2')")
        .map_err(|e| format!("đọc cột session_v2: {e}"))?;
    let cols: Vec<String> = cols
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| format!("đọc cột session_v2: {e}"))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("đọc cột session_v2: {e}"))?;
    let co = |c: &str| cols.iter().any(|x| x == c);
    let col = |c: &str| {
        if co(c) {
            c.to_string()
        } else {
            "NULL".to_string()
        }
    };
    if !co("agent") && !(co("time_busy") && co("time_idle")) {
        return Ok(forks);
    }
    let sql = format!(
        "SELECT id, {}, {}, {} FROM session_v2",
        col("agent"),
        col("time_busy"),
        col("time_idle")
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("đọc trạng thái session_v2: {e}"))?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, Option<i64>>(3)?,
            ))
        })
        .map_err(|e| format!("truy vấn trạng thái session_v2: {e}"))?;
    let mut state: BTreeMap<String, Option<bool>> = BTreeMap::new();
    for row in rows {
        let (id, agent, busy_at, idle_at) =
            row.map_err(|e| format!("đọc trạng thái session_v2: {e}"))?;
        if agent.as_deref() == Some("huba-fork") {
            forks.insert(id.clone());
        }
        state.insert(id, busy_state(busy_at, idle_at));
    }
    for s in sessions.iter_mut() {
        if let Some(b) = state.get(&s.id) {
            s.busy = *b;
        }
    }
    Ok(forks)
}

/// Phần THUẦN: bận khi mốc bận mới hơn mốc rảnh (hoặc chưa rảnh lần nào).
pub fn busy_state(busy_at: Option<i64>, idle_at: Option<i64>) -> Option<bool> {
    match (busy_at, idle_at) {
        (Some(b), Some(i)) => Some(b > i),
        (Some(_), None) => Some(true),
        (None, Some(_)) => Some(false),
        (None, None) => None,
    }
}

/// Số phiên MỚI NHẤT được dựng đuôi — danh sách điện thoại chỉ hiện chừng ấy,
/// và mỗi đuôi là vài chục bản ghi JSON (đo 12/11: tới 26 KB một bản ghi).
const TAIL_SESSIONS: usize = 30;
/// Số bản ghi cuối mỗi phiên — đủ phủ `KHAI_QUET_LUOT` lượt nói của bộ đọc
/// lời tự khai, kể cả khi xen giữa là lượt chỉ-gọi-công-cụ.
const TAIL_ROWS: usize = 40;
/// Trần ký tự cho một bản ghi thô trong đuôi (chỉ để ĐẾM đường dẫn).
const TAIL_RAW_CHARS: usize = 6000;

/// Dựng đuôi phiên theo hình dạng nhật ký Claude cho [`OcSession::tail`].
///
/// 🔴 Quang 2026-11-12: *"làm sao để chạy được như dùng acc khác, nó phải cùng
/// trải nghiệm chứ"*. Đo cùng phút: `/new acc3 …` và `/new acc8 aihub` cùng mở
/// ở `~/projects`; hàng Claude lên `[huba]` (nhãn đọc từ lời tự khai + đường
/// dẫn trong nhật ký), hàng opencode KẸT `[projects]` dù 20/40 bản ghi của nó
/// nhắc `~/projects/aihub/…` — vì nhánh opencode chỉ nhìn THƯ MỤC MỞ PHIÊN.
///
/// Không chép luật nhãn sang đây: mỗi lượt nói assistant thành một dòng
/// `{"type":"assistant","message":{"content":[{"type":"text",…}]}}` (đúng thứ
/// `declared_in_tail_counted` đọc), và bản ghi thô đi kèm ở dòng riêng cho
/// `folder_from_tail` đếm đường dẫn của công cụ. Bản ghi thô bị cắt ⇒ không còn
/// là JSON hợp lệ ⇒ bộ đọc lời khai tự bỏ qua; bộ đếm chỉ tìm chuỗi nên vẫn đếm.
fn fill_tails_v2(conn: &Connection, sessions: &mut [OcSession]) -> Result<(), String> {
    use rusqlite::params_from_iter;
    if !has_table(conn, "session_message")? {
        return Ok(());
    }
    let ids: Vec<&str> = sessions
        .iter()
        .take(TAIL_SESSIONS)
        .map(|s| s.id.as_str())
        .collect();
    if ids.is_empty() {
        return Ok(());
    }
    let ph = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT session_id, type, data FROM ( \
             SELECT session_id, type, data, seq, \
                    ROW_NUMBER() OVER (PARTITION BY session_id ORDER BY seq DESC) rn \
             FROM session_message WHERE session_id IN ({ph}) \
         ) WHERE rn <= {TAIL_ROWS} ORDER BY session_id, seq ASC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("đọc đuôi session_message: {e}"))?;
    let rows = stmt
        .query_map(params_from_iter(ids.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("truy vấn đuôi session_message: {e}"))?;
    let mut tails: BTreeMap<String, String> = BTreeMap::new();
    for row in rows {
        let (sid, kind, data) = row.map_err(|e| format!("đọc đuôi session_message: {e}"))?;
        let tail = tails.entry(sid).or_default();
        tail.push_str(&exec::truncate(&data, TAIL_RAW_CHARS).replace('\n', " "));
        tail.push('\n');
        if kind != "assistant" {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        let text = v
            .get("content")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter(|it| it.get("type").and_then(Value::as_str) == Some("text"))
                    .filter_map(|it| it.get("text").and_then(Value::as_str))
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        let rec = serde_json::json!({
            "type": "assistant",
            "message": { "content": [ { "type": "text", "text": text } ] }
        });
        tail.push_str(&rec.to_string());
        tail.push('\n');
    }
    for s in sessions.iter_mut() {
        if let Some(t) = tails.remove(&s.id) {
            s.tail = t;
        }
    }
    Ok(())
}

/// Một cửa sổ hạn mức gói Go — đúng kiểu `UsageLimits.Meter` của opencode.
#[derive(Debug, Clone, PartialEq)]
pub struct GoMeter {
    pub percent: i64,
    pub limited: bool,
    pub resets_at: Option<String>,
}

/// Hạn mức gói Go do MÁY CHỦ opencode trả (`UsageLimits.Go`).
#[derive(Debug, Clone, PartialEq)]
pub struct GoLimits {
    pub rolling: GoMeter,
    pub weekly: GoMeter,
    pub monthly: GoMeter,
}

/// 🔴 Quang 12/11: *"Không có cách nào lấy được thông tin từ cli à"*. CÓ: chính
/// opencode 2.2.7 hỏi `GET <origin>/zen/go/v1/usage` (Bearer khoá, hàm `tC` trong
/// binary) và trả `{usage:{rolling,weekly,monthly}}`, mỗi cửa sổ
/// `{status:"ok"|"rate-limited", percent, resetsAt}`. Đo 12/11 05:5xZ: acc8 và
/// acc9 trả CÙNG số (một gói, hai khoá) — rolling 0 · weekly 50 · monthly 75;
/// header `x-opencode-org-id` không bắt buộc. Bản tự cộng token trong kho acc8
/// trước đó ra tuần 1% — SAI 50 lần vì không thấy lượt của khoá khác ⟹ đã gỡ.
pub const GO_USAGE_URL: &str = "https://opencode.ai/zen/go/v1/usage";

/// Phần THUẦN: thân JSON của `/zen/go/v1/usage` → [`GoLimits`].
pub fn parse_go_limits(body: &Value) -> Result<GoLimits, String> {
    let meter = |k: &str| -> Result<GoMeter, String> {
        let m = body
            .pointer(&format!("/usage/{k}"))
            .ok_or_else(|| format!("thiếu usage.{k}"))?;
        let percent = m
            .get("percent")
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("usage.{k}.percent không phải số"))?;
        let status = m.get("status").and_then(Value::as_str).unwrap_or("");
        Ok(GoMeter {
            percent: percent.ceil() as i64,
            limited: status == "rate-limited",
            resets_at: m
                .get("resetsAt")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    };
    Ok(GoLimits {
        rolling: meter("rolling")?,
        weekly: meter("weekly")?,
        monthly: meter("monthly")?,
    })
}

/// Đọc khoá của tài khoản từ `env_file` (dòng `[export ]TÊN=giá trị`). Giá trị
/// KHÔNG bao giờ đi vào log (luật 4) — lỗi chỉ nêu tên biến và đường dẫn.
fn read_key(acc: &ClaudeAccountCfg) -> Result<String, String> {
    let file = acc
        .env_file
        .as_deref()
        .filter(|f| !f.is_empty())
        .ok_or("tài khoản không khai env_file")?;
    let name = acc
        .api_key_env
        .as_deref()
        .filter(|n| !n.is_empty())
        .unwrap_or("OPENCODE_API_KEY");
    let path = crate::config::expand_home(Path::new(file));
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("đọc {}: {e}", path.display()))?;
    text.lines()
        .map(|l| l.trim().trim_start_matches("export ").trim())
        .find_map(|l| l.strip_prefix(&format!("{name}=")))
        .map(|v| v.trim().trim_matches(['"', '\'']).to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("{} không có biến {name}", path.display()))
}

/// Bản đọc nhớ trong tiến trình — mọi đường CHỌN tài khoản gọi `quota::read_all`,
/// có khi nhiều lần một vòng; một lượt mạng ~0,7 s mỗi lần là phí.
type GoCache = BTreeMap<String, (i64, Result<GoLimits, String>)>;
static GO_CACHE: std::sync::OnceLock<std::sync::Mutex<GoCache>> = std::sync::OnceLock::new();
/// Tuổi tối đa của bản đọc nhớ.
pub const GO_CACHE_MS: i64 = 5 * 60_000;

/// Hạn mức gói Go của tài khoản, hỏi máy chủ (nhớ [`GO_CACHE_MS`]). Trả kèm mốc
/// ĐỌC thật (`fetched_at_ms`), không phải mốc gọi hàm.
pub fn go_limits(acc: &ClaudeAccountCfg, now_ms: i64) -> (i64, Result<GoLimits, String>) {
    let cache = GO_CACHE.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()));
    if let Ok(m) = cache.lock() {
        if let Some((at, r)) = m.get(&acc.name) {
            if now_ms - at < GO_CACHE_MS {
                return (*at, r.clone());
            }
        }
    }
    let r = fetch_go_limits(acc);
    if let Ok(mut m) = cache.lock() {
        m.insert(acc.name.clone(), (now_ms, r.clone()));
    }
    (now_ms, r)
}

fn fetch_go_limits(acc: &ClaudeAccountCfg) -> Result<GoLimits, String> {
    let key = read_key(acc)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("không dựng được HTTP client: {e}"))?;
    let resp = client
        .get(GO_USAGE_URL)
        .bearer_auth(key.trim())
        // Cloudflare chặn chữ ký mặc định của vài client (đo 12/11: 403 «1010»
        // với Python-urllib) — tự xưng như opencode.
        .header("user-agent", "opencode/2.2.7")
        .send()
        .map_err(|e| {
            format!(
                "không gọi được {GO_USAGE_URL}: {}",
                exec::truncate(&e.to_string(), 160)
            )
        })?;
    let ma = resp.status().as_u16();
    if ma != 200 {
        return Err(format!("{GO_USAGE_URL} trả HTTP {ma}"));
    }
    let body: Value = resp
        .json()
        .map_err(|e| format!("thân trả về không phải JSON: {e}"))?;
    parse_go_limits(&body)
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
                tail: String::new(),
                busy: None,
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
    /// Thư mục huba mở cửa sổ — cần cho hàng [`phien_cho_cau_dau`], thứ CHƯA có
    /// hàng nào trong kho để đọc `directory`. Tệp ghi trước 13/11 không có ⟹ `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

/// Id phiên opencode do HUBA đặt trước, cùng khuôn opencode tự sinh.
///
/// 🔴 Vì sao (Ubuntu 13/11): `/new -a acc8` không đề bài thì opencode CHƯA có
/// phiên nào — nó chỉ sinh phiên ở câu đầu tiên — nên huba không có id để ghi
/// liên kết cửa sổ. Từ bản vá phiên ma 12/11, hàng không liên kết bị ẩn ⟹ phiên
/// ấy KHÔNG BAO GIỜ lên danh sách (chú thích cũ hứa «tự lên khi gõ câu đầu»).
/// Ghép muộn theo «thư mục + giờ» thì mơ hồ: Quang mở 3 cửa sổ cùng thư mục.
/// opencode 2.2.7 có `--session <id>` = *"continue, or create if it does not
/// exist"*; ĐO trong hồ sơ tạm: TUI hỏi `GET /api/session/<id>` ⟹ 404, và câu
/// đầu tiên sinh phiên mang ĐÚNG id ấy. Nên biết id ngay lúc mở, như `--resume`.
///
/// Khuôn: `ses_` + 12 hex (48 bit thấp của `!(ms·0x1000 + đếm)` — sắp giảm dần
/// theo giờ như id thật, đối chiếu `ses_e2d1955d…` trong kho Ubuntu) + 14 base62.
pub fn tao_id_phien(now_ms: i64) -> String {
    use std::hash::{BuildHasher, Hasher};
    static DEM: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dem = DEM.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let n = (now_ms.max(0) as u64)
        .wrapping_mul(0x1000)
        .wrapping_add(dem & 0xfff);
    let thoi_gian = !n & 0xffff_ffff_ffff;
    const B62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    // `RandomState` lấy hạt từ hệ điều hành — đủ để hai lần mở không trùng,
    // không cần thêm crate (Cargo chạy `--offline`).
    let mut duoi = String::with_capacity(14);
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u64(dem);
    h.write_i64(now_ms);
    let mut r = h.finish();
    for i in 0..14 {
        if i == 7 {
            let mut h2 = std::collections::hash_map::RandomState::new().build_hasher();
            h2.write_u64(r);
            r = h2.finish();
        }
        duoi.push(B62[(r % 62) as usize] as char);
        r /= 62;
    }
    format!("ses_{thoi_gian:012x}{duoi}")
}

/// Hàng «phiên chờ câu đầu»: liên kết huba đã ghi cho một id đặt trước
/// ([`tao_id_phien`]) mà kho CHƯA có hàng, và TUI ở tty ấy còn sống.
///
/// Không có hàng này thì cửa sổ vừa mở chỉ lên dạng `win-<tty>`, con trỏ trỏ
/// vào cái tên ấy, và câu đầu tiên sinh phiên `ses_…` làm hàng trần biến mất —
/// con trỏ treo. Có nó thì MỘT id đi suốt đời phiên.
///
/// Ba luật, đều fail-closed (thà không bịa hàng):
/// * `tui_song = None` (`ps` mù) ⟹ rỗng — không phân biệt được sống/chết.
/// * chỉ id khuôn opencode, tty khác rỗng và có TUI sống.
/// * tty là của liên kết MỚI NHẤT trên nó ([`tty_cua_lien_ket_moi_nhat`]) —
///   tty được dùng lại (luật 11b), liên kết cũ không được mượn TUI mới.
pub fn phien_cho_cau_dau(
    bindings: &BTreeMap<String, WindowBinding>,
    kho_ids: &std::collections::HashSet<String>,
    tui_song: Option<&std::collections::HashSet<String>>,
) -> Vec<(String, WindowBinding)> {
    let Some(song) = tui_song else {
        return Vec::new();
    };
    bindings
        .iter()
        .filter(|(id, _)| id.starts_with("ses_") && !kho_ids.contains(*id))
        .filter(|(_, b)| {
            let t = b.tty.trim_start_matches("/dev/");
            !t.is_empty() && song.contains(t)
        })
        .filter(|(id, b)| tty_cua_lien_ket_moi_nhat(bindings, id, b))
        .map(|(id, b)| (id.clone(), b.clone()))
        .collect()
}

/// Liên kết `id → b` có phải liên kết MỚI NHẤT trên tty của nó không.
///
/// Cùng một `pts/1` lần lượt mang nhiều cửa sổ; liên kết cũ hơn trên tty ấy là
/// phiên đã rời cửa sổ, dù TUI hiện ở đó còn sống. Bằng giờ ⟹ id lớn hơn thắng
/// (thứ tự cố định, không phụ thuộc thứ tự đọc).
pub fn tty_cua_lien_ket_moi_nhat(
    bindings: &BTreeMap<String, WindowBinding>,
    id: &str,
    b: &WindowBinding,
) -> bool {
    let t = b.tty.trim_start_matches("/dev/");
    !bindings.iter().any(|(k, o)| {
        k != id
            && o.tty.trim_start_matches("/dev/") == t
            && (o.at_ms > b.at_ms || (o.at_ms == b.at_ms && k.as_str() > id))
    })
}

/// Id sau `--session` trong dòng lệnh một TUI opencode (bỏ nháy). Thuần.
pub fn session_trong_argv(command: &str) -> Option<String> {
    let mut it = command.split_whitespace();
    while let Some(t) = it.next() {
        if t == "--session" {
            let id = it.next()?.trim_matches(|c| c == '\'' || c == '"');
            return id.starts_with("ses_").then(|| id.to_string());
        }
    }
    None
}

/// Liên kết còn THIẾU, đọc từ TUI đang sống: `(id, tty)` cho mỗi TUI mang
/// `--session <id>` mà id đã có trong kho của tài khoản này và chưa có liên kết.
///
/// 🔴 Quang 13/11 (mac): phiên acc8 treo bảng hỏi mà huba không báo, không đọc
/// được màn. Cửa sổ do `acc-mo-vai.sh` mở qua `huba lenh-mo` — huba chỉ IN dòng
/// lệnh, không phải nó mở cửa sổ, nên không có lúc nào ghi `windows.json` như
/// `/new` ⟹ phiên bị ẩn (luật phiên ma), chỉ còn hàng `win-<tty>`. Nay `lenh-mo`
/// đặt trước id, và mỗi vòng huba tự ghép từ `ps`.
///
/// Fail-closed: id không có trong kho của tài khoản này (TUI của tài khoản khác,
/// hoặc chưa gửi câu đầu) ⟹ bỏ; hai TUI mang cùng id ⟹ bỏ cả hai; đã có liên
/// kết ⟹ không đè (liên kết `/new` / `--resume` ghi lúc mở là nguồn đúng hơn).
pub fn lien_ket_thieu(
    tuis: &[(String, String)],
    bindings: &BTreeMap<String, WindowBinding>,
    kho_ids: &std::collections::HashSet<String>,
) -> Vec<(String, String)> {
    let mut theo_id: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (tty, command) in tuis {
        let t = tty.trim_start_matches("/dev/");
        if !crate::sessions::is_real_tty(t) {
            continue;
        }
        if let Some(id) = session_trong_argv(command) {
            theo_id.entry(id).or_default().push(t.to_string());
        }
    }
    theo_id
        .into_iter()
        .filter(|(id, _)| kho_ids.contains(id) && !bindings.contains_key(id))
        .filter_map(|(id, ttys)| match ttys.as_slice() {
            [mot] => Some((id, mot.clone())),
            _ => {
                crate::logging::warn(
                    "opencode_bind_argv_ambiguous",
                    json!({ "session": id, "ttys": ttys,
                            "effect": "không ghép — hai cửa sổ mang cùng một id" }),
                );
                None
            }
        })
        .collect()
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
    dir: Option<&str>,
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
            dir: dir.map(str::to_string),
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

// ── Máy chủ dịch vụ mang cấu hình CŨ ──────────────────────────────────────
//
// TUI `opencode` 2.x không tự chạy một mình: lần đầu nó dựng `opencode serve
// --service` cho phân vùng (XDG_CONFIG_HOME) rồi NỐI vào; dịch vụ sống tiếp sau
// khi TUI thoát, và giữ nguyên `OPENCODE_CONFIG_CONTENT` lúc dựng. Đo 13/11 trên
// Ubuntu: dịch vụ pid 13351 dựng 09:32 (chưa có `kem`), `/new` lúc 09:57 truyền
// cấu hình MỚI đúng (`/proc/<tui>/environ` có `kem/bin/project-agent`) mà màn vẫn
// «1 MCP failed · 1 plugin failed» — TUI chỉ nối vào dịch vụ cũ. Cấu hình đổi
// (plugin, MCP, bức tường quyền) không bao giờ tới phiên mới cho tới khi dịch vụ chết.

/// Phán quyết cho một tiến trình khi `/new` sắp mở phân vùng `profile_cfg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DichVu {
    /// Không phải dịch vụ của phân vùng này.
    KhongLienQuan,
    /// Dịch vụ của phân vùng này, cùng cấu hình — để yên.
    Khop,
    /// Dịch vụ của phân vùng này, cấu hình KHÁC (hoặc không đọc ra) — phải dựng lại.
    Lech,
}

/// Thuần: `argv` + biến môi trường của một tiến trình ⟹ [`DichVu`]. So cấu hình
/// bằng JSON đã PHÂN TÍCH (thứ tự khoá không phải sự khác biệt).
pub fn xet_dich_vu(
    argv: &[String],
    env: &BTreeMap<String, String>,
    profile_cfg: &str,
    noi_dung: &Value,
) -> DichVu {
    let la_opencode = argv
        .first()
        .and_then(|a| Path::new(a).file_name())
        .is_some_and(|n| n == "opencode");
    let la_dich_vu = argv.iter().any(|a| a == "serve") && argv.iter().any(|a| a == "--service");
    if !la_opencode || !la_dich_vu {
        return DichVu::KhongLienQuan;
    }
    if env.get("XDG_CONFIG_HOME").map(String::as_str) != Some(profile_cfg) {
        return DichVu::KhongLienQuan;
    }
    match env
        .get("OPENCODE_CONFIG_CONTENT")
        .and_then(|c| serde_json::from_str::<Value>(c).ok())
    {
        Some(v) if &v == noi_dung => DichVu::Khop,
        _ => DichVu::Lech,
    }
}

/// TUI (không phải dịch vụ) của cùng phân vùng — còn TUI thì giết dịch vụ là cắt phiên đang sống.
pub fn la_tui_cua(argv: &[String], env: &BTreeMap<String, String>, profile_cfg: &str) -> bool {
    argv.first()
        .and_then(|a| Path::new(a).file_name())
        .is_some_and(|n| n == "opencode")
        && !argv.iter().any(|a| a == "serve" || a == "run")
        && env.get("XDG_CONFIG_HOME").map(String::as_str) == Some(profile_cfg)
}

#[cfg(target_os = "linux")]
fn doc_proc(pid: i32) -> Option<(Vec<String>, BTreeMap<String, String>)> {
    let tach = |b: Vec<u8>| -> Vec<String> {
        b.split(|c| *c == 0)
            .filter(|x| !x.is_empty())
            .map(|x| String::from_utf8_lossy(x).into_owned())
            .collect()
    };
    let argv = tach(std::fs::read(format!("/proc/{pid}/cmdline")).ok()?);
    let env = tach(std::fs::read(format!("/proc/{pid}/environ")).ok()?)
        .into_iter()
        .filter_map(|kv| {
            kv.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect();
    Some((argv, env))
}

/// Trước `/new` một acc opencode: dịch vụ của phân vùng mang cấu hình CŨ thì tắt
/// để TUI dựng lại bằng cấu hình mới — CHỈ khi không còn TUI nào của phân vùng
/// đang nối vào (không cắt phiên đang sống). Mọi nhánh đều ghi log.
///
/// macOS: CHƯA làm — `ps` không đưa biến môi trường của tiến trình khác ra một
/// cách tin được; ghi ở sổ nợ. Nền ấy vẫn mắc đúng lỗi này.
#[cfg(target_os = "linux")]
pub fn don_dich_vu_lech(acc: &ClaudeAccountCfg) {
    let profile_cfg = crate::accounts::opencode_profile_dir(acc)
        .join("config")
        .display()
        .to_string();
    let noi_dung = crate::accounts::opencode_config_content(acc);
    let Ok(rd) = std::fs::read_dir("/proc") else {
        crate::logging::warn(
            "opencode_dich_vu_khong_do_duoc",
            json!({ "account": acc.name, "why": "không đọc được /proc" }),
        );
        return;
    };
    let mut lech: Vec<i32> = Vec::new();
    let mut tuis: Vec<i32> = Vec::new();
    for e in rd.flatten() {
        let Some(pid) = e.file_name().to_str().and_then(|s| s.parse::<i32>().ok()) else {
            continue;
        };
        let Some((argv, env)) = doc_proc(pid) else {
            continue;
        };
        match xet_dich_vu(&argv, &env, &profile_cfg, &noi_dung) {
            DichVu::Lech => lech.push(pid),
            DichVu::Khop | DichVu::KhongLienQuan => {}
        }
        if la_tui_cua(&argv, &env, &profile_cfg) {
            tuis.push(pid);
        }
    }
    if lech.is_empty() {
        return;
    }
    if !tuis.is_empty() {
        crate::logging::warn(
            "opencode_dich_vu_cu_con_phien",
            json!({ "account": acc.name, "dich_vu": lech, "tui": tuis,
                    "effect": "phiên mới nối vào dịch vụ CŨ ⟹ plugin/MCP/bức tường của cấu hình cũ, tới khi đóng hết phiên của acc này" }),
        );
        return;
    }
    for pid in lech {
        let r = exec::run("kill", &[&pid.to_string()], RunOpts::default());
        let mut het = false;
        for _ in 0..30 {
            if !Path::new(&format!("/proc/{pid}")).exists() {
                het = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if het {
            crate::logging::info(
                "opencode_dich_vu_cu_da_tat",
                json!({ "account": acc.name, "pid": pid,
                        "why": "cấu hình khác cấu hình /new sắp mở — TUI sẽ dựng dịch vụ mới" }),
            );
        } else {
            crate::logging::warn(
                "opencode_dich_vu_cu_khong_tat",
                json!({ "account": acc.name, "pid": pid,
                        "kill": format!("{:?}", r.map(|x| x.code)),
                        "effect": "phiên mới vẫn nối vào dịch vụ cấu hình cũ" }),
            );
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn don_dich_vu_lech(_acc: &ClaudeAccountCfg) {}

/// Ghi mức variant TUI nhớ cho `model` vào `model.json` (thuần trên `Value`).
/// `None` ⟹ tệp không phải object như mong đợi — không sửa.
pub fn dat_variant_trong_state(doc: &mut Value, model: &str, muc: &str) -> Option<()> {
    let obj = doc.as_object_mut()?;
    let v = obj.entry("variant").or_insert_with(|| json!({}));
    v.as_object_mut()?.insert(model.to_string(), json!(muc));
    Some(())
}

/// `/new` mở ĐÚNG mức effort của acc (Quang 13/11: *"lẽ ra phải mặc định là high"*).
///
/// Đo trên Ubuntu: cấu hình đã mang `agent.build.variant = "high"` mà màn vẫn
/// Default — vì TUI NHỚ lựa chọn của lần trước ở `state/opencode/model.json`
/// (`"variant":{"<model>":"default"}`, do chính một lượt Ctrl+T để lại) và nó đè
/// cấu hình. Nên trước khi mở TUI, ghi mức của acc vào đúng ô ấy. Ghi tệp mới rồi
/// đổi tên; tệp đọc hỏng ⟹ KHÔNG đụng, ghi log.
pub fn dat_variant_tui(acc: &ClaudeAccountCfg, model: &str, muc: &str) {
    let path = crate::accounts::opencode_profile_dir(acc)
        .join("state")
        .join("opencode")
        .join("model.json");
    let mut doc = match std::fs::read_to_string(&path) {
        Ok(t) => match serde_json::from_str::<Value>(&t) {
            Ok(v) => v,
            Err(e) => {
                crate::logging::warn(
                    "opencode_variant_state_hong",
                    json!({ "account": acc.name, "path": path.display().to_string(),
                            "err": e.to_string(), "effect": "không ghi mức — /new mở mức TUI nhớ" }),
                );
                return;
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            json!({ "recent": [], "favorite": [], "variant": {} })
        }
        Err(e) => {
            crate::logging::warn(
                "opencode_variant_state_hong",
                json!({ "account": acc.name, "path": path.display().to_string(),
                        "err": e.to_string(), "effect": "không ghi mức — /new mở mức TUI nhớ" }),
            );
            return;
        }
    };
    let cu = doc
        .get("variant")
        .and_then(|v| v.get(model))
        .and_then(Value::as_str)
        .map(str::to_string);
    if cu.as_deref() == Some(muc) {
        return;
    }
    if dat_variant_trong_state(&mut doc, model, muc).is_none() {
        crate::logging::warn(
            "opencode_variant_state_hong",
            json!({ "account": acc.name, "path": path.display().to_string(),
                    "err": "model.json không phải object", "effect": "không ghi mức" }),
        );
        return;
    }
    let tmp = path.with_extension("json.huba-tmp");
    let ghi = path
        .parent()
        .map(std::fs::create_dir_all)
        .unwrap_or(Ok(()))
        .and_then(|_| std::fs::write(&tmp, doc.to_string()))
        .and_then(|_| std::fs::rename(&tmp, &path));
    match ghi {
        Ok(()) => crate::logging::info(
            "opencode_variant_tui_dat",
            json!({ "account": acc.name, "model": model, "tu": cu, "thanh": muc }),
        ),
        Err(e) => crate::logging::warn(
            "opencode_variant_state_khong_ghi",
            json!({ "account": acc.name, "path": path.display().to_string(), "err": e.to_string() }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn lk(tty: &str, at_ms: i64) -> WindowBinding {
        WindowBinding {
            window: 5,
            tty: tty.into(),
            at_ms,
            dir: Some("/w".into()),
        }
    }

    fn tap(xs: &[&str]) -> HashSet<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    /// Quang 13/11 mac: cửa sổ acc8 do `acc-mo-vai.sh` mở (qua `lenh-mo`) không
    /// có liên kết ⟹ phiên ẩn, bảng hỏi không báo. Ghép từ `--session` trong argv.
    #[test]
    fn ghep_cua_so_tu_session_trong_argv() {
        let oc = "/Users/x/.opencode/bin/opencode --session ses_e2b689ef7fffHubaDoThu0001A --prompt cd /w && đọc brief";
        assert_eq!(
            session_trong_argv(oc).as_deref(),
            Some("ses_e2b689ef7fffHubaDoThu0001A")
        );
        assert_eq!(
            session_trong_argv("opencode --session 'ses_1'").as_deref(),
            Some("ses_1")
        );
        assert_eq!(session_trong_argv("opencode --prompt cd /w"), None);
        assert_eq!(session_trong_argv("opencode --session"), None);

        let tuis = vec![
            ("ttys004".to_string(), oc.to_string()),
            (
                "ttys011".to_string(),
                "opencode --prompt việc khác".to_string(),
            ),
        ];
        let kho = tap(&["ses_e2b689ef7fffHubaDoThu0001A"]);
        let trong = BTreeMap::new();
        assert_eq!(
            lien_ket_thieu(&tuis, &trong, &kho),
            vec![(
                "ses_e2b689ef7fffHubaDoThu0001A".to_string(),
                "ttys004".to_string()
            )]
        );
        // Đã có liên kết ⟹ không đè.
        let mut co = BTreeMap::new();
        co.insert(
            "ses_e2b689ef7fffHubaDoThu0001A".to_string(),
            lk("ttys009", 1),
        );
        assert!(lien_ket_thieu(&tuis, &co, &kho).is_empty());
        // Id chưa có trong kho của tài khoản này ⟹ không ghép (có thể là acc khác).
        assert!(lien_ket_thieu(&tuis, &trong, &tap(&[])).is_empty());
        // Hai cửa sổ cùng id ⟹ không ghép cái nào.
        let doi = vec![
            ("ttys004".to_string(), oc.to_string()),
            ("ttys005".to_string(), oc.to_string()),
        ];
        assert!(lien_ket_thieu(&doi, &trong, &kho).is_empty());
        // tty không thật (`??`) ⟹ bỏ.
        let mu = vec![("??".to_string(), oc.to_string())];
        assert!(lien_ket_thieu(&mu, &trong, &kho).is_empty());
    }

    #[test]
    fn id_dat_truoc_cung_khuon_id_opencode_that() {
        // Id thật trong kho Ubuntu 13/11: `ses_e2d1955daffewmTXb7FBLtOMq1`,
        // time_created 1794539694647 (id sinh sớm hơn ~18 ms).
        let id = tao_id_phien(1_794_539_694_629);
        assert_eq!(id.len(), "ses_e2d1955daffewmTXb7FBLtOMq1".len(), "{id}");
        assert!(crate::sessions::is_opencode_session_id(&id), "{id}");
        assert!(
            id.starts_with("ses_e2d1955"),
            "phần giờ phải giải như id thật: {id}"
        );
        let hex = &id[4..16];
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "{id}"
        );
        assert!(id[16..].chars().all(|c| c.is_ascii_alphanumeric()), "{id}");
        // Sắp GIẢM dần theo giờ, như opencode.
        let sau = tao_id_phien(1_794_539_694_629 + 60_000);
        assert!(sau[4..16] < id[4..16], "{sau} phải nhỏ hơn {id}");
        // Hai lần mở cùng một ms không trùng.
        assert_ne!(tao_id_phien(1), tao_id_phien(1));
    }

    #[test]
    fn cua_so_chua_co_phien_trong_kho_len_hang_cho_cau_dau() {
        let mut b = BTreeMap::new();
        b.insert("ses_moi".to_string(), lk("pts/1", 200));
        let song = tap(&["pts/1"]);
        let r = phien_cho_cau_dau(&b, &HashSet::new(), Some(&song));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, "ses_moi");
        assert_eq!(r[0].1.dir.as_deref(), Some("/w"));
    }

    #[test]
    fn hang_cho_cau_dau_fail_closed() {
        let mut b = BTreeMap::new();
        b.insert("ses_moi".to_string(), lk("pts/1", 200));
        let song = tap(&["pts/1"]);
        // `ps` mù ⟹ không bịa hàng.
        assert!(phien_cho_cau_dau(&b, &HashSet::new(), None).is_empty());
        // TUI đã chết ⟹ không có hàng.
        assert!(phien_cho_cau_dau(&b, &HashSet::new(), Some(&tap(&["pts/9"]))).is_empty());
        // Kho đã có hàng ⟹ hàng THẬT lo, không dựng bản sao.
        assert!(phien_cho_cau_dau(&b, &tap(&["ses_moi"]), Some(&song)).is_empty());
        // Không phải khuôn id opencode / tty rỗng.
        let mut c = BTreeMap::new();
        c.insert("win-ttypts1".to_string(), lk("pts/1", 200));
        c.insert("ses_rong".to_string(), lk("", 300));
        assert!(phien_cho_cau_dau(&c, &HashSet::new(), Some(&song)).is_empty());
    }

    #[test]
    fn tty_dung_lai_thuoc_lien_ket_moi_nhat() {
        // pts/1 từng mang `ses_cu` (chưa gõ câu nào, cửa sổ đã đóng), rồi /new
        // mới nhận lại đúng pts/1 cho `ses_moi`. TUI sống ở pts/1 là của ses_moi.
        let mut b = BTreeMap::new();
        b.insert("ses_cu".to_string(), lk("pts/1", 100));
        b.insert("ses_moi".to_string(), lk("/dev/pts/1", 200));
        let song = tap(&["pts/1"]);
        let r = phien_cho_cau_dau(&b, &HashSet::new(), Some(&song));
        assert_eq!(
            r.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(),
            vec!["ses_moi"]
        );
        assert!(!tty_cua_lien_ket_moi_nhat(&b, "ses_cu", &b["ses_cu"]));
        assert!(tty_cua_lien_ket_moi_nhat(&b, "ses_moi", &b["ses_moi"]));
        // Bằng giờ ⟹ thứ tự cố định (id lớn hơn), đúng MỘT bên thắng.
        let mut d = BTreeMap::new();
        d.insert("ses_a".to_string(), lk("pts/2", 100));
        d.insert("ses_b".to_string(), lk("pts/2", 100));
        let thang: Vec<_> = d
            .iter()
            .filter(|(k, v)| tty_cua_lien_ket_moi_nhat(&d, k, v))
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(thang, vec!["ses_b"]);
    }

    #[test]
    fn lien_ket_cu_khong_co_dir_van_doc_duoc() {
        let cu = r#"{"ses_x":{"window":3,"tty":"ttys004","at_ms":7}}"#;
        let m: BTreeMap<String, WindowBinding> = serde_json::from_str(cu).expect("tệp cũ");
        assert_eq!(m["ses_x"].dir, None);
        let ghi = serde_json::to_string(&m).expect("ghi");
        assert!(!ghi.contains("dir"), "None không ghi ra: {ghi}");
    }

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

    /// Phiên ma 12/11: hàng opencode chỉ sống khi có TUI ở tty liên kết. Dòng
    /// `ps` thật đo trên máy cùng ngày.
    #[test]
    fn nhan_ra_tui_opencode_trong_ps() {
        use crate::sessions::is_opencode_tui;
        assert!(is_opencode_tui(
            "/Users/x/.opencode/bin/opencode --prompt aihub\n\nTRƯỚC KHI LÀM"
        ));
        assert!(is_opencode_tui("opencode"));
        // Service chung và fork `/ask` không phải cửa sổ của ai.
        assert!(!is_opencode_tui(
            "/Users/x/.opencode/bin/opencode serve --service"
        ));
        assert!(!is_opencode_tui(
            "/Users/x/.opencode/bin/opencode run --format json --session ses_x --fork"
        ));
        assert!(!is_opencode_tui("claude --permission-mode auto"));
        assert!(!is_opencode_tui("/usr/bin/opencode-helper"));
        assert!(!is_opencode_tui(""));
    }

    /// Hàng opencode phải có bận/rảnh như hàng Claude, và fork `/ask` của
    /// huba (agent `huba-fork`) KHÔNG lên danh sách — số đo acc8 12/11.
    #[test]
    fn v2_ban_ranh_va_an_fork_cua_huba() {
        assert_eq!(busy_state(Some(5), None), Some(true));
        assert_eq!(busy_state(Some(5), Some(9)), Some(false));
        assert_eq!(busy_state(Some(9), Some(5)), Some(true));
        assert_eq!(busy_state(None, None), None);
        let tmp = std::env::temp_dir().join(format!(
            "huba-oc-state-{}-{}",
            std::process::id(),
            crate::logging::now_iso().replace([':', '.', 'T', 'Z'], "_")
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = tmp.join("opencode.db");
        {
            let c = Connection::open(&db).unwrap();
            c.execute_batch(
                "CREATE TABLE session_v2 (
                    id TEXT PRIMARY KEY, parent_id TEXT, fork_session_id TEXT,
                    directory TEXT NOT NULL, title TEXT, agent TEXT,
                    cost REAL DEFAULT 0 NOT NULL,
                    tokens_input INTEGER DEFAULT 0 NOT NULL,
                    tokens_output INTEGER DEFAULT 0 NOT NULL,
                    tokens_reasoning INTEGER DEFAULT 0 NOT NULL,
                    tokens_cache_read INTEGER DEFAULT 0 NOT NULL,
                    tokens_cache_write INTEGER DEFAULT 0 NOT NULL,
                    model TEXT, time_created INTEGER NOT NULL,
                    time_updated INTEGER NOT NULL, time_archived INTEGER,
                    time_busy INTEGER, time_idle INTEGER
                );
                INSERT INTO session_v2 (id,fork_session_id,directory,title,agent,time_created,time_updated,time_busy,time_idle) VALUES
                  ('ses_chay',NULL,'/p','đang chạy','build',1,9,8,NULL),
                  ('ses_xong',NULL,'/p','xong','build',1,8,3,7),
                  ('ses_fork','ses_chay','/p','x (fork #1)','huba-fork',1,7,4,6),
                  ('ses_tay','ses_chay','/p','fork chủ máy tự làm','build',1,6,NULL,NULL);",
            )
            .unwrap();
        }
        let got = read_sessions_at(&db).expect("đọc được");
        let ids: Vec<&str> = got.iter().map(|s| s.id.as_str()).collect();
        // Fork của HUBA ẩn; fork chủ máy tự tạo (agent khác) thì vẫn là phiên.
        assert_eq!(ids, vec!["ses_chay", "ses_xong", "ses_tay"]);
        assert_eq!(got[0].busy, Some(true));
        assert_eq!(got[1].busy, Some(false));
        assert_eq!(got[2].busy, None);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Quang 12/11: `/new acc8 aihub` mở ở gốc workspace rồi làm việc trong
    /// `aihub`, mà hàng vẫn đọc `[projects]` — trong khi phiên Claude cùng
    /// hoàn cảnh lên đúng dự án. Đuôi dựng từ kho opencode phải cho
    /// `folder_for_session` (cửa nhãn CHUNG) đọc ra dự án thật.
    #[test]
    fn duoi_v2_cho_nhan_du_an_nhu_phien_claude() {
        let tmp = std::env::temp_dir().join(format!(
            "huba-oc-duoi-{}-{}",
            std::process::id(),
            crate::logging::now_iso().replace([':', '.', 'T', 'Z'], "_")
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = tmp.join("opencode.db");
        {
            let c = Connection::open(&db).unwrap();
            c.execute_batch(
                "CREATE TABLE session_v2 (
                    id TEXT PRIMARY KEY, parent_id TEXT, directory TEXT NOT NULL,
                    title TEXT, cost REAL DEFAULT 0 NOT NULL,
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
                INSERT INTO session_v2 (id,directory,title,time_created,time_updated) VALUES
                  ('ses_dem','/Users/x/projects','đếm',1,9),
                  ('ses_khai','/Users/x/projects','khai',1,8);
                INSERT INTO session_message (id,session_id,type,seq,time_created,time_updated,data) VALUES
                  ('d1','ses_dem','user',1,1,1,'{\"content\":[{\"type\":\"text\",\"text\":\"aihub\"}]}'),
                  ('d2','ses_dem','assistant',2,2,2,'{\"content\":[{\"type\":\"tool\",\"state\":{\"input\":{\"path\":\"/Users/x/projects/aihub/MEMORY.md\"}}}]}'),
                  ('d3','ses_dem','assistant',3,3,3,'{\"content\":[{\"type\":\"tool\",\"state\":{\"input\":{\"path\":\"/Users/x/projects/aihub/src/a.ts\"}}}]}'),
                  ('d4','ses_dem','assistant',4,4,4,'{\"content\":[{\"type\":\"text\",\"text\":\"Rig cannot reproduce.\"}]}'),
                  ('k1','ses_khai','assistant',1,1,1,'{\"content\":[{\"type\":\"text\",\"text\":\"[tfl5] Đã đọc sổ.\"}]}');",
            )
            .unwrap();
        }
        let got = read_sessions_at(&db).expect("đọc được");
        let ws = "/Users/x/projects";
        let dem = got.iter().find(|s| s.id == "ses_dem").unwrap();
        let khai = got.iter().find(|s| s.id == "ses_khai").unwrap();
        let id = |s: &str| format!("{s}-{}", std::process::id());
        // Đếm đường dẫn công cụ ⇒ dự án thật, không phải thư mục mở phiên.
        assert_eq!(
            crate::sessions::folder_for_session(&id("dem"), &dem.tail, ws, || None).as_deref(),
            Some("aihub"),
            "đuôi: {}",
            dem.tail
        );
        // Lời tự khai `[tfl5]` ở đầu lượt nói ⇒ đọc được, cùng luật với Claude
        // (luật ấy chỉ nhận tên là THƯ MỤC CÓ THẬT ⇒ dựng gốc thật).
        let ws_that = tmp.join("ws");
        std::fs::create_dir_all(ws_that.join("tfl5")).unwrap();
        assert_eq!(
            crate::sessions::folder_for_session(
                &id("khai"),
                &khai.tail,
                &ws_that.to_string_lossy(),
                || None
            )
            .as_deref(),
            Some("tfl5")
        );
        // Đối chứng: đuôi rỗng (bản trước 12/11) KHÔNG ra được dự án — tức bài
        // này đỏ nếu `fill_tails_v2` bị gỡ.
        assert_eq!(
            crate::sessions::folder_for_session(&id("rong"), "", ws, || None),
            None
        );
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
