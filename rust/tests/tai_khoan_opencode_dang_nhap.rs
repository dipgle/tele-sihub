//! `/accounts <tên> -l` cho hàng `cli: "opencode"` đăng nhập ChatGPT (provider `openai`).
//!
//! Quang 15/11: *"cài acc9 chạy opencode và dùng model của acc10"* · *"làm nhánh đăng nhập
//! opencode từ tele đi"*. Trước bản này `account_login::start` chỉ biết codex|claude ⟹ một
//! hàng opencode rơi vào nhánh claude và mở `claude auth login` (sai CLI, sai tài khoản).
//! Mỗi bài có vế NGƯỢC (màn/hàng KHÔNG phải opencode-openai ⟹ không nhận), để bài xanh
//! không xanh vì hàm nhận tất cả.

mod common;

use huba::account_login::{
    bo_mau, la_man_codex, la_man_dang_nhap, la_man_opencode, login_pid, opencode_after,
    opencode_device,
};
use huba::accounts::{opencode_login_line, opencode_login_provider};
use huba::config::ClaudeAccountCfg;

/// Màn THẬT của `opencode auth login openai --method chatgpt-headless` (opencode 2.2.7,
/// Ubuntu tmux, 15/11 acc9 — mã đã dùng xong, giữ nguyên văn). Dòng đầu là lệnh huba gõ.
const MAN_CHO: &str = "user1@user1-ubuntu:/mnt/data/projects$ env XDG_CONFIG_HOME=$HOME/.opencode-acc9/config XDG_DATA_HOME=$HOME/.opencode-acc9/data XDG_CACHE_HOME=$HOME/.opencode-acc9/cache XDG_STATE_HOME=$HOME/.opencode-acc9/state OPENCODE_DISABLE_AUTOUPDATE=1 BROWSER=/bin/true $HOME/.opencode/bin/opencode auth login openai --method chatgpt-headless --standalone

┌  Connect an integration
│
◇  Authorization started
│
●  Enter code: JKYZ-B36HQ
│
●  https://auth.openai.com/codex/device
│
◇  Waiting for authorization...
";

/// Hai kết cục THẬT đo cùng ngày: lượt nhập mã xong (19:00) và lượt để mã hết hạn (16:13).
const DUOI_XONG: &str = "│
◆  Connected to OpenAI
│
└  Done

user1@user1-ubuntu:/mnt/data/projects$ ";
const DUOI_HET_HAN: &str = "│
■  Authorization expired
│
└  Failed

user1@user1-ubuntu:/mnt/data/projects$ ";

fn hang(ten: &str, cli: Option<&str>, model: Option<&str>) -> ClaudeAccountCfg {
    ClaudeAccountCfg {
        name: ten.into(),
        config_dir: Some(format!("~/.claude-{ten}")),
        cli: cli.map(str::to_string),
        model: model.map(str::to_string),
        ..Default::default()
    }
}

#[test]
fn doc_duoc_duong_dan_va_ma_tren_man_that() {
    assert!(la_man_opencode(MAN_CHO));
    assert!(
        la_man_dang_nhap(MAN_CHO),
        "cửa chốt chung cũng nhận màn opencode"
    );
    assert!(!la_man_codex(MAN_CHO), "màn opencode không phải màn codex");
    assert_eq!(
        opencode_device(MAN_CHO),
        Some((
            "https://auth.openai.com/codex/device".to_string(),
            "JKYZ-B36HQ".to_string()
        ))
    );
    assert_eq!(opencode_after(MAN_CHO), "", "chưa có kết cục");
}

#[test]
fn man_chua_in_ma_hoac_ma_la_thi_khong_doan() {
    let chua_ma = MAN_CHO.split("●  Enter code").next().unwrap();
    assert_eq!(opencode_device(chua_ma), None, "chưa có dòng mã ⟹ None");
    let ma_la = MAN_CHO.replace("JKYZ-B36HQ", "please wait");
    assert_eq!(
        opencode_device(&ma_la),
        None,
        "chữ lạ sau `Enter code:` ⟹ None"
    );
    // Màn của claude / codex không phải màn opencode.
    let claude = "$ claude auth login\nIf the browser didn't open, visit: https://claude.com/x\nPaste code here if prompted >";
    assert!(!la_man_opencode(claude));
    let codex = "$ CODEX_HOME=x codex login --device-auth\n   https://auth.openai.com/codex/device\n2. Enter this one-time code\n   HBAC-ERA9M\n";
    assert!(!la_man_opencode(codex));
    assert_eq!(opencode_device(codex), None);
}

#[test]
fn ket_cuc_doc_nguyen_van_sau_dong_cho() {
    assert_eq!(
        opencode_after(&format!("{MAN_CHO}{DUOI_XONG}")),
        "Connected to OpenAI · Done"
    );
    assert_eq!(
        opencode_after(&format!("{MAN_CHO}{DUOI_HET_HAN}")),
        "Authorization expired · Failed"
    );
}

#[test]
fn chi_hang_opencode_model_openai_moi_co_dang_nhap() {
    let acc9 = hang("acc9", Some("opencode"), Some("openai/gpt-6.1-sol"));
    assert_eq!(opencode_login_provider(&acc9), Some("openai"));
    // Ngược: opencode chạy khoá API (env_file) — không có luồng đăng nhập.
    let acc8 = hang(
        "acc8",
        Some("opencode"),
        Some("opencode-go/deepseek-v4.1-flash"),
    );
    assert_eq!(opencode_login_provider(&acc8), None);
    assert_eq!(
        opencode_login_provider(&hang("x", Some("opencode"), None)),
        None
    );
    // Ngược: model openai nhưng KHÔNG phải hàng opencode.
    let codex = hang("acc10", Some("codex"), Some("openai/gpt-6.1-sol"));
    assert_eq!(opencode_login_provider(&codex), None);
    assert_eq!(
        opencode_login_provider(&hang("acc1", None, Some("openai/x"))),
        None
    );
}

#[test]
fn dong_lenh_dang_nhap_dung_phan_vung_khong_mang_bi_mat() {
    let cfg = common::cfg_for_tests();
    let mut acc9 = hang("acc9", Some("opencode"), Some("openai/gpt-6.1-sol"));
    acc9.env_file = Some("~/.opencode_acc9.env".into());
    acc9.api_key_env = Some("OPENCODE_API_KEY".into());
    let l = opencode_login_line(&cfg, &acc9, "openai");
    for phan in [
        "XDG_CONFIG_HOME=$HOME/.opencode-acc9/config",
        "XDG_DATA_HOME=$HOME/.opencode-acc9/data",
        "XDG_CACHE_HOME=$HOME/.opencode-acc9/cache",
        "XDG_STATE_HOME=$HOME/.opencode-acc9/state",
        " BROWSER=true ",
        " auth login openai --method chatgpt-headless",
    ] {
        assert!(l.contains(phan), "thiếu «{phan}» trong: {l}");
    }
    assert!(
        la_man_opencode(&format!("$ {l}\n●  https://auth.openai.com/codex/device\n")),
        "màn của CHÍNH dòng lệnh này phải được chốt là của mình"
    );
    // macOS không có `/bin/true` (chỉ `/usr/bin/true`) ⟹ tên trần, PATH tìm trên cả hai máy.
    assert!(!l.contains("/bin/true"), "{l}");
    // Đăng nhập không cần khoá API, không cần cấu hình phiên.
    assert!(!l.contains(".opencode_acc9.env"), "không nạp env_file: {l}");
    assert!(!l.contains("OPENCODE_CONFIG_CONTENT"), "{l}");
    // Ngược: phân vùng của acc khác không lọt vào.
    let khac = opencode_login_line(&cfg, &hang("acc7", Some("opencode"), None), "openai");
    assert!(!khac.contains(".opencode-acc9"), "{khac}");
}

#[test]
fn tim_tien_trinh_dang_nhap_opencode_tren_tty() {
    let ps = "  4101 -bash\n  4180 /home/u/.opencode/bin/opencode auth login openai --method chatgpt-headless --standalone\n";
    assert_eq!(login_pid(ps), Some(4180));
    assert_eq!(login_pid("  4101 -bash\n"), None, "CLI đã thoát ⟹ None");
}

#[test]
fn bo_mau_cua_auth_list() {
    // Khuôn `opencode auth list` đo 15/11 (tô màu cả khi không có tty).
    let co_mau =
        "\u{1b}[1mOpenAI\u{1b}[0m  OpenAI                      \u{1b}[32mstored\u{1b}[0m\n";
    assert_eq!(
        bo_mau(co_mau),
        "OpenAI  OpenAI                      stored\n"
    );
    assert_eq!(bo_mau("không màu"), "không màu", "chữ không màu giữ nguyên");
}
