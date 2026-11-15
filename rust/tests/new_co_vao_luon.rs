//! `/new -y` / `-f` — tài khoản ĐANG KHOÁ vẫn mở, không hỏi xác nhận.
//!
//! Hà 15/11: *«lệnh mở phiên cần thêm cờ để nếu acc đang khóa vẫn vào luôn mà không hỏi
//! xác nhận»*. Cờ chỉ được bóc trong dãy cờ ĐẦU lệnh — đề bài giữ nguyên văn (luật cờ:
//! lặng lẽ ăn một phần đề bài là chạy một việc khác việc đã gõ).

use huba::pipeline::tach_co_vao_luon;

const NEW_FLAGS: &[&str] = &["a", "acc", "account", "s", "p", "project", "duan", "du-an"];

#[test]
fn co_vao_luon_o_dau_lenh_duoc_nhan() {
    assert_eq!(
        tach_co_vao_luon("-a acc10 -y sửa lịch", NEW_FLAGS),
        (true, "-a acc10 sửa lịch".to_string())
    );
    assert_eq!(
        tach_co_vao_luon("-y -a acc10 -s dwork sửa lịch", NEW_FLAGS),
        (true, "-a acc10 -s dwork sửa lịch".to_string())
    );
    assert_eq!(
        tach_co_vao_luon("-f -a acc10", NEW_FLAGS),
        (true, "-a acc10".to_string()),
        "-f cũng nhận, không đề bài vẫn được"
    );
}

#[test]
fn co_trong_de_bai_khong_bi_an() {
    // ĐỐI CHỨNG NGƯỢC: `-f` thuộc ĐỀ BÀI ⟹ giữ nguyên, và không tính là cờ.
    assert_eq!(
        tach_co_vao_luon("-a acc2 chạy rm -f x.tmp", NEW_FLAGS),
        (false, "-a acc2 chạy rm -f x.tmp".to_string())
    );
    assert_eq!(
        tach_co_vao_luon("sửa lỗi -y trong tệp", NEW_FLAGS),
        (false, "sửa lỗi -y trong tệp".to_string())
    );
    // Không cờ ⟹ nguyên văn.
    assert_eq!(
        tach_co_vao_luon("-a acc9 đọc CLAUDE.md", NEW_FLAGS),
        (false, "-a acc9 đọc CLAUDE.md".to_string())
    );
    assert_eq!(tach_co_vao_luon("", NEW_FLAGS), (false, String::new()));
}
