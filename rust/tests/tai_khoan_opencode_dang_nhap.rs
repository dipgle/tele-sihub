mod common;

use huba::account_login::{
    bo_mau, la_man_dang_nhap, la_man_opencode, login_pid, opencode_after, opencode_device,
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
    // Màn của CLI khác (claude) không phải màn opencode.
    let claude = "$ claude auth login\nIf the browser didn't open, visit: https://claude.com/x\nPaste code here if prompted >";
    assert!(!la_man_opencode(claude));
    let man_khac = "$ some-other-cli login\n   https://example.test/device\n2. Enter this one-time code\n   HBAC-ERA9M\n";
    assert!(!la_man_opencode(man_khac));
    assert_eq!(opencode_device(man_khac), None);
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
    let cli_la = hang("acc10", Some("codex"), Some("openai/gpt-6.1-sol"));
    assert_eq!(opencode_login_provider(&cli_la), None);
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

/// Hà 15/11 *«/accounts chưa hiện thông tin sử dụng»* — chọn B: hỏi `/wham/usage`.
/// Thân THẬT đo 15/11 bằng token acc9 (bỏ các ô danh tính; số giữ nguyên văn).
#[test]
fn han_muc_chatgpt_tu_wham_usage() {
    use huba::opencode::wham_sang_rate_limits;
    use huba::quota::{codex_quota_tu, is_chatgpt_opencode};
    let than: serde_json::Value = serde_json::from_str(
        r#"{"plan_type":"prolite","rate_limit":{"allowed":true,"limit_reached":false,
            "primary_window":{"used_percent":46,"limit_window_seconds":604800,
            "reset_after_seconds":486315,"reset_at":1795235473},"secondary_window":null},
            "code_review_rate_limit":null,"rate_limit_reached_type":null,
            "user_id":"BI-MAT-KHONG-DUOC-LOT","email":"khong@lot.vn"}"#,
    )
    .unwrap();
    let rl = wham_sang_rate_limits(&than).expect("có rate_limit");
    let chu = rl.to_string();
    assert!(
        !chu.contains("BI-MAT") && !chu.contains("khong@lot"),
        "danh tính không đi tiếp: {chu}"
    );
    let q = codex_quota_tu("acc9", &rl, "2026-11-15T13:30:00+00:00");
    assert_eq!(q.week_pct, Some(46));
    assert_eq!(
        q.hour5_pct,
        Some(46),
        "gói chỉ có cửa tuần ⟹ 5 giờ gán ngược từ tuần"
    );
    assert!(q
        .week_resets_at
        .as_deref()
        .unwrap()
        .starts_with("2026-11-21"));
    assert!(q.why_unknown.is_none());
    assert!(q.fetched_at_ms.is_some());
    // Kịch trần: `limit_reached` mà không có loại ⟹ 100 %.
    let mut kich = than.clone();
    kich["rate_limit"]["limit_reached"] = true.into();
    let q = codex_quota_tu(
        "acc9",
        &wham_sang_rate_limits(&kich).unwrap(),
        "2026-11-15T13:30:00+00:00",
    );
    assert_eq!(q.week_pct, Some(100));
    // Ngược: thân không có rate_limit ⟹ None (không bịa 0 %).
    assert_eq!(
        wham_sang_rate_limits(&serde_json::json!({"plan_type":"free"})),
        None
    );
    // Chỉ hàng opencode + model openai mới đi đường này.
    assert!(is_chatgpt_opencode(&hang(
        "acc9",
        Some("opencode"),
        Some("openai/gpt-6.1-sol")
    )));
    assert!(!is_chatgpt_opencode(&hang(
        "acc8",
        Some("opencode"),
        Some("opencode-go/deepseek-v4.1-flash")
    )));
    assert!(!is_chatgpt_opencode(&hang(
        "acc10",
        Some("codex"),
        Some("openai/gpt-6.1-sol")
    )));
}

/// Hỏi THẬT máy chủ ChatGPT bằng đăng nhập của acc9 trên máy này (chỉ đọc kho opencode) —
/// chạy tay: `cargo test --test tai_khoan_opencode_dang_nhap doc_that -- --ignored --nocapture`
#[test]
#[ignore = "gọi máy chủ ChatGPT bằng tài khoản thật — chạy tay"]
fn doc_that_acc9_chatgpt() {
    let q = huba::quota::chatgpt_opencode_quota(&hang(
        "acc9",
        Some("opencode"),
        Some("openai/gpt-6.1-sol"),
    ));
    println!(
        "acc9: tuần {:?} (reset {:?}) · 5h {:?} · đọc lúc {:?} · không đo được: {:?}",
        q.week_pct, q.week_resets_at, q.hour5_pct, q.fetched_at_ms, q.why_unknown
    );
    assert!(q.why_unknown.is_none(), "{:?}", q.why_unknown);
    assert!(q.week_pct.is_some());
}

/// Hà 15/11 (ảnh `/accounts` acc9): *«sao lại có nội dung gán vào phiên ở danh sách tài
/// khoản»*. Dòng in `4 phiên: Đọc quy tắc dự án CLAUDE.md, Huba UI handoff continuation, …`
/// (tiêu đề opencode = nội dung câu đầu) · `API key (apiKeyHelper ngoài huba)` (sổ Claude
/// CŨ của acc9) · `model: nguồn không có hàng nào` (nguồn ChatGPT không bao giờ có bảng model).
#[test]
fn dong_accounts_cua_acc9_khong_mang_noi_dung_hay_so_cu() {
    use huba::sessions::LiveSession;
    // ① Phiên theo DỰ ÁN, gộp trùng, không phải tiêu đề.
    let p = |folder: &str, name: &str| LiveSession {
        folder: folder.into(),
        name: name.into(),
        ..Default::default()
    };
    let mut ui = p("huba", "Huba UI handoff continuation");
    ui.lane = "ui".into();
    let rows = [
        p("remote", "Đọc quy tắc dự án CLAUDE.md"),
        ui,
        p("dwork", "V-0194 M1 schedule UI continuation"),
        p("dwork", "Brief bàn giao dci-ke-nhiem-1511-v0194"),
        p("", "projects-f5"),
    ];
    let refs: Vec<&LiveSession> = rows.iter().collect();
    let s = huba::runtime::nhan_du_an_phien(&refs);
    assert_eq!(s, "remote, huba/ui, dwork ×2, projects-f5");
    assert!(!s.contains("CLAUDE.md") && !s.contains("Brief"), "{s}");

    // ② Câu khai mẫu số model chỉ cho nguồn CÓ bảng model.
    let than: serde_json::Value = serde_json::from_str(
        r#"{"rate_limit":{"limit_reached":false,"primary_window":{"used_percent":50,
            "limit_window_seconds":604800,"reset_at":1795235473},"secondary_window":null},
            "rate_limit_reached_type":null}"#,
    )
    .unwrap();
    let q = huba::quota::codex_quota_tu(
        "acc9",
        &huba::opencode::wham_sang_rate_limits(&than).unwrap(),
        "2026-11-15T14:00:00+00:00",
    );
    let now = 1_794_754_800_000;
    assert!(!q.say_cho(now, false).contains("nguồn không có hàng nào"));
    assert!(
        q.say(now).contains("nguồn không có hàng nào"),
        "ĐỐI CHỨNG: đường Claude vẫn khai mẫu số"
    );

    // ③ Email ChatGPT từ claims token (JWT dựng tay, khuôn claim đo 15/11).
    use base64::Engine;
    let b = |v: &str| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(v);
    let jwt = format!(
        "{}.{}.chuky",
        b(r#"{"alg":"RS256"}"#),
        b(r#"{"https://api.openai.com/profile":{"email":"chu@may.vn","email_verified":true}}"#)
    );
    let c = huba::opencode::jwt_claims(&jwt).unwrap();
    assert_eq!(
        huba::opencode::email_tu_claims(&c).as_deref(),
        Some("chu@may.vn")
    );
    assert_eq!(huba::opencode::jwt_claims("khong-phai-jwt"), None);

    // ④ Hàng opencode KHÔNG đọc sổ Claude cũ dù `settings.json` còn `apiKeyHelper`.
    let tmp = std::env::temp_dir().join(format!("huba-acc-cu-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(
        tmp.join("settings.json"),
        r#"{"apiKeyHelper":"/x/helper-ngoai.sh"}"#,
    )
    .unwrap();
    let mut cfg = common::cfg_for_tests();
    let mk = |ten: &str, cli: Option<&str>, model: Option<&str>| ClaudeAccountCfg {
        name: ten.into(),
        config_dir: Some(tmp.to_string_lossy().to_string()),
        cli: cli.map(str::to_string),
        model: model.map(str::to_string),
        ..Default::default()
    };
    cfg.claude_accounts = vec![
        mk(
            "acc8",
            Some("opencode"),
            Some("opencode-go/deepseek-v4.1-flash"),
        ),
        mk("accX", None, None),
    ];
    let ids = huba::account_login::danh_tinh_tat_ca(&cfg);
    let cua = |t: &str| {
        ids.iter()
            .find(|(n, _)| n == t)
            .and_then(|(_, e)| e.clone())
    };
    assert_eq!(
        cua("acc8").as_deref(),
        Some("opencode · gói Go (khoá trong env_file)")
    );
    assert_eq!(
        cua("accX").as_deref(),
        Some("API key (apiKeyHelper ngoài huba)"),
        "ĐỐI CHỨNG: hàng Claude cùng thư mục vẫn đọc apiKeyHelper"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}
