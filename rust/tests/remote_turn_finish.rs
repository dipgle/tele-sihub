//! Lượt do chủ máy gõ TỪ TELEGRAM xong thì phải có tin về, dù ngắn hơn `MIN_RUN_SEC`.
//!
//! Hà 21/11: *«lượt xử lý xong ko thấy gửi về tele»*. Đo trên phiên `a91869f2`: lượt 4 phút ⟹ có 💤;
//! bốn lượt sau (8 · 32 · 63 · 23 giây) ⟹ im, đúng luật cũ — luật sinh ra cho phiên có người ngồi nhìn.
//! Bài này khoá ba điều: (1) lượt ngắn do Telegram gõ ⟹ CÓ tin; (2) ĐỐI CHỨNG NGƯỢC — cùng lượt ngắn mà
//! chủ máy KHÔNG gõ từ Telegram (hoặc lần gõ đã cũ) ⟹ vẫn im; (3) câu nói ra không còn «sau 0 phút».

use std::collections::BTreeMap;

use huba::sessions::LiveSession;
use huba::watch::{
    changes, changes_remote, thoi_luong, Change, Idle, Mark, MIN_RUN_SEC, REMOTE_LEAD_SEC,
};

const NOW: i64 = 1_800_000_000;
const ID: &str = "a91869f2-6f51-4ae8-81f6-d1eaef362177";

fn so_dang_chay(bat_dau: i64) -> BTreeMap<String, Mark> {
    let mut m = BTreeMap::new();
    m.insert(
        ID.to_string(),
        Mark {
            s: format!("working@{bat_dau}"),
            y: "ttys009".into(),
            k: "interactive".into(),
            f: NOW - 3600,
            a: "acc1".into(),
            i: 4242,
            o: "terminal".into(),
            ..Default::default()
        },
    );
    m
}

fn phien_ranh() -> LiveSession {
    LiveSession {
        session_id: ID.to_string(),
        name: "[huba]".into(),
        host: "terminal".into(),
        working: false,
        ..Default::default()
    }
}

fn so_finished(c: &[Change]) -> usize {
    c.iter()
        .filter(|x| matches!(x, Change::Finished { .. }))
        .count()
}

fn go(t: i64) -> BTreeMap<String, i64> {
    let mut m = BTreeMap::new();
    m.insert(ID.to_string(), t);
    m
}

#[test]
fn a_short_turn_typed_from_telegram_is_announced() {
    let bat_dau = NOW - 30; // lượt 30 giây ≪ MIN_RUN_SEC
    let prev = so_dang_chay(bat_dau);
    let (c, _) = changes_remote(&prev, &[phien_ranh()], NOW, &[], &go(bat_dau - 2));
    assert_eq!(
        so_finished(&c),
        1,
        "lượt ngắn do Telegram gõ không có tin: {c:?}"
    );
    // Câu nói ra phải nói giây, không phải «0 phút».
    if let Some(Change::Finished { ran_sec, .. }) = c.first() {
        assert_eq!(*ran_sec, 30);
        let cau = c[0].say(&Idle::Prompt, None);
        assert!(cau.contains("30 giây"), "{cau}");
        assert!(!cau.contains("0 phút"), "{cau}");
    }
}

/// Đối chứng ngược #1 — CÙNG lượt ngắn, chủ máy không gõ từ Telegram ⟹ IM như trước.
#[test]
fn the_same_short_turn_without_a_telegram_input_stays_quiet() {
    let prev = so_dang_chay(NOW - 30);
    let (c, _) = changes(&prev, &[phien_ranh()], NOW, &[]);
    assert_eq!(
        so_finished(&c),
        0,
        "phép đo mù: lượt ngắn bình thường cũng kêu: {c:?}"
    );
    let (c2, _) = changes_remote(&prev, &[phien_ranh()], NOW, &[], &BTreeMap::new());
    assert_eq!(so_finished(&c2), 0, "{c2:?}");
}

/// Đối chứng ngược #2 — lần gõ từ Telegram đã CŨ (trước lượt này quá `REMOTE_LEAD_SEC`) ⟹ không tính.
#[test]
fn an_old_telegram_input_does_not_make_a_later_desk_turn_loud() {
    let bat_dau = NOW - 30;
    let prev = so_dang_chay(bat_dau);
    let (c, _) = changes_remote(
        &prev,
        &[phien_ranh()],
        NOW,
        &[],
        &go(bat_dau - REMOTE_LEAD_SEC - 1),
    );
    assert_eq!(so_finished(&c), 0, "{c:?}");
    // biên: đúng REMOTE_LEAD_SEC trước ⟹ còn tính
    let (c2, _) = changes_remote(
        &prev,
        &[phien_ranh()],
        NOW,
        &[],
        &go(bat_dau - REMOTE_LEAD_SEC),
    );
    assert_eq!(so_finished(&c2), 1, "{c2:?}");
}

/// Lần gõ nằm trong TƯƠNG LAI so với «bây giờ» (đồng hồ lệch) không được mở cửa.
#[test]
fn a_future_timestamp_never_opens_the_gate() {
    let prev = so_dang_chay(NOW - 30);
    let (c, _) = changes_remote(&prev, &[phien_ranh()], NOW, &[], &go(NOW + 500));
    assert_eq!(so_finished(&c), 0, "{c:?}");
}

/// Phiên KHÁC được gõ từ Telegram không làm phiên này kêu.
#[test]
fn another_sessions_telegram_input_is_not_this_ones() {
    let prev = so_dang_chay(NOW - 30);
    let mut m = BTreeMap::new();
    m.insert("phien-khac".to_string(), NOW - 31);
    let (c, _) = changes_remote(&prev, &[phien_ranh()], NOW, &[], &m);
    assert_eq!(so_finished(&c), 0, "{c:?}");
}

/// Lượt dài vẫn báo như cũ dù không có dấu Telegram (không hồi quy).
#[test]
fn a_long_turn_is_still_announced_without_any_marker() {
    let prev = so_dang_chay(NOW - MIN_RUN_SEC - 5);
    let (c, _) = changes(&prev, &[phien_ranh()], NOW, &[]);
    assert_eq!(so_finished(&c), 1, "{c:?}");
}

#[test]
fn duration_wording() {
    assert_eq!(thoi_luong(0), "0 giây");
    assert_eq!(thoi_luong(59), "59 giây");
    assert_eq!(thoi_luong(60), "1 phút");
    assert_eq!(thoi_luong(250), "4 phút");
    assert_eq!(thoi_luong(-5), "0 giây");
}
