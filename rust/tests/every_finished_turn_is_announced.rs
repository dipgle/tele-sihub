//! Mọi lượt xong đều có tin về Telegram — dài hay ngắn.
//!
//! Hà 21/11: *«gửi tin xong xử lý xong thì output kết thúc là phải gửi về tele luôn»*. Cửa 120 giây (`MIN_RUN_SEC`)
//! từng im lượt 8 · 25 · 32 · 38 · 63 giây trên phiên `a91869f2`. Bài này khoá: lượt 1 giây cũng báo; lượt đang
//! chạy (chưa xong) thì KHÔNG báo (đối chứng ngược); câu báo nói giây, không nói «0 phút».

use std::collections::BTreeMap;

use huba::sessions::LiveSession;
use huba::watch::{changes, thoi_luong, Change, Idle, Mark};

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

fn phien(working: bool) -> LiveSession {
    LiveSession {
        session_id: ID.to_string(),
        name: "[huba]".into(),
        host: "terminal".into(),
        working,
        ..Default::default()
    }
}

fn so_finished(c: &[Change]) -> usize {
    c.iter()
        .filter(|x| matches!(x, Change::Finished { .. }))
        .count()
}

#[test]
fn a_turn_of_any_length_is_announced_when_it_ends() {
    for giay in [1, 8, 25, 63, 119, 120, 300] {
        let prev = so_dang_chay(NOW - giay);
        let (c, _) = changes(&prev, &[phien(false)], NOW, &[]);
        assert_eq!(so_finished(&c), 1, "lượt {giay} giây không có tin: {c:?}");
    }
}

/// Đối chứng ngược: lượt CHƯA xong thì không báo "xong" (phép đo đổi được trạng thái).
#[test]
fn a_turn_still_running_is_not_announced() {
    let prev = so_dang_chay(NOW - 30);
    let (c, _) = changes(&prev, &[phien(true)], NOW, &[]);
    assert_eq!(so_finished(&c), 0, "{c:?}");
}

#[test]
fn a_short_turn_says_seconds_not_zero_minutes() {
    let prev = so_dang_chay(NOW - 20);
    let (c, _) = changes(&prev, &[phien(false)], NOW, &[]);
    let cau = c[0].say(&Idle::Prompt, None);
    assert!(cau.contains("20 giây"), "{cau}");
    assert!(!cau.contains("0 phút"), "{cau}");
}

#[test]
fn duration_wording() {
    assert_eq!(thoi_luong(0), "0 giây");
    assert_eq!(thoi_luong(59), "59 giây");
    assert_eq!(thoi_luong(60), "1 phút");
    assert_eq!(thoi_luong(250), "4 phút");
    assert_eq!(thoi_luong(-5), "0 giây");
}
