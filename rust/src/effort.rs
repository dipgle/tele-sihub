//! `/effort` — đổi mức effort của một phiên ĐANG chạy, bằng đúng lệnh chủ máy gõ
//! ở terminal (`/effort <mức>`).
//!
//! 🔴 Quang 2026-11-10: *"tại sao mở phiên mặc định lại là XHigh, có tự chọn được
//! không?"* rồi *"thêm lệnh để đổi được cho phiên đang chạy ghi vào quy tắc để
//! tự model đánh giá việc nào cần ở mức độ nào để chuyển"*. Hai cửa vào, một lõi
//! ([`set_on_window`]): route Telegram `/effort` cho chủ máy, và `huba effort`
//! gọi từ Bash để CHÍNH phiên tự đổi mức của nó.
//!
//! Phép thử cầu nối: ngồi ở máy anh gõ `/effort high` vào ô nhập — huba gõ đúng
//! chữ ấy vào đúng ô ấy, không chọc vào `settings.json` hay cờ nào khác.
//!
//! **Sự thật về CLI (`claude` 2.1.280), đo 2026-11-10 trên cửa sổ thật:**
//! - Mức nhận: `low · medium · high · xhigh · max` (`claude --help`). Không có
//!   mức "tự động".
//! - Rảnh, phiên CHƯA có lịch sử ⟹ đổi ngay. Rảnh, ĐÃ có lịch sử ⟹ hộp
//!   *"Change effort level?"* — *"Switching … means the full history gets re-read
//!   on your next message"*: **mỗi lần đổi giữa phiên = mất bộ đệm**.
//! - ĐANG BẬN ⟹ đổi được (lúc in chữ: không hộp; lúc chờ một công cụ: CÓ hộp —
//!   [`set_on_window`] bấm `1`). Câu trả lời đang sinh dở GIỮ mức cũ; lần gọi
//!   model KẾ TIẾP đã theo mức mới — kể cả trong cùng lượt: phiên `7ba26fed`
//!   chạy `huba effort high` từ Bash lúc đang `medium`, bản ghi `assistant`
//!   ngay sau `tool_result` đã mang `effort: high`.
//! - Mọi lần đổi đều ghi `modelSettings.<model>.effortLevel` vào `settings.json`
//!   của tài khoản ("saved as your default"), NHƯNG trong `~/projects` dòng
//!   `effortLevel` của `.claude/settings.json` dự án THẮNG nó; cờ `--effort` lúc
//!   mở phiên thắng cả hai.
//!
//! **Bẫy đã trả giá ngay trong lượt đo:** chữ nằm sẵn trong ô + `/effort` dán
//! vào ⟹ một tin LẪN gửi cho model; một `\n` vô hình đầu ô ⟹ `"\n/exit"` tới
//! model như tin nhắn. Nên [`first_move`] nhìn ô TRƯỚC khi gõ, và chỉ báo "đã
//! đổi" khi màn có thêm dòng `Set effort level to <mức>` ([`applied_marks`]).

use serde_json::json;

use crate::keys::BoxState;
use crate::logging;

/// Các mức CLI nhận, thấp → cao. Một nguồn cho route, CLI và câu báo lỗi.
pub const LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

/// Đọc một mức chủ máy gõ. Không phân biệt hoa thường; sai thì nói đủ các mức.
pub fn parse_level(raw: &str) -> Result<&'static str, String> {
    let t = raw.trim().to_ascii_lowercase();
    LEVELS.iter().copied().find(|l| *l == t).ok_or_else(|| {
        format!(
            "mức '{}' không có — chọn một trong: {}",
            crate::exec::truncate(raw.trim(), 20),
            LEVELS.join(" · ")
        )
    })
}

/// Đúng chữ chủ máy gõ ở terminal.
pub fn command_text(level: &str) -> String {
    format!("/effort {level}")
}

/// Màn đã gộp khoảng trắng — dòng `Set effort level to medium (saved …):` bị
/// cửa sổ hẹp bẻ xuống dòng vẫn đếm được.
fn flat(screen: &str) -> String {
    screen.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Số lần màn in `Set effort level to <mức>` — câu CLI in ra khi ĐÃ đổi.
///
/// So TRƯỚC/SAU chứ không hỏi "có hay không": màn còn giữ dòng của lần đổi cũ.
/// `to high` không phải chuỗi con của `to xhigh` (chữ sau `to ` là `x`) — bài
/// `applied_marks_dem_dong_xac_nhan_ca_luc_ranh_lan_luc_ban` khoá đúng ca ấy.
pub fn applied_marks(screen: &str, level: &str) -> usize {
    flat(screen)
        .matches(&format!("Set effort level to {level}"))
        .count()
}

/// Hộp *"Change effort level?"* đang hỏi xác nhận ĐÚNG mức này.
///
/// Đòi cả tiêu đề lẫn dòng `1. Yes, switch to <mức>` — bấm `1` chỉ khi hộp
/// đang xin đúng việc chủ máy vừa ra lệnh, không phải một hộp nào khác.
pub fn confirm_dialog(screen: &str, level: &str) -> bool {
    let s = flat(screen);
    if !s.contains("Change effort level?") {
        return false;
    }
    s.contains(&format!("1. Yes, switch to {level}"))
}

/// Nước đi ĐẦU, đọc từ màn trước khi gõ phím nào.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirstMove {
    /// Không gõ gì — kèm lý do để nói ra.
    Refuse(String),
    /// Ô trống ⟹ xoá sạch rác vô hình rồi gõ lệnh, một lượt ghi.
    TypeClean,
    /// Phiên rảnh, ô có chữ — có thể là GỢI Ý MỜ (gõ vào là nó tắt), có thể là
    /// NHÁP của chủ máy. Màn chữ không phân biệt được hai thứ ấy ⟹ gõ lệnh mà
    /// chưa gửi ([`crate::keys::type_held`]) rồi nhìn lại ô.
    Probe,
}

/// Quyết [`FirstMove`] từ một màn. THUẦN.
///
/// Phiên ĐANG BẬN mà ô có chữ thì TỪ CHỐI chứ không dò: lượt dò kết thúc bằng
/// ESC, và trên phiên đang chạy ESC là phím CẮT lượt. Không đọc được chân màn
/// (`screen_running` = `None`) cũng tính là bận — mù thì không ra tay.
pub fn first_move(screen: &str) -> FirstMove {
    if !crate::keys::parse_choices(screen).is_empty() {
        return FirstMove::Refuse(
            "màn đang mở một hộp chọn — gõ vào đó là trả lời hộp ấy".to_string(),
        );
    }
    match crate::keys::box_state(screen) {
        BoxState::NoBox => FirstMove::Refuse(
            "màn không thấy ô nhập (hộp quyền đang che, hoặc phiên đã thoát)".to_string(),
        ),
        BoxState::Empty => FirstMove::TypeClean,
        BoxState::Text(_) => match crate::keys::screen_running(screen) {
            Some(false) => FirstMove::Probe,
            _ => FirstMove::Refuse(format!(
                "ô nhập đang có chữ «{}» mà phiên đang chạy — gõ chen vào là gửi một tin lẫn \
                 cho model. `/clear` rồi gõ lại.",
                box_preview(screen)
            )),
        },
    }
}

/// Ô nhập sau lượt dò nói gì.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeSays {
    /// Ô chỉ có đúng lệnh vừa gõ ⟹ thứ có trước là gợi ý mờ (hoặc rỗng).
    OnlyOurs,
    /// Ô có thêm chữ khác ⟹ nháp thật của chủ máy. Mang chữ ô đang có.
    Mixed(String),
    /// Ô đã trống ⟹ lệnh đã đi (ESC không giữ được) — chuyển sang kiểm kết quả.
    Gone,
    /// Không thấy ô nhập nữa (hộp xác nhận vừa bật lên).
    NoBox,
}

/// Chấm ô nhập sau lượt [`crate::keys::type_held`]. THUẦN.
pub fn probe_verdict(screen: &str, command: &str) -> ProbeSays {
    match crate::keys::box_state(screen) {
        BoxState::NoBox => ProbeSays::NoBox,
        BoxState::Empty => ProbeSays::Gone,
        BoxState::Text(_) => {
            let t = crate::keys::input_box_text(screen).unwrap_or_default();
            if crate::keys::squash_box(&t) == crate::keys::squash_box(command) {
                ProbeSays::OnlyOurs
            } else {
                ProbeSays::Mixed(t)
            }
        }
    }
}

fn box_preview(screen: &str) -> String {
    crate::exec::truncate(
        crate::keys::input_box_text(screen)
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .as_str(),
        60,
    )
}

/// Kết cục một lượt đổi — ba nhánh, không gộp (luật 13②).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Màn có thêm dòng `Set effort level to <mức>`. `confirmed` = huba đã bấm
    /// `1` ở hộp xác nhận (phiên có lịch sử ⟹ lượt kế đọc lại toàn bộ).
    Applied { confirmed: bool },
    /// Không đổi — kèm lý do. Ô nhập của chủ máy giữ nguyên.
    Refused(String),
    /// Đã gõ, nhưng màn không cho thấy kết quả — KHÔNG được đọc thành "xong".
    NotSeen(String),
}

impl Outcome {
    /// Câu nói với người — một câu cho cả Telegram lẫn CLI.
    pub fn say(&self, who: &str, level: &str) -> String {
        match self {
            Outcome::Applied { confirmed: false } => format!(
                "⚙️ {who}: effort → {level}. Có hiệu lực từ LẦN GỌI MODEL KẾ (câu đang sinh dở \
                 giữ mức cũ)."
            ),
            Outcome::Applied { confirmed: true } => format!(
                "⚙️ {who}: effort → {level} (đã bấm xác nhận). Lần gọi model kế ĐỌC LẠI toàn bộ \
                 lịch sử — mất bộ đệm một lần."
            ),
            Outcome::Refused(why) => format!("⚠ {who}: CHƯA đổi effort — {why}"),
            Outcome::NotSeen(tail) => format!(
                "⚠ {who}: đã gõ `{}` nhưng màn không hiện dòng xác nhận — chưa biết đã đổi \
                 chưa. `/shot` để xem. Đuôi màn: {tail}",
                command_text(level)
            ),
        }
    }

    /// Mã thoát cho `huba effort`: 0 đổi được · 1 từ chối · 2 không đo được.
    pub fn exit_code(&self) -> i32 {
        match self {
            Outcome::Applied { .. } => 0,
            Outcome::Refused(_) => 1,
            Outcome::NotSeen(_) => 2,
        }
    }
}

// ── opencode: effort = VARIANT của model, đổi vòng bằng Ctrl+T ──────────────
//
// Quang 13/11: *"effort của opencode chỉ có 3 mức: low high max"*. Đo trên TUI
// opencode 2.2.7 (Ubuntu, acc8, DeepSeek V4.1 Flash): phím `variant.cycle` =
// `ctrl+t`; vòng Default → low → high → max → Default; mức đang dùng là HẬU TỐ
// của dòng trạng thái trong ô nhập (`┃  Build · DeepSeek V4.1 Flash 1017 OpenCode
// Go · high`), Default thì không hậu tố. Gõ `/effort high` kiểu claude vào ô
// opencode là gửi một TIN cho model — đã xảy ra đúng một lần lúc đo ("High there…").
// Một lần trong lượt đo, Ctrl+T mở hộp "Select variant" thay vì đổi vòng (phiên
// chưa có tin nhắn) — chưa rõ luật ⟹ thấy hộp thì Esc và TỪ CHỐI, không gõ chữ vào.

/// Mức effort opencode chủ máy gõ được. `default` = không hậu tố.
pub const OPENCODE_LEVELS: [&str; 4] = ["default", "low", "high", "max"];

/// Đọc một mức cho phiên opencode. Thuần.
pub fn parse_opencode_level(raw: &str) -> Result<&'static str, String> {
    let t = raw.trim().to_ascii_lowercase();
    OPENCODE_LEVELS
        .iter()
        .copied()
        .find(|l| *l == t)
        .ok_or_else(|| {
            format!(
                "opencode không có mức '{}' — chọn một trong: {}",
                crate::exec::truncate(raw.trim(), 20),
                OPENCODE_LEVELS.join(" · ")
            )
        })
}

/// Mức variant đang hiện trên màn opencode — `None` khi không thấy dòng trạng
/// thái của ô nhập (đừng đoán "default" từ một màn không đọc ra). Thuần.
pub fn opencode_variant(screen: &str) -> Option<String> {
    let dong = screen
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| l.starts_with('┃') && l.contains(" · "))?;
    let cuoi = dong.rsplit(" · ").next().unwrap_or("").trim();
    const BIET: [&str; 6] = ["minimal", "low", "medium", "high", "xhigh", "max"];
    Some(if BIET.contains(&cuoi) {
        cuoi.to_string()
    } else {
        "default".to_string()
    })
}

/// Đổi variant của phiên opencode trong cửa sổ `window`: Ctrl+T từng cú, đọc lại
/// sau MỖI cú, dừng khi màn hiện đúng mức. Không bao giờ gõ chữ.
pub fn set_opencode(window: i64, level: &str) -> anyhow::Result<Outcome> {
    // macOS: phím đi bằng `do script` ⟹ kèm một CR không tắt được; ô opencode còn
    // chữ thì CR ấy GỬI chữ cho model. Chưa đo trên Mac ⟹ không bấm liều.
    if cfg!(target_os = "macos") {
        return Ok(Outcome::Refused(
            "macOS chưa hỗ trợ đổi variant opencode từ xa (Ctrl+T đi kèm một Enter) — \
             đổi tay bằng Ctrl+T trong cửa sổ"
                .into(),
        ));
    }
    use std::thread::sleep;
    use std::time::Duration;
    let doc = || -> anyhow::Result<(String, Option<String>)> {
        let s = crate::keys::screen_text(window)?;
        let v = opencode_variant(&s);
        Ok((s, v))
    };
    let (_, dau) = doc()?;
    let Some(dau) = dau else {
        return Ok(Outcome::Refused(
            "không đọc ra dòng trạng thái của ô nhập opencode — không bấm gì".into(),
        ));
    };
    if dau == level {
        return Ok(Outcome::Applied { confirmed: false });
    }
    // Vòng có tối đa 4–5 mức tuỳ model; 6 cú mà chưa tới là model không có mức ấy.
    for cu in 1..=6 {
        crate::keys::press(window, "ctrl-t")?;
        sleep(Duration::from_millis(1200));
        let (s, v) = doc()?;
        if s.contains("Select variant") {
            crate::keys::press(window, "esc")?;
            logging::warn(
                "effort_opencode_hop_chon",
                json!({ "window": window, "level": level, "cu": cu }),
            );
            return Ok(Outcome::Refused(
                "opencode mở hộp «Select variant» thay vì đổi vòng — đã Esc, chưa đổi. \
                 Chọn tay bằng /shot rồi bấm, hoặc thử lại sau tin nhắn đầu."
                    .into(),
            ));
        }
        logging::info(
            "effort_opencode_cu",
            json!({ "window": window, "cu": cu, "thay": v, "can": level }),
        );
        match v {
            Some(v) if v == level => return Ok(Outcome::Applied { confirmed: false }),
            Some(v) if v == dau => {
                return Ok(Outcome::Refused(format!(
                "đã đi hết vòng variant của model mà không có mức '{level}' — giữ nguyên '{dau}'"
            )))
            }
            Some(_) => {}
            None => return Ok(Outcome::NotSeen(box_preview(&s))),
        }
    }
    let (s, _) = doc()?;
    Ok(Outcome::NotSeen(box_preview(&s)))
}

/// Phím xoá đi trước lệnh ở nhánh ô trống — đủ nuốt vài dòng trống vô hình.
const CLEAN_DELS: usize = 32;

/// Đổi effort của phiên trong cửa sổ `window` (macOS). Lõi chung của route
/// `/effort` và `huba effort`.
pub fn set_on_window(window: i64, level: &str) -> anyhow::Result<Outcome> {
    use std::thread::sleep;
    use std::time::Duration;
    let cmd = command_text(level);
    let before = crate::keys::screen_text(window)?;
    let marks0 = applied_marks(&before, level);
    let first = first_move(&before);
    logging::info(
        "effort_first_move",
        json!({ "window": window, "level": level, "move": format!("{first:?}") }),
    );
    match first {
        FirstMove::Refuse(why) => return Ok(Outcome::Refused(why)),
        FirstMove::TypeClean => crate::keys::type_after_dels(window, CLEAN_DELS, &cmd)?,
        FirstMove::Probe => {
            crate::keys::type_held(window, &cmd)?;
            sleep(Duration::from_millis(500));
            let s = crate::keys::screen_text(window)?;
            match probe_verdict(&s, &cmd) {
                ProbeSays::OnlyOurs => crate::keys::press(window, "enter")?,
                ProbeSays::Mixed(t) => {
                    // Rút lại ĐÚNG phần mình gõ: lệnh + dấu xuống dòng ESC để lại.
                    crate::keys::delete_held(window, cmd.chars().count() + 1)?;
                    sleep(Duration::from_millis(500));
                    let after = crate::keys::screen_text(window)?;
                    let con = crate::keys::still_in_box(&after, &cmd);
                    logging::warn(
                        "effort_refused_draft",
                        json!({ "window": window, "level": level,
                                "lenh_con_trong_o": con,
                                "effect": "ô có nháp của chủ máy — đã rút lệnh ra, không gửi" }),
                    );
                    // `t` đọc SAU lượt dò ⟹ đuôi là chính lệnh huba gõ; kể
                    // lại cho chủ máy thì chỉ kể chữ của anh.
                    let cua_chu = t.rsplit_once(cmd.as_str()).map_or(t.as_str(), |(a, _)| a);
                    let nhap = crate::exec::truncate(
                        cua_chu
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .as_str(),
                        60,
                    );
                    return Ok(Outcome::Refused(if con {
                        format!(
                            "ô nhập có nháp «{nhap}»; tôi rút `{cmd}` ra nhưng nó VẪN còn trong ô \
                             — chưa gửi gì. Xoá tay hoặc `/clear`."
                        )
                    } else {
                        format!(
                            "ô nhập có nháp «{nhap}» — không gõ chen vào (nháp giữ nguyên). \
                             `/clear` rồi gõ lại."
                        )
                    }));
                }
                ProbeSays::Gone | ProbeSays::NoBox => {}
            }
        }
    }
    let mut confirmed = false;
    let mut entered = false;
    let mut last = String::new();
    for wait_ms in [500u64, 900, 1500, 2000] {
        sleep(Duration::from_millis(wait_ms));
        last = crate::keys::screen_text(window)?;
        if applied_marks(&last, level) > marks0 {
            logging::info(
                "effort_applied",
                json!({ "window": window, "level": level, "confirmed": confirmed }),
            );
            return Ok(Outcome::Applied { confirmed });
        }
        if !confirmed && confirm_dialog(&last, level) {
            crate::keys::press(window, "1")?;
            confirmed = true;
            continue;
        }
        if !entered && crate::keys::still_in_box(&last, &cmd) {
            crate::keys::press(window, "enter")?;
            entered = true;
        }
    }
    let tail: String = last
        .lines()
        .filter(|l| !l.trim().is_empty())
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ⏎ ");
    let tail = crate::exec::truncate(&tail.split_whitespace().collect::<Vec<_>>().join(" "), 200);
    logging::warn(
        "effort_not_seen",
        json!({ "window": window, "level": level, "confirmed": confirmed, "entered": entered,
                "effect": "đã gõ lệnh nhưng màn không thêm dòng 'Set effort level to' — không khai là đã đổi" }),
    );
    Ok(Outcome::NotSeen(tail))
}
