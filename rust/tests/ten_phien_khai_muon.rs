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
    ten_rong(id, head, tail, None)
}

/// `rong` = các lượt nói đọc từ cửa sổ RỘNG của nhật ký (None = không đọc được / không gọi tới).
fn ten_rong(id: &str, head: Option<&str>, tail: &str, rong: Option<String>) -> Option<String> {
    let ws = workspace();
    huba::sessions::folder_for_head_or_tail(id, head, tail, ws.to_str().unwrap(), "", || rong)
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

/// ⑤ CA ĐO LẠI 24/11 17:0x — nhật ký toàn công cụ: 256 KB đuôi chỉ phủ 6 lượt nói và MỘT thẻ (phiên vừa chuyển
/// giao), nhưng cửa sổ rộng có ba. Đầu im + đuôi một lần ⟹ phải đọc rộng.
#[test]
fn duoi_hep_chi_mot_lan_thi_doc_rong() {
    let duoi_hep = jsonl(&[
        noi("Việc."),
        noi("[alpha/dci] Đã chuyển phiên."),
        noi("Xong."),
    ]);
    let rong = jsonl(&[
        noi("[alpha/dci] Đang chờ kiểm."),
        noi("Commit xong."),
        noi("[alpha/dci] Kiểm xanh."),
        noi("Việc."),
        noi("[alpha/dci] Đã chuyển phiên."),
    ]);
    assert_eq!(
        ten_rong("khai-muon-5", Some(&dau_im()), &duoi_hep, Some(rong)).as_deref(),
        Some("alpha")
    );
}

/// ⑥ ĐỐI CHỨNG — cửa sổ rộng cũng chỉ có MỘT lần ⟹ vẫn là nhắc, không tên.
#[test]
fn rong_cung_chi_mot_lan_van_khong_ten() {
    let duoi_hep = jsonl(&[noi("[alpha/dci] Một lần."), noi("Xong.")]);
    let rong = jsonl(&[noi("Việc."), noi("[alpha/dci] Một lần."), noi("Xong.")]);
    assert_eq!(
        ten_rong("khai-muon-6", Some(&dau_im()), &duoi_hep, Some(rong)),
        None
    );
}

/// ⑦ `read_spoken_wide` chỉ giữ lượt nói, bỏ công cụ/đính kèm, và với tệp > 4 MiB vẫn thấy lượt cuối.
#[test]
fn doc_rong_chi_giu_luot_noi_va_thay_cuoi_tep() {
    let mut dong = vec![noi("[alpha] lượt ĐẦU, nằm ngoài cửa sổ 4 MiB.")];
    for _ in 0..3 {
        dong.push(
            serde_json::json!({"type": "attachment", "payload": "x".repeat(1_500_000)}).to_string(),
        );
    }
    dong.push(
        serde_json::json!({"type": "assistant", "message": {"role": "assistant", "content": [
            {"type": "tool_use", "id": "t", "name": "Bash", "input": {"command": "ls"}}]}})
        .to_string(),
    );
    dong.push(noi("[alpha/dci] lượt CUỐI."));
    let dir = std::env::temp_dir().join(format!("huba-doc-rong-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("r.jsonl");
    std::fs::write(&path, jsonl(&dong)).unwrap();
    assert!(std::fs::metadata(&path).unwrap().len() > 4 * 1024 * 1024);

    let r = huba::sessions::read_spoken_wide(&path).unwrap();
    assert!(r.contains("lượt CUỐI"), "mất lượt cuối: {} byte", r.len());
    assert!(!r.contains("lượt ĐẦU"), "lượt đầu phải nằm ngoài cửa sổ");
    assert!(
        !r.contains("attachment") && !r.contains("tool_use"),
        "phải bỏ dòng không phải lượt nói"
    );
    assert!(r.len() < 1024, "phình {} byte", r.len());
}
