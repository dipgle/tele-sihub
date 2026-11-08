# huba — operating rules

`huba` puts the **Claude CLI sessions running on this Mac** on a phone. One
Telegram chat carries ORDERS (`/session`, `/ask`, `/new`, `/tell`, `/stop`,
`/handover`) and carries back what every session is doing. Nothing here reads
mail, and nothing here spends money unless the owner presses a button.

🔴 **One channel since 2026-08-14** (Hà: *"tạm thời không dùng tfl5 để xem cứ
xóa hết đi"*). A tfl5 chat room and a phone page ran alongside Telegram until
then; the page had been dead for two days without anyone noticing. Gone with it:
`portal.rs`, `live.rs`, `fe/` + 19 `.mjs`, the poll stage (`/ingest`),
`adapters.tfl5`, `trust`, and three unused crates. History:
`memory/ra-soat-2026-08-14.md`.

Read `README.md` for the architecture and the CLI. Read `PLAN.md` for what is
built vs. pending. This file is the rules for working ON huba.

## The one test every design decision must pass

Hà, 2026-08-11, restating the founding intent: *"cli claude cài trên máy tôi,
huba là **cầu kết nối** ra ui để tôi làm việc, điều khiển, giao tiếp phiên"*.

**huba is a BRIDGE, not an owner.** The sessions belong to the CLI on this Mac;
huba carries them to a screen he can reach and carries his hands back. That gives
one test, and it cuts both ways:

- **Anything huba does that has no equivalent at the terminal is a smell.** It
  means huba invented a way of working he cannot see, take over, or reason about.
  The example that forced the rule, and the one that proves it pays: `/new` used
  to create a `--bg` session — headless, no window, no live screen, impossible
  to type into without stopping it first. He would never produce that by sitting
  down at the machine; he would open a window. Three features built on
  2026-08-10 (the activity line, `/btw`, screen reading) simply did not work
  there, and that was the tell. **Fixed 2026-08-11**: `/new` now runs `do script`
  and opens a real Terminal window, matched back to `claude agents` by **tty** —
  the only handle that exists at that moment (the name is auto-assigned, the id
  does not exist yet). `--bg` survives as the fallback when no window can be
  opened, or when `new_in_terminal` is off.
  Closing had to follow, or the bridge is one-way: huba could open a window and
  then refuse to close it. `/stop` on a huba-opened window session types `/exit`,
  waits for Terminal to report the tab idle, then closes the window — Hà's own
  definition of *tắt hẳn*.
- **Anything he can do at the terminal but not from the phone is a gap.** Today:
  watch more than one session's screen at once (`/shot` reads the followed
  session only), scroll back further than the captured window, answer an OS
  dialog. The first two used to be pinned to `portal.rs:96`/`:108`; that file
  went with the page on 2026-08-14, but the GAPS did not — they are properties
  of what a phone can reach, not of the code that failed to reach it.

When a choice is unclear, ask what he would do sitting at the machine, and make
the phone do that — not something cleverer.

## What huba is NOT (2026-08-08)

huba was an inbox: GitHub notifications, project devlogs, email and Telegram fed
one queue, a bounded `claude -p` call triaged every item, and a policy engine
decided send-vs-ask. **That product is deleted** — adapters, triage, policy,
redaction of outbound replies, the outbox, the act stage, the local web console,
the inbox tab on the phone, and every number with a `$` in front of it.

Three measurements decided it, and they are worth keeping because they answer
"can we bring a bit of it back?" with numbers instead of taste:

- **65% of what it carried was CI noise** — 117 of 180 items were GitHub CI
  notifications, and they cost $5.89 of $9.12 total spend.
- **$2.24 of one day's $2.98 triage bill** belonged to the github and devlog
  branches *after they had already been deleted* — the ceiling built to contain
  that spending was mostly the ghost of a dead product, and it ended up standing
  between the owner and his own machine.
- The thing he actually wanted — drive a session from a phone — had run **once**.

Hà, three times in one evening: *"bỏ hết github rồi sao vẫn trần chuồng gì thế"*
· *"sao vẫn nhắc tới tiền vậy, đã bảo xóa hết github rồi mà"* · *"đã bảo xóa
hoàn toàn rồi cơ mà"*. **Before adding anything, ask: does this help him watch
or drive a session from a phone?** If not, it does not belong here.

## Stack

Rust 2021, crate trong `rust/`, hai binary: **`huba`** (CLI) và **`hubad`** (vòng lặp, **đồng bộ có chủ ý**). 🔴 `unsafe` sống ở **ĐÚNG MỘT TỆP**: `cgkeys.rs` (ba luật của nó ở `NOTES-ky-thuat.md §Stack`).
- Deps: `rusqlite` (bundled), `reqwest` (blocking+rustls), `serde`/`serde_json`, `clap`, `regex`, `chrono`, `anyhow`, `base64` — đều trong cache ⇒ `cargo build --offline` chạy được.
- Store: `data/huba.sqlite` (WAL), ba bảng `runs` · `cursors` · `spend` (+ `schema_meta`).
- Test: `cd rust && cargo test --offline --no-fail-fast`; **`./gate.sh`** là cổng đầy đủ (fmt · clippy · build · chạy mọi binary test **từ thư mục sạch** · doctest).
- `./huba …` là wrapper: build lần đầu rồi exec `rust/target/release/huba`.
- 🔴 **Test chậm 2–5 giờ là do `target/debug/deps/`, không phải do bộ test.** Chép binary ra ngoài `deps/` rồi chạy — `gate.sh` làm việc ấy. Số đo và bốn thứ KHÔNG phải nguyên nhân: `NOTES-ky-thuat.md §Stack`.

## Gốc workspace: `~/projects` — và đừng gõ nó vào mã (2026-08-12)

Gốc dời từ `~/Documents/projects` sang **`~/projects`** ngày 2026-08-12 (macOS
gác Documents bằng TCC; quyền đọc chớp tắt giữa phiên dù đã cấp đủ). Đường cũ
còn sống như **symlink**, nên mọi thứ vẫn chạy — đó chính là chỗ nguy hiểm: một
đường dẫn cũ gõ trong mã KHÔNG gãy, nó chỉ đi vòng qua đúng thư mục vừa bị bỏ.

huba tự biết mình ở đâu, và mọi đường dẫn phải bắt nguồn từ đó:
`HUB_CONFIG` (trong plist) → `cfg.hub_home` → `cfg.workspace_root`
→ danh sách dự án, `cwd` của mọi `/new`, và cây mã mà bảng sức khoẻ đem ra so.
Kịch bản `.mjs` thì tự định vị bằng `HERE`/`import.meta.url`, không hỏi `$HOME`.

**huba nay nằm ở `~/projects/huba`, không còn trong `AI/` (2026-08-13.)** Hà:
*"chuyển ra ngoài thư mục gốc đi"*, sau khi đọc nhãn `[AI/huba]` và hỏi *"huba làm
việc đâu liên quan tới ai"*. `AI/` là một ngăn kéo xếp hồ sơ, không phải chủ đề.

Và đừng đếm số bậc để tìm gốc. Dòng cũ là `hub_home/../..` — gõ cứng đúng hình
dạng `<workspace>/AI/huba`, nên lượt chuyển này làm nó tính ra `~/`, im lặng:
danh sách dự án rỗng, `/new` mở nhầm thư mục, bảng sức khoẻ thôi so được cây mã.
Nay `config::find_workspace_root` **đi ngược lên tìm** thư mục có chứa ngăn kéo
đã khai trong `project_roots` — đo được, và đúng ở cả hai chỗ huba từng nằm, nên
lần chuyển sau nữa không phải đụng dòng nào.

📌 Cái giá đã trả, đúng hai hình dạng — cả hai đều **không kêu một tiếng nào**:
- `runtime.rs` so bản cài với `~/Documents/projects/AI/huba/rust`. Mất cây mã ⟹
  hàm trả `None` ⟹ bảng sức khoẻ **thôi cảnh báo "daemon đang chạy mã hôm qua"**,
  tức mất đúng thứ duy nhất phát hiện ra việc quên `install.sh`. Nay bám
  `hub_home` (`runtime::source_tree`), và không tìm thấy cây mã thì **ghi log**.
- `com.dipgle.hubd.plist` vẫn trỏ `HUB_CONFIG` vào đường cũ trong khi bản
  ĐANG CÀI đã sửa tay. Bản cài đúng, repo sai ⟹ **`install.sh` lượt sau cài đè
  lại đường cũ**, lặng lẽ, và `workspace_root` của cả huba đi theo nó.

Ngoại lệ được giữ nguyên văn: **bản chụp màn thật** trong test
(`rust/tests/sessions.rs`, `/btw` 2026-08-11) mang `~/Documents/projects` vì hôm
ấy gốc nằm ở đó. Bằng chứng đã chụp thì không "sửa cho hợp thời".

## Non-negotiables

1. **Nothing on this Mac runs unattended without a wall.** `/new` and `/tell`
   start a `claude` process with the owner's own shell, from one sentence typed
   on a phone. They run behind `sessions::DENIED_TOOLS`: no push/merge/reset, no
   ssh/scp/rsync/sudo/rm/docker/launchctl/`*deploy*`, no WebFetch/WebSearch.
   Writes inside the working tree stay allowed — that is the work. Never widen
   this list to make a feature fit.
2. **Anything huba runs on a live session runs on a FORK, read-only.** `/ask` and
   `/handover` go through `sessions::fork_call`: `--fork-session` so the original
   transcript is untouched, and `FORK_TOOLS` (`Read,Grep,Glob`) so a question
   typed on a phone has no hand to write with. Measured 2026-08-08: `--tools ""`
   breaks on tool-heavy history, and `--disallowedTools` without an allowlist
   loads the full tool schema ($0.2185 for one sentence). The UC's real
   acceptance is the ORIGINAL file: same bytes, same mtime, same `last_activity`.
   A correct answer that added a turn to the live session is a FAILED UC.
3. **No silent failure.** Every error path logs (`rust/src/logging.rs`) *and*
   leaves a row where one exists (`runs.err`). An `Err` mapped to a default
   without a log line is a bug — same rule as a swallowed `catch {}`.
   The shape that broke this rule twelve times is `db.get_cursor(k).ok()` —
   `Ok(None)` means "never set" and `Err` means SQLite itself failed, and
   folding them together made every phone command answer *"no session
   selected"* on a broken database, with nothing in the log. Read cursors
   through **`Db::cursor_or_log`**; the gate lives in one place because twelve
   call sites means the thirteenth forgets. Same reasoning as
   `pending_for_display` (`sessions.rs`) and `telegram::update_sender`: put the
   rule at the source, not at every caller.
4. **Credentials come from env vars only.** The config holds the *name* of the
   env var (`confirm.bot_token_env`, `confirm.chat_id_env`), never the value. A
   missing secret means SKIP-WITH-LOG, not a crash and not a silent no-op —
   `telegram::Inbox::start` says which key name is missing and leaves everything
   else running. (`adapters::Skip`, the error type that used to carry this
   through a run row, went with the poll stage on 2026-08-14; the rule did not.)
   Secrets for the daemon come from `huba.env` (chmod 600), never the plist,
   never the config. Log key NAMES only. The real environment always wins.
5. 🔴 **huba KHÔNG giấu chữ với chủ máy — quét thì ghi, không chặn** (2026-08-16).
   Hà: *"huba là cổng để làm việc từ xa qua tele không cần giấu gì hết, giấu thì
   phải ngồi vào máy để làm vậy thì cần gì huba nữa"*.

   Luật cũ đọc ngược lại (*"nothing about a session leaves this Mac
   unscanned"*), và nó sinh ra từ một lần chạy thật: `huba sessions` in ra một
   phiên có mật khẩu đăng nhập trong lượt cuối. Cái nó bỏ quên là ĐÍCH ĐẾN —
   buồng chat riêng của chính chủ máy, đúng chỗ `/shot` đã gửi nguyên màn hình
   từ 14/08. Nên nó không bảo vệ ai; nó chỉ bắt anh đi bộ về chỗ cái máy.

   Cái giá đo được, **sáu cửa** trong `sessions.rs` cùng một hình dạng
   "quét-rồi-thay-bằng-câu-xin-lỗi": lời cuối phiên (`last_prose` → `None`, tin
   báo cụt), **câu hỏi đang chờ** (`pending_question` xoá sạch lựa chọn ⟹
   `/pick` hết cái để bấm ⟹ phiên đứng kẹt), phần xem trước trong danh sách,
   `/stream`, lý do một lệnh chết, bản bàn giao + câu trả lời `/ask`. Cộng một
   cửa trong `pipeline.rs`: kết quả lệnh ▶️ **không được dán vào phiên**.

   Nay tất cả đi qua **`sessions::note_preview_risk`** — ghi `preview_risk_noted`
   vào log rồi ĐI TIẾP. Một chỗ duy nhất, để lần sau không ai chép lại phép
   "quét rồi giấu". Ba bài kiểm khoá hành vi cũ được **đảo chiều**, không xoá
   (`tests/sessions.rs`), nên "vá lại cho an toàn" là làm đỏ một bài kiểm có
   chủ, không phải sửa một chỗ hở.

   Cái KHÔNG đổi: luật 4 (bí mật của chính huba — token bot, `huba.env` — vẫn chỉ
   log tên khoá, không log giá trị), và `redaction::file_risk` vẫn NÓI RA khi
   một tệp gửi đi có dấu hiệu. Nói ≠ giấu.
6. **Cursors advance only after the orders they cover have run.** A crash must
   re-read, never skip. 🔴 Reduced in scope 2026-08-14: the poll cursors went
   with the poll stage, and Telegram advances its own `offset` inside
   `getUpdates`. What is left under this rule is `focus:session`, the watch
   book, and the project pin.
   🔴 **2026-10-02: Telegram's `offset` is back under it** (Hà, after a dead
   battery: *"sao không có log dự phòng"*). The `offset` used to advance while
   the order lived only in memory, so a power cut / dead `hubad` / restart in
   that window lost the order with no trace. Now every update goes into
   `data/huba.sqlite` table `tin_den` (`fullfsync`) BEFORE the offset moves
   (`telegram::Inbox::nhan_mot`); the command batch marks it `chay` → `xong`;
   boot replays `nhan`, and reports `chay` as *đang chạy dở* WITHOUT re-running
   it (a second `/type` types twice). Anything that kills `hubad` from inside a
   batch must call `tin_den::khep_lo_truoc_khi_tu_khoi_dong` first —
   `runtime::restart_daemon` does, or every `/upgrade` reports itself as
   interrupted. Rows are kept 7 days: that table IS the backup log.
7. **huba takes ORDERS from the owner only.** The gate is `chat_id`
   (`HUB_TELEGRAM_CHAT_ID` in `huba.env`); anyone else typing `/new` is just
   typing text, and the refusal is LOGGED, never silently dropped.

   🔴 **One gate, at the channel** (2026-08-14). There were two: this one, and
   `trust.tfl5_user_tids` checked inside `parse_command`. The second existed
   because a chat ROOM lets anyone in, so being present said nothing about being
   the owner. Telegram has no such shape. Keeping both left the inner gate
   unable to refuse — the only call site had to invent a typist (take `first()`
   of the owner list, compare it against that same list) — except when the list
   was EMPTY, where it refused *everything* in silence. A gate that cannot say
   no, and fails closed only by accident, is not defence in depth; it is a
   second answer to a question that must have one.
   `telegram::update_sender` is where the reading now lives, and the asymmetry
   inside it is the part worth guarding: a BUTTON is gated on who pressed it
   (`callback_query.from.id`), TEXT on which chat it arrived in
   (`message.chat.id`). In a private chat those two numbers are equal, so
   confusing them passes every hand test and only fails once the bot is added to
   a group. Test: `tests/telegram.rs::a_message_from_another_chat_is_not_an_order`.

   **Call them ROUTES** (Hà 2026-08-11: *"tại sao không gọi nó là route?"* — no
   good reason, and it is the truer word). That is exactly the shape: a button
   on the phone hits a named path, a handler runs, an ack comes back; the chat
   room is only the transport. Say "route" in docs and in conversation — the
   code's own `CommandKind` is internal vocabulary, and speaking it at the owner
   is how `/new` ended up being explained to someone who only ever sees buttons.
   One caveat the word must not hide: these routes are **not open**. Only the
   owner's chat can invoke them, so they are authenticated at the human layer,
   not public endpoints.

   Routes: session · new · ask · tell · stop · close · handover · type · key ·
   **pick** · shot · **refresh** · runin · **terminal** · run · doctor · set ·
   upgrade · help · **accounts**.
   ⚠ Dòng trên là bản đọc cho người, KHÔNG phải nguồn: bảng thật là
   `commands::ROUTES`, và nó đã trôi khỏi dòng này ít nhất một lần (`front`,
   `anh`, `clean`, `clear`, `tab`, `web`, `enter`, `right` có trong bảng mà
   không có ở đây). Cần con số đúng thì đọc bảng, đừng đếm dấu chấm giữa.
   (`ingest`/`poll` died 2026-08-14 with the poll stage — it read the chat room,
   and Telegram pushes. `cmd`/`win`/`project` died 2026-08-15: đo trên toàn bộ
   log thì `win` và `project` chưa chạy lần nào từ 26/07, `cmd` đúng một lần và
   lần ấy là chạm menu nên chẳng thực thi gì. Hà chốt *"Bỏ cả 3"*, và về `cmd`:
   *"Không cần cmd vì có terminal là dán vào được"* — ngồi ở máy thì dán thẳng,
   đúng phép thử cầu nối. `win` KHÔI PHỤC ngay sau đó thành `/terminal`: Hà —
   *"cái tên win hơi mơ hồ mà bạn cũng không đưa vào help nên tôi ko hề biết"*.
   Con số "0 lượt dùng" đo SỰ VÔ HÌNH — route để `listed: false` thì không vào
   menu ☰ — chứ không đo sự vô dụng. Đừng lấy nó làm bằng chứng để gỡ lần nữa.)

   **`/pick` vs `/key`, và vì sao phải là hai** (2026-08-13). `AskUserQuestion`
   có thể mang **nhiều câu** trong một bảng, vẽ thành một thanh tab
   (`←  ☒ Vá ACL  ☐ Đăng nhập  ✔ Submit  →`), và bảng ấy **chỉ gửi đi được khi
   không còn ô trống**. `/key <số>` gửi số vào câu ĐANG MỞ — đủ cho bảng một
   câu, và là ngõ cụt cho bảng nhiều câu: các câu sau nằm sau một phím mũi tên
   mà `arrow_verdict` (đúng luật) từ chối gửi khi màn có hộp chọn. Hà bấm nút,
   câu 1 xong, bảng vẫn đứng: *"chọn option xong thì vẫn còn bước nữa nên không
   pass qua được"*. `/pick <câu>.<lựa chọn>` đi được vì nó **không gửi mũi tên
   trần**: cả dãy (mũi tên + số) vào MỘT `do script`, nên chỉ có đúng một dấu
   xuống dòng và nó nằm sau con số. Ba điều nó không làm: không đếm phím để đoán
   vị trí (chủ máy có thể vừa tự bấm — vị trí đọc từ màn mỗi lần), không gõ khi
   `Withheld`/`Blind`, không tin mã trả về (đọc lại bảng, so số ô trống
   trước/sau). Nguồn nội dung vẫn là NHẬT KÝ (`pending_question` nay lấy cả
   `questions[*]`, không chỉ `[0]`); màn chỉ trả lời "đang đứng ở câu nào".

   **Flags, not slots** (Hà 2026-08-12: *"kiến trúc lại lệnh cho hợp lý, ví dụ
   `/new -a acc2 -s dwork`"*). `/new` reads `-a <acc>` and `-s <project>`
   anywhere in the line (`pipeline::split_flags`), and the old positional form
   (`/new <project> @acc <task>`) still parses — it is in the owner's muscle
   memory and in Telegram buttons already sent. Two rules the parser must keep:
   **only KNOWN flags are lifted** out of the text (an unknown `-x` stays in the
   task, because silently eating part of a task produces a session that runs a
   different job than the one typed), and a flag whose value slot holds another
   known flag comes back EMPTY rather than swallowing it.
   An **empty task is allowed on the window path** — that is what the owner does
   at the machine: open a window, type `claude`, then talk. `claude ''` is "an
   empty task", not "no task", so no positional argument is passed at all. The
   `--bg` fallback still refuses it: that path has no window to type into. A route that already answered
   must end its arm with `Some(ack)` — the fall-through that used to exist
   logged "Không tìm thấy decision #0" as the reply for every `/session` and
   `/ask` ever issued.
8. **huba consumes nothing on its own.** There is no triage, so a cycle costs
   nothing; the only calls are `/ask`, `/handover`, `/new`, `/tell` — a person
   pressing a button, at the same cost as typing it in the terminal. They are
   **counted in `spend`, never refused**.
   **What that cost IS, exactly** (settled 2026-08-09, after this file said
   "money" once too often): this Mac is on **Max** — `claude auth status` →
   `subscriptionType: max`. Nothing is invoiced per call; a call spends **plan
   quota**. The `total_cost_usd` the CLI hands back is computed at API LIST
   price, so read every `$` in this repo as a **size gauge** — "how big was that
   call" — never as a bill. The per-call cap still bites, because the CLI
   computes that same number internally regardless of subscription. A daily ceiling on them was built and
   thrown out the same day (2026-08-08). The per-call cap stays and is
   **measured, not guessed**: `sessions::fork_cost_estimate` sizes it from the
   transcript (`USD_PER_MB`, from a real 0.986 MB → $1.72), because a flat cap
   smaller than the load cost means paying for a call that dies anyway.
9. **No money on the screen.** `spend` records silently so the question can be
   ANSWERED if it is ever asked; it stops being asked on every screen. The
   snapshot carries no `owner_spend`, no `owner_budget`, no `cost_days`, no
   `budget`, and `cost_usd` is `#[serde(skip_serializing)]` on `Handover`,
   `Aside` and `Told`. This grew back once already (ceiling → price tag).
   The two guards that asserted ABSENCE (`portal.rs` and `fe-board-uc.mjs`) went
   with the page on 2026-08-14 and were **replaced the same day** by
   `tests/no_money_on_screen.rs`, which stands LOWER than either: it reads the
   serialised shape of the three structs themselves, not one channel's snapshot,
   so it still holds for a sender nobody has written yet. Three things it does
   that the old pair did not — copy them when writing the next absence guard:
   it fills `cost_usd` with a value **no other field could produce** (so the
   assert cannot pass by the number simply being 0.0); it asserts the JSON
   contains something expected BEFORE asserting what it lacks (an empty
   serialisation must not read as "clean"); and it carries a test that
   deliberately feeds the guard a leak to prove the guard can go red. Verified
   by hand as well: dropping `skip_serializing` from `Handover::cost_usd` turns
   it red with exit 101.
10. **`claude` CLI facts that cost a real run to learn.** Bản đầy đủ: `NOTES-ky-thuat.md §Luật 10`. Ba điều cắn ngay: `--bg` xung khắc `-p` ⟹ prompt là POSITIONAL; `--disallowedTools` là VARIADIC ⟹ prompt đứng sau bị ăn mất; `claude stop` nhận id **NGẮN 8 ký tự**. (Subagent nền: `tool_result` về NGAY — "đã phóng", không phải "đã xong".)
11. **huba speaks only on a CHANGE, never on a state.** `watch.rs` compares this
    cycle's snapshot with the previous one and announces only transitions — a
    session that finished its turn, a session that ended. Announcing a *state*
    from a loop that ticks every ~10s is a phone that buzzes forever, and a phone
    that buzzes forever gets muted, taking the messages that mattered with it.
    Three rules hold the line: say it once; say **nothing** on the first round
    after a restart (an empty book means huba just woke up, not that everything
    just changed); and treat *leaving the list* as the main "ended" signal,
    because `claude agents` drops a stopped session within seconds and usually
    never shows it as `dead`. `IDLE_AFTER_SEC` (180s) is a promise about TRUTH,
    not speed: this repo's own `cargo test` runs over two minutes, so a shorter
    window announces "finished" while the session is still working — a wrong
    sentence, not a late one, and it goes to Telegram. Both mouths (the room and
    Telegram) say the SAME sentence (`Change::say`), or nobody can reconcile them
    later. Neither call spends quota.
11b. **A failed measurement is not a fact about the world.** Số đo + bốn hình dạng: `NOTES-ky-thuat.md §Luật 11b`. Tối thiểu: `claude agents` lỗi ⟹ danh sách RỖNG **không** phải "ba phiên chết" (dùng `blind`, giữ sổ); tty là số **DÙNG LẠI** (hỏi snapshot của chính mình trước: `window_taken_over`); `??` **không** phải cửa sổ (`sessions::is_real_tty`); và **máy móc của huba không bao giờ được kêu** (`sessions::is_hub_own_probe`).
12. **The phone page is the only UI.** There is no local console any more. If you
    add a surface, it must go through the same room commands — one path, one set
    of books.
13. **macOS facts that cost a night to learn.** Bản đầy đủ (số đo, lịch sử, tấm bia): `NOTES-ky-thuat.md §Luật 13`. Bốn luật phải nhớ tại chỗ, không cần mở notes:
    `do script` mới là đường gõ vào phiên sống (**KHÔNG** `System Events keystroke` — bị từ chối 1002); nó **LUÔN thêm một newline** và không tắt được; `claude` đọc cả dòng là **PASTE** nên phải gửi Enter **RIÊNG** khi chữ còn trong ô (`keys::still_in_box`); mũi tên vừa DI vừa CHỐT nên chỉ gửi khi `keys::arrow_verdict` chứng minh không có hộp chọn.
    🔴 **huba KHÔNG đổi kích thước cửa sổ đang sống** — cổng `rust/tests/no_window_resizing.rs` sẽ đỏ. Cửa sổ MỚI mở hết cỡ; cửa sổ/tab đang sống thì không đụng, đọc hụt tới đâu khai tới đó.
    Ký + cài bằng `install_update.sh` (bản cài ở `~/Library/Application Support/hub/bin/hubd`), **không** ký tay `target/release`.
14. **A button must have a live session to lead into** (2026-08-12). The
    "👁 Vào phiên" button used to be attached on ONE condition — `id != focused`
    — which asks *"is this the followed session"* and never *"is this session
    still alive"*. So death notices grew a button into a session that no longer
    exists, and Hà read one: *"tại sao 1 phiên đã tắt mà vẫn gắn nút vào phiên
    để làm gì?"* · *"hình như phiên nào bạn cũng mặc định gắn nút vào phiên, quá
    vô lý"*. `pipeline::enter_button` is the single decision now: alive → itself;
    ended → **nothing**, unless another session took its window over, in which
    case the button points at *that* session and the label carries *its* name.
    A button that names the dead is a button that lies.

## MỘT CỬA cho chữ của phiên đi ra Telegram (2026-08-16)

Hà, ba câu trong một buổi: *"lệnh `/shot` hay phản hồi tự động gửi về tele đều
phải qua định dạng trước khi gửi → cái nhận được ở tele phải thao tác được với
các lệnh link của phiên đó"* · *"mọi thứ nhìn thấy ở tele phải đồng nhất"* ·
*"dành cho nội dung lấy từ phiên thôi"*.

Đo được cái hỏng: chỉ `/shot` và tin tự phát đi qua bộ định dạng
(`pipeline::say_session_data` — bảng `SessionData`: lệnh → ▶️/🖥, lựa chọn →
☑, ô nhập → ⏎). Ack của **mọi route khác** đi bằng `send_text` trần, nên cùng
một câu của phiên, cùng một dòng lệnh trong đó, khi thì bấm được khi thì không
— tuỳ nó ra bằng cửa nào, mà người đọc không có cách nào biết trước.

**Cửa: `pipeline::say_from_session`** (và `reply_from_session` cho ack của một
route). Đã nối: `/ask`, `/handover`, `/runin`, nút ▶️ (`RunQuick`), tin báo
lệnh chạy xong (`say_back`). Hai thứ phải giữ khi nối thêm chỗ mới:

- **CHỈ nội dung lấy từ phiên.** Tin thuần của huba (`/help`, danh sách tài
  khoản, *"không mở được cửa sổ"*) không có phiên nào để gắn action; gắn bừa là
  nút trỏ vào chỗ trống.
- **Chỉ ĐỊNH DẠNG cái đang có, không thêm nội dung.** `say_from_session` lọc
  `commands_of` theo `text.contains(...)` trước khi dựng nút, vì `session_layout`
  cố ý nối thêm khu *"Lệnh phiên chạy không được"* cho lệnh nó không thấy trong
  chữ — đúng cho `/shot` (ảnh màn thiếu dòng bị cổng quyền chặn), sai cho mọi
  câu khác: một ack hai dòng sẽ mọc thêm cả danh sách lệnh không ai hỏi.
- `/session` trơn KHÔNG đi qua cửa: tin ấy nói về NHIỀU phiên, gắn theo một
  `sid` là gắn sai phiên cho phần lớn các dòng; nút mỗi hàng (`sess:<id>`) đã
  tự mang phiên của nó.

## Hai nút cho một dòng lệnh — khác nhau ở ĐÍCH của kết quả (2026-08-16)

Hà: *"lệnh chạy phải có 2 nút: 1 là chạy xong lấy kết quả đưa vào phiên, 1 nút
là chạy terminal được kết quả gửi về tele"*.

- **▶️ `run_<mã>`** — huba chạy hộ (`zsh -lc`, thư mục của chính dòng lệnh ấy),
  rồi **dán kết quả vào phiên** (`watch_long_job` → `type_and_send`). Phiên đọc
  được nên nó đi tiếp được.
- **🖥 `term_<mã>`** — mở một cửa sổ Terminal riêng, gõ lệnh vào đó, rồi
  **kết quả về Telegram** (`watch_terminal_job`). Trước 16/08 nút này làm đúng
  nửa việc: mở cửa sổ rồi bỏ đó, tức chỉ dùng được khi chủ máy đang ngồi trước
  máy — đúng lúc anh không cần huba.

`watch_terminal_job` hỏi `keys::tab_busy` mỗi 3 giây (chính Terminal trả lời về
tab của nó; `ps` biến mất trước khi shell kịp in dấu nhắc), rồi đọc màn cửa sổ
ấy và cắt **từ dòng lệnh trở xuống**. Ba ca hỏng đều NÓI RA, không im: mất dấu
cửa sổ, không đọc được màn, và quá `LONG_JOB_MAX_SEC` (lúc ấy nói "huba thôi
canh", vì cửa sổ vẫn còn đó — khác hẳn "lệnh chết").

## Câu xác nhận TRƠN: một dòng sống, không phải một dòng mỗi cú bấm (2026-08-17)

Hà 2026-08-14: *"Có thể đổi cách phản hồi tin đã gửi bằng 1 emoji trực tiếp vào
tin nhắn cho gọn"*; 2026-08-17: *"Khi bấm ở phản hồi nên sửa tin tại phản hồi đó
luôn không cần gửi 1 tin mới"*. Ba đường ra, theo thứ tự:

1. **Thả dấu lên tin của chủ máy** — khi lệnh đến từ một tin chữ anh gõ. Rẻ
   nhất, không chiếm dòng nào.
2. **Sửa câu xác nhận trước** (`telegram::send_ack` + `fold_ack`) — khi câu mới
   GIỐNG HỆT câu đang nằm ở đáy buồng chat: `✓ đã gửi · 🟩 [tfl5] ×3`.
3. **Một dòng mới** — khi câu khác đi, hoặc khi có tin nào khác đã chen vào.

🔴 Hai sự thật phải giữ, mỗi cái đã trả giá một lần:

- **Tiếng vọng `/start` không phải chỗ thả dấu.** Bấm một liên kết trong chữ ⟹
  client gửi `/start <payload>` ⟹ huba XOÁ tin ấy ngay. Bản trước vẫn mang
  `message_id` của nó sang đường trả lời: **73 lần `message to react not found`
  trong ngày 17/08**, mỗi lần một dòng chữ thừa. `telegram::ack_target` nay là
  chỗ duy nhất trả lời câu ấy.
- **Bot KHÔNG thả được dấu lên tin của CHÍNH nó** — đo thật trên buồng chat:
  `setMessageReaction` trả `REACTION_INVALID`. Nên "thả dấu lên tin chứa nút" là
  ngõ cụt; đừng thử lại.

Sổ `ack_live` chỉ đúng chừng nào câu ấy còn ở ĐÁY. Thêm một đường gửi mới thì
phải gọi `forget_ack_live()` trong đó — thiếu một cửa là sửa một dòng người đọc
đã cuộn qua.

## When you change something

- Changed the snapshot shape or a chat verb? `install_update.sh` in the same
  pass — it builds, signs and restarts the launchd job. The running binary is
  the consumer, and a stale one silently overwrites the new shape (twice on
  2026-08-07). `cargo build` alone updates `target/`, which nothing runs.
- A verb that parses must have a handler. A verb with no handler is worse than
  an unknown one: the channel accepts it, nothing happens, nothing says so.
  🔴 The corollary bit on 2026-08-14: a verb whose handler has nothing left to
  DO is the same bug wearing a uniform. `/ingest` still parsed, still ran, still
  answered — with *"disabled in config"*, forever, because the channel it polled
  no longer existed. It was deleted, not left `listed: false`.
- **The acceptance surface shrank on 2026-08-14, and shrank UNEVENLY. Read this
  before claiming anything is verified.** Until then, every deploy ended with a
  picture: `fe-deploy.mjs` ran `fe-shots.mjs` after a successful activate, and
  you opened `ui-shots/after-<version>-*.png` before saying a word. That was
  mechanical on purpose — twice on 2026-08-10 the assertions were green while
  the screen was wrong (a "snapshot is stale" warning **cut off** mid-sentence
  while 7/7 checks passed, because they read `textContent`, which holds the whole
  string even when the screen truncates it; and the "what is it doing" line
  rendering **above** the session name, so the eye read `Brewing…` before knowing
  which session). *An assertion tests what you thought to check; a picture shows
  what you didn't.*
  That whole layer is gone with the page: no `fe-*.mjs`, no screenshots, no
  Playwright, no `HUB_UC_MAX_USD` gate. What is left is 922 Rust tests, and Rust
  tests are exactly the kind that were green both of those times. **So the honest
  bar is now: run the thing in the real Telegram chat and look at the reply.**
  Do not write "verified" off a green `cargo test` — that was never sufficient
  here, and now there is nothing behind it.

## Project layout

```
huba                     wrapper script → rust/target/release/huba
huba.config.json         config (no secrets — only env var NAMES)
gate.sh                 full gate: fmt · clippy · build · run every test binary
                        FROM A CLEAN DIR (see the Tests bullet) · doctest
install_update.sh       build → install a SIGNED hubad where launchd runs it
sign.sh                 re-sign one binary with the stable identity
make-signing-cert.sh    create that identity — ONCE, ever
rust/src/main.rs        CLI: doctor self-install setup init once status sessions ask handover
rust/src/bin/hubad.rs    daemon loop (pid lock, exponential backoff, local alarm)
rust/src/{config,db}.rs config + validation + secret_from_env() · runs/cursors/spend
rust/src/pipeline.rs    one cycle: run the orders that arrived, answer, keep books
rust/src/telegram.rs    THE channel: getUpdates thread, buttons, files, update_sender
rust/src/verbs.rs       pure parser: text → a route. No network, no channel, no owner
rust/src/commands.rs    the route TABLE — one source for /help + setMyCommands
rust/src/sessions.rs    list · stream · fork (/ask, /handover) · background (/new, /tell, /stop)
rust/src/keys.rs        type into a real Terminal window, and read that window back
rust/src/watch.rs       "vừa xong" / "vừa tắt" — so hai lượt ảnh chụp, nói MỘT lần
rust/src/redaction.rs   leak scan, used by session previews
rust/src/setup.rs       `huba setup`: a 127.0.0.1 page that writes huba.env (chmod 600)
rust/src/adapters/      what a command IS (CommandKind, ChannelCommand, Health)
rust/tests/             integration tests + captured real fixture
huba.env(.example)       secrets for launchd runs — chmod 600, gitignored
com.dipgle.hubd.plist   launchd unit (runs the INSTALLED hubad, not target/)
legacy-node/            archived prototype
```

🔴 **Những tệp trên rời khỏi thư mục `deploy/` ngày 2026-08-16**, theo lệnh Hà:
*"xóa deploy đi sửa thành /huba/install_update.sh"*. Lý do không phải thẩm mỹ —
workspace **chặn mọi lệnh Bash NÊU TÊN** một tệp có chữ ấy, kể cả `ls`, `grep`
đọc-thuần và `git mv`. Nên cái tên biến một script chỉ chép một binary vào
`$HOME` và kickstart một launchd job thành thứ không session nào chạy hay bảo
trì nổi, trong khi nó không đụng tới một máy chủ nào. Hàng rào bắn vào cái TÊN,
không vào rủi ro. Đừng đặt lại chữ ấy vào tên tệp trong repo này.
`pipeline::is_self_rebuild` vẫn nhận đường dẫn CŨ — nó nằm trong những tin
Telegram đã gửi đi và trong sổ lệnh gợi ý.

🔴 **Gone 2026-08-14** (in git before `cf20874`): `rust/src/portal.rs` (read-only
snapshot pushed to tfl5 as docs), `rust/src/live.rs` (held-open `/ws/chat`
socket), `rust/src/adapters/tfl5.rs`, `fe/index.html` (the phone page),
`fe-deploy.mjs`, 17 more `fe-*.mjs` Playwright scripts, `console-acl.mjs`, and
`ui-shots/`.
