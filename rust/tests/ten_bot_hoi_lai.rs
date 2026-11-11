//! Tên bot phải được HỎI LẠI khi lượt `getMe` lúc khởi động hỏng.
//!
//! 🔴 Quang 2026-11-11, sau hai lần máy sập: *"Sao cứ khởi động lại máy danh
//! sách phiên lại thành thế này"* · *"Phải chạy lệnh upgrade"*. Đo trong
//! `logs/huba.log`: hubd lên lúc 07:04:51 khi mạng CHƯA có —
//! `telegram_getme_failed … client error (Connect)` — rồi mọi lượt `/session`
//! sau đó ghi `session_taps_partial rows:8 linked:0` ("danh sách rơi về khối nút
//! ở đáy"), cho tới khi `/upgrade` khởi động lại hubd lúc 07:19.
//!
//! Gốc: `getMe` chỉ được gọi MỘT lần ở đầu `read_forever`. Hỏng là
//! `BOT_USERNAME` rỗng mãi ⟹ `deep_link` trả `None` mãi ⟹ không dòng nào trong
//! danh sách bấm được.
//!
//! Không có máy chủ Telegram giả (URL API viết cứng), nên khoá bằng hai lớp:
//! đọc mã nguồn của vòng đọc (phải hỏi lại SAU một lượt `getUpdates` thành công
//! — lúc ấy mạng đã chứng minh là có), và hàm thuần quyết định có hỏi lại không.

use std::time::{Duration, Instant};

fn read_forever_src() -> String {
    let src = include_str!("../src/telegram.rs");
    let start = src
        .find("fn read_forever(&self)")
        .expect("không thấy fn read_forever trong telegram.rs");
    let rest = &src[start..];
    let end = rest
        .find("fn so_tin<")
        .expect("không thấy hàm kế sau read_forever (so_tin) — đổi tên thì sửa bài này");
    rest[..end].to_string()
}

#[test]
fn vong_doc_hoi_lai_ten_bot_sau_mot_luot_poll_thanh_cong() {
    let body = read_forever_src();
    // Đối chứng: bài này phải đang nhìn đúng vòng đọc, không phải một khúc rỗng.
    let loop_at = body
        .find("loop {")
        .expect("read_forever không còn `loop {`");
    let rejected_at = body[loop_at..]
        .find("poll_rejected(&resp)")
        .map(|i| i + loop_at)
        .expect("vòng đọc không còn kiểm poll_rejected");
    let retry_at = body[rejected_at..]
        .find("self.hoi_ten_bot(")
        .map(|i| i + rejected_at);
    assert!(
        retry_at.is_some(),
        "vòng đọc KHÔNG hỏi lại tên bot sau một lượt getUpdates thành công — \
         getMe hỏng lúc khởi động là danh sách phiên mất liên kết tới khi khởi động lại hubd"
    );
    assert!(
        body[rejected_at..retry_at.unwrap()].contains("can_hoi_lai_ten_bot("),
        "lượt hỏi lại phải đi qua can_hoi_lai_ten_bot (có nhịp lùi), không gõ getMe mỗi vòng"
    );
}

#[test]
fn da_biet_ten_thi_khong_hoi_nua() {
    let now = Instant::now();
    assert!(!huba::telegram::can_hoi_lai_ten_bot(true, None, now));
    assert!(!huba::telegram::can_hoi_lai_ten_bot(
        true,
        Some(now - Duration::from_secs(3600)),
        now
    ));
}

#[test]
fn chua_biet_ten_thi_hoi_ngay_lan_dau_roi_lui_nhip() {
    let now = Instant::now();
    let gap = huba::telegram::HOI_LAI_TEN_BOT_SAU;
    assert!(huba::telegram::can_hoi_lai_ten_bot(false, None, now));
    // Vừa hỏi xong ⟹ chưa hỏi lại (mạng chập thì không gõ getMe mỗi 3 giây).
    assert!(!huba::telegram::can_hoi_lai_ten_bot(false, Some(now), now));
    assert!(!huba::telegram::can_hoi_lai_ten_bot(
        false,
        Some(now - (gap - Duration::from_secs(1))),
        now
    ));
    assert!(huba::telegram::can_hoi_lai_ten_bot(
        false,
        Some(now - gap),
        now
    ));
}
