//! Mặc định `auto_handover.at_percent` = 60 — con số mà MÁY MỚI nhận.
//!
//! 🔴 Quang 21/11: gói cài không mang `huba.config.json`; máy mới chạy `huba init` và nhận mặc định
//! TRONG MÃ. Mặc định cũ là 80 trong khi repo (và chủ máy) đã chốt 60 từ 12/11 ⟹ máy cài mới lặng lẽ
//! chạy một ngưỡng khác với bản đang dùng. Bài này khoá mặc định, qua cả hai đường nó đi vào
//! (`Default` và khoá vắng khi nạp tệp), và có đối chứng ngược: giá trị chủ máy đã khai thì GIỮ.

use huba::config::AutoHandoverCfg;

#[test]
fn the_built_in_default_is_sixty() {
    assert_eq!(AutoHandoverCfg::default().at_percent, 60);
}

#[test]
fn a_config_without_the_key_falls_back_to_sixty() {
    let c: AutoHandoverCfg = serde_json::from_str(r#"{"enabled":true}"#).expect("nạp được");
    assert_eq!(c.at_percent, 60, "khoá vắng phải rơi về mặc định mới");
}

/// Đối chứng ngược: mặc định đổi không được nuốt giá trị chủ máy đã khai (máy này đang để 80).
#[test]
fn a_value_the_owner_set_is_kept() {
    let c: AutoHandoverCfg =
        serde_json::from_str(r#"{"enabled":true,"at_percent":80}"#).expect("nạp được");
    assert_eq!(c.at_percent, 80);
}

/// `huba init` ghi cấu hình từ chính mặc định này — đo đúng thứ máy mới nhận.
#[test]
fn what_init_would_write_carries_sixty() {
    let v = serde_json::to_value(AutoHandoverCfg::default()).expect("ghi được");
    assert_eq!(v["at_percent"], 60);
}
