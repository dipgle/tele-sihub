//! Số cũ, mất đăng nhập, gói không hạn 5 giờ — ba câu trả lời cho câu hỏi của Quang
//! 12/11: *"Tại sao acc6 và acc8 % thấp hơn acc3 lại không được chọn mở phiên mới?
//! Có map nhầm số liệu không"* · *"Ko dùng tới thì nó chả cũ"*.
//!
//! Đo 12/11 trên máy thật:
//! - acc6: sổ `.claude.json` 64 % tuần, già 29 giờ, mốc 5 giờ đã qua. `claude auth
//!   status` ⟹ `loggedIn:false` (mã thoát 1, kèm JSON). `claude -p /usage` ⟹ rc 0,
//!   chỉ `Total cost: $0.0000 …` — vậy mà bản cũ ghi hàng ấy với `at_ms` mới, như
//!   một lượt đo thành công.
//! - acc3: `five_hour: null` + `limits` chỉ hàng nhóm `weekly` đang áp ⟹ huba đọc
//!   "chưa đo được", biểu đồ đầu phiên đọc 5G = 0.
//!
//! Bài dựng `claude` GIẢ: `auth status` trả `loggedIn` theo tệp `<config_dir>/dn`
//! (không có tệp ⟹ in rác, tức "không đo được"); `-p /usage` chỉ in bảng chi phí.

mod common;

use std::path::{Path, PathBuf};

use huba::config::{ClaudeAccountCfg, Config};
use huba::db::Db;
use huba::quota::{ap_dang_nhap, khong_dung_tu, rank, rank_can_tren, read, Rank, Ranked};
use huba::runtime::{hang_khong_ra_so, xep_hang_tai_khoan};

const GIO: i64 = 3_600_000;

fn bay_gio() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn iso(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .unwrap()
        .to_rfc3339()
}

fn tam(ten: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "huba-socu-{ten}-{}-{}",
        std::process::id(),
        bay_gio()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Sổ kiểu acc6 ngày 12/11: tuần 64 % mốc còn 4 ngày, 5 giờ 2 % mốc ĐÃ QUA, ghi 29 giờ trước.
fn so_acc6(dir: &Path, now: i64) -> i64 {
    let fetched = now - 29 * GIO;
    let doc = serde_json::json!({
        "oauthAccount": { "emailAddress": "x@bia" },
        "hasCompletedOnboarding": true,
        "cachedUsageUtilization": {
            "fetchedAtMs": fetched,
            "utilization": {
                "five_hour": { "utilization": 2, "resets_at": iso(now - 28 * GIO) },
                "seven_day": { "utilization": 64, "resets_at": iso(now + 96 * GIO) },
            }
        }
    });
    std::fs::write(dir.join(".claude.json"), doc.to_string()).unwrap();
    fetched
}

fn dat_mtime(p: &Path, ms: i64) {
    let f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(p)
        .unwrap();
    f.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_millis(ms as u64))
        .unwrap();
}

// ── ② map nhầm acc3: gói không còn hạn 5 giờ ───────────────────────────────────

#[test]
fn five_hour_rong_va_chi_han_tuan_thi_5g_bang_0() {
    let d = tam("5g");
    let now = bay_gio();
    let tuan = serde_json::json!([
        { "kind": "weekly_all", "group": "weekly", "percent": 82, "is_active": true },
        { "kind": "weekly_scoped", "group": "weekly", "percent": 0, "is_active": false }
    ]);
    let ghi = |limits: serde_json::Value| {
        let doc = serde_json::json!({
            "oauthAccount": { "emailAddress": "x@bia" },
            "hasCompletedOnboarding": true,
            "cachedUsageUtilization": { "fetchedAtMs": now, "utilization": {
                "five_hour": null,
                "seven_day": { "utilization": 82, "resets_at": iso(now + 96 * GIO) },
                "limits": limits,
            }}
        });
        std::fs::write(d.join(".claude.json"), doc.to_string()).unwrap();
    };
    ghi(tuan);
    let q = read("acc3", Some(&d));
    assert_eq!(q.hour5_pct, Some(0), "acc3 gói không hạn 5 giờ: {q:?}");
    assert_eq!(rank(&q, now), Rank::Free(82));

    // ĐỐI CHỨNG: còn một hạn KHÁC tuần đang áp ⟹ không đoán, vẫn "chưa đo được".
    ghi(serde_json::json!([
        { "kind": "weekly_all", "group": "weekly", "is_active": true },
        { "kind": "session", "group": "session", "is_active": true }
    ]));
    let q = read("acc3", Some(&d));
    assert_eq!(q.hour5_pct, None);
    assert_eq!(rank(&q, now), Rank::Unknown);

    // ĐỐI CHỨNG: `limits` rỗng ⟹ không đủ chứng cứ.
    ghi(serde_json::json!([]));
    assert_eq!(read("acc3", Some(&d)).hour5_pct, None);
}

// ── ① số cũ = cận trên, chỉ khi không có dấu dùng ──────────────────────────────

#[test]
fn dau_dung_history_va_sessions_lat_duoc_hai_chieu() {
    let d = tam("dau");
    let now = bay_gio();
    let fetched = now - 10 * GIO;
    assert!(
        khong_dung_tu("t", &d, fetched),
        "chưa có tệp nào ⟹ không dấu dùng"
    );

    let h = d.join("history.jsonl");
    dat_mtime(&h, fetched - GIO);
    assert!(khong_dung_tu("t", &d, fetched), "history CŨ hơn mốc đo");
    dat_mtime(&h, fetched + GIO);
    assert!(!khong_dung_tu("t", &d, fetched), "có câu gõ SAU mốc đo");
    dat_mtime(&h, fetched - GIO);

    std::fs::create_dir_all(d.join("sessions")).unwrap();
    dat_mtime(&d.join("sessions").join("9.key"), fetched + GIO);
    assert!(
        khong_dung_tu("t", &d, fetched),
        "chỉ *.json là phiên — tệp khoá không tính"
    );
    dat_mtime(&d.join("sessions").join("9.json"), fetched + GIO);
    assert!(!khong_dung_tu("t", &d, fetched), "có phiên đang sống");
}

#[test]
fn rank_can_tren_cua_so_qua_moc_ve_0_cua_so_chua_qua_giu_so() {
    let d = tam("ct");
    let now = bay_gio();
    so_acc6(&d, now);
    let q = read("acc6", Some(&d));
    assert!(q.khong_dung_tu_khi_do);
    assert_eq!(rank(&q, now), Rank::Unknown, "rank() thuần sổ KHÔNG đổi");
    assert_eq!(rank_can_tren(&q, now), Rank::Free(64));

    let mut co_dung = q.clone();
    co_dung.khong_dung_tu_khi_do = false;
    assert_eq!(rank_can_tren(&co_dung, now), Rank::Unknown);
}

fn hang(ten: &str, r: Rank) -> Vec<Ranked> {
    vec![Ranked {
        name: ten.into(),
        rank: r,
    }]
}

#[test]
fn ap_dang_nhap_hai_chieu() {
    let d = tam("ap");
    let now = bay_gio();
    let fetched = so_acc6(&d, now);
    let qs = vec![read("acc6", Some(&d))];
    let so = |dn: Option<bool>, luc: i64| {
        let mut row = hang_khong_ra_so("Total cost: $0.0000", dn);
        row["err_at_ms"] = serde_json::json!(luc);
        serde_json::json!({ "acc6": row })
    };

    // 🔴 CẤY HỎNG: mất đăng nhập ⟹ KHÔNG BAO GIỜ thành ứng viên.
    let u = so(Some(false), now);
    let r = ap_dang_nhap(hang("acc6", Rank::Unknown), &qs, Some(&u), now);
    assert_eq!(r[0].rank, Rank::NotReady);

    // CẤY LÀNH: còn đăng nhập + không dấu dùng ⟹ cận trên.
    let u = so(Some(true), now);
    let r = ap_dang_nhap(hang("acc6", Rank::Unknown), &qs, Some(&u), now);
    assert_eq!(r[0].rank, Rank::Free(64));

    // Không hỏi được ⟹ không đoán.
    let u = so(None, now);
    let r = ap_dang_nhap(hang("acc6", Rank::Unknown), &qs, Some(&u), now);
    assert_eq!(r[0].rank, Rank::Unknown);

    // Sổ CLI MỚI HƠN lượt hỏi (có người vừa dùng được) ⟹ câu "chưa đăng nhập" đã hết hạn.
    let u = so(Some(false), fetched - GIO);
    let r = ap_dang_nhap(hang("acc6", Rank::Free(10)), &qs, Some(&u), now);
    assert_eq!(r[0].rank, Rank::Free(10));

    // Khoá là quyết định của chủ máy.
    let u = so(Some(true), now);
    let r = ap_dang_nhap(hang("acc6", Rank::Locked), &qs, Some(&u), now);
    assert_eq!(r[0].rank, Rank::Locked);
}

// ── ③ `/usage` không ra số KHÔNG phải "vừa đo" ────────────────────────────────

#[test]
fn hang_khong_ra_so_la_loi_khong_mang_at_ms() {
    let r = hang_khong_ra_so("Total cost: $0.0000", Some(false));
    assert!(r["err"].as_str().unwrap().contains("CHƯA ĐĂNG NHẬP"), "{r}");
    assert_eq!(r["dang_nhap"], false);
    assert!(r.get("at_ms").is_none());
    let r = hang_khong_ra_so("Total cost: $0.0000", None);
    assert!(r.get("dang_nhap").is_none());
    assert!(r.get("err").is_some());
}

// ── Đường thật: xep_hang_tai_khoan với `claude` giả ───────────────────────────

fn claude_gia(dir: &Path) -> PathBuf {
    let p = dir.join("claude-gia.sh");
    std::fs::write(
        &p,
        "#!/bin/bash\n\
         if [ \"$1\" = auth ]; then\n\
           dn=$(cat \"$CLAUDE_CONFIG_DIR/dn\" 2>/dev/null) || { echo rac; exit 1; }\n\
           echo \"{\\\"loggedIn\\\":$dn}\"; [ \"$dn\" = true ] && exit 0; exit 1\n\
         fi\n\
         printf '{\"result\":\"Total cost:            $0.0000\\\\nTotal duration (API):  0s\"}\\n'\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    p
}

fn may(dir: &Path, dn: Option<&str>) -> (Db, Config) {
    let db = Db::open(&dir.join("huba.sqlite")).expect("open db");
    let mut cfg = common::cfg_for_tests();
    cfg.claude_cli = claude_gia(dir).display().to_string();
    let d = dir.join("acc6");
    std::fs::create_dir_all(&d).unwrap();
    so_acc6(&d, bay_gio());
    if let Some(v) = dn {
        std::fs::write(d.join("dn"), v).unwrap();
    }
    cfg.claude_accounts = vec![ClaudeAccountCfg {
        name: "acc6".into(),
        config_dir: Some(d.display().to_string()),
        ..Default::default()
    }];
    (db, cfg)
}

#[test]
fn duong_that_mat_dang_nhap_thi_chua_dung_duoc() {
    let dir = tam("that-false");
    let (db, cfg) = may(&dir, Some("false"));
    let r = xep_hang_tai_khoan(&cfg, &db, Some(0));
    assert_eq!(r[0].rank, Rank::NotReady, "{r:?}");
    let so = db.get_cursor(huba::runtime::USAGE_KEY).unwrap().unwrap();
    let so: serde_json::Value = serde_json::from_str(&so).unwrap();
    assert!(
        so["acc6"].get("at_ms").is_none(),
        "không ra số mà vẫn đóng dấu vừa đo: {so}"
    );
}

#[test]
fn duong_that_con_dang_nhap_thi_can_tren() {
    let dir = tam("that-true");
    let (db, cfg) = may(&dir, Some("true"));
    let r = xep_hang_tai_khoan(&cfg, &db, Some(0));
    assert_eq!(r[0].rank, Rank::Free(64), "{r:?}");
}

#[test]
fn duong_that_khong_hoi_duoc_thi_khong_doan() {
    let dir = tam("that-none");
    let (db, cfg) = may(&dir, None);
    let r = xep_hang_tai_khoan(&cfg, &db, Some(0));
    assert_eq!(r[0].rank, Rank::Unknown, "{r:?}");
}
