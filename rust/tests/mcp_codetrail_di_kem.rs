//! Quang 13/11: *"cài cả mcp nữa, huba luôn đi kèm mcp codetrail"* — sau khi opencode
//! acc8 trên Ubuntu báo «1 MCP failed»: hàng acc trỏ `~/projects/AI/codetrail/…`, thứ
//! không có (hoặc là binary macOS) trên máy Linux.
use huba::accounts::chen_mcp_codetrail;
use serde_json::json;

#[test]
fn hang_acc_da_khai_chi_thay_chuong_trinh() {
    let mut root = json!({ "mcp": { "project-agent": {
        "type": "local",
        "command": ["/home/u/projects/AI/codetrail/mcp/project-agent-rs/target/release/project-agent", "--x"],
        "environment": { "PROJECTS_ROOT": "/home/u/projects" }
    }, "khac": { "type": "local", "command": ["khac"] } } });
    chen_mcp_codetrail(
        &mut root,
        "/home/u/huba/kem/bin/project-agent",
        "/home/u/projects",
    );
    let pa = &root["mcp"]["project-agent"];
    assert_eq!(
        pa["command"],
        json!(["/home/u/huba/kem/bin/project-agent", "--x"])
    );
    assert_eq!(pa["environment"]["PROJECTS_ROOT"], "/home/u/projects");
    // MCP khác không bị đụng.
    assert_eq!(root["mcp"]["khac"]["command"], json!(["khac"]));
}

#[test]
fn chua_khai_thi_them_moi_che_do_router() {
    let mut root = json!({ "model": "m" });
    chen_mcp_codetrail(&mut root, "/b/project-agent", "/home/u/projects");
    assert_eq!(
        root["mcp"]["project-agent"],
        json!({ "type": "local", "command": ["/b/project-agent"],
                "environment": { "PROJECTS_ROOT": "/home/u/projects" } })
    );
    assert_eq!(root["model"], "m");
}

#[test]
fn lenh_rong_thanh_mot_chuong_trinh() {
    let mut root = json!({ "mcp": { "project-agent": { "type": "local", "command": [] } } });
    chen_mcp_codetrail(&mut root, "/b/pa", "/p");
    assert_eq!(root["mcp"]["project-agent"]["command"], json!(["/b/pa"]));
}
