use huba::accounts::{claude_het_hoi_doc_ngoai_json, claude_trust_json, tin_thu_muc, TinCay};
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

// ── Hộp «auto mode — cho đọc ngoài thư mục làm việc?» (18/11) ─────────────────
//
// Ca thật 18/11 (Ubuntu, khởi động lại máy): phiên đầu tiên `/new acc3` mang
// claude 2.1.327 — một `Read` của chính tệp tool-result của nó (nằm ngoài cwd)
// dựng hộp bốn lựa chọn «Allow this read outside the working directories?» và
// ĐỨNG chờ một phím. Đo trong nhị phân 2.1.327: hộp chỉ hiện khi
// `hasSeenAutoModeOutsideReadPrompt` của sổ tài khoản KHÔNG true; lựa chọn
// «Yes, and keep allowing» chính là việc đặt cờ ấy (`ipa`). Đo trên máy: acc3 ·
// acc6 có cờ, acc2 · acc4 · acc5 · acc7 KHÔNG — tài khoản nào mở lần đầu cũng
// gặp lại đúng hộp này.

#[test]
fn so_chua_danh_dau_thi_ghi_danh_dau_doc_ngoai() {
    let co = r#"{"oauthAccount":{"emailAddress":"a@b"},"projects":{"/g":{"lastCost":2}}}"#;
    let moi: Value =
        serde_json::from_str(&claude_het_hoi_doc_ngoai_json(co).unwrap().unwrap()).unwrap();
    assert_eq!(moi["hasSeenAutoModeOutsideReadPrompt"], true);
    assert_eq!(moi["projects"]["/g"]["lastCost"], 2, "khoá khác giữ nguyên");
    assert_eq!(moi["oauthAccount"]["emailAddress"], "a@b");
    assert!(
        moi.get("permissions").is_none(),
        "KHÔNG tự thêm permissions — chặn/cho đọc vẫn là quyết định của chủ máy"
    );
}

#[test]
fn so_da_danh_dau_thi_khong_ghi_lai() {
    let da = r#"{"hasSeenAutoModeOutsideReadPrompt":true}"#;
    assert_eq!(claude_het_hoi_doc_ngoai_json(da).unwrap(), None);
    assert!(
        claude_het_hoi_doc_ngoai_json("{hỏng").is_err(),
        "sổ hỏng ⟹ Err, KHÔNG viết lại"
    );
    assert!(claude_het_hoi_doc_ngoai_json("[]").is_err());
}

#[test]
fn so_da_chan_doc_ngoai_thi_khong_go_chan() {
    // Chủ máy từng bấm «No, and block» ⟹ `permissions` đã có khoá chặn. Cờ
    // «đã thấy hộp» vẫn ghi, nhưng KHÔNG được bỏ khoá chặn ấy.
    let co = r#"{"permissions":{"blockReadsOutsideWorkingDirectories":true},"hasSeenAutoModeEntryWarning":true}"#;
    let moi: Value =
        serde_json::from_str(&claude_het_hoi_doc_ngoai_json(co).unwrap().unwrap()).unwrap();
    assert_eq!(moi["hasSeenAutoModeOutsideReadPrompt"], true);
    assert_eq!(
        moi["permissions"]["blockReadsOutsideWorkingDirectories"],
        true
    );
}

#[test]
fn mo_phien_ghi_danh_dau_ke_ca_khi_thu_muc_da_tin() {
    // Ca thật: acc3 ĐÃ tin `/mnt/data/projects` từ 17/11 nhưng cờ chưa có ⟹
    // nhánh claude của `tin_thu_muc` phải ghi cờ dù trust đã đủ.
    let goc = tempfile::tempdir().unwrap();
    let so = tempfile::tempdir().unwrap();
    let so_json = so.path().join(".claude.json");
    let da_tin = format!(
        r#"{{"projects":{{"{}":{{"hasTrustDialogAccepted":true}}}}}}"#,
        that(goc.path()).replace('\\', "\\\\")
    );
    std::fs::write(&so_json, da_tin).unwrap();
    let a = acc("acc3", None, so.path());
    let cfg = cfg_voi(goc.path(), vec![a.clone()]);
    assert_eq!(
        tin_thu_muc(&cfg, Some(&a), goc.path()),
        Ok(TinCay::VuaTatHopDocNgoai),
        "trust đã đủ — lượt này chỉ ghi cờ hộp đọc-ngoài"
    );
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&so_json).unwrap()).unwrap();
    assert_eq!(v["hasSeenAutoModeOutsideReadPrompt"], true);
    assert_eq!(
        v["projects"][that(goc.path())]["hasTrustDialogAccepted"],
        true
    );
    // Lần hai: cả hai khoá đã đủ ⟹ không ghi gì nữa.
    assert_eq!(tin_thu_muc(&cfg, Some(&a), goc.path()), Ok(TinCay::DaCo));
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
        "crate::accounts::tin_truoc_khi_mo(cfg, Some(&acc), root);",
        "",
        1,
    );
    assert_ne!(hong, SESSIONS, "mẫu cấy không còn trong mã — sửa bài thử");
    assert_eq!(mo_thieu_cua(&hong).len(), 1);
}
