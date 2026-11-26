//! K4 — gửi một tin từ giao diện web (`huba ui`) vào một phiên Claude.
//!
//! Cầu nối, không phải chủ (CLAUDE.md): đây là việc chủ máy làm khi ngồi trước terminal — gõ một câu vào ô nhập — nên đi
//! ĐÚNG đường mà Telegram `/type` và `/runin` đã đi, dùng lại đúng các hàng rào ấy chứ không dựng đường mới:
//! tab phải đúng phiên (`pipeline::paste_target_ok`) → gõ + kiểm màn (`keys::type_and_send`) → đối chiếu NHẬT KÝ phiên
//! (`pipeline::cau_da_vao_nhat_ky`) — màn khai «đã gửi» chưa đủ (đo 24/09: 21/98 lượt chỉ vào hội thoại sau hơn 60 s).
//!
//! Hai nguyên tắc riêng của đường này:
//! 1. **Mặc định là TỪ CHỐI**, và nói vì sao ([`refuse_reason`], hàm thuần). Gõ vào phiên không lùi lại được.
//! 2. **Không bao giờ tự gửi lại** khi kết quả mơ hồ. Chỉ báo bốn điều: đã nhận · còn trong ô nhập · chưa xác nhận ·
//!    bị từ chối. Giao diện giữ bản nháp trừ khi đã nhận; gửi lại là quyết định của người bấm.

use serde::Serialize;
use serde_json::json;

use crate::config::Config;
use crate::logging;
use crate::sessions::LiveSession;

/// Trần độ dài một tin (ký tự). Tin dài hơn là tệp/đoạn dán, không phải một câu nói với phiên.
pub const MAX_CHARS: usize = 8000;

/// Kết cục của một lần gửi — bốn điều giao diện được phép nói, thêm `Failed` cho lỗi hạ tầng (không gõ được).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Nhật ký phiên ĐÃ nhận tin — bằng chứng duy nhất đáng tin.
    Sent,
    /// Chữ VẪN nằm trong ô nhập của phiên, chưa chạy.
    InBox,
    /// Không xác nhận được (màn nói đã gửi mà nhật ký chưa thấy, hoặc không đọc được màn/nhật ký).
    Unverified,
    /// Bị từ chối TRƯỚC khi gõ — chưa có chữ nào vào phiên.
    Refused,
    /// Lỗi lúc gõ — có thể đã gõ một phần; xem màn trước khi gửi lại.
    Failed,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Sent => "sent",
            Outcome::InBox => "in_box",
            Outcome::Unverified => "unverified",
            Outcome::Refused => "refused",
            Outcome::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendResult {
    pub outcome: Outcome,
    /// Câu tiếng Việt cho người đọc.
    pub message: String,
}

impl SendResult {
    fn new(outcome: Outcome, message: impl Into<String>) -> Self {
        SendResult {
            outcome,
            message: message.into(),
        }
    }
}

/// Phiên + chữ này có được phép gõ không? `None` = được; `Some(vì sao)` = từ chối. THUẦN: chỉ đọc hàng phiên đã đệm và
/// chữ; việc nhìn màn THẬT lúc gõ nằm ở [`send`].
pub fn refuse_reason(s: &LiveSession, text: &str) -> Option<String> {
    if text.trim().is_empty() {
        return Some("Tin trống — gõ chữ rồi hãy gửi.".into());
    }
    if text.chars().count() > MAX_CHARS {
        return Some(format!(
            "Tin dài quá {MAX_CHARS} ký tự — chia nhỏ, hoặc để thành tệp rồi nhờ phiên đọc."
        ));
    }
    if text.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Some("Tin có ký tự điều khiển (không in được) — xoá chúng rồi gửi lại.".into());
    }
    if text.trim_start().starts_with('/') {
        return Some(
            "Tin bắt đầu bằng «/» sẽ chạy như một lệnh của Claude (/exit, /clear…), nên giao diện chưa gửi loại này — \
             gõ ở terminal hoặc Telegram /type."
                .into(),
        );
    }
    if crate::sessions::is_opencode_session_id(&s.session_id) {
        return Some("Giao diện chưa gửi được vào phiên opencode (hiện chỉ Claude) — dùng Telegram hoặc terminal.".into());
    }
    if s.kind != "interactive" {
        return Some(format!(
            "Phiên loại «{}» không có ô nhập để gõ vào — chỉ phiên mở trong cửa sổ Terminal nhận được tin.",
            s.kind
        ));
    }
    if s.host != "terminal" {
        return Some(format!(
            "Phiên này chạy ở «{}», không phải cửa sổ Terminal nên huba không gõ vào được.",
            s.host
        ));
    }
    if !crate::sessions::is_real_tty(&s.tty) {
        return Some("Phiên không gắn với cửa sổ Terminal nào (không có tty) nên huba không gõ vào được.".into());
    }
    if s.asking_password {
        return Some("Phiên đang chờ MẬT KHẨU — huba không gõ vào ô đó. Nhập ở terminal.".into());
    }
    if s.asking.is_some() || s.screen_choices > 0 {
        return Some(
            "Phiên đang hỏi / mở hộp chọn — chữ gõ vào đó sẽ thành lựa chọn chứ không phải tin nhắn. Trả lời ở Telegram (/pick) \
             hoặc terminal."
                .into(),
        );
    }
    if s.account_dead.is_some() {
        return Some("Tài khoản của phiên đang hỏng (đăng nhập hết hạn?) — phiên chưa nhận tin được.".into());
    }
    if s.limited.is_some() {
        return Some("Phiên đang bị chặn vì hết hạn mức — chờ tới giờ mở lại.".into());
    }
    None
}

fn refused(session: &str, chars: usize, why: String) -> SendResult {
    logging::info(
        "ui_send",
        json!({ "session": session, "chars": chars, "result": "refused", "why": why }),
    );
    SendResult::new(Outcome::Refused, why)
}

/// Gửi `text` vào phiên `s` — chạy THẬT trên Terminal. Chặn tới khi biết kết cục (tối đa ~vài chục giây).
///
/// Không bao giờ gõ hai lần: mọi nhánh lỗi/mơ hồ trả về ngay, KHÔNG thử lại.
pub fn send(cfg: &Config, s: &LiveSession, text: &str) -> SendResult {
    let chars = text.chars().count();
    if let Some(why) = refuse_reason(s, text) {
        return refused(&s.session_id, chars, why);
    }
    // Hạng gấp: một người đang chờ kết quả, không phải việc quét nền (cùng lý do `pipeline::dan_vao_phien`).
    let _lane = crate::exec::urgent();
    let w = match crate::keys::window_of(&s.tty) {
        Ok(Some(w)) => w,
        Ok(None) => {
            return refused(
                &s.session_id,
                chars,
                "Không tìm thấy cửa sổ Terminal của phiên (đã đóng?) — không gõ.".into(),
            )
        }
        Err(e) => {
            return refused(
                &s.session_id,
                chars,
                format!("Không hỏi được Terminal cửa sổ nào của phiên ({e}) — không gõ."),
            )
        }
    };
    // Nhìn màn THẬT ngay lúc gõ: bản đệm danh sách phiên cũ tới ~10 s, đủ để một hộp chọn vừa mở.
    match crate::keys::screen_text(w) {
        Ok(screen) => {
            if !crate::keys::parse_choices(&screen).is_empty() {
                return refused(
                    &s.session_id,
                    chars,
                    "Phiên vừa mở một hộp chọn — chữ gõ vào đó sẽ thành lựa chọn. Trả lời ở Telegram (/pick) hoặc terminal."
                        .into(),
                );
            }
        }
        Err(e) => {
            return refused(
                &s.session_id,
                chars,
                format!("Không đọc được màn phiên ({e}) nên không dám gõ."),
            )
        }
    }
    if let Err(why) = crate::pipeline::paste_target_ok(w, &s.tty, &s.session_id) {
        return refused(&s.session_id, chars, format!("Không gõ: {why}"));
    }
    // Mốc nhật ký ĐẶT TRƯỚC khi gõ — chỉ phần ghi thêm sau mốc mới là bằng chứng cho lượt NÀY.
    let nhat_ky = crate::sessions::delivery_transcript(cfg, s)
        .and_then(|p| std::fs::metadata(&p).ok().map(|m| (p, m.len())));
    let r = match crate::keys::type_and_send(w, text) {
        Ok(crate::keys::Delivered::Gone) => match &nhat_ky {
            Some((p, tu)) => {
                let thay = crate::pipeline::cho_nhat_ky(
                    p,
                    *tu,
                    std::time::Duration::from_secs(crate::pipeline::XAC_NHAN_NHAT_KY_SEC),
                    |m| crate::pipeline::cau_da_vao_nhat_ky(m, text),
                );
                match thay {
                    Some(true) => SendResult::new(Outcome::Sent, "Phiên đã nhận tin."),
                    Some(false) => SendResult::new(
                        Outcome::Unverified,
                        "Màn khai tin đã đi, nhưng nhật ký phiên CHƯA thấy sau vài giây. Xem phiên trước khi gửi lại — \
                         gửi lại ngay có thể thành hai tin.",
                    ),
                    None => SendResult::new(
                        Outcome::Unverified,
                        "Màn khai tin đã đi, nhưng không đọc được nhật ký phiên để xác nhận. Xem phiên trước khi gửi lại.",
                    ),
                }
            }
            None => SendResult::new(
                Outcome::Unverified,
                "Màn khai tin đã đi, nhưng không tìm thấy nhật ký phiên để xác nhận. Xem phiên trước khi gửi lại.",
            ),
        },
        Ok(crate::keys::Delivered::StillInBox) => SendResult::new(
            Outcome::InBox,
            "Chữ vẫn nằm trong ô nhập của phiên, chưa chạy. Xem phiên: có thể cần bấm Enter ở đó (đừng gửi lại — sẽ thành hai bản).",
        ),
        Ok(crate::keys::Delivered::Unverified(why)) => SendResult::new(
            Outcome::Unverified,
            format!("Không kiểm được tin đã đi chưa ({why}). Xem phiên trước khi gửi lại."),
        ),
        Err(e) => SendResult::new(
            Outcome::Failed,
            format!("Lỗi lúc gõ vào Terminal: {e}. Có thể đã gõ một phần — xem phiên trước khi gửi lại."),
        ),
    };
    logging::info(
        "ui_send",
        json!({ "session": s.session_id, "chars": chars, "result": r.outcome.as_str() }),
    );
    r
}
