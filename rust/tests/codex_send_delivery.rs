//! Codex nhận kết quả huba qua user message, không qua schema Claude.
use huba::pipeline::khoi_da_vao_nhat_ky;

const CMD: &str = "python3 -c 'print(\"HUBA_SEND_PROBE\")'";
fn block() -> String {
    format!("[huba chạy hộ]\n$ {CMD}\n✅ xong\nHUBA_SEND_PROBE")
}

#[test]
fn rollout_codex_xac_nhan_khoi_user_da_nhan() {
    let v = serde_json::json!({"type":"response_item","payload":{"type":"message","role":"user",
        "content":[{"type":"input_text","text":block()}]}});
    assert!(khoi_da_vao_nhat_ky(&v.to_string(), CMD));
}

#[test]
fn item_user_codex_cung_la_bang_chung_nhan() {
    let v = serde_json::json!({"type":"event_msg","payload":{"type":"item_completed",
        "item":{"type":"UserMessage","content":[{"type":"text","text":block()}]}}});
    assert!(khoi_da_vao_nhat_ky(&v.to_string(), CMD));
}

#[test]
fn loi_assistant_va_tool_nhac_khoi_khong_phai_nhan() {
    for payload in [
        serde_json::json!({"type":"message","role":"assistant","content":[{"type":"output_text","text":block()}]}),
        serde_json::json!({"type":"function_call_output","output":block()}),
        serde_json::json!({"type":"agent_message","author":"/root/scout","content":[{"text":block()}]}),
    ] {
        let v = serde_json::json!({"type":"response_item","payload":payload});
        assert!(!khoi_da_vao_nhat_ky(&v.to_string(), CMD));
    }
}

#[test]
fn codex_ui_that_bo_enter_dau_phai_thay_chu_con_trong_o() {
    // Capture UI thật: .tmp/probe-codex-missed-enter-baseline.log. Cố tình chỉ
    // paste, chưa Enter; baseline tự Enter dọn sau chụp, journal nhận đúng 1.
    let screen = include_str!("fixtures/codex-composer-unsent-20261115.txt");
    let typed = "[huba chạy hộ · tự kiểm kênh gửi]\n$ python3 -c 'print(\"HUBA_INPUT_SEND_BASELINE_1794702424\")'\n✅ xong\nHUBA_INPUT_SEND_BASELINE_1794702424\nBản tin tự kiểm kênh gửi.";
    assert!(huba::keys::still_in_box(screen, typed));
    assert!(huba::keys::input_box_text(screen)
        .unwrap()
        .contains("HUBA_INPUT_SEND_BASELINE"));
}

#[test]
fn composer_codex_rong_co_chu_khac_va_khong_do_duoc_la_ba_trang_thai() {
    use huba::keys::{delivery_on_screen, Delivered};
    let frame = |s: &str| {
        format!("› {s}\n\n GPT-6.1-Sol high · /workspace · Main [default]\n ← for agents · ? for shortcuts\n")
    };
    assert_eq!(
        delivery_on_screen(&frame("Ask Codex to do anything"), &block(), true),
        Delivered::Gone
    );
    assert_eq!(
        delivery_on_screen(&frame(""), &block(), true),
        Delivered::Gone
    );
    assert!(matches!(
        delivery_on_screen(&frame("bản nháp của chủ máy"), &block(), true),
        Delivered::Unverified(_)
    ));
    assert!(matches!(
        delivery_on_screen("› chữ trong hội thoại\n", &block(), true),
        Delivered::Unverified(_)
    ));
    // Shell echo không phải ô nhập, vẫn giữ hành vi shell (không Enter thừa).
    assert_eq!(
        delivery_on_screen("$ echo x\nx\n$ ", "echo x", false),
        Delivered::Gone
    );
}

#[test]
fn doi_chieu_nhat_ky_chi_nhan_dung_lenh_khong_nhan_loi_khac() {
    let v = serde_json::json!({"type":"response_item","payload":{"type":"message","role":"user",
        "content":[{"type":"input_text","text":block()}]}})
    .to_string();
    assert!(!khoi_da_vao_nhat_ky(&v, "echo lenh-khac"));
    assert!(!khoi_da_vao_nhat_ky("", CMD));
    assert!(huba::pipeline::cau_da_vao_nhat_ky(&v, &block()));
    assert!(!huba::pipeline::cau_da_vao_nhat_ky(&v, "HUBA_SEND_PROBE"));
}

#[test]
fn nhan_paste_codex_chi_khop_so_ky_tu_cua_dung_khoi() {
    use huba::keys::{delivery_on_screen, Delivered};
    // Khuôn capture thật của biến thể 2923 chars (2946 bytes), 15/11.
    let text = format!("[huba chạy hộ]\n{}", "kết quả ".repeat(400));
    let frame = |n| {
        format!("› [Pasted Content {n} chars]\n\n GPT-6.1-Sol high · /workspace · Main [default]\n tab to queue message\n")
    };
    assert_eq!(
        delivery_on_screen(&frame(text.chars().count()), &text, true),
        Delivered::StillInBox
    );
    assert!(matches!(
        delivery_on_screen(&frame(text.len()), &text, true),
        Delivered::Unverified(_)
    ));
    assert!(matches!(
        delivery_on_screen(&frame(99), &text, true),
        Delivered::Unverified(_)
    ));
}

#[test]
fn khong_enter_ho_khoi_khac_hoac_ban_nhap_noi_them() {
    use huba::keys::{delivery_on_screen, Delivered};
    let typed = block();
    let frame = |s: &str| {
        format!("› {s}\n\n GPT-6.1-Sol high · /workspace · Main [default]\n tab to queue message\n")
    };
    assert_eq!(
        delivery_on_screen(&frame(&typed), &typed, true),
        Delivered::StillInBox
    );
    for other in [
        format!("{typed}\nBẢN NHÁP CHỦ MÁY"),
        format!("{typed}{typed}"),
        typed.replace("HUBA_SEND_PROBE", "LENH_KHAC_CHUNG_TIEN_TO"),
    ] {
        assert!(
            matches!(
                delivery_on_screen(&frame(&other), &typed, true),
                Delivered::Unverified(_)
            ),
            "không được Enter hộ: {other}"
        );
    }
}
