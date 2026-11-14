# huba — operating rules

`huba` puts the **Claude CLI sessions running on this Mac** on a phone. One
Telegram chat carries ORDERS (`/session`, `/ask`, `/new`, `/tell`, `/stop`,
`/handover`) and carries back what every session is doing. Nothing here reads
mail, and nothing here spends money unless the owner presses a button.

🔴 **One channel since 2026-08-14** (Hà: *"tạm thời không dùng tfl5 để xem cứ
xóa hết đi"*). A tfl5 chat room and a phone page ran alongside Telegram; the page
had been dead for two days without anyone noticing. Gone with it: `portal.rs`,
`live.rs`, `fe/` + 19 `.mjs`, the poll stage (`/ingest`), `adapters.tfl5`,
`trust`, three unused crates. History: `memory/ra-soat-2026-08-14.md` and,
for everything cut before 2026-11-08, the git history of this file.

Read `README.md` for architecture and CLI; `PLAN.md` for built-vs-pending;
`NOTES-ky-thuat.md` for the full story behind any rule marked §. This file is the
rules for working ON huba.

## The one test every design decision must pass

Hà, 2026-08-11: *"cli claude cài trên máy tôi, huba là **cầu kết nối** ra ui để
tôi làm việc, điều khiển, giao tiếp phiên"*.

**huba is a BRIDGE, not an owner.** Sessions belong to the CLI on this Mac; huba
carries them to a screen he can reach and carries his hands back. One test, and
it cuts both ways:

- **Anything huba does that has no equivalent at the terminal is a smell.** It
  invented a way of working he cannot see or take over. The case that forced the
  rule: `/new` used to create a `--bg` session — headless, no window, impossible
  to type into without stopping it; three features built on 2026-08-10 did not
  work there. **Fixed 2026-08-11**: `/new` runs `do script` and opens a real
  Terminal window, matched back to `claude agents` by **tty** (the only handle
  that exists at that moment). `/stop` types `/exit`, waits for the tab to go
  idle, then closes the window — Hà's definition of *tắt hẳn*. `--bg` survives
  only as fallback when no window can be opened or `new_in_terminal` is off.
- **Anything he can do at the terminal but not from the phone is a gap.** Today:
  watch more than one session's screen at once (`/shot` reads the followed
  session only), scroll back further than the captured window, answer an OS
  dialog. GAPS are properties of what a phone can reach — not of the code that
  once failed to reach it.

When a choice is unclear, ask what he would do sitting at the machine, and make
the phone do that — not something cleverer.

## What huba is NOT (2026-08-08)

huba was an inbox: GitHub notifications, devlogs, email and Telegram fed one
queue, a bounded `claude -p` triaged each item, a policy engine decided
send-vs-ask. **That product is deleted** — adapters, triage, policy, redaction of
outbound replies, the outbox, the act stage, the local web console, the inbox
tab, every number with a `$` in front of it.

Three measurements keep it deleted: 65% of what it carried was CI noise (117 of
180 items, $5.89 of $9.12 spend); $2.24 of one day's $2.98 bill belonged to
branches already deleted; the thing he wanted — drive a session from a phone —
had run **once**. **Before adding anything ask: does this help him watch or
drive a session from a phone?** If not, it does not belong here.

## Stack

Rust 2021, crate in `rust/`, two binaries: **`huba`** (CLI) and **`hubad`**
(loop, deliberately synchronous). 🔴 `unsafe` lives in **EXACTLY ONE FILE**:
`cgkeys.rs` (its three rules: `NOTES-ky-thuat.md §Stack`).
- Deps: `rusqlite` (bundled), `reqwest` (blocking+rustls), `serde`/`serde_json`,
  `clap`, `regex`, `chrono`, `anyhow`, `base64` — all cached ⇒ `cargo build --offline` works.
- Store: `data/huba.sqlite` (WAL), tables `runs` · `cursors` · `spend` · `tin_den` (+ `schema_meta`).
- Test: `cd rust && cargo test --offline --no-fail-fast`; **`./gate.sh`** is the
  full gate (fmt · clippy · build · every test binary **from a clean dir** · doctest).
- `./huba …` is a wrapper: builds on first use, then execs `rust/target/release/huba`.
- 🔴 **A 2–5 hour test run is `target/debug/deps/`, not the test suite.** Copy the
  binary out of `deps/` before running — `gate.sh` does that. Measurements and the
  four things that are NOT the cause: `NOTES-ky-thuat.md §Stack`.

## Gốc workspace: `~/projects` — và đừng gõ nó vào mã (2026-08-12)

Root moved from `~/Documents/projects` to **`~/projects`** (macOS gates Documents
with TCC; the permission flickers mid-session). The old path lives on as a
**symlink**, which is the danger: a stale path in code does NOT break — it
silently routes through the directory that was abandoned.

huba knows where it is, and every path must start from there: `HUB_CONFIG` (in
the plist) → `cfg.hub_home` → `cfg.workspace_root` → project list, `cwd` of every
`/new`, the tree the health table compares. `.mjs` scripts locate themselves with
`HERE`/`import.meta.url`, never `$HOME`.
**huba lives at `~/projects/huba`, not in `AI/` (2026-08-13)** — `AI/` is a filing
drawer, not a topic. Never count directory levels to find the root: the old
`hub_home/../..` hardcoded `<workspace>/AI/huba`, so the move silently produced
`~/`, an empty project list and a wrong `/new` cwd. `config::find_workspace_root`
now walks UP looking for a declared `project_roots` entry.

📌 The price, both shapes **silent**: `runtime.rs` compared the install against
`~/Documents/projects/AI/huba/rust` — losing the tree returned `None` and the
health table **stopped warning** "daemon is running yesterday's code". Now it
follows `hub_home` (`runtime::source_tree`) and **logs** when the tree is missing.
`com.dipgle.hubd.plist` still pointed `HUB_CONFIG` at the old path while the
INSTALLED copy was hand-fixed — repo wrong, so the next `install.sh` re-installed
the old path quietly. Preserved verbatim: the real screen capture in
`rust/tests/sessions.rs` (`/btw`, 2026-08-11) — captured evidence is not "fixed
to match today".

## Non-negotiables

1. **Nothing on this Mac runs unattended without a wall.** `/new` and `/tell`
   start `claude` with the owner's own shell from one phone sentence, behind
   `sessions::DENIED_TOOLS`: no push/merge/reset, no ssh/scp/rsync/sudo/rm/
   docker/launchctl/`*deploy*`, no WebFetch/WebSearch. Writes inside the tree stay
   allowed — that is the work. Never widen this list to make a feature fit.
2. **Anything huba runs on a live session runs on a FORK, read-only.** `/ask` and
   `/handover` go through `sessions::fork_call`: `--fork-session` (original
   transcript untouched) + `FORK_TOOLS` (`Read,Grep,Glob`) — a question typed on a
   phone has no hand to write with. Measured 2026-08-08: `--tools ""` breaks on
   tool-heavy history, and `--disallowedTools` without an allowlist loads the full
   tool schema ($0.2185 for one sentence). The UC's real acceptance is the
   ORIGINAL file: same bytes, same mtime, same `last_activity`. A correct answer
   that added a turn to the live session is a FAILED UC.
3. **No silent failure.** Every error path logs (`rust/src/logging.rs`) *and*
   leaves a row where one exists (`runs.err`). An `Err` mapped to a default
   without a log line is a bug — same rule as a swallowed `catch {}`.
   The shape that broke this twelve times: `db.get_cursor(k).ok()` folds "never
   set" and "SQLite failed" together, so every phone command answered *"no session
   selected"* on a broken database with nothing in the log. Read cursors through
   **`Db::cursor_or_log`**. Same reasoning for `pending_for_display`
   (`sessions.rs`) and `telegram::update_sender`: put the rule at the source, not
   at every caller.
4. **Credentials come from env vars only.** The config holds the *name*
   (`confirm.bot_token_env`, `confirm.chat_id_env`), never the value. A missing
   secret is SKIP-WITH-LOG — `telegram::Inbox::start` names the missing key and
   keeps everything else running. Daemon secrets come from `huba.env`
   (chmod 600), never the plist, never the config. Log key NAMES only. The real
   environment always wins.
5. 🔴 **huba KHÔNG giấu chữ với chủ máy — quét thì ghi, không chặn** (2026-08-16).
   Hà: *"huba là cổng để làm việc từ xa qua tele không cần giấu gì hết, giấu thì
   phải ngồi vào máy để làm vậy thì cần gì huba nữa"*. The destination is the
   owner's own private chat — the same place `/shot` has sent whole screens since
   14/08 — so hiding protects nobody; it only walks him back to the machine.
   The measured cost was **six doors** in `sessions.rs` with one shape
   "scan-then-replace-with-an-apology": `last_prose`, `pending_question` (wiping
   the options left `/pick` nothing to press), the session-list preview,
   `/stream`, a died command's reason, the handover body + `/ask` answer — plus
   one in `pipeline.rs` (a command's ▶️ result was **not** pasted into the
   session). All of them now go through **`sessions::note_preview_risk`**: log
   `preview_risk_noted`, then CARRY ON. One place, so nobody re-copies the shape.
   Three tests locking the old behaviour were **reversed, not deleted**
   (`tests/sessions.rs`) — "fixing it back for safety" turns a deliberate test red.
   What does NOT change: rule 4 (huba's own secrets — bot token, `huba.env` —
   still log key names only), and `redaction::file_risk` still SAYS so when a file
   being sent looks risky. Nói ≠ giấu.
6. **Cursors advance only after the orders they cover have run.** A crash must
   re-read, never skip. 🔴 **Telegram's `offset` is under this rule since
   2026-10-02** (Hà, after a dead battery: *"sao không có log dự phòng"*): every
   update goes into `data/huba.sqlite` table `tin_den` (`fullfsync`) BEFORE the
   offset moves (`telegram::Inbox::nhan_mot`); the batch marks it `chay` → `xong`;
   boot replays `nhan` and reports `chay` as *đang chạy dở* WITHOUT re-running it
   (a second `/type` types twice). Anything that kills `hubad` from inside a batch
   must call `tin_den::khep_lo_truoc_khi_tu_khoi_dong` first —
   `runtime::restart_daemon` does, or every `/upgrade` reports itself interrupted.
   Rows are kept 7 days: that table IS the backup log. (What is left of the old
   poll cursors: `focus:session`, the watch book, the project pin.)
7. **huba takes ORDERS from the owner only.** The gate is `chat_id`
   (`HUB_TELEGRAM_CHAT_ID` in `huba.env`); anyone else typing `/new` is just
   typing text, and the refusal is LOGGED, never silently dropped.

   🔴 **One gate, at the channel** (2026-08-14). `trust.tfl5_user_tids` was a
   second gate that could no longer refuse — its only call site had to invent a
   typist — except when its list was EMPTY, where it refused everything in
   silence. A gate that cannot say no is not defence in depth; it is a second
   answer to a question that must have one. `telegram::update_sender` is where the
   reading lives, and the asymmetry is what to guard: a BUTTON is gated on who
   pressed it (`callback_query.from.id`), TEXT on which chat it arrived in
   (`message.chat.id`). In a private chat those numbers are equal, so confusing
   them passes every hand test and only fails once the bot joins a group.
   Test: `tests/telegram.rs::a_message_from_another_chat_is_not_an_order`.

   **Call them ROUTES** (Hà 2026-08-11: *"tại sao không gọi nó là route?"*). A
   button on the phone hits a named path, a handler runs, an ack comes back; the
   chat room is only transport. Say "route" in docs and conversation — the code's
   `CommandKind` is internal vocabulary. These routes are **not open**: only the
   owner's chat can invoke them, so they are authenticated at the human layer.

   Routes: session · new · ask · tell · stop · close · handover · type · key ·
   **pick** · shot · **refresh** · runin · **terminal** · run · doctor · set ·
   upgrade · help · **accounts**.
   ⚠ That line is for humans, NOT the source: the real table is
   `commands::ROUTES`, and it has drifted at least once (`front`, `anh`, `clean`,
   `clear`, `tab`, `web`, `enter`, `right` are in the table and not here). Need a
   count, read the table — do not count dots. (`ingest`/`poll` died 2026-08-14
   with the poll stage. `cmd`/`win`/`project` died 2026-08-15: `win` and `project`
   had run zero times since 26/07, `cmd` once and that was a menu tap. `win`
   returned immediately as `/terminal` — Hà: *"cái tên win hơi mơ hồ mà bạn cũng
   không đưa vào help nên tôi ko hề biết"*. The zero-count measured
   INVISIBILITY — `listed: false` keeps a route out of the ☰ menu — not
   uselessness. Never reuse it as evidence to delete again.)

   **`/pick` vs `/key`, and why both exist** (2026-08-13). `AskUserQuestion` can
   carry **several questions** in one table drawn as a tab bar
   (`←  ☒ Vá ACL  ☐ Đăng nhập  ✔ Submit  →`), and that table only sends when no
   cell is empty. `/key <số>` sends a digit into the OPEN question — enough for
   one question, a dead end for many: later questions sit behind an arrow key
   that `arrow_verdict` (correctly) refuses to send while a choice box is up. Hà:
   *"chọn option xong thì vẫn còn bước nữa nên không pass qua được"*.
   `/pick <câu>.<lựa chọn>` works because it does **not** send a bare arrow: the
   whole sequence (arrows + digit) goes into ONE `do script`, so there is exactly
   one newline and it follows the digit. Three things it does not do: count keys
   to guess position (the owner may have tapped — position is read from the
   screen each time), type while `Withheld`/`Blind`, or trust the return code
   (re-read the table, compare empty-cell counts before/after). Content still
   comes from the TRANSCRIPT (`pending_question` reads all `questions[*]`, not
   just `[0]`); the screen only answers "which question is open".

   **Flags, not slots** (Hà 2026-08-12: *"kiến trúc lại lệnh cho hợp lý, ví dụ
   `/new -a acc2 -s dwork`"*). `/new` reads `-a <acc>` and `-s <project>` anywhere
   (`pipeline::split_flags`); the old positional form still parses — it is in his
   muscle memory and in buttons already sent. Two parser rules: **only KNOWN flags
   are lifted** (an unknown `-x` stays in the task — silently eating part of a
   task runs a different job than the one typed), and a flag whose value slot
   holds another known flag comes back EMPTY rather than swallowing it.
   An **empty task is allowed on the window path** — open a window, type `claude`,
   then talk; `claude ''` is "an empty task", so no positional arg is passed. The
   `--bg` fallback still refuses it (no window to type into). A route that already
   answered must end its arm with `Some(ack)` — the old fall-through logged
   "Không tìm thấy decision #0" as the reply for every `/session` and `/ask`.
8. **huba consumes nothing on its own.** No triage, so a cycle costs nothing; the
   only calls are `/ask`, `/handover`, `/new`, `/tell` — a person pressing a
   button, at terminal price. They are **counted in `spend`, never refused**.
   **What that cost IS** (settled 2026-08-09): this Mac is on **Max**, so nothing
   is invoiced per call — a call spends **plan quota**. `total_cost_usd` is
   computed at API LIST price: read every `$` here as a **size gauge**, never a
   bill. The per-call cap still bites (the CLI computes that number regardless),
   and it is **measured, not guessed**: `sessions::fork_cost_estimate` sizes it
   from the transcript (`USD_PER_MB`, from a real 0.986 MB → $1.72). A flat cap
   below the load cost pays for a call that dies anyway.
9. **No money on the screen.** `spend` records silently so the question can be
   ANSWERED if asked; it stops being asked on every screen. The snapshot carries
   no `owner_spend`, no `owner_budget`, no `cost_days`, no `budget`, and
   `cost_usd` is `#[serde(skip_serializing)]` on `Handover`, `Aside` and `Told`.
   Guard: `tests/no_money_on_screen.rs` reads the serialised shape of the three
   structs, not one channel's snapshot, so it holds for senders not yet written.
   Three properties to copy for the next absence guard: fill `cost_usd` with a
   value **no other field could produce** (so it cannot pass by being 0.0); assert
   the JSON contains something expected BEFORE asserting what it lacks (empty
   serialisation must not read as "clean"); carry a test that feeds the guard a
   leak to prove it can go red.
10. **`claude` CLI facts that cost a real run to learn.** Full: `NOTES-ky-thuat.md §Luật 10`.
    `--bg` conflicts with `-p` (prompt goes POSITIONAL); `--disallowedTools` is
    VARIADIC (it eats a prompt placed after it); `claude stop` takes the **SHORT
    8-char id**. (Background subagent: `tool_result` arrives IMMEDIATELY — it
    means "launched", not "done".)
11. **huba speaks only on a CHANGE, never on a state.** `watch.rs` compares this
    cycle's snapshot with the last and announces only transitions — a session that
    finished its turn, one that ended. Announcing a state from a ~10s loop is a
    phone that buzzes forever, and a phone that buzzes forever gets muted, taking
    the messages that mattered with it. Three rules: say it once; say **nothing**
    on the first round after a restart (an empty book means huba just woke up);
    treat *leaving the list* as the main "ended" signal (`claude agents` drops a
    stopped session in seconds and usually never shows it `dead`).
    `IDLE_AFTER_SEC` (180s) is a promise about TRUTH, not speed — this repo's
    `cargo test` runs over two minutes, so a shorter window announces "finished"
    while the session is still working: a wrong sentence, not a late one, and it
    goes to Telegram. Both mouths say the SAME sentence (`Change::say`).
11b. **A failed measurement is not a fact about the world.** Full: `NOTES-ky-thuat.md §Luật 11b`.
    `claude agents` failing means an EMPTY list, **not** "three sessions died"
    (use `blind`, keep the book); tty numbers are **REUSED** (ask your own snapshot
    first: `window_taken_over`); `??` is **not** a window (`sessions::is_real_tty`);
    and **huba's own machinery must never ring** (`sessions::is_hub_own_probe`).
12. **The phone page is the only UI.** There is no local console. A new surface
    must go through the same room commands — one path, one set of books.
13. **macOS facts that cost a night.** Full: `NOTES-ky-thuat.md §Luật 13`. Four
    to remember in place: `do script` is the only way to type into a live session
    (**NOT** `System Events keystroke` — refused 1002); it **ALWAYS appends a
    newline** and cannot be turned off; `claude` reads the whole line as a **PASTE**
    so Enter must be sent **SEPARATELY** while text is still in the box
    (`keys::still_in_box`); arrows both MOVE and COMMIT, so only send them when
    `keys::arrow_verdict` proves no choice box is up.
    🔴 **huba does NOT resize a live window** — `rust/tests/no_window_resizing.rs`
    goes red. NEW windows open full size; live windows/tabs are never touched, and
    whatever the capture misses is reported as missed.
    Sign + install with `install_update.sh` (installed copy:
    `~/Library/Application Support/hub/bin/hubd`), **never** sign
    `target/release` by hand.
14. **A button must have a live session to lead into** (2026-08-12). The
    "👁 Vào phiên" button used to be attached on ONE condition — `id != focused` —
    which asks "is this the followed session" and never "is it still alive". Hà
    read one: *"tại sao 1 phiên đã tắt mà vẫn gắn nút vào phiên để làm gì?"* ·
    *"hình như phiên nào bạn cũng mặc định gắn nút vào phiên, quá vô lý"*.
    `pipeline::enter_button` is the single decision: alive → itself; ended →
    **nothing**, unless another session took the window over, in which case the
    button points at *that* session and the label carries *its* name. A button
    that names the dead is a button that lies.

## MỘT CỬA cho chữ của phiên đi ra Telegram (2026-08-16)

Hà: *"lệnh `/shot` hay phản hồi tự động gửi về tele đều phải qua định dạng trước
khi gửi → cái nhận được ở tele phải thao tác được với các lệnh link của phiên đó"*
· *"mọi thứ nhìn thấy ở tele phải đồng nhất"* · *"dành cho nội dung lấy từ phiên
thôi"*. The measured break: only `/shot` and spontaneous notices went through the
formatter (`pipeline::say_session_data`, table `SessionData`: command → ▶️/🖥,
choice → ☑, input box → ⏎); every other route's ack used bare `send_text`, so the
same session sentence was tappable or not depending on which door it left by.

**Door: `pipeline::say_from_session`** (and `reply_from_session` for a route ack).
Wired: `/ask`, `/handover`, `/runin`, the ▶️ button (`RunQuick`), `say_back`. Two
things to keep when wiring more:
- **ONLY content taken from a session.** huba's own lines (`/help`, account list,
  *"không mở được cửa sổ"*) have no session to attach actions to; a random
  button points into empty space.
- **Only FORMAT what exists, never ADD content.** `say_from_session` filters
  `commands_of` by `text.contains(...)` before building buttons, because
  `session_layout` deliberately appends a *"Lệnh phiên chạy không được"* section
  for commands it did not find — right for `/shot` (the capture misses a line
  blocked by the permission gate), wrong everywhere else: a two-line ack would
  grow a list of commands nobody asked for.
- plain `/session` does NOT go through the door: that message is about MANY
  sessions, and attaching one `sid` mislabels most of its lines; each row's own
  button (`sess:<id>`) already carries its session.

## Hai nút cho một dòng lệnh — khác nhau ở ĐÍCH của kết quả (2026-08-16)

Hà: *"lệnh chạy phải có 2 nút: 1 là chạy xong lấy kết quả đưa vào phiên, 1 nút
là chạy terminal được kết quả gửi về tele"*.
- **▶️ `run_<mã>`** — huba runs it (`zsh -lc`, in that line's own directory), then
  **pastes the result into the session** (`watch_long_job` → `type_and_send`). The
  session reads it and can continue.
- **🖥 `term_<mã>`** — opens its own Terminal window, types the command there, and
  the **result goes to Telegram** (`watch_terminal_job`). Before 16/08 this did
  half the job: opened the window and walked away — usable only when the owner was
  sitting at the machine, exactly when he does not need huba.
`watch_terminal_job` asks `keys::tab_busy` every 3s (Terminal answers about its own
tab; `ps` vanishes before the shell prints a prompt), then reads that window and
cuts **from the command line down**. Three failure cases all SPEAK: lost window
handle, unreadable screen, and over `LONG_JOB_MAX_SEC` — which says "huba thôi
canh" because the window still exists, unlike "the command died".

## Câu xác nhận TRƠN: một dòng sống, không phải một dòng mỗi cú bấm (2026-08-17)

Hà 2026-08-14: *"Có thể đổi cách phản hồi tin đã gửi bằng 1 emoji trực tiếp vào
tin nhắn cho gọn"*; 2026-08-17: *"Khi bấm ở phản hồi nên sửa tin tại phản hồi đó
luôn không cần gửi 1 tin mới"*. Three exits, in order: 1. **react to the owner's
own message** when the order came from text he typed. 2. **edit the previous
ack** (`telegram::send_ack` + `fold_ack`) when the new sentence is IDENTICAL to
the one at the bottom of the chat. 3. **a new line** when it differs or something
intervened.

🔴 Two facts, each paid for once:
- **A `/start` echo is not a react target.** Tapping an inline link makes the
  client send `/start <payload>`; huba DELETES that message at once. The previous
  build still carried its `message_id` into the reply path: **73
  `message to react not found` in one day (17/08)**. `telegram::ack_target` is now
  the single place that answers this.
- **A bot CANNOT react to its OWN message** — measured in the real chat:
  `setMessageReaction` returns `REACTION_INVALID`. "React to the button message"
  is a dead end; do not retry it.

The `ack_live` book is only valid while that sentence is at the BOTTOM. Every new
send path must call `forget_ack_live()`, or it edits a line the reader scrolled past.

## Một cổng bảo mật dùng chung (2026-11-11)

- Luật nằm ở `~/projects/scripts/cong_bao_mat.py` + `bao-mat-policy.json`; adapter **không** chép luật.
- Claude: `guard-write-path.py` (`PreToolUse`) gọi cổng; `--disallowedTools` đọc `sessions::denied_tools()` từ cùng policy.
- opencode: plugin `scripts/cong-bao-mat-plugin` gọi cổng ở `tool.execute.before` / `chat.message` / `permission.ask`; `accounts::opencode_config_content` tự chèn plugin cho mọi acc `cli:"opencode"`.
- Nâng cấp một lần: sửa policy/cổng rồi `python3 scripts/cong_bao_mat.py --tu-kiem`. `FALLBACK_DENIED_TOOLS` chỉ dùng khi policy biến mất; bài test khoá nó khớp policy.

## When you change something

- **Adding an account or a new model source / CLI?** Walk `docs/kiem-tra-them-tai-khoan-va-nguon-model.md`
  first — every account takes the SAME input (Quang 14/11); a CLI's write tools must be measured, not assumed
  (codex `apply_patch` slipped the gate until 14/11).

- Changed the snapshot shape or a chat verb? `install_update.sh` in the same
  pass — it builds, signs, restarts the launchd job. The running binary is the
  consumer, and a stale one silently overwrites the new shape (twice on
  2026-08-07). `cargo build` alone updates `target/`, which nothing runs.
- A verb that parses must have a handler. A verb with no handler is worse than an
  unknown one: the channel accepts it, nothing happens, nothing says so.
  🔴 Corollary: a verb whose handler has nothing left to DO is the same bug in
  uniform — `/ingest` still parsed, ran and answered *"disabled in config"*
  forever, because the channel it polled no longer existed. Delete it; do not
  leave it `listed: false`.
- **The acceptance surface shrank on 2026-08-14, and shrank UNEVENLY.** Until
  then every deploy ended with a picture: `fe-deploy.mjs` ran `fe-shots.mjs`, and
  you opened `ui-shots/after-<version>-*.png` before saying a word. That was
  mechanical on purpose — twice on 2026-08-10 assertions were green while the
  screen was wrong (a "snapshot is stale" warning **cut off** mid-sentence while
  7/7 checks passed, because they read `textContent`; and the "what is it doing"
  line rendering **above** the session name). *An assertion tests what you thought
  to check; a picture shows what you didn't.* That layer is gone with the page: no
  `fe-*.mjs`, no screenshots, no Playwright. What is left is Rust tests — exactly
  the kind that were green both of those times. **The honest bar is now: run the
  thing in the real Telegram chat and look at the reply.** Never write "verified"
  off a green `cargo test`.

## Project layout

```
huba                     wrapper script → rust/target/release/huba
huba.config.json         config (no secrets — only env var NAMES)
gate.sh                  full gate: fmt · clippy · build · every test binary
                         FROM A CLEAN DIR (see the Tests bullet) · doctest
install_update.sh        build → install a SIGNED hubad where launchd runs it
sign.sh                  re-sign one binary with the stable identity
make-signing-cert.sh     create that identity — ONCE, ever
rust/src/main.rs         CLI: doctor self-install setup init once status sessions ask handover
rust/src/bin/hubad.rs    daemon loop (pid lock, exponential backoff, local alarm)
rust/src/{config,db}.rs  config + validation + secret_from_env() · runs/cursors/spend
rust/src/pipeline.rs     one cycle: run arrived orders, answer, keep books
rust/src/telegram.rs     THE channel: getUpdates thread, buttons, files, update_sender
rust/src/verbs.rs        pure parser: text → a route. No network, no channel, no owner
rust/src/commands.rs     the route TABLE — one source for /help + setMyCommands
rust/src/sessions.rs     list · stream · fork (/ask, /handover) · background (/new, /tell, /stop)
rust/src/keys.rs         type into a real Terminal window, and read that window back
rust/src/watch.rs        "vừa xong" / "vừa tắt" — compare two snapshots, say it ONCE
rust/src/redaction.rs    leak scan, used by session previews
rust/src/setup.rs        `huba setup`: a 127.0.0.1 page that writes huba.env (chmod 600)
rust/src/adapters/       what a command IS (CommandKind, ChannelCommand, Health)
rust/tests/              integration tests + captured real fixture
huba.env(.example)       secrets for launchd runs — chmod 600, gitignored
com.dipgle.hubd.plist    launchd unit (runs the INSTALLED hubad, not target/)
legacy-node/             archived prototype
```

🔴 **Those files left `deploy/` on 2026-08-16**, on Hà's order: *"xóa deploy đi sửa
thành /huba/install_update.sh"*. Not aesthetics — the workspace **blocks every Bash
command NAMING** a file with that word, including `ls` and `grep` and `git mv`. The
name turned a script that copies one binary into `$HOME` and kickstarts a launchd
job into something no session can run or maintain. The barrier fires at the NAME,
not the risk. Never put that word back in a filename here.
`pipeline::is_self_rebuild` still accepts the OLD path — it lives in Telegram
messages already sent and in the suggestion book.

🔴 **Gone 2026-08-14 in the OLD repo — physically removed HERE on 2026-11-09.**
`rust/src/portal.rs` (snapshot pushed to tfl5 as docs), `rust/src/live.rs`
(held-open `/ws/chat`), `rust/src/adapters/tfl5.rs`, `fe/index.html`,
`fe-deploy.mjs`, 17 more `fe-*.mjs` Playwright scripts, `console-acl.mjs`.
The re-import that created this repo's first commit (`507716c`, 2026-10-29,
289 files) came from a tree that still carried them: 23 tracked files were
deleted on 2026-11-09 after measuring that **nothing executes them** (no
`package.json` script, no import from `web.mjs`, no `mod` for `portal`/`live`
in `lib.rs`). `cf20874` is NOT an ancestor of HEAD — it lives only in the
archived `luu-tru` history, and its tree still had all 21 fe/acl files, so the
old sentence described a history this repo never had. `ui-shots/` (35 MB, 226
PNGs) is **gitignored and untracked** ⇒ left on disk: deleting it would not be
recoverable.
