//! Phiên KHAI TÊN MUỘN: đầu nhật ký không có `[dự-án]` nào, lời khai chỉ xuất hiện về cuối.
//!
//! 🔴 Hà 24/11 16:20 (ảnh `/session`): *"Chạy tới 50% rồi chưa ra tên"* — phiên `b978536d` hiện
//! `projects-ff`. ĐO trên nhật ký ấy: 46 lượt nói, **42 lượt đầu không có thẻ**, 4 lượt cuối mở bằng
//! `[dwork/dci]` (phiên được nhắc theo luật giữa chừng). Tên chỉ đọc ở ĐẦU nhật ký (Hà 23/11 «chỉ lấy
//! tên ở vài lượt đầu») và đầu im ⟹ không có tên, mãi mãi — dù đuôi có ba lời khai nhất quán.
//!
//! Sửa: đầu im thì hỏi tiếp ĐUÔI, với luật cũ của đuôi (hai lượt nhất quán mới là khai). Đầu CÓ tên
//! thì đầu thắng như trước — không mở lại cửa «tên nhảy theo việc hôm nay».

use std::io::Write;

fn workspace() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("huba-khai-muon-{}", std::process::id()));
    for d in ["alpha", "beta"] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }
    dir
}

fn noi(text: &str) -> String {
    serde_json::json!({
        "type": "assistant",
        "message": {"role": "assistant", "content": [{"type": "text", "text": text}]}
    })
    .to_string()
}

fn jsonl(dong: &[String]) -> String {
    let mut s = Vec::new();
    for d in dong {
        writeln!(s, "{d}").unwrap();
    }
    String::from_utf8(s).unwrap()
}

fn ten(id: &str, head: Option<&str>, tail: &str) -> Option<String> {
    let ws = workspace();
    huba::sessions::folder_for_head_or_tail(id, head, tail, ws.to_str().unwrap(), "")
}

fn dau_im() -> String {
    jsonl(&[
        noi("Đang đọc luật."),
        noi("Đã đọc xong, bắt đầu."),
        noi("Làm tiếp."),
    ])
}

/// ① CA CỦA HÀ — đầu im, đuôi có ba lời khai nhất quán.
#[test]
fn dau_im_thi_hoi_tiep_duoi() {
    let duoi = jsonl(&[
        noi("Việc giữa chừng."),
        noi("[alpha/dci] Đang chờ kiểm."),
        noi("Commit xong."),
        noi("[alpha/dci] Kiểm xanh."),
        noi("[alpha/dci] Đẩy được rồi."),
    ]);
    assert_eq!(
        ten("khai-muon-1", Some(&dau_im()), &duoi).as_deref(),
        Some("alpha")
    );
}

/// ② ĐỐI CHỨNG NGƯỢC — một lần trong đuôi vẫn chỉ là NHẮC (luật cũ của đuôi không nới).
#[test]
fn mot_lan_trong_duoi_van_chi_la_nhac() {
    let duoi = jsonl(&[
        noi("Việc."),
        noi("[alpha/dci] Một lần duy nhất."),
        noi("Xong."),
    ]);
    assert_eq!(ten("khai-muon-2", Some(&dau_im()), &duoi), None);
}

/// ③ Đầu CÓ tên ⟹ đầu thắng, đuôi nói dự án khác cũng không đổi được.
#[test]
fn dau_co_ten_thi_dau_thang() {
    let dau = jsonl(&[
        noi("[beta] Vào phiên."),
        noi("[beta] Đã đọc luật."),
        noi("[beta] Làm tiếp."),
    ]);
    let duoi = jsonl(&[
        noi("[alpha/dci] a"),
        noi("[alpha/dci] b"),
        noi("[alpha/dci] c"),
    ]);
    assert_eq!(
        ten("khai-muon-3", Some(&dau), &duoi).as_deref(),
        Some("beta")
    );
}

/// ④ Không đọc được đầu (`None`) ⟹ đi thẳng đuôi như cũ.
#[test]
fn khong_co_dau_thi_dung_duoi() {
    let duoi = jsonl(&[noi("[alpha] x"), noi("[alpha] y"), noi("[alpha] z")]);
    assert_eq!(ten("khai-muon-4", None, &duoi).as_deref(), Some("alpha"));
}
