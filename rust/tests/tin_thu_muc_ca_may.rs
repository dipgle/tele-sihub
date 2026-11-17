//! «Tin thư mục» là chuyện của MÁY (Quang 17/11): một lời khai — gốc workspace —
//! dịch xuống sổ riêng của tài khoản sắp mở, CLI nào cũng qua MỘT cửa.
//!
//! Ca thật 17/11 (Ubuntu): `/new acc3` — sổ acc3 chưa tin `/mnt/data/projects`,
//! claude 2.1.326 vẽ hộp KHÔNG đánh số, con trỏ sẵn ở «No, exit»; huba không thấy
//! hộp, gõ đề bài + Enter ⟹ phiên chết. Trước bản này chỉ codex có ghi sẵn.
use huba::accounts::{claude_trust_json, tin_thu_muc, TinCay};
use huba::config::{ClaudeAccountCfg, Config};
use serde_json::Value;
use std::path::Path;

fn cfg_voi(goc: &Path, accs: Vec<ClaudeAccountCfg>) -> Config {
    Config {
        workspace_root: goc.to_path_buf(),
        claude_accounts: accs,
        ..Default::default()
    }
}

fn acc(name: &str, cli: Option<&str>, dir: &Path) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: name.into(),
        cli: cli.map(Into::into),
        config_dir: Some(dir.to_string_lossy().into_owned()),
        ..Default::default()
    }
}

fn that(p: &Path) -> String {
    std::fs::canonicalize(p)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

#[test]
fn claude_json_chi_cham_dung_mot_khoa() {
    let co = r#"{"oauthAccount":{"emailAddress":"a@b"},"projects":{"/khac":{"x":1},"/g":{"lastCost":2}}}"#;
    let moi: Value = serde_json::from_str(&claude_trust_json(co, "/g").unwrap().unwrap()).unwrap();
    assert_eq!(moi["projects"]["/g"]["hasTrustDialogAccepted"], true);
    assert_eq!(
        moi["projects"]["/g"]["lastCost"], 2,
        "khoá khác của cùng mục giữ nguyên"
    );
    assert_eq!(moi["projects"]["/khac"]["x"], 1);
    assert_eq!(
        moi["oauthAccount"]["emailAddress"], "a@b",
        "đăng nhập giữ nguyên"
    );
}

#[test]
fn claude_json_da_tin_thi_khong_ghi_so_hong_thi_khong_de() {
    let da = r#"{"projects":{"/g":{"hasTrustDialogAccepted":true}}}"#;
    assert_eq!(claude_trust_json(da, "/g").unwrap(), None);
    assert!(
        claude_trust_json("{hỏng", "/g").is_err(),
        "sổ hỏng ⟹ Err, KHÔNG viết lại"
    );
    assert!(claude_trust_json("[]", "/g").is_err());
}

#[test]
fn claude_ghi_mot_lan_lan_hai_da_co() {
    let goc = tempfile::tempdir().unwrap();
    let so = tempfile::tempdir().unwrap();
    std::fs::write(
        so.path().join(".claude.json"),
        r#"{"oauthAccount":{"emailAddress":"x@y"}}"#,
    )
    .unwrap();
    let a = acc("acc3", None, so.path());
    let cfg = cfg_voi(goc.path(), vec![a.clone()]);
    assert_eq!(tin_thu_muc(&cfg, Some(&a), goc.path()), Ok(TinCay::VuaGhi));
    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(so.path().join(".claude.json")).unwrap())
            .unwrap();
    assert_eq!(
        v["projects"][that(goc.path())]["hasTrustDialogAccepted"],
        true
    );
    assert_eq!(v["oauthAccount"]["emailAddress"], "x@y");
    assert_eq!(tin_thu_muc(&cfg, Some(&a), goc.path()), Ok(TinCay::DaCo));
    // Thư mục CON của gốc: claude thừa kế từ cha ⟹ cùng một khoá gốc, không thêm mục mới.
    let con = goc.path().join("du-an");
    std::fs::create_dir(&con).unwrap();
    assert_eq!(tin_thu_muc(&cfg, Some(&a), &con), Ok(TinCay::DaCo));
}

#[test]
fn claude_chua_co_so_thi_khong_tao_thay_cli() {
    let goc = tempfile::tempdir().unwrap();
    let so = tempfile::tempdir().unwrap();
    let a = acc("acc4", None, so.path());
    let cfg = cfg_voi(goc.path(), vec![a.clone()]);
    assert_eq!(
        tin_thu_muc(&cfg, Some(&a), goc.path()),
        Ok(TinCay::ChuaCoSo)
    );
    assert!(!so.path().join(".claude.json").exists());
}

#[test]
fn codex_ghi_config_toml_theo_dung_thu_muc() {
    let goc = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let a = acc("acc10", Some("codex"), home.path());
    let cfg = cfg_voi(goc.path(), vec![a.clone()]);
    assert_eq!(tin_thu_muc(&cfg, Some(&a), goc.path()), Ok(TinCay::VuaGhi));
    let toml = std::fs::read_to_string(home.path().join("config.toml")).unwrap();
    assert!(
        toml.contains(&format!("[projects.\"{}\"]", that(goc.path()))),
        "{toml}"
    );
    assert!(toml.contains("trust_level = \"trusted\""));
    assert_eq!(tin_thu_muc(&cfg, Some(&a), goc.path()), Ok(TinCay::DaCo));
}

#[test]
fn opencode_khong_co_hop_va_ngoai_goc_khong_ghi() {
    let goc = tempfile::tempdir().unwrap();
    let ngoai = tempfile::tempdir().unwrap();
    let so = tempfile::tempdir().unwrap();
    let oc = acc("acc8", Some("opencode"), so.path());
    let cl = acc("acc3", None, so.path());
    std::fs::write(so.path().join(".claude.json"), "{}").unwrap();
    let cfg = cfg_voi(goc.path(), vec![oc.clone(), cl.clone()]);
    assert_eq!(
        tin_thu_muc(&cfg, Some(&oc), goc.path()),
        Ok(TinCay::KhongCoHop)
    );
    assert_eq!(
        tin_thu_muc(&cfg, Some(&cl), ngoai.path()),
        Ok(TinCay::NgoaiGoc)
    );
    assert_eq!(
        std::fs::read_to_string(so.path().join(".claude.json")).unwrap(),
        "{}"
    );
}

#[test]
fn cong_tac_mac_dinh_bat_va_tat_duoc() {
    assert!(Config::default().tin_goc_workspace);
    let tat: Config = serde_json::from_str(r#"{"tin_goc_workspace": false}"#).unwrap();
    assert!(!tat.tin_goc_workspace);
    let cu: Config = serde_json::from_str("{}").unwrap();
    assert!(cu.tin_goc_workspace, "cấu hình cũ không có khoá ⟹ bật");
}

// ── Cổng cấu trúc: MỌI chỗ mở phiên đều qua cửa trust ─────────────────────────

/// Số dòng `keys::open_window(` / spawn `--bg` KHÔNG có `tin_truoc_khi_mo` trong
/// 6 dòng ngay trên. Cửa sổ terminal trơn (`BARE_TERMINAL_CMD`) không mở CLI ⟹ bỏ qua.
fn mo_thieu_cua(src: &str) -> Vec<usize> {
    let dong: Vec<&str> = src.lines().collect();
    let mut thieu = vec![];
    for (i, l) in dong.iter().enumerate() {
        let mo = (l.contains("crate::keys::open_window(") && !l.contains("BARE_TERMINAL_CMD"))
            || l.contains("vec![\"--bg\", &task_with_project");
        if mo
            && !dong[i.saturating_sub(6)..i]
                .iter()
                .any(|t| t.contains("tin_truoc_khi_mo"))
        {
            thieu.push(i + 1);
        }
    }
    thieu
}

const SESSIONS: &str = include_str!("../src/sessions.rs");

#[test]
fn moi_cho_mo_phien_deu_qua_cua_trust() {
    let so_cho = SESSIONS.matches("crate::keys::open_window(").count();
    assert!(so_cho >= 4, "mẫu số: {so_cho} chỗ open_window");
    assert_eq!(mo_thieu_cua(SESSIONS), Vec::<usize>::new());
}

#[test]
fn doi_chung_go_mot_cua_thi_cong_do() {
    // Gỡ lời gọi ở MỘT chỗ ⟹ cổng phải chỉ ra đúng chỗ ấy.
    let hong = SESSIONS.replacen(
        "crate::accounts::tin_truoc_khi_mo(cfg, Some(acc), dir);",
        "",
        1,
    );
    assert_ne!(hong, SESSIONS, "mẫu cấy không còn trong mã — sửa bài thử");
    assert_eq!(mo_thieu_cua(&hong).len(), 1);
}
