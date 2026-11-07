//! Nhật ký của một phiên nằm dưới thư mục cấu hình của ĐÚNG tài khoản chạy nó —
//! không phải lúc nào cũng dưới gốc chung `~/.claude/projects`.
//!
//! 🔴 07/11 ([dwork/dhub] chuyển lời Quang: *"sửa tận gốc"*): `huba sessions` báo
//! `⚠ chưa có nhật ký phiên` cho MỌI phiên của acc8 và acc9. Đo trên máy cùng
//! ngày: `projects` của acc2–acc7 là liên kết về `~/.claude/projects`, còn
//! `~/.claude-acc8/projects` (60 nhật ký) và `~/.claude-acc9/projects` (15) là
//! thư mục THẬT. Ca sống: acc9 pid 72133, phiên `0532c61d`, nhật ký ở
//! `~/.claude-acc9/projects/-private-tmp-claude-501-dinh-tuyen-cwd/0532c61d….jsonl`
//! mà huba chỉ tìm ở gốc chung. Cùng lượt `huba sessions`, dòng thứ hai mang câu
//! ấy (acc6 `950b1b21`) là ca THẬT không có tệp ở đâu cả — nên bài đối chứng
//! dưới đây giữ cho câu «chưa có nhật ký» vẫn nói được khi nó đúng.
#![cfg(unix)] // máy thu nhỏ dựng liên kết `projects` bằng `std::os::unix::fs::symlink`

use std::path::{Path, PathBuf};

use huba::config::{ClaudeAccountCfg, Config};
use huba::sessions;

const ID_RIENG: &str = "0532c61d-3077-4653-ad15-84ded0f03c68";
const ID_CHUNG: &str = "7a1b2c3d-0000-4000-8000-000000000001";
const ID_KHONG_CO: &str = "950b1b21-8829-4526-8fc1-3f114518fb16";
const CWD: &str = "/private/tmp/claude-501/dinh-tuyen-cwd";

const LUOT_HOI: &str = r#"{"type":"user","message":{"role":"user","content":"đo thử"},"timestamp":"2026-11-07T01:00:00.000Z"}"#;
const LUOT_DAP: &str = r#"{"type":"assistant","message":{"role":"assistant","stop_reason":"end_turn","content":[{"type":"text","text":"[dwork/dhub] Đã đo xong."}]},"timestamp":"2026-11-07T01:00:05.000Z"}"#;

fn acc(name: &str, dir: &Path) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: name.into(),
        config_dir: Some(dir.to_string_lossy().into_owned()),
        launch: None,
        locked: false,
    }
}

fn ghi_nhat_ky(root: &Path, cwd: &str, id: &str) -> PathBuf {
    let dir = root.join("projects").join(sessions::transcript_slug(cwd));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("{id}.jsonl"));
    std::fs::write(&f, format!("{LUOT_HOI}\n{LUOT_DAP}\n")).unwrap();
    f
}

/// Máy thu nhỏ đúng hình máy thật 07/11: gốc chung · acc3 liên kết về gốc chung
/// · acc9 có `projects` RIÊNG giữ nhật ký `ID_RIENG`.
struct May {
    _tmp: tempfile::TempDir,
    cfg: Config,
    chung: PathBuf,
    acc9: PathBuf,
}

fn may() -> May {
    let tmp = tempfile::tempdir().unwrap();
    let chung = tmp.path().join("claude");
    let acc3 = tmp.path().join("claude-acc3");
    let acc9 = tmp.path().join("claude-acc9");
    std::fs::create_dir_all(chung.join("projects")).unwrap();
    std::fs::create_dir_all(&acc3).unwrap();
    std::os::unix::fs::symlink(chung.join("projects"), acc3.join("projects")).unwrap();
    ghi_nhat_ky(&acc9, CWD, ID_RIENG);
    ghi_nhat_ky(&chung, CWD, ID_CHUNG);
    let cfg = Config {
        claude_transcript_root: chung.to_string_lossy().into_owned(),
        claude_accounts: vec![acc("acc3", &acc3), acc("acc9", &acc9)],
        ..Default::default()
    };
    May {
        _tmp: tmp,
        cfg,
        chung,
        acc9,
    }
}

// ── Ca hỏng 07/11: nhật ký dưới `projects` RIÊNG của tài khoản ────────────────

#[test]
fn stream_doc_duoc_nhat_ky_nam_o_projects_rieng_cua_tai_khoan() {
    let m = may();
    let s = sessions::stream(&m.cfg, ID_RIENG, CWD, 10);
    assert_eq!(
        s.note, None,
        "phiên acc9 có nhật ký thật mà huba khai thiếu"
    );
    assert!(
        s.events
            .iter()
            .any(|e| format!("{e:?}").contains("Đã đo xong")),
        "đọc được tệp thì phải ra lời cuối của phiên: {:?}",
        s.events
    );
}

#[test]
fn cac_cua_chi_co_id_cung_tim_duoc_nhat_ky_rieng() {
    let m = may();
    assert!(
        sessions::transcript_idle_seconds(&m.cfg, ID_RIENG).is_some(),
        "transcript_idle_seconds"
    );
    assert_eq!(
        sessions::turn_open_at_death(&m.cfg, ID_RIENG),
        Some(false),
        "turn_open_at_death: lượt cuối là end_turn"
    );
}

/// Đường `huba sessions` đi: ảnh chụp mỗi vòng chỉ `stat` đường thẳng theo `cwd`
/// trên các gốc đã tính một lần (`snapshot` → `transcript_by_cwd`).
#[test]
fn anh_chup_tim_duong_thang_tren_moi_goc() {
    let m = may();
    let roots = sessions::transcript_roots(&m.cfg);
    let p = sessions::transcript_by_cwd(&roots, CWD, ID_RIENG).expect("nhật ký acc9");
    assert!(p.starts_with(&m.acc9), "{}", p.display());
    assert!(sessions::transcript_by_cwd(&roots, CWD, ID_KHONG_CO).is_none());
}

/// acc3 liên kết về gốc chung ⟹ hỏi gốc ấy MỘT lần, không quét hai lượt.
#[test]
fn goc_lien_ket_ve_goc_chung_chi_hoi_mot_lan() {
    let m = may();
    let roots = sessions::transcript_roots(&m.cfg);
    assert_eq!(roots, vec![m.chung.clone(), m.acc9.clone()]);
}

/// Tài khoản mặc định (không khai `config_dir`) KHÔNG kéo `~/.claude` thật vào
/// một cấu hình đã trỏ gốc riêng.
#[test]
fn tai_khoan_mac_dinh_khong_them_goc() {
    let m = may();
    let mut cfg = m.cfg.clone();
    cfg.claude_accounts = vec![ClaudeAccountCfg {
        name: "mặc định".into(),
        config_dir: None,
        launch: None,
        locked: false,
    }];
    assert_eq!(sessions::transcript_roots(&cfg), vec![m.chung.clone()]);
}

// ── Không gãy đường cũ ────────────────────────────────────────────────────────

#[test]
fn nhat_ky_o_goc_chung_van_doc_duoc() {
    let m = may();
    let s = sessions::stream(&m.cfg, ID_CHUNG, CWD, 10);
    assert_eq!(s.note, None);
    assert!(!s.events.is_empty());
}

// ── Đối chứng: câu «chưa có nhật ký» vẫn phải nói được khi nó ĐÚNG ─────────────

#[test]
fn khong_co_tep_o_goc_nao_thi_van_bao_chua_co_nhat_ky() {
    let m = may();
    let s = sessions::stream(&m.cfg, ID_KHONG_CO, CWD, 10);
    assert_eq!(s.note.as_deref(), Some("chưa có nhật ký phiên"));
    assert!(sessions::transcript_idle_seconds(&m.cfg, ID_KHONG_CO).is_none());
    // Dựng máy đúng như mô tả — bài kiểm không được xanh vì máy thử rỗng.
    assert!(m.acc9.join("projects").is_dir() && m.chung.join("projects").is_dir());
}
