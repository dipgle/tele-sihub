//! Gõ vào phiên opencode không được dựng lại cả ảnh chụp chỉ để tìm cửa sổ.
//!
//! 🔴 Hà 24/11: *"opencode nhận … tin vẫn chậm"*. ĐO (nhật ký hubd, lượt `1+1=` lúc 15:07Z): tin vào 15:07:19.456 →
//! `keys_typed` 15:07:23.852 → `command_done` 5485 ms; phiên Claude cùng lúc 1,4 s. Đường nhanh `window_target_from_book`
//! đòi `pid` > 0, hàng opencode có `pid` 0 ⟹ luôn rơi về `sessions::snapshot` (~3,1 s).
//!
//! Đường nhanh cho opencode dùng đúng bằng chứng mà chính ảnh chụp dùng để gán `tty`: sổ liên kết + TUI còn sống.
//! Năm phép đo: ① đủ ba điều ⟹ có cửa sổ (RED trên mã cũ) · ②–⑤ thiếu một điều ⟹ `None` (đi đường chậm, KHÔNG đoán).

use std::collections::{BTreeMap, HashSet};

use huba::opencode::WindowBinding;
use huba::sessions::opencode_target_from_parts;

const ID: &str = "ses_df1e8475afffkFIa0jxvQL6tbe";

fn so(mark_y: &str, n: &str) -> String {
    serde_json::json!({
        ID: {"s": "idle", "y": mark_y, "k": "opencode", "n": n, "a": "acc8", "i": 0, "o": "terminal",
             "d": "projects", "l": "🟪 [projects]", "f": 1}
    })
    .to_string()
}

fn lien_ket(tty: &str, at_ms: i64) -> WindowBinding {
    WindowBinding {
        window: 156,
        tty: tty.into(),
        at_ms,
        dir: None,
    }
}

fn song(ttys: &[&str]) -> HashSet<String> {
    ttys.iter().map(|s| s.to_string()).collect()
}

/// ① CA CỦA HÀ — liên kết mới nhất + TUI sống ⟹ có cửa sổ, nhãn lấy từ sổ.
#[test]
fn du_ba_dieu_thi_co_cua_so_khong_can_anh_chup() {
    let b = BTreeMap::from([(ID.to_string(), lien_ket("pts/12", 100))]);
    let r =
        opencode_target_from_parts(&so("pts/12", "Simple addition"), ID, &b, &song(&["pts/12"]))
            .expect("phải có cửa sổ");
    assert_eq!(r.session_id, ID);
    assert_eq!(r.tty, "pts/12");
    assert_eq!(r.account, "acc8");
    assert_eq!(r.label, "[projects]", "nhãn phải bỏ ô màu");
}

/// ② TUI đã tắt ⟹ không cửa sổ (đi đường chậm để nói «không thấy phiên»).
#[test]
fn tui_da_tat_thi_khong_co_cua_so() {
    let b = BTreeMap::from([(ID.to_string(), lien_ket("pts/12", 100))]);
    assert!(opencode_target_from_parts(&so("pts/12", "x"), ID, &b, &song(&["pts/18"])).is_none());
}

/// ③ tty được dùng lại: có liên kết MỚI HƠN của phiên khác trên cùng tty ⟹ phiên này đã rời cửa sổ.
#[test]
fn co_lien_ket_moi_hon_tren_cung_tty_thi_khong_muon_cua_so() {
    let b = BTreeMap::from([
        (ID.to_string(), lien_ket("pts/12", 100)),
        ("ses_phienkia".to_string(), lien_ket("/dev/pts/12", 200)),
    ]);
    assert!(opencode_target_from_parts(&so("pts/12", "x"), ID, &b, &song(&["pts/12"])).is_none());
}

/// ④ chưa có liên kết ⟹ không đoán.
#[test]
fn khong_co_lien_ket_thi_khong_doan() {
    let b = BTreeMap::new();
    assert!(opencode_target_from_parts(&so("pts/12", "x"), ID, &b, &song(&["pts/12"])).is_none());
}

/// ⑤ sổ theo dõi chưa biết phiên này (hoặc không tên) ⟹ để đường chậm lo.
#[test]
fn so_theo_doi_chua_biet_phien_thi_khong_co_cua_so() {
    let b = BTreeMap::from([(ID.to_string(), lien_ket("pts/12", 100))]);
    assert!(opencode_target_from_parts("{}", ID, &b, &song(&["pts/12"])).is_none());
    assert!(opencode_target_from_parts(&so("pts/12", ""), ID, &b, &song(&["pts/12"])).is_none());
}
