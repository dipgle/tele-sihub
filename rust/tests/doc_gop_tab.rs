//! Số Apple event không được tăng theo số tab (15/11: Terminal dưới tải).
//! Chỉ duyệt dữ liệu đã đọc gộp; inspector vẫn được cô lập theo cửa sổ.

fn than<'a>(src: &'a str, name: &str) -> &'a str {
    let start = src.find(name).expect("phải có hàm để kiểm");
    let rest = &src[start..];
    &rest[..rest.find("\n}\n").expect("phải có cuối hàm")]
}

fn doc_gop(src: &str) -> bool {
    [
        "fn window_script(",
        "fn window_any_script(",
        "fn tabs_script(",
    ]
    .iter()
    .all(|name| {
        let body = than(src, name)
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        body.contains("tty of every tab")
            && body.contains("busy of every tab")
            && body.contains("processes of every tab")
            && !body.contains("tty of t")
            && !body.contains("busy of t")
            && !body.contains("processes of t")
            && !body.contains("contents of tab k")
            && !body.contains("custom title of tb")
            && !body.contains("count of tabs of w")
    })
}

#[test]
fn ba_duong_quet_khong_gui_apple_event_cho_tung_tab() {
    assert!(doc_gop(include_str!("../src/keys.rs")));
}

#[test]
fn cong_bat_duoc_ban_cu_va_chap_nhan_doc_gop() {
    let lanh = ["window_script", "window_any_script", "tabs_script"]
        .map(|name| format!("fn {name}() {{\n tty of every tab; busy of every tab; processes of every tab\n}}\n"))
        .join("");
    assert!(doc_gop(&lanh));
    for getter in [
        "tty of t",
        "busy of tb",
        "processes of t",
        "contents of tab k",
        "custom title of tb",
        "count of tabs of w",
    ] {
        let hong = lanh.replacen("tty of every tab", getter, 1);
        assert!(!doc_gop(&hong), "lọt Apple event mỗi tab: {getter}");
    }
}
