# tele-sihub (huba) — watch and drive the Claude CLI sessions on your Mac from your phone

*Tiếng Việt: [README.vi.md](README.vi.md)*

On a working Mac there are often a dozen `claude` sessions running in Terminal
windows. `huba` (the program's name; this repository is its public home) puts
them into **one Telegram chat**: see what each session is doing, open a new one,
type straight into a running session, ask a side question without disturbing its
work, stop it, or close it out with a handover so you can continue at the desk.

Traffic goes both ways, and **the Mac is always the one calling out** — no port is
opened on the machine, and no address points back to it:

```
  phone ──► Telegram ──► hubad (long-poll getUpdates) ──► claude CLI on the Mac
    ▲                              │
    └──── notifications + buttons ◄┘   (only on a CHANGE, never on a state)
```

> **Language note.** This README is in English. Code comments, the other docs
> (`CLAUDE.md`, `PLAN.md`, `UC.md`, …) and the messages huba sends to Telegram are
> in **Vietnamese** — it was built for a Vietnamese-speaking owner.

**Platform:** macOS (Terminal.app + AppleScript). A Windows port exists but has
not yet run on a real Windows machine — see below.

## All you need is your phone

With huba running, **a phone with Telegram is enough to get the work done.** Leave
the Mac at home and go out: you can still see what every session did, answer its
questions with a tap, give it the next task, open a new session for a new job, and
have the files it produces sent to you in the chat.

**No more waiting before you can leave.** Carrying a laptop means closing it, and
closing it means waiting for every running session to finish first — or killing
work halfway. With huba the Mac simply stays at home and keeps working; you walk
out the door whenever you like and pick the sessions up from your phone.

Set it up once and forget it: keep the Mac **plugged in, online and awake**
(turn off automatic sleep, or use `caffeinate`) and **logged in**. Locking the
screen is fine — typing into sessions and `/shot` go through Terminal and keep
working on the lock screen; only `/anh` (screenshots) and `/web` (driving Chrome)
need it unlocked.

## Telegram is the main channel

Telegram is a **full command channel**: every verb in the tables below can be
typed there, and most day-to-day work needs no typing at all — huba attaches the
buttons.

- **Plain text (no leading `/`) goes straight into the session you are following.**
  Tapping a session in the list is like sitting down in it; after that, whatever
  you type, it receives. If you are not following any session, huba **does not
  guess** one to type into — typing into the wrong window cannot be undone.
- **Buttons instead of remembering syntax:** the session list (one button per
  session), each option of a session that stopped to ask a question,
  `📄 Xem đầy đủ` (view in full, opens that session), `📎` to fetch a file the session
  just mentioned, `👁 Vào phiên` (enter session), `⏎ Gửi` (send).
- **The owner gate:** `HUB_TELEGRAM_CHAT_ID` in `huba.env`. Messages from any other
  chat are **logged and dropped**, never silently ignored. This is the **only**
  owner gate: a second one used to exist for a chat room that has since been
  removed, and once the room was gone it could no longer refuse anyone — except
  when its list was empty, where it silently swallowed every command.
- **It speaks only on a change.** `watch.rs` compares two snapshots and announces
  exactly once: a session finished, stopped to ask, stopped on an error, or ended.
  Announcing *state* from a 10-second loop is a phone that buzzes forever — and a
  phone like that gets muted, taking the messages that mattered with it.
- **Sessions running inside VS Code / Cursor are not listed.** Typing into a
  session goes through Terminal.app, addressed by tty, and an editor session has no
  Terminal window — a row you can watch but not drive is not a bridge, just one
  more name to confuse. The number of skipped sessions is still counted and reported.

> **History.** huba once ran a second channel (a chat room plus a phone web page)
> alongside Telegram; it was removed on 2026-08-14 after the page turned out to
> have been dead for two days without anyone noticing — the price of two
> interfaces for a one-user product.
>
> Before that, huba was an **inbox** (GitHub · devlogs · email · Telegram → triage
> with `claude -p` → approve/send). That branch was **deleted on 2026-08-08**: 65%
> of what it carried was CI notifications, which ate $5.89 of $9.12 total spend,
> while the thing actually needed — driving sessions remotely — had run once.
> The plain consequence: **huba consumes no quota on its own**; a `claude` call
> happens only when you press a button, at the same cost as typing it in a terminal.

## Up and running in 4 commands

```bash
cd tele-sihub          # your clone
./huba setup           # config page on 127.0.0.1 → writes huba.env (chmod 600)
./huba doctor          # real checks: claude CLI, Telegram, project folders
./huba sessions        # every live claude session, across all accounts
./huba once            # one cycle: run the orders that arrived → books → notifications
```

Secrets (the Telegram bot token + chat id) live in `huba.env` (chmod 600); only the
**names** of the environment variables live in `huba.config.json`. See
`huba.env.example`.

`huba setup` listens on `127.0.0.1` only, is entered with a one-time ticket in the
URL, and **never shows a stored value again** — it only says whether a key is
*set* or *not set*. Reading a 600 file and pumping it back out over HTTP would
build exactly the leak the 600 permission exists to prevent.

Setting up the workspace from scratch (folders, workspace root, the three
questions people usually ask): `ARCHITECTURE.md`.

## Start with the machine (launchd)

```bash
./huba self-install                                  # build → sign → install → restart

# The plist is a TEMPLATE: three placeholders must be replaced with absolute
# paths for this machine (launchd does not understand `~` or `$HOME`, and a public
# repository carries nobody's machine paths)
sed -e "s|__HOME__|$HOME|g" \
    -e "s|__INSTALL_DIR__|$HOME/Library/Application Support/hub|g" \
    -e "s|__HUB_CONFIG__|$PWD/huba.config.json|g" \
    com.dipgle.hubd.plist > ~/Library/LaunchAgents/com.dipgle.hubd.plist

launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.dipgle.hubd.plist
launchctl list | grep hubd        # the system label is still com.dipgle.hubd
tail -f ~/Library/Logs/hubd.err
```

`./huba self-install` does exactly the three steps of `install_update.sh` (build →
sign with a certificate → install to its own path), **rewritten in Rust** rather
than calling the script: huba has to be able to install itself, or every fix
needs someone sitting at the machine to type one line — exactly what this project
exists to remove. From the phone: `/upgrade`. If any step fails, **the installed
copy is left untouched**.

launchd runs the **installed copy** `~/Library/Application Support/hub/bin/hubd`,
not the `cargo` build in `target/`. In short (details in `CLAUDE.md`): macOS pins
permissions to a binary's signature, and `cargo` ad-hoc re-signs after **every**
build/test, so the copy in `target/` keeps changing identity and loses its grants
on the next restart. If you change the code and forget `install_update.sh`, the
daemon keeps running the old code — huba compares the newest source file with the
installed binary to catch exactly that.

`hubad` holds a pid lock (`data/hubd.lock`) so two daemons never run at once; cycle
errors are counted with exponential backoff (up to 10 minutes), and after 5 in a
row it writes `logs/notify.log` and shows a macOS notification.

## Running on Windows (⚠ NOT yet run on a real Windows machine)

The Windows build drives **Windows Terminal** (`wt.exe`) instead of Terminal.app:
keystrokes via `SendInput`, screen reading via UI Automation
(`rust/src/keys_win.rs`). It is cross-compiled and linted on EVERY `./gate.sh` run
(step ②b, `x86_64-pc-windows-gnu`, clippy `-D warnings`), but has never run on a
Windows machine.

1. **Measure first** (read-only, ~10 seconds):
   `powershell -ExecutionPolicy Bypass -File scripts\do-windows.ps1`
   ⟹ a file `%TEMP%\huba-do-windows-*.txt`. It answers what the
   session-to-window matching still needs to know (what `claude agents --json`
   prints on Windows, which process `claude` runs under, the Windows Terminal
   window class).
2. **Configure**: `huba.config.json` (the `claude_accounts` of THAT machine) +
   `huba.env` — `huba setup` opens a page to fill them in. ⚠ Use a **separate
   Telegram bot** for the Windows machine: two hubd processes on one bot fight over
   `getUpdates` (409 Conflict).
3. **Install**: from a NORMAL PowerShell (not "Run as Administrator" — UIPI blocks
   keystrokes across privilege levels):
   `powershell -ExecutionPolicy Bypass -File install_update.ps1` (needs Rust), or
   `… install_update.ps1 -BinDir <folder with hubad.exe + huba.exe>` with a prebuilt
   copy. It installs into `%LOCALAPPDATA%\hub\bin`, adds it to PATH, sets
   `HUB_CONFIG`, and registers the `huba-hubd` task (Task Scheduler, at logon,
   restarted if it dies). Check: `install_update.ps1 -Verify`.

**Known NOT to work on Windows yet**: matching a running session to its window
(waiting on the step-1 measurement) · `/upgrade` (prints the reinstall command
instead of doing it) · `/anh` window screenshots · lock-screen detection · `/web`
driving Chrome (AppleScript) · system notifications after repeated cycle failures
(only `logs/notify.log`).

## Commands

Only the owner can give orders: Telegram is gated by `chat_id`
(`telegram::update_sender` reads the right field for each update shape); anyone
else typing `/new` is just typing text. The parser (`verbs::parse_command`) is a
pure function — text in, a route out — so it neither knows nor needs to know which
channel a command came from.

**Claude sessions**

| Command | What it does |
|---|---|
| `/sessions` (or a bare `/session`) | list live sessions, one button each — tap to follow (`/session -` to stop following) |
| *(plain text, no `/`)* | type straight into the session you follow |
| `/new [-a acc] [-s project] <task>` | open a **real Terminal window** running `claude` and type the task into it; an empty task is fine (open the window, talk later) |
| `/ask <question>` | side question to the followed session, **on a fork** — the original session gains no turn |
| `/tell <text>` | continue a background session (`/stop` it first) |
| `/stop [id]` | background session: stop, conversation kept · window session opened by huba: **shut down for good** — `/exit`, then close the window |
| `/handover [id]` | close out: a handover note + a new session with the same context + the `--resume` command |

**Typing into a session window** (Terminal sessions only — see the Telegram section above)

| Command | What it does |
|---|---|
| `/type <text>` | type text into the followed session, plus Enter when it can measure that the text is still sitting in the input box |
| `/key <up\|down\|left\|right\|enter\|esc\|tab\|space\|1-9>` | press one key |
| `/shot` | read the text currently on the session's screen |

**Operations**

| Command | What it does |
|---|---|
| `/accounts` | the accounts: which session belongs to which, remaining quota, which account `/new` uses by default, which are 🔒 locked |
| `/accounts detail <name>` (or `/accounts acc3`) | details of ONE account: email · name · organization · role · plan · folder · quota · running sessions · lock — read from `.claude.json`, without running `claude` |
| `/new -a <locked account> …` | huba ASKS for confirmation on Telegram; approve and it opens that one session, the lock stays |
| `/accounts khoa <name>` · `/accounts mo <name>` | lock / unlock an account (`"locked": true` in `claude_accounts`; *khoá* = lock, *mở* = unlock) — a locked account is never picked automatically, never used to open a session even with `-a`, never probed for `/usage` |
| `/terminal <command line>` | open a real Terminal window (with a tty) and run the command — for `sudo`, `ssh -t`, `passwd`; the window stays open for typing the password |
| `/upgrade` | huba rebuilds itself from the current code and restarts |
| `/run` · `/doctor` | run one cycle now · real checks |
| `/set <key> <value>` | change one config field (validated + backed up + written atomically) |
| `/help` | this table, read straight from the code (`commands::ROUTES` is the source) |

`/ask`, `/handover`, `/new`, `/tell` call the real `claude`, so they **use your
plan's quota** — exactly as if you typed them in a terminal, no more.

On a **Max** plan (`claude auth status` → `subscriptionType: max`) there is no
per-call bill. The `total_cost_usd` the CLI reports is computed at **API list
price**, so read it as a **gauge of how big a call was**, not money taken from an
account. The `spend` table records that gauge silently; nothing shows it on screen.

## Guard rails (why it is designed this way)

- **Background sessions run behind a deny list** (`sessions::DENIED_TOOLS`): no
  push/merge/reset, no ssh/scp/sudo/rm/docker/launchctl/`*deploy*`, no
  WebFetch/WebSearch. Writing files inside the working tree is allowed — that is
  the work.
- **Ask/handover run on a fork** with an allowlist of `Read,Grep,Glob`: a question
  typed on a phone **has no hand to write with**. The acceptance test is the
  ORIGINAL transcript file: same bytes, same mtime.
- **Pressing keys requires LOOKING first.** In `claude`, an arrow key both moves
  and confirms, so huba sends one only when it can **prove** there is no choice
  dialog on screen — `keys::look` returns `Saw`/`Withheld`/`Blind` instead of
  folding all three into `None`, because folding them fails OPEN exactly when huba
  is blindest.
- **One way in, one set of books**: everything that changes state goes through
  Telegram as an ORDER, so there is always a trace where a person can read it.
- **huba does not hide text from the owner.** The destination is the owner's own
  private chat, so session text is passed through as-is; a possible secret is
  *logged* (`preview_risk_noted`), not blanked out. huba's own secrets (bot token,
  `huba.env`) are never logged — only key names are.

## Configuration

`huba.config.json` — no secrets, only the NAMES of environment variables:

```jsonc
{
  "poll_interval_sec": 120,
  "call": { "max_budget_usd": 0.5, "timeout_sec": 240 },  // cap for ONE call
  // The Telegram channel. The key is called `confirm` because it started life as
  // two Confirm/Cancel buttons; it is now the main command channel. NAMES only.
  "confirm": { "enabled": true,
               "bot_token_env": "HUB_TELEGRAM_BOT_TOKEN",
               "chat_id_env": "HUB_TELEGRAM_CHAT_ID",   // this is the owner gate
               "timeout_sec": 90 },
  "auto_handover": { "enabled": true, "at_percent": 60, "idle_sec": 120 },
  "projects": { "dwork": {}, "sdvi": {} },                 // folder names
  "claude_accounts": [{ "name": "acc1" }, { "name": "acc2", "config_dir": "~/.claude-acc2" }]
}
```

Keys left over from the inbox era (`triage`, `act`, `autonomy`, `routing`,
`daily_budget_usd`, `max_triage_per_cycle`, `web`, `leak_patterns`, and later
`adapters` + `trust`) are now unknown keys: an old file still loads, and they drop
off on the next write. A huba that refused to start because of a stale key would
cut the owner off from their only channel.

## Data

`data/huba.sqlite` (WAL) — three tables:

| table | what it holds |
|---|---|
| `runs` | one row per **cycle** (`run_once`): ok/error, so `/doctor` does not have to guess. A cycle is judged by the error lines it logged, not by its own return value — otherwise every row would read "ok" and the panel would be blind |
| `cursors` | the followed session, the `watch:sessions` book, the project pin, the latest handover/side answer |
| `spend` | each `claude` call you pressed for, with its size gauge — recorded so the question can be answered, not to be displayed |

(plus `schema_meta`.) Telegram updates are also written to a `tin_den` table
**before** the long-poll offset moves, so a power cut or a dead daemon cannot lose
an order; rows are kept for 7 days.

## Tests

```bash
./gate.sh      # full gate: fmt · clippy (+ Windows target) · build · EVERY test binary · doctests
# or, plainly:
cd rust && cargo test --offline --no-fail-fast
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`--no-fail-fast` matters: plain `cargo test` stops at the first red target, and a
partial run reads like a verdict. `gate.sh` also copies the test binaries out of
`target/debug/deps/` before running them — on the machine this was built on, a
fresh binary's first run inside that (huge) directory stalled ~2 minutes each.

A few tests read real folders under `~/projects` on the author's machine and will
fail elsewhere (for example `rust/tests/session_label_declared.rs`, which says so in
its doc comment).

Green tests are necessary, not sufficient: the real path must run at least once
(`./huba once`, then a REAL command typed in the Telegram chat).

## Contact

- Email: [info@dipgle.com](mailto:info@dipgle.com)

## Support the project

If you find this useful, you can support it via **Binance Pay** — scan the code
with the Binance app or open the link; you choose the currency and the amount.

- Link: <https://app.binance.com/uni-qr/XjaYe6fE>

<img src="docs/donate-binance-qr.png" alt="Binance Pay QR code for donations" width="220">
