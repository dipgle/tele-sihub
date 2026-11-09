//! GĐ 3 — MỘT cửa ghi cho `claude_accounts[]` (`config::edit_accounts`).
//!
//! Chủ máy chốt 09/11: **xoá = chỉ khoá**, và trang quản lý là chủ thể GHI thứ
//! hai bên cạnh buồng chat. Hai bản sao của "đọc tệp → sửa → ghi" là hai chỗ để
//! lệch nhau, nên GĐ 3 gom về `config::edit_accounts` + hai thao tác trong
//! `accounts`: [`huba::accounts::upsert_account`] (thêm/sửa) và
//! [`huba::pipeline::set_account_locked`] (khoá/mở).
//!
//! Mỗi bài dưới đây có vế NGƯỢC: một bài xanh không được xanh vì cửa chặn tất.

mod common;

use std::path::Path;

use huba::accounts::{upsert_account, AccountWrite};
use huba::config::{ClaudeAccountCfg, Config};

fn tep_cau_hinh(dir: &Path) -> Config {
    let path = dir.join("huba.config.json");
    let mut c = common::cfg_for_tests();
    c.claude_accounts = vec![
        ClaudeAccountCfg {
            name: "acc1".into(),
            launch: Some("claude".into()),
            ..Default::default()
        },
        ClaudeAccountCfg {
            name: "acc3".into(),
            config_dir: Some("~/.claude-acc3".into()),
            launch: Some("claude3".into()),
            locked: true,
            ..Default::default()
        },
    ];
    c.config_file = path;
    c.hub_home = dir.to_path_buf();
    huba::config::save(&c).unwrap();
    c
}

fn doc(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn hang(path: &Path, ten: &str) -> serde_json::Value {
    doc(path)["claude_accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == ten)
        .cloned()
        .unwrap_or_else(|| panic!("không có hàng '{ten}' trong {}", path.display()))
}

#[test]
fn them_tai_khoan_moi_ghi_du_truong_va_khong_dung_hang_khac() {
    let dir = tempfile::tempdir().unwrap();
    let c = tep_cau_hinh(dir.path());
    let path = c.config_file.clone();
    let truoc: Vec<String> = doc(&path)["claude_accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap().to_string())
        .collect();

    let moi = ClaudeAccountCfg {
        name: " acc9 ".into(),
        config_dir: Some("~/.claude-acc9".into()),
        base_url: Some("https://example.test".into()),
        env_file: Some("~/.claude-acc9/env".into()),
        api_key_env: Some("OPENCODE_API_KEY".into()),
        model: Some("m9".into()),
        ..Default::default()
    };
    assert_eq!(
        upsert_account(&c, moi).unwrap(),
        AccountWrite::Added,
        "tên mới phải là THÊM"
    );

    let h = hang(&path, "acc9");
    assert_eq!(h["config_dir"], "~/.claude-acc9");
    assert_eq!(h["base_url"], "https://example.test");
    assert_eq!(
        h["api_key_env"], "OPENCODE_API_KEY",
        "chỉ TÊN biến (luật 4)"
    );
    assert_eq!(h["model"], "m9");
    assert_eq!(h["name"], "acc9", "tên được cắt khoảng trắng");
    assert!(h.get("locked").is_none(), "mới thêm thì không khoá");

    let sau: Vec<String> = doc(&path)["claude_accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap().to_string())
        .collect();
    let mut mong = truoc.clone();
    mong.push("acc9".into());
    assert_eq!(sau, mong, "hàng cũ y nguyên, hàng mới ở CUỐI");

    // Nạp lại từ tệp: thứ trang sẽ thấy ở lần mở sau.
    let nap = huba::config::load(Some(&path)).unwrap();
    assert!(nap.claude_accounts.iter().any(|a| a.name == "acc9"));
    // Không có giá trị bí mật nào trong tệp cấu hình.
    let tho = std::fs::read_to_string(&path).unwrap();
    assert!(
        !tho.contains("printenv"),
        "cấu hình không mang dòng lệnh: {tho}"
    );
}

#[test]
fn sua_tai_khoan_giu_nguyen_khoa_va_khong_them_hang_thu_hai() {
    let dir = tempfile::tempdir().unwrap();
    let c = tep_cau_hinh(dir.path());
    let path = c.config_file.clone();

    // acc3 đang KHOÁ trong tệp; biểu mẫu gửi `locked: false` (mặc định).
    let sua = ClaudeAccountCfg {
        name: "acc3".into(),
        config_dir: Some("~/.claude-acc3".into()),
        launch: Some("claude3".into()),
        model: Some("m-moi".into()),
        locked: false,
        ..Default::default()
    };
    assert_eq!(
        upsert_account(&c, sua).unwrap(),
        AccountWrite::Updated,
        "tên đã có phải là SỬA"
    );

    let h = hang(&path, "acc3");
    assert_eq!(h["model"], "m-moi", "trường biểu mẫu phải vào tệp");
    assert_eq!(h["locked"], true, "biểu mẫu không được MỞ KHOÁ hộ chủ máy");
    let so_hang = doc(&path)["claude_accounts"].as_array().unwrap().len();
    assert_eq!(so_hang, 2, "sửa không được đẻ thêm hàng");

    // Vế ngược: mở khoá bằng đường riêng thì hàng mở lại được.
    huba::pipeline::set_account_locked(&c, "acc3", false).unwrap();
    assert!(
        hang(&path, "acc3").get("locked").is_none(),
        "mở khoá phải xoá hẳn khoá (hình dạng cũ)"
    );
}

#[test]
fn ten_hong_bi_tu_choi_va_tep_khong_bi_dung_toi() {
    let dir = tempfile::tempdir().unwrap();
    let c = tep_cau_hinh(dir.path());
    let path = c.config_file.clone();
    let truoc = std::fs::read_to_string(&path).unwrap();

    for ten in ["", "   ", "acc 9", "acc/9", ".huba", &"x".repeat(41)] {
        let e = upsert_account(
            &c,
            ClaudeAccountCfg {
                name: ten.to_string(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(!e.is_empty(), "tên {ten:?} phải bị từ chối KÈM lý do");
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        truoc,
        "lượt bị từ chối không được chạm tệp"
    );
    // Vế ngược: cùng hàm, tên hợp lệ, thì ghi thật.
    upsert_account(
        &c,
        ClaudeAccountCfg {
            name: "acc_9-b".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hang(&path, "acc_9-b")["name"], "acc_9-b");
}

#[test]
fn tep_cau_hinh_hong_thi_bao_loi_chu_khong_ghi_de() {
    let dir = tempfile::tempdir().unwrap();
    let c = tep_cau_hinh(dir.path());
    let path = c.config_file.clone();
    std::fs::write(&path, "{ không phải JSON").unwrap();
    let hong = std::fs::read_to_string(&path).unwrap();

    let e = upsert_account(
        &c,
        ClaudeAccountCfg {
            name: "acc9".into(),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(e.contains("JSON"), "lỗi phải nói tệp hỏng: {e}");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        hong,
        "không được ghi đè"
    );
}

/// `edit_accounts` KHÔNG cho `f` trả lỗi mà vẫn ghi: đọc → `f` hỏng → không ghi.
#[test]
fn f_tra_loi_thi_khong_ghi_gi() {
    let dir = tempfile::tempdir().unwrap();
    let c = tep_cau_hinh(dir.path());
    let path = c.config_file.clone();
    let truoc = std::fs::read_to_string(&path).unwrap();
    let r = huba::config::edit_accounts(&c, |rows| {
        rows.clear(); // hình dạng cấm nhất: xoá sạch
        anyhow::bail!("dừng giữa chừng")
    });
    assert!(r.is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        truoc,
        "f hỏng ⇒ tệp y nguyên"
    );
}
