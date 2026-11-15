//! Điều kiện tự chọn Codex: đăng nhập gói, quota thật/tươi, không nhầm API key.
use huba::config::{ClaudeAccountCfg, Config};
use huba::quota::{rank_all, Rank};
use serde_json::{json, Value};
use std::path::Path;

const NOW: i64 = 1_795_000_000_000;
const CHATGPT: &str = r#"{"auth_mode":"chatgpt","tokens":{"access_token":"test-access","refresh_token":"test-refresh"}}"#;
fn stamp(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .unwrap()
        .to_rfc3339()
}
fn limits(used: i64) -> Value {
    json!({"primary":{"used_percent":used,"window_minutes":10080,"resets_at":NOW/1000+7*86400},
        "secondary":null,"rate_limit_reached_type":null})
}
fn account(home: &Path) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: "acc10".into(),
        config_dir: Some(home.display().to_string()),
        cli: Some("codex".into()),
        ..Default::default()
    }
}
fn config(home: &Path) -> Config {
    Config {
        claude_accounts: vec![account(home)],
        ..Default::default()
    }
}
fn log(home: &Path, file: &str, ts: &str, used: i64) -> std::path::PathBuf {
    let dir = home.join("sessions/2026/11/18");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(file);
    std::fs::write(&p, format!("{}\n",json!({"timestamp":ts,"type":"event_msg","payload":{"type":"token_count","rate_limits":limits(used)}}))).unwrap();
    p
}
fn fixture(auth: &str, ts: &str) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("auth.json"), auth).unwrap();
    log(d.path(), "rollout-current.jsonl", ts, 32);
    d
}

#[test]
fn chatgpt_co_so_tuoi_duoc_chon_va_khoa_thi_khong() {
    let d = fixture(CHATGPT, &stamp(NOW));
    let mut cfg = config(d.path());
    let r = rank_all(&cfg, NOW);
    assert_eq!(r[0].rank, Rank::Free(32));
    assert_eq!(
        huba::watch::suggest_account("", &r, &[], 0).as_deref(),
        Some("acc10")
    );
    cfg.claude_accounts[0].locked = true;
    assert_eq!(rank_all(&cfg, NOW)[0].rank, Rank::Locked);
}

#[test]
fn api_key_khong_duoc_tu_chon_bang_quota_chatgpt_cu() {
    for auth in [
        r#"{"auth_mode":"apikey","OPENAI_API_KEY":"test-placeholder"}"#,
        r#"{"auth_mode":"api_key"}"#,
        r#"{"OPENAI_API_KEY":"test-placeholder"}"#,
    ] {
        let d = fixture(auth, &stamp(NOW));
        let cfg = config(d.path());
        let r = rank_all(&cfg, NOW);
        assert_eq!(r[0].rank, Rank::ApiKey);
        assert_eq!(huba::watch::suggest_account("", &r, &[], 0), None);
        let overlaid =
            huba::quota::overlay_live(r, &json!({"acc10":{"session_pct":0,"week_pct":0}}));
        assert_eq!(
            overlaid[0].rank,
            Rank::ApiKey,
            "cache /usage không được mở tài khoản trả tiền"
        );
    }
}

#[test]
fn file_auth_rong_hong_hoac_khong_ro_loai_khong_tinh_la_da_dang_nhap() {
    for auth in [
        "{}",
        "{",
        r#"{"auth_mode":"new-unknown-mode"}"#,
        r#"{"auth_mode":"chatgpt"}"#,
        r#"{"auth_mode":"chatgpt","tokens":{"access_token":"test-access","refresh_token":""}}"#,
    ] {
        let d = fixture(auth, &stamp(NOW));
        let r = rank_all(&config(d.path()), NOW);
        assert_eq!(r[0].rank, Rank::NotReady);
        assert_eq!(huba::watch::suggest_account("", &r, &[], 0), None);
    }
}

#[test]
fn quota_cu_thieu_hoac_hong_moc_do_khong_duoc_tu_chon() {
    for ts in [
        stamp(NOW - 8 * 86400 * 1000),
        "bad-date".into(),
        stamp(NOW + 86400 * 1000),
    ] {
        let d = fixture(CHATGPT, &ts);
        let r = rank_all(&config(d.path()), NOW);
        assert_eq!(r[0].rank, Rank::NotReady, "quota ts={ts}");
        assert_eq!(huba::watch::suggest_account("", &r, &[], 0), None);
    }
}

#[test]
fn quota_chon_moc_su_kien_moi_nhat_khong_chon_mtime_file_fork() {
    let d = tempfile::tempdir().unwrap();
    let fresh = log(d.path(), "rollout-older-file.jsonl", &stamp(NOW), 32);
    let fork = log(
        d.path(),
        "rollout-newer-fork.jsonl",
        &stamp(NOW - 3600 * 1000),
        89,
    );
    let time = std::time::UNIX_EPOCH + std::time::Duration::from_millis(NOW as u64);
    std::fs::File::options()
        .write(true)
        .open(&fresh)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(time))
        .unwrap();
    std::fs::File::options()
        .write(true)
        .open(&fork)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(time + std::time::Duration::from_secs(1)))
        .unwrap();
    let (rl, ts) = huba::accounts::codex_rate_limits(&account(d.path()))
        .unwrap()
        .unwrap();
    assert_eq!(rl["primary"]["used_percent"], 32);
    assert_eq!(ts, stamp(NOW));
}

#[test]
fn quota_thieu_cua_so_hoac_so_hong_khong_tu_chon() {
    for field in ["used_percent", "window_minutes", "resets_at"] {
        let d = fixture(CHATGPT, &stamp(NOW));
        let mut rl = limits(32);
        rl["primary"].as_object_mut().unwrap().remove(field);
        let p = d.path().join("sessions/2026/11/18/rollout-current.jsonl");
        std::fs::write(
            p,
            format!(
                "{}\n",
                json!({"timestamp":stamp(NOW),"payload":{"type":"token_count","rate_limits":rl}})
            ),
        )
        .unwrap();
        assert_eq!(
            rank_all(&config(d.path()), NOW)[0].rank,
            Rank::NotReady,
            "missing {field}"
        );
    }
    let d = fixture(CHATGPT, &stamp(NOW));
    log(d.path(), "rollout-current.jsonl", &stamp(NOW), -12);
    assert_eq!(rank_all(&config(d.path()), NOW)[0].rank, Rank::NotReady);
    std::fs::remove_dir_all(d.path().join("sessions")).unwrap();
    assert_eq!(rank_all(&config(d.path()), NOW)[0].rank, Rank::NotReady);
}
