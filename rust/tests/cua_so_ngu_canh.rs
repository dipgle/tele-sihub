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

use std::path::Path;

use huba::sessions::context_window;

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
