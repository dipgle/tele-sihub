//! GỬI bảng hỏi NHIỀU CÂU trên cửa sổ Terminal **THẬT** — phím rời, một Enter.
//!
//! Bốn bài trong `submit_bang_nhieu_cau.rs` đo KẾ HOẠCH trên chữ màn chụp. Bài
//! này đo **cửa sổ thật**: nó bấm số để trả lời hết các câu, rồi đi tới trang
//! `Submit` bằng mũi tên rời và bấm ĐÚNG MỘT Enter, rồi ĐỌC LẠI màn để đòi
//! thanh tab phải biến mất.
//!
//! Vì sao phải có: Quang 08/11 — *"chọn xong là bị treo, phải Ctrl+C"*. Treo là
//! trạng thái mà một bài kiểm trên chuỗi KHÔNG nhìn thấy được; nó chỉ hiện ra
//! khi phím thật rơi vào TUI thật. Bài kiểm chuỗi giữ hình dạng kế hoạch, bài
//! này giữ kết cục.
//!
//! ```text
//! HUB_ASK_WINDOW=86175 cargo test --offline --test submit_bang_that_live \
//!   -- --ignored --nocapture
//! ```
//!
//! `#[ignore]` vì nó gõ vào cửa sổ thật của chủ máy — không được chạy trong lượt
//! `cargo test` thường. Cùng khuôn `bare_esc_live.rs`.

#![cfg(target_os = "macos")]

use huba::keys;

fn window() -> i64 {
    std::env::var("HUB_ASK_WINDOW")
        .expect("cần HUB_ASK_WINDOW=<id cửa sổ Terminal>")
        .parse()
        .expect("HUB_ASK_WINDOW phải là một số")
}

fn man(w: i64, khi: &str) -> String {
    let s = keys::screen_text(w).unwrap_or_else(|e| panic!("không đọc được màn {khi}: {e}"));
    s
}

/// Gửi một dãy phím RỜI vào cửa sổ. Dùng để trả lời những hộp thoại mà huba
/// không có route riêng (hộp tin cậy thư mục, hộp chọn của TUI). Tách riêng để
/// lượt đo bảng không phải giả vờ rằng nó tự mở được cửa sổ.
#[test]
#[ignore = "gõ vào cửa sổ THẬT — chạy tay bằng --ignored"]
fn gui_phim_roi() {
    let w = window();
    let raw = std::env::var("HUB_KEYS").expect("cần HUB_KEYS=down,enter");
    let phim: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    assert!(!phim.is_empty(), "HUB_KEYS rỗng");
    // Quyền Trợ năng là điều kiện CẦN của phím rời; thiếu nó thì sự kiện rơi
    // vào hư không TRONG IM LẶNG. Hỏi trước, đừng để lượt đo hỏng bị đọc thành
    // "phím rời không ăn".
    assert!(
        huba::cgkeys::trusted(),
        "tiến trình này chưa có quyền Trợ năng — KHÔNG đo được, đừng kết luận"
    );
    println!("gửi phím rời vào cửa sổ {w}: {phim:?}");
    keys::send_bare(w, &phim).expect("gửi phím rời hỏng");
}

/// 🎯 Bài chính: trả lời HẾT rồi GỬI bằng phím rời; bảng phải biến mất.
#[test]
#[ignore = "gõ vào cửa sổ THẬT — chạy tay bằng --ignored"]
fn gui_bang_nhieu_cau_bang_phim_roi() {
    let w = window();
    assert!(
        huba::cgkeys::trusted(),
        "tiến trình này chưa có quyền Trợ năng — KHÔNG đo được, đừng kết luận"
    );

    // Chờ bảng xuất hiện: khi bài này chạy nền TRƯỚC lúc bảng được dựng (ca đo
    // 08/11 — phiên đang chạy chính bài kiểm này), cửa sổ chưa có tab nào để
    // đọc, và "không thấy bảng" lúc ấy KHÔNG phải kết luận gì.
    let mut truoc = String::new();
    for _ in 0..45 {
        truoc = man(w, "chờ bảng");
        if keys::ask_table(&truoc).is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    let bang = keys::ask_table(&truoc).unwrap_or_else(|| {
        panic!(
            "cửa sổ {w} không có thanh tab sau 90s — bài này không đo được gì. 400 ký tự cuối:\n{}",
            cuoi(&truoc, 400)
        )
    });
    let n = bang.answered.len();
    println!(
        "TRƯỚC: {n} câu, đã trả lời = {:?}, tiêu đề = {:?}",
        bang.answered, bang.headers
    );
    assert!(
        n >= 2,
        "bảng chỉ có {n} câu — đây là hộp MỘT câu, đường cũ (không đụng tới) mới là đường của nó"
    );

    // ① Trả lời từng câu bằng ĐÚNG một phím số, đọc lại sau mỗi lần. Số ô đã
    //    trả lời phải TĂNG đúng một: đó là phép đo cho biết cú bấm rơi vào câu
    //    đang mở, chứ không phải vào câu bên cạnh.
    let mut da_tra_loi = bang.answered.iter().filter(|b| **b).count();
    for vong in 0..n {
        if da_tra_loi >= n {
            break;
        }
        keys::send_bare(w, &["1".to_string()]).expect("gửi số hỏng");
        std::thread::sleep(std::time::Duration::from_millis(900));
        let sau = man(w, "sau khi bấm số");
        let Some(b) = keys::ask_table(&sau) else {
            panic!("bảng biến mất sau khi bấm số ở vòng {vong} — không phải kết cục mong đợi");
        };
        let moi = b.answered.iter().filter(|x| **x).count();
        println!("vòng {vong}: đã trả lời {da_tra_loi} → {moi}");
        assert!(
            moi > da_tra_loi,
            "bấm số mà không chốt thêm câu nào (vòng {vong}) — cú bấm rơi sai chỗ, \
             đừng đi tiếp rồi quy tội cho đường gửi"
        );
        da_tra_loi = moi;
    }
    assert_eq!(
        da_tra_loi, n,
        "còn câu chưa trả lời thì bảng KHÔNG gửi đi được"
    );

    // ② Đi tới trang `Submit` rồi bấm Enter — đúng hàm mà `/send_…` dùng.
    let san_sang = man(w, "TRƯỚC KHI GỬI");
    let at = std::env::var("HUB_ASK_AT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(n - 1);
    let phim = keys::submit_bare_keys(&san_sang, at)
        .unwrap_or_else(|| panic!("bảng nhiều câu mà không dựng được kế hoạch phím rời (at={at})"));
    println!("GỬI bằng phím rời: {phim:?} (đang đứng câu {})", at + 1);
    let (enter, mui_ten) = phim.split_last().unwrap();
    if !mui_ten.is_empty() {
        keys::send_bare(w, mui_ten).expect("gửi mũi tên hỏng");
        std::thread::sleep(std::time::Duration::from_millis(600));
    }
    keys::send_bare(w, std::slice::from_ref(enter)).expect("gửi Enter hỏng");
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // ③ KẾT CỤC: thanh tab phải BIẾN MẤT. Đây là câu hỏi của cả bài — bảng đã
    //    gửi đi, hay nó đứng trắng như ca treo?
    let sau = man(w, "SAU");
    println!("SAU (400 ký tự cuối):\n{}", cuoi(&sau, 400));
    assert!(
        keys::ask_table(&sau).is_none(),
        "bảng VẪN CÒN sau cú gửi — đúng hình dạng treo cần diệt:\n{}",
        cuoi(&sau, 600)
    );
}

fn cuoi(s: &str, n: usize) -> String {
    let mut v: Vec<char> = s.chars().collect();
    if v.len() > n {
        v = v.split_off(v.len() - n);
    }
    v.into_iter().collect()
}
