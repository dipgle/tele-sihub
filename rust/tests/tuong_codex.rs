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

/// MCP giả trả đúng một công cụ (khuôn stdio JSON-RPC của project-agent).
const MCP_GIA: &str = "#!/usr/bin/env python3\nimport sys,json\nfor l in sys.stdin:\n o=json.loads(l)\n if o.get('id')==1: print(json.dumps({'jsonrpc':'2.0','id':1,'result':{}}),flush=True)\n if o.get('id')==2: print(json.dumps({'jsonrpc':'2.0','id':2,'result':{'tools':[{'name':'get_context_brief'}]}}),flush=True)\n";

/// Gốc workspace GIẢ đủ khâu bắt buộc: `CLAUDE.md` · `scripts/phien.sh` · `.mcp.json`
/// trỏ MCP giả. KHÔNG có `mo-dau-phien.py` trong gốc — khâu chung nằm CẠNH cổng
/// (máy cài bằng gói Linux không có `<gốc>/scripts/` của workspace).
fn goc_gia(d: &std::path::Path) -> std::path::PathBuf {
    let goc = d.join("ws");
    std::fs::create_dir_all(goc.join("scripts")).unwrap();
    std::fs::write(goc.join("CLAUDE.md"), "LUAT-THU-0714\n").unwrap();
    std::fs::write(
        goc.join("scripts").join("phien.sh"),
        "echo \"KHOI-DONG-THU $1 $(pwd)\"\n",
    )
    .unwrap();
    let mcp = d.join("mcp-gia");
    std::fs::write(&mcp, MCP_GIA).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&mcp, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(
        goc.join(".mcp.json"),
        serde_json::json!({ "mcpServers": { "project-agent": { "command": mcp } } }).to_string(),
    )
    .unwrap();
    goc
}

#[test]
fn tuong_codex_hai_chieu() {
    let goc_that = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("scripts");
    let cong_that = goc_that.join("cong_bao_mat.py");
    if !cong_that.is_file() {
        // Không phải "xanh": máy này không có cây workspace để đo.
        panic!("KHÔNG ĐO ĐƯỢC: thiếu {}", cong_that.display());
    }
    std::env::remove_var("HUBA_MCP_PROJECT_AGENT");
    let d = tempfile::tempdir().unwrap();
    let goc = goc_gia(d.path());
    // Tên thư mục mang `accN` ⟹ khâu 1 xác định được tài khoản (gốc giả không có huba.config.json).
    let home = d.path().join("codex-acc77");
    std::fs::create_dir_all(&home).unwrap();
    let acc = hang(&home);

    // Chưa đăng nhập ⟹ không mở (không có auth.json).
    std::env::set_var("HUBA_OPENCODE_PLUGIN", goc_that.join("cong-bao-mat-plugin"));
    let e = huba::accounts::ensure_codex_wall(&acc, &goc).unwrap_err();
    assert!(e.contains("chưa đăng nhập"), "{e}");

    // Tệp đánh dấu đăng nhập (RỖNG — không phải một khoá thật).
    std::fs::write(home.join("auth.json"), "").unwrap();
    let ok = huba::accounts::ensure_codex_wall(&acc, &goc).expect("đủ khâu ⟹ dựng được tường");
    assert!(ok.contains("workspace-write"), "{ok}");
    assert!(ok.contains("TÀI KHOẢN PHIÊN: acc77 · codex"), "{ok}");
    assert!(ok.contains("MCP project-agent: ✓ 1 công cụ"), "{ok}");
    let hooks = std::fs::read_to_string(home.join("hooks.json")).unwrap();
    assert!(hooks.contains("--codex-pretooluse"), "{hooks}");
    assert!(hooks.contains("cong_bao_mat.py"), "{hooks}");
    assert!(hooks.contains("PreToolUse"), "{hooks}");
    assert!(
        hooks.contains("SessionStart")
            && hooks.contains("mo-dau-phien.py")
            && hooks.contains("--codex-hook"),
        "{hooks}"
    );
    assert!(!home.join("huba-nap-luat.py").exists());

    // ĐỐI CHỨNG NGƯỢC — MCP là BẮT BUỘC: binary khai trong .mcp.json biến mất ⟹ không mở.
    let mcp = d.path().join("mcp-gia");
    std::fs::rename(&mcp, d.path().join("mcp-cat")).unwrap();
    let e = huba::accounts::ensure_codex_wall(&acc, &goc).unwrap_err();
    assert!(e.contains("KHÔNG mở phiên") && e.contains("MCP"), "{e}");
    std::fs::rename(d.path().join("mcp-cat"), &mcp).unwrap();

    // …luật gốc là BẮT BUỘC: thiếu CLAUDE.md ⟹ không mở.
    std::fs::rename(goc.join("CLAUDE.md"), goc.join("CLAUDE.cat")).unwrap();
    let e = huba::accounts::ensure_codex_wall(&acc, &goc).unwrap_err();
    assert!(
        e.contains("KHÔNG mở phiên") && e.contains("luật chung"),
        "{e}"
    );
    std::fs::rename(goc.join("CLAUDE.cat"), goc.join("CLAUDE.md")).unwrap();

    // Máy cài bằng gói không có `phien.sh` ⟹ VẪN mở (khối khởi động không bắt buộc;
    // luật gốc + MCP mới bắt buộc).
    std::fs::remove_file(goc.join("scripts").join("phien.sh")).unwrap();
    huba::accounts::ensure_codex_wall(&acc, &goc).expect("thiếu phien.sh vẫn mở");
    std::fs::write(
        goc.join("scripts").join("phien.sh"),
        "echo \"KHOI-DONG-THU $1 $(pwd)\"\n",
    )
    .unwrap();

    // …không biết gốc workspace (cấu hình rỗng) ⟹ không mở.
    let e = huba::accounts::ensure_codex_wall(&acc, std::path::Path::new("")).unwrap_err();
    assert!(e.contains("KHÔNG mở phiên"), "{e}");

    // ĐỐI CHỨNG NGƯỢC: cổng giả LUÔN cho qua ⟹ phải từ chối mở.
    let gia = d.path().join("gia");
    std::fs::create_dir_all(gia.join("cong-bao-mat-plugin")).unwrap();
    std::fs::write(gia.join("cong_bao_mat.py"), "import sys\nsys.exit(0)\n").unwrap();
    // Khâu mở đầu ĐỦ cạnh cổng giả ⟹ thứ bị bắt dưới đây đúng là CỔNG, không phải khâu mở đầu.
    std::fs::copy(
        goc_that.join("mo-dau-phien.py"),
        gia.join("mo-dau-phien.py"),
    )
    .unwrap();
    std::env::set_var("HUBA_OPENCODE_PLUGIN", gia.join("cong-bao-mat-plugin"));
    let e = huba::accounts::ensure_codex_wall(&acc, &goc).unwrap_err();
    assert!(
        e.contains("cổng bảo mật trả sai") && e.contains("KHÔNG mở phiên"),
        "cổng luôn-cho-qua phải bị bắt: {e}"
    );

    // …và cổng giả CHẶN HẾT cũng bị bắt (lệnh thường phải qua).
    std::fs::write(gia.join("cong_bao_mat.py"), "import sys\nsys.exit(2)\n").unwrap();
    let e = huba::accounts::ensure_codex_wall(&acc, &goc).unwrap_err();
    assert!(
        e.contains("cổng bảo mật trả sai") && e.contains("KHÔNG mở phiên"),
        "cổng chặn-hết phải bị bắt: {e}"
    );
    // …và cổng ĐÚNG mà thiếu khâu mở đầu cạnh nó (gói thiếu tệp) ⟹ không mở.
    std::fs::copy(&cong_that, gia.join("cong_bao_mat.py")).unwrap();
    std::fs::remove_file(gia.join("mo-dau-phien.py")).unwrap();
    let e = huba::accounts::ensure_codex_wall(&acc, &goc).unwrap_err();
    assert!(
        e.contains("khâu soạn đoạn mở đầu") && e.contains("KHÔNG mở phiên"),
        "{e}"
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

/// Dựng tường THẬT cho `~/.codex-acc10` bằng đúng cổng + khâu mở đầu của cây này — chạy tay:
/// `cargo test --test tuong_codex dung_that -- --ignored --nocapture`
#[test]
#[ignore = "ghi hooks.json của tài khoản thật — chạy tay"]
fn dung_that_acc10() {
    let home = huba::config::home_dir().unwrap();
    let goc = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap();
    let acc = ClaudeAccountCfg {
        name: "acc10".into(),
        config_dir: Some(home.join(".codex-acc10").display().to_string()),
        cli: Some("codex".into()),
        ..Default::default()
    };
    println!("{}", huba::accounts::ensure_codex_wall(&acc, &goc).unwrap());
}

/// Chạy khâu chung ở chế độ hook codex, `CODEX_HOME` = `home`, phiên đứng ở `cwd`.
fn hook_mo_dau(
    goc: &std::path::Path,
    home: &std::path::Path,
    cwd: &std::path::Path,
) -> serde_json::Value {
    use std::io::Write;
    let mut c = std::process::Command::new("python3")
        .arg(huba::accounts::mo_dau_script())
        .args(["--cli", "codex", "--codex-hook", "--goc"])
        .arg(goc)
        .env("CODEX_HOME", home)
        .env_remove("HUBA_MCP_PROJECT_AGENT")
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
}

/// Quang 14/11: *«đoạn đầu giống acc claude đâu?»* — bản `44d6bd8` đưa khối khởi
/// động vào NGỮ CẢNH (nghiệm thu: model chép lại) nhưng màn codex trống, vì màn chỉ
/// in hook qua `systemMessage` (đo 14/11 cửa sổ Terminal thật: «↳ Hook · …»).
/// Khâu chung: `systemMessage` mang dòng TÀI KHOẢN + MCP + khối khởi động; luật gốc
/// chỉ dán khi codex không tự đọc (cwd ngoài gốc git của workspace).
#[test]
fn mo_dau_phien_hien_man_va_vao_ngu_canh() {
    let d = tempfile::tempdir().unwrap();
    let goc = goc_gia(d.path());
    let home = d.path().join("codex-acc77");
    std::fs::create_dir_all(&home).unwrap();
    let khac = d.path().join("khac");
    std::fs::create_dir_all(&khac).unwrap();
    for g in [&goc, &khac] {
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(g)
            .status()
            .unwrap();
    }
    let v = hook_mo_dau(&goc, &home, &khac);
    let man = v["systemMessage"].as_str().unwrap_or("");
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap_or("");
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "SessionStart");
    assert!(man.starts_with("TÀI KHOẢN PHIÊN: acc77 · codex"), "{man}");
    assert!(man.contains("MCP project-agent: ✓ 1 công cụ"), "{man}");
    assert!(
        man.contains(&format!(
            "KHOI-DONG-THU main {}",
            khac.canonicalize().unwrap().display()
        )),
        "{man}"
    );
    assert!(
        ctx.contains("LUAT-THU-0714"),
        "cwd ngoài gốc ⟹ dán luật: {ctx}"
    );
    assert!(
        !man.contains("LUAT-THU-0714"),
        "luật không đổ lên màn: {man}"
    );
    let trong = hook_mo_dau(&goc, &home, &goc);
    assert!(
        !trong["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap_or("")
            .contains("LUAT-THU-0714"),
        "đứng đúng kho gốc ⟹ codex tự đọc, không dán lần hai: {trong}"
    );

    // khối khởi động hỏng ⟹ ĐÚNG câu phiên Claude in, giữ phần đã in.
    std::fs::write(
        goc.join("scripts").join("phien.sh"),
        "echo nua-chung; exit 3\n",
    )
    .unwrap();
    let man = hook_mo_dau(&goc, &home, &khac)["systemMessage"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(
        man.contains("HOOK SessionStart HỎNG (mã 3)") && man.contains("nua-chung"),
        "{man}"
    );

    // MCP mất ⟹ đoạn mở đầu ra ⛔ ngay dòng đầu, cả trên màn lẫn ngữ cảnh.
    std::fs::remove_file(d.path().join("mcp-gia")).unwrap();
    let v = hook_mo_dau(&goc, &home, &khac);
    let man = v["systemMessage"].as_str().unwrap_or("");
    assert!(
        man.starts_with("⛔ ĐOẠN MỞ ĐẦU THIẾU KHÂU BẮT BUỘC"),
        "{man}"
    );
    assert!(
        v["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap_or("")
            .contains("⛔ ĐOẠN MỞ ĐẦU"),
        "{v}"
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
    // Đo 14/11 (codex-cli 0.169.0) hai chiều: `-a never` + không khoá ⟹ «MCP tool call
    // requires approval, but approval policy is never» (ảnh Quang, phiên aihub);
    // `default_tools_approval_mode="approve"` ⟹ get_context_brief chạy (MCP_OK).
    assert!(
        a.contains(
            &"mcp_servers.project-agent.default_tools_approval_mode=\"approve\"".to_string()
        ),
        "MCP codetrail phải gọi được như phiên Claude: {a:?}"
    );
    assert_eq!(a.len(), 3, "{a:?}");
    assert_eq!(
        a[0],
        format!("mcp_servers.project-agent.command=\"{}\"", bin.display())
    );
    assert!(
        a[1].starts_with("mcp_servers.project-agent.env={PROJECTS_ROOT=\""),
        "{a:?}"
    );
    // Nguồn mặc định = `<gốc>/.mcp.json` của workspace (thứ git mang theo), `${HOME}` mở ra.
    std::env::remove_var("HUBA_MCP_PROJECT_AGENT");
    std::fs::write(
        d.path().join(".mcp.json"),
        r#"{"mcpServers":{"project-agent":{"command":"${HOME}/.local/x/project-agent"}}}"#,
    )
    .unwrap();
    let home = huba::config::home_dir().unwrap();
    assert_eq!(
        huba::accounts::codex_mcp_bin(d.path()),
        home.join(".local/x/project-agent").display().to_string()
    );
    assert_eq!(
        huba::accounts::codex_mcp_bin(&d.path().join("khong-co")),
        ""
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

/// Quang 14/11 «mọi tài khoản cùng trải nghiệm»: phiên Claude có hook `Stop` +
/// `UserPromptSubmit` → `scripts/nhac-chuyen-phien.py` (nhắc chuyển phiên khi ngữ
/// cảnh ≥ 50 %). Đo 14/11 codex-cli 0.169.0: hai sự kiện ấy bắn với payload y khuôn
/// Claude (`transcript_path`, `stop_hook_active`), `decision: block` và
/// `additionalContext` đều tới model. Script nằm CẠNH cổng (gói Linux: `kem/scripts/`).
#[test]
fn nhac_chuyen_phien_nhu_claude() {
    let gate = std::path::Path::new("/x/kem/scripts/cong_bao_mat.py");
    let goc = std::path::Path::new("/ws");
    let v = huba::accounts::codex_hooks_json(gate, Some(goc), "/b/project-agent");
    let mo = v["hooks"]["SessionStart"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap_or("");
    assert!(
        mo.contains("/x/kem/scripts/mo-dau-phien.py")
            && mo.contains("--goc '/ws'")
            && mo.contains("--codex-hook")
            && mo.contains("--mcp '/b/project-agent'"),
        "{mo}"
    );
    for su_kien in ["Stop", "UserPromptSubmit"] {
        let lenh = v["hooks"][su_kien][0]["hooks"][0]["command"]
            .as_str()
            .unwrap_or_else(|| panic!("thiếu hook {su_kien}: {v}"));
        assert!(
            lenh.contains("/x/kem/scripts/nhac-chuyen-phien.py"),
            "{su_kien}: {lenh}"
        );
    }
    // Không biết workspace ở đâu ⟹ không bịa đường script.
    let v = huba::accounts::codex_hooks_json(gate, None, "");
    assert!(v["hooks"].get("Stop").is_none(), "{v}");
}
