//! Profile XDG khác nhau vẫn cùng cổng mặc định 49374 trên opencode 2.2.7.
//! /new và fork phải dùng máy chủ riêng, giữ cấu hình/quyền của chính lượt mở.
#[cfg(unix)]
use huba::sessions;
use huba::{
    accounts,
    config::{ClaudeAccountCfg, Config},
};

#[test]
fn cac_acc_opencode_mo_may_chu_rieng_tren_hai_he() {
    let cfg = Config::default();
    for name in ["acc8", "acc9"] {
        let acc = ClaudeAccountCfg {
            name: name.into(),
            cli: Some("opencode".into()),
            model: Some("opencode-go/deepseek-v4.1-flash".into()),
            ..Default::default()
        };
        let sh = accounts::launch_line(&cfg, Some(&acc));
        let ps = accounts::launch_line_ps(&cfg, Some(&acc));
        assert_eq!(sh.matches("--standalone").count(), 1);
        assert_eq!(ps.matches("--standalone").count(), 1);
        assert!(sh.contains("OPENCODE_CONFIG_CONTENT="));
        assert!(sh.contains("XDG_STATE_HOME="));
        let cfg = Config {
            claude_accounts: vec![acc],
            ..Default::default()
        };
        #[cfg(unix)]
        {
            let (_, argv) = sessions::account_call(
                &cfg,
                Some(name),
                &["run", "--fork", "--session", "ses_old", "question"],
            );
            assert!(
                argv.iter().any(|s| s.contains("--standalone")),
                "fork cũng phải nhận cấu hình mới"
            );
        }
        #[cfg(not(unix))]
        let _ = cfg;
    }
}

#[test]
fn server_chu_may_chon_thi_giu_va_khong_lap_co_standalone() {
    let mut acc = ClaudeAccountCfg {
        name: "acc9".into(),
        cli: Some("opencode".into()),
        extra_args: vec!["--server".into(), "http://127.0.0.1:12345".into()],
        ..Default::default()
    };
    let cfg = Config::default();
    for render in [accounts::launch_line, accounts::launch_line_ps] {
        let line = render(&cfg, Some(&acc));
        assert!(line.contains("--server"));
        assert!(!line.contains("--standalone"));
    }
    acc.extra_args = vec!["--standalone".into()];
    for render in [accounts::launch_line, accounts::launch_line_ps] {
        assert_eq!(render(&cfg, Some(&acc)).matches("--standalone").count(), 1);
    }
}
