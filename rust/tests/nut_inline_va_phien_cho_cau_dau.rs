//! Quang 13/11 (Ubuntu, bot `@ai_hubha_bot`): *"còn 1 phiên -> bấm /close -> hiện
//! xác nhận nhưng bấm không có tác dụng"*.
//!
//! Hai lỗi trong cùng chuỗi, đo trên máy thật:
//! 1. `getWebhookInfo` của bot trả `"allowed_updates": ["message"]` — Telegram NHỚ
//!    danh sách của lượt gọi trước; `getUpdates` của huba không khai ⟹ mọi
//!    `callback_query` bị giữ lại (sổ `tin_den` Ubuntu: 0 callback từ trước tới
//!    nay) ⟹ `confirm_asked` → 90 s → `TimedOut`.
//! 2. Phiên còn lại là hàng «(chờ câu đầu)» `ses_e2cac047…`; `/shot` vào nó trả
//!    «không còn trong danh sách» vì `listed_in_opencode` chỉ hỏi KHO, mà phiên
//!    chưa có trong kho (opencode sinh ở câu đầu) — chỉ có liên kết huba đặt trước.
//!
//! MỘT bài trong tệp: bài đặt `HOME` của cả tiến trình.

mod common;

use huba::config::ClaudeAccountCfg;

#[test]
fn get_updates_khai_callback_query_va_phien_cho_cau_dau_van_co_ten() {
    // ── 1. URL getUpdates mang allowed_updates có callback_query ──────────────
    let url = huba::telegram::url_get_updates("https://x/botT/getUpdates", 7, 20);
    assert!(
        url.starts_with("https://x/botT/getUpdates?offset=7&timeout=20&"),
        "{url}"
    );
    assert!(
        url.contains(
            "allowed_updates=%5B%22message%22%2C%22edited_message%22%2C%22callback_query%22%5D"
        ),
        "phải khai tường minh, mã hoá URL: {url}"
    );
    assert!(huba::telegram::ALLOWED_UPDATES.contains("\"callback_query\""));

    // ── 2. Phiên chỉ có LIÊN KẾT (chưa có trong kho) vẫn «có tên» ─────────────
    let tmp = tempfile::tempdir().expect("tempdir");
    std::env::set_var("HOME", tmp.path());
    let mut cfg = common::cfg_for_tests();
    cfg.claude_accounts = vec![ClaudeAccountCfg {
        name: "accoc".into(),
        cli: Some("opencode".into()),
        ..Default::default()
    }];
    let acc = cfg.claude_accounts[0].clone();
    let id = "ses_e2cac0471fffGFyTCGSKYANqMg";
    // MẪU SỐ: chưa ghi gì ⟹ KHÔNG có tên (kho chưa tồn tại = rỗng hợp lệ).
    assert!(!huba::sessions::listed_in_opencode(&cfg, id));
    huba::opencode::write_binding(&acc, id, 13, "pts/2", Some("/w")).expect("ghi liên kết");
    assert!(
        huba::sessions::listed_in_opencode(&cfg, id),
        "phiên chờ câu đầu (chỉ có liên kết) bị coi là đã đi"
    );
    // Id khác vẫn KHÔNG có tên — hàm không trả `true` vô điều kiện.
    assert!(!huba::sessions::listed_in_opencode(&cfg, "ses_khac"));
}
