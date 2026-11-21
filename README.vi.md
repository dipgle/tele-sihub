# huba — các phiên Claude CLI trên máy này, xem và điều khiển từ điện thoại

*English: [README.md](README.md)*

Trên một máy đang làm việc lúc nào cũng có mươi phiên `claude` chạy trong các cửa sổ terminal.
`huba` đưa chúng vào **một buồng chat Telegram**: xem phiên nào đang làm gì, mở
phiên mới, gõ thẳng vào một phiên đang chạy, hỏi chen ngang mà không phá việc nó
đang làm, dừng nó, hoặc đóng sổ lấy bản bàn giao để làm tiếp trên máy.

Đường đi hai chiều, và **máy này luôn là bên gọi ra ngoài** — không mở cổng nào
vào máy, không có địa chỉ nào trỏ về đây:

```
  điện thoại ──► Telegram ──► hubad (long-poll getUpdates) ──► claude CLI trên máy
       ▲                              │
       └──── tin báo + nút bấm ◄──────┘   (chỉ khi có THAY ĐỔI, không báo trạng thái)
```

**Nền tảng** (mã là một crate Rust; mỗi hệ điều hành có cách riêng để gõ vào cửa sổ phiên và đọc lại):

| Hệ | Cách huba gõ vào / đọc phiên | Trình cài | Tình trạng |
|---|---|---|---|
| macOS | Terminal.app + AppleScript (`do script`) | `install_update.sh` (launchd) | bản gốc, dùng hằng ngày |
| Linux (Ubuntu) | pane **tmux** | `install_update_linux.sh` (`systemd --user`) | đã cài và đang chạy trên máy tác giả |
| Windows | **console** của phiên (`AttachConsole` + `WriteConsoleInputW`) | `CAI.cmd` → `install_update.ps1` (Task Scheduler) | mới dựng chéo và kiểm clippy — **chưa chạy trên máy Windows thật** |

Phiên Claude CLI là đích chính. Trên Linux, phiên `opencode` cũng liệt kê và điều khiển được (tài khoản `"cli": "opencode"`).

## Để máy ở nhà, cứ đi chơi

**Chỉ cần điện thoại và Telegram.**

Bình thường muốn ra ngoài là phải dọn máy: gập laptop lại — mà trước đó phải ngồi
chờ mọi phiên Claude chạy xong, hoặc đành cắt ngang việc đang làm dở. Có huba thì
chẳng phải dọn gì. Máy ở nhà cứ làm tiếp, còn bạn muốn đi lúc nào thì đi.

Trên điện thoại, bạn vẫn:

- biết ngay phiên nào vừa xong và nó đã làm gì;
- trả lời câu phiên đang hỏi chỉ bằng một cú bấm;
- giao việc tiếp theo, hoặc mở phiên mới cho việc mới;
- nhận tệp kết quả gửi thẳng vào buồng chat.

Ngồi quán cà phê, đi du lịch hay đi chơi với bạn bè — công việc vẫn chạy.

**Cài một lần:** để máy cắm sạc, có mạng, đã đăng nhập và tắt chế độ tự ngủ (hoặc
dùng `caffeinate`). Khoá màn hình thoải mái — gõ vào phiên và `/shot` vẫn chạy; chỉ
`/anh` (chụp ảnh màn) và `/web` (lái Chrome) cần mở khoá.

## Telegram là kênh chính

Từ 2026-08-11 Telegram không còn là cái loa hai nút mà là **kênh ra lệnh đầy
đủ**: mọi động từ ở bảng dưới gõ được từ đó, và phần lớn việc thường ngày không
phải gõ gì cả — huba gắn sẵn nút.

- **Chữ thường (không có dấu `/`) đi thẳng vào phiên đang theo.** Bấm một phiên
  trong danh sách là coi như đang ngồi trong phiên ấy; sau đó gõ gì nó nhận nấy.
  Chưa theo phiên nào thì huba **không đoán** một phiên để gõ vào — gõ nhầm cửa
  sổ là thứ không lùi lại được.
- **Nút thay cho việc nhớ cú pháp:** danh sách phiên (mỗi phiên một nút), từng
  lựa chọn của một phiên đang dừng lại hỏi, `📄 Xem đầy đủ` (mở luôn phiên ấy),
  `📎` lấy một tệp phiên vừa nhắc tới, `👁 Vào phiên`, `⏎ Gửi`. Khi ô nhập đang
  chứa **gợi ý mờ** của Claude chứ không phải chữ đã gõ, `/shot` đổi nút gửi thành
  `💡⏎` — bấm là nhận gợi ý rồi gửi luôn (chỉ Linux/tmux; nơi khác nhãn vẫn là `⏎`).
- **Cổng người:** biến `HUB_TELEGRAM_CHAT_ID` trong `huba.env`. Tin từ chat khác
  bị **ghi log rồi bỏ**, không im lặng. Từ 2026-08-14 đây là cổng người **duy
  nhất** — cổng thứ hai (`trust.tfl5_user_tids`, kiểm trong bộ phân tích lệnh)
  đi cùng phòng chat, và nó đáng đi: sau khi phòng đóng, chỗ gọi phải tự bịa ra
  "người gõ" để đi qua chính nó, nên nó không bao giờ từ chối được — trừ khi
  danh sách rỗng, và khi ấy nó nuốt sạch mọi mệnh lệnh trong im lặng.
- **Chỉ nói khi có thay đổi.** `watch.rs` so hai lượt ảnh chụp rồi báo đúng một
  lần: phiên vừa xong, phiên dừng lại hỏi, phiên dừng vì lỗi, phiên đã tắt. Báo
  theo *trạng thái* trên một vòng lặp 10 giây là cái điện thoại rung mãi không
  thôi — và cái loa như thế thì người ta tắt, mất luôn những lần đáng nghe.
- **Phiên chạy trong VS Code / Cursor không lên danh sách** (2026-08-13). Gõ vào
  chúng phải qua Terminal.app, địa chỉ bằng tty, mà phiên editor không có cửa sổ
  Terminal nào — một dòng chỉ xem được mà không lái được thì nó không phải cây
  cầu, nó là một cái tên nữa để nhầm. Số phiên bị bỏ vẫn được đếm và nói ra.
- **Phiên tự nhờ huba chạy lệnh (hòm thư).** Có lệnh bị chặn ngay trong phiên (`git
  push`, `ssh`, `rm`…). Phiên ghi MỘT dòng lệnh vào `huba-run.txt` trong thư mục
  scratchpad của chính nó; huba đọc **id phiên từ đường dẫn** (không ai gõ id), chạy
  lệnh, rồi dán kết quả vào ô nhập của phiên ấy. Linux: quét các gốc scratchpad dưới
  `/tmp` và `$TMPDIR` (cả `CLAUDE_CODE_TMPDIR`); Windows: `%TEMP%\claude`.

> **Kênh thứ hai đã đóng ngày 2026-08-14.** Tới hôm ấy còn một phòng chat trên
> tfl5 và một trang điện thoại chạy song song với Telegram. Hà: *"tạm thời không
> dùng tfl5 để xem cứ xóa hết đi"* — trang đã tắt thở từ hai ngày trước mà không
> ai nhận ra, đúng cái giá của hai giao diện cho một sản phẩm một người dùng.
> Đi theo nó: `portal.rs`, `live.rs` (socket `/ws/chat`), cả chặng hỏi vòng
> (`/ingest`), `adapters.tfl5`, `trust`, và ba thư viện không còn ai gọi
> (`tungstenite`, `axum`, `tokio`).

> **Trước đây huba là một hộp thư** (GitHub · devlog · email · Telegram → triage
> bằng `claude -p` → duyệt/gửi). Nhánh đó đã **xoá hẳn ngày 2026-08-08**: 65%
> hộp việc là thông báo CI, ngốn $5.89/$9.12 tổng chi, trong khi việc thật sự
> cần — điều khiển phiên từ xa — mới chạy đúng một lần. Chi tiết trong
> `CLAUDE.md`. Hệ quả thẳng thắn: **huba không còn tự tiêu hạn mức**; chỉ khi bạn
> bấm nút mới có một lần gọi `claude`, đúng giá như gõ ở terminal.

## Chạy trong 4 lệnh

```bash
cd ~/projects/huba
./huba setup           # trang cấu hình ở 127.0.0.1 → ghi huba.env (chmod 600)
./huba doctor          # kiểm tra thật: claude CLI, Telegram, thư mục dự án
./huba sessions        # mọi phiên claude đang sống, mọi tài khoản
./huba once            # một vòng: chạy những lệnh đã tới → sổ sách → cái loa
```

Bí mật (token bot Telegram + chat id) nằm trong `huba.env` (chmod 600), **chỉ tên
biến** nằm trong `huba.config.json`. Xem `huba.env.example`.

`huba setup` chỉ nghe ở `127.0.0.1`, vào bằng vé một lần trong URL, và **không
bao giờ hiện lại giá trị đã lưu** — nó chỉ nói khoá ấy *đã có* hay *chưa có*.
Đọc một file 600 rồi bơm ngược ra HTTP là tự tay dựng đúng cái đường rò mà file
600 sinh ra để chặn.

Dựng chỗ làm việc từ đầu (thư mục, gốc workspace, ba câu hay hỏi ngược):
`ARCHITECTURE.md`.

## Tự chạy cùng máy — Linux (systemd --user)

Cần `tmux`. Phiên muốn huba điều khiển phải chạy trong pane tmux (`/new` tự mở một pane).

```bash
bash scripts/dong-goi-linux.sh             # dựng → .tmp/huba-linux-<sha>.tar.gz (in sha256)
tar -xzf huba-linux-<sha>.tar.gz && cd huba-linux-<sha>
bash install_update_linux.sh               # cài HOẶC cập nhật; chạy lại bao nhiêu lần cũng được
bash install_update_linux.sh --verify      # bản cài == gói này không? dịch vụ có chạy không?
```

Trình cài chép gói vào `$HUBA_HOME` (mặc định `~/huba`) và **không bao giờ đè**
`huba.config.json`, `huba.env`, `data/`. Rồi nó tạo cấu hình nếu thiếu, thêm tài
khoản còn thiếu từ mẫu, cài unit `systemd --user` `huba-hubd`, thêm lối vào «huba — quản
lý» trong menu ứng dụng, và tự kiểm (`huba linux-tu-kiem`: mở pane tmux, gõ, đọc lại,
Ctrl-C, đóng). Nhật ký: `~/huba/logs/hubd.err`, `journalctl --user -u huba-hubd`. Mã
thoát: 0 xong · 1 hỏng · 2 không đo được. Công tắc: `HUBA_HOME`, `HUBA_KHONG_CAI_CLI=1`
(không tự cài `claude`/`opencode`), `HUBA_KHONG_MO_TRANG=1` (không mở trang cài đặt).

Gói còn mang `kem/` — phần dùng chung (luật chung cho phiên, skill/tác tử, cổng bảo mật
đứng sau danh sách cấm của `/new`, MCP `project-agent`, hook nhắc chuyển phiên, mẫu tài khoản). Đường dẫn của
chúng được ghi vào `huba.env` (`HUBA_LUAT_GOC`, `BAO_MAT_POLICY`, `HUBA_MCP_PROJECT_AGENT`,
`HUBA_OPENCODE_PLUGIN`).

**Bạn CHỌN thư mục làm việc lúc cài** (gốc workspace chứa các dự án): trình cài hỏi (Enter =
`~/projects`), hoặc đặt `HUBA_GOC=<thư mục>` để bỏ qua câu hỏi. Chưa có thư mục thì tạo, rồi ghi
thành `workspace_root`. Mọi thành phần đi theo nó: `CLAUDE.md` và `.claude/{skills,agents,settings.json}`
nằm **trong** thư mục ấy (tệp đã có thì GIỮ, không ghi đè), hook nhắc chuyển phiên (≥ 50 %) được
**gộp** vào `settings.json` của nó, MCP `project-agent` nhận `PROJECTS_ROOT` = thư mục ấy, và
`/new` mở phiên ở đó.

## Tự chạy cùng máy — macOS (launchd)

```bash
./huba self-install                                  # build → ký → cài → khởi động lại

# plist là BẢN MẪU: ba dấu chỗ phải thay bằng đường tuyệt đối của máy này
# (launchd không hiểu `~` hay `$HOME`, mà repo công khai thì không mang đường
#  dẫn của máy ai cả)
sed -e "s|__HOME__|$HOME|g" \
    -e "s|__INSTALL_DIR__|$HOME/Library/Application Support/hub|g" \
    -e "s|__HUB_CONFIG__|$PWD/huba.config.json|g" \
    com.dipgle.hubd.plist > ~/Library/LaunchAgents/com.dipgle.hubd.plist

launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.dipgle.hubd.plist
launchctl list | grep hubd        # nhãn hệ thống VẪN là com.dipgle.hubd
tail -f ~/Library/Logs/hubd.err
```

`./huba self-install` làm đúng ba bước của `install_update.sh` (build → ký bằng
chứng chỉ → cài ra đường riêng), **viết lại trong Rust** chứ không gọi script:
huba phải tự cài được chính nó, không thì mỗi bản vá lại cần một người ngồi ở
máy gõ một dòng — đúng thứ dự án này sinh ra để bỏ đi. Từ điện thoại: `/upgrade`.
Hỏng ở bất kỳ bước nào thì **không đụng vào bản đang cài**.

launchd chạy **bản đã cài** `~/Library/Application Support/hub/bin/hubd`, không
phải bản `cargo` trong `target/`. Lý do nằm ở `CLAUDE.md` §12, tóm tắt: macOS
neo quyền theo chữ ký, `cargo` ký ad-hoc lại sau **mỗi** lệnh build/test, nên
bản trong `target/` đổi danh tính liên tục và mất quyền ngay lần bật máy sau.
Sửa mã xong mà quên `install_update.sh` thì daemon vẫn chạy mã cũ — tab **Sức
khoẻ** có một hàng nói thẳng điều đó.

`hubad` giữ một pid-lock (`data/hubd.lock`) nên hai daemon không cùng chạy; lỗi
vòng chạy được đếm và backoff luỹ thừa (tối đa 10 phút), sau 5 lần liên tiếp thì
ghi `logs/notify.log` + hiện thông báo macOS.

## Chạy trên Windows (⚠ CHƯA chạy thử trên máy Windows thật)

Mỗi phiên `claude` nằm trong một **console** (một tab Windows Terminal, hay một cửa sổ
console cổ điển). huba gắn vào console ấy (`AttachConsole`), đặt phím thẳng vào hàng
nhập (`WriteConsoleInputW`) và đọc chữ lại (`ReadConsoleOutputCharacterW`) — không cần
tiêu điểm, không cần cửa sổ ở trước, không giành chuột/bàn phím của người đang ngồi máy
(`rust/src/keys_win.rs`). Trong sổ của huba, "tty" của phiên Windows là `con<pid shell gốc>`.
Nó được dựng-kiểm chéo ở MỌI lượt `./gate.sh` (bước ②b, `x86_64-pc-windows-gnu`, clippy
`-D warnings`), nhưng **chưa từng chạy trên máy Windows thật**.

**Dựng gói** (trên Linux có `zig` hoặc mingw): `bash scripts/dong-goi-windows.sh` →
`.tmp/huba-windows-<sha>.zip` (`huba.exe`, `hubad.exe`, các tệp cài, thư mục `kem\` dùng
chung kèm `project-agent.exe`).

**Cài, không phải gõ lệnh:** giải nén, mở thư mục `huba-windows-*`, bấm đôi **`CAI.cmd`**.
Nó gỡ dấu "tải từ mạng", thử chạy `huba.exe`, tạo cấu hình, mở trang điền token bot
Telegram nếu chưa có, rồi cài và chạy nền (Task Scheduler `huba-hubd`, lúc đăng nhập,
chết thì chạy lại), hỏi bạn bằng hộp thoại ở chỗ cần — gồm hộp chọn **thư mục làm việc** (Cancel = `%USERPROFILE%\projects`);
luật, skill, hook nhắc và MCP đều đi theo thư mục ấy. Nhật ký: `cai.log`. ⚠ Dùng **bot
Telegram riêng** cho máy Windows: hai hubd cùng một bot thì giành `getUpdates` (409
Conflict). Phần `kem\` cần Python 3; trình cài tự thử
`winget install Python.Python.3.12 --scope user`, không được thì bỏ qua phần đó và nói rõ.

**Application Control.** `huba.exe` **chưa ký số**. *Smart App Control* của Windows 11
(hoặc chính sách WDAC/AppLocker trên máy của tổ chức) chặn chương trình chưa ký bằng câu
"An Application Control policy has blocked this file" — gói không có gì gỡ được. Máy cá
nhân có thể tắt Smart App Control (tắt rồi không bật lại được nếu không cài lại Windows);
máy cơ quan thì quản trị viên phải cho phép tệp. `CAI.cmd` nhận ra tình huống này và nói rõ.

**Đo trước** (~30 giây, không cần cấu hình): `.\huba.exe windows-tu-kiem` trong PowerShell
THƯỜNG mở hai cửa sổ console, gõ thử, đọc lại rồi đóng; in ✅/❌/⏭ từng mục và ghi một tệp
kết quả. Mã thoát 0 = đạt hết · 1 = có mục hỏng · 2 = có mục KHÔNG ĐO ĐƯỢC. Chạy nó **trước
khi** tin bất kỳ dòng nào của mục này. Đo phụ, chỉ đọc: `scripts\do-windows.ps1`. Chạy huba
bằng PowerShell THƯỜNG, không "Run as Administrator" — tiến trình thường không gắn được vào
console của tiến trình nâng quyền.

**CHƯA làm trên Windows**: `/anh` chụp ảnh cửa sổ · `/web` lái Chrome · phát hiện màn khoá ·
`/upgrade` (báo lệnh thay vì tự làm) · xoá hàng chờ trong ô nhập (`/clean`) · cuộn ngược
quá khung nhìn. **Chưa đo**: `claude` (Node) có nhận phím mũi tên và khối dán nhiều dòng gửi
qua console không — lượt `/new` thật đầu tiên chính là phép đo ấy.

## Lệnh

Chỉ chủ máy ra lệnh được: Telegram gác bằng `chat_id` (`telegram::update_sender`
đọc đúng ô cho từng hình dạng update), người khác gõ `/new` thì đó chỉ là chữ.
Bộ phân tích (`verbs::parse_command`) là hàm thuần — vào là chữ, ra là một
route — nên nó không biết và không cần biết lệnh tới từ kênh nào.

**Phiên Claude**

| Lệnh | Việc |
|---|---|
| `/sessions` (hay `/session` trống) | danh sách phiên đang sống, mỗi phiên một nút — bấm để theo (`/session -` để thôi theo) |
| *(chữ thường, không dấu `/`)* | gõ thẳng vào phiên đang theo |
| `/new [-a acc] [-s dự án] [-y] <việc>` | mở một **cửa sổ terminal thật** (tab Terminal / pane tmux / cửa sổ console) chạy CLI rồi gõ việc ấy vào; việc rỗng cũng được (mở cửa sổ rồi nói sau); `-y` mở tài khoản đang khoá mà không hỏi |
| `/ask <câu hỏi>` | hỏi bên lề phiên đang theo, **trên bản fork** — phiên gốc không thêm lượt nào |
| `/tell <nội dung>` | nói tiếp vào phiên nền (phải `/stop` nó trước) |
| `/stop [id]` | phiên nền: dừng, hội thoại vẫn giữ · phiên cửa sổ huba mở: **tắt hẳn** — `/exit` rồi đóng cửa sổ |
| `/handover [id]` | đóng sổ: bản bàn giao + phiên mới giữ nguyên ngữ cảnh + lệnh `--resume` |

**Gõ thẳng vào cửa sổ phiên** (chỉ phiên có cửa sổ terminal — xem mục Telegram ở trên)

| Lệnh | Việc |
|---|---|
| `/type <chữ>` | gõ chữ vào phiên đang theo, kèm Enter khi đo được là chữ còn nằm trong ô |
| `/key <up\|down\|left\|right\|enter\|esc\|tab\|space\|1-9>` | bấm một phím |
| `/enter` · `/esc` · `/right` | gửi chữ đang nằm trong ô nhập · đóng hộp thoại đang mở · nhận gợi ý mờ của Claude (→) rồi `/enter` để gửi |
| `/pick <câu>.<lựa chọn>` · `/tab <số>` | trả lời một câu của bảng hỏi nhiều câu · sang tab ấy mà không chọn |
| `/clear` · `/clean` | xoá chữ đã gõ trong ô nhập · xoá cả ô nhập VÀ hàng chờ |
| `/ctrlc` | Ctrl+C: ngắt lượt đang chạy dở |
| `/shot` | đọc chữ đang hiện trên màn của phiên, kèm các nút phù hợp (`💡⏎` khi ô đang chứa gợi ý mờ) |
| `/anh` · `/front` | ảnh thật của màn hình (macOS) · đưa cửa sổ phiên ra trước |
| `/effort <low…max>` | đổi mức effort của phiên đang theo |

**Vận hành**

| Lệnh | Việc |
|---|---|
| `/accounts` | các tài khoản: phiên nào của ai, còn bao nhiêu hạn mức, `/new` mặc định vào tài khoản nào, cái nào 🔒 đang khoá |
| `/accounts detail <tên>` (hoặc `/accounts acc3`) | chi tiết MỘT tài khoản: email · tên · tổ chức · vai trò · gói · thư mục · hạn mức · phiên đang chạy · khoá — đọc sổ `.claude.json`, không chạy `claude` |
| `/new -a <tài khoản khoá> …` | huba HỎI xác nhận trên Telegram; bấm đồng ý thì mở đúng lượt đó, khoá giữ nguyên |
| `/accounts <tên> -b` · `/accounts <tên> -a` | khoá / mở một tài khoản (`"locked": true` trong `claude_accounts`) — đang khoá thì huba không tự chọn, không mở phiên kể cả khi gõ `-a`, không dò `/usage` bằng nó. Cờ khác: `-l` đăng nhập lại · `-k <API key>` dùng API key (tính tiền; `-k xoa` về gói) |
| `/runin <id> <lệnh>` | máy chạy, kết quả dán vào phiên ấy (hòm thư ở trên làm việc này khi phiên tự nhờ) |
| `/close [id]` | đóng hẳn phiên VÀ cửa sổ của nó |
| `/terminal` | liệt kê cửa sổ terminal trần (không chạy gì); muốn mở mới thì dùng `/new` |
| `/web` | Chrome trên máy (macOS): trống = các tab · địa chỉ = mở |
| `/upgrade` | huba tự dựng lại chính nó từ mã hiện tại rồi khởi động lại (Windows: báo lệnh thay vì tự làm) |
| `/run` · `/doctor` | chạy một vòng ngay · kiểm tra thật |
| `/set <khoá> <giá trị>` | sửa một trường cấu hình (validate + backup + ghi nguyên tử) |
| `/help` | bảng này, đọc thẳng từ mã |

`/ask`, `/handover`, `/new`, `/tell` gọi `claude` thật nên **ăn vào hạn mức
của gói** — đúng như khi bạn tự gõ ở terminal, không hơn.

Máy này chạy gói **Max** (`claude auth status` → `subscriptionType: max`): không
có hoá đơn tính theo từng lần gọi. Con số `total_cost_usd` mà CLI trả về được
quy theo **giá API niêm yết**, nên nó là **thước đo một cú gọi TO cỡ nào**, không
phải tiền bị trừ khỏi tài khoản. Sổ `spend` ghi thước đo ấy, im lặng; màn hình
không hiện.

## Hàng rào (vì sao thiết kế vậy)

- **Phiên nền chạy sau một danh sách cấm** (`sessions::DENIED_TOOLS`): không
  push/merge/reset, không ssh/scp/sudo/rm/docker/launchctl/`*deploy*`, không
  WebFetch/WebSearch. Ghi file trong cây làm việc thì được — đó là công việc.
- **Hỏi/bàn giao chạy trên bản fork** với allowlist `Read,Grep,Glob`: một câu
  hỏi gõ trên điện thoại **không có tay để ghi**. Nghiệm thu của UC này là tệp
  nhật ký GỐC: y nguyên số byte, y nguyên mtime.
- **Xem trước nội dung phiên bị quét rò rỉ** trước khi lên ảnh chụp
  (`redaction::leak_scan`) — lần chạy thật đầu tiên đã in ra một phiên có mật
  khẩu trong lượt cuối. Cổng đặt ở NGUỒN (`sessions::snapshot`), không ở từng
  chỗ gửi — nên một kênh mọc thêm sau này cũng không đi vòng qua được.
- **Bấm phím thì phải NHÌN trước.** Một mũi tên trong `claude` vừa di chuyển vừa
  xác nhận, nên huba chỉ gửi khi **chứng minh được** màn không có hộp chọn —
  `keys::look` trả `Saw`/`Withheld`/`Blind` chứ không gộp cả ba vào `None`, vì
  gộp là fail OPEN đúng lúc huba mù nhất (kể cả lúc trên màn đang có mật khẩu).
- **Một đường vào, một cuốn sổ**: mọi thứ đổi trạng thái đều đi qua Telegram
  dưới dạng LỆNH, nên luôn có dấu vết ở nơi người đọc được.

## Cấu hình

`huba.config.json` — không có bí mật, chỉ có TÊN biến môi trường:

```jsonc
{
  "poll_interval_sec": 120,
  "call": { "max_budget_usd": 0.5, "timeout_sec": 240 },  // trần MỘT lần gọi
  // Kênh Telegram. Tên khoá là `confirm` vì thời nó mới sinh ra chỉ có hai nút
  // Xác nhận/Huỷ; nay đây là kênh ra lệnh chính. Chỉ TÊN biến, không có giá trị.
  "confirm": { "enabled": true,
               "bot_token_env": "HUB_TELEGRAM_BOT_TOKEN",
               "chat_id_env": "HUB_TELEGRAM_CHAT_ID",   // đây là cổng người
               "timeout_sec": 90 },
  "auto_handover": { "enabled": true, "at_percent": 60, "idle_sec": 120 },
  "projects": { "dwork": {}, "sdvi": {} },                 // tên thư mục
  "claude_accounts": [{ "name": "acc1" }, { "name": "acc2", "config_dir": "~/.claude-acc2" }]
}
```

Khoá cũ của thời hộp thư (`triage`, `act`, `autonomy`, `routing`,
`daily_budget_usd`, `max_triage_per_cycle`, `web`, `leak_patterns`) nay là khoá
lạ, và từ 2026-08-14 có thêm `adapters` + `trust` (phòng chat tfl5): tệp cũ vẫn
nạp được, lần ghi kế tiếp thì rụng. Đó không phải chuyện lý thuyết — tệp thật
trên máy đang mang cả hai, và một huba từ chối khởi động vì một khoá cũ là một
huba cắt đứt chủ máy khỏi kênh duy nhất của mình.

## Dữ liệu

`data/huba.sqlite` (WAL) — ba bảng:

| bảng | giữ gì |
|---|---|
| `runs` | mỗi **vòng** (`run_once`): ok/lỗi, để `/doctor` không phải đoán. Tới 2026-08-14 đây là mỗi lượt ĐỌC KÊNH; chặng ấy đi cùng tfl5, và bảng phải đổi người ghi chứ không được bỏ trống — một khối "lỗi gần đây" luôn rỗng là phép đo mù, tệ hơn không có |
| `cursors` | phiên đang theo, sổ `watch:sessions`, ghim dự án, bản bàn giao/hỏi-bên-lề gần nhất |
| `spend` | mỗi lần gọi `claude` do bạn bấm, kèm thước đo độ lớn — ghi để trả lời được, không để trưng |

(cộng `schema_meta`.) Bốn bảng của thời hộp thư đã đi hẳn ở bước schema 4
(2026-08-10) — chúng sống thừa hai ngày sau khi sản phẩm ấy bị xoá và giữ 379
dòng không truy vấn nào với tới. Migration ấy chạy với `foreign_keys` TẮT (bốn
bảng tham chiếu lẫn nhau; bản đầu chết ngay lúc khởi động, exit 70) và **log
từng bảng kèm số dòng** chứ không "đã dọn xong" một câu.

## Trang trên điện thoại — ĐÃ GỠ (2026-08-14)

`fe/index.html` (bốn tab: Phiên · Trao đổi · Sức khoẻ · Cấu hình), `fe-deploy.mjs`
và 18 kịch bản nghiệm thu `fe-*.mjs` chạy Playwright trên bundle đã deploy — tất
cả đi cùng phòng chat tfl5.

Vì sao đáng ghi lại chứ không xoá lặng: bản rà soát 2026-08-14 đo được trang ấy
**đã tắt thở hai ngày** mà không ai nhận ra. Một sản phẩm một người dùng nuôi hai
giao diện thì cái ít dùng hơn sẽ hỏng âm thầm, và bộ nghiệm thu của nó — vốn là
thứ phải phát hiện ra điều đó — lại là thứ ít được chạy nhất. Lịch sử đầy đủ:
`memory/ra-soat-2026-08-14.md`; mã còn nguyên trong git trước `cf20874`.

## Test

```bash
cd rust && cargo test --offline --no-fail-fast   # 922 test / 146 target
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Test xanh là điều kiện cần, không phải đủ: đường thật phải chạy ít nhất một lần
(`./huba once`, rồi một mệnh lệnh gõ THẬT trong buồng Telegram).

## Bản Node cũ

`legacy-node/` là bản mẫu đầu tiên, giữ làm đối chiếu. Không chạy nữa.

## Liên hệ

- Email: [info@dipgle.com](mailto:info@dipgle.com)

## Ủng hộ

Thấy dự án có ích thì có thể ủng hộ qua **Binance Pay** — quét mã bằng app Binance
hoặc mở liên kết; người gửi tự chọn loại tiền và số tiền.

- Liên kết: <https://app.binance.com/uni-qr/XjaYe6fE>

<img src="docs/donate-binance-qr.png" alt="Mã QR Binance Pay để ủng hộ" width="220">
