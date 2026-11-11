//! Mở một phiên THẬT qua đúng đường `/new` (không Telegram) — để nghiệm thu.
//! Dùng: cargo run --offline --example mo-phien -- <account> <task...>
fn main() {
    let mut args = std::env::args().skip(1);
    let account = args.next().unwrap_or_else(|| "acc8".into());
    let task = args.collect::<Vec<_>>().join(" ");
    let cfg = huba::config::load(None).expect("config");
    let root = cfg.workspace_root.clone();
    let s = huba::sessions::start_background(&cfg, "", &root, &task, Some(&account), None, false)
        .expect("mở phiên");
    println!(
        "STARTED session={} window={} cwd={}",
        s.session_id, s.window, s.cwd
    );
}
