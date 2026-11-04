//! Số vào bảng hỏi NHIỀU câu — đo trên cửa sổ `claude` THẬT, hai đường.
//!
//! Bài thuần `so_tren_bang_nhieu_cau.rs` chỉ khoá phép CHỌN đường; còn câu "đường
//! nào làm gì với CLI" thì chỉ CLI thật trả lời được. Hai bài, mỗi bài một bảng
//! 3 câu MỚI (chưa câu nào trả lời, đang đứng ở câu 1), câu hỏi đúng ba chữ dưới:
//!
//! * `duong_byte_nhay_coc` — đường CŨ (`keys::press`, số + CR một lượt ghi) bấm
//!   `1` ở câu 1. Giả thuyết gốc: CLI chốt HAI lần ⟹ câu 2 bị nhảy qua.
//! * `duong_roi_di_dung_tung_cau` — đường MỚI (`keys::send_bare`) bấm `1` bốn
//!   lần: câu 1 → câu 2 → câu 3 → trang Submit → gửi. Nhật ký phiên phải nhận
//!   câu trả lời (`Your questions have been answered`).
//!
//! ```text
//! HUB_ASK_WINDOW=<id> HUB_ASK_SESSION=<session id> \
//!   cargo test --offline --test so_tren_bang_nhieu_cau_live -- --ignored --nocapture <tên bài>
//! ```
//!
//! `#[ignore]` vì nó gõ vào cửa sổ thật.

#![cfg(target_os = "macos")]

use huba::keys::{ask_table, cursor_on, parse_choices, press, screen_text, send_bare};

const CAU: [&str; 3] = ["Câu thử A?", "Câu thử B?", "Câu thử C?"];

fn cua_so() -> i64 {
    std::env::var("HUB_ASK_WINDOW")
        .expect("cần HUB_ASK_WINDOW=<id cửa sổ>")
        .parse()
        .expect("HUB_ASK_WINDOW phải là một số")
}

fn cau_hoi() -> Vec<String> {
    CAU.iter().map(|s| s.to_string()).collect()
}

/// Đọc màn sau một nhịp vẽ lại; trả (ô đã trả lời, câu đang mở, có trang Submit).
fn doc(w: i64, nhan: &str) -> (Vec<bool>, Option<usize>, bool, String) {
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let man = screen_text(w).expect("không đọc được màn");
    let bang = ask_table(&man);
    let tra_loi = bang.map(|b| b.answered).unwrap_or_default();
    let mo = cursor_on(&man, &cau_hoi());
    let submit = parse_choices(&man)
        .iter()
        .any(|(_, l)| l.contains("Submit answers"));
    println!("[{nhan}] trả lời={tra_loi:?} · câu mở={mo:?} · trang Submit={submit}");
    (tra_loi, mo, submit, man)
}

fn bat_dau(w: i64) {
    let (tra_loi, mo, _, man) = doc(w, "TRƯỚC");
    assert_eq!(
        tra_loi,
        vec![false, false, false],
        "cửa sổ {w} không phải một bảng 3 câu MỚI — bài này không đo được gì:\n{man}"
    );
    assert_eq!(mo, Some(0), "bảng không đứng ở câu 1:\n{man}");
}

#[test]
#[ignore = "gõ vào cửa sổ THẬT — chạy tay bằng --ignored"]
fn duong_byte_nhay_coc() {
    let w = cua_so();
    bat_dau(w);
    press(w, "1").expect("do script hỏng");
    let (tra_loi, mo, _, man) = doc(w, "SAU byte '1'");
    // Đây là phép ĐO giả thuyết, không phải hành vi mong muốn: đường cũ phải
    // nhảy CÓC (đứng ở câu 3, câu 2 còn trống). Ra câu 2 thì giả thuyết sai.
    assert_eq!(tra_loi, vec![true, false, false], "{man}");
    assert_eq!(
        mo,
        Some(2),
        "đường byte KHÔNG nhảy cóc — giả thuyết gốc sai:\n{man}"
    );

    // Đúng cú bấm của ca treo: byte `1` ở câu CUỐI ⟹ chỉ số vượt trang Submit ⟹
    // màn không còn thanh tab, không còn trang Submit — mà bảng vẫn chờ.
    press(w, "1").expect("do script hỏng");
    let (tra_loi, _, submit, man) = doc(w, "SAU byte '1' ở câu cuối");
    assert!(
        tra_loi.is_empty() && !submit,
        "đường byte ở câu cuối KHÔNG ra màn treo — giả thuyết gốc sai:\n{man}"
    );

    // Lối gỡ đã khuyên chủ máy: `/key left` (byte: ←  + CR). `←` lùi chỉ số về
    // đúng trang Submit; CR rơi vào cây `null` nên không chốt gì.
    press(w, "left").expect("do script hỏng");
    let (_, _, submit, man) = doc(w, "SAU byte '←' trên màn treo");
    assert!(
        submit,
        "`/key left` KHÔNG đưa trang Submit về — lối gỡ sai:\n{man}"
    );
}

#[test]
#[ignore = "gõ vào cửa sổ THẬT — chạy tay bằng --ignored"]
fn duong_roi_di_dung_tung_cau() {
    let w = cua_so();
    let sid = std::env::var("HUB_ASK_SESSION").expect("cần HUB_ASK_SESSION=<session id>");
    bat_dau(w);
    let mot = vec!["1".to_string()];

    send_bare(w, &mot).expect("phím rời hỏng (quyền Trợ năng?)");
    let (tra_loi, mo, _, man) = doc(w, "SAU rời '1' ở câu 1");
    assert_eq!((tra_loi, mo), (vec![true, false, false], Some(1)), "{man}");

    send_bare(w, &mot).unwrap();
    let (tra_loi, mo, _, man) = doc(w, "SAU rời '1' ở câu 2");
    assert_eq!((tra_loi, mo), (vec![true, true, false], Some(2)), "{man}");

    send_bare(w, &mot).unwrap();
    let (tra_loi, _, submit, man) = doc(w, "SAU rời '1' ở câu 3");
    assert_eq!(tra_loi, vec![true, true, true], "{man}");
    assert!(
        submit,
        "câu cuối KHÔNG dẫn tới trang Submit — đúng hình dạng treo:\n{man}"
    );

    send_bare(w, &mot).unwrap();
    let (_, _, submit, man) = doc(w, "SAU rời '1' ở trang Submit");
    assert!(!submit, "bấm Submit mà trang vẫn còn:\n{man}");

    // Bằng chứng cuối là NHẬT KÝ của phiên, không phải màn: câu trả lời phải tới.
    let home = std::env::var("HOME").unwrap();
    let mut tim_thay = false;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let pattern = format!("{home}/.claude/projects");
        for d in std::fs::read_dir(&pattern).unwrap().flatten() {
            let p = d.path().join(format!("{sid}.jsonl"));
            if let Ok(s) = std::fs::read_to_string(&p) {
                if s.contains("Your questions have been answered") {
                    tim_thay = true;
                }
            }
        }
        if tim_thay {
            break;
        }
    }
    println!("nhật ký nhận câu trả lời = {tim_thay}");
    assert!(tim_thay, "20 s mà nhật ký phiên {sid} chưa có câu trả lời");
}
