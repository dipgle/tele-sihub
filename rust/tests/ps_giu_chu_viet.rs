//! `ps` qua `exec::run` phải trả chữ tiếng Việt NGUYÊN VĂN dù tiến trình gọi
//! không có locale — đúng hoàn cảnh `hubd` dưới launchd.
//!
//! 🔴 Quang 13/11 «rất nhiều phiên đang chạy acc8 nhưng danh sách phiên lại không
//! có»: hubd không ghép được cửa sổ theo đề bài `--prompt` vì `ps` (thiếu `LANG`)
//! in `đọc` thành `M-DM^QM-aM-;M^Mc`; `huba sessions` chạy tay trong shell thì
//! ghép được — xem `exec::locale_cho`.
#![cfg(unix)]

use std::process::Command;
use std::time::Duration;

use huba::exec::{run, RunOpts};

#[test]
fn ps_khong_ma_hoa_chu_viet_khi_thieu_locale() {
    let mut con = Command::new("/bin/sh")
        .args(["-c", "sleep 20; :", "đọc-brief-thử"])
        .spawn()
        .expect("dựng tiến trình thử");
    std::thread::sleep(Duration::from_millis(200));
    let pid = con.id().to_string();
    let out = run(
        "ps",
        &["-o", "command=", "-p", &pid],
        RunOpts {
            timeout: Some(Duration::from_secs(10)),
            // Như launchd: không có locale nào. (`LC_ALL` là thứ `exec::run` tự
            // đặt cho `ps`; xoá nó ở đây là gỡ chính bản vá.)
            env: vec![("LANG".to_string(), None), ("LC_CTYPE".to_string(), None)],
            ..Default::default()
        },
    );
    let _ = con.kill();
    let _ = con.wait();
    let out = out.expect("chạy ps");
    assert!(
        out.stdout.contains("đọc-brief-thử"),
        "ps phải giữ nguyên chữ, nhận được: {:?}",
        out.stdout
    );
}
