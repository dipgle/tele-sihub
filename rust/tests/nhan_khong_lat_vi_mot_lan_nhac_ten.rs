//! Một phiên NHẮC tên dự án khác thì không được đổi tên theo.
//!
//! 🔴 Hà 2026-09-16, ảnh danh sách Telegram: *"Phiên main chạy một hồi nhắc đến
//! huba thì tên bị nhảy sang luôn, không hiểu đang bắt kiểu gì, như thế này làm
//! sao tôi biết thế nào để quản lý"*. Hàng `1249d1ef` — việc của nó đúng là
//! *"Fix main branch session disconnection"* của **dwork** — hiện ra `[huba]`.
//!
//! Đo trên chính nhật ký ấy (256 KB đuôi, 18 lượt `assistant`), dòng kích hoạt
//! là **lượt nói của chính phiên**, lúc 2026-09-16T07:07:18.882Z:
//!
//! ```text
//! `[huba]` xác nhận chẩn đoán mới (11/162 = 6,8 %, cùng dân số) — và **bắt được…
//! ```
//!
//! Hai phiên đang nhắn cho nhau, nên phiên dwork **nói VỀ** huba — còn bộ dò đọc
//! thành *"tôi LÀ huba"*. Hai cổng cũ đều không đỡ được: chuỗi ấy nằm ở ĐẦU lượt
//! nói, và `huba` là thư mục CÓ THẬT.
//!
//! **Và nhãn sai KHÔNG TỰ LÀNH**: lời khai được ghi vào sổ nhớ, rồi từ đó luôn
//! thắng phép đếm đường dẫn (`recall_declared` đứng trên `counted`).
//!
//! ⚠ HAI ĐƯỜNG SỬA HIỂN NHIÊN ĐÃ BỊ ĐO RA LÀ SAI — đừng thử lại:
//! · *"lấy nhãn xuất hiện nhiều nhất"*: trong 256 KB đuôi ấy có **đúng một** dòng
//!   tự khai, và nó là dòng sai. Đa số của một tập một phần tử vẫn là phần tử ấy.
//! · *"đối chiếu sổ `scripts/.session-bind`"*: phiên ấy **không có** bản ghi nào
//!   — sổ ghi theo lượt GHI TỆP qua hook, mà nó đang chỉ đọc.
//!
//! Cổng thật nằm ở SỐ LẦN: luật workspace bắt phiên tự khai ở **MỌI** lượt nói,
//! nên một cái tên xuất hiện đúng một lần trong cả cửa sổ đuôi là tên được NHẮC.

fn workspace() -> String {
    let home = std::env::var("HOME").expect("HOME");
    format!("{home}/projects")
}

/// Dựng đuôi nhật ký từ các lượt nói của phiên, theo đúng hình dạng thật.
fn duoi(luot: &[&str]) -> String {
    luot.iter()
        .map(|t| {
            serde_json::json!({
                "type": "assistant",
                "message": {"role": "assistant", "content": [{"type": "text", "text": t}]}
            })
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Nguyên văn dòng đã gây lỗi, cắt ngắn.
const NHAC_TEN_HUBA: &str =
    "`[huba]` xác nhận chẩn đoán mới (11/162 = 6,8 %, cùng dân số) — và bắt được một lỗi mẫu số";

/// CA CỦA HÀ: phiên đã mang nhãn `dwork`, rồi nhắc `huba` ĐÚNG MỘT LẦN.
#[test]
fn mot_lan_nhac_ten_khong_lat_duoc_nhan_dang_giu() {
    let ws = workspace();
    let sid = "test-nhac-ten-1249d1ef";

    // ① Phiên tự khai `dwork` như thường lệ ⟹ nhãn vào sổ.
    let truoc = duoi(&[
        "[dwork] Đang đo lại chồng lấn từng cặp bàn giao.",
        "[dwork] Xong, p50 125 giây.",
    ]);
    assert_eq!(
        huba::sessions::folder_for_session(sid, &truoc, &ws, "").as_deref(),
        Some("dwork"),
        "phiên tự khai dwork mà không nhận được nhãn"
    );

    // ② Rồi nó NHẮC tên huba đúng một lần, ở đầu lượt nói.
    let sau = duoi(&[
        "Đã đo lại: 11/162 lượt bắn không thấy mốc mở cửa sổ.",
        NHAC_TEN_HUBA,
        "Tiếp tục dựng dòng thời gian cho lượt 18:45.",
    ]);
    assert_eq!(
        huba::sessions::folder_for_session(sid, &sau, &ws, "").as_deref(),
        Some("dwork"),
        "một lần NHẮC tên dự án khác đã lật được nhãn — đúng cái Hà chụp màn 16/09"
    );
}

/// 🔄 ĐẢO CHIỀU 2026-11-23 22:2x — Hà chốt qua hộp chọn trên Telegram:
/// **tên chốt MỘT LẦN**, không đổi giữa đời phiên (*"sau thì lấy làm gì"*).
///
/// 🪦 Luật cũ (giữ nguyên văn): *"Chiều ngược: phiên THẬT SỰ đổi dự án thì vẫn
/// phải đổi được nhãn. Cổng chống lật mà chặn luôn cả lượt đổi thật thì nó
/// không phải cổng, nó là một cái nhãn đông cứng."* — hết hiệu lực: cái giá của
/// nó là đọc nhật ký mọi vòng để có thể đổi một cái tên Hà nói không cần đổi.
#[test]
fn ten_da_chot_khong_doi_du_khai_hai_lan() {
    let ws = workspace();
    let sid = "test-doi-that-9c44d3fe";

    let truoc = duoi(&["[dwork] Đang làm dwork.", "[dwork] Vẫn dwork."]);
    assert_eq!(
        huba::sessions::folder_for_session(sid, &truoc, &ws, "").as_deref(),
        Some("dwork")
    );

    let sau = duoi(&[
        "[huba] Chuyển sang huba, đang đọc quota.rs.",
        "[huba] Đã vá xong, chạy bài kiểm.",
    ]);
    assert_eq!(
        huba::sessions::folder_for_session(sid, &sau, &ws, "").as_deref(),
        Some("dwork"),
        "tên đã chốt thì các lượt sau KHÔNG được đổi (Hà 23/11 22:2x)"
    );
}
/// 🔄 ĐẢO CHIỀU 2026-11-20, không xoá: phiên CHƯA có nhãn nào thì MỘT lời khai
/// KHÔNG còn đủ để gieo — trừ khi nó khớp scope `cwd`.
///
/// Bản trước khoá điều ngược lại (*"phiên mới mà phải đợi tới lượt nói thứ hai
/// mới có tên thì cổng đang chặn nhầm"*) và đổi một cái nhãn sai lấy một cái nhãn
/// trống là "đổi ngang". Ca 16/11 chứng minh nó không ngang: một tiền tố
/// assistant tự viết không phải bằng chứng chủ máy đã giao dự án, và nhãn sai
/// đứng im thay vì trống. Vế PHẢI GIỮ: hai lượt nhất quán, hoặc một lượt khớp
/// scope, vẫn gieo được ngay.
#[test]
fn phien_chua_co_nhan_thi_mot_lan_khai_chua_du_de_gieo() {
    let ws = workspace();
    let tail = duoi(&["[huba] Lượt nói đầu tiên của phiên này."]);
    assert_eq!(
        huba::sessions::folder_for_session("test-gieo-moi-aa11bb22", &tail, &ws, ""),
        None,
        "một tiền tố assistant đơn lẻ đã gieo được tên ở phiên mở tại gốc"
    );
    // Vế phải giữ ①: khớp scope (CLI mở phiên trong `huba`) ⇒ gieo ngay.
    assert_eq!(
        huba::sessions::folder_for_session("test-gieo-scope-aa11bb22", &tail, &ws, "huba")
            .as_deref(),
        Some("huba"),
        "tiền tố khớp scope cwd phải đủ để gieo"
    );
    // Vế phải giữ ②: hai lượt nhất quán ⇒ gieo dù không có scope.
    let hai = duoi(&["[huba] Lượt một.", "[huba] Lượt hai."]);
    assert_eq!(
        huba::sessions::folder_for_session("test-gieo-hai-aa11bb22", &hai, &ws, "").as_deref(),
        Some("huba")
    );
}

/// Lượt `user` KHÔNG phải lời khai — luật cũ, giữ nguyên, khoá lại vì bản vá
/// này đụng đúng vòng lặp ấy.
#[test]
fn loi_chu_may_go_khong_phai_loi_tu_khai() {
    let ws = workspace();
    let sid = "test-user-noi-cc33dd44";
    let tail = format!(
        "{}\n{}",
        serde_json::json!({
            "type": "user",
            "message": {"role": "user", "content": [{"type": "text", "text": "[huba] xem lại hộ tôi"}]}
        }),
        duoi(&["[dwork] Đang làm dwork."])
    );
    assert_eq!(
        huba::sessions::folder_for_session(sid, &tail, &ws, "dwork").as_deref(),
        Some("dwork"),
        "câu chủ máy gõ nhắc tên dự án khác là chuyện thường — không được tính là lời khai"
    );
}
