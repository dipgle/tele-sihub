//! Tường của phiên codex (`accounts::ensure_codex_wall`) — luật 1, bản codex.
//!
//! Tệp RIÊNG vì bài này đổi biến môi trường của cả tiến trình
//! (`HUBA_OPENCODE_PLUGIN` quyết đường tới cổng) — chung tệp với bài khác là
//! chạy song song trên cùng một môi trường.
//!
//! Hai chiều, cùng một hàm: cổng THẬT (`~/projects/scripts/cong_bao_mat.py` của
//! cây này) ⟹ dựng được tường, `hooks.json` trỏ đúng cổng; cổng GIẢ luôn cho qua
//! ⟹ PHẢI từ chối mở (một tường không chặn được gì không phải tường).

use huba::config::ClaudeAccountCfg;

fn hang(home: &std::path::Path) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: "acc-thu".into(),
        config_dir: Some(home.display().to_string()),
        cli: Some("codex".into()),
        ..Default::default()
    }
}

#[test]
fn tuong_codex_hai_chieu() {
    let goc = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("scripts");
    let cong_that = goc.join("cong_bao_mat.py");
    if !cong_that.is_file() {
        // Không phải "xanh": máy này không có cây workspace để đo.
        panic!("KHÔNG ĐO ĐƯỢC: thiếu {}", cong_that.display());
    }
    let d = tempfile::tempdir().unwrap();
    let home = d.path().join("codex-home");
    std::fs::create_dir_all(&home).unwrap();
    let acc = hang(&home);

    // Chưa đăng nhập ⟹ không mở (không có auth.json).
    std::env::set_var("HUBA_OPENCODE_PLUGIN", goc.join("cong-bao-mat-plugin"));
    let e = huba::accounts::ensure_codex_wall(&acc).unwrap_err();
    assert!(e.contains("chưa đăng nhập"), "{e}");

    // Tệp đánh dấu đăng nhập (RỖNG — không phải một khoá thật).
    std::fs::write(home.join("auth.json"), "").unwrap();
    let luat = d.path().join("CLAUDE.md");
    std::fs::write(&luat, "# luật thử\n").unwrap();
    std::env::set_var("HUBA_LUAT_GOC", &luat);
    let ok = huba::accounts::ensure_codex_wall(&acc).expect("cổng thật phải dựng được tường");
    let h = std::fs::read_to_string(home.join("hooks.json")).unwrap();
    assert!(
        h.contains("SessionStart") && h.contains("huba-nap-luat.py"),
        "{h}"
    );
    assert!(home.join("huba-nap-luat.py").is_file());
    std::env::remove_var("HUBA_LUAT_GOC");
    assert!(ok.contains("workspace-write"), "{ok}");
    let hooks = std::fs::read_to_string(home.join("hooks.json")).unwrap();
    assert!(hooks.contains("--codex-pretooluse"), "{hooks}");
    assert!(hooks.contains("cong_bao_mat.py"), "{hooks}");
    assert!(hooks.contains("PreToolUse"), "{hooks}");

    // ĐỐI CHỨNG NGƯỢC: cổng giả LUÔN cho qua ⟹ phải từ chối mở.
    let gia = d.path().join("gia");
    std::fs::create_dir_all(gia.join("cong-bao-mat-plugin")).unwrap();
    std::fs::write(gia.join("cong_bao_mat.py"), "import sys\nsys.exit(0)\n").unwrap();
    std::env::set_var("HUBA_OPENCODE_PLUGIN", gia.join("cong-bao-mat-plugin"));
    let e = huba::accounts::ensure_codex_wall(&acc).unwrap_err();
    assert!(
        e.contains("KHÔNG mở phiên"),
        "cổng luôn-cho-qua phải bị bắt: {e}"
    );

    // …và cổng giả CHẶN HẾT cũng bị bắt (lệnh thường phải qua).
    std::fs::write(gia.join("cong_bao_mat.py"), "import sys\nsys.exit(2)\n").unwrap();
    let e = huba::accounts::ensure_codex_wall(&acc).unwrap_err();
    assert!(
        e.contains("KHÔNG mở phiên"),
        "cổng chặn-hết phải bị bắt: {e}"
    );
    std::env::remove_var("HUBA_OPENCODE_PLUGIN");
}

#[test]
fn tin_cay_thu_muc_ghi_mot_lan_khong_de_quyet_dinh_cu() {
    use huba::accounts::codex_trust_block as k;
    // Nguyên văn config.toml của acc10 lúc 14:4x 14/11.
    let that = "[tui]\nscreen_reader_detection_done = true\n\n[projects.\"/mnt/data/projects\"]\ntrust_level = \"trusted\"\n";
    let them = k(that, "/mnt/data/projects/bts").expect("thư mục mới ⟹ thêm");
    assert!(
        them.contains("[projects.\"/mnt/data/projects/bts\"]"),
        "{them}"
    );
    assert!(them.contains("trust_level = \"trusted\""), "{them}");
    // Đã có bảng của thư mục ⟹ KHÔNG ghi (kể cả khi chủ máy chọn untrusted).
    assert_eq!(k(that, "/mnt/data/projects"), None);
    let tu_choi = "[projects.\"/x\"]\ntrust_level = \"untrusted\"\n";
    assert_eq!(k(tu_choi, "/x"), None);
    // Tệp không kết thúc bằng xuống dòng ⟹ không dính khối vào dòng cuối.
    assert!(k("a = 1", "/y").unwrap().starts_with('\n'));
    // Dấu nháy trong đường dẫn được thoát.
    assert!(k("", "/a\"b").unwrap().contains("[projects.\"/a\\\"b\"]"));
}

/// Dựng tường THẬT cho `~/.codex-acc10` bằng đúng cổng + luật của bản cài — chạy tay:
/// `cargo test --test tuong_codex dung_that -- --ignored --nocapture`
#[test]
#[ignore = "ghi hooks.json của tài khoản thật — chạy tay"]
fn dung_that_acc10() {
    let home = huba::config::home_dir().unwrap();
    std::env::set_var(
        "HUBA_OPENCODE_PLUGIN",
        home.join("huba/kem/scripts/cong-bao-mat-plugin"),
    );
    std::env::set_var("HUBA_LUAT_GOC", "/mnt/data/projects/CLAUDE.md");
    let acc = ClaudeAccountCfg {
        name: "acc10".into(),
        config_dir: Some(home.join(".codex-acc10").display().to_string()),
        cli: Some("codex".into()),
        ..Default::default()
    };
    println!("{}", huba::accounts::ensure_codex_wall(&acc).unwrap());
}

/// Script `SessionStart`: đứng NGOÀI kho gốc ⟹ đưa luật vào ngữ cảnh; đứng ĐÚNG kho
/// gốc (codex đã tự đọc `CLAUDE.md`) ⟹ không nạp lần hai.
#[test]
fn nap_luat_goc_hai_chieu() {
    let d = tempfile::tempdir().unwrap();
    let goc = d.path().join("ws");
    let khac = d.path().join("khac");
    std::fs::create_dir_all(&goc).unwrap();
    std::fs::create_dir_all(&khac).unwrap();
    let git = |dir: &std::path::Path| {
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(dir)
            .status()
            .unwrap()
    };
    git(&goc);
    git(&khac);
    let luat = goc.join("CLAUDE.md");
    std::fs::write(&luat, "LUAT-THU-0714\n").unwrap();
    let script = d.path().join("nap.py");
    std::fs::write(&script, huba::accounts::CODEX_NAP_LUAT_PY).unwrap();
    let chay = |cwd: &std::path::Path| -> serde_json::Value {
        use std::io::Write;
        let mut c = std::process::Command::new("python3")
            .arg(&script)
            .arg(&luat)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        c.stdin
            .take()
            .unwrap()
            .write_all(serde_json::json!({ "cwd": cwd }).to_string().as_bytes())
            .unwrap();
        let o = c.wait_with_output().unwrap();
        assert!(o.status.success());
        serde_json::from_slice(&o.stdout).unwrap()
    };
    let ngoai = chay(&khac);
    let ctx = ngoai["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap_or("");
    assert!(ctx.contains("LUAT-THU-0714"), "{ngoai}");
    assert_eq!(ngoai["hookSpecificOutput"]["hookEventName"], "SessionStart");
    let trong = chay(&goc);
    assert!(
        !trong["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap_or("")
            .contains("LUAT-THU-0714"),
        "đứng đúng kho gốc ⟹ không nạp luật lần hai: {trong}"
    );
}

/// Quang 14/11: *«mở phiên mới bằng acc10 vẫn chưa có nội dung đầu giống các tài
/// khoản khác»*. Phiên Claude mở ra luôn có khối KHỞI ĐỘNG PHIÊN — hook SessionStart
/// của workspace `bash ~/projects/scripts/phien.sh main` (`.claude/settings.json`).
/// codex chạy ĐÚNG lệnh ấy (cạnh `CLAUDE.md`), ở `cwd` của phiên, kể cả ở gốc; hỏng
/// thì nói ĐÚNG câu phiên Claude nói, không im.
#[test]
fn khoi_dong_phien_nhu_claude() {
    let d = tempfile::tempdir().unwrap();
    let goc = d.path().join("ws");
    std::fs::create_dir_all(goc.join("scripts")).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&goc)
        .status()
        .unwrap();
    let luat = goc.join("CLAUDE.md");
    std::fs::write(&luat, "LUAT\n").unwrap();
    let khoi = goc.join("scripts").join("phien.sh");
    let script = d.path().join("nap.py");
    std::fs::write(&script, huba::accounts::CODEX_NAP_LUAT_PY).unwrap();
    let chay = || -> String {
        use std::io::Write;
        let mut c = std::process::Command::new("python3")
            .arg(&script)
            .arg(&luat)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        c.stdin
            .take()
            .unwrap()
            .write_all(serde_json::json!({ "cwd": goc }).to_string().as_bytes())
            .unwrap();
        let o = c.wait_with_output().unwrap();
        assert!(o.status.success());
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        v["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    std::fs::write(&khoi, "echo \"KHOI-DONG-THU $1 $(pwd)\"\n").unwrap();
    let ctx = chay();
    assert!(
        ctx.contains(&format!("KHOI-DONG-THU main {}", goc.display())),
        "{ctx}"
    );
    std::fs::write(&khoi, "echo nua-chung; exit 3\n").unwrap();
    let ctx = chay();
    assert!(ctx.contains("HOOK SessionStart HỎNG (mã 3)"), "{ctx}");
    assert!(ctx.contains("nua-chung"), "giữ phần đã in: {ctx}");
    std::fs::remove_file(&khoi).unwrap();
    assert!(
        chay().contains("HOOK SessionStart HỎNG"),
        "thiếu script ⟹ nói ra"
    );
}

#[test]
fn mcp_codetrail_cho_codex() {
    use huba::accounts::codex_mcp_codetrail_args_cho as k;
    assert!(
        k("acc10", "").is_empty(),
        "gói không khai binary ⟹ không gắn"
    );
    let d = tempfile::tempdir().unwrap();
    let bin = d.path().join("project-agent");
    std::fs::write(&bin, "").unwrap();
    let a = k("acc10", &bin.to_string_lossy());
    assert_eq!(a.len(), 2, "{a:?}");
    assert_eq!(
        a[0],
        format!("mcp_servers.project-agent.command=\"{}\"", bin.display())
    );
    assert!(
        a[1].starts_with("mcp_servers.project-agent.env={PROJECTS_ROOT=\""),
        "{a:?}"
    );
}

#[test]
fn dong_lenh_codex_mang_tuong() {
    let c = huba::config::Config::default();
    let d = tempfile::tempdir().unwrap();
    let line = huba::accounts::launch_line(&c, Some(&hang(d.path())));
    for f in huba::accounts::CODEX_WALL_FLAGS {
        assert!(line.contains(f), "thiếu cờ tường `{f}`: {line}");
    }
    assert!(!line.contains("danger-full-access"), "{line}");
    // Effort mặc định high (Quang 14/11), ô `effort` của hàng thắng.
    assert!(line.contains("model_reasoning_effort=\"high\""), "{line}");
    let mut a = hang(d.path());
    a.effort = Some("medium".into());
    let l2 = huba::accounts::launch_line(&c, Some(&a));
    assert!(l2.contains("model_reasoning_effort=\"medium\""), "{l2}");
    assert!(!l2.contains("\"high\""), "{l2}");
    assert!(
        !line.contains("--dangerously-bypass-approvals-and-sandbox"),
        "{line}"
    );
}

/// Luật 1 bản codex (đo 14/11, codex-cli 0.169.0, hai chiều trên lượt thật):
/// mặc định codex TỰ tìm web («Searched the web» trên phiên pts/11; `exec` trả
/// «ripgrep 15.2.0») — `-c web_search="disabled"` ⟹ «KHONG_CO_WEB». MCP mặc định
/// `codex_apps` (308 công cụ connector ChatGPT) mất khi `-c features.apps=false`.
/// Hai cờ đứng SAU `extra_args` của hàng — ô cấu hình không mở lại được tường.
#[test]
fn dong_lenh_codex_tat_web_va_apps_sau_extra_args() {
    let c = huba::config::Config::default();
    let d = tempfile::tempdir().unwrap();
    let mut a = hang(d.path());
    a.extra_args = vec!["-c".into(), "web_search=\"live\"".into()];
    let line = huba::accounts::launch_line(&c, Some(&a));
    let tat_web = line
        .rfind("-c 'web_search=\"disabled\"'")
        .unwrap_or_else(|| panic!("thiếu tắt web: {line}"));
    let mo_web = line
        .find("web_search=\"live\"")
        .expect("extra_args phải còn");
    assert!(tat_web > mo_web, "tường phải đứng SAU extra_args: {line}");
    let tat_apps = line
        .rfind("-c 'features.apps=false'")
        .unwrap_or_else(|| panic!("thiếu tắt codex_apps: {line}"));
    assert!(tat_apps > mo_web, "{line}");
}
