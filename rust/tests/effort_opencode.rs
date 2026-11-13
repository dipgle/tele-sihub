//! Quang 13/11: *"effort của opencode chỉ có 3 mức: low high max"*. Các màn dưới
//! chép NGUYÊN VĂN từ `tmux capture-pane` trên Ubuntu (acc8, opencode 2.2.7).
use huba::effort::{opencode_variant, parse_opencode_level};

const DEFAULT: &str = "     Build · DeepSeek V4.1 Flash 1017 · 4.8s · 26.3 tok/s
  ┃
  ┃  Build · DeepSeek V4.1 Flash 1017 OpenCode Go
  ╹▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
  /run/media/user1/DATA/projects                    6.6K (1%)  ctrl+p commands";

const HIGH: &str = "     Build · DeepSeek V4.1 Flash 1017 · 4.8s · 26.3 tok/s
  ┃
  ┃  Build · DeepSeek V4.1 Flash 1017 OpenCode Go · high
  ╹▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
  /run/media/user1/DATA/projects                    6.6K (1%)  ctrl+p commands";

#[test]
fn doc_muc_tu_hau_to_dong_trang_thai() {
    assert_eq!(opencode_variant(DEFAULT).as_deref(), Some("default"));
    assert_eq!(opencode_variant(HIGH).as_deref(), Some("high"));
    let max = HIGH.replace("Go · high", "Go · max");
    assert_eq!(opencode_variant(&max).as_deref(), Some("max"));
}

/// Dòng chân của một câu trả lời (`· 4.8s · 26.3 tok/s`) KHÔNG phải dòng trạng thái
/// — nó không mang `┃`. Màn không có ô nhập ⟹ None, không đoán "default".
#[test]
fn khong_doan_khi_khong_thay_o_nhap() {
    assert_eq!(
        opencode_variant("     Build · DeepSeek V4.1 Flash 1017 · 4.8s · 26.3 tok/s"),
        None
    );
    assert_eq!(opencode_variant(""), None);
}

#[test]
fn muc_opencode_hop_le() {
    assert_eq!(parse_opencode_level("HIGH"), Ok("high"));
    assert_eq!(parse_opencode_level("default"), Ok("default"));
    // Mức của claude mà opencode không có ⟹ nói rõ, không đổi bừa.
    assert!(parse_opencode_level("xhigh").is_err());
    assert!(parse_opencode_level("medium").is_err());
}

/// `/new` mở opencode ở mức `high` (hoặc `effort` của hàng acc), không phải Default.
#[test]
fn new_mac_dinh_high_cho_build_va_plan() {
    use huba::accounts::dat_variant_mac_dinh;
    use serde_json::json;
    let mut r = json!({});
    dat_variant_mac_dinh(&mut r, "acc8", "opencode-go/ds", None);
    for a in ["build", "plan"] {
        assert_eq!(r["agent"][a]["variant"], "high", "{r}");
        assert_eq!(r["agent"][a]["model"], "opencode-go/ds");
    }
    let mut r = json!({});
    dat_variant_mac_dinh(&mut r, "acc8", "m", Some("max"));
    assert_eq!(r["agent"]["build"]["variant"], "max");
    // Chủ máy đã khai ⟹ giữ, kể cả model của agent.
    let mut r = json!({ "agent": { "build": { "model": "khac", "variant": "low" } } });
    dat_variant_mac_dinh(&mut r, "acc8", "m", None);
    assert_eq!(
        r["agent"]["build"],
        json!({ "model": "khac", "variant": "low" })
    );
    assert_eq!(r["agent"]["plan"]["variant"], "high");
}
