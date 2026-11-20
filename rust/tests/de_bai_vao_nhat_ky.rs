//! `/new <chữ>` — đề bài đã vào NHẬT KÝ phiên chưa (Hà 2026-11-20).
//!
//! Ca thật: huba gõ đề bài 09:14:45Z và ghi `new_task_typed` (màn không thấy
//! chữ trong ô vì ô chưa được vẽ), mà `claude` chỉ NHẬN câu lúc 09:15:31Z. Nguồn
//! đúng là nhật ký `.jsonl`. Hai chiều: có ⟹ true, chưa có ⟹ false.

use huba::sessions::de_bai_da_vao_nhat_ky;

const DE_BAI: &str =
    "remote: khi zoom màn và chọn chế độ chuột độc lập thì phải luôn đảm bảo chuột trong khung nhìn";

/// Dòng nhật ký THẬT của lượt `user` (dạng đo ở phiên 09866d7c, rút gọn trường).
fn dong_user(chu: &str) -> String {
    serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": chu },
        "promptSource": "typed",
    })
    .to_string()
}

/// Dòng hook SessionStart — có đủ chữ khác, KHÔNG phải lượt nhập.
fn dong_hook() -> String {
    serde_json::json!({
        "type": "attachment",
        "attachment": { "type": "hook_success", "content": "KHỞI ĐỘNG PHIÊN" },
    })
    .to_string()
}

#[test]
fn a_journal_with_the_task_as_a_user_turn_counts_as_received() {
    let de_bai = format!("{DE_BAI}\n\nTRƯỚC KHI LÀM: đọc CLAUDE.md");
    let nhat_ky = format!("{}\n{}\n", dong_hook(), dong_user(&de_bai));
    assert!(de_bai_da_vao_nhat_ky(&nhat_ky, &de_bai));
}

#[test]
fn a_journal_that_only_has_startup_lines_is_not_received() {
    // ĐỐI CHỨNG NGƯỢC: đúng cảnh 09:14:47Z — hook xong, câu chưa tới.
    let nhat_ky = format!("{}\n", dong_hook());
    assert!(!de_bai_da_vao_nhat_ky(&nhat_ky, DE_BAI));
}

#[test]
fn another_user_turn_is_not_the_task() {
    let nhat_ky = format!("{}\n", dong_user("một câu hoàn toàn khác của chủ máy"));
    assert!(!de_bai_da_vao_nhat_ky(&nhat_ky, DE_BAI));
}

#[test]
fn the_task_quoted_by_a_non_input_line_is_not_received() {
    // Chữ đề bài nằm trong dòng `assistant` (phiên nhắc lại) — chưa phải lượt nhập.
    let dong = serde_json::json!({
        "type": "assistant",
        "message": { "role": "assistant", "content": [{ "type": "text", "text": DE_BAI }] },
    })
    .to_string();
    assert!(!de_bai_da_vao_nhat_ky(&format!("{dong}\n"), DE_BAI));
}

#[test]
fn a_queued_turn_counts_as_received() {
    let dong = serde_json::json!({
        "type": "queue-operation",
        "operation": "enqueue",
        "content": DE_BAI,
    })
    .to_string();
    assert!(de_bai_da_vao_nhat_ky(&format!("{dong}\n"), DE_BAI));
}

#[test]
fn text_parts_in_array_content_count() {
    let dong = serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": DE_BAI }] },
    })
    .to_string();
    assert!(de_bai_da_vao_nhat_ky(&format!("{dong}\n"), DE_BAI));
}

#[test]
fn an_empty_task_is_never_received() {
    // Đề bài rỗng không có dấu vân tay — không được ra `true` tình cờ.
    let nhat_ky = format!("{}\n", dong_user("bất kỳ"));
    assert!(!de_bai_da_vao_nhat_ky(&nhat_ky, ""));
    assert!(!de_bai_da_vao_nhat_ky(&nhat_ky, "   \n "));
}
