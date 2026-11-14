//! Hàng tài khoản `cli: "codex"` (OpenAI Codex CLI, đăng nhập ChatGPT).
//!
//! Quang 14/11: *"cài cli của chatgpt đi để thêm acc10"* · *"Phải đăng nhập lệnh
//! giống claude"*. huba ĐĂNG NHẬP được hàng codex (`/accounts <tên> -l`) nhưng
//! CHƯA mở phiên trên nó (codex không có `--disallowedTools` ⟹ chưa có tường của
//! luật 1). Mỗi bài có vế NGƯỢC: cùng cấu hình, hàng claude ⟹ không bị chặn, để
//! một bài xanh không xanh vì cửa chặn tất cả.

mod common;

use huba::account_login::{codex_after, codex_device, la_man_codex, la_man_dang_nhap, login_pid};
use huba::config::{ClaudeAccountCfg, Config};
use huba::sessions::ensure_account_usable;

/// Màn THẬT của `codex login --device-auth` (codex-cli 0.169.0, 14/11, acc10) —
/// mã đã dùng xong, giữ nguyên văn làm bằng chứng. Dòng đầu là lệnh huba gõ.
const MAN_CHO: &str = "user1@user1-ubuntu:~$ CODEX_HOME=$HOME/.codex-acc10 $HOME/.npm-global/bin/codex login --device-auth
Welcome to Codex [v0.169.0]
OpenAI's command-line coding agent

Follow these steps to sign in with ChatGPT using device code authorization:

1. Open this link in your browser and sign in to your account
   https://auth.openai.com/codex/device

2. Enter this one-time code (expires in 15 minutes)
   HBAC-ERA9M

Continue only if you started this login in Codex. If a website or another person gave you this code, cancel.
";

fn hang(ten: &str, cli: Option<&str>) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: ten.into(),
        config_dir: Some(format!("~/.claude-{ten}")),
        cli: cli.map(str::to_string),
        ..Default::default()
    }
}

fn cfg_hai() -> Config {
    let mut c = common::cfg_for_tests();
    c.claude_accounts = vec![hang("acc1", None), hang("acc10", Some("codex"))];
    c
}

#[test]
fn doc_duoc_duong_dan_va_ma_tren_man_that() {
    assert!(la_man_codex(MAN_CHO));
    assert!(
        la_man_dang_nhap(MAN_CHO),
        "cửa chốt chung cũng nhận màn codex"
    );
    assert_eq!(
        codex_device(MAN_CHO),
        Some((
            "https://auth.openai.com/codex/device".to_string(),
            "HBAC-ERA9M".to_string()
        ))
    );
    // Chưa có kết cục: chỉ còn câu cảnh báo cố định, đã bỏ.
    assert_eq!(codex_after(MAN_CHO), "");
}

#[test]
fn man_chua_in_ma_hoac_ma_la_thi_khong_doan() {
    let chua_ma = MAN_CHO.split("2. Enter").next().unwrap();
    assert_eq!(codex_device(chua_ma), None, "chưa có dòng mã ⟹ None");
    let ma_la = MAN_CHO.replace("HBAC-ERA9M", "please wait");
    assert_eq!(
        codex_device(&ma_la),
        None,
        "dòng sau `one-time code` không phải mã ⟹ None"
    );
    // Màn của claude `auth login` không phải màn codex.
    let claude = "$ claude auth login\nIf the browser didn't open, visit: https://claude.com/x\nPaste code here if prompted >";
    assert!(!la_man_codex(claude));
    assert_eq!(codex_device(claude), None);
}

#[test]
fn ket_cuc_doc_nguyen_van_sau_khoi_ma() {
    let xong = format!("{MAN_CHO}\nSuccessfully logged in\nuser1@user1-ubuntu:~$\n");
    assert_eq!(codex_after(&xong), "Successfully logged in");
}

#[test]
fn tim_tien_trinh_dang_nhap_codex_tren_tty() {
    let ps = "  4101 -bash\n  4180 node /home/u/.npm-global/bin/codex login --device-auth\n  4187 /x/codex login --device-auth\n";
    assert_eq!(login_pid(ps), Some(4180));
    assert_eq!(login_pid("  4101 -bash\n"), None, "CLI đã thoát ⟹ None");
}

#[test]
fn hang_codex_khong_mo_phien_hang_claude_van_mo() {
    let c = cfg_hai();
    let e = ensure_account_usable(&c, Some("acc10"))
        .unwrap_err()
        .to_string();
    assert!(e.contains("acc10") && e.contains("codex"), "{e}");
    assert!(
        e.contains("/accounts <tên> -l"),
        "phải chỉ đường đăng nhập: {e}"
    );
    assert!(ensure_account_usable(&c, Some("acc1")).is_ok(), "vế ngược");
}

#[test]
fn cli_codex_hop_le_cli_la_bi_tu_choi() {
    let c = cfg_hai();
    let a = &c.claude_accounts[1];
    assert!(a.is_codex() && !a.is_claude() && !a.is_opencode());
    assert_eq!(a.cli_name(), "codex");
    assert!(c.claude_accounts[0].is_claude());
    assert!(huba::config::validate(&c).is_ok(), "codex là CLI hợp lệ");

    let mut sai = cfg_hai();
    sai.claude_accounts[1].cli = Some("gemini".into());
    let e = format!("{:?}", huba::config::validate(&sai).unwrap_err());
    assert!(
        e.contains("gemini"),
        "vế ngược: CLI lạ phải bị từ chối: {e}"
    );
}

#[test]
fn dong_lenh_codex_dat_codex_home_rieng() {
    let c = cfg_hai();
    let a = &c.claude_accounts[1];
    let line = huba::accounts::launch_line(&c, Some(a));
    assert!(line.starts_with("CODEX_HOME="), "{line}");
    assert!(line.contains(".claude-acc10"), "{line}");
    assert!(!line.contains("CLAUDE_CONFIG_DIR"), "{line}");
    let login = huba::accounts::codex_login_line(&c, a);
    assert!(login.ends_with(" login --device-auth"), "{login}");
    let env = huba::accounts::env_for_child(a);
    assert_eq!(env.len(), 1);
    assert_eq!(env[0].0, "CODEX_HOME");
}

/// Ca thật 14/11: `/accounts` in acc10 NGAY SAU acc1 (tệp cấu hình xếp theo chữ),
/// không nói nó là codex, còn ghi «opencode» — người đọc báo «chưa thấy tài khoản 10».
#[test]
fn danh_sach_xep_tu_nhien_va_noi_ro_hang_codex() {
    let mut c = common::cfg_for_tests();
    // Đúng thứ tự trong tệp cấu hình đang chạy hôm ấy.
    c.claude_accounts = vec![
        hang("acc1", None),
        hang("acc10", Some("codex")),
        hang("acc2", None),
        hang("acc9", None),
    ];
    let quotas = huba::quota::read_all(&c);
    let s = huba::runtime::accounts_text(
        &c,
        &huba::sessions::SessionsSnapshot::default(),
        &serde_json::json!({}),
        &quotas,
        &Default::default(),
        0,
    );
    let vi = |t: &str| {
        s.find(&format!("\n{t} "))
            .unwrap_or_else(|| panic!("thiếu {t}:\n{s}"))
    };
    assert!(
        vi("acc1") < vi("acc2") && vi("acc2") < vi("acc9") && vi("acc9") < vi("acc10"),
        "{s}"
    );
    let khuc = &s[vi("acc10")..];
    assert!(khuc.contains("codex (ChatGPT)"), "{s}");
    assert!(
        !khuc.contains("opencode"),
        "hàng codex không được mang nhãn opencode:\n{s}"
    );
    assert!(
        !khuc.contains("không có phiên nào đang chạy"),
        "phiên codex chưa đo ⟹ không khai 0:\n{s}"
    );
    // Vế ngược: hàng claude vẫn đi câu cũ.
    assert!(
        s[vi("acc2")..vi("acc9")].contains("không có phiên nào đang chạy"),
        "{s}"
    );
}

#[test]
fn ten_tu_nhien_so_khuc_so_theo_gia_tri() {
    use huba::runtime::ten_tu_nhien as k;
    assert!(k("acc2") < k("acc10"));
    assert!(k("acc1") < k("acc2"));
    assert!(k("acc") < k("acc1"));
    assert!(k("acc9") < k("acc10"));
}
