//! Quang 13/11 (ảnh `/close`): ba tin cho một việc — `🔒 Đóng hẳn…? ✅ Đã xác nhận`
//! · `⏳ Đã gửi lệnh thoát…` · `⏹ … đã thoát` — *"chỉnh cho gọn lại thành 1 tin
//! thôi"*. Câu trả lời nối vào tin hỏi; chặng «đã thoát» (vòng sau) sửa tiếp nhờ
//! sổ chờ đóng mang `m`/`mt`.

use huba::pipeline::{chu_noi, Closing};

#[test]
fn so_cho_dong_cu_khong_co_tin_song_van_doc_duoc() {
    // Bản ghi ghi trước 13/11: không có `m`/`mt` ⟹ 0 / rỗng ⟹ gửi tin riêng như cũ.
    let cu = r#"{"w":7,"n":"[projects]","t":100,"c":0,"h":0,"r":0,"x":100}"#;
    let c: Closing = serde_json::from_str(cu).expect("sổ cũ");
    assert_eq!(c.m, 0);
    assert!(c.mt.is_empty());
}

#[test]
fn ba_chang_thanh_mot_tin() {
    let hoi = "🔒 Đóng hẳn phiên [projects] (acc8)?\n\n✅ Đã xác nhận";
    let sau_ack = chu_noi(
        hoi,
        "⏳ Đã gửi Ctrl+C — chờ CLI xong lượt dở rồi đóng cửa sổ (kiểm 30 giây/lần).",
    );
    let cuoi = chu_noi(&sau_ack, "⏹ [projects] đã thoát — cửa sổ đã đóng (chờ 2s).");
    assert_eq!(cuoi.lines().filter(|l| l.starts_with('🔒')).count(), 1);
    assert!(cuoi.contains("✅ Đã xác nhận\n⏳"), "{cuoi}");
    assert!(
        cuoi.ends_with("⏹ [projects] đã thoát — cửa sổ đã đóng (chờ 2s)."),
        "{cuoi}"
    );
    // Dòng tiến độ lượt trước («⏳ chưa đóng được…») được THAY: chặng sau nối vào
    // `mt` (chữ tới hết dòng ⏳ đầu), không nối vào chữ đang hiện.
    let giua = chu_noi(&sau_ack, "⏳ [projects] chưa đóng được sau 2 phút — …");
    let cuoi2 = chu_noi(
        &sau_ack,
        "⏹ [projects] đã thoát — cửa sổ đã đóng (chờ 130s).",
    );
    assert!(giua.contains("chưa đóng được"));
    assert!(!cuoi2.contains("chưa đóng được"), "{cuoi2}");
}
