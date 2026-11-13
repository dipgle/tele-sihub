//! Ubuntu 13/11: `opencode serve --service` dựng 09:32 bằng cấu hình chưa có `kem`
//! sống tiếp; `/new` 09:57 truyền cấu hình MỚI mà màn vẫn «1 MCP failed · 1 plugin
//! failed» — TUI chỉ nối vào dịch vụ cũ. Đây là phần thuần của phép nhận diện.
use std::collections::BTreeMap;

use huba::config::ClaudeAccountCfg;
use huba::opencode::{la_tui_cua, xet_dich_vu, DichVu};
use serde_json::json;

const HO: &str = "/home/u/.opencode-acc8/config";

fn env(cfg: &str, noi_dung: &str) -> BTreeMap<String, String> {
    [
        ("XDG_CONFIG_HOME".to_string(), cfg.to_string()),
        ("OPENCODE_CONFIG_CONTENT".to_string(), noi_dung.to_string()),
    ]
    .into_iter()
    .collect()
}

fn argv(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}

#[test]
fn dich_vu_cung_phan_vung_cau_hinh_khac_la_lech() {
    let moi = json!({ "plugin": ["/k/cong-bao-mat-plugin"], "model": "m" });
    let a = argv(&["/home/u/.opencode/bin/opencode", "serve", "--service"]);
    assert_eq!(
        xet_dich_vu(&a, &env(HO, r#"{"plugin":["/old"],"model":"m"}"#), HO, &moi),
        DichVu::Lech
    );
    // Đối chứng: cùng cấu hình, KHÁC thứ tự khoá ⟹ khớp, không giết oan.
    assert_eq!(
        xet_dich_vu(
            &a,
            &env(HO, r#"{"model":"m","plugin":["/k/cong-bao-mat-plugin"]}"#),
            HO,
            &moi
        ),
        DichVu::Khop
    );
    // Cấu hình không đọc ra ⟹ lệch (dựng lại an toàn hơn tin một thứ không đọc được).
    assert_eq!(xet_dich_vu(&a, &env(HO, "{hong"), HO, &moi), DichVu::Lech);
}

#[test]
fn khong_dung_dich_vu_acc_khac_hay_tui() {
    let moi = json!({});
    let dv = argv(&["/x/opencode", "serve", "--service"]);
    assert_eq!(
        xet_dich_vu(&dv, &env("/home/u/.opencode-acc9/config", "{}"), HO, &moi),
        DichVu::KhongLienQuan
    );
    let tui = argv(&["/x/opencode"]);
    assert_eq!(
        xet_dich_vu(&tui, &env(HO, "{\"a\":1}"), HO, &moi),
        DichVu::KhongLienQuan
    );
    assert!(la_tui_cua(&tui, &env(HO, "{}"), HO));
    assert!(!la_tui_cua(&dv, &env(HO, "{}"), HO));
    assert!(!la_tui_cua(&tui, &env("/khac", "{}"), HO));
    // Tên gần giống không phải opencode.
    assert!(!la_tui_cua(
        &argv(&["/x/opencode-helper"]),
        &env(HO, "{}"),
        HO
    ));
}

/// Hàng acc chép từ máy khác khai cổng ở đường của máy ấy ⟹ chỉ còn MỘT cổng.
#[test]
fn chi_con_mot_cong_bao_mat() {
    std::env::set_var("HUBA_OPENCODE_PLUGIN", "/k/scripts/cong-bao-mat-plugin");
    let acc: ClaudeAccountCfg = serde_json::from_value(json!({
        "name": "accx", "cli": "opencode",
        "opencode_config": { "plugin": [
            "/home/u/projects/scripts/cong-bao-mat-plugin", "/khac/plugin-rieng"
        ] }
    }))
    .unwrap();
    let v = huba::accounts::opencode_config_content(&acc);
    assert_eq!(
        v["plugin"],
        json!(["/khac/plugin-rieng", "/k/scripts/cong-bao-mat-plugin"])
    );
}
