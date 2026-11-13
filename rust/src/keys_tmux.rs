//! Linux (Ubuntu) — cùng vai trò với `keys.rs` trên macOS và `keys_win.rs` trên
//! Windows: mở một chỗ cho phiên `claude`, gõ vào đó, đọc lại nó. Quang 12/11:
//! *"Làm bản Ubuntu dùng tmux đi"*.
//!
//! # Mô hình: PANE tmux, không phải cửa sổ
//!
//! * "Cửa sổ" của huba = một **pane** tmux; số `i64` trong sổ là phần số của
//!   `pane_id` (`%12` ⟹ `12`). Pane id do tmux cấp, không tái dùng trong một lần
//!   chạy máy chủ tmux.
//! * "tty" = `pane_tty` (`/dev/pts/N`, cất dạng `pts/N`) — tty THẬT, đúng thứ
//!   `ps -o tty=` in cho tiến trình `claude` chạy trong pane. Nhờ vậy luật ghép
//!   phiên ↔ cửa sổ bằng tty của macOS dùng nguyên được.
//! * `/new` mở một WINDOW tmux mới trong phiên tmux [`PHIEN`] (chưa có thì dựng,
//!   tách rời) rồi GÕ dòng lệnh vào shell của nó — như `do script` gõ vào shell
//!   của tab Terminal. Lệnh chạy xong shell vẫn còn, chủ máy `tmux attach -t huba`
//!   là thấy và gõ tiếp được: đúng luật cầu nối (CLAUDE.md "huba is a BRIDGE").
//! * Gõ: `send-keys -l` (chữ nguyên văn) · phím rời theo tên tmux (`Enter`,
//!   `Up`, `C-c`…). Khối nhiều dòng đi bằng `paste-buffer -p` (dán có ngoặc khi
//!   chương trình xin) — gửi từng dòng kèm Enter là gửi từng mảnh đề bài.
//! * Đọc: `capture-pane -p` (khung đang hiện); cuộn ngược bằng `-S -<dòng>`.
//!
//! Không cần quyền hệ điều hành nào, không cần màn hình đồ hoạ, không giành
//! tiêu điểm: tmux là máy chủ chữ, nên hubd chạy bằng systemd `--user` vẫn gõ
//! và đọc được — miễn CÙNG người dùng (cùng ổ cắm `/tmp/tmux-<uid>/default`).
//!
//! Đo trên Ubuntu thật: `huba linux-tu-kiem` ([`tu_kiem`]).
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::exec::{run, RunOpts};
use crate::keys::{Cleared, Closed, Tab, TabState};

/// Phiên tmux chứa mọi cửa sổ huba mở. Đè bằng `HUBA_TMUX_SESSION`.
pub const PHIEN: &str = "huba";

/// Cỡ phiên tmux tách rời lúc dựng (mặc định của tmux là 80×24 — TUI `claude`
/// gãy dòng khắp nơi ở cỡ ấy). Khi chủ máy `attach`, tmux tự co theo màn thật.
const CO_COT: &str = "220";
const CO_HANG: &str = "60";

fn ten_phien() -> String {
    std::env::var("HUBA_TMUX_SESSION")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| PHIEN.to_string())
}

/// Gọi `tmux …` với trần 10 giây; mã ≠ 0 ⟹ `Err` kèm stderr (không nuốt).
fn tmux(args: &[&str]) -> Result<String> {
    let r = run(
        "tmux",
        args,
        RunOpts {
            timeout: Some(Duration::from_secs(10)),
            ..Default::default()
        },
    )?;
    if r.timed_out {
        return Err(anyhow!("tmux {} quá 10 giây", args.first().unwrap_or(&"")));
    }
    if r.code != Some(0) {
        return Err(anyhow!(
            "tmux {} thoát {:?}: {}",
            args.first().unwrap_or(&""),
            r.code,
            r.stderr.trim()
        ));
    }
    Ok(r.stdout)
}

fn dich(window: i64) -> String {
    format!("%{window}")
}

/// `%12` ⟹ `12`. Thuần.
pub fn so_pane(pane_id: &str) -> Option<i64> {
    pane_id.trim().strip_prefix('%')?.parse().ok()
}

/// Một hàng của `list-panes -a` theo khuôn [`KHUON_PANE`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    pub window: i64,
    /// `pts/3` — đã bỏ `/dev/`.
    pub tty: String,
    pub pid: i64,
    /// `pane_current_command` — tên chương trình đang đứng trước trong pane.
    pub lenh: String,
    pub title: String,
}

/// Dấu tách các trường. 🔴 KHÔNG dùng ký tự điều khiển: đo Ubuntu 24.04 · tmux 3.4,
/// 0x1F ra khỏi `-F` thành bốn chữ `\037` ⟹ không hàng nào tách được ⟹ mọi pane
/// "không còn" (tự kiểm ⑤⑥ đỏ). Chuỗi in được, không có trong tên lệnh/tty.
pub const TACH: &str = "~|huba|~";
/// Khuôn `list-panes -F`; nhan đề đứng CUỐI vì có thể mang mọi thứ.
pub const KHUON_PANE: &str =
    "#{pane_id}~|huba|~#{pane_tty}~|huba|~#{pane_pid}~|huba|~#{pane_current_command}~|huba|~#{pane_title}";

/// Đọc đầu ra `list-panes -a -F KHUON_PANE`. Hàng hỏng bị BỎ kèm log, không
/// đoán. Thuần.
///
/// 🔴 Một pane có thể hiện NHIỀU hàng: `list-panes -a` đi theo từng PHIÊN, và
/// phiên nhóm của cửa sổ xem ([`PHIEN_XEM`]) chia chung mọi window với [`PHIEN`].
/// Không khử thì mỗi cửa sổ xem nhân đôi danh sách phiên ⟹ giữ hàng ĐẦU mỗi pane id.
pub fn doc_panes(out: &str) -> Vec<Pane> {
    let mut da_thay = std::collections::HashSet::new();
    out.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let o: Vec<&str> = l.splitn(5, TACH).collect();
            let p = match o.as_slice() {
                [id, tty, pid, lenh, title] => {
                    so_pane(id)
                        .zip(pid.trim().parse().ok())
                        .map(|(w, pid)| Pane {
                            window: w,
                            tty: tty.trim().trim_start_matches("/dev/").to_string(),
                            pid,
                            lenh: lenh.trim().to_string(),
                            title: title.to_string(),
                        })
                }
                _ => None,
            };
            if p.is_none() {
                crate::logging::warn(
                    "tmux_pane_row_unparsed",
                    serde_json::json!({ "row": crate::exec::truncate(l, 200) }),
                );
            }
            p
        })
        .filter(|p| da_thay.insert(p.window))
        .collect()
}

/// Mọi pane của máy chủ tmux. Chưa có máy chủ nào ("no server running") ⟹
/// danh sách RỖNG — đúng nghĩa: không có cửa sổ nào; lỗi khác ⟹ `Err`.
pub fn panes() -> Result<Vec<Pane>> {
    match tmux(&["list-panes", "-a", "-F", KHUON_PANE]) {
        Ok(out) => Ok(doc_panes(&out)),
        Err(e) if khong_co_may_chu(&e.to_string()) => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Câu tmux nói khi chưa có máy chủ. Thuần.
pub fn khong_co_may_chu(err: &str) -> bool {
    err.contains("no server running")
        || err.contains("error connecting to")
        || err.contains("No such file or directory")
}

fn pane(window: i64) -> Result<Option<Pane>> {
    Ok(panes()?.into_iter().find(|p| p.window == window))
}

/// Shell ĐANG RẢNH: chương trình đứng trước trong pane chính là shell. Thuần.
pub fn la_shell(lenh: &str) -> bool {
    matches!(
        lenh.trim_start_matches('-'),
        "bash" | "zsh" | "sh" | "dash" | "fish" | "ksh" | "tcsh" | "csh"
    )
}

/// Tên phím của huba ⟹ tên phím của tmux. Số một chữ đi NGUYÊN VĂN (`-l`). Thuần.
pub fn ten_phim(name: &str) -> Result<(&'static str, Option<String>)> {
    // (`tên tmux`, `Some(chữ)` khi phải gửi nguyên văn)
    let t = match name {
        "enter" => "Enter",
        "esc" => "Escape",
        "up" => "Up",
        "down" => "Down",
        "left" => "Left",
        "right" => "Right",
        "tab" => "Tab",
        "space" => "Space",
        "backspace" => "BSpace",
        "ctrl-c" | "ctrlc" | "^c" => "C-c",
        // opencode `variant.cycle` (effort) — `effort::set_opencode`.
        "ctrl-t" => "C-t",
        d if d.len() == 1 && d.chars().all(|c| c.is_ascii_digit()) => {
            return Ok(("", Some(d.to_string())));
        }
        other => return Err(anyhow!("không biết phím '{other}'")),
    };
    Ok((t, None))
}

// ── Mặt tiếp xúc mà `keys.rs` gọi tới ──────────────────────────────────────

pub fn window_of(tty: &str) -> Result<Option<i64>> {
    let t = tty.trim().trim_start_matches("/dev/");
    if !crate::sessions::is_real_tty(t) {
        return Ok(None);
    }
    Ok(panes()?.into_iter().find(|p| p.tty == t).map(|p| p.window))
}

/// Mỗi pane một tty — không có "tab khác cùng cửa sổ" để mà nhầm.
pub fn window_of_any(tty: &str) -> Result<Option<i64>> {
    window_of(tty)
}

pub fn selected_tab_tty(window: i64) -> Result<String> {
    pane(window)?
        .map(|p| p.tty)
        .ok_or_else(|| anyhow!("pane %{window} không còn"))
}

pub fn window_gone(window: i64) -> Result<bool> {
    Ok(pane(window)?.is_none())
}

/// Chữ đang hiện trong pane (khung nhìn), giữ nguyên chỗ gãy dòng như màn —
/// cùng hình dạng `contents of tab` của Terminal.app mà bộ phân tích đã quen.
pub fn screen_text(window: i64) -> Result<String> {
    tmux(&["capture-pane", "-p", "-t", &dich(window)])
}

/// Khung nhìn + `steps` khung cuộn ngược. `du` không cần: tmux trả một lượt.
pub fn screen_scrollback(window: i64, steps: usize, _du: impl Fn(&str) -> bool) -> Result<String> {
    let (hang, _) = window_size(window)?;
    let lui = format!("-{}", (steps as i64 + 1) * hang.max(1));
    tmux(&["capture-pane", "-p", "-S", &lui, "-t", &dich(window)])
}

/// Gõ một khối chữ — KHÔNG tự bấm Enter (hợp đồng của `keys::type_and_send`).
pub fn type_into(window: i64, text: &str) -> Result<()> {
    if text.contains('\n') || text.contains('\r') {
        let buf = format!("huba-{}-{window}", std::process::id());
        tmux(&["set-buffer", "-b", &buf, "--", text])?;
        // `-p`: dán có ngoặc NẾU chương trình đã xin (claude có); `-d`: xoá bộ đệm.
        tmux(&["paste-buffer", "-p", "-d", "-b", &buf, "-t", &dich(window)])?;
        Ok(())
    } else {
        tmux(&["send-keys", "-t", &dich(window), "-l", "--", text]).map(|_| ())
    }
}

/// Dãy phím RỜI, nghỉ 30 ms giữa các phím để TUI kịp vẽ (như bản Windows).
pub fn press_writes(window: i64, writes: &[Vec<String>]) -> Result<()> {
    for group in writes {
        for key in group {
            if key == "clear" {
                continue; // không phải một phím thật — xem `clear_box`.
            }
            match ten_phim(key)? {
                (_, Some(chu)) => {
                    tmux(&["send-keys", "-t", &dich(window), "-l", "--", &chu])?;
                }
                (t, None) => {
                    tmux(&["send-keys", "-t", &dich(window), t])?;
                }
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    }
    Ok(())
}

pub fn send_bare(window: i64, keys: &[String]) -> Result<()> {
    press_writes(window, &[keys.to_vec()])
}

/// Bận/rảnh: chương trình đứng trước trong pane có phải shell không.
pub fn tab_state(window: i64) -> Result<TabState> {
    Ok(match pane(window)? {
        None => TabState::Gone,
        Some(p) if la_shell(&p.lenh) => TabState::Idle,
        Some(_) => TabState::Busy,
    })
}

/// Số tiến trình trên tty của pane (shell + con). Pane đã hết ⟹ 0: shell thoát
/// thì tmux đóng pane, nên "hết pane" và "0 tiến trình" là một.
pub fn tab_proc_count(window: i64) -> Result<usize> {
    Ok(tab_process_count(window)?.unwrap_or(0))
}

/// Pane `%window` có tiến trình tên `name` không — bản Linux của `keys::tab_runs`
/// (`processes of tab` của Terminal.app). Pane đã hết ⟹ `Err`, không phải `false`:
/// "không có pane" và "pane không chạy opencode" dẫn tới hai cách thoát khác nhau.
pub fn tab_runs(window: i64, name: &str) -> Result<bool> {
    let Some(p) = pane(window)? else {
        return Err(anyhow!("pane %{window} không còn"));
    };
    let r = run(
        "ps",
        &["-o", "comm=", "-t", &p.tty],
        RunOpts {
            timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    )?;
    if r.code != Some(0) {
        return Err(anyhow!(
            "ps -t {} thoát {:?}: {}",
            p.tty,
            r.code,
            r.stderr.trim()
        ));
    }
    Ok(crate::keys::tab_procs_has(&r.stdout, name))
}

/// `Ok(None)` = pane đã HẾT — cùng hợp đồng với macOS.
pub fn tab_process_count(window: i64) -> Result<Option<usize>> {
    let Some(p) = pane(window)? else {
        return Ok(None);
    };
    let r = run(
        "ps",
        &["-o", "pid=", "-t", &p.tty],
        RunOpts {
            timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    )?;
    Ok(Some(
        r.stdout.lines().filter(|l| !l.trim().is_empty()).count(),
    ))
}

/// Cỡ pane tính bằng KÝ TỰ `(hàng, cột)` — cùng đơn vị với macOS.
pub fn window_size(window: i64) -> Result<(i64, i64)> {
    let out = tmux(&[
        "display-message",
        "-p",
        "-t",
        &dich(window),
        "#{pane_height} #{pane_width}",
    ])?;
    let mut it = out.split_whitespace().map(|x| x.parse::<i64>());
    match (it.next(), it.next()) {
        (Some(Ok(h)), Some(Ok(w))) => Ok((h, w)),
        _ => Err(anyhow!("tmux không trả cỡ pane %{window}: {out:?}")),
    }
}

/// Chọn window + pane trong tmux — client nào đang `attach` sẽ thấy nó.
pub fn bring_to_front(window: i64) -> Result<()> {
    tmux(&["select-window", "-t", &dich(window)])?;
    tmux(&["select-pane", "-t", &dich(window)]).map(|_| ())
}

pub fn focus_window(window: i64) -> Result<()> {
    bring_to_front(window)
}

/// Pane đang được chọn của phiên tmux mà client gần nhất đang xem.
pub fn front_window() -> Result<Option<i64>> {
    match tmux(&["display-message", "-p", "#{pane_id}"]) {
        Ok(out) => Ok(so_pane(&out)),
        Err(e) if khong_co_may_chu(&e.to_string()) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn terminal_pid() -> Result<i32> {
    Err(anyhow!(
        "không có khái niệm PID Terminal.app trên Linux (tmux)"
    ))
}

/// Đóng pane — chỉ khi shell ĐANG RẢNH (không đè lên một chương trình đang chạy).
pub fn close_window(window: i64) -> Result<Closed> {
    match tab_state(window)? {
        TabState::Gone => return Ok(Closed::Gone),
        TabState::Busy => {
            anyhow::bail!("pane %{window} còn chương trình đang chạy — không đóng đè lên")
        }
        TabState::Idle => {}
    }
    // Cửa sổ xem trước: giết pane trước thì phiên nhóm vẫn còn window khác của
    // `huba` ⟹ cửa sổ trên màn hình nhảy sang một phiên KHÁC thay vì đóng.
    dong_cua_so_xem(window);
    tmux(&["kill-pane", "-t", &dich(window)])?;
    if window_gone(window)? {
        Ok(Closed::Gone)
    } else {
        anyhow::bail!("đã kill-pane mà pane %{window} vẫn còn")
    }
}

pub fn close_hidden_again(window: i64) -> Result<bool> {
    let _ = window;
    Ok(false)
}

pub fn screen_locked() -> Option<bool> {
    None
}

pub fn photograph_window(window: i64, path: &std::path::Path) -> Result<()> {
    let _ = (window, path);
    Err(anyhow!("không có ảnh trên tmux — dùng /shot (chữ)"))
}

pub fn frame_is_blank(path: &std::path::Path) -> Option<bool> {
    let _ = path;
    None
}

/// Xoá ô nhập bằng Backspace đủ số ký tự đang hiện, rồi PHÁN bằng lượt đọc lại.
pub fn clear_box(window: i64) -> Result<Cleared> {
    use crate::keys::{box_state, BoxState};
    let n = match box_state(&screen_text(window)?) {
        BoxState::NoBox => return Ok(Cleared::NoBox),
        BoxState::Empty => return Ok(Cleared::Clean),
        BoxState::Text(n) => n,
    };
    let lan = n.to_string();
    tmux(&["send-keys", "-t", &dich(window), "-N", &lan, "BSpace"])?;
    std::thread::sleep(Duration::from_millis(150));
    Ok(match box_state(&screen_text(window)?) {
        BoxState::Empty => Cleared::Clean,
        BoxState::NoBox => Cleared::NoBox,
        BoxState::Text(_) => Cleared::TextLeft,
    })
}

pub fn clear_queue(window: i64) -> Result<(usize, usize)> {
    let _ = window;
    Err(anyhow!("chưa cài: xoá hàng chờ trên Linux"))
}

/// Gõ vào pane tmux không đòi quyền hệ điều hành nào — luôn `true`.
pub fn trusted() -> bool {
    true
}

/// Mọi pane, kèm chữ trên màn khi `with_screens`.
pub fn terminal_tabs(with_screens: bool) -> Result<Vec<Tab>> {
    let mut tabs = Vec::new();
    for p in panes()? {
        let screen = if with_screens {
            match screen_text(p.window) {
                Ok(s) => Some(s),
                Err(e) => {
                    crate::logging::warn(
                        "tmux_pane_unreadable",
                        serde_json::json!({ "pane": p.window, "err": e.to_string() }),
                    );
                    continue;
                }
            }
        } else {
            None
        };
        let procs = run(
            "ps",
            &["-o", "comm=", "-t", &p.tty],
            RunOpts {
                timeout: Some(Duration::from_secs(5)),
                ..Default::default()
            },
        )
        .map(|r| {
            r.stdout
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_else(|e| {
            crate::logging::warn(
                "tmux_pane_procs_unreadable",
                serde_json::json!({ "pane": p.window, "tty": p.tty, "err": e.to_string() }),
            );
            Vec::new()
        });
        tabs.push(Tab {
            busy: !la_shell(&p.lenh),
            tty: p.tty,
            procs,
            screen,
            title: p.title,
        });
    }
    Ok(tabs)
}

// ── Cửa sổ XEM trên màn hình ───────────────────────────────────────────────
//
// Quang 13/11 sau `/new -a acc8` đầu tiên trên Ubuntu: *"chưa thấy mở được
// terminal"*. Pane tmux tách rời là đường ĐIỀU KHIỂN (gõ/đọc không cần màn
// hình), nhưng trên máy có người ngồi thì `/new` của Mac mở một cửa sổ THẬT —
// thiếu nó là thiếu đúng thứ luật cầu nối đòi. Nên: tmux vẫn giữ pane, còn một
// cửa sổ terminal trên màn hình GẮN vào pane ấy. Đo tay trên Ubuntu 26.04 /
// GNOME Wayland: `ptyxis --new-window -- tmux attach -t huba` ⟹ Quang thấy opencode.
//
// Mỗi cửa sổ xem là một phiên NHÓM `huba-xem-<pane>` (chung window với [`PHIEN`],
// window đang xem riêng): `attach` thẳng vào `huba` thì `/new` thứ hai kéo cả
// cửa sổ thứ nhất sang phiên mới. `destroy-unattached` ⟹ chủ máy đóng cửa sổ là
// phiên nhóm tự hết; pane (và phiên claude trong đó) vẫn sống.

/// Tiền tố tên phiên nhóm của cửa sổ xem.
pub const PHIEN_XEM: &str = "huba-xem-";

/// Trình terminal biết cách chạy một lệnh: `(tên, tham số đứng trước lệnh)`.
/// Thứ tự = thứ được thử trước; ptyxis là terminal mặc định của Ubuntu ≥ 25.10.
pub const TERMINALS: &[(&str, &[&str])] = &[
    ("ptyxis", &["--new-window", "--"]),
    ("gnome-terminal", &["--window", "--"]),
    ("kgx", &["--"]),
    ("konsole", &["-e"]),
    ("xfce4-terminal", &["-x"]),
    ("xterm", &["-e"]),
];

/// Dòng lệnh mở cửa sổ xem pane `%w` của phiên `phien` bằng `term`. Thuần.
/// `None` ⟹ `term` không có trong [`TERMINALS`] (đoán cờ của một trình lạ là
/// mở ra một cửa sổ chạy SAI lệnh).
pub fn lenh_xem(term: &str, phien: &str, w: i64) -> Option<Vec<String>> {
    let ten = std::path::Path::new(term).file_name()?.to_str()?;
    let (_, truoc) = TERMINALS.iter().find(|(t, _)| *t == ten)?;
    let mut v: Vec<String> = std::iter::once(term.to_string())
        .chain(truoc.iter().map(|s| s.to_string()))
        .collect();
    let xem = format!("{PHIEN_XEM}{w}");
    for a in [
        "tmux",
        "new-session",
        "-t",
        &format!("={phien}"),
        "-s",
        &xem,
        ";",
        "set-option",
        "destroy-unattached",
        "on",
        ";",
        "select-window",
        "-t",
        &dich(w),
    ] {
        v.push(a.to_string());
    }
    Some(v)
}

/// Có màn hình đồ hoạ để mở cửa sổ không — đọc biến môi trường. Thuần (nhận hàm đọc).
pub fn co_man_hinh(env: impl Fn(&str) -> Option<String>) -> bool {
    ["WAYLAND_DISPLAY", "DISPLAY"]
        .iter()
        .any(|k| env(k).is_some_and(|v| !v.trim().is_empty()))
}

/// Trình terminal dùng được: `HUBA_TERMINAL` (tên trong [`TERMINALS`]) thắng,
/// không thì cái đầu tiên có trên PATH.
fn chon_terminal() -> Option<String> {
    let tren_path = |ten: &str| -> Option<String> {
        std::env::var_os("PATH").and_then(|p| {
            std::env::split_paths(&p)
                .map(|d| d.join(ten))
                .find(|f| f.is_file())
                .map(|f| f.display().to_string())
        })
    };
    if let Some(t) = std::env::var("HUBA_TERMINAL")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        return Some(if t.contains('/') {
            t
        } else {
            tren_path(&t).unwrap_or(t)
        });
    }
    TERMINALS.iter().find_map(|(t, _)| tren_path(t))
}

/// Mở cửa sổ xem cho pane `%w`. Không mở được KHÔNG làm hỏng `/new` (pane đã có,
/// gõ/đọc vẫn chạy) — nhưng mọi đường bỏ qua đều GHI LOG kèm lý do.
fn mo_cua_so_xem(phien: &str, w: i64) {
    let doc_env = |k: &str| std::env::var(k).ok();
    if !co_man_hinh(doc_env) {
        crate::logging::info(
            "tmux_xem_bo_qua",
            serde_json::json!({ "pane": w, "why": "không có WAYLAND_DISPLAY/DISPLAY — máy không có màn hình đồ hoạ",
                                "effect": "phiên vẫn chạy trong tmux: tmux attach -t huba" }),
        );
        return;
    }
    let Some(term) = chon_terminal() else {
        crate::logging::warn(
            "tmux_xem_bo_qua",
            serde_json::json!({ "pane": w, "why": "không tìm thấy trình terminal nào",
                                "da_thu": TERMINALS.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
                                "effect": "phiên vẫn chạy trong tmux: tmux attach -t huba" }),
        );
        return;
    };
    let Some(argv) = lenh_xem(&term, phien, w) else {
        crate::logging::warn(
            "tmux_xem_bo_qua",
            serde_json::json!({ "pane": w, "terminal": term, "why": "HUBA_TERMINAL không nằm trong danh sách đã biết cờ" }),
        );
        return;
    };
    use std::os::unix::process::CommandExt;
    let spawn = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .process_group(0)
        .spawn();
    if let Err(e) = spawn {
        crate::logging::warn(
            "tmux_xem_khong_mo",
            serde_json::json!({ "pane": w, "terminal": term, "err": e.to_string() }),
        );
        return;
    }
    // Mã thoát của trình terminal KHÔNG nói gì (ptyxis chuyển yêu cầu cho bản
    // đang chạy rồi thoát 0). Phép đo thật: có client tmux gắn vào phiên xem chưa.
    let xem = format!("{PHIEN_XEM}{w}");
    for _ in 0..25 {
        std::thread::sleep(Duration::from_millis(200));
        if let Ok(out) = tmux(&[
            "list-clients",
            "-t",
            &format!("={xem}"),
            "-F",
            "#{client_tty}",
        ]) {
            if !out.trim().is_empty() {
                crate::logging::info(
                    "tmux_xem_da_gan",
                    serde_json::json!({ "pane": w, "terminal": term, "client": out.trim() }),
                );
                return;
            }
        }
    }
    crate::logging::warn(
        "tmux_xem_khong_gan",
        serde_json::json!({ "pane": w, "terminal": term, "cho_ms": 5000,
                            "effect": "không thấy cửa sổ trên màn hình — phiên vẫn chạy: tmux attach -t huba" }),
    );
}

/// Đóng cửa sổ xem của pane `%w` (giết phiên nhóm ⟹ client thoát ⟹ terminal đóng).
/// Không có phiên xem là chuyện thường (chủ máy đã đóng, máy không màn hình).
fn dong_cua_so_xem(w: i64) {
    let xem = format!("{PHIEN_XEM}{w}");
    match tmux(&["kill-session", "-t", &format!("={xem}")]) {
        Ok(_) => crate::logging::info("tmux_xem_da_dong", serde_json::json!({ "pane": w })),
        Err(e) if e.to_string().contains("can't find session") => {}
        Err(e) if khong_co_may_chu(&e.to_string()) => {}
        Err(e) => crate::logging::warn(
            "tmux_xem_khong_dong",
            serde_json::json!({ "pane": w, "err": e.to_string() }),
        ),
    }
}

/// Mở một WINDOW tmux mới (phiên [`PHIEN`], chưa có thì dựng tách rời), gõ `cmd`
/// vào shell của nó, trả `(số pane, "pts/N")` — kèm một cửa sổ XEM trên màn hình
/// khi máy có màn hình đồ hoạ ([`mo_cua_so_xem`]).
pub fn open_window(cmd: &str) -> Result<(i64, String)> {
    let (w, tty) = mo_pane(cmd)?;
    mo_cua_so_xem(&ten_phien(), w);
    Ok((w, tty))
}

/// Phần tmux của [`open_window`], KHÔNG bật cửa sổ — tự kiểm dùng cái này.
fn mo_pane(cmd: &str) -> Result<(i64, String)> {
    let phien = ten_phien();
    let khuon = "#{pane_id} #{pane_tty}";
    let co_phien = tmux(&["has-session", "-t", &format!("={phien}")]).is_ok();
    let out = if co_phien {
        tmux(&[
            "new-window",
            "-d",
            "-t",
            &format!("={phien}:"),
            "-P",
            "-F",
            khuon,
        ])?
    } else {
        tmux(&[
            "new-session",
            "-d",
            "-s",
            &phien,
            "-x",
            CO_COT,
            "-y",
            CO_HANG,
            "-P",
            "-F",
            khuon,
        ])?
    };
    let mut it = out.split_whitespace();
    let (Some(w), Some(tty)) = (it.next().and_then(so_pane), it.next()) else {
        return Err(anyhow!("tmux không trả pane mới: {out:?}"));
    };
    let tty = tty.trim_start_matches("/dev/").to_string();
    // Chờ shell vẽ dấu nhắc rồi mới gõ — gõ trước khi shell đọc thì vẫn vào hàng
    // đợi của tty, nhưng chờ một nhịp cho chắc với shell khởi động chậm (.bashrc).
    std::thread::sleep(Duration::from_millis(300));
    tmux(&["send-keys", "-t", &dich(w), "-l", "--", cmd])?;
    tmux(&["send-keys", "-t", &dich(w), "Enter"])?;
    crate::logging::info(
        "tmux_window_opened",
        serde_json::json!({ "pane": w, "tty": tty, "session": phien, "new_session": !co_phien }),
    );
    Ok((w, tty))
}

// ── Tự kiểm trên máy Linux thật ────────────────────────────────────────────

/// Kết quả một mục tự kiểm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ket {
    Dat(String),
    Hong(String),
    /// Không đo được — KHÔNG phải đạt (luật 13②).
    Kdd(String),
}

/// `huba linux-tu-kiem`: mở một pane THỬ trong phiên tmux riêng, gõ một mốc,
/// đọc lại, phím rời, đóng. Không đụng phiên tmux `huba` của chủ máy.
pub fn tu_kiem() -> Vec<(String, Ket)> {
    let mut out: Vec<(String, Ket)> = Vec::new();
    let ver = run(
        "tmux",
        &["-V"],
        RunOpts {
            timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    );
    match ver {
        Ok(r) if r.code == Some(0) => {
            out.push(("① có tmux".into(), Ket::Dat(r.stdout.trim().into())))
        }
        Ok(r) => {
            out.push((
                "① có tmux".into(),
                Ket::Hong(format!("tmux -V thoát {:?}", r.code)),
            ));
            return out;
        }
        Err(e) => {
            out.push((
                "① có tmux".into(),
                Ket::Hong(format!(
                    "không chạy được tmux ({e}) — sudo apt install tmux"
                )),
            ));
            return out;
        }
    }
    let phien = format!("huba-tu-kiem-{}", std::process::id());
    std::env::set_var("HUBA_TMUX_SESSION", &phien);
    // `mo_pane`, không `open_window`: tự kiểm không bật cửa sổ lên màn hình chủ máy.
    let mo = mo_pane("echo HUBA_MOC_$((40+2)); cat");
    let (w, tty) = match mo {
        Ok(x) => x,
        Err(e) => {
            out.push(("② mở pane".into(), Ket::Hong(e.to_string())));
            std::env::remove_var("HUBA_TMUX_SESSION");
            return out;
        }
    };
    out.push(("② mở pane".into(), Ket::Dat(format!("%{w} · {tty}"))));
    let cho = |muc: &str| -> Option<String> {
        for _ in 0..25 {
            std::thread::sleep(Duration::from_millis(200));
            if let Ok(s) = screen_text(w) {
                if s.contains(muc) {
                    return Some(s);
                }
            }
        }
        None
    };
    out.push((
        "③ lệnh chạy trong shell của pane (đọc lại mốc 42)".into(),
        match cho("HUBA_MOC_42") {
            Some(_) => Ket::Dat("thấy HUBA_MOC_42".into()),
            None => Ket::Hong("không thấy mốc sau 5 giây".into()),
        },
    ));
    let gian = format!("go-thu-{}", std::process::id());
    let go = type_into(w, &gian).and_then(|_| press_writes(w, &[vec!["enter".into()]]));
    out.push((
        "④ gõ chữ + Enter, cat in lại".into(),
        match go {
            Err(e) => Ket::Hong(e.to_string()),
            Ok(()) => match cho(&format!("{gian}\n{gian}")) {
                Some(_) => Ket::Dat("chữ vào ô + Enter gửi đi".into()),
                None => Ket::Hong("không thấy chữ in lại hai lần".into()),
            },
        },
    ));
    out.push((
        "⑤ bận khi có chương trình (cat), tty khớp window_of".into(),
        match (tab_state(w), window_of(&tty)) {
            (Ok(TabState::Busy), Ok(Some(x))) if x == w => Ket::Dat("Busy · khớp".into()),
            (a, b) => Ket::Hong(format!("{a:?} · {b:?}")),
        },
    ));
    // /close rẽ opencode bằng câu này (Quang 13/11 «có vẻ thiếu /») — hai chiều:
    // thấy đúng chương trình đang chạy, và KHÔNG nhận nhầm tên khác.
    out.push((
        "⑤b tab_runs thấy cat, không thấy opencode".into(),
        match (tab_runs(w, "cat"), tab_runs(w, "opencode")) {
            (Ok(true), Ok(false)) => Ket::Dat("cat ✓ · opencode ✗".into()),
            (a, b) => Ket::Hong(format!("cat {a:?} · opencode {b:?}")),
        },
    ));
    let ctrl = press_writes(w, &[vec!["ctrl-c".into()]]);
    std::thread::sleep(Duration::from_millis(400));
    out.push((
        "⑥ Ctrl+C dừng chương trình ⟹ rảnh".into(),
        match (ctrl, tab_state(w)) {
            (Ok(()), Ok(TabState::Idle)) => Ket::Dat("Idle".into()),
            (a, b) => Ket::Hong(format!("{a:?} · {b:?}")),
        },
    ));
    out.push((
        "⑦ đóng pane".into(),
        match close_window(w) {
            Ok(Closed::Gone) => Ket::Dat("pane hết".into()),
            other => Ket::Hong(format!("{other:?}")),
        },
    ));
    // Dọn phiên tmux thử — hỏng thì NÓI (một phiên thử bỏ lại là rác trên máy chủ máy).
    out.push((
        "⑦b dọn phiên tmux thử".into(),
        match tmux(&["kill-session", "-t", &format!("={phien}")]) {
            Ok(_) => Ket::Dat(phien.clone()),
            // Pane cuối đóng ở ⑦ thì tmux tự xoá phiên, và nếu đó là phiên cuối thì tắt
            // luôn máy chủ (đo Ubuntu 24.04: "no server running") — cả hai là đã sạch.
            Err(e)
                if e.to_string().contains("can't find session")
                    || khong_co_may_chu(&e.to_string()) =>
            {
                Ket::Dat(format!("{phien} đã tự hết cùng pane cuối"))
            }
            Err(e) => Ket::Hong(format!("phiên {phien} còn sót: {e}")),
        },
    ));
    std::env::remove_var("HUBA_TMUX_SESSION");
    let co_claude = run(
        "claude",
        &["--version"],
        RunOpts {
            timeout: Some(Duration::from_secs(20)),
            ..Default::default()
        },
    );
    out.push((
        "⑧ có claude CLI".into(),
        match co_claude {
            Ok(r) if r.code == Some(0) => Ket::Dat(r.stdout.trim().into()),
            Ok(r) => Ket::Kdd(format!("claude --version thoát {:?}", r.code)),
            Err(e) => Ket::Kdd(format!("không thấy claude ({e}) — cài rồi đo lại")),
        },
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_hang_pane_va_bo_hang_hong() {
        let out = "%3~|huba|~/dev/pts/4~|huba|~1234~|huba|~claude~|huba|~✳ Sửa lỗi\n%x~|huba|~/dev/pts/5~|huba|~1~|huba|~bash~|huba|~\n%7~|huba|~/dev/pts/6\n";
        let p = doc_panes(out);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].window, 3);
        assert_eq!(p[0].tty, "pts/4");
        assert_eq!(p[0].pid, 1234);
        assert_eq!(p[0].title, "✳ Sửa lỗi");
    }

    /// Phiên nhóm của cửa sổ xem làm `list-panes -a` in một pane HAI lần.
    #[test]
    fn mot_pane_hai_phien_chi_tinh_mot() {
        let hang = "%3~|huba|~/dev/pts/4~|huba|~1234~|huba|~opencode~|huba|~t";
        let out = format!("{hang}\n%5~|huba|~/dev/pts/6~|huba|~9~|huba|~bash~|huba|~\n{hang}\n");
        let p = doc_panes(&out);
        assert_eq!(p.iter().map(|x| x.window).collect::<Vec<_>>(), vec![3, 5]);
    }

    #[test]
    fn lenh_xem_gan_dung_pane_bang_phien_nhom() {
        let v = lenh_xem("/usr/bin/ptyxis", "huba", 7).unwrap();
        assert_eq!(&v[..3], &["/usr/bin/ptyxis", "--new-window", "--"]);
        let duoi = v[3..].join(" ");
        assert_eq!(
            duoi,
            "tmux new-session -t =huba -s huba-xem-7 ; set-option destroy-unattached on ; select-window -t %7"
        );
        assert_eq!(lenh_xem("xterm", "huba", 1).unwrap()[1], "-e");
        // Trình lạ: KHÔNG đoán cờ.
        assert!(lenh_xem("/opt/la/terminal-la", "huba", 1).is_none());
    }

    #[test]
    fn man_hinh_do_bien_moi_truong() {
        let env = |m: &'static [(&'static str, &'static str)]| {
            move |k: &str| m.iter().find(|(a, _)| *a == k).map(|(_, b)| b.to_string())
        };
        assert!(co_man_hinh(env(&[("WAYLAND_DISPLAY", "wayland-0")])));
        assert!(co_man_hinh(env(&[("DISPLAY", ":0")])));
        assert!(!co_man_hinh(env(&[])));
        assert!(!co_man_hinh(env(&[("DISPLAY", " ")])));
    }

    #[test]
    fn shell_ranh_va_ban() {
        assert!(la_shell("bash") && la_shell("-zsh"));
        assert!(!la_shell("claude") && !la_shell("node") && !la_shell("cat"));
    }

    #[test]
    fn ten_phim_tmux() {
        assert_eq!(ten_phim("enter").unwrap().0, "Enter");
        assert_eq!(ten_phim("ctrl-c").unwrap().0, "C-c");
        assert_eq!(ten_phim("ctrl-t").unwrap().0, "C-t");
        assert_eq!(ten_phim("backspace").unwrap().0, "BSpace");
        assert_eq!(ten_phim("2").unwrap().1.as_deref(), Some("2"));
        assert!(ten_phim("f13").is_err());
    }

    #[test]
    fn khong_may_chu_la_danh_sach_rong() {
        assert!(khong_co_may_chu(
            "no server running on /tmp/tmux-1000/default"
        ));
        assert!(!khong_co_may_chu("can't find pane: %9"));
    }
}
