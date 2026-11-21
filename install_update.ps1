# install_update.ps1 — bản Windows của install_update.sh: dựng → cài → chạy nền tự khởi động.
#
#   powershell -ExecutionPolicy Bypass -File install_update.ps1            # dựng từ mã rồi cài
#   powershell -ExecutionPolicy Bypass -File install_update.ps1 -BinDir D:\huba-win   # cài bản dựng sẵn
#   powershell -ExecutionPolicy Bypass -File install_update.ps1 -Verify    # CHỈ ĐỌC: bản cài có phải bản vừa dựng
#
# Chạy bằng PowerShell THƯỜNG — KHÔNG "Run as Administrator". huba gắn vào console của từng phiên
# (AttachConsole) để gõ phím và đọc màn; một tiến trình thường KHÔNG gắn được vào console của tiến
# trình chạy nâng quyền, nên huba và các cửa sổ `claude` phải cùng mức quyền thường.
# Đo trước khi cài: `huba.exe windows-tu-kiem` (trong gói) — gõ + đọc console thật, không cần cấu hình.
#
# Khác macOS ở ba chỗ (Hà 30/09 «viết để chạy được window»):
#   · không ký chứng chỉ: Windows không gắn quyền theo chữ ký như TCC (xem rust/src/keys_win.rs);
#   · launchd ⟹ Task Scheduler, tác vụ `huba-hubd`, chạy lúc đăng nhập, tự chạy lại khi chết;
#   · cài ra %LOCALAPPDATA%\hub\bin (hubd.exe + huba.exe), thư mục ấy vào PATH người dùng,
#     và HUB_CONFIG trỏ vào huba.config.json của cây này để `huba` gõ ở đâu cũng tìm ra cấu hình.
#
# ⚠ CHƯA CHẠY THỬ TRÊN MÁY WINDOWS THẬT (viết trên macOS, 30/09). Mỗi bước in mốc và DỪNG khi hỏng.
# ⚠ Một bot Telegram chỉ nhận lệnh ở MỘT nơi: hubd trên Windows mà dùng CÙNG bot với máy Mac thì
#   hai bên giành getUpdates (409 Conflict). Dùng bot riêng cho máy Windows, hoặc tắt hubd bên kia.

param(
    [string]$BinDir = '',
    [switch]$Verify
)
$ErrorActionPreference = 'Stop'

# 🔴 Hệ điều hành TRƯỚC mọi việc (Quang 15/11: «đây là máy win mà trình cài không kiểm tra hệ điều
# hành trước khi làm à»). PowerShell 7 chạy được cả trên macOS/Linux ⟹ tự hỏi: Windows PowerShell 5.1
# không có $IsWindows nhưng có $env:OS = 'Windows_NT'.
if (-not ($IsWindows -or $env:OS -eq 'Windows_NT')) {
    Write-Host 'SAI MÁY: install_update.ps1 dành cho Windows — macOS: install_update.sh / gói huba-mac-*.tar.gz (install_mac.sh) · Linux: gói huba-linux-*.tar.gz (install_update_linux.sh)'
    exit 2
}

$Here   = Split-Path -Parent $MyInvocation.MyCommand.Path
$Rust   = Join-Path $Here 'rust'
$Dest   = Join-Path $env:LOCALAPPDATA 'hub\bin'
$Task   = 'huba-hubd'
$Config = Join-Path $Here 'huba.config.json'

function Moc($s) { Write-Host ("[{0}] {1}" -f (Get-Date -Format 'HH:mm:ss'), $s) }

# Nguồn của hai binary: bản dựng sẵn (-BinDir) hay dựng từ mã.
if ($BinDir) {
    $SrcHubd = Join-Path $BinDir 'hubad.exe'
    $SrcCli  = Join-Path $BinDir 'huba.exe'
} else {
    $SrcHubd = Join-Path $Rust 'target\release\hubad.exe'
    $SrcCli  = Join-Path $Rust 'target\release\huba.exe'
}
$DstHubd = Join-Path $Dest 'hubd.exe'
$DstCli  = Join-Path $Dest 'huba.exe'

if ($Verify) {
    if (-not (Test-Path $DstHubd)) { Moc "CHUA CAI: khong co $DstHubd"; exit 1 }
    if (-not (Test-Path $SrcHubd)) { Moc "KHONG DO DUOC: khong co ban dung $SrcHubd"; exit 2 }
    $a = (Get-FileHash $DstHubd -Algorithm SHA256).Hash
    $b = (Get-FileHash $SrcHubd -Algorithm SHA256).Hash
    if ($a -eq $b) { Moc "KHOP — ban cai la ban dung hien co ($a)"; exit 0 }
    Moc "LECH — ban cai $a, ban dung $b"; exit 1
}

# ──────────────────────────────────────────────────────────────────────────────────────────
# Phần DÙNG CHUNG với gói Linux (`kem\`): luật chung, skill, cổng bảo mật, MCP codetrail, mẫu tài khoản.
# Quang 21/11 «đồng bộ hết tính năng để dựng bản cài window». Có thư mục `kem\` cạnh trình cài thì làm.
# ⚠ CHƯA CHẠY THỬ TRÊN WINDOWS THẬT (viết trên Ubuntu, không có PowerShell ở đó để chạy).
# ──────────────────────────────────────────────────────────────────────────────────────────
$Kem = Join-Path $Here 'kem'

# Python THẬT (3.x). Windows không có sẵn: `python.exe` trong WindowsApps chỉ là lối tắt mở Microsoft Store
# (chạy ra mã ≠ 0) — nên phải chạy thử, không tin `Get-Command`. Cài Node.js KHÔNG kéo theo Python.
function Tim-Python {
    foreach ($c in @(@('py', '-3'), @('python'), @('python3'))) {
        $cmd = Get-Command $c[0] -ErrorAction SilentlyContinue
        if (-not $cmd) { continue }
        $pre = @($c | Select-Object -Skip 1)
        $out = & $cmd.Source @pre -c 'import sys; print(sys.version_info[0])' 2>$null
        if ($LASTEXITCODE -eq 0 -and ("$out").Trim() -eq '3') { return @{ Exe = $cmd.Source; Pre = $pre } }
    }
    return $null
}

# Ghi một khoá vào huba.env (thay dòng cũ, giữ mọi dòng khác). UTF-8 KHÔNG BOM: BOM dính vào tên khoá
# đầu tiên và hubd đọc sai. Biến thật của môi trường vẫn thắng (config::load_one_env_file).
function Dat-KhoaEnv($Tep, $Khoa, $GiaTri) {
    $dong = @()
    if (Test-Path $Tep) { $dong = @(Get-Content -LiteralPath $Tep -Encoding UTF8 | Where-Object { $_ -notmatch ('^\s*' + [regex]::Escape($Khoa) + '\s*=') }) }
    $dong += "$Khoa=$GiaTri"
    [System.IO.File]::WriteAllLines($Tep, [string[]]$dong, (New-Object System.Text.UTF8Encoding($false)))
}

function Cai-Kem($Py) {
    $Cai = Join-Path $Kem 'huba_cai.py'
    $Mau = Join-Path $Kem 'tai-khoan.mau.json'
    $env:PYTHONUTF8 = '1'                                   # in tiếng Việt ra console cp1252 không chết
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    function Chay-Cai { param([string[]]$A) & $Py.Exe @($Py.Pre) $Cai @A; if ($LASTEXITCODE -ne 0) { throw "huba_cai.py $($A[0]) thoat $LASTEXITCODE" } }
    Moc 'kem: cau hinh (tai khoan con thieu + goc du an)'
    Chay-Cai @('cau-hinh', $Config, $Mau)
    $goc = (& $Py.Exe @($Py.Pre) $Cai goc-workspace $Config | Select-Object -Last 1)
    Moc 'kem: luat chung (CLAUDE.md o goc workspace — da co thi GIU)'
    Chay-Cai @('luat-goc', $Config, (Join-Path $Kem 'CLAUDE.md'))
    Moc 'kem: skill + tac tu (da co thi GIU)'
    Chay-Cai @('skill-goc', $Config, (Join-Path $Kem 'claude'))
    Moc 'kem: hook nhac chuyen phien (>= 50% ngu canh) vao settings cua goc workspace (gop, khong de)'
    Chay-Cai @('hook-nhac', $Config, (Join-Path $Kem 'scripts\nhac-chuyen-phien.py'))
    Moc 'kem: thu muc tai khoan + junction projects (so phien chung)'
    Chay-Cai @('thu-muc-acc', $Config)
    $pa = Join-Path $Kem 'bin\project-agent.exe'
    if (Test-Path $pa) {
        Moc 'kem: MCP codetrail cho moi tai khoan claude'
        Chay-Cai @('mcp-claude', $Config, $pa)
    } else { Moc "CANH BAO: khong co $pa — MCP codetrail khong duoc cai" }
    # Đường dẫn cho hubd: ghi vào huba.env (không phụ thuộc môi trường Task Scheduler thấy lúc nào).
    $envf = Join-Path $Here 'huba.env'
    Dat-KhoaEnv $envf 'HUBA_OPENCODE_PLUGIN' (Join-Path $Kem 'scripts\cong-bao-mat-plugin')
    Dat-KhoaEnv $envf 'BAO_MAT_POLICY' (Join-Path $Kem 'scripts\bao-mat-policy.json')
    if (Test-Path $pa) { Dat-KhoaEnv $envf 'HUBA_MCP_PROJECT_AGENT' $pa }
    if ($goc -and (Test-Path (Join-Path $goc 'CLAUDE.md'))) { Dat-KhoaEnv $envf 'HUBA_LUAT_GOC' (Join-Path $goc 'CLAUDE.md') }
    Moc "kem: da ghi duong dan cong bao mat / MCP / luat vao $envf"
}

# ① Cấu hình: chưa có thì để huba tạo (huba init), như trình cài Linux.
if (-not (Test-Path $Config)) {
    $cli0 = if ($BinDir) { Join-Path $BinDir 'huba.exe' } else { Join-Path $Rust 'target\release\huba.exe' }
    if (Test-Path $cli0) {
        Moc 'chua co huba.config.json — chay huba init'
        Push-Location $Here
        try { & $cli0 init; if ($LASTEXITCODE -ne 0) { throw "huba init thoat $LASTEXITCODE" } } finally { Pop-Location }
    }
}
if (-not (Test-Path $Config)) { throw "khong thay $Config — can cau hinh truoc (huba.config.json)" }

if (Test-Path $Kem) {
    $py = Tim-Python
    if (-not $py -and (Get-Command winget -ErrorAction SilentlyContinue) -and $env:HUBA_KHONG_CAI_CLI -ne '1') {
        Moc 'khong co Python — thu cai bang winget (pham vi nguoi dung, khong can admin)...'
        & winget install -e --id Python.Python.3.12 --scope user --accept-package-agreements --accept-source-agreements
        # winget cập nhật PATH của lần đăng nhập SAU; nạp lại PATH hiện tại để tìm thấy ngay.
        $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')
        $py = Tim-Python
    }
    if ($py) {
        Cai-Kem $py
    } else {
        Moc 'CANH BAO: khong co Python 3 — BO QUA luat chung / skill / MCP / tai khoan. Cai Python (winget install Python.Python.3.12) roi chay lai trinh cai.'
    }
} else {
    Moc 'goi khong co thu muc kem — chi cai huba (khong co luat chung/skill/cong bao mat)'
}

# ① Dựng (nếu không dùng bản dựng sẵn).
if (-not $BinDir) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "khong thay cargo — cai Rust (https://rustup.rs) hoac chay voi -BinDir <thu muc co hubad.exe + huba.exe>"
    }
    Moc 'dung ban release...'
    Push-Location $Rust
    try {
        & cargo build --release
        if ($LASTEXITCODE -ne 0) { throw "cargo build --release thoat $LASTEXITCODE — ban dang chay GIU NGUYEN" }
    } finally { Pop-Location }
}
foreach ($f in @($SrcHubd, $SrcCli)) { if (-not (Test-Path $f)) { throw "khong thay $f" } }

# ② Dừng bản đang chạy — hubd.exe đang chạy thì Windows KHOÁ tệp, chép đè sẽ hỏng.
if (Get-ScheduledTask -TaskName $Task -ErrorAction SilentlyContinue) {
    Moc "dung tac vu $Task..."
    Stop-ScheduledTask -TaskName $Task -ErrorAction SilentlyContinue
}
$han = (Get-Date).AddSeconds(20)
while ((Get-Process -Name hubd -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $han)) { Start-Sleep -Milliseconds 300 }
if (Get-Process -Name hubd -ErrorAction SilentlyContinue) {
    throw "hubd.exe van song sau 20 giay — khong chep de (ban dang chay GIU NGUYEN)"
}

# ③ Cài.
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Copy-Item $SrcHubd $DstHubd -Force
Copy-Item $SrcCli  $DstCli  -Force
Moc "da cai $DstHubd + $DstCli"

# ④ PATH + HUB_CONFIG cho người dùng (không đụng biến của máy).
$p = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not (($p -split ';') -contains $Dest)) {
    [Environment]::SetEnvironmentVariable('Path', (($p.TrimEnd(';') + ';' + $Dest).TrimStart(';')), 'User')
    Moc "da them $Dest vao PATH nguoi dung (mo cua so moi moi thay)"
}
[Environment]::SetEnvironmentVariable('HUB_CONFIG', $Config, 'User')

# ⑤ Tác vụ tự khởi động: lúc đăng nhập, mức quyền THƯỜNG, chết thì chạy lại, không giới hạn giờ.
#    Thư mục làm việc = cây huba ⟹ hubd tìm ra huba.config.json + huba.env ở đây.
$action    = New-ScheduledTaskAction -Execute $DstHubd -WorkingDirectory $Here
$trigger   = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings  = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) `
                -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) `
                -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Limited
Register-ScheduledTask -TaskName $Task -Action $action -Trigger $trigger -Settings $settings `
    -Principal $principal -Force | Out-Null
Moc "da dang ky tac vu $Task"

# ⑥ Chạy và ĐO lại — mã thoát của Start-ScheduledTask chỉ nói "đã bảo chạy", không nói "đang chạy".
Start-ScheduledTask -TaskName $Task
$han = (Get-Date).AddSeconds(15)
$proc = $null
while (-not $proc -and ((Get-Date) -lt $han)) {
    Start-Sleep -Milliseconds 500
    $proc = Get-Process -Name hubd -ErrorAction SilentlyContinue
}
if (-not $proc) {
    throw "hubd.exe KHONG song sau 15 giay — xem logs\huba.log va 'Get-ScheduledTaskInfo $Task'"
}
Moc ("hubd dang chay, pid " + ($proc | Select-Object -First 1).Id)
