//! Tài khoản MẶC ĐỊNH đọc sổ của nó ở `~/.claude` — KHÔNG theo `$CLAUDE_CONFIG_DIR`
//! mà tiến trình cha để lại.
//!
//! Bàn giao 07/11 (phiên huba `eac8b0bc` → kế nhiệm): gõ `huba …` **trong một phiên
//! Claude Code** thì tiến trình con thừa hưởng `CLAUDE_CONFIG_DIR` của chính phiên
//! ấy. `account_book_dir` (rust/src/sessions.rs) đang đọc biến đó cho tài khoản
//! không khai `config_dir`, nên nó đọc sổ của tài khoản KIA: mỗi phiên hiện **hai
//! lần** — một dưới tên tài khoản thật, một dưới tên tài khoản mặc định (đang khoá).
//! Lách tạm đã dùng: `env -u CLAUDE_CONFIG_DIR ./huba …`; bài này khoá hành vi đúng.
//!
//! Đây là nguyên tắc đã có sẵn ở phía GỌI CLI (`list_account_cli`: "Absent variable
//! = default account") — bài này đòi phía ĐỌC TỆP cùng một luật.
//!
//! ⚠ Một bài duy nhất trong tệp này: nó sửa biến môi trường của CẢ tiến trình, nên
//! không được có bài thứ hai chạy song song trong cùng binary.
//!
//! Chạy: `cd rust && cargo test --offline --test sach_mac_dinh_khong_theo_bien_phien`

use std::fs;
use std::path::Path;

use huba::config::{ClaudeAccountCfg, Config};

/// Đặt biến môi trường rồi TRẢ LẠI giá trị cũ khi ra khỏi phạm vi — kể cả khi
/// bài kiểm panic (Drop vẫn chạy).
struct BienTam {
    ten: &'static str,
    cu: Option<std::ffi::OsString>,
}

impl BienTam {
    fn dat(ten: &'static str, gia: &Path) -> Self {
        let cu = std::env::var_os(ten);
        std::env::set_var(ten, gia);
        Self { ten, cu }
    }
}

impl Drop for BienTam {
    fn drop(&mut self) {
        match &self.cu {
            Some(v) => std::env::set_var(self.ten, v),
            None => std::env::remove_var(self.ten),
        }
    }
}

fn ghi_so(config_dir: &Path, ten_tep: &str, session_id: &str) {
    let sessions = config_dir.join("sessions");
    fs::create_dir_all(&sessions).expect("tạo được <config>/sessions");
    fs::write(
        sessions.join(ten_tep),
        format!(r#"{{"sessionId":"{session_id}","pid":4242,"status":"idle","statusUpdatedAt":1}}"#),
    )
    .expect("ghi được tệp sổ");
}

#[test]
fn tai_khoan_mac_dinh_doc_so_o_nha_khong_doc_bien_cua_phien() {
    let goc = tempfile::tempdir().expect("tempdir");
    let nha = goc.path().join("nha"); // HOME giả
    let acc2 = goc.path().join("acc2");
    ghi_so(&nha.join(".claude"), "100.json", "sess-mac-dinh");
    ghi_so(&acc2, "200.json", "sess-acc2");

    let _nha = BienTam::dat("HOME", &nha);
    // Đây đúng là thứ một phiên Claude Code (acc2) để lại cho tiến trình con.
    let _bien_phien = BienTam::dat("CLAUDE_CONFIG_DIR", &acc2);

    let cfg = Config {
        claude_accounts: vec![
            // acc1 = tài khoản MẶC ĐỊNH: không khai `config_dir` ⟹ `~/.claude`.
            ClaudeAccountCfg {
                name: "acc1".into(),
                config_dir: None,
                launch: None,
                locked: true,
            },
            ClaudeAccountCfg {
                name: "acc2".into(),
                config_dir: Some(acc2.display().to_string()),
                launch: None,
                locked: false,
            },
        ],
        ..Default::default()
    };

    let van_tay = huba::sessions::book_status_fingerprint(&cfg);

    assert_eq!(
        van_tay.matches("sess-mac-dinh").count(),
        1,
        "sổ của tài khoản mặc định phải đọc từ ~/.claude: {van_tay}"
    );
    assert_eq!(
        van_tay.matches("sess-acc2").count(),
        1,
        "sổ acc2 KHÔNG được đọc hai lần (một lần dưới tên tài khoản mặc định): {van_tay}"
    );
}
