//! Tin "BẢNG ĐÃ ĐỦ" phải có ĐÍCH CHẠM Submit, không chỉ câu mời gõ hai bước.
//!
//! 🔴 Quang 2026-11-08 10:46, ảnh buồng chat + *"Bấm hết rồi nhưng lại thiếu
//! submit"*: trả lời xong cả ba câu, huba gửi đúng một dòng chữ
//! `Bảng ĐÃ ĐỦ — bấm /send_6b3ef665 để gửi`, và đường gửi thật (sang tab
//! `Submit` rồi Enter — `keys::submit_bare_keys`) không có gì để chạm vào.
//!
//! Bài kiểm này đo phần RENDER: `submit: true` + một dòng `Submit` trong chữ ⟹
//! neo `✅` trỏ `/send_<sid>` bám đúng dòng ấy. Đối chứng ngược nằm ngay sau:
//! `submit: false` thì KHÔNG được có đích chạm nào — nếu cờ không có tác dụng
//! thì bài đầu xanh cũng vô nghĩa.
//!
//! ⚠ Phần KHÔNG chứng minh được ở đây, khai thẳng: `pick_answer` đặt cờ ấy chỉ
//! khi số câu trống về 0 — việc ấy cần một màn Terminal thật, nên nó thuộc phép
//! nghiệm thu trên máy thật, không phải bài kiểm đơn vị này.

use huba::pipeline::{render_session_data, SessionData};

/// Nguyên văn hình dạng tin huba gửi sau câu trả lời cuối.
const ACK: &str = "✅ [huba] · câu 3 (Câu 3) → chọn 2. Bảng ĐÃ ĐỦ — chạm để gửi:\nSubmit";

fn data(submit: bool) -> SessionData {
    SessionData {
        sid: "6b3ef665-d46e-4033-85f5-7386351d0e56".into(),
        submit,
        ..Default::default()
    }
}

#[test]
fn tin_bao_bang_da_du_mang_dich_cham_gui() {
    huba::telegram::set_bot_username("ai_agents_bot");
    let out = render_session_data(ACK, &data(true));
    let dong = out
        .lines()
        .find(|l| l.contains("start=send_6b3ef665"))
        .unwrap_or_else(|| panic!("tin BẢNG ĐÃ ĐỦ không có đích chạm Submit:\n{out}"));
    assert!(
        dong.contains("Submit"),
        "đích chạm gửi bám nhầm dòng: «{}» — phải bám dòng «Submit»",
        dong.trim()
    );
}

/// ĐỐI CHỨNG NGƯỢC: không có cờ thì không có đích chạm — nếu bài này cũng xanh
/// thì bài trên không đo gì cả (neo mọc ra vô điều kiện).
#[test]
fn doi_chung_khong_co_co_thi_khong_co_dich_cham() {
    huba::telegram::set_bot_username("ai_agents_bot");
    let out = render_session_data(ACK, &data(false));
    assert!(
        !out.contains("start=send_6b3ef665"),
        "neo Submit mọc ra dù cờ tắt — phép đo mù:\n{out}"
    );
}
