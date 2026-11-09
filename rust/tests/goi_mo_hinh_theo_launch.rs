//! Lượt gọi `claude` CÓ DÙNG MÔ HÌNH (fork của `/ask`·`/handover`, phiên nền
//! `--bg`) phải đi qua dòng `launch` của tài khoản khi dòng ấy là một DÒNG LỆNH
//! đủ — không chỉ mượn mỗi `CLAUDE_CONFIG_DIR`.
//!
//! 🔴 07/11 (Quang: *"Vá luôn nợ fork thiếu env acc8/acc9 đi"*). Đo trên
//! `huba.config.json` cùng ngày: acc8 `launch` = `ANTHROPIC_BASE_URL=…11434
//! ANTHROPIC_API_KEY=… CLAUDE_CONFIG_DIR=… claude --model …` (ollama), acc9 =
//! `env $(cat …env | xargs) ANTHROPIC_BASE_URL=… claude --model …`. `fork_call`
//! và `start_background` gọi thẳng `cfg.claude_cli` với đúng một biến
//! `CLAUDE_CONFIG_DIR` ⟹ máy chủ, khoá và `--model` của tài khoản rơi mất, lượt
//! gọi đi nhầm sang Anthropic bằng một thư mục chưa đăng nhập.
//!
//! Đối chứng: `launch` MỘT TỪ (`claude3` — alias trong `~/.zshrc`) giữ đường
//! cũ, vì `zsh -c` không nạp alias; đi shell là gãy một tài khoản đang chạy tốt.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use huba::config::{ClaudeAccountCfg, Config};
use huba::sessions::{self, LiveSession};

const ID: &str = "4e5f6a7b-0000-4000-8000-0000000000aa";

/// `claude` giả: ghi lại biến thử + từng đối số (mỗi dòng một cái) rồi trả một
/// phong bì JSON đúng hình `claude -p --output-format json`.
fn claude_gia(dir: &Path) -> PathBuf {
    let p = dir.join("claude-gia");
    std::fs::write(
        &p,
        format!(
            "#!/bin/sh\n\
             {{ echo \"env=${{HUBA_THU_MAY:-}}\"; for a in \"$@\"; do echo \"arg=$a\"; done; }} > '{}'\n\
             cat > /dev/null\n\
             echo '{{\"session_id\":\"moi-1\",\"result\":\"ban giao qua '\"${{HUBA_THU_MAY:-truc-tiep}}\"'\",\"total_cost_usd\":0}}'\n",
            dir.join("goi.txt").display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p
}

fn acc(name: &str, dir: &Path, launch: Option<String>) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: name.into(),
        config_dir: Some(dir.to_string_lossy().into_owned()),
        launch,
        ..Default::default()
    }
}

struct May {
    tmp: tempfile::TempDir,
    cfg: Config,
}

fn may() -> May {
    let tmp = tempfile::tempdir().unwrap();
    let gia = claude_gia(tmp.path());
    let cfg = Config {
        // Đường TRỰC TIẾP cũng chạm `claude` giả — nên bài đỏ vì THIẾU biến,
        // không phải vì không có chương trình nào để gọi.
        claude_cli: gia.to_string_lossy().into_owned(),
        claude_transcript_root: tmp.path().join("chung").to_string_lossy().into_owned(),
        claude_accounts: vec![
            acc(
                "acc8",
                &tmp.path().join("acc8"),
                Some(format!(
                    "HUBA_THU_MAY=launch-acc8 {} --model mo-hinh-acc8",
                    gia.display()
                )),
            ),
            acc("acc3", &tmp.path().join("acc3"), Some("claude3".into())),
        ],
        ..Default::default()
    };
    May { tmp, cfg }
}

fn phien(m: &May, account: &str) -> LiveSession {
    LiveSession {
        session_id: ID.into(),
        account: account.into(),
        cwd: m.tmp.path().to_string_lossy().into_owned(),
        name: "thu".into(),
        ..Default::default()
    }
}

fn da_goi(m: &May) -> Vec<String> {
    std::fs::read_to_string(m.tmp.path().join("goi.txt"))
        .expect("claude giả chưa được gọi")
        .lines()
        .map(str::to_string)
        .collect()
}

// ── Ca hỏng: tài khoản có `launch` là dòng lệnh đủ ────────────────────────────

#[test]
fn ban_giao_tren_acc8_di_qua_dong_launch() {
    let m = may();
    let h = sessions::handover(&m.cfg, &phien(&m, "acc8")).expect("handover");
    let goi = da_goi(&m);
    assert_eq!(
        goi[0], "env=launch-acc8",
        "biến của launch rơi mất: {goi:?}"
    );
    assert_eq!(
        goi[1..3],
        ["arg=--model".to_string(), "arg=mo-hinh-acc8".to_string()],
        "đối số của launch rơi mất: {goi:?}"
    );
    assert!(h.checkpoint.contains("launch-acc8"), "{}", h.checkpoint);
}

/// Đi qua launch KHÔNG được làm rơi rào của fork (luật 2): vẫn `--fork-session`,
/// vẫn chỉ `Read,Grep,Glob`.
#[test]
fn qua_launch_van_giu_rao_fork() {
    let m = may();
    sessions::handover(&m.cfg, &phien(&m, "acc8")).expect("handover");
    let goi = da_goi(&m);
    for can in [
        "arg=--fork-session",
        "arg=Read,Grep,Glob",
        &format!("arg={ID}"),
    ] {
        assert!(goi.iter().any(|l| l == can), "thiếu {can}: {goi:?}");
    }
}

/// Đối số có khoảng trắng · nháy · `$(…)` · `$HOME` đi tới CLI NGUYÊN VĂN —
/// shell chỉ giãn dòng `launch` (của chủ máy), không bao giờ giãn chữ gõ từ điện
/// thoại.
#[test]
fn doi_so_qua_shell_di_nguyen_van() {
    let m = may();
    let doc = m.tmp.path().join("khong-duoc-tao");
    let ac = format!("it's \"x\" $HOME $(touch {}) `id`", doc.display());
    let (prog, argv) = sessions::account_call(&m.cfg, Some("acc8"), &["--bg", &ac]);
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let out = huba::exec::run(&prog, &argv, Default::default()).expect("chạy");
    assert_eq!(out.code, Some(0), "{}", out.stderr);
    let goi = da_goi(&m);
    assert!(
        goi.contains(&format!("arg={ac}")),
        "đối số bị giãn: {goi:?}"
    );
    assert!(!doc.exists(), "$(…) trong đối số đã bị CHẠY");
}

// ── Cùng lớp: `auth status` (`/accounts -k` + khối tài khoản) · dò `/usage` ────
//
// 🔴 07/11 (Quang: *"Nối luôn /usage và /accounts -k qua launch đi"*). Đo trên CLI
// thật cùng ngày: acc8 đường TRỰC TIẾP ⟹ `auth status` = `loggedIn:false
// authMethod:none`; qua `launch` ⟹ `loggedIn:true · api_key · ANTHROPIC_API_KEY`.
// `-p /usage` qua `launch`: mã 0, `num_turns 0`, `total_cost_usd 0` (không tốn).

fn acc8(m: &May) -> ClaudeAccountCfg {
    m.cfg
        .claude_accounts
        .iter()
        .find(|a| a.name == "acc8")
        .unwrap()
        .clone()
}

#[test]
fn auth_status_cua_accounts_k_di_qua_launch() {
    let m = may();
    let v = huba::api_key::auth_status(&m.cfg, &acc8(&m)).expect("auth status");
    let goi = da_goi(&m);
    assert_eq!(
        goi[0], "env=launch-acc8",
        "biến của launch rơi mất: {goi:?}"
    );
    assert_eq!(
        goi[1..],
        [
            "arg=--model".to_string(),
            "arg=mo-hinh-acc8".to_string(),
            "arg=auth".to_string(),
            "arg=status".to_string()
        ],
        "{goi:?}"
    );
    assert!(v.is_object(), "{v}");
}

#[test]
fn do_usage_di_qua_launch() {
    let m = may();
    let db = huba::db::Db::open(&m.tmp.path().join("huba.sqlite")).unwrap();
    huba::runtime::usage_lam_moi_mot(&m.cfg, &db, "acc8", 0);
    let goi = da_goi(&m);
    assert_eq!(
        goi[0], "env=launch-acc8",
        "biến của launch rơi mất: {goi:?}"
    );
    assert!(
        goi.windows(2)
            .any(|w| w == ["arg=-p".to_string(), "arg=/usage".to_string()]),
        "{goi:?}"
    );
}

// ── Đối chứng: launch MỘT TỪ (alias) giữ đường trực tiếp ──────────────────────

#[test]
fn launch_mot_tu_giu_duong_truc_tiep() {
    let m = may();
    let (prog, argv) = sessions::account_call(&m.cfg, Some("acc3"), &["-p", "x"]);
    assert_eq!(prog, m.cfg.claude_cli);
    assert_eq!(argv, vec!["-p".to_string(), "x".to_string()]);
    let h = sessions::handover(&m.cfg, &phien(&m, "acc3")).expect("handover");
    assert_eq!(da_goi(&m)[0], "env=", "alias không mang biến thử");
    assert!(h.checkpoint.contains("truc-tiep"), "{}", h.checkpoint);
}
