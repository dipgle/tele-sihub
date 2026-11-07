//! THĂM DÒ trên máy thật: dòng `launch` THẬT của từng tài khoản nhiều từ có
//! chạy được qua `sessions::model_call` không — bằng `claude --version` (không
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
fn dong_launch_that_chay_duoc_qua_model_call() {
    let cfg = huba::config::load(None).expect("đọc huba.config.json");
    let mut mau = 0usize;
    let mut hong = Vec::new();
    for a in &cfg.claude_accounts {
        let (prog, argv) = sessions::model_call(&cfg, Some(&a.name), &["--version"]);
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
    }
    println!("MAU_SO={mau} HONG={}", hong.len());
    assert!(
        mau > 0,
        "KHÔNG ĐO ĐƯỢC: không tài khoản nào có launch nhiều từ"
    );
    assert!(hong.is_empty(), "launch thật không chạy: {hong:?}");
}
