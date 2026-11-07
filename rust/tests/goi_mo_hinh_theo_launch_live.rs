//! THĂM DÒ trên máy thật: dòng `launch` THẬT của từng tài khoản nhiều từ có
//! chạy được qua `sessions::account_call` không — bằng `claude --version` (không
//! gọi mô hình, không tốn hạn mức, không in khoá).
//!
//! `cargo test --offline --test goi_mo_hinh_theo_launch_live -- --ignored --nocapture`
//!
//! Mẫu số = tài khoản có `launch` nhiều từ; 0 tài khoản ⟹ KHÔNG ĐO ĐƯỢC (panic
//! riêng), không phải xanh.
#![cfg(unix)]

use huba::sessions;

#[test]
#[ignore]
fn dong_launch_that_chay_duoc_qua_account_call() {
    let cfg = huba::config::load(None).expect("đọc huba.config.json");
    let mut mau = 0usize;
    let mut hong = Vec::new();
    for a in &cfg.claude_accounts {
        let (prog, argv) = sessions::account_call(&cfg, Some(&a.name), &["--version"]);
        if prog == cfg.claude_cli {
            continue;
        }
        mau += 1;
        let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
        let out = huba::exec::run(&prog, &argv, Default::default()).expect("chạy");
        let dong = out.stdout.lines().next().unwrap_or("").to_string();
        println!("{} ⟹ {prog} · mã {:?} · «{dong}»", a.name, out.code);
        if out.code != Some(0) || !dong.contains("Claude Code") {
            hong.push(a.name.clone());
        }
        // `auth status` qua CÙNG cửa `/accounts -k` dùng — chỉ in các trường xác
        // thực, không in email/khoá.
        match huba::api_key::auth_status(&cfg, a) {
            Ok(v) => {
                let truong = |k: &str| v.get(k).cloned().unwrap_or_default();
                println!(
                    "{} auth status ⟹ loggedIn {} · authMethod {} · apiKeySource {}",
                    a.name,
                    truong("loggedIn"),
                    truong("authMethod"),
                    truong("apiKeySource")
                );
                if v.get("loggedIn").and_then(|x| x.as_bool()) != Some(true) {
                    hong.push(format!("{} (auth)", a.name));
                }
            }
            Err(e) => {
                println!("{} auth status ⟹ HỎNG {e}", a.name);
                hong.push(format!("{} (auth)", a.name));
            }
        }
    }
    println!("MAU_SO={mau} HONG={}", hong.len());
    assert!(
        mau > 0,
        "KHÔNG ĐO ĐƯỢC: không tài khoản nào có launch nhiều từ"
    );
    assert!(hong.is_empty(), "launch thật không chạy: {hong:?}");
}
