//! Tên phiên phải đọc được dù TIN NHẮN ĐẦU rất to.
//!
//! 🔴 Hà 24/11 (ảnh `/session`): phiên huba `103e5837` hiện `projects-1c` thay vì
//! `[huba]` — *"đây là phiên huba mà sao lại không hiện đúng được"*.
//!
//! ĐO trên chính nhật ký ấy: tin nhắn đầu (luật workspace + bản bàn giao nạp vào) dài
//! ~220 KB, nên câu trả lời đầu tiên mang `[huba]` nằm ở **byte 226 469**. `read_head`
//! cũ đọc **64 KiB đầu theo BYTE** ⟹ chỉ thấy MỘT dòng user bị cắt cụt, không dòng
//! `assistant` nào ⟹ không có lời khai ⟹ nhãn trung tính `projects-<mã>`, ghim luôn.
//!
//! Sửa: «vài lượt đầu» tính theo DÒNG, không theo byte — dòng nào to hơn một lượt nói
//! bình thường (không phân tích được thành JSON khi bị cắt) thì bỏ qua, không ăn ngân sách.
//!
//! Ba phép đo: ① ca của Hà (RED trên mã cũ) · ② đối chứng ngược: nhật ký nhỏ vẫn như cũ ·
//! ③ trần quét — một tệp một dòng khổng lồ không làm hubd đọc hết cả tệp.

use std::io::Write;

fn workspace() -> String {
    let home = std::env::var("HOME").expect("HOME");
    format!("{home}/projects")
}

fn dong_user(text: &str) -> String {
    serde_json::json!({
        "type": "user",
        "message": {"role": "user", "content": [{"type": "text", "text": text}]}
    })
    .to_string()
}

fn dong_assistant(text: &str) -> String {
    serde_json::json!({
        "type": "assistant",
        "message": {"role": "assistant", "content": [{"type": "text", "text": text}]}
    })
    .to_string()
}

fn ghi(ten: &str, dong: &[String]) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("huba-ten-tin-dau-lon-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(ten);
    let mut f = std::fs::File::create(&path).unwrap();
    for d in dong {
        writeln!(f, "{d}").unwrap();
    }
    path
}

fn luot_huba() -> Vec<String> {
    vec![
        dong_assistant("[huba] Đang vào phiên theo luật: đọc CLAUDE.md của dự án."),
        dong_assistant("[huba] Đã quét: gate xanh, việc kế là đo lại tên phiên."),
        dong_assistant("[huba] Đang làm tiếp việc kế."),
    ]
}

/// ① CA CỦA HÀ — tin nhắn đầu 220 KB, tên chỉ xuất hiện SAU nó.
#[test]
fn tin_nhan_dau_220kb_khong_che_mat_ten_phien() {
    // Hình dạng THẬT đo trên nhật ký 103e5837: tin user nhỏ, rồi attachment 78 KB và hai
    // api-request 41 + 43 KB (đều KHÔNG phải hội thoại), rồi mới tới lượt nói đầu tiên.
    let to = |kieu: &str, n: usize| {
        serde_json::json!({"type": kieu, "payload": "x".repeat(n)}).to_string()
    };
    let mut dong = vec![
        dong_user("mở phiên huba"),
        to("attachment", 78_000),
        to("api-request-shape", 41_500),
        to("api-request-blob", 43_500),
        dong_user(&"luật workspace nạp vào phiên. ".repeat(7_500)), // một tin user ~225 KB
    ];
    dong.extend(luot_huba());
    let path = ghi("ca-cua-ha.jsonl", &dong);
    assert!(
        std::fs::metadata(&path).unwrap().len() > 200 * 1024,
        "dữ kiện của ca: tệp phải lớn hơn 200 KB"
    );

    let head = huba::sessions::read_head(&path).expect("đọc head");
    let ten =
        huba::sessions::folder_for_session("test-tin-dau-lon-103e5837", &head, &workspace(), "");
    assert_eq!(
        ten.as_deref(),
        Some("huba"),
        "tin nhắn đầu to làm mất tên: head chỉ có {} byte",
        head.len()
    );
}

/// ② ĐỐI CHỨNG NGƯỢC — nhật ký nhỏ (ca bình thường) vẫn ra đúng tên như trước.
#[test]
fn nhat_ky_nho_van_ra_ten_nhu_cu() {
    let mut dong = vec![dong_user("mở phiên huba")];
    dong.extend(luot_huba());
    let path = ghi("nho.jsonl", &dong);
    let head = huba::sessions::read_head(&path).expect("đọc head");
    let ten = huba::sessions::folder_for_session("test-tin-dau-nho-aaaa", &head, &workspace(), "");
    assert_eq!(ten.as_deref(), Some("huba"));
}

/// ③ TRẦN QUÉT — một dòng khổng lồ không dòng mới không bắt hubd đọc hết cả tệp, và head
/// không phình vượt ngân sách (mỗi vòng ~10 s × mọi phiên).
#[test]
fn mot_dong_khong_lo_khong_lam_head_phinh_hay_doc_het_tep() {
    let path = ghi(
        "khong-lo.jsonl",
        &[dong_user(&"x".repeat(40 * 1024 * 1024))],
    );
    let head = huba::sessions::read_head(&path).expect("đọc head");
    assert!(
        head.len() <= 256 * 1024,
        "head phình {} byte — vượt ngân sách",
        head.len()
    );
}
