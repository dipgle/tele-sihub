//! Tài khoản Claude CHƯA có `<config>/sessions` không được làm huba spawn `claude agents --json` MỖI lần chụp.
//!
//! 🔴 Hà 24/11: *"opencode nhận … tin vẫn chậm"*. ĐO: gõ vào opencode `command_done` 5,5 giây (Claude 1,4 giây) vì phải
//! dựng lại cả ảnh chụp (~3,1 s), trong đó `ms_ask_accounts` đứng yên ~800 ms. Nhật ký: 75/75 vòng gần nhất có
//! `claude_books_missing` cho CẢ BỐN tài khoản `acc2 · acc4 · acc5 · acc7` (không có thư mục `sessions/`) ⟹ mỗi lần chụp
//! chạy `claude agents --json` bốn lần ≈ 200 ms một lần, và lần nào cũng trả `[]`.
//!
//! Sửa: kết quả RỖNG của đường CLI được nhớ ngắn hạn cho tài khoản còn thiếu `sessions/`. Ba phép đo:
//! ① hai lần hỏi liên tiếp ⟹ CLI chỉ bị gọi MỘT lần (RED trên mã cũ) · ② ĐỐI CHỨNG: CLI trả phiên thật thì KHÔNG nhớ —
//! lần hai vẫn hỏi lại, nên phiên mới hiện ra ngay · ③ `sessions/` xuất hiện thì đường đọc sổ nhanh lấy lại, không
//! gọi CLI nữa.

mod common;

use huba::config::ClaudeAccountCfg;
use huba::sessions::{listed_in_books, Listed};

const ID: &str = "0123abcd-0000-4000-8000-000000000001";

/// Một `claude` GIẢ: ghi dấu mỗi lần bị gọi, in `noi_dung`.
fn dung(
    ten_acc: &str,
    noi_dung: &str,
) -> (tempfile::TempDir, huba::config::Config, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dau = tmp.path().join("da-goi.txt");
    let gia = tmp.path().join("claude-gia.sh");
    std::fs::write(
        &gia,
        format!(
            "#!/bin/sh\necho x >> '{}'\necho '{}'\n",
            dau.display(),
            noi_dung
        ),
    )
    .expect("ghi claude giả");
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&gia, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    let cfg_dir = tmp.path().join(format!("cfg-{ten_acc}"));
    std::fs::create_dir_all(&cfg_dir).expect("tạo cfg_dir");
    let mut cfg = common::cfg_for_tests();
    cfg.claude_cli = gia.display().to_string();
    cfg.claude_accounts = vec![ClaudeAccountCfg {
        name: ten_acc.into(),
        config_dir: Some(cfg_dir.display().to_string()),
        cli: None,
        ..Default::default()
    }];
    (tmp, cfg, dau)
}

fn so_lan_goi(dau: &std::path::Path) -> usize {
    std::fs::read_to_string(dau)
        .unwrap_or_default()
        .lines()
        .count()
}

/// ① CA CỦA HÀ — tài khoản trống, hai lần chụp ⟹ CLI chỉ chạy một lần.
#[test]
fn ket_qua_rong_cua_cli_duoc_nho_ngan_han() {
    let (_tmp, cfg, dau) = dung("acc-nho-rong", "[]");
    assert_eq!(listed_in_books(&cfg, ID), Listed::No);
    assert_eq!(listed_in_books(&cfg, ID), Listed::No);
    assert_eq!(
        so_lan_goi(&dau),
        1,
        "mỗi lần chụp lại spawn `claude agents --json` cho tài khoản không có phiên nào"
    );
}

/// ② ĐỐI CHỨNG NGƯỢC — CLI trả MỘT phiên thật ⟹ không nhớ, lần sau vẫn hỏi lại (phiên mới không bị che).
#[test]
fn ket_qua_co_phien_thi_khong_nho() {
    let hang = format!(r#"[{{"sessionId":"{ID}","cwd":"/x","status":"idle"}}]"#);
    let (_tmp, cfg, dau) = dung("acc-nho-co-phien", &hang);
    assert_eq!(listed_in_books(&cfg, ID), Listed::Yes);
    assert_eq!(listed_in_books(&cfg, ID), Listed::Yes);
    assert_eq!(
        so_lan_goi(&dau),
        2,
        "kết quả CÓ phiên bị nhớ ⟹ phiên mới sẽ hiện chậm"
    );
}

/// ③ `sessions/` xuất hiện (CLI ghi sổ khi có phiên) ⟹ đọc sổ ngay, không qua CLI, dù đang trong thời hạn nhớ.
#[test]
fn co_so_sessions_thi_doc_so_khong_qua_cli() {
    let (tmp, cfg, dau) = dung("acc-nho-co-so", "[]");
    assert_eq!(listed_in_books(&cfg, ID), Listed::No);
    assert_eq!(so_lan_goi(&dau), 1);
    let sessions = tmp.path().join("cfg-acc-nho-co-so").join("sessions");
    std::fs::create_dir_all(&sessions).expect("tạo sessions/");
    std::fs::write(
        sessions.join("1234.json"),
        format!(
            r#"{{"pid":{},"sessionId":"{ID}","cwd":"/x","status":"idle","kind":"interactive","startedAt":1,"updatedAt":1}}"#,
            std::process::id()
        ),
    )
    .expect("ghi sổ phiên");
    assert_eq!(listed_in_books(&cfg, ID), Listed::Yes);
    assert_eq!(so_lan_goi(&dau), 1, "có sổ rồi mà vẫn gọi CLI");
}
