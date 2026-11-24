//! GHIM TÊN PHIÊN xuống đĩa từ lần đầu thấy (Quang 23/11: *«hiện tại rất hay mất tên, từ đầu phiên đã quét được tên rồi phải gắn
//! cố định … thi thoảng đang làm nó nhảy sang tên khác rất là ảo»*).
//!
//! ĐO (log hubd, `session_change`, từ 13/11: 59/160 phiên đổi nhãn; từ 20/11: 10/25 — SAU vá `c1bb2ad`) — ba kiểu:
//! · mất ngoặc dự án: `[dwork/account]·user1-ubuntu` → `projects-90·user1-ubuntu`;
//! · đổi dự án trong ngoặc: `[projects]·dfef9718` → `[zkey]`, `[aihub]` → `[huba]·…`;
//! · đổi đuôi sau dấu `·`: `[dwork/dci]·user1-ubuntu` → `[dwork/dci]·2eafa487` (đuôi phụ thuộc BẠN CÙNG NHÃN đang sống).
//!
//! GỐC: `label_sessions` TÍNH LẠI nhãn mỗi lần chụp từ nhật ký + các phiên kế bên, và bộ nhớ «nhãn đã biết» (`FOLDER_MEMO`) nằm
//! trong RAM từng tiến trình — mất mỗi lần khởi động lại hubd (hôm nay đã 5 lần), và `huba ui`/CLI mỗi nơi một bản riêng.
//!
//! LUẬT (một lớp mỏng đặt SAU `label_sessions`, không đụng logic suy dự án):
//! 1. Lần đầu thấy một phiên ⟹ nhãn tính được LÚC ĐÓ được GHIM (đĩa: `data/ten-phien.json`, dùng chung hubd · CLI · `huba ui`).
//! 2. Có ghim ⟹ hiện ghim, bỏ qua nhãn tính lại. Hai ngoại lệ DUY NHẤT: (a) ghim là TRUNG TÍNH («chưa giao dự án»: không ngoặc, hoặc
//!    ngoặc là tên gốc workspace) mà nhãn mới có dự án THẬT ⟹ nâng lên đúng MỘT lần rồi ghim luôn; (b) chủ máy tự đặt/bỏ (`huba ten-phien`).
//! 3. Hai phiên sống trùng nhãn ⟹ phiên ĐẾN SAU mang đuôi `·<mã>` (và ghim luôn); phiên đến trước KHÔNG bị đổi.
//! 4. Cửa sổ trống (`host == "shell"`) không ghim (tty bị tái dùng). Ghim không thấy > 14 ngày thì bỏ. Tệp hỏng ⟹ học lại và NÓI RA.
//!
//! ⚠ Đảo chiều quyết định 05/09 («luôn gắn đuôi việc đang làm, đuôi đổi theo việc»): đuôi nay đóng băng ở lần đầu thấy.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::sessions::LiveSession;

/// Ghim của phiên không thấy quá chừng này thì bỏ.
const GIU_GIAY: i64 = 14 * 86_400;
/// Làm mới mốc «đã thấy» không thường hơn chừng này (đỡ ghi đĩa mỗi vòng).
const LAM_MOI_GIAY: i64 = 3_600;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    pub label: String,
    /// Lần cuối thấy phiên (epoch giây).
    pub seen: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PinBook {
    #[serde(default)]
    pub pins: BTreeMap<String, Pin>,
}

impl PinBook {
    /// Đọc sổ. Vắng tệp = sổ rỗng, KHÔNG phải lỗi. Tệp hỏng = sổ rỗng + lời cảnh báo (chỗ gọi phải ghi log — không im lặng).
    pub fn load(path: &Path) -> (PinBook, Option<String>) {
        match std::fs::read_to_string(path) {
            Ok(s) => match serde_json::from_str::<PinBook>(&s) {
                Ok(b) => (b, None),
                Err(e) => (
                    PinBook::default(),
                    Some(format!("sổ tên phiên hỏng ({}): {e}", path.display())),
                ),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (PinBook::default(), None),
            Err(e) => (
                PinBook::default(),
                Some(format!(
                    "không đọc được sổ tên phiên ({}): {e}",
                    path.display()
                )),
            ),
        }
    }

    /// Ghi nguyên tử (tệp tạm cùng thư mục rồi đổi tên) — hubd, CLI và `huba ui` cùng đọc nên không được để lộ tệp ghi dở.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("tạo {}", dir.display()))?;
        }
        let tmp = path.with_extension(format!("json.tmp{}", std::process::id()));
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)
            .with_context(|| format!("ghi {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("đổi tên {} → {}", tmp.display(), path.display()))?;
        Ok(())
    }

    /// Chủ máy tự đặt tên (thắng mọi nhãn tính lại).
    pub fn set(&mut self, id: &str, label: &str, now: i64) {
        self.pins.insert(
            id.to_string(),
            Pin {
                label: label.to_string(),
                seen: now,
            },
        );
    }

    /// Bỏ ghim (phiên sẽ học lại từ nhãn tính được lần kế). `false` nếu vốn không có.
    pub fn remove(&mut self, id: &str) -> bool {
        self.pins.remove(id).is_some()
    }

    /// Tìm khoá theo TIỀN TỐ duy nhất (chủ máy gõ 8 ký tự đầu). `Err` khi không có hoặc nhiều hơn một.
    pub fn find(&self, prefix: &str) -> Result<String, String> {
        let hits: Vec<&String> = self.pins.keys().filter(|k| k.starts_with(prefix)).collect();
        match hits.as_slice() {
            [one] => Ok((*one).clone()),
            [] => Err(format!(
                "không có phiên nào bắt đầu bằng «{prefix}» trong sổ ghim"
            )),
            many => Err(format!("«{prefix}» khớp {} phiên — gõ dài hơn", many.len())),
        }
    }
}

/// Nhãn «chưa giao dự án»: không có ngoặc dự án (tên tự sinh / tên việc), hoặc ngoặc trỏ vào CHÍNH gốc workspace
/// (`[projects]`, `[projects/dci]`) — nghĩa là phiên mở ở gốc và chưa ai giao dự án nào.
pub fn is_neutral(label: &str, root_name: &str) -> bool {
    let l = label.trim();
    let Some(rest) = l.strip_prefix('[') else {
        return true;
    };
    let inner = rest.split(']').next().unwrap_or("");
    let project = inner.split('/').next().unwrap_or("");
    project == root_name
}

/// `true` khi ghim chỉ có dự án (`[dwork]`, đuôi nào cũng được) còn nhãn mới là CHÍNH dự án ấy kèm làn (`[dwork/dci]`).
/// Chỉ một chiều: nhãn đã có làn thì không bao giờ đổi làn, và dự án khác (`[dworker/x]`) không khớp.
fn them_lan_cung_du_an(ghim: &str, moi: &str) -> bool {
    let trong = |l: &str| -> Option<String> {
        let rest = l.trim().strip_prefix('[')?;
        Some(rest.split(']').next()?.to_string())
    };
    let (Some(g), Some(m)) = (trong(ghim), trong(moi)) else {
        return false;
    };
    !g.is_empty()
        && !g.contains('/')
        && m.strip_prefix(g.as_str())
            .and_then(|r| r.strip_prefix('/'))
            .is_some_and(|lan| !lan.trim().is_empty())
}

/// Áp ghim lên các hàng. Trả `true` khi sổ có đổi (chỗ gọi mới ghi đĩa).
pub fn apply_pins(rows: &mut [LiveSession], book: &mut PinBook, root_name: &str, now: i64) -> bool {
    let mut changed = false;
    // 1. ghim / áp ghim từng hàng
    for r in rows.iter_mut() {
        if r.host == "shell" || r.session_id.is_empty() || r.label.trim().is_empty() {
            continue;
        }
        match book.pins.get_mut(&r.session_id) {
            None => {
                book.pins.insert(
                    r.session_id.clone(),
                    Pin {
                        label: r.label.clone(),
                        seen: now,
                    },
                );
                changed = true;
            }
            Some(p) => {
                if is_neutral(&p.label, root_name) && !is_neutral(&r.label, root_name) {
                    // được giao dự án THẬT lần đầu: nâng đúng một lần rồi ghim luôn
                    p.label = r.label.clone();
                    p.seen = now;
                    changed = true;
                } else if them_lan_cung_du_an(&p.label, &r.label) {
                    // Hà 24/11 («Tên vẫn thiếu», `[dwork]` thay vì `[dwork/dci]`): phiên mới mở chưa nói gì nên ghim
                    // `[dwork]`; làn đến sau là chi tiết CÙNG dự án, không phải đổi danh tính ⟹ nâng đúng một lần.
                    p.label = r.label.clone();
                    p.seen = now;
                    changed = true;
                } else {
                    r.label = p.label.clone();
                    if now - p.seen >= LAM_MOI_GIAY {
                        p.seen = now;
                        changed = true;
                    }
                }
            }
        }
    }
    // 2. trùng nhãn giữa các phiên đang sống: người đến SAU mang đuôi mã, người đến trước giữ nguyên
    let mut theo_nhan: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, r) in rows.iter().enumerate() {
        if r.host != "shell" && !r.session_id.is_empty() && !r.label.trim().is_empty() {
            theo_nhan.entry(r.label.clone()).or_default().push(i);
        }
    }
    for (nhan, mut ids) in theo_nhan {
        if ids.len() < 2 {
            continue;
        }
        ids.sort_by(|a, b| {
            (rows[*a].started_at_ms, &rows[*a].session_id)
                .cmp(&(rows[*b].started_at_ms, &rows[*b].session_id))
        });
        for i in ids.into_iter().skip(1) {
            let moi = format!("{nhan}·{}", crate::sessions::short_id(&rows[i].session_id));
            rows[i].label = moi.clone();
            if let Some(p) = book.pins.get_mut(&rows[i].session_id) {
                p.label = moi;
            }
            changed = true;
        }
    }
    // 3. bỏ ghim của phiên đã lâu không thấy
    let truoc = book.pins.len();
    book.pins.retain(|_, p| now - p.seen <= GIU_GIAY);
    changed || book.pins.len() != truoc
}

/// Đọc sổ ở `path`, áp ghim, ghi lại nếu có đổi. Lỗi đọc/ghi được NÓI RA trong log, không làm hỏng ảnh chụp.
pub fn pin_labels_at(rows: &mut [LiveSession], path: &Path, root_name: &str, now: i64) {
    let (mut book, canh_bao) = PinBook::load(path);
    if let Some(c) = canh_bao {
        crate::logging::warn("ten_phien_book_unreadable", serde_json::json!({ "why": c }));
    }
    if apply_pins(rows, &mut book, root_name, now) {
        if let Err(e) = book.save(path) {
            crate::logging::warn(
                "ten_phien_book_save_failed",
                serde_json::json!({ "path": path.display().to_string(), "err": format!("{e:#}") }),
            );
        }
    }
}
