use huba::telegram::Inbox;

/// Trần Telegram cho `text` (đếm ký tự UTF-16; chữ Việt/emoji trong BMP ⟹ = số ký tự).
const TRAN: usize = 4096;

fn tin_kieu_accounts(so_hang: usize) -> String {
    let mut s = format!("👤 {so_hang} tài khoản · 🔒 9 đang khoá\n\n");
    for i in 1..=so_hang {
        s.push_str(&format!(
            "acc{i} · nguoi{i}@gmail.com · không có phiên nào đang chạy\n    🔒 ĐANG KHOÁ — huba không tự chọn, không tự mở phiên, không tự dò /usage · đo lại: /accounts acc{i} · mở: /accounts acc{i} -a\n    hạn mức: tuần 36% · 5 tiếng 4% · Fable 0% · hạng: chưa đo được · đo 1103 tiếng trước\n    phiên gần nhất (projects · mở 48 ngày trước, dừng 47 ngày trước)\n\n"
        ));
    }
    s
}

#[test]
fn tin_dai_hon_tran_di_thanh_nhieu_tin_moi_tin_duoi_tran() {
    let text = tin_kieu_accounts(20);
    assert!(text.chars().count() > TRAN, "ca thử phải VƯỢT trần thật");
    let than = Inbox::plain_message_bodies("1", &text);
    assert!(than.len() >= 2, "vượt trần ⟹ phải chia: {} tin", than.len());
    for (i, b) in than.iter().enumerate() {
        let t = b["text"].as_str().unwrap_or("");
        assert!(
            t.chars().count() <= TRAN,
            "mẩu {i} dài {} > {TRAN}",
            t.chars().count()
        );
    }
    // Không mất chữ: mọi hàng tài khoản đều có trong một mẩu nào đó.
    let gop: String = than
        .iter()
        .map(|b| b["text"].as_str().unwrap_or("").to_string())
        .collect();
    for i in 1..=20 {
        assert!(
            gop.contains(&format!("nguoi{i}@gmail.com")),
            "mất hàng acc{i}"
        );
    }
    // Mẩu sau có dấu nối (luật cắt-làm-đôi-thì-nói-ra của `split_for_telegram`).
    assert!(than[1]["text"]
        .as_str()
        .unwrap_or("")
        .contains("nối tiếp tin trên"));
}

#[test]
fn tin_ngan_van_la_mot_tin_nguyen_van() {
    let than = Inbox::plain_message_bodies("1", "một dòng ngắn");
    assert_eq!(than.len(), 1);
    assert_eq!(than[0]["text"], "một dòng ngắn");
    assert_eq!(than[0]["chat_id"], "1");
}
