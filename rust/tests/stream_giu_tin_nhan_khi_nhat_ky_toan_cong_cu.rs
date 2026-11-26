//! Màn Chat (`sessions::stream`) phải luôn có CÂU NÓI gần nhất của phiên, kể cả khi cuối nhật ký toàn công cụ.
//!
//! 🔴 25/11 (chụp ảnh màn Chat phiên đang làm, Hà: *"Sửa tin nhắn trước"*): phiên này nuốt nhiều kết quả công cụ
//! rất dài nên 256 KB cuối nhật ký chỉ chứa 16 sự kiện công cụ, không có lấy một câu nói — màn Chat chỉ hiện «Đang
//! làm» + ô thu gọn, không thấy tin nhắn nào. `last_prose_of_file_at` (tin báo) đã nới khung dần vì đúng ca này;
//! `stream` thì chưa.
//!
//! Ba phép đo: ① nhật ký có câu nói NẰM XA hơn 256 KB cuối ⇒ phải thấy câu ấy (RED trên mã cũ) · ② ĐỐI CHỨNG: nhật
//! ký ngắn, câu nói trong khung đầu ⇒ kết quả y như cũ (không đọc thừa, không đổi `older_hidden`) · ③ nhật ký chỉ
//! có công cụ, KHÔNG có câu nói nào ⇒ vẫn trả công cụ, không treo, không bịa câu.
#![cfg(unix)]

mod common;

use std::path::Path;

use huba::sessions;

const ID: &str = "0123abcd-0000-4000-8000-0000000000aa";
const CWD: &str = "/mnt/data/projects/huba";

fn dong_noi(chu: &str) -> String {
    format!(
        r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"{chu}"}}]}},"timestamp":"2026-11-25T01:00:00.000Z"}}"#
    )
}

fn dong_cong_cu(i: usize, to: usize) -> String {
    let nhieu = "x".repeat(to);
    format!(
        r#"{{"type":"user","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"t{i}","content":"{nhieu}"}}]}},"timestamp":"2026-11-25T01:00:01.000Z"}}"#
    )
}

fn may(noi_dung: &str) -> (tempfile::TempDir, huba::config::Config) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("claude");
    let f = sessions::transcript_path(&root, CWD, ID);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, noi_dung).unwrap();
    let mut cfg = common::cfg_for_tests();
    cfg.claude_transcript_root = root.display().to_string();
    let _: &Path = &f;
    (tmp, cfg)
}

/// ① Câu nói nằm cách cuối tệp ~600 KB toàn kết quả công cụ.
#[test]
fn cau_noi_xa_hon_khung_cuoi_van_hien() {
    let mut s = String::new();
    s.push_str(&dong_noi("CAU-NOI-CU-NHAT"));
    s.push('\n');
    for i in 0..60 {
        s.push_str(&dong_cong_cu(i, 10_000)); // ~10 KB mỗi dòng ⇒ ~600 KB
        s.push('\n');
    }
    let (_t, cfg) = may(&s);
    let st = sessions::stream(&cfg, ID, CWD, 40);
    assert!(
        st.events
            .iter()
            .any(|e| e.kind == "say" && e.text.contains("CAU-NOI-CU-NHAT")),
        "không có câu nói nào trong {} sự kiện (older_hidden={})",
        st.events.len(),
        st.older_hidden
    );
}

/// ② ĐỐI CHỨNG: nhật ký ngắn — kết quả giữ nguyên, câu nói nằm trong `limit` cuối.
#[test]
fn nhat_ky_ngan_giu_nguyen() {
    let mut s = String::new();
    s.push_str(&dong_noi("CAU-A"));
    s.push('\n');
    for i in 0..5 {
        s.push_str(&dong_cong_cu(i, 100));
        s.push('\n');
    }
    s.push_str(&dong_noi("CAU-B"));
    s.push('\n');
    let (_t, cfg) = may(&s);
    let st = sessions::stream(&cfg, ID, CWD, 40);
    assert_eq!(st.events.len(), 7);
    assert_eq!(st.older_hidden, 0);
    let st = sessions::stream(&cfg, ID, CWD, 3);
    assert_eq!(st.events.len(), 3, "câu nói cuối đã nằm trong 3 sự kiện cuối");
    assert_eq!(st.older_hidden, 4);
}

/// ③ Không có câu nói nào trong cả tệp ⇒ vẫn trả công cụ, không bịa câu.
#[test]
fn khong_co_cau_noi_thi_van_tra_cong_cu() {
    let mut s = String::new();
    for i in 0..60 {
        s.push_str(&dong_cong_cu(i, 10_000));
        s.push('\n');
    }
    let (_t, cfg) = may(&s);
    let st = sessions::stream(&cfg, ID, CWD, 40);
    assert!(!st.events.is_empty());
    assert!(st.events.iter().all(|e| e.kind != "say"));
}
