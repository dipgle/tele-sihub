//! Lazy cold-start identity evidence, independent of OpenCode's preview window.
//! Read-only; return only a validated `[project/module]`, never a model payload.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use rusqlite::{params, Connection, OpenFlags};

// Unicode White_Space, matching Rust's `char::is_whitespace`; language syntax,
// not an operational threshold. The parity test enumerates every scalar value.
const WHITE_SPACE: &str = "\u{9}\u{a}\u{b}\u{c}\u{d} \u{85}\u{a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}";
type HistoryKey = (PathBuf, String, String);
type HistoryCache = HashMap<HistoryKey, (i64, Option<String>)>;
static CACHE: OnceLock<Mutex<HistoryCache>> = OnceLock::new();

pub(super) fn cached_opencode_head(
    path: &Path,
    session_id: &str,
    workspace_root: &str,
    revision: i64,
) -> Result<Option<String>, String> {
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (
        path.to_path_buf(),
        session_id.to_string(),
        workspace_root.to_string(),
    );
    if let Some((seen, value)) = cache
        .lock()
        .map_err(|_| "identity history cache poisoned")?
        .get(&key)
    {
        if *seen == revision {
            return Ok(value.clone());
        }
    }
    let value = opencode_head(path, session_id, workspace_root)?;
    // Errors are not cached: transient corruption/permission failures retry and
    // remain logged. Successful None is cached too, for project-only sessions.
    cache
        .lock()
        .map_err(|_| "identity history cache poisoned")?
        .insert(key, (revision, value.clone()));
    Ok(value)
}

/// Replay declarations through the shared transition policy, not "newest wins".
/// Query all history but transfer at most42chars per assistant text (the shared
/// parser accepts40 inside brackets). This is lazy and revision-cached above.
/// No title/user/tool content is allowed to provide identity.
pub(super) fn opencode_head(
    path: &Path,
    session_id: &str,
    workspace_root: &str,
) -> Result<Option<String>, String> {
    match std::fs::metadata(path) {
        Ok(meta) if meta.is_file() => {}
        Ok(_) => return Err("identity history path is not a regular file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("inspect identity history file: {e}")),
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("open identity history: {e}"))?;
    let has_messages: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='session_message')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("inspect identity history schema: {e}"))?;
    if !has_messages {
        return Ok(None);
    }
    let malformed: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM session_message
         WHERE session_id=?1 AND type='assistant' AND NOT json_valid(data))",
            [session_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("validate identity history: {e}"))?;
    if malformed {
        return Err("identity history contains malformed assistant JSON".into());
    }
    // Normalize text blocks exactly like fill_tails_v2: trim, exclude empty,
    // join in block order. A later quotation cannot become the opening prefix.
    let opening_trim = format!("{WHITE_SPACE}`*_#>");
    let mut stmt = conn
        .prepare(
            "WITH declarations AS (
           SELECT m.seq,
             (SELECT ltrim(group_concat(part,char(10)),?2) FROM (
                SELECT trim(json_extract(t.value,'$.text'),?3) AS part
                FROM json_each(m.data,'$.content') t
                WHERE json_extract(t.value,'$.type')='text'
                  AND length(trim(json_extract(t.value,'$.text'),?3)) > 0
                ORDER BY CAST(t.key AS INTEGER)
              )) AS text
           FROM session_message m WHERE m.session_id=?1 AND m.type='assistant'
         )
         SELECT substr(text,1,42) FROM declarations
         WHERE text IS NOT NULL ORDER BY seq ASC",
        )
        .map_err(|e| format!("prepare identity history: {e}"))?;
    let rows = stmt
        .query_map(params![session_id, opening_trim, WHITE_SPACE], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|e| format!("query identity history: {e}"))?;
    let mut reducer = super::identity_reducer::IdentityReducer::new(None, super::KHAI_QUET_LUOT);
    for row in rows {
        let text = row.map_err(|e| format!("read identity history: {e}"))?;
        reducer.push(super::declared_parts(&text, workspace_root));
    }
    Ok(reducer.current.map(|(folder, lane)| {
        as_tail(&if lane.is_empty() {
            folder
        } else {
            format!("{folder}/{lane}")
        })
    }))
}

fn as_tail(label: &str) -> String {
    serde_json::json!({
        "type":"assistant",
        "message":{"content":[{"type":"text","text":format!("[{label}]")}]}
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use serde_json::json;

    struct History {
        tmp: tempfile::TempDir,
        db: Connection,
    }

    impl History {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            std::fs::create_dir_all(tmp.path().join("huba")).unwrap();
            std::fs::create_dir_all(tmp.path().join("other")).unwrap();
            let db = Connection::open(tmp.path().join("history.db")).unwrap();
            db.execute_batch(
                "CREATE TABLE session_message(session_id TEXT,type TEXT,seq INTEGER,data TEXT);",
            )
            .unwrap();
            Self { tmp, db }
        }

        fn add(&self, seq: i64, role: &str, content: serde_json::Value) {
            self.db
                .execute(
                    "INSERT INTO session_message VALUES ('ses_fixture',?1,?2,?3)",
                    params![role, seq, json!({"content":content}).to_string()],
                )
                .unwrap();
        }

        fn prefix(&self, seq: i64, text: &str) {
            self.add(seq, "assistant", json!([{"type":"text","text":text}]));
        }

        fn read(&self) -> Option<(String, String)> {
            let root = self.tmp.path().to_string_lossy();
            opencode_head(&self.tmp.path().join("history.db"), "ses_fixture", &root)
                .unwrap()
                .and_then(|tail| super::super::declared_in_tail(&tail, &root))
        }
    }

    #[test]
    fn cold_start_recovers_mid_history_module_beyond_preview_window() {
        let h = History::new();
        h.prefix(1, "Ordinary opening, without a label.");
        h.prefix(40, "[huba/ui] Declared in the middle, not at the start.");
        for seq in 41..140 {
            h.add(
                seq,
                "assistant",
                json!([{"type":"tool","state":{"status":"completed"}}]),
            );
        }
        assert_eq!(h.read(), Some(("huba".into(), "ui".into())));
    }

    #[test]
    fn bare_project_keeps_module_only_within_the_same_project() {
        let h = History::new();
        h.prefix(1, "[huba/ui] Original module.");
        h.prefix(2, "[huba] Same project, no replacement module.");
        assert_eq!(h.read(), Some(("huba".into(), "ui".into())));
        h.prefix(3, "[other] Project changed.");
        h.prefix(4, "[other] Project change confirmed.");
        assert_eq!(h.read(), Some(("other".into(), String::new())));
        h.prefix(5, "[huba] Returning does not resurrect a stale module.");
        h.prefix(6, "[huba] Return confirmed.");
        assert_eq!(h.read(), Some(("huba".into(), String::new())));
    }

    #[test]
    fn user_tool_and_quoted_later_text_are_not_identity() {
        let h = History::new();
        h.add(
            1,
            "user",
            json!([{"type":"text","text":"[huba/user] Not assistant."}]),
        );
        h.add(
            2,
            "assistant",
            json!([{"type":"tool","state":{"content":"[huba/tool] Not prose."}}]),
        );
        h.add(3,"assistant",json!([{"type":"text","text":"Here is a quotation:"},{"type":"text","text":"[huba/quoted] Someone else."}]));
        assert_eq!(h.read(), None);
        h.add(4,"assistant",json!([{"type":"reasoning","text":"not a declaration"},{"type":"text","text":"  **[huba/ui]** Actual declaration."}]));
        assert_eq!(h.read(), Some(("huba".into(), "ui".into())));
    }

    #[test]
    fn history_is_scoped_to_the_requested_session() {
        let h = History::new();
        h.prefix(1, "[huba/ui] Actual session.");
        h.db.execute(
            "INSERT INTO session_message VALUES ('ses_other','assistant',2,?1)",
            [
                json!({"content":[{"type":"text","text":"[huba/wrong] Another session."}]})
                    .to_string(),
            ],
        )
        .unwrap();
        assert_eq!(h.read(), Some(("huba".into(), "ui".into())));
    }

    #[test]
    fn broken_schema_is_an_error_not_no_declaration() {
        let h = History::new();
        h.db.execute_batch("ALTER TABLE session_message RENAME COLUMN data TO broken_data;")
            .unwrap();
        assert!(opencode_head(
            &h.tmp.path().join("history.db"),
            "ses_fixture",
            &h.tmp.path().to_string_lossy()
        )
        .is_err());
    }

    #[test]
    fn malformed_history_is_not_a_valid_empty_result() {
        let h = History::new();
        h.db.execute(
            "INSERT INTO session_message VALUES ('ses_fixture','assistant',1,'{broken')",
            [],
        )
        .unwrap();
        assert!(opencode_head(
            &h.tmp.path().join("history.db"),
            "ses_fixture",
            &h.tmp.path().to_string_lossy()
        )
        .is_err());
    }

    #[test]
    fn unicode_prefix_trimming_matches_the_shared_parser() {
        for code in 0..=u32::from(char::MAX) {
            if let Some(c) = char::from_u32(code) {
                assert_eq!(WHITE_SPACE.contains(c), c.is_whitespace(), "U+{code:04X}");
            }
        }
        let h = History::new();
        h.prefix(1, "\u{a0}**[huba/ui]** Unicode whitespace.");
        assert_eq!(h.read(), Some(("huba".into(), "ui".into())));
    }

    #[test]
    fn no_declaration_cache_invalidates_on_revision_and_errors_retry() {
        let h = History::new();
        let path = h.tmp.path().join("history.db");
        let root = h.tmp.path().to_string_lossy();
        assert_eq!(
            cached_opencode_head(&path, "ses_fixture", &root, 1).unwrap(),
            None
        );
        h.prefix(1, "[huba/ui] New declaration in a new source revision.");
        assert_eq!(
            cached_opencode_head(&path, "ses_fixture", &root, 1).unwrap(),
            None
        );
        assert!(cached_opencode_head(&path, "ses_fixture", &root, 2)
            .unwrap()
            .is_some());
        h.db.execute_batch("ALTER TABLE session_message RENAME COLUMN data TO broken_data;")
            .unwrap();
        assert!(cached_opencode_head(&path, "ses_fixture", &root, 3).is_err());
        h.db.execute_batch("ALTER TABLE session_message RENAME COLUMN broken_data TO data;")
            .unwrap();
        assert!(cached_opencode_head(&path, "ses_fixture", &root, 3)
            .unwrap()
            .is_some());
    }
}
