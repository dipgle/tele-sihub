//! Phiếu zkey 23/11 — tên tệp trong `.inbox` phải tự nói được: ngày-giờ, dự
//! án, mã phiên, tên người gửi; và KHÔNG BAO GIỜ đè bản trước.
//!
//! Phần nhãn phiên nằm ở `sessions.rs` (`mod phieu_zkey_20261123`).

use huba::telegram::{ghi_khong_de, inbox_file_name};

#[test]
fn ten_tep_du_bon_manh() {
    assert_eq!(
        inbox_file_name("20261123-0845", "zkey", "dfef9718", "anh-AQADnxlrG.jpg"),
        "20261123-0845_zkey_dfef9718_anh-AQADnxlrG.jpg"
    );
    // Dự án có ngăn kéo / làn: dấu `/` không được thành thư mục con.
    assert_eq!(
        inbox_file_name("20261123-0845", "AI/zkey", "dfef9718", "log.txt"),
        "20261123-0845_AI-zkey_dfef9718_log.txt"
    );
    // Không biết dự án/phiên: vẫn có chỗ đứng rõ ràng, không ra chuỗi rỗng.
    assert_eq!(
        inbox_file_name("20261123-0845", "", "", "a.jpg"),
        "20261123-0845_khong-ro_khong-ro_a.jpg"
    );
}

#[test]
fn hai_tep_cung_ten_khong_de_nhau() {
    let d = tempfile::tempdir().expect("tempdir");
    let ten = "20261123-0845_zkey_dfef9718_image.jpg";
    let p1 = ghi_khong_de(d.path(), ten, b"ban-mot").expect("ghi lan 1");
    let p2 = ghi_khong_de(d.path(), ten, b"ban-hai").expect("ghi lan 2");
    assert_ne!(p1, p2, "lần thứ hai phải ra tên KHÁC");
    assert_eq!(p1.file_name().unwrap().to_str().unwrap(), ten);
    assert_eq!(
        p2.file_name().unwrap().to_str().unwrap(),
        "20261123-0845_zkey_dfef9718_image-2.jpg"
    );
    assert_eq!(
        std::fs::read(&p1).unwrap(),
        b"ban-mot",
        "bản 1 phải còn nguyên"
    );
    assert_eq!(std::fs::read(&p2).unwrap(), b"ban-hai");
}

#[test]
fn ba_lan_cung_ten_thi_ra_2_va_3() {
    let d = tempfile::tempdir().expect("tempdir");
    let ten = "x.txt";
    let mut ten_da_co = Vec::new();
    for _ in 0..3 {
        let p = ghi_khong_de(d.path(), ten, b"x").expect("ghi");
        ten_da_co.push(p.file_name().unwrap().to_str().unwrap().to_string());
    }
    assert_eq!(ten_da_co, vec!["x.txt", "x-2.txt", "x-3.txt"]);
}
