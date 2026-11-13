//! `huba doctor` trên máy Ubuntu thật (13/11, user1-ubuntu, chỉ có acc8 opencode):
//!
//! ```text
//! {"account":"acc8","hau_qua":"KHÔNG gợi ý tài khoản này — mở cửa sổ ra là nó đứng",
//!  "msg":"account_not_ready","path":"/home/user1/.claude-acc8/.claude.json", …}
//! ```
//!
//! acc opencode KHÔNG có sổ `.claude.json`; doctor gọi thẳng `quota::read` nên đẻ
//! một dòng log NÓI SAI (đường chọn thật `read_all` vẫn rẽ đúng). Bốn chỗ khác đã
//! tự chép phép rẽ `is_opencode()`; doctor là chỗ thứ năm và là chỗ quên — nay
//! mọi đường đi qua `quota::read_acc`. Bài này chạy CHÍNH binary.

use std::process::Command;

fn chay_doctor(cli: &str) -> String {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cfg_dir = tmp.path().join("cfg-accx");
    // Đúng ca máy thật: `claude` đã chạy một lần trong thư mục ấy, chưa đăng nhập.
    std::fs::create_dir_all(&cfg_dir).expect("tạo cfg_dir");
    std::fs::write(cfg_dir.join(".claude.json"), "{}").expect("ghi sổ");
    let cfg = tmp.path().join("huba.config.json");
    let body = serde_json::json!({
        "claude_accounts": [{
            "name": "accx",
            "config_dir": cfg_dir.display().to_string(),
            "cli": cli,
        }],
    });
    std::fs::write(&cfg, body.to_string()).expect("ghi config");
    let out = Command::new(env!("CARGO_BIN_EXE_huba"))
        .arg("doctor")
        .env("HUB_CONFIG", &cfg)
        .env("HOME", tmp.path())
        .output()
        .expect("chạy huba doctor");
    let all = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // Phải thấy doctor ĐÃ chấm tài khoản này — đầu ra rỗng không được đọc là "sạch".
    assert!(
        all.contains("accx"),
        "doctor không in tài khoản accx:\n{all}"
    );
    all
}

#[test]
fn doctor_khong_doc_so_claude_cua_acc_opencode() {
    let all = chay_doctor("opencode");
    assert!(
        !all.contains("account_not_ready"),
        "acc opencode bị chấm bằng sổ .claude.json:\n{all}"
    );
}

/// Đối chứng ngược: cùng cấu hình nhưng acc claude (sổ không có `oauthAccount`) ⇒ dòng log ấy PHẢI
/// còn — bài trên không xanh nhờ log tắt hay doctor không đọc sổ nào.
#[test]
fn doi_chung_acc_claude_thieu_so_van_keu() {
    let all = chay_doctor("claude");
    assert!(
        all.contains("account_not_ready"),
        "acc claude chưa đăng nhập mà doctor không kêu:\n{all}"
    );
}
