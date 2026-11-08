//! Nhãn lựa chọn bị NHẮC trong câu hỏi thì không được cướp `☑` của dòng lựa chọn.
//!
//! 🔴 Quang 2026-11-08 10:46, ảnh buồng chat + *"Tab 2 Mất lựa chọn 2"*: bảng ba
//! câu, tab 2 có hai lựa chọn mà dòng thứ hai không có `☑` bấm được; thay vào đó
//! một `☑` lạc hiện ở ĐẦU DÒNG CÂU HỎI phía trên.
//!
//! Gốc, đo trên chính tin ấy: câu hỏi của tôi viết nguyên văn nhãn
//! `(Lựa chọn 2-2)`, và `line_carries` khớp bằng `line.contains(cmd)` — nên dòng
//! VĂN XUÔI khớp TRƯỚC dòng lựa chọn thật, `html_with_links_last` gắn neo vào
//! dòng đầu tiên khớp, và dòng lựa chọn ở dưới không còn gì để bấm.
//!
//! Đối chứng ngược nằm ngay trong tệp: nhãn KHÔNG bị nhắc trong câu hỏi thì vẫn
//! phải bám đúng dòng của nó (bài thứ ba) — sửa mà làm đỏ bài ấy là sửa hỏng.

use huba::pipeline::html_with_links;

/// Nguyên văn hình dạng tin thật: câu hỏi nhắc tên nhãn, rồi thanh tab, rồi
/// hai dòng lựa chọn của tab đang mở.
const TIN: &str = "\
❓ Đo giao diện hộp nhiều câu — CÂU 2: bấm DÒNG THỨ HAI (Lựa chọn 2-2) — phép đo chính.\n\
←  ☒ Câu 1  ☒ Câu 2  ☐ Câu 3  ✔ Submit  →\n\
❯ 1. Lựa chọn 2-1 \n\
     Lựa chọn thứ nhất của tab hai.\n\
  2. Lựa chọn 2-2 \n\
     Lựa chọn thứ hai của tab hai.\n\
  3. Type something.\n";

fn neo(nhan: &str, payload: &str) -> (String, Vec<(String, String)>) {
    (
        nhan.to_string(),
        vec![(
            format!("https://t.me/ai_agents_bot?start={payload}"),
            "\t☑".to_string(),
        )],
    )
}

/// Dòng nào trong `html` mang `payload` — trả nguyên dòng đó để bài kiểm nói
/// được nó bám NHẦM vào đâu, chứ không chỉ "False".
fn dong_mang(html: &str, payload: &str) -> Option<String> {
    html.lines()
        .find(|l| l.contains(payload))
        .map(|l| l.trim().to_string())
}

/// Bóc thẻ `<a …>` khỏi một dòng để so CHỮ, không so HTML — nhãn lựa chọn bị
/// bọc thẻ khi neo chiếm trọn dòng, nên `2. Lựa chọn 2-2` bị chẻ làm ba mẩu.
fn chu_cua_dong(dong: &str) -> String {
    let mut ra = String::new();
    let mut trong_the = false;
    for ch in dong.chars() {
        match ch {
            '<' => trong_the = true,
            '>' => trong_the = false,
            c if !trong_the => ra.push(c),
            _ => {}
        }
    }
    ra.trim().to_string()
}

/// 🔴 CA CỦA QUANG: nhãn bị nhắc trong câu hỏi ⇒ `☑` phải bám DÒNG LỰA CHỌN.
#[test]
fn nhan_bi_nhac_trong_cau_hoi_van_bam_dung_dong_lua_chon() {
    let anchors = vec![
        neo("Lựa chọn 2-1", "pick_6b3ef665_2_1"),
        neo("Lựa chọn 2-2", "pick_6b3ef665_2_2"),
    ];
    let (html, linked, unlinked) = html_with_links(TIN, &anchors);
    assert!(unlinked.is_empty(), "neo rơi mất: {unlinked:?}\n{html}");
    assert_eq!(linked, 2, "phải chèn đủ hai đích chạm:\n{html}");

    let dong = dong_mang(&html, "pick_6b3ef665_2_2").expect("không dòng nào mang ☑ của 2-2");
    let chu = chu_cua_dong(&dong);
    // Glyph `☑` được chèn vào chính dòng ấy (trước nhãn hoặc trước `2.`), nên
    // phép so là "đúng dòng lựa chọn", không phải "nguyên văn không thêm gì".
    assert!(
        chu.starts_with("2.") && chu.contains("Lựa chọn 2-2"),
        "☑ của lựa chọn 2-2 bám nhầm dòng: «{chu}» — phải bám dòng «2. Lựa chọn 2-2»"
    );
    assert!(
        !chu.contains("bấm DÒNG THỨ HAI"),
        "☑ bám vào CÂU HỎI (nó nhắc tên nhãn) thay vì dòng lựa chọn: «{chu}»"
    );
}

/// Và dòng CÂU HỎI không được mang đích chạm nào — nó là chữ, không phải lựa chọn.
#[test]
fn dong_cau_hoi_khong_mang_dich_cham() {
    let anchors = vec![
        neo("Lựa chọn 2-1", "pick_6b3ef665_2_1"),
        neo("Lựa chọn 2-2", "pick_6b3ef665_2_2"),
    ];
    let (html, _, _) = html_with_links(TIN, &anchors);
    let cau_hoi = html
        .lines()
        .find(|l| l.contains("bấm DÒNG THỨ HAI"))
        .expect("mất dòng câu hỏi");
    assert!(
        !cau_hoi.contains("t.me/"),
        "câu hỏi bị gắn đích chạm: «{}»",
        cau_hoi.trim()
    );
}

/// ĐỐI CHỨNG NGƯỢC — nhãn KHÔNG bị nhắc trong câu hỏi thì đường cũ vẫn đúng.
#[test]
fn doi_chung_nhan_khong_bi_nhac_van_bam_dung() {
    let tin = TIN.replace("(Lựa chọn 2-2)", "(ô thứ hai)");
    let anchors = vec![
        neo("Lựa chọn 2-1", "pick_6b3ef665_2_1"),
        neo("Lựa chọn 2-2", "pick_6b3ef665_2_2"),
    ];
    let (html, linked, unlinked) = html_with_links(&tin, &anchors);
    assert!(unlinked.is_empty(), "neo rơi: {unlinked:?}\n{html}");
    assert_eq!(linked, 2, "{html}");
    let dong = dong_mang(&html, "pick_6b3ef665_2_2").expect("không dòng nào mang 2-2");
    let chu = chu_cua_dong(&dong);
    assert!(
        chu.starts_with("2.") && chu.contains("Lựa chọn 2-2"),
        "{chu}"
    );
}
