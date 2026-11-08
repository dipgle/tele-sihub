# NOTES kỹ thuật — số đo và lịch sử cắt khỏi CLAUDE.md (2026-11-08)

> Tệp này KHÔNG tự nạp. Mỗi mục được trỏ từ `CLAUDE.md`; đọc đúng mục khi đụng việc ấy.

> Nội dung dưới đây là **nguyên văn** phần đã cắt — đừng viết lại, chỉ thêm.


---

## Luật 13 — macOS facts that cost a night to learn (2026-08-10)

13. **macOS facts that cost a night to learn** (2026-08-10). Typing into a live
    session goes through Terminal's own `do script`, NOT `System Events keystroke`
    — the latter is refused outright (*"osascript is not allowed to send
    keystrokes (1002)"*) and no amount of granting Accessibility fixes it,
    because the process asking is `/usr/bin/osascript`, a system binary. `do
    script` needs only **Automation**, which huba already has. It **always
    appends a newline** and that cannot be turned off — but read the next
    paragraph before concluding that a typed line therefore gets SENT.
    🔴 **"Appends a newline" ≠ "the TUI submits it"** (Hà, 2026-08-12: *"nhận
    được text nhưng không tự gửi… có vẻ như thiếu enter?"*). The text and the
    newline arrive in ONE write, and `claude`'s input box reads that as a
    **paste**: the line lands in the box and the newline goes into the content
    instead of ending it. True for a shell, false for this TUI — and the two
    behaved differently long enough for this file to state the wrong rule.
    huba now looks before it speaks: after typing it re-reads the screen, and if
    the text is still sitting in the box it sends a **separate** Enter
    (`keys::still_in_box` + `press(w, "enter")`). Three measured conditions gate
    that extra keystroke, because a stray Enter is not undoable — the text is
    still visible, the session is not busy (busy means it already queued, i.e.
    it WAS submitted), and no choice dialog is on screen (there Enter CONFIRMS).
    A line under 6 characters never triggers it: "2" or "ok" appears on almost
    any screen, and a false match would fire an Enter nobody asked for.
    It also means an arrow key both MOVES and CONFIRMS, so huba sends
    an arrow only when it can **prove there is no choice dialog** — not merely
    when it fails to see one. That distinction is the whole gate: `screen_of`
    used to fold three outcomes into `None` (no window · osascript failed ·
    **the screen looks like it holds a secret**), and the gate read `None` as
    "no dialog" and sent, so it failed OPEN exactly when huba was blindest —
    including the case where a password was on screen. `keys::look` now returns
    `Saw` / `Withheld` / `Blind`, and `keys::arrow_verdict` sends only on a
    proven-empty choice list. `Withheld` still decides correctly: the choice
    COUNT is a number, and a number carries no text off the machine.
    And when the session is mid-run, `claude` **queues** the text
    rather than showing it — so a reply must read the screen back and say WHERE
    the text landed. `osascript` returning 0 proves only that bytes reached the
    tab.
    🔴 **Cái MÀN chỉ là bản VẼ, không phải bản LƯU** (2026-08-19, đo trên phiên
    `[tcc/amm]` khi Hà hỏi *"làm sao đủ nội dung ngữ cảnh"*). Ba con số, đừng
    dò lại:
    · `contents of tab` = **26 đoạn / 1487 ký tự** — đúng khung nhìn;
    · `history of tab` (toàn bộ cuộn lại) = **42 đoạn / 3487 ký tự**, và 16 dòng
      thêm ấy là *"Last login…"* + câu lệnh mở phiên, **0 dòng hội thoại** — TUI
      vẽ ĐÈ tại chỗ nên bộ đệm cuộn rỗng. **Đường này là ngõ cụt, đừng thử lại.**
      🔴 **ĐO LẠI 2026-08-20** khi Hà hỏi *"Tại sao không dùng thanh cuộn"* rồi
      vặn tiếp *"một phiên chạy tầm 50% context thì nó phải dài ít nhất 10 trang
      màn hình"* — câu vặn ấy đúng, và nó lôi ra chỗ câu chữ cũ nói hớ.
      **Thanh cuộn KHÔNG hỏng, và Terminal KHÔNG hề bỏ lưu.** Cửa sổ thử, shell
      thường in 500 dòng: khung nhìn 25 dòng, `history` **504 dòng**, có cả dòng
      ĐẦU đã cuộn mất. AppleScript đọc scrollback tốt.
      Thứ vắng mặt là hội thoại của `claude`: mốc 40 ký tự rút từ **lượt nói thứ
      3** của phiên (lấy từ `.jsonl`) vắng ở **CẢ** `contents` **LẪN** `history`;
      và `history` của cửa sổ ấy đọc **4 lần cách nhau nhiều phút vẫn đúng 43
      dòng** trong khi hội thoại chạy hàng chục lượt — nó đóng băng ở phần trước
      lúc `claude` khởi động (2051 ký tự chênh = `Last login…` + câu lệnh mở
      phiên, 0 dòng hội thoại).
      ⟹ Nói cho đúng: **10 trang lịch sử ấy nằm trong bộ nhớ của chính `claude`**,
      TUI tự vẽ lại khi chủ máy cuộn; Terminal không giữ bản sao nào, nên
      AppleScript không có gì để với tới. Muốn lấy nó phải bắt TUI TỰ CUỘN —
      `keys::screen_scrollback`, bánh xe chuột, đo được 934 → 1391 ký tự sau 10
      lượt. (Đường "bắt TUI VẼ LẠI bằng cách nới cửa sổ" đã chết 30/08, xem tấm
      bia bên dưới.)
      ⚠ **Bẫy đo, cắn ba lượt liên tiếp trước khi ra được số trên:** TUI in DÒNG
      LỆNH đang chạy lên chính cái màn đang đo, nên một mốc viết vào argv (hay
      vào lệnh sinh ra nó) tự xuất hiện trên màn ⟹ phép đo trả "CÓ" ở mọi ô, tự
      khớp chính nó. Mốc phải rút từ NHẬT KÝ và không bao giờ đi qua dòng lệnh.
      Cùng họ với `pgrep -f 'cargo test…'` khớp đúng argv của chính watcher.
    · **CUỘN bằng bánh xe** (`keys::screen_scrollback`) — đường DUY NHẤT còn lại
      để lấy thêm chữ, và nó không đụng tới cỡ cửa sổ. Luôn trả màn về đáy, kể
      cả khi đọc hỏng giữa chừng.
    🔴 **huba KHÔNG ĐỔI KÍCH THƯỚC CỬA SỔ NỮA — Hà 2026-08-30**: *"đừng thay đổi
    kích thước của sổ terminal nữa, bỏ hết các chỗ đi, để nó luôn full màn hình
    nếu là cửa sổ mới, còn là tab thì không động đến kích thước"*.
    · **Cửa sổ MỚI** (`keys::open_window`) sinh ra đã hết cỡ: xin
      `keys::FULL_SCREEN_ASK` (999) ngay trong lượt `do script`, TRƯỚC `delay 1`
      vì `claude` đọc bề ngang lúc khởi động. Terminal **kẹp giùm** cho vừa màn
      hình — xin 999 nhận về 61×206, không một lỗi nào — nên một dòng mã lấy
      đúng tối đa ở MỌI màn hình, kể cả cái chưa ai đo. `open_window` trả về cả
      cỡ THẬT rồi ghi `window_opened_full_screen`: "đã mở hết cỡ" là một mệnh đề,
      và mệnh đề nào cũng phải có số đứng sau.
    · **Cửa sổ / tab ĐANG SỐNG**: không đụng, không bao giờ. Đọc được tới đâu thì
      nói tới đó, và **khai ra chỗ đọc hụt** (`⚠ Màn đang hẹp…`, `cut_note` của
      `/shot`, `keys::tab_bar_cut` → `ask_table_seen`) chứ không im.
    🪦 `keys::screen_text_tall` + `GROW_ASK` (19/08–30/08) nới cửa sổ đang sống
    rồi trả lại chiều cũ. Nó lấy thêm thật — đo được: `24×80 ⟹ 1081 ký tự` ·
    `nới cao ⟹ 2689` · `nới cả ngang ⟹ **3943**` — nhưng cái giá là cửa sổ của
    chủ máy tự phóng to rồi thu nhỏ trước mắt anh ở mỗi `/shot`, mỗi `/pick`, mỗi
    cú bấm phím nguy hiểm (tới **ba** lần một lượt `/pick`, mỗi lần kèm
    `delay 1.2` cứng). **Đừng dựng lại nó dưới một cái tên khác** — cổng
    `rust/tests/no_window_resizing.rs` sẽ đỏ.
    ⚠ `61×206` là trần CỨNG của màn hình này: một lượt dài hơn khung ấy thì màn
    không có cách nào lấy trọn — nhật ký mới giữ được, xem dưới.
    🔴 **Và đừng hỏi màn "có lời nào của phiên không"** (cùng ngày, ảnh `/shot`
    của chính phiên `[huba]`): phép đo cũ đếm ký tự `⏺`, mà `⏺` đứng ở ĐẦU lượt
    — đúng thứ cuộn khỏi khung nhìn trước nhất. Nó khai *"màn không có lời nào
    của phiên"* trong khi màn toàn là lời của phiên, rồi dán lại 600 ký tự đầu
    của chính lượt ấy. Hỏi NHẬT KÝ rồi đối chiếu với màn
    (`sessions::said_shown_on_screen`), và chỉ bù khi nới hết cỡ rồi vẫn thiếu —
    lúc bù thì bù NGUYÊN VĂN, `split_for_telegram` đã lo phần cắt.
    Bản sao đầy đủ của HỘI THOẠI thì nằm ở nhật ký `.jsonl`, không ở màn. Ngoại
    lệ đã đo: **bảng hỏi ĐANG TREO chưa được ghi vào nhật ký** (0 lần
    `AskUserQuestion` trong 3,6 MB nhật ký amm trong khi hộp nằm trên màn) — nên
    hộp chọn buộc phải đọc từ màn.
    🔴 **Ô nhập của `claude` KHÔNG còn khung** (cùng ngày): `╭` `╰` `│` đều **0
    lần** trên màn thật; nay chỉ còn **một vạch `─` suốt bề ngang**, và ô nằm
    GIỮA HAI vạch. Neo `rfind('╭')` vì thế trượt ở mọi lượt đọc rồi rơi âm thầm
    về đường lùi "bốn dòng cuối" — đủ để mọi thứ trông vẫn chạy, và đã trả giá:
    khối kết quả `▶️` nằm lại trong ô nhập **hơn một tiếng** trong khi huba báo
    *"✅ đã dán vào phiên"*, còn `⌫` thì hai lần đều không xoá nổi. Một cái neo
    duy nhất: `keys::box_start`.
    **Opening and closing a window** (2026-08-11). `do script "<cmd>"` makes a
    new window and returns its *tab*; `tty of` that tab is the handle that ties
    it to the `claude agents` row that appears seconds later. Closing is
    ordered, and the order is not politeness: `/exit` first, then close, because
    closing a window with a live process raises Terminal's own *"Do you want to
    terminate running processes?"* modal — and a modal **blocks every automation
    command after it**, so getting this backwards gags huba. Two measurements
    pin the steps: typing `/exit` really does end the CLI (pid gone, `busy` →
    `false`), so no `kill` and no lost end-of-session bookkeeping; and the
    window does **not** close itself when the shell exits (the profile keeps it,
    showing `[Process completed]`), so the close step is required, not cosmetic.
    If the tab is still busy after 10s, huba refuses to close — better a live
    window than a modal that silences everything.
    One trap unique to this path: the command goes through a **shell**, so every
    `DENIED_TOOLS` pattern must be quoted. `Bash(git push:*)` bare is a shell
    syntax error, and the window would open to a red line with no session and no
    guard rail. The `--bg` path never had this because it passes argv directly.
    **Autostart survives rebuilds now — and here is the whole mechanism, because
    two of the three obvious ways to do it are wrong** (measured 2026-08-10).
    TCC pins a grant to a binary's *designated requirement*. `cargo` ad-hoc-signs
    everything it links, and an ad-hoc DR is `cdhash H"…"` — a hash of the bytes,
    so every build is a different program to macOS. Signing with a certificate
    changes the DR to `identifier "com.dipgle.hubd" and certificate root = H"…"`,
    which is an *identity*: proven by two builds with different bytes
    (`6e9f7db7…` → `bb381cfe…`) carrying the same DR. The cert is self-signed,
    lives in the login keychain, and is deliberately **not** trusted — `codesign`
    signs happily with an untrusted identity and TCC matches on the requirement,
    so nothing here needs an admin password.
    - **Signing `target/release/hubad` in place does not hold.** The next
      `cargo test --release` or `cargo clippy --all-targets` relinks and stamps
      its own ad-hoc signature over it, silently. Caught only because `hubad`
      prints `hubd_signature` at boot and it read `adhoc` twenty minutes after
      being signed `cert`. So launchd runs an **installed copy** at
      `~/Library/Application Support/hub/bin/hubd`, out of cargo's reach; put it
      there with `install_update.sh`, never by hand.
    - **That split introduces its own silent failure** — build, test green,
      deploy the page, and the daemon is still running yesterday's code because
      nobody ran `install.sh`. The health panel answers it, by comparing the
      newest `.rs` under `rust/src` against the installed binary's mtime.
      **Not** by comparing the built artifact: `cargo test --release` produces a
      *different binary* from `cargo build --release` (`2f624e8b…` vs
      `bbd8ba58…`, and the next build flips it back), so any check reading
      `target/` cries wolf after every test run, and a warning that cries wolf
      is a warning nobody reads.
    `hubad` prints `hubd_boot` before touching anything, then `hubd_signature`
    (`cert` · `adhoc` · `unreadable`), so "never entered main", "will lose its
    grants at the next build" and "cannot even read itself" are three
    distinguishable lines instead of one pid sitting there.
    **What was NOT true, though the last two sessions believed it:** that TCC
    blocks the launchd copy from `~/Documents`. It does not — the launchd copy
    loads `huba.env` and the pid lock from there, with `hub_env_loaded` as
    evidence. The `EX_CONFIG` (78) hang was `StandardOutPath`, which **launchd
    itself** opens before the program runs; moving the logs to `~/Library/Logs`
    fixed that and nothing else was ever blocked. A binary running by hand from
    a terminal borrows that terminal's grants and honestly reports `adhoc`; only
    the launchd copy needs an identity of its own.


---

## Luật 11b — a failed measurement is not a fact about the world (2026-08-12)

11b. **A failed measurement is not a fact about the world** (2026-08-12, two
    bugs one family — the same family as `keys::look` returning `Blind`).
    - `claude agents` failed for all three accounts at 14:44:07 (`spawn claude
      failed: No such file or directory` — `npm` was overwriting the binary mid-
      run, from a session that auto-updated itself every 30 minutes). The list
      came back EMPTY and rule 11's *"leaving the list means it ended"* read that
      empty list as three deaths: three `⏹ đã tắt` messages in 8 seconds, for
      three sessions that were alive (one of them kept working for another two
      hours). The snapshot now carries `blind` (accounts that could not be
      listed, machine-readable — `notes` is the sentence for humans), the book
      remembers each session's account (`watch::Mark::a`), and a session missing
      from a blind account KEEPS ITS BOOK ENTRY and says nothing
      (`session_end_unknown`). Keeping the entry is half the fix: dropping it
      made the session come back as "new", so its REAL death got announced a
      second time. One blind account must not gag the others — there is a test.
    - **A tty is a number that gets REUSED**, so "is some tab still holding this
      tty" answers "is there a window", never "is it still THAT session's
      window". `projects-d8` lived in `ttys002` (open since 12:28); the owner
      exited the CLI and typed `claude` again in that same window at 16:41:16;
      at 16:42:33 huba noticed the old session gone, asked about the tty, and
      said *"cửa sổ terminal còn mở"* — technically true, and a lie in effect,
      because the window it describes is already running something else. Ask
      your OWN snapshot first (`sessions::window_taken_over`) and name the
      session that took the window over.
    - **`??` is not a window, and `??` == `??` is not a takeover** (2026-08-12,
      the same fix biting back). `ps` prints `??` for a process with no
      controlling terminal, and `??` is not empty — so the `tty.is_empty()` gate
      let it through, `window_taken_over` matched `??` against `??`, and huba
      announced `⏹ huba-67 đã tắt — cửa sổ ấy nay đang chạy phiên huba-ec.` about
      **two of its own `/usage` probes, neither of which ever had a window**.
      The rule was already written correctly in `keys::window_of` and in the
      `terminal`/`detached` labeller — three hand copies, and the fourth site
      forgot. One predicate now: **`sessions::is_real_tty`**.
    - **huba's own machinery must never ring** (Hà, reading that message:
      *"tại sao 1 phiên đã tắt mà vẫn gắn nút vào phiên"* · *"quá vô lý"*). The
      lifetime gate (`MIN_LIFE_SEC`) catches most `/usage` probes, but it
      measures the wrong thing: what makes that death not-news is not that it
      was *short*, it is that it is *huba's*. Measured: one probe sat in
      `claude agents` for **11 minutes** (the probe hung to its 60s ceiling),
      walked past the 120s gate, and rang. The tell is `cwd` — hubad runs from
      its own `WorkingDirectory` and children inherit it, and no human session
      lives there (`sessions::is_hub_own_probe`). Two gates, because a session
      can be filtered while alive *or* already sitting in the book from before
      the upgrade — dropping one leaves a burst of false deaths on first run.


---

## Luật 10 — `claude` CLI facts that cost a real run to learn

10. **`claude` CLI facts that cost a real run to learn.** Do not re-derive them:
    `--bg` conflicts with `-p`, so the prompt is POSITIONAL — and
    `--disallowedTools` is VARIADIC, so a prompt placed after it is eaten as one
    more pattern and the agent comes up with no task at all. `claude stop` takes
    the SHORT id (8 chars); a full uuid answers "No job matching". Resuming a
    LIVE background agent is refused outright — stop it first, then `--resume`
    appends to the same session (same id, transcript grows). There is no
    primitive for typing into a running session, so UC-S05b level 1 has no path.
    And a background session opened anywhere under this workspace comes up
    `state: "blocked"` on an interactive MCP-approval dialog it can never
    answer; `sessions::start_background` watches for that, stops it, and reports
    the one-time fix rather than claiming success.
    **A BACKGROUND subagent's `tool_result` arrives immediately** — it says
    "agent launched", not "agent finished" — so pairing `tool_use` with
    `tool_result` reports a fan-out as over the instant it starts. Measured
    2026-08-10: two agents running, `huba sessions` said `pending 0`, and the
    background case is precisely the one the count exists for (a blocking
    subagent leaves the parent visibly busy; a background one leaves it looking
    idle). Three structural facts make the real answer reachable, none of them a
    string match on CLI prose: a background agent writes
    `<slug>/<session_id>/subagents/agent-<agentId>.jsonl` (`isSidechain: true`)
    beside `agent-<agentId>.meta.json`, whose `toolUseId` names the call that
    spawned it; the parent transcript later receives a `<task-notification>`
    block whose `<tool-use-id>` is that same id — carried by **three different
    record shapes**, not one: a normal `user` turn
    (`message.content[].text`) when the parent was idle, and
    `queue-operation.content` then `attachment.prompt` when the agent finished
    while the parent was mid-command. Reading only the first shape leaves a
    ghost on the screen forever, and no acceptance script catches it, because
    they all measure while an agent is genuinely running; and a subagent is NOT
    a process, so `ps` will never find one. A dead session's un-notified agents
    are NOT running — the process took them with it — so liveness gates the
    count (`sessions::pending_for_display`), or the screen grows ghosts.


---

## Stack — deps, cửa sổ test, và bẫy `deps/`

## Stack

- **Rust 2021**, crate in `rust/`, two binaries: `huba` (CLI) and `hubad` (loop).
  Deliberately **synchronous** — this process spends its life waiting on
  `claude` and a Telegram long-poll, so an async runtime would add moving parts
  without removing a single wait.
  🔴 **`unsafe` sống ở ĐÚNG MỘT TỆP: `cgkeys.rs`** (Hà gỡ luật *"no unsafe
  anywhere"* ngày 2026-08-19, cho đúng chỗ này). Vì sao phải có: mọi lượt ghi
  qua `do script` kèm một CR không tắt được, nên trên hộp chọn huba **không có
  phím nào chỉ DI mà không CHỐT** — một nút "sang tab bên phải" sẽ trả lời hộ
  câu đang mở. Đo được, và trả giá bằng việc thật: một cú Enter lạc chốt
  `☐ RPC pool` → `☒` trên bảng hỏi của phiên amm. `CGEventPostToPid` đưa phím
  rời thẳng vào tiến trình Terminal, không qua `do script`, nên không kèm gì.
  Đổi lại: cần quyền **Trợ năng** cấp cho `hubad` — bám được là nhờ nó đã ký
  chứng chỉ cố định từ 10/08 (ad-hoc thì mỗi lần build là một chương trình khác
  và quyền rụng sau đúng một `cargo build`).
  Ba luật của tệp ấy, đọc trước khi sửa: `unsafe` không rời khỏi nó · hỏi quyền
  SAU khi thử (tiến trình nền chỉ vào được danh sách Trợ năng khi đã THỬ, nên
  từ chối thử = không bao giờ được cấp) · dịch hết tên phím TRƯỚC khi gửi phím
  đầu tiên.
- Deps: `rusqlite` (bundled), `reqwest` (blocking + rustls), `serde`/`serde_json`,
  `clap`, `regex`, `chrono`, `anyhow`, `base64`. All in the local cargo cache →
  `cargo build --offline` works.
  🔴 `tungstenite`, `axum` and `tokio` were dropped 2026-08-14 — **none had a
  single `use` left in `src/`**, and cargo never said a word. `tungstenite` was
  the `/ws/chat` socket; `axum`+`tokio` were built for a local web console
  deleted on 2026-08-08 (rule 12) and outlived it by six days, dragging an async
  runtime into a deliberately synchronous process. `huba setup` never used them —
  it is a std-library `TcpListener`.
- Store: `data/huba.sqlite` (WAL). Three tables — `runs`, `cursors`, `spend`
  (plus `schema_meta`). The four inbox tables are GONE as of schema step 4
  (2026-08-10): they had outlived the product by two days and held 379 rows no
  query could reach. Two facts worth keeping from that migration — the four
  reference each other, so it has to run with `foreign_keys` OFF (the first
  version died at boot on `FOREIGN KEY constraint failed`, exit 70), and it
  logs each table with its row count rather than a silent "cleaned up".
  🔴 `runs` changed WRITER on 2026-08-14, not shape: it held one row per channel
  poll, and the poll stage went with tfl5. `run_once` now writes one row per
  cycle. Leaving it unwritten was the alternative, and it is the worse one — the
  "recent errors" line in `/doctor` reads this table, so an empty writer makes
  a panel that can never go red. A measurement that cannot fail is worse than no
  measurement: it still occupies the screen and still reads as reassurance.
  ⚠ That `/doctor` sentence was **false when first written here**, and the
  correction is the point: the errors block lived in `runtime::snapshot`, whose
  only caller was `portal.rs` — so it had no reader at all, and `/doctor` never
  showed it. Found by Hà asking what `/doctor` actually does. Fixed by moving
  the code to match the promise (`pipeline::recent_errors_line`), not the
  promise to match the code. `runtime::snapshot` + `errors_block` are now
  orphaned and should go with `portal.rs`.
  **And "one row per cycle" alone was not enough**, which is the part worth
  remembering: `run_once` almost never returns `Err` (every handler swallows its
  failure into a sentence for whoever typed the command), so every row would
  have been `ok` and the panel would have been blind a second time, one commit
  after the warning against it. So a cycle is judged by the `error` LINES it
  produced (`logging::error_count`), not by its own return value — rule 3
  already requires every error path to log, so this is the same claim read from
  the other end, not an approximation. Only the event NAME is carried onto the
  row, never `fields`: that string reaches the phone through `/doctor`, and
  `fields` is where a bot token leaked once already.
  📐 Measured before trusting it (`logs/huba.log`, 2026-08-14): 83,060 `info` ·
  1,626 `warn` · 120 `error`. So this panel means *errors*, not *all trouble* —
  most of huba's trouble lives at `warn` and deliberately stays there. An empty
  block reads "no ERRORS", never "nothing worth looking at".
- Tests: `cd rust && cargo test --offline --no-fail-fast` → **953 tests across 148
  blocks (147 test binaries + doctests), 0 warnings** — and it runs in **55
  seconds** (measured 2026-09-13). `--no-fail-fast` is not optional: plain `cargo
  test` stops at the first red target, and the run that taught us printed
  `66 ok / 1 failed` having touched **67 of 141** test files — a denominator that
  reads like a verdict.
  🔴 **A gate that takes hours is not a slow test suite — it is
  `target/debug/deps/`.** The first run of a freshly-built test binary stalls
  **~125 seconds with `user 0.00 sys 0.00`** (waiting, not computing) — but ONLY
  while it sits in `deps/`. Measured 2026-09-13; same bytes, four places, same
  minute, first execution of each:

  | where the identical bytes sat | first run |
  |---|---|
  | `.tmp/verbs_copy` — outside `target/` | **1s** |
  | `target/debug/deps/verbs-7c82…` | **103s** |
  | `target/debug/deps/zz_probe_dir` — same bytes, new name, still `deps/` | **121s** |
  | `target/debug/zz_thu/zz_probe_sach` — `target/debug/`, empty dir | **0s** |

  `deps/` held **1,375,586 entries** when that was measured. Four things it is
  NOT, each measured, so nobody re-derives them: not the binary's size
  (1,584,920 B stalls as long as 9,182,112 B), not the signature (`codesign -f
  -s -` on fresh binaries still took 178s · 166s), not `cargo` (a 9.9 MB
  `rustc`-built binary outside `deps/` runs in 2s, with or without `-C
  debuginfo=2`), and not `com.apple.provenance` (a system xattr — `xattr -c`
  leaves it in place). Running them **24 at a time changes nothing**: they still
  finish ~128s apart (111 · 210 · 319 · 441 · 548 · 645 · 777 · 920), so whatever
  does this serialises absolutely. Do not "fix" this with parallelism, and read
  any past "2x from parallel" as a suite that happened to hold binaries already
  run once.
  ⇒ **The fix costs two seconds: copy the test binaries out of `deps/` and run
  the copies.** `./gate.sh` does that. End-to-end on a fully fresh build of all
  147 binaries, same tree, same day: **in place 5h18m (`SUITE_WALL=13570s`) →
  from a clean directory 39m48s (`CHEP_WALL=2s`, `SUITE_WALL=79s`)**. What is
  left is compile and nothing else (clippy 1207s + build 1086s). Granting the
  Terminal developer-tool exemption (`sudo spctl developer-mode
  enable-terminal`, then reopen Terminal — the grant is read when a process
  starts) should help too and is still worth having, but it is no longer the
  only door.
- `./huba …` is a wrapper that builds on first use then execs `rust/target/release/huba`.
