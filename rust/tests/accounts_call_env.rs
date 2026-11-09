//! Dòng lệnh DỰNG từ trường cấu trúc phải mang đúng môi trường tới tiến trình
//! con — khoá hồi quy cho nợ "fork/`--bg` rơi env" (vá 07/11, nhưng lúc ấy chỉ
//! có đường `launch` viết tay).
//!
//! Bài kiểm chạy THẬT `/bin/zsh -c '<line> "$@"'` với một `claude` giả: đây là
//! đúng đường `sessions::account_call` sinh ra, nên nếu `env_for_child` hay
//! `launch_line` quên một biến thì tiến trình con thấy biến rỗng.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use huba::config::{ClaudeAccountCfg, Config};

/// `claude` giả: in `CLAUDE_CONFIG_DIR|ANTHROPIC_BASE_URL|VUI|<argv…>` một dòng.
fn claude_gia(dir: &Path) -> PathBuf {
    let p = dir.join("claude-gia");
    std::fs::write(
        &p,
        "#!/bin/sh\n\
         echo \"${CLAUDE_CONFIG_DIR:-}|${ANTHROPIC_BASE_URL:-}|${VUI:-}|$*\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p
}

/// Chạy đúng `(prog, argv)` mà `account_call` trả về.
///
/// 🔴 Xoá ba biến trước khi chạy: chính TIẾN TRÌNH CHẠY BÀI KIỂM có thể đang
/// mang acc (phiên viết bài này chạy acc9 ⇒ `CLAUDE_CONFIG_DIR` +
/// `ANTHROPIC_BASE_URL` có thật trong env). Đo ở đây là "dòng lệnh đặt được
/// gì", không phải "con thừa hưởng gì" — để lẫn hai thứ thì bài kiểm xanh/đỏ
/// theo cái máy đang chạy nó.
fn chay(prog: &str, argv: &[String]) -> String {
    let out = Command::new(prog)
        .args(argv)
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("ANTHROPIC_BASE_URL")
        .env_remove("VUI")
        .output()
        .expect("chạy zsh");
    assert!(
        out.status.success(),
        "mã {:?} · stderr {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn dong_cau_truc_mang_env_va_thu_tu_doi_so_tới_tien_trinh_con() {
    let tmp = tempfile::tempdir().unwrap();
    let gia = claude_gia(tmp.path());
    let mut acc = ClaudeAccountCfg {
        name: "moi".into(),
        config_dir: Some(tmp.path().join("acc-moi").display().to_string()),
        base_url: Some("https://vd.ai/go/anthropic".into()),
        ..Default::default()
    };
    acc.extra_env.insert("VUI".into(), "1".into());
    let cfg = Config {
        claude_cli: gia.display().to_string(),
        claude_accounts: vec![acc.clone()],
        ..Default::default()
    };

    let (prog, argv) = huba::sessions::account_call(&cfg, Some("moi"), &["-p", "xin chào"]);
    assert_eq!(prog, "/bin/zsh", "dòng nhiều từ phải đi qua shell");
    let got = chay(&prog, &argv);
    let duong_dan = tmp.path().join("acc-moi").display().to_string();
    assert_eq!(
        got,
        format!("{duong_dan}|https://vd.ai/go/anthropic|1|-p xin chào"),
        "env hoặc đối số rơi mất"
    );
}

#[test]
fn dong_chi_co_config_dir_van_qua_shell_va_co_env() {
    let tmp = tempfile::tempdir().unwrap();
    let gia = claude_gia(tmp.path());
    let acc = ClaudeAccountCfg {
        name: "acc4".into(),
        config_dir: Some(tmp.path().join("acc4").display().to_string()),
        ..Default::default()
    };
    let cfg = Config {
        claude_cli: gia.display().to_string(),
        claude_accounts: vec![acc],
        ..Default::default()
    };
    let (prog, argv) = huba::sessions::account_call(&cfg, Some("acc4"), &["auth", "status"]);
    assert_eq!(prog, "/bin/zsh");
    let got = chay(&prog, &argv);
    let duong_dan = tmp.path().join("acc4").display().to_string();
    assert_eq!(got, format!("{duong_dan}|||auth status"));
}

#[test]
fn tai_khoan_mac_dinh_giu_duong_truc_tiep() {
    let tmp = tempfile::tempdir().unwrap();
    let gia = claude_gia(tmp.path());
    let cfg = Config {
        claude_cli: gia.display().to_string(),
        ..Default::default()
    };
    let (prog, argv) = huba::sessions::account_call(&cfg, None, &["-p", "x"]);
    assert_eq!(prog, gia.display().to_string(), "mặc định KHÔNG qua shell");
    assert_eq!(argv, vec!["-p".to_string(), "x".to_string()]);
    // Và môi trường của nó là "không đặt gì" — `CLAUDE_CONFIG_DIR` = None.
    let env = huba::runtime::account_env(&ClaudeAccountCfg {
        name: "mặc định".into(),
        ..Default::default()
    });
    assert_eq!(env, vec![("CLAUDE_CONFIG_DIR".to_string(), None)]);
}

#[test]
fn launch_legacy_van_thang_o_tang_call() {
    let tmp = tempfile::tempdir().unwrap();
    let gia = claude_gia(tmp.path());
    let acc = ClaudeAccountCfg {
        name: "acc9".into(),
        config_dir: Some(tmp.path().join("acc9").display().to_string()),
        launch: Some(format!("VUI=legacy {}", gia.display())),
        model: Some("deepseek-v4.1-flash".into()),
        ..Default::default()
    };
    let cfg = Config {
        claude_cli: gia.display().to_string(),
        claude_accounts: vec![acc],
        ..Default::default()
    };
    let (prog, argv) = huba::sessions::account_call(&cfg, Some("acc9"), &["-p"]);
    assert_eq!(prog, "/bin/zsh");
    let got = chay(&prog, &argv);
    // `launch` không đặt CLAUDE_CONFIG_DIR/base_url; `model` bị CHE ⇒ không có
    // `--model` trên dòng chạy. Đúng luật di trú: dòng viết tay thắng.
    assert_eq!(got, "||legacy|-p");
}
