//! Regression for the real OpenCode -> LiveSession -> display boundary.
//! Local synthetic SQLite only; no mutation of a live CLI or owner transcript.

use super::*;
use crate::opencode::{read_sessions_at, OcSession};
use rusqlite::{params, Connection};

struct Fixture {
    _tmp: tempfile::TempDir,
    cfg: Config,
    account: ClaudeAccountCfg,
    db: Connection,
    path: PathBuf,
    id: String,
}

impl Fixture {
    fn new(case: &str) -> Self {
        let tmp = tempfile::tempdir().expect("fixture directory");
        let root = tmp.path().join("workspace");
        std::fs::create_dir_all(root.join("huba")).unwrap();
        std::fs::create_dir_all(root.join("other")).unwrap();
        let path = tmp.path().join("opencode.db");
        let db = Connection::open(&path).unwrap();
        db.execute_batch(
            "CREATE TABLE session_v2 (
                id TEXT PRIMARY KEY, parent_id TEXT, directory TEXT NOT NULL,
                title TEXT, cost REAL NOT NULL DEFAULT 0,
                tokens_input INTEGER NOT NULL DEFAULT 0,
                tokens_output INTEGER NOT NULL DEFAULT 0,
                tokens_reasoning INTEGER NOT NULL DEFAULT 0,
                tokens_cache_read INTEGER NOT NULL DEFAULT 0,
                tokens_cache_write INTEGER NOT NULL DEFAULT 0,
                model TEXT, time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL, time_archived INTEGER
             );
             CREATE TABLE session_message (
                id TEXT PRIMARY KEY, session_id TEXT NOT NULL,
                type TEXT NOT NULL, seq INTEGER NOT NULL,
                time_created INTEGER NOT NULL, time_updated INTEGER NOT NULL,
                data TEXT NOT NULL
             );",
        )
        .unwrap();
        let id = format!(
            "ses_{case}_{}",
            tmp.path().file_name().unwrap().to_string_lossy()
        );
        db.execute(
            "INSERT INTO session_v2 (id,directory,title,time_created,time_updated)
             VALUES (?1,?2,'UI handoff',1,2)",
            params![id, root.join("huba").to_string_lossy()],
        )
        .unwrap();
        Self {
            _tmp: tmp,
            cfg: Config {
                workspace_root: root,
                ..Default::default()
            },
            account: ClaudeAccountCfg {
                name: "fixture".into(),
                cli: Some("opencode".into()),
                ..Default::default()
            },
            db,
            path,
            id,
        }
    }

    fn message(&self, seq: i64, role: &str, text: &str) {
        let data = json!({"content":[{"type":"text","text":text}]}).to_string();
        self.db
            .execute(
                "INSERT INTO session_message VALUES (?1,?2,?3,?4,?4,?4,?5)",
                params![format!("m{seq}"), self.id, role, seq, data],
            )
            .unwrap();
        self.db
            .execute(
                "UPDATE session_v2 SET time_updated=?1 WHERE id=?2",
                params![seq, self.id],
            )
            .unwrap();
    }

    fn source(&self) -> OcSession {
        read_sessions_at(&self.path)
            .unwrap()
            .into_iter()
            .find(|s| s.id == self.id)
            .unwrap()
    }

    fn live(&self, source: OcSession) -> LiveSession {
        opencode_live_session(
            &self.cfg,
            &self.account,
            source,
            "pts/fixture".into(),
            &self.path,
        )
    }

    fn label(&self, row: LiveSession) -> LiveSession {
        let mut snap = SessionsSnapshot {
            sessions: vec![row],
            ..Default::default()
        };
        label_sessions(&mut snap.sessions, &self.cfg.workspace_root);
        snap.sessions.remove(0)
    }
}

#[test]
fn sqlite_prefix_reaches_module_and_display_label() {
    let f = Fixture::new("ui");
    f.message(1, "assistant", "[huba/ui] Đang kiểm bản thiết kế.");
    let row = f.label(f.live(f.source()));
    assert_eq!(row.folder, "huba");
    assert_eq!(
        row.lane, "ui",
        "source prefix must not disappear at the row boundary"
    );
    assert_eq!(row.label, "[huba/ui]");
    assert_eq!(row.kind, "opencode");
    assert_eq!(row.tty, "pts/fixture");
}

#[test]
fn different_modules_remain_distinct_for_the_same_project() {
    let ui = Fixture::new("two_ui");
    let core = Fixture::new("two_core");
    ui.message(1, "assistant", "[huba/ui] Bản thiết kế.");
    core.message(1, "assistant", "[huba/core] Bộ đọc phiên.");
    let mut rows = vec![ui.live(ui.source()), core.live(core.source())];
    label_sessions(&mut rows, &ui.cfg.workspace_root);
    assert_eq!(rows[0].label, "[huba/ui]");
    assert_eq!(rows[1].label, "[huba/core]");
    assert_ne!(rows[0].label, rows[1].label);
}

#[test]
fn user_prefix_and_plain_assistant_do_not_invent_a_module() {
    let f = Fixture::new("no_module");
    f.message(1, "user", "[huba/not-my-module] Nhắc một module khác.");
    f.message(2, "assistant", "Đang đọc tài liệu.");
    let row = f.live(f.source());
    assert_eq!(row.folder, "huba");
    assert!(row.lane.is_empty());
}

#[test]
fn module_survives_a_tail_without_a_declaration() {
    let f = Fixture::new("memo");
    f.message(1, "assistant", "[huba/ui] Đang kiểm.");
    assert_eq!(f.live(f.source()).lane, "ui");
    let mut next = f.source();
    next.tail = json!({"type":"assistant","message":{"content":[{"type":"text","text":"Đang chạy bài kiểm."}]}}).to_string();
    let row = f.label(opencode_live_session(
        &f.cfg,
        &f.account,
        next,
        "pts/fixture".into(),
        &f._tmp.path().join("missing-history.db"),
    ));
    assert_eq!(row.folder, "huba");
    assert_eq!(row.lane, "ui");
    assert_eq!(row.label, "[huba/ui]");
}

#[test]
fn project_only_prefix_does_not_acquire_an_arbitrary_module() {
    let f = Fixture::new("project_only");
    f.message(1, "assistant", "[huba] Chưa tự khai module.");
    let row = f.live(f.source());
    assert_eq!(row.folder, "huba");
    assert!(row.lane.is_empty());
}

#[test]
fn a_bare_prefix_preserves_the_same_project_module_in_shared_memo() {
    let f = Fixture::new("bare_after_module");
    f.message(1, "assistant", "[huba/ui] Original module.");
    f.live(f.source());
    f.message(2, "assistant", "[huba] Still the same project.");
    let source = f.source();
    let ws = f.cfg.workspace_root.to_string_lossy();
    assert_eq!(
        folder_for_session(&f.id, &source.tail, &ws, || None).as_deref(),
        Some("huba")
    );
    assert_eq!(lane_for_session(&f.id, &source.tail, &ws, || None), "ui");
}

#[test]
fn rejected_project_flip_cannot_donate_its_module() {
    let f = Fixture::new("reject_flip");
    f.message(1, "assistant", "[huba/ui] Original identity.");
    f.live(f.source());
    f.message(
        2,
        "assistant",
        "[other/review] A single mention, not a project change.",
    );
    let source = f.source();
    let ws = f.cfg.workspace_root.to_string_lossy();
    assert_eq!(
        folder_for_session(&f.id, &source.tail, &ws, || None).as_deref(),
        Some("huba")
    );
    assert_eq!(lane_for_session(&f.id, &source.tail, &ws, || None), "ui");
}

#[test]
fn a_mismatched_head_cannot_supply_another_projects_module() {
    let f = Fixture::new("head_mismatch");
    f.message(1, "assistant", "[huba] The accepted project.");
    f.live(f.source());
    let head=json!({"type":"assistant","message":{"content":[{"type":"text","text":"[other/review] Old project."}]}}).to_string();
    assert!(
        lane_for_session(&f.id, "", &f.cfg.workspace_root.to_string_lossy(), || Some(
            head
        ))
        .is_empty()
    );
}

#[test]
fn an_accepted_project_change_does_not_reuse_the_old_module() {
    let f = Fixture::new("accepted_flip");
    f.message(1, "assistant", "[huba/ui] Original identity.");
    f.live(f.source());
    f.message(2, "assistant", "[other] New project, explicitly repeated.");
    f.message(3, "assistant", "[other] New project, still no module.");
    let row = f.live(f.source());
    assert_eq!(row.folder, "other");
    assert!(row.lane.is_empty());
}

#[test]
fn a_cold_row_recovers_a_mid_history_module_after_the_40_row_preview() {
    let f = Fixture::new("cold_history");
    f.message(1, "assistant", "No declaration in the opening.");
    f.message(
        20,
        "assistant",
        "[huba/ui] Declaration in the middle of history.",
    );
    for seq in 21..80 {
        let data = if seq >= 78 {
            json!({"content":[{"type":"tool","state":{"input":{"path":f.cfg.workspace_root.join(format!("other/src/{seq}.rs")).to_string_lossy()}}}]})
        } else {
            json!({"content":[{"type":"tool","state":{"status":"completed"}}]})
        };
        f.db.execute(
            "INSERT INTO session_message VALUES (?1,?2,'assistant',?3,?3,?3,?4)",
            params![format!("m{seq}"), f.id, seq, data.to_string()],
        )
        .unwrap();
    }
    let source = f.source();
    assert!(
        !source.tail.contains("[huba/ui]"),
        "negative control: declaration must be out of the preview"
    );
    let row = f.label(f.live(source));
    assert_eq!(row.folder, "huba");
    assert_eq!(row.lane, "ui");
    assert_eq!(row.label, "[huba/ui]");
}

fn append_tool_batch(f: &Fixture) {
    for seq in 50..110 {
        f.db.execute(
            "INSERT INTO session_message VALUES (?1,?2,'assistant',?3,?3,?3,?4)",
            params![
                format!("m{seq}"),
                f.id,
                seq,
                json!({"content":[{"type":"tool","state":{"status":"completed"}}]}).to_string()
            ],
        )
        .unwrap();
    }
}

#[test]
fn warm_and_cold_both_reject_one_off_project_mentions() {
    let f = Fixture::new("warm_cold_reject");
    f.message(1, "assistant", "[huba/ui] Original identity.");
    f.message(2, "assistant", "[huba/ui] Original identity repeated.");
    f.live(f.source());
    f.message(3, "assistant", "[other/review] One-off mention.");
    append_tool_batch(&f);
    let warm = f.live(f.source());
    folder_memo().lock().unwrap().remove(&f.id);
    let cold = f.live(f.source());
    assert_eq!((warm.folder, warm.lane), ("huba".into(), "ui".into()));
    assert_eq!((cold.folder, cold.lane), ("huba".into(), "ui".into()));
}

#[test]
fn warm_and_cold_accept_a_repeated_project_transition() {
    let f = Fixture::new("warm_cold_accept");
    f.message(1, "assistant", "[huba/ui] Original identity.");
    f.live(f.source());
    f.message(2, "assistant", "[other/review] Changed project.");
    f.message(3, "assistant", "[other/review] Changed project repeated.");
    let warm = f.live(f.source());
    append_tool_batch(&f);
    folder_memo().lock().unwrap().remove(&f.id);
    let cold = f.live(f.source());
    assert_eq!((warm.folder, warm.lane), ("other".into(), "review".into()));
    assert_eq!((cold.folder, cold.lane), ("other".into(), "review".into()));
}

#[test]
fn missed_intervening_project_change_does_not_resurrect_a_module() {
    let f = Fixture::new("unseen_transition");
    f.message(1, "assistant", "[huba/ui] Original identity.");
    f.live(f.source());
    f.message(2, "assistant", "[other] Changed project.");
    f.message(3, "assistant", "[other] Changed project repeated.");
    f.message(4, "assistant", "[huba] Returned, no module declared.");
    f.message(5, "assistant", "[huba] Return confirmed.");
    let row = f.live(f.source());
    assert_eq!(row.folder, "huba");
    assert!(
        row.lane.is_empty(),
        "a real intervening transition must clear the old module"
    );
}
