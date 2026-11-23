# tele-sihub (huba) — watch and drive the Claude CLI sessions on your computer from your phone

*Tiếng Việt: [README.vi.md](README.vi.md)*

On a working machine there are often a dozen `claude` sessions running in terminal
windows. `huba` (the program's name; this repository is its public home) puts
them into **one Telegram chat**: see what each session is doing, open a new one,
type straight into a running session, ask a side question without disturbing its
work, stop it, or close it out with a handover so you can continue at the desk.

Traffic goes both ways, and **the computer is always the one calling out** — no port is
opened on the machine, and no address points back to it:

```
  phone ──► Telegram ──► hubad (long-poll getUpdates) ──► claude CLI on the computer
    ▲                              │
    └──── notifications + buttons ◄┘   (only on a CHANGE, never on a state)
```

> **Language note.** This README is in English. Code comments, the other docs
> (`CLAUDE.md`, `PLAN.md`, `UC.md`, …) and the messages huba sends to Telegram are
> in **Vietnamese** — it was built for a Vietnamese-speaking owner.

**Platforms** (the code is one Rust crate; each OS has its own way to type into a
session window and read it back):

| OS | How huba types into / reads a session | Installer | Status |
|---|---|---|---|
| macOS | Terminal.app + AppleScript (`do script`) | `install_update.sh` (launchd) | the original, used daily |
| Linux (Ubuntu) | **tmux** panes | `install_update_linux.sh` (`systemd --user`) | installed and running on the author's machine |
| Windows | the session's **console** (`AttachConsole` + `WriteConsoleInputW`) | `CAI.cmd` → `install_update.ps1` (Task Scheduler) | cross-built and linted only — **has not run on a real Windows machine** |

Sessions of the Claude CLI are the main target. On Linux, `opencode` sessions can
be listed and driven too (accounts with `"cli": "opencode"`).

## Leave the laptop at home

**Your phone and Telegram are all you need.**

Stepping out usually means packing up: close the lid — but first sit and wait for
every Claude session to finish, or cut a job off halfway. With huba there is
nothing to pack. The Mac stays at home and keeps working; you leave whenever you
want.

From your phone you can still:

- know the moment a session finishes, and what it did;
- answer a session's question with a single tap;
- hand it the next task, or start a new session for a new job;
- get the files it produces delivered straight to the chat.

At a café, on a trip, out with friends — the work keeps moving.

**One-time setup:** keep the computer plugged in, online and logged in, with automatic
sleep turned off (macOS: `caffeinate`). Locking the screen is fine — typing into
sessions and `/shot` keep working; only `/anh` (screenshots) and `/web` (driving
Chrome) need it unlocked (macOS).

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
  just mentioned, `👁 Vào phiên` (enter session), `⏎ Gửi` (send). When the input
  box holds Claude's grey **suggestion** (ghost text) rather than something typed,
  `/shot` labels the send button `💡⏎` — the tap accepts the suggestion and sends
  it (Linux/tmux only; elsewhere the label stays `⏎`).
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
  session goes through a terminal window addressed by tty, and an editor session has no
  such window — a row you can watch but not drive is not a bridge, just one
  more name to confuse. The number of skipped sessions is still counted and reported.
- **A session can ask huba to run a command for it** (the *mailbox*). Some commands
  are blocked inside a session on purpose (`git push`, `ssh`, `rm`…). The session
  writes one command line into `huba-run.txt` in its own scratchpad folder; huba
  reads the **session id from that path** (nobody types an id), runs the command,
  and pastes the result back into that session's input. On Linux the scratchpad
  roots under `/tmp` and `$TMPDIR` (and `CLAUDE_CODE_TMPDIR`) are all scanned; on
  Windows `%TEMP%\claude`.

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

## Before you start

There are two machines in every install: the one that **builds** a package and the one
that **runs** huba (they can be the same). Packages are built from a clone of this
repository sitting inside its workspace (`dong-goi-*.sh` also packs the shared rules,
skills, the security gate in `../scripts/` and the `project-agent` MCP server from
`../AI/codetrail/`) — a package step that misses any of them stops with exit 2 instead
of building half a package.

| | Build machine | Machine that runs huba |
|---|---|---|
| **macOS** (Apple Silicon only) | Rust/`cargo`; commit first — the package is named after `HEAD` but packs the tree as it is (the packer only warns about uncommitted changes) | `python3`; `claude` / `opencode` are installed by the installer at the version pinned in `cli-ghim.env`; `hubd` is signed ON this machine |
| **Linux** | Rust/`cargo` + target `x86_64-unknown-linux-gnu` (`HUBA_LINUX_TARGET` for ARM); from a Mac also `cargo-zigbuild` + `zig` | `tmux`, `python3`, `systemd --user` |
| **Windows** | Linux or macOS with `zig` (or mingw) | Python 3 (the installer tries `winget`) |

Every machine also needs a **Telegram bot** (token from @BotFather) — one bot per
machine, because two `hubd` processes on one bot fight over `getUpdates`.

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

## A window on the machine — `huba ui`

```bash
./huba ui            # opens a window (Chrome/Edge/Chromium/Brave `--app` when one is installed)
./huba ui --no-open  # print the URL and serve — headless, or a machine with no display
```

Which browser: macOS tries Google Chrome, Edge, Brave, Chromium in that order; Linux
looks for `google-chrome`, `google-chrome-stable`, `chromium`, `chromium-browser`,
`brave-browser`, `microsoft-edge` on `PATH` and starts the first one with `--app=<url>`
(its own window, no address bar). A machine with **only Firefox** has no `--app`, so
it falls back to `xdg-open` (Linux) / `open` (macOS) and you get an ordinary browser
tab. Either way the URL is printed first, so a failed open never loses the page.

A small web page served by huba itself from `ui/` (next to the `huba` binary), for
when you are sitting at the machine and want a list instead of chat commands. It
reads the same snapshot `huba sessions` reads, so the two agree on the count.

- **Loopback only.** It binds `127.0.0.1`, prints a **one-time ticket** in the URL and
  trades it for a `HttpOnly; SameSite=Strict` cookie; the ticket is refused the second
  time. `Host` must be the exact loopback authority (DNS-rebinding), `POST` checks
  `Origin`, request body is capped at 1 MiB, and a path cannot climb out of `ui/`.
  The page ships a CSP (`default-src 'self'`, no scripts but its own files) and
  `frame-ancestors 'none'`.
- **What it does today:** the session list with the machine's real sessions, search
  and paging over the whole list, and three states that never lie — empty, normal, and
  *data is stale* (a failed read keeps the last picture instead of drawing "0 sessions").
- **What it does not do yet:** read one session's conversation (K3), send anything
  (K4), or the account/token screens. Telegram remains the full command channel; this
  is a read-only window while it grows.

## Start with the machine — Linux (systemd --user)

Needs `tmux`. Sessions you want huba to drive must run inside tmux panes (`/new`
opens one for you).

```bash
bash scripts/dong-goi-linux.sh             # build → .tmp/huba-linux-<sha>.tar.gz (prints sha256)
tar -xzf huba-linux-<sha>.tar.gz && cd huba-linux-<sha>
bash install_update_linux.sh               # install OR update; safe to re-run
bash install_update_linux.sh --verify      # installed binary == this package? service running?
```

The installer copies the package into `$HUBA_HOME` (default `~/huba`) and **never
overwrites** `huba.config.json`, `huba.env` or `data/`. It then creates the config
if missing, adds missing accounts from a template, installs the `systemd --user`
unit `huba-hubd`, adds a "huba — quản lý" launcher, and runs a self-check
(`huba linux-tu-kiem`: open a tmux pane, type, read back, Ctrl-C, close). Logs:
`~/huba/logs/hubd.err`, `journalctl --user -u huba-hubd`. Exit code: 0 done · 1
failed · 2 could not measure. Switches: `HUBA_HOME`, `HUBA_KHONG_CAI_CLI=1` (do not
auto-install `claude`/`opencode`), `HUBA_KHONG_MO_TRANG=1` (do not open the setup page).

The package also carries `kem/` — the shared bundle (common rules for sessions,
skills/agents, the security gate behind `/new`'s deny list, the `project-agent`
MCP server, the context-reminder hook, an account template). Its paths are written into `huba.env`
(`HUBA_LUAT_GOC`, `BAO_MAT_POLICY`, `HUBA_MCP_PROJECT_AGENT`, `HUBA_OPENCODE_PLUGIN`).

**You choose the working folder at install time** (the workspace root that holds your
projects): the installer asks (Enter = `~/projects`), or set `HUBA_GOC=<folder>` to skip
the question. It creates the folder if missing and writes it as `workspace_root`. Everything
else follows it: `CLAUDE.md` and `.claude/{skills,agents,settings.json}` go **inside** that
folder (existing files are kept, never overwritten), the context-reminder hook (≥ 50 %) is
**merged** into its `settings.json`, the `project-agent` MCP server gets `PROJECTS_ROOT` =
that folder, and `/new` opens sessions there.

## Start with the machine — macOS (launchd)

**A new Mac, from a package** (no clone needed on the target):

```bash
bash scripts/dong-goi-mac.sh                     # on the build Mac → .tmp/huba-mac-<sha>.tar.gz (prints sha256)
tar -xzf huba-mac-<sha>.tar.gz
bash huba-mac-<sha>/install_mac.sh               # install OR update; safe to re-run
bash ~/huba/install_mac.sh --verify              # installed hubd == this package? launchd job loaded?
```

The installer copies the package into `$HUBA_HOME` (default `~/huba`) and **never
overwrites** `huba.config.json`, `huba.env` or `data/`. It creates `~/projects` and puts the
shared `CLAUDE.md` + skills/agents there (existing files are kept), sets up the account
folders and the `project-agent` MCP server, installs the pinned CLIs, creates the signing
identity **"Hub Local Signing"** once (`make-signing-cert.sh`), signs `hubd`, writes the
launchd plist and starts it. Exit code: 0 done · 1 failed · 2 could not measure (package
incomplete, or not a Mac). Switches: `HUBA_HOME`, `HUBA_KHONG_CAI_CLI=1`,
`HUBA_KHONG_MO_TRANG=1`. To update, unpack the newer package and run its `install_mac.sh`.

**Three things only a person can do** (system dialogs and secrets cannot be automated —
huba cannot type into a session until the first two are done):

1. System Settings › Privacy & Security › **Accessibility**: add
   `~/Library/Application Support/hub/bin/hubd`.
2. The first time huba drives Terminal, macOS asks *"hubd wants to control Terminal"* —
   press **Allow**.
3. The Telegram bot: the installer opens the `huba setup` page if `huba.env` has no bot
   yet (paste the token, send the bot one message, Save). Then log each account in from
   Telegram: `/accounts <acc> -l`.

The same notes travel inside the package as `DOC-TRUOC.txt`.

**A Mac that keeps a clone** (the developer's machine). `bash install_update.sh` builds,
signs, installs and restarts; `bash install_update.sh --verify` says whether the installed
`hubd` is the current build. The installer turns on `core.hooksPath=.githooks`, after which
every `git pull` that changes runnable files **re-installs by itself** in the background
(log in `.tmp/tu-cai-<time>.log`; `HUBA_KHONG_TU_CAI=1` skips it). By hand, the same steps
without the script:

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
copy is left untouched**. ⚠ `/upgrade` rebuilds the code **already on the machine** — it
does not `git pull`; with the hook above, a pull alone is enough.

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

Each `claude` session lives in a **console** (one Windows Terminal tab, or one
classic console window). huba attaches to that console (`AttachConsole`), writes
keys straight into its input queue (`WriteConsoleInputW`) and reads the text back
(`ReadConsoleOutputCharacterW`) — no focus, no window in front, and it never
grabs the keyboard or mouse of the person at the machine (`rust/src/keys_win.rs`).
In huba's books a Windows session's "tty" is `con<pid of the root shell>`.
It is cross-compiled and linted on EVERY `./gate.sh` run (step ②b,
`x86_64-pc-windows-gnu`, clippy `-D warnings`), but it has **never run on a real
Windows machine**.

**Build the package** (on Linux with `zig` or mingw): `bash scripts/dong-goi-windows.sh`
→ `.tmp/huba-windows-<sha>.zip` (`huba.exe`, `hubad.exe`, installer scripts, `kem\`
shared bundle with `project-agent.exe`).

**Install, no commands to type:** unzip, open the `huba-windows-*` folder, and
double-click **`CAI.cmd`**. It removes the "downloaded from the internet" mark,
test-runs `huba.exe`, creates the config, opens the page for the Telegram bot token
if there is none, then installs and starts huba in the background (Task Scheduler
`huba-hubd`, at logon, restarted if it dies), asking through dialog boxes where it
needs you — including a folder picker for your **working folder** (Cancel = `%USERPROFILE%\projects`);
rules, skills, the reminder hook and the MCP server all follow that folder. Its log is `cai.log`. ⚠ Use a **separate Telegram bot** for the Windows
machine: two hubd processes on one bot fight over `getUpdates` (409 Conflict).
Python 3 is needed for the shared bundle; the installer tries
`winget install Python.Python.3.12 --scope user` and skips that part, saying so,
if it cannot.

**Application Control.** `huba.exe` is **not code-signed**. Windows 11 *Smart App
Control* (or a WDAC/AppLocker policy on a managed machine) blocks unsigned
programs with "An Application Control policy has blocked this file" — nothing in
the package can lift that. On a personal machine you can turn Smart App Control
off (it cannot be turned back on without reinstalling Windows); on a managed one an
administrator has to allow the file. `CAI.cmd` detects this and says so.

**Measure first** (about 30 s, needs no config): `.\huba.exe windows-tu-kiem` in a
normal PowerShell opens two console windows, types, reads back and closes them;
it prints ✅/❌/⏭ per item and writes a result file. Exit code 0 = all pass · 1 = a
failure · 2 = something could not be measured. Run it **before** trusting any line
of this section. Read-only extra: `scripts\do-windows.ps1`. Run huba from a NORMAL
PowerShell, not "Run as Administrator" — a normal process cannot attach to the
console of an elevated one.

**Not done on Windows yet**: `/anh` window screenshots · `/web` driving Chrome ·
lock-screen detection · `/upgrade` (prints the command instead of doing it) ·
clearing the queued input (`/clean`) · scrolling back beyond the visible window.
**Unmeasured**: that `claude` (Node) accepts arrow keys and multi-line paste
delivered through the console — the first real `/new` is that measurement.

## Commands

**Every command explains itself:** `/<command> -h` (or `--help`) shows its syntax, flags and an
example, e.g. `/new -h`. `/set -h` lists the config sections and `/set <section> -h` lists each field
with its type, current value and default — so you can see what `/set` accepts before you set it.
`-h` only counts when it stands alone after the command name; a task like `/new fix it -h` is still a task.

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
| `/new [-a acc] [-s project] [-y] <task>` | open a **real terminal window** (Terminal tab / tmux pane / console window) running the CLI and type the task into it; an empty task is fine (open the window, talk later); `-y` opens a locked account without asking |
| `/ask <question>` | side question to the followed session, **on a fork** — the original session gains no turn |
| `/stop [id]` | background session: stop, conversation kept · window session opened by huba: **shut down for good** — `/exit`, then close the window |
| `/handover [id]` | close out: a handover note + a new session with the same context + the `--resume` command |

**Typing into a session window** (sessions with a terminal window only — see the Telegram section above)

| Command | What it does |
|---|---|
| `/type <text>` | type text into the followed session, plus Enter when it can measure that the text is still sitting in the input box |
| `/key <up\|down\|left\|right\|enter\|esc\|tab\|space\|1-9>` | press one key |
| `/enter` · `/esc` · `/right` | send what is in the input box · close the open dialog · accept Claude's grey suggestion (→), then `/enter` to send |
| `/pick <question>.<choice>` · `/tab <n>` | answer one question of a multi-question dialog · move to that tab without choosing |
| `/clear` · `/clean` | clear the typed text in the input box · clear that AND the queue |
| `/ctrlc` | Ctrl+C: interrupt the turn in progress |
| `/shot` | read the text currently on the session's screen, with the buttons that apply (`💡⏎` when the box holds a suggestion) |
| `/anh` · `/front` | a real screenshot (macOS) · bring the session's window to the front |
| `/effort <low…max>` | change the followed session's effort level |

**Operations**

| Command | What it does |
|---|---|
| `/accounts` | the accounts: which session belongs to which, remaining quota, which account `/new` uses by default, which are 🔒 locked |
| `/accounts detail <name>` (or `/accounts acc3`) | details of ONE account: email · name · organization · role · plan · folder · quota · running sessions · lock — read from `.claude.json`, without running `claude` |
| `/new -a <locked account> …` | huba ASKS for confirmation on Telegram; approve and it opens that one session, the lock stays |
| `/accounts <name> -b` · `/accounts <name> -a` | lock / unlock an account (`"locked": true` in `claude_accounts`) — a locked account is never picked automatically, never used to open a session even with `-a`, never probed for `/usage`. Other flags: `-l` log in again · `-k <API key>` use an API key (metered; `-k xoa` goes back to the plan) |
| `/runin <id> <command>` | run on the machine and paste the result into that session (the mailbox above does this on the session's own request) |
| `/close [id]` | close the session AND its window |
| `/terminal` | list the bare terminal windows (runs nothing); to open one, use `/new` |
| `/web` | Chrome on the machine (macOS): empty = the tabs · an address = open it |
| `/upgrade` | huba rebuilds itself from the current code and restarts (Windows: prints the command instead) |
| `/run` · `/doctor` | run one cycle now · real checks |
| `/set <key> <value>` | change one config field (validated + backed up + written atomically) |
| `/help` | this table, read straight from the code (`commands::ROUTES` is the source) |

**From one session to another** (CLI — a session runs these from its own Bash; all
inter-session talk goes through huba, not `SendMessage` straight to another session's socket)

| Command | What it does |
|---|---|
| `huba gui <id> "<text>"` | **ask another session to do something**: types `[huba · từ <sender>] <text>` into that session's input box and sends it (same typing core as `/type`), then posts one `📨 sender → target` line to Telegram so the owner sees every hand-off. Refuses: an id prefix matching several sessions · the caller itself · a bare shell window (`win-<tty>` — text there would run as a shell command, outside the deny list) · a target with a choice dialog open (Enter would pick an option) · text over 2000 characters (write a file and send its path). Exit 0 sent (the text left the input box, read off the screen) · 1 refused · 2 could not measure, or typed but not seen to leave the box |
| `huba ask <id> "<question>"` | **ask a question only**: answered by a read-only fork; the target session gains no turn |
| `huba sessions` | the ids to use above |

`/ask`, `/handover`, `/new` call the real `claude`, so they **use your
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
  The one session-to-session door, `huba gui` (owner's choice, 2026-11-23: *send
  straight, tell Telegram*), does not wait for a tap — so it posts every hand-off
  to Telegram instead, and it never types into a bare shell or an open dialog.
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
./gate.sh      # full gate: fmt · clippy (+ Windows and Linux targets) · build · EVERY test binary · doctests
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
