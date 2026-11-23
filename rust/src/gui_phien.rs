//! `huba gui <phiên> "<chữ>"` — một phiên NHỜ phiên khác làm việc, đi qua huba.
//!
//! 🔴 Quang 19/11: *«mọi giao tiếp phiên qua huba chứ»* — sau khi một phiên nhắn
//! thẳng phiên khác bằng `SendMessage` suốt hai ngày. Nhưng tới 23/11 huba chỉ có
//! `ask` (hỏi một BẢN SAO, không tới phiên thật) và `effort`; việc «nhờ làm» còn
//! đúng một đường là tay chủ máy gõ `/session` → `/type` → `/enter` trên Telegram.
//! Ảnh Quang gửi 23/11: phiên `tfl5/may-chu` phải soạn sẵn ba lệnh ấy cho anh gõ.
//! Quang chọn 23/11: *gửi thẳng, báo Tele* — không chờ anh bấm (cùng tinh thần hòm
//! thư `huba-run.txt`), nhưng mỗi lần gửi anh đều THẤY trên Telegram.
//!
//! Lõi gõ là lõi của route `/type` và nút ▶️ (`keys::type_and_send`: cú Enter RỜI,
//! nhìn rồi mới bấm). Mô-đun này chỉ giữ phần QUYẾT ĐỊNH — thuần, để bài kiểm khoá
//! được: gửi cho ai, chữ gì, khi nào KHÔNG được gõ.

use crate::keys::Delivered;
use crate::sessions::{LiveSession, Ready};

/// Trần độ dài chữ gửi. Một lời nhờ dài hơn thế là một bản giao việc — ghi ra
/// tệp rồi gửi ĐƯỜNG DẪN (đúng như ảnh 23/11: «Đọc .tmp/brief-…md rồi làm tiếp»).
pub const TRAN_KY_TU: usize = 2000;

/// Kết quả tra phiên đích trong ảnh chụp.
#[derive(Debug)]
pub enum Dich<'a> {
    Mot(&'a LiveSession),
    Khong,
    /// Tiền tố khớp nhiều phiên — KHÔNG bốc đại một cái: gõ nhầm phiên là đưa
    /// việc cho người khác (cùng lý do `session_of_this_terminal` trả `None`).
    Nhieu(Vec<&'a LiveSession>),
    /// Id chỉ khớp một CỬA SỔ TRẦN (`win-<tty>`, không có `claude` trong đó).
    /// 🔴 Gõ vào đó là chữ chạy thẳng thành LỆNH SHELL — đi vòng qua tường
    /// `DENIED_TOOLS` mà mọi phiên `/new` đứng sau (luật 1). Không bao giờ gõ.
    CuaSoTran,
}

/// Khớp đủ id trước; không có thì khớp TIỀN TỐ — và chỉ nhận khi đúng một phiên.
/// Chỉ xét PHIÊN CLI; hàng cửa sổ trần không bao giờ là đích.
pub fn tim_dich<'a>(live: &'a [LiveSession], want: &str) -> Dich<'a> {
    let want = want.trim();
    if want.is_empty() {
        return Dich::Khong;
    }
    // Id GÕ VÀO đã mang dạng cửa sổ ⟹ từ chối ngay, không cần danh sách: đo
    // 23/11 (load ~100) phép dò Terminal hết giờ nên hàng `win-ttys000` vắng khỏi
    // ảnh chụp, và câu trả lời rơi thành «không thấy» thay vì «không bao giờ gõ».
    if want.starts_with(crate::sessions::SHELL_ID_PREFIX) {
        return Dich::CuaSoTran;
    }
    let la_cua_so = |s: &LiveSession| s.session_id.starts_with(crate::sessions::SHELL_ID_PREFIX);
    let phien = || live.iter().filter(|s| !la_cua_so(s));
    if let Some(s) = phien().find(|s| s.session_id == want) {
        return Dich::Mot(s);
    }
    let khop: Vec<&LiveSession> = phien().filter(|s| s.session_id.starts_with(want)).collect();
    match khop.len() {
        0 if live
            .iter()
            .any(|s| la_cua_so(s) && s.session_id.starts_with(want)) =>
        {
            Dich::CuaSoTran
        }
        0 => Dich::Khong,
        1 => Dich::Mot(khop[0]),
        _ => Dich::Nhieu(khop),
    }
}

/// Chữ thật sự gõ vào ô nhập của phiên đích: nhãn NGƯỜI GỬI đứng đầu, để phiên
/// nhận biết đây là lời nhờ của một phiên khác qua huba chứ không phải chủ máy gõ.
///
/// Xuống dòng ⟹ một dấu cách: `do script` kèm sẵn một dấu xuống dòng và `claude`
/// đọc cả khối như một cú DÁN (luật 13) — chữ nhiều dòng là chữ có Enter ở giữa.
pub fn chu_gui(tu: Option<&str>, chu: &str) -> Result<String, String> {
    let gon = chu.split_whitespace().collect::<Vec<_>>().join(" ");
    if gon.is_empty() {
        return Err("chữ gửi rỗng — không có gì để gõ".into());
    }
    let n = gon.chars().count();
    if n > TRAN_KY_TU {
        return Err(format!(
            "chữ gửi dài {n} ký tự > trần {TRAN_KY_TU} — ghi ra tệp trong dự án của phiên đích \
             rồi gửi đường dẫn"
        ));
    }
    let tu = tu
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("terminal ngoài phiên");
    Ok(format!("[huba · từ {tu}] {gon}"))
}

/// Màn phiên đích có cho gõ không. `None` = gõ được; `Some((mã, câu))` = DỪNG.
///
/// Hộp chọn đang mở ⟹ chữ + CR rơi vào hộp và CHỐT một lựa chọn hộ chủ máy —
/// thứ không lùi lại được (ca thật 15/11: `/new` gõ đề bài vào hộp tin-thư-mục,
/// CR chốt «No, exit», phiên chết). Không đọc được màn ⟹ mã 2: mù thì không ra tay.
pub fn chan_khi(ready: &Ready) -> Option<(i32, String)> {
    match ready {
        Ready::Yes => None,
        Ready::Blocked { options, shape } => Some((
            1,
            format!(
                "phiên đích đang mở hộp chọn ({options} mục, {}) — gõ lúc này là CHỐT hộ một \
                 lựa chọn; CHƯA gõ gì. Chờ phiên ấy được trả lời rồi gửi lại.",
                shape.as_str()
            ),
        )),
        Ready::Blind(why) => Some((
            2,
            format!("KHÔNG ĐO ĐƯỢC màn phiên đích ({why}) — CHƯA gõ gì"),
        )),
    }
}

/// Mã thoát + câu cho kết quả gõ. Chỉ `Gone` (chữ đã rời ô — đo trên màn) là 0.
pub fn ket_qua(d: &Delivered) -> (i32, String) {
    match d {
        Delivered::Gone => (0, "đã gửi — chữ đã rời ô nhập".into()),
        Delivered::StillInBox => (
            2,
            "đã GÕ nhưng chữ còn nằm trong ô nhập (chưa chạy) — phiên đích hoặc chủ máy phải Enter"
                .into(),
        ),
        Delivered::Unverified(why) => (
            2,
            format!("đã gõ nhưng KHÔNG kiểm được chữ đã đi chưa ({why})"),
        ),
    }
}

/// Câu báo Telegram — một dòng, chủ máy đọc là biết ai nhờ ai việc gì.
pub fn cau_bao(tu: &str, den: &str, chu: &str, ket: &str) -> String {
    format!(
        "📨 {tu} → {den}: {}\n{ket}",
        crate::exec::truncate(chu, 300)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::BoxShape;

    fn phien(id: &str) -> LiveSession {
        LiveSession {
            session_id: id.into(),
            ..Default::default()
        }
    }

    #[test]
    fn tien_to_khop_nhieu_phien_thi_khong_boc_dai() {
        let live = vec![phien("abc111"), phien("abc222"), phien("def333")];
        assert!(matches!(tim_dich(&live, "abc"), Dich::Nhieu(v) if v.len() == 2));
        assert!(matches!(tim_dich(&live, "abc2"), Dich::Mot(s) if s.session_id == "abc222"));
        assert!(matches!(tim_dich(&live, "def333"), Dich::Mot(_)));
        assert!(matches!(tim_dich(&live, "zzz"), Dich::Khong));
        assert!(matches!(tim_dich(&live, "  "), Dich::Khong));
    }

    #[test]
    fn cua_so_shell_tran_khong_bao_gio_la_dich() {
        let live = vec![phien("win-ttys007"), phien("abc111")];
        assert!(matches!(tim_dich(&live, "win-ttys007"), Dich::CuaSoTran));
        assert!(matches!(tim_dich(&live, "win"), Dich::CuaSoTran));
        // "w" không khớp phiên CLI nào, chỉ khớp cửa sổ ⟹ vẫn từ chối, không phải "không thấy".
        assert!(matches!(tim_dich(&live, "w"), Dich::CuaSoTran));
        assert!(matches!(tim_dich(&live, "abc"), Dich::Mot(s) if s.session_id == "abc111"));
        // Phép dò Terminal hỏng ⟹ hàng cửa sổ vắng khỏi danh sách — vẫn phải từ chối.
        assert!(matches!(
            tim_dich(&[phien("abc111")], "win-ttys000"),
            Dich::CuaSoTran
        ));
    }

    #[test]
    fn khop_du_id_thang_tien_to() {
        // "ab" vừa là id đủ của một phiên vừa là tiền tố của phiên khác.
        let live = vec![phien("abc"), phien("ab")];
        assert!(matches!(tim_dich(&live, "ab"), Dich::Mot(s) if s.session_id == "ab"));
    }

    #[test]
    fn chu_gui_mang_nhan_nguoi_gui_va_bo_xuong_dong() {
        let c = chu_gui(Some("[tfl5/may-chu]"), "Đọc .tmp/brief.md\nrồi làm tiếp").unwrap();
        assert_eq!(
            c,
            "[huba · từ [tfl5/may-chu]] Đọc .tmp/brief.md rồi làm tiếp"
        );
        assert!(!c.contains('\n'));
        let c = chu_gui(None, "x").unwrap();
        assert!(c.starts_with("[huba · từ terminal ngoài phiên] "));
    }

    #[test]
    fn chu_gui_rong_hoac_qua_tran_bi_tu_choi() {
        assert!(chu_gui(Some("a"), " \n\t ").is_err());
        assert!(chu_gui(Some("a"), &"x".repeat(TRAN_KY_TU)).is_ok());
        assert!(chu_gui(Some("a"), &"x".repeat(TRAN_KY_TU + 1)).is_err());
    }

    #[test]
    fn hop_chon_dang_mo_thi_khong_go() {
        let numbered = Ready::Blocked {
            options: 3,
            shape: BoxShape::Numbered,
        };
        assert_eq!(chan_khi(&numbered).map(|(m, _)| m), Some(1));
        let khong_so = Ready::Blocked {
            options: 2,
            shape: BoxShape::Unnumbered,
        };
        assert_eq!(chan_khi(&khong_so).map(|(m, _)| m), Some(1));
        assert_eq!(chan_khi(&Ready::Blind("x".into())).map(|(m, _)| m), Some(2));
        assert!(chan_khi(&Ready::Yes).is_none());
    }

    #[test]
    fn chi_chu_da_roi_o_moi_la_ma_0() {
        assert_eq!(ket_qua(&Delivered::Gone).0, 0);
        assert_eq!(ket_qua(&Delivered::StillInBox).0, 2);
        assert_eq!(ket_qua(&Delivered::Unverified("mù".into())).0, 2);
    }
}
