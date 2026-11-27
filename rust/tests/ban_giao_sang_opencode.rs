//! Bàn giao SANG một tài khoản opencode phải nhận ra phiên kế nhiệm.
//!
//! 🔴 Đo 16/11 trên Mac (3 lượt `/handover` sang acc9): `start_fresh_after_handover` chờ id
//! bằng cách của Claude (nhật ký `.jsonl` + `claude agents`) cho MỌI tài khoản. opencode không
//! sinh nhật ký ấy ⟹ luôn `handover_new_session_never_appeared` ⟹ cửa sổ cũ bị GIỮ (hai phiên
//! cùng module) và phiên mới không có liên kết cửa sổ nên bị ẩn khỏi `/session`.
//!
//! Bài này chấm chỗ quyết `sessions::cho_id_ke_nhiem` — phần SAU `keys::open_window` (mở cửa sổ
//! thật thì không đặt vào bộ kiểm: nó bật cửa sổ lên màn hình chủ máy). Ba phép đo:
//! ① kho có phiên mới trong đúng thư mục ⟹ trả id + ghi liên kết cửa sổ (RED trên mã cũ:
//! đường Claude trả `None`) · ② phiên trong thư mục KHÁC ⟹ `None` · ③ phiên tạo TRƯỚC lúc mở
//! (chính phiên cũ) ⟹ `None` — không nhận nhầm phiên cũ làm kế nhiệm.

mod common;

use huba::config::ClaudeAccountCfg;
use huba::sessions::cho_id_ke_nhiem;
use rusqlite::Connection;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn ban_giao_sang_acc_opencode_nhan_ra_phien_ke_nhiem() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // `opencode_profile_dir` = $HOME/.opencode-<tên acc>. Một bài duy nhất trong tệp ⟹ đổi HOME an toàn.
    std::env::set_var("HOME", tmp.path());
    let mut cfg = common::cfg_for_tests();
    cfg.claude_accounts = vec![ClaudeAccountCfg {
        name: "accx".into(),
        config_dir: Some(tmp.path().join("cfg-accx").display().to_string()),
        cli: Some("opencode".into()),
        ..Default::default()
    }];
    let acc = cfg.claude_accounts[0].clone();

    let cwd = tmp.path().join("du-an");
    std::fs::create_dir_all(&cwd).expect("tạo cwd");
    let cwd_s = cwd.to_string_lossy().to_string();

    let opened_at = UNIX_EPOCH + Duration::from_millis(1_000_000);
    let kho = tmp.path().join(".opencode-accx/data/opencode");
    std::fs::create_dir_all(&kho).expect("tạo thư mục kho");
    let c = Connection::open(kho.join("opencode.db")).expect("mở kho");
    c.execute_batch(
        "CREATE TABLE session_v2 (
            id TEXT PRIMARY KEY, parent_id TEXT, directory TEXT NOT NULL, title TEXT, model TEXT,
            tokens_input INTEGER, tokens_output INTEGER, tokens_reasoning INTEGER,
            tokens_cache_read INTEGER, tokens_cache_write INTEGER, cost REAL,
            time_created INTEGER NOT NULL, time_updated INTEGER NOT NULL, time_archived INTEGER
         );",
    )
    .expect("dựng kho");
    let them = |id: &str, dir: &str, created: i64| {
        c.execute(
            "INSERT INTO session_v2 (id,directory,title,time_created,time_updated) VALUES (?1,?2,'t',?3,?3)",
            rusqlite::params![id, dir, created],
        )
        .expect("chèn phiên");
    };

    // ③ chỉ có phiên CŨ (tạo trước lúc mở) trong đúng thư mục ⟹ không nhận.
    them("ses_cu", &cwd_s, 999_000);
    // ② phiên mới nhưng ở thư mục KHÁC ⟹ không nhận.
    them("ses_khac", "/thu-muc/khac", 1_000_500);
    assert_eq!(
        cho_id_ke_nhiem(&cfg, "accx", &cwd, opened_at, 42, "pts9", Some("ses_cu"), 1),
        None,
        "không có phiên kế nhiệm trong đúng thư mục mà vẫn trả id"
    );

    // ① phiên kế nhiệm xuất hiện trong đúng thư mục ⟹ trả id + ghi liên kết cửa sổ.
    them("ses_moi", &cwd_s, 1_000_200);
    assert_eq!(
        cho_id_ke_nhiem(&cfg, "accx", &cwd, opened_at, 42, "pts9", Some("ses_cu"), 5),
        Some("ses_moi".to_string()),
        "bàn giao sang acc opencode không nhận ra phiên kế nhiệm"
    );
    let lien_ket = huba::opencode::read_bindings(&acc).expect("đọc liên kết");
    let b = lien_ket
        .get("ses_moi")
        .unwrap_or_else(|| panic!("không ghi liên kết cửa sổ: {lien_ket:?}"));
    assert_eq!((b.window, b.tty.as_str()), (42, "pts9"));
    assert_eq!(b.dir.as_deref(), Some(cwd_s.as_str()));
}
