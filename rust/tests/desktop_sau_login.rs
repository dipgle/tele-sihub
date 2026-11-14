#![cfg(unix)]

use huba::keys_tmux::{co_man_hinh, moi_truong_cua_so};

#[test]
fn daemon_truoc_login_doc_duoc_desktop_sau_login() {
    // Ca thật: hubd chỉ có runtime/bus, GNOME đã nhập display vào systemd.
    let env = |k: &str| (k == "XDG_RUNTIME_DIR").then(|| "/run/user/1000".into());
    let moi = moi_truong_cua_so(env, r#"{"DISPLAY":":7","WAYLAND_DISPLAY":"wayland-9","XAUTHORITY":"/run/user/1000/auth with space","PATH":"/untrusted","HUB_TELEGRAM_BOT_TOKEN":"not-a-real-secret"}"#).unwrap();
    assert!(
        co_man_hinh(|k| moi.get(k).cloned()),
        "desktop đã login nhưng bị bỏ qua"
    );
    assert_eq!(moi["WAYLAND_DISPLAY"], "wayland-9");
    assert_eq!(moi["XAUTHORITY"], "/run/user/1000/auth with space");
    assert_eq!(moi["XDG_RUNTIME_DIR"], "/run/user/1000");
    assert!(!moi.contains_key("PATH"));
    assert!(!moi.contains_key("HUB_TELEGRAM_BOT_TOKEN"));
}

#[test]
fn desktop_moi_thang_env_cu_va_khong_cache_lan_doc() {
    let env = |k: &str| (k == "WAYLAND_DISPLAY").then(|| "wayland-old".into());
    for display in ["wayland-3", "wayland-8"] {
        let json = format!(r#"{{"WAYLAND_DISPLAY":"{display}"}}"#);
        assert_eq!(
            moi_truong_cua_so(env, &json).unwrap()["WAYLAND_DISPLAY"],
            display
        );
    }
}

#[test]
fn chua_login_khong_bia_man_hinh() {
    let moi = moi_truong_cua_so(|_| None, r#"{"PATH":"/usr/bin","DISPLAY":" "}"#).unwrap();
    assert!(moi.is_empty());
    assert!(!co_man_hinh(|k| moi.get(k).cloned()));
}

#[test]
fn doc_loi_khong_duoc_coi_la_desktop_rong() {
    assert!(moi_truong_cua_so(|_| None, "not json").is_err());
    assert!(moi_truong_cua_so(|_| None, r#"{"DISPLAY":3}"#).is_err());
}

#[test]
#[ignore = "mở terminal thật vào pane có sẵn; cần HUBA_DESKTOP_PANE và desktop đã login"]
fn mo_terminal_that_tu_env_truoc_login() {
    use huba::exec::{run, RunOpts};
    assert!(
        !co_man_hinh(|k| std::env::var(k).ok()),
        "chạy với env -u DISPLAY -u WAYLAND_DISPLAY để tái hiện hubd trước login"
    );
    let pane: i64 = std::env::var("HUBA_DESKTOP_PANE")
        .expect("pane thật cần mở")
        .parse()
        .unwrap();
    huba::keys_tmux::bring_to_front(pane).unwrap();
    let target = format!("=huba-xem-{pane}");
    let clients = run(
        "tmux",
        &["list-clients", "-t", &target, "-F", "#{client_tty}"],
        RunOpts::default(),
    )
    .unwrap();
    assert!(
        clients.ok() && !clients.stdout.trim().is_empty(),
        "terminal chưa gắn: {clients:?}"
    );
    let selected = run(
        "tmux",
        &[
            "display-message",
            "-p",
            "-t",
            &format!("{target}:"),
            "#{pane_id}",
        ],
        RunOpts::default(),
    )
    .unwrap();
    assert!(selected.ok());
    assert_eq!(selected.stdout.trim(), format!("%{pane}"));
    println!("TERMINAL_CLIENT={} PANE=%{pane}", clients.stdout.trim());
}
