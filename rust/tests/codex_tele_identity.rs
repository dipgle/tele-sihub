//! Regressions from acc10's real session_meta shape (14 Nov 2026).
use huba::codex_sessions::rollout_among;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../.tmp/codex-identity-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn rollout(&self, name: &str, id: &str, source: serde_json::Value) -> PathBuf {
        let p = self.0.join(format!("rollout-{name}.jsonl"));
        let v = serde_json::json!({"type":"session_meta", "payload":{"id":id,"cwd":"/mnt/data/projects","source":source}});
        std::fs::write(&p, format!("{v}\n")).unwrap();
        p
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn agent_con_moi_hon_khong_thay_phien_chinh() {
    let f = Files::new();
    let root = f.rollout("2026-11-14T23-47-23-root", "root", serde_json::json!("cli"));
    let child = f.rollout("2026-11-14T23-48-26-child", "child", serde_json::json!({"subagent":{"thread_spawn":{"parent_thread_id":"root","depth":1,"agent_path":"/root/audit_tele"}}}));
    assert_eq!(
        rollout_among(&[root.clone(), child], &f.0)
            .unwrap()
            .unwrap(),
        root,
        "agent con mở sau không được thay id/tty của phiên chính"
    );
}

#[test]
fn phien_chinh_ten_moi_hon_van_duoc_nhan() {
    let f = Files::new();
    let child = f.rollout(
        "a-child",
        "child",
        serde_json::json!({"subagent":{"thread_spawn":{"parent_thread_id":"root"}}}),
    );
    let root = f.rollout("z-root", "root", serde_json::json!("cli"));
    assert_eq!(
        rollout_among(&[child, root.clone(), root.clone()], &f.0)
            .unwrap()
            .unwrap(),
        root
    );
}

#[test]
fn codex_doc_dung_du_an_va_lan_tu_loi_phien() {
    let f = Files::new();
    std::fs::create_dir(f.0.join("dwork")).unwrap();
    let tail = serde_json::json!({"type":"response_item","payload":{
        "type":"message","role":"assistant","content":[{"type":"output_text","text":"[dwork/review] Đang kiểm tra."}]}}).to_string();
    let ws = f.0.to_str().unwrap();
    assert_eq!(
        huba::sessions::folder_declared(&tail, ws).as_deref(),
        Some("dwork")
    );
    assert_eq!(
        huba::sessions::lane_for_session("codex-tele-lane", &tail, ws, || None),
        "review"
    );
    let user = tail.replace("\"assistant\"", "\"user\"");
    assert_eq!(
        huba::sessions::folder_declared(&user, ws),
        None,
        "chủ máy nhắc dự án không phải phiên tự khai"
    );
}

#[test]
fn metadata_hong_khong_bien_thanh_khong_co_phien() {
    let f = Files::new();
    let root = f.rollout("a-root", "root", serde_json::json!("cli"));
    let broken = f.0.join("rollout-z-hong.jsonl");
    std::fs::write(&broken, "{\"type\":\"session_meta\",").unwrap();
    assert!(rollout_among(&[root.clone(), broken.clone()], &f.0).is_err());
    std::fs::remove_file(&broken).unwrap();
    assert!(
        rollout_among(&[root, broken], &f.0).is_err(),
        "fd đọc hỏng cũng phải báo chưa đo được"
    );
}

#[test]
fn chi_con_hoac_nhieu_root_la_chua_do_duoc() {
    let f = Files::new();
    let child = f.rollout(
        "child",
        "child",
        serde_json::json!({"subagent":{"thread_spawn":{"parent_thread_id":"root"}}}),
    );
    assert!(rollout_among(std::slice::from_ref(&child), &f.0).is_err());
    let root = f.rollout("root", "root", serde_json::json!("cli"));
    assert_eq!(
        rollout_among(&[child, root.clone()], &f.0),
        Ok(Some(root.clone()))
    );
    let other = f.rollout("other", "other", serde_json::json!("cli"));
    assert!(rollout_among(&[root, other], &f.0).is_err());
    assert_eq!(rollout_among(&[], &f.0), Ok(None));
}

#[test]
fn nguon_la_khong_duoc_chon_theo_ten() {
    let f = Files::new();
    let p = f.rollout("unknown", "root", serde_json::json!("new-source"));
    assert!(rollout_among(&[p], &f.0).is_err());
}

#[test]
fn ten_viec_la_de_bai_chu_may_khong_phai_luat_hay_tin_cuoi() {
    use huba::codex_sessions::user_task;
    let message = |text: &str| {
        serde_json::json!({"type":"response_item","payload":{
        "type":"message","role":"user","content":[{"type":"input_text","text":text}]}})
        .to_string()
    };
    let head = [
        message("# AGENTS.md instructions for /mnt/data/projects\n[remote] là ví dụ"),
        message("<environment_context>...</environment_context>"),
        message("huba: kiểm tra quản lý phiên acc10\nChi tiết"),
        message("Báo tiến độ đi"),
    ]
    .join("\n");
    assert_eq!(
        user_task(&head).as_deref(),
        Some("huba: kiểm tra quản lý phiên acc10")
    );
    assert_eq!(
        user_task(&message("<environment_context>...</environment_context>")),
        None
    );
    assert_eq!(user_task("partial json"), None);
}

#[test]
fn caption_bo_du_an_trung_nhung_giu_ten_chu_may_dat() {
    use huba::codex_sessions::task_caption;
    assert_eq!(
        task_caption("huba: Kiểm tra phiên acc10", "huba", ""),
        "Kiểm tra phiên acc10"
    );
    assert_eq!(
        task_caption("dwork/review: Rà soát", "dwork", "review"),
        "Rà soát"
    );
    assert_eq!(
        task_caption("OAuth: đo đăng nhập", "huba", ""),
        "OAuth: đo đăng nhập"
    );
    assert_eq!(task_caption("huba:", "huba", ""), "huba:");
}

#[test]
fn khong_dem_hai_lan_cung_loi_assistant_de_lat_du_an() {
    let f = Files::new();
    std::fs::create_dir(f.0.join("dwork")).unwrap();
    std::fs::create_dir(f.0.join("huba")).unwrap();
    let sid = "codex-dont-count-item-completed-twice";
    let msg = |text: &str| {
        serde_json::json!({"type":"response_item","payload":{
        "type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}})
        .to_string()
    };
    let ws = f.0.to_str().unwrap();
    assert_eq!(
        huba::sessions::folder_for_session(sid, &msg("[dwork] Đang làm."), ws, || None).as_deref(),
        Some("dwork")
    );
    let once = format!(
        "{}\n{}",
        msg("[huba] Nhắc một lần."),
        serde_json::json!({"type":"event_msg","payload":{
        "type":"item_completed","item":{"type":"AgentMessage","content":[{"type":"Text","text":"[huba] Nhắc một lần."}]}}})
    );
    assert_eq!(
        huba::sessions::folder_for_session(sid, &once, ws, || None).as_deref(),
        Some("dwork")
    );
    let twice = format!("{once}\n{}", msg("[huba] Lời khai thứ hai."));
    assert_eq!(
        huba::sessions::folder_for_session(sid, &twice, ws, || None).as_deref(),
        Some("huba")
    );
}

fn event(kind: &str) -> String {
    format!(
        "{}\n",
        serde_json::json!({"type":"event_msg","payload":{"type":kind}})
    )
}

fn append(path: &std::path::Path, text: &str) {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
}

#[test]
fn luot_dai_khong_thanh_dung_chi_vi_marker_troi_khoi_duoi() {
    use huba::codex_sessions::{parse_rollout, turn_busy};
    let f = Files::new();
    let p = f.rollout("root", "root", serde_json::json!("cli"));
    append(&p, &event("task_started"));
    let output = serde_json::json!({"type":"response_item","payload":{"type":"function_call_output","output":"x".repeat(300_000)}}).to_string();
    append(&p, &format!("{output}\n"));
    assert_eq!(
        parse_rollout("", &output).busy,
        None,
        "đối chứng: parser đuôi không thể biết marker đã trôi"
    );
    assert_eq!(turn_busy(&p), Ok(Some(true)));
    append(&p, &event("task_complete"));
    assert_eq!(turn_busy(&p), Ok(Some(false)));
    append(&p, &event("task_started"));
    assert_eq!(turn_busy(&p), Ok(Some(true)));
    append(&p, &event("turn_aborted"));
    assert_eq!(turn_busy(&p), Ok(Some(false)));
}

#[test]
fn dem_agent_dang_chay_khong_dem_agent_da_xong_con_fd() {
    use huba::codex_sessions::pending_children;
    let f = Files::new();
    let root = f.rollout("root", "root", serde_json::json!("cli"));
    let child = f.rollout(
        "child",
        "child",
        serde_json::json!({"subagent":{"thread_spawn":{"parent_thread_id":"root"}}}),
    );
    let other = f.rollout(
        "other",
        "other",
        serde_json::json!({"subagent":{"thread_spawn":{"parent_thread_id":"elsewhere"}}}),
    );
    append(&child, &event("task_started"));
    append(&other, &event("task_started"));
    let paths = vec![root, child.clone(), child.clone(), other];
    assert_eq!(pending_children(&paths, &f.0, "root"), Ok(1));
    append(&child, &event("task_complete"));
    assert_eq!(pending_children(&paths, &f.0, "root"), Ok(0));
    append(&child, &event("task_started"));
    assert_eq!(pending_children(&paths, &f.0, "root"), Ok(1));
    append(&child, &event("turn_aborted"));
    assert_eq!(pending_children(&paths, &f.0, "root"), Ok(0));
}

#[test]
fn khong_doc_duoc_child_khong_khai_khong_co_subagent() {
    use huba::codex_sessions::pending_children;
    let f = Files::new();
    let child = f.rollout(
        "child",
        "child",
        serde_json::json!({"subagent":{"thread_spawn":{"parent_thread_id":"root"}}}),
    );
    assert!(pending_children(std::slice::from_ref(&child), &f.0, "root").is_err());
    append(&child, &event("task_started"));
    assert_eq!(
        pending_children(std::slice::from_ref(&child), &f.0, "root"),
        Ok(1)
    );
}

#[test]
fn codex_dung_chung_man_hinh_trang_thai_voi_acc1() {
    use huba::sessions::{state_of, LiveSession, ST_RUN, ST_WAIT};
    let mut s = LiveSession {
        kind: "codex".into(),
        account: "acc10".into(),
        session_id: "01a1daf4-ded2-7dc3-bc43-fdf9ad9dd25c".into(),
        folder: "huba".into(),
        working: true,
        pending_subagents: 2,
        ..Default::default()
    };
    let text = huba::pipeline::session_list_text(std::slice::from_ref(&s), "", 0);
    assert_eq!(state_of(&s).0, ST_RUN);
    assert!(text.contains("2 subagent"), "{text}");
    let (html, _) = huba::pipeline::session_list_html(&text, std::slice::from_ref(&s));
    assert!(
        html.contains("01a1daf4"),
        "đích bấm phải giữ id phiên chính: {html}"
    );
    s.working = false;
    s.pending_subagents = 0;
    assert_eq!(state_of(&s).0, ST_WAIT);
    let text = huba::pipeline::session_list_text(std::slice::from_ref(&s), "", 0);
    assert!(
        !text.contains("subagent"),
        "agent đã xong phải mất bộ đếm: {text}"
    );
    s.kind = "interactive".into();
    s.account = "acc1".into();
    assert_eq!(state_of(&s).0, ST_WAIT, "cùng bảng trạng thái với acc1");
}

#[test]
fn chua_co_moc_luot_khac_voi_da_dung() {
    let mut s = huba::sessions::LiveSession {
        kind: "codex".into(),
        status: Some("unknown".into()),
        ..Default::default()
    };
    assert_eq!(huba::sessions::state_of(&s).0, huba::sessions::ST_UNKNOWN);
    s.status = Some("idle".into());
    assert_eq!(huba::sessions::state_of(&s).0, huba::sessions::ST_WAIT);
}
