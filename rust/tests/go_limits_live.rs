//! Đo SỐNG hạn mức gói Go qua đúng đường Rust của huba (`opencode::go_limits`)
//! — gọi mạng, đọc khoá thật ⟹ `#[ignore]`, chỉ chạy tay:
//! `HUB_CONFIG=~/projects/huba/huba.config.json cargo test --test go_limits_live -- --ignored --nocapture`
//! In số, KHÔNG in khoá.

#[test]
#[ignore]
fn go_limits_that_cua_cac_acc_goi_go() {
    let cfg = huba::config::load(None).expect("HUB_CONFIG trỏ vào huba.config.json thật");
    let now = huba::quota::now_ms();
    let mut so = 0;
    for a in cfg
        .claude_accounts
        .iter()
        .filter(|a| huba::quota::is_go_plan(a))
    {
        let (luc, r) = huba::opencode::go_limits(a, now);
        let g = r.unwrap_or_else(|e| panic!("{}: KHÔNG ĐO ĐƯỢC — {e}", a.name));
        eprintln!(
            "{} @{luc}: rolling {}% · weekly {}% · monthly {}%",
            a.name, g.rolling.percent, g.weekly.percent, g.monthly.percent
        );
        let q = huba::quota::go_quota(&a.name, Ok(g), luc);
        eprintln!("  ⇒ hạng {:?}", huba::quota::rank(&q, now));
        so += 1;
    }
    assert!(
        so > 0,
        "không có acc gói Go nào trong cấu hình — phép đo mù"
    );
}
