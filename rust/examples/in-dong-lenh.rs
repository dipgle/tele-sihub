//! In dòng launch THẬT của một tài khoản — để đo bằng mắt trước khi mở cửa sổ.
//! Dùng: cargo run --offline --example in-dong-lenh -- acc8 [--window]
fn main() {
    let cfg = huba::config::load(None).expect("config");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let name = args.first().cloned().unwrap_or_else(|| "acc8".into());
    let row = cfg.claude_accounts.iter().find(|a| a.name == name);
    if args.iter().any(|a| a == "--window") {
        println!(
            "{}",
            huba::sessions::window_command(&cfg, Some(&name), &cfg.workspace_root, None, None)
        );
    } else {
        println!("{}", huba::accounts::effective_launch(&cfg, row));
    }
}
