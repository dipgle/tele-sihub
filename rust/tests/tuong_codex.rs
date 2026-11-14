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
    let ok = huba::accounts::ensure_codex_wall(&acc).expect("cổng thật phải dựng được tường");
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
