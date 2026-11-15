//! acc codex phải được XẾP HẠNG bằng hạn mức thật, không bị ép «khoá».
//!
//! 🔴 Quang 15/11: *«thế tài khoản acc10 vứt cho chó gặm à»* · *«có vấn đề gì ở chỗ lọc
//! tài khoản vậy mà suốt từ hôm qua tới giờ không chạy được gì»*. `quota::rank_all` xếp
//! MỌI hàng codex là `Rank::Locked` (chốt tạm của `a5d4235` khi huba chưa mở được phiên
//! codex — không ai gỡ sau `9d7b1ce`). Chủ máy khoá 9 acc Claude ⟹ bàn giao tự động và
//! `/new` không còn ai để chọn — log 14/11 23:58Z → 15/11 01:01Z, 6 lượt
//! `auto_handover_window_failed` «Không còn tài khoản nào đang mở».

use huba::config::{ClaudeAccountCfg, Config};
use huba::quota::{codex_quota_tu, rank, rank_all, Rank};
use serde_json::json;

const NOW: i64 = 1_795_000_000_000; // 2026-11-18 — trước mốc mở lại bên dưới

/// Nguyên văn khuôn `rate_limits` của acc10 (rollout 15/11, codex-cli 0.169, gói prolite).
fn that(used: f64, chan: bool) -> serde_json::Value {
    json!({ "limit_id": "codex", "primary": { "used_percent": used, "window_minutes": 10080,
            "resets_at": 1795235473 }, "secondary": null,
            "rate_limit_reached_type": if chan { json!("primary") } else { json!(null) },
            "plan_type": "prolite" })
}

#[test]
fn mot_cua_so_tuan_thi_o_5_gio_mang_chinh_no() {
    let q = codex_quota_tu("acc10", &that(31.0, false), "2026-11-15T00:41:01.000Z");
    assert_eq!(q.week_pct, Some(31));
    assert_eq!(
        q.hour5_pct,
        Some(31),
        "Quang 15/11: «5h bằng đúng hạn tuần»"
    );
    assert_eq!(q.week_resets_at, q.hour5_resets_at);
    assert!(q.fetched_at_ms.is_some());
    assert_eq!(rank(&q, NOW), Rank::Free(31));
}

/// Gán ngược TỪ TRÊN XUỐNG: chỉ có cửa sổ tháng ⟹ tuần = 5 giờ = tháng; chỉ có cửa sổ
/// 5 giờ ⟹ KHÔNG đẩy lên tuần (ô tuần trống ⟹ chưa đo được).
#[test]
fn gan_nguoc_tu_tren_xuong() {
    let thang = json!({ "primary": { "used_percent": 12.0, "window_minutes": 43200,
                        "resets_at": 1795235473 }, "secondary": null, "rate_limit_reached_type": null });
    let q = codex_quota_tu("acc10", &thang, "2026-11-15T00:41:01.000Z");
    assert_eq!((q.week_pct, q.hour5_pct), (Some(12), Some(12)));
    let hai = json!({ "primary": { "used_percent": 70.0, "window_minutes": 300, "resets_at": 1795235473 },
                      "secondary": { "used_percent": 20.0, "window_minutes": 43200, "resets_at": 1795235473 },
                      "rate_limit_reached_type": null });
    let q = codex_quota_tu("acc10", &hai, "2026-11-15T00:41:01.000Z");
    assert_eq!(
        (q.week_pct, q.hour5_pct),
        (Some(20), Some(70)),
        "5 giờ có số riêng thì giữ"
    );
    let chi_5h = json!({ "primary": { "used_percent": 70.0, "window_minutes": 300, "resets_at": 1795235473 },
                         "secondary": null, "rate_limit_reached_type": null });
    let q = codex_quota_tu("acc10", &chi_5h, "2026-11-15T00:41:01.000Z");
    assert_eq!(
        (q.week_pct, q.hour5_pct),
        (None, Some(70)),
        "không gán từ dưới lên"
    );
}

#[test]
fn cham_tran_thi_kich_tran_khong_rong_thi_khong_do_duoc() {
    let q = codex_quota_tu("acc10", &that(40.0, true), "2026-11-15T00:41:01.000Z");
    assert_eq!(rank(&q, NOW), Rank::Full);
    let rong = codex_quota_tu("acc10", &json!({ "primary": null, "secondary": null }), "");
    assert_eq!(rank(&rong, NOW), Rank::Unknown);
    assert!(rong.why_unknown.is_some());
}

/// Hàng codex ĐÃ đăng nhập, có nhật ký ghi hạn mức ⟹ `rank_all` cho hạng THẬT, không
/// `Locked`; KHOÁ trong cấu hình ⟹ `Locked` (khoá của chủ máy vẫn là khoá).
#[test]
fn rank_all_khong_ep_codex_thanh_khoa() {
    let d = tempfile::tempdir().unwrap();
    let home = d.path().join("codex-acc10");
    let ngay = home.join("sessions/2026/11/15");
    std::fs::create_dir_all(&ngay).unwrap();
    std::fs::write(home.join("auth.json"), "{}").unwrap();
    std::fs::write(
        ngay.join("rollout-x.jsonl"),
        format!(
            "{}\n",
            json!({ "timestamp": "2026-11-15T00:41:01.000Z",
                    "payload": { "type": "token_count", "rate_limits": that(31.0, false) } })
        ),
    )
    .unwrap();
    let hang = |locked: bool| ClaudeAccountCfg {
        name: "acc10".into(),
        config_dir: Some(home.display().to_string()),
        cli: Some("codex".into()),
        locked,
        ..Default::default()
    };
    let mut cfg = Config {
        claude_accounts: vec![hang(false)],
        ..Default::default()
    };
    let r = rank_all(&cfg, NOW);
    assert_eq!(r.len(), 1);
    assert_eq!(
        r[0].rank,
        Rank::Free(31),
        "codex mở khoá phải xếp bằng số thật"
    );

    cfg.claude_accounts = vec![hang(true)];
    assert_eq!(rank_all(&cfg, NOW)[0].rank, Rank::Locked);

    // Chưa đăng nhập ⟹ chưa dùng được (không gợi ý), KHÔNG phải khoá.
    std::fs::remove_file(home.join("auth.json")).unwrap();
    cfg.claude_accounts = vec![hang(false)];
    assert_eq!(rank_all(&cfg, NOW)[0].rank, Rank::NotReady);
}
