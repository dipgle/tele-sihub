//! Liên kết chạm được của phiên opencode phải đi trọn vòng: mã dựng ra
//! (`sessions::ma_lien_ket`) ⟹ bộ đọc lệnh (`verbs::parse_command`) ⟹ tra lại
//! đúng phiên (`pipeline::same_session`).
//!
//! 🔴 Quang 13/11 (mac): bấm «A — đúng» ở tin hỏi của phiên acc8 ⟹ log
//! `telegram_not_a_command {"head":"/start k_ses_e2b5_1"}` — mã cắt 8 ký tự đầu
//! của id nguyên ra `ses_e2b5`, bộ đọc tách ở dấu `_` ⟹ mọi nút của phiên
//! opencode câm.

use huba::adapters::CommandKind;
use huba::pipeline::same_session;
use huba::sessions::ma_lien_ket;
use huba::verbs::parse_command;

const OC: &str = "ses_e2b5ef6d7fffuELciNJ4fVSRFa";
const CLAUDE: &str = "f75d5ead-01f3-49e0-8aa4-ac553db3834f";

fn vong_tron(id: &str, lenh: &str, cho: &str) {
    let parsed = parse_command(lenh).unwrap_or_else(|| panic!("{lenh} phải là một lệnh"));
    assert!(
        matches!(parsed.0, CommandKind::Key | CommandKind::Pick),
        "{lenh} ⟹ {parsed:?}"
    );
    let sid = parsed.2.split_whitespace().next().unwrap_or_default();
    assert_eq!(sid, cho, "{lenh} ⟹ {parsed:?}");
    assert!(same_session(id, sid), "{sid} phải tra ra {id}");
}

#[test]
fn nut_lua_chon_cua_phien_opencode_di_tron_vong() {
    let m = ma_lien_ket(OC);
    assert_eq!(m, "e2b5ef6d");
    vong_tron(OC, &format!("/start k_{m}_1"), &m);
    vong_tron(OC, &format!("/k_{m}_2"), &m);
    vong_tron(OC, &format!("/send_{m}"), &m);
    vong_tron(OC, &format!("/start clr_{m}"), &m);
}

#[test]
fn nut_cua_phien_claude_khong_doi() {
    let m = ma_lien_ket(CLAUDE);
    assert_eq!(m, "f75d5ead");
    vong_tron(CLAUDE, &format!("/start k_{m}_1"), &m);
}

#[test]
fn ma_ngan_opencode_khong_khop_phien_khac() {
    assert!(!same_session("ses_e2b5ef6dXXXXuELciNJ4fVSRFa", "e2b5ef6e"));
    assert!(!same_session(CLAUDE, "e2b5ef6d"));
}
