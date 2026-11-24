//! opencode xong lượt thì huba phải báo NGAY, không chờ hết nhịp ngủ 120 giây.
//!
//! 🔴 Hà 24/11: *"có vẻ opencode nhận và phản hồi tin vẫn chậm"*. ĐO trên phiên `df9df371` (acc8): bản ghi
//! `idle` của opencode (= `session_v2.time_idle`) rồi tin Telegram tới sau **87 · 25 · 88 · 54 · 51 · 112 giây**
//! (6 lượt, 24/11 08:02 → 09:58). Nhật ký hubd: `cycle_woken_by_status after_ms=109046` — vòng ngủ tới khi
//! `book_status_fingerprint` đổi, mà dấu vân tay ấy CHỈ đọc sổ `sessions/` của Claude, nên một lượt opencode xong
//! không đánh thức được ai: phải chờ nhịp `poll_interval_sec` hoặc một phiên Claude nào đó đổi trạng thái.
//!
//! Sửa: dấu vân tay gồm cả `id:time_busy:time_idle` của mọi phiên opencode. Ba phép đo:
//! ① lượt xong (`time_idle` đổi) ⟹ vân tay ĐỔI (RED trên mã cũ) · ② chỉ `time_updated` đổi (opencode ghi mỗi
//! token) ⟹ vân tay KHÔNG đổi — không thì mỗi token đánh thức một vòng 3 giây · ③ không có kho ⟹ không panic.

mod common;

use huba::config::ClaudeAccountCfg;
use huba::sessions::book_status_fingerprint;
use rusqlite::Connection;

fn cfg_voi_acc_opencode(home: &std::path::Path) -> huba::config::Config {
    let mut cfg = common::cfg_for_tests();
    cfg.claude_accounts = vec![ClaudeAccountCfg {
        name: "accx".into(),
        config_dir: Some(home.join("cfg-accx").display().to_string()),
        cli: Some("opencode".into()),
        ..Default::default()
    }];
    cfg
}

#[test]
fn luot_opencode_xong_doi_van_tay_nhung_token_thi_khong() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // `opencode_profile_dir` = $HOME/.opencode-<tên acc>.
    std::env::set_var("HOME", tmp.path());
    let cfg = cfg_voi_acc_opencode(tmp.path());

    // ③ chưa có kho ⟹ vẫn trả được vân tay, không panic.
    let _ = book_status_fingerprint(&cfg);

    let dir = tmp.path().join(".opencode-accx/data/opencode");
    std::fs::create_dir_all(&dir).expect("tạo thư mục kho");
    let db = dir.join("opencode.db");
    let c = Connection::open(&db).expect("mở kho");
    c.execute_batch(
        "CREATE TABLE session_v2 (
            id TEXT PRIMARY KEY, parent_id TEXT, directory TEXT NOT NULL, title TEXT, agent TEXT,
            time_created INTEGER NOT NULL, time_updated INTEGER NOT NULL,
            time_busy INTEGER, time_idle INTEGER
         );
         INSERT INTO session_v2 (id,directory,title,agent,time_created,time_updated,time_busy,time_idle)
         VALUES ('ses_a','/p','đang chạy','build',1,100,100,NULL);",
    )
    .expect("dựng kho");

    let dang_chay = book_status_fingerprint(&cfg);
    assert!(
        dang_chay.contains("ses_a"),
        "vân tay không nhắc phiên opencode nào: {dang_chay:?}"
    );

    // ② opencode ghi `time_updated` theo từng token — KHÔNG được đánh thức vòng.
    c.execute(
        "UPDATE session_v2 SET time_updated = 150 WHERE id='ses_a'",
        [],
    )
    .expect("cập nhật");
    assert_eq!(
        book_status_fingerprint(&cfg),
        dang_chay,
        "time_updated đổi làm vân tay đổi ⟹ mỗi token đánh thức một vòng"
    );

    // ① lượt xong: `time_idle` được ghi ⟹ vân tay PHẢI đổi.
    c.execute(
        "UPDATE session_v2 SET time_idle = 200, time_updated = 200 WHERE id='ses_a'",
        [],
    )
    .expect("cập nhật");
    assert_ne!(
        book_status_fingerprint(&cfg),
        dang_chay,
        "lượt opencode xong mà vân tay không đổi ⟹ huba chỉ biết khi hết nhịp ngủ"
    );
}
