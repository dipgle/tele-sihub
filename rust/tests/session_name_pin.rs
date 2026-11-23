//! Tên phiên GHIM XUỐNG ĐĨA từ lần đầu thấy (Quang 23/11: «hiện tại rất hay mất tên, từ đầu phiên đã quét được tên rồi phải gắn
//! cố định … thi thoảng đang làm nó nhảy sang tên khác rất là ảo»).
//!
//! Ca lấy từ CHÍNH log hubd (`session_change`, từ 20/11: 10/25 phiên đổi nhãn): mất ngoặc dự án (`[dwork/account]·user1-ubuntu` →
//! `projects-90·user1-ubuntu`), đổi đuôi (`[dwork/dci]·user1-ubuntu` → `[dwork/dci]·2eafa487`), nhãn trung tính được gắn dự án
//! thật (`[projects]·dfef9718` → `[zkey]`). Đối chứng: chuỗi nhãn THÔ có > 1 giá trị, chuỗi sau ghim chỉ còn một.

use std::collections::BTreeSet;

use huba::sessions::LiveSession;
use huba::ten_phien::{apply_pins, is_neutral, PinBook};

const ROOT: &str = "projects";
const T0: i64 = 1_795_000_000;

fn hang(id: &str, label: &str, bat_dau: i64) -> LiveSession {
    LiveSession {
        session_id: id.to_string(),
        label: label.to_string(),
        name: label.to_string(),
        host: "terminal".into(),
        started_at_ms: bat_dau,
        ..Default::default()
    }
}

/// Cho một phiên đi qua chuỗi nhãn thô theo thời gian; trả chuỗi nhãn SAU ghim.
fn qua_ghim(book: &mut PinBook, id: &str, thô: &[&str]) -> Vec<String> {
    thô.iter()
        .enumerate()
        .map(|(i, l)| {
            let mut rows = vec![hang(id, l, 1)];
            apply_pins(&mut rows, book, ROOT, T0 + (i as i64) * 60);
            rows[0].label.clone()
        })
        .collect()
}

fn khac_nhau(v: &[String]) -> usize {
    v.iter().collect::<BTreeSet<_>>().len()
}

// ── ba kiểu đổi nhãn có thật ────────────────────────────────────────────────────────────────────────────────────────────

#[test]
fn a_tail_that_churns_is_frozen_at_first_sight() {
    let thô = [
        "[dwork/dci]·user1-ubuntu",
        "[dwork/dci]·2eafa487",
        "[dwork/dci]·",
        "[dwork/dci]·fix nốt việc giở…",
    ];
    assert!(
        khac_nhau(&thô.iter().map(|s| s.to_string()).collect::<Vec<_>>()) > 1,
        "đối chứng: nhãn thô PHẢI đổi"
    );
    let mut b = PinBook::default();
    let ra = qua_ghim(&mut b, "2eafa487-c", &thô);
    assert_eq!(khac_nhau(&ra), 1, "{ra:?}");
    assert_eq!(ra[0], "[dwork/dci]·user1-ubuntu");
}

#[test]
fn a_known_project_is_never_lost_to_an_auto_name() {
    // ca thật 3a53bed8: mất ngoặc dự án, rơi về tên tự sinh
    let thô = [
        "[dwork/account]·user1-ubuntu",
        "projects-90·user1-ubuntu",
        "projects-90",
        "[dwork/account]·user1-ubuntu",
    ];
    let mut b = PinBook::default();
    let ra = qua_ghim(&mut b, "3a53bed8-e", &thô);
    assert_eq!(khac_nhau(&ra), 1, "{ra:?}");
    assert_eq!(ra[3], "[dwork/account]·user1-ubuntu");
}

#[test]
fn one_real_project_never_jumps_to_another() {
    // ca thật: [aihub] → [huba]·id, [remote]·id → [huba], [init-project] → [zkey]
    for thô in [
        ["[aihub]", "[huba]·01a1d979"],
        ["[remote]·01a1da62", "[huba]"],
        ["[init-project]", "[zkey]"],
    ] {
        let mut b = PinBook::default();
        let ra = qua_ghim(&mut b, "s1", &thô);
        assert_eq!(khac_nhau(&ra), 1, "{thô:?} ⟹ {ra:?}");
        assert_eq!(ra[0], thô[0]);
    }
}

/// Nhãn TRUNG TÍNH («chưa giao dự án») được gắn dự án thật ĐÚNG MỘT LẦN — rồi ghim luôn (ca `[projects]·dfef9718` → `[zkey]`).
#[test]
fn a_neutral_label_gains_a_real_project_once_then_sticks() {
    let thô = [
        "[projects]·dfef9718",
        "[projects]·dfef9718",
        "[zkey]·",
        "[huba]·x",
        "[projects]",
    ];
    let mut b = PinBook::default();
    let ra = qua_ghim(&mut b, "ses_dfef97186fff", &thô);
    assert_eq!(
        ra,
        [
            "[projects]·dfef9718",
            "[projects]·dfef9718",
            "[zkey]·",
            "[zkey]·",
            "[zkey]·"
        ]
    );
    // ngăn kéo gốc cũng là trung tính: `[projects/dci]` → `[dwork/dci]`
    let mut b2 = PinBook::default();
    let r2 = qua_ghim(&mut b2, "ses_dff8f9", &["[projects/dci]·", "[dwork/dci]·"]);
    assert_eq!(r2[1], "[dwork/dci]·");
}

#[test]
fn what_counts_as_neutral() {
    for l in [
        "projects-90·user1-ubuntu",
        "Thử HUBA với lựa chọn tab",
        "[projects]·dfef9718",
        "[projects/dci]",
        "",
        "[projects]",
    ] {
        assert!(is_neutral(l, ROOT), "«{l}» phải là trung tính");
    }
    for l in [
        "[dwork/dci]·x",
        "[zkey]",
        "[huba]·user1-ubuntu",
        "[remote/android]",
    ] {
        assert!(!is_neutral(l, ROOT), "«{l}» KHÔNG được là trung tính");
    }
}

// ── trùng nhãn: người đến sau mang đuôi, người đến trước KHÔNG đổi ────────────────────────────────────────────────────────

#[test]
fn a_newcomer_with_the_same_label_gets_a_suffix_and_the_old_one_keeps_its_name() {
    let mut b = PinBook::default();
    let mut r1 = vec![hang("aaaa1111-x", "[dwork/review]", 100)];
    apply_pins(&mut r1, &mut b, ROOT, T0);
    assert_eq!(r1[0].label, "[dwork/review]");
    // phiên thứ hai cùng nhãn đến sau
    let mut r2 = vec![
        hang("aaaa1111-x", "[dwork/review]", 100),
        hang("bbbb2222-y", "[dwork/review]", 200),
    ];
    apply_pins(&mut r2, &mut b, ROOT, T0 + 60);
    assert_eq!(
        r2[0].label, "[dwork/review]",
        "phiên cũ bị đổi tên vì có người mới đến"
    );
    assert_ne!(r2[1].label, "[dwork/review]");
    assert!(
        r2[1].label.starts_with("[dwork/review]·"),
        "{}",
        r2[1].label
    );
    // phiên cũ tắt: phiên mới GIỮ đuôi (không rụng lại thành nhãn trần)
    let mut r3 = vec![hang("bbbb2222-y", "[dwork/review]", 200)];
    apply_pins(&mut r3, &mut b, ROOT, T0 + 120);
    assert_eq!(
        r3[0].label, r2[1].label,
        "nhãn rụng đuôi khi bạn cùng nhãn tắt"
    );
}

#[test]
fn shell_windows_are_never_pinned() {
    let mut b = PinBook::default();
    let mut r = vec![LiveSession {
        host: "shell".into(),
        session_id: "tty-1".into(),
        label: "zsh".into(),
        ..Default::default()
    }];
    apply_pins(&mut r, &mut b, ROOT, T0);
    assert!(b.pins.is_empty(), "cửa sổ trống bị ghim (tty bị TÁI DÙNG)");
}

// ── bền qua khởi động lại ──────────────────────────────────────────────────────────────────────────────────────────────────

#[test]
fn pins_survive_a_restart_through_the_file() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("data").join("ten-phien.json");
    let mut rows = vec![hang("s-ben", "[dwork/dci]·user1-ubuntu", 1)];
    huba::ten_phien::pin_labels_at(&mut rows, &p, ROOT, T0);
    assert!(p.is_file(), "chưa ghi tệp");
    // tiến trình mới (sổ trong RAM mất sạch): nhãn thô đổi, nhãn hiển thị KHÔNG đổi
    let mut rows2 = vec![hang("s-ben", "projects-77", 1)];
    huba::ten_phien::pin_labels_at(&mut rows2, &p, ROOT, T0 + 5000);
    assert_eq!(rows2[0].label, "[dwork/dci]·user1-ubuntu");
}

#[test]
fn a_corrupt_file_starts_clean_and_says_so() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("ten-phien.json");
    std::fs::write(&p, "{không phải json").unwrap();
    let (b, canh_bao) = PinBook::load(&p);
    assert!(b.pins.is_empty());
    assert!(canh_bao.is_some(), "tệp hỏng mà im lặng");
    // đối chứng ngược: tệp vắng KHÔNG phải lỗi
    let (b2, cb2) = PinBook::load(&d.path().join("khong-co.json"));
    assert!(b2.pins.is_empty() && cb2.is_none());
}

#[test]
fn an_unchanged_book_is_not_rewritten_every_cycle() {
    let mut b = PinBook::default();
    let mut r = vec![hang("s1", "[huba]·a", 1)];
    assert!(
        apply_pins(&mut r, &mut b, ROOT, T0),
        "lần đầu phải báo có đổi"
    );
    let mut r = vec![hang("s1", "[huba]·a", 1)];
    assert!(
        !apply_pins(&mut r, &mut b, ROOT, T0 + 30),
        "không đổi gì mà vẫn đòi ghi đĩa"
    );
    let mut r = vec![hang("s1", "[huba]·a", 1)];
    assert!(
        apply_pins(&mut r, &mut b, ROOT, T0 + 4000),
        "sau hơn 1 giờ phải làm mới mốc «đã thấy»"
    );
}

#[test]
fn pins_of_long_gone_sessions_are_dropped() {
    let mut b = PinBook::default();
    let mut r = vec![hang("cu", "[a]", 1)];
    apply_pins(&mut r, &mut b, ROOT, T0);
    let mut r = vec![hang("moi", "[b]", 1)];
    apply_pins(&mut r, &mut b, ROOT, T0 + 15 * 86400);
    assert!(
        !b.pins.contains_key("cu"),
        "ghim của phiên 15 ngày trước còn đó"
    );
    assert!(b.pins.contains_key("moi"));
}

// ── lối thoát: chủ máy tự đặt / bỏ ghim ───────────────────────────────────────────────────────────────────────────────────────

#[test]
fn the_owner_can_rename_and_unpin() {
    let mut b = PinBook::default();
    let mut r = vec![hang("s1", "[projects]·abc", 1)];
    apply_pins(&mut r, &mut b, ROOT, T0);
    b.set("s1", "[zkey]·việc A", T0);
    let mut r = vec![hang("s1", "[projects]·abc", 1)];
    apply_pins(&mut r, &mut b, ROOT, T0 + 60);
    assert_eq!(r[0].label, "[zkey]·việc A", "tên chủ máy đặt bị ghi đè");
    assert!(b.remove("s1"));
    assert!(
        !b.remove("s1"),
        "bỏ ghim một phiên chưa ghim phải báo false"
    );
    let mut r = vec![hang("s1", "[huba]·mới", 1)];
    apply_pins(&mut r, &mut b, ROOT, T0 + 120);
    assert_eq!(r[0].label, "[huba]·mới", "bỏ ghim xong phải học lại từ đầu");
}
