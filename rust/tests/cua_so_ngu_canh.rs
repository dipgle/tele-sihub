//! Cửa sổ ngữ cảnh: lấy SỐ CỦA CLI, không lấy số huba tự đoán.
//!
//! Vì sao (Quang 2026-11-07: *"chạy mãi vẫn <20% đã thấy báo auto-compact"*):
//! huba tính `%` ngữ cảnh bằng `context_tokens / cửa_sổ`, và cửa sổ ấy là một
//! hằng số ĐOÁN trong mã — "haiku 200k, còn lại 1M". Model chạy qua máy chủ
//! ngoài (`deepseek-v4.1-flash`, acc8/acc9) KHÔNG có cửa sổ 1M.
//!
//! Hai phép đo, cả hai đọc từ máy này, không suy từ tài liệu:
//!  ① CLI tự khai trong nhật ký debug của chính nó (`~/.claude-acc9/debug/*.txt`):
//!     `autocompact: … effectiveWindow=180000` + `thresholdSource=unknown-model`.
//!     Model lạ với CLI ⇒ CLI giả định **180 000**, không phải 1 000 000.
//!  ② Nhật ký phiên ghi lại 81 lượt nén tự động, `preTokens` nằm trong
//!     165 754–172 767 — đúng khoảng 93% của 180 000. Với 1 000 000 thì chính
//!     những lượt ấy đọc ra 16–17%, tức "còn thênh thang" trong khi CLI đã nén.
//!
//! Và một phép đo cho vế ngược lại, để bản vá không kéo cả Claude thật xuống
//! 180k: một phiên `claude-opus-5-5` chạy tới **888 197** token mà chưa có mốc
//! nén nào ⇒ cửa sổ của model Claude thật lớn hơn 888k; giữ 1 000 000.
//!
//! ⚠ MẪU SỐ: bài cuối tệp quét MÃ NGUỒN, và phép quét mã nguồn rất dễ xanh oan
//! (đường dẫn sai ⟹ 0 dòng khớp ⟹ "sạch"). Nên nó chứng minh đã đọc đúng tệp
//! trước khi kết luận, và có đối chứng ngược: một đoạn mã cấy sẵn chỗ đoán cửa
//! sổ phải bị đếm ra.
//!
//! 🔴 **Phép đo thứ ba (07/11 tối, Quang: *"xem trực tiếp thì thấp mà ở danh
//! sách session thì lại 89%"*)** — chính CLI dùng HAI con số: nhật ký debug của
//! nó ghi `effectiveWindow=180000`, còn `/context` của nó in *"Auto-compact
//! window: 200k tokens (default for an unrecognized model)"*. Đo trên phiên
//! `585e8e6b`: `/context` lúc 15:19:29Z in **89 400** token / 200k = **45%**,
//! lượt gọi cuối 15:17:10Z = **89 395** — lệch 5 token, tức phép ĐỌC của huba
//! đúng, chỉ mẫu số khác. Điện thoại phải in số của `/context`
//! (`display_window`); cửa nén thật (`context_window`) giữ cho QUYẾT ĐỊNH.

use std::path::Path;

use huba::sessions::{context_pct, context_window, display_window};

#[test]
fn model_la_thi_theo_so_cli_tu_khai_180k() {
    assert_eq!(
        context_window(Some("deepseek-v4.1-flash")),
        180_000,
        "model lạ với CLI ⇒ CLI giả định 180 000 (effectiveWindow đo được), không phải 1 000 000"
    );
    // Không đọc được model từ nhật ký thì cũng rơi về đúng con số ấy — thà báo
    // sớm còn hơn báo muộn; 180k KHÔNG làm huba bàn giao sớm hơn thực tế.
    assert_eq!(context_window(None), 180_000);
}

#[test]
fn claude_that_giu_cua_so_lon_con_haiku_200k() {
    assert_eq!(context_window(Some("claude-opus-5-5")), 1_000_000);
    assert_eq!(context_window(Some("claude-sonnet-5")), 1_000_000);
    assert_eq!(context_window(Some("claude-haiku-4-5-20251001")), 200_000);
}

/// 🔴 Số trên ĐIỆN THOẠI phải trùng số `/context` của phiên — ca thật, đo được.
///
/// Phiên `585e8e6b`, 07/11: `/context` in `89.4k/200k (45%)`, lượt gọi cuối
/// 89 395 token. Điện thoại in 45%, KHÔNG phải 50%.
#[test]
fn so_hien_thi_khop_con_so_cli_in() {
    let model = Some("deepseek-v4.1-flash");
    assert_eq!(
        display_window(model),
        200_000,
        "CLI in «Auto-compact window: 200k tokens (default for an unrecognized model)» — \
         mẫu số hiển thị của nó là 200k"
    );
    assert_eq!(
        context_pct(89_395, display_window(model)),
        45,
        "phải trùng con số CLI in (45%), không lệch một bậc"
    );
    // ĐỐI CHỨNG NGƯỢC: mẫu số của QUYẾT ĐỊNH cho ra 50% — tức bài này ĐỎ được
    // nếu ai nối nhầm hai mẫu số vào nhau.
    assert_eq!(context_pct(89_395, context_window(model)), 50);
    // Và công thức là LÀM TRÒN, không cắt cụt: 44,7% ⇒ 45%, cắt cụt ra 44%.
    assert_eq!(context_pct(89_395, 200_000), 45);
    assert_eq!(context_pct(89_400, 200_000), 45);
    assert_eq!(
        context_pct(0, 200_000),
        0,
        "phiên chưa có lượt nào ⇒ 0, không phải 100"
    );
    assert_eq!(
        context_pct(500_000, 200_000),
        100,
        "kẹp 100 — không in 250%"
    );
}

/// Model Claude thật / haiku: HAI mẫu số bằng nhau, nên hiển thị không đổi so
/// với trước bản vá — bản vá chỉ đụng model lạ (đường acc8/acc9).
#[test]
fn model_claude_va_haiku_hai_mau_so_bang_nhau() {
    for m in [
        Some("claude-opus-5-5"),
        Some("claude-sonnet-5"),
        Some("claude-haiku-4-5-20251001"),
        None,
    ] {
        if m.is_some_and(|v| v.contains("claude") || v.contains("haiku")) {
            assert_eq!(
                context_window(m),
                display_window(m),
                "model đã biết: một mẫu số"
            );
        }
    }
    // Không đọc được model = model lạ ⇒ hai mẫu số KHÁC nhau, có chủ ý.
    assert_eq!(display_window(None), 200_000);
    assert_eq!(context_window(None), 180_000);
}

/// Đếm số chỗ TỰ ĐOÁN cửa sổ trong một đoạn mã nguồn.
///
/// Tách thành hàm thuần để ĐỐI CHỨNG NGƯỢC chạy được ở bài cuối: một hàm luôn
/// trả 0 cũng làm cả tệp này xanh, nên nó phải chứng minh đếm được.
fn dem_cho_tu_doan(text: &str) -> usize {
    text.matches("contains(\"haiku\")").count()
}

#[test]
fn khong_con_cho_nao_tu_doan_cua_so() {
    let goc = Path::new(env!("CARGO_MANIFEST_DIR"));
    // (tệp, số chỗ ĐOÁN được phép) — con số của `sessions.rs` là CHÍNH hàm
    // `context_window`, tức nơi duy nhất được phép có bảng; `pipeline.rs` là
    // không chỗ nào, vì ở đó chỉ được GỌI.
    let cho_phep = [("src/pipeline.rs", 0usize), ("src/sessions.rs", 1usize)];
    let mut da_doc = 0usize;
    for (ten, duoc_phep) in cho_phep {
        let duong = goc.join(ten);
        let src = std::fs::read_to_string(&duong)
            .unwrap_or_else(|e| panic!("KHÔNG ĐO ĐƯỢC: không đọc nổi {}: {e}", duong.display()));
        // Mẫu số: tệp thật, đủ lớn, đúng tệp. Thiếu mốc nào thì ĐỎ, không phải xanh.
        assert!(
            src.len() > 50_000,
            "KHÔNG ĐO ĐƯỢC: {} chỉ {} byte — sai tệp",
            duong.display(),
            src.len()
        );
        assert_eq!(
            dem_cho_tu_doan(&src),
            duoc_phep,
            "số chỗ TỰ ĐOÁN cửa sổ ngữ cảnh trong {ten} sai — mọi chỗ phải đi qua \
             sessions::context_window để ra CÙNG một con số"
        );
        da_doc += 1;
    }
    assert_eq!(da_doc, 2, "KHÔNG ĐO ĐƯỢC: chỉ đọc {da_doc}/2 tệp");
}

/// Đối chứng ngược: bộ đếm phải ĐỎ được với mã có cấy chỗ đoán cửa sổ.
#[test]
fn doi_chung_nguoc_bo_dem_do_duoc() {
    let co_cay =
        r#"let window: u64 = if s.model.as_deref().is_some_and(|m| m.contains("haiku")) {"#;
    assert_eq!(
        dem_cho_tu_doan(co_cay),
        1,
        "bộ đếm không thấy chỗ cấy ⟹ cổng MÙ"
    );
    assert_eq!(
        dem_cho_tu_doan("let window = crate::sessions::context_window(model);"),
        0
    );
}
