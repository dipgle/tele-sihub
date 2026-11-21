# cai.ps1 - cai huba tren Windows: bam doi CAI.cmd, tra loi cac hop thoai, xong. Khong phai go lenh.
# Chay bang PowerShell THUONG (khong Run as Administrator). Chua chay thu tren Windows that.
# Toan bo diem ra duoc ghi vao cai.log canh tep nay - gui tep ay ve neu co loi.
$ErrorActionPreference = 'Stop'
$Here = Split-Path -Parent $MyInvocation.MyCommand.Path
$Cli  = Join-Path $Here 'huba.exe'
$Log  = Join-Path $Here 'cai.log'
try { Start-Transcript -Path $Log -Force | Out-Null } catch { }
Add-Type -AssemblyName System.Windows.Forms
function Moc($s) { Write-Host ("[{0}] {1}" -f (Get-Date -Format 'HH:mm:ss'), $s) }
# Hop thoai: $nut = 'OK' | 'OKCancel' | 'YesNo'. Tra ve chuoi nut da bam.
function Hop($noidung, $nut = 'OK', $icon = 'Information') {
    [System.Windows.Forms.MessageBox]::Show($noidung, 'Cai huba', $nut, $icon).ToString()
}
function Dung($ma, $noidung) {
    Moc $noidung
    Hop "$noidung`n`nChi tiet: $Log" 'OK' 'Error' | Out-Null
    try { Stop-Transcript | Out-Null } catch { }
    exit $ma
}

try {
    if (-not (Test-Path $Cli)) { Dung 2 "Khong thay huba.exe trong $Here. Hay giai nen DU goi zip roi bam CAI.cmd trong thu muc huba-windows-..." }

    # 1) Tep tai tu mang mang dau "Mark of the Web" - go ra.
    Moc 'go dau tai-tu-mang khoi cac tep trong goi'
    Get-ChildItem -Path $Here -Recurse -File | Unblock-File

    # 2) Thu chay that. Bi chan boi chinh sach (Smart App Control / WDAC / AppLocker) thi hoi nguoi dung, KHONG tu vong qua.
    $chay = $false
    for ($lan = 1; $lan -le 5 -and -not $chay; $lan++) {
        try {
            & $Cli --version | Out-Null
            if ($LASTEXITCODE -eq 0) { $chay = $true } else { throw "huba.exe thoat $LASTEXITCODE" }
        } catch {
            Moc "huba.exe khong chay duoc: $($_.Exception.Message)"
            $sac = $null
            try { $sac = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy' -ErrorAction Stop).VerifiedAndReputablePolicyState } catch { }
            if ($sac -eq 1 -or $sac -eq 2) {
                $tl = Hop ("Windows dang chan huba.exe vi chua duoc ky so (Smart App Control).`n`n" +
                    "Cach xu ly: bam OK de mo Windows Security, vao 'App & browser control' > 'Smart App Control' > chon Off.`n" +
                    "Luu y: tat Smart App Control roi thi khong bat lai duoc neu khong cai lai Windows.`n`n" +
                    "Tat xong, bam OK trong hop thoai tiep theo de thu lai. Khong muon tat thi bam Cancel.") 'OKCancel' 'Warning'
                if ($tl -ne 'OK') { Dung 2 'Da dung: huba.exe bi Smart App Control chan va ban chon khong tat.' }
                try { Start-Process 'windowsdefender://' } catch { Moc 'khong mo duoc Windows Security - mo tay: Start > Windows Security' }
                $tl2 = Hop 'Da tat Smart App Control xong chua? Bam OK de thu chay lai huba.exe.' 'OKCancel' 'Question'
                if ($tl2 -ne 'OK') { Dung 2 'Da dung theo yeu cau.' }
            } else {
                Dung 2 ("huba.exe bi chan hoac loi (Smart App Control = $sac).`n" +
                    "Neu day la may cua co quan/truong: chinh sach Application Control dang chan tep chua ky so - can quan tri vien cho phep huba.exe.")
            }
        }
    }
    if (-not $chay) { Dung 2 'huba.exe van khong chay duoc sau 5 lan thu.' }

    # 3) Cau hinh: chua co thi tao. claude_accounts de trong = dung tai khoan claude cua chinh may nay.
    Push-Location $Here
    try {
        if (-not (Test-Path (Join-Path $Here 'huba.config.json'))) {
            Moc 'tao cau hinh (huba init)'
            & $Cli init
            if ($LASTEXITCODE -ne 0) { throw "huba init thoat $LASTEXITCODE" }
        }

        # 3b) Thu muc lam viec (goc workspace) do NGUOI CAI CHON. Luat chung, skill, hook, MCP, huba.env deu di theo thu muc nay.
    #     Da khai trong cau hinh hoac da dat HUBA_GOC thi bo qua. Bam Cancel = dung thu muc mac dinh (%USERPROFILE%\projects, chua co thi tao).
    $cfgText = Get-Content (Join-Path $Here 'huba.config.json') -Raw -ErrorAction SilentlyContinue
    if (-not $env:HUBA_GOC -and ($cfgText -notmatch '"workspace_root"\s*:\s*"[^"]+"')) {
        $mac = Join-Path $env:USERPROFILE 'projects'
        $dlg = New-Object System.Windows.Forms.FolderBrowserDialog
        $dlg.Description = 'Chon THU MUC LAM VIEC (noi chua cac du an cua ban). Bam Make New Folder de tao thu muc moi.'
        $dlg.ShowNewFolderButton = $true
        $dlg.SelectedPath = $(if (Test-Path $mac) { $mac } else { $env:USERPROFILE })
        if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK -and $dlg.SelectedPath) {
            $env:HUBA_GOC = $dlg.SelectedPath
        } else {
            $env:HUBA_GOC = $mac
        }
        Moc "thu muc lam viec: $env:HUBA_GOC"
    }

    # 4) Bot Telegram: chua co token trong huba.env thi mo trang dien (trinh duyet, 127.0.0.1), cho toi khi luu xong.
        #    DUNG BOT RIENG cho may Windows - hai hubd cung mot bot se giang getUpdates (409).
        $envf = Join-Path $Here 'huba.env'
        $coToken = (Test-Path $envf) -and ((Get-Content $envf -ErrorAction SilentlyContinue) -match '^\s*HUB_TELEGRAM_BOT_TOKEN\s*=\s*\S+')
        if (-not $coToken) {
            Hop ("Tiep theo: trinh duyet se mo trang dien bot Telegram.`n`n" +
                "Can: token bot (tao bot rieng cho may nay qua @BotFather) va chat id cua ban.`n" +
                "Dien xong bam Luu tren trang do - cua so nay se tu chay tiep.") 'OK' 'Information' | Out-Null
            & $Cli setup
            if ($LASTEXITCODE -ne 0) { throw "huba setup thoat $LASTEXITCODE" }
        }
    } finally { Pop-Location }

    # 5) Cai + chay nen (Task Scheduler huba-hubd) + luat chung/skill (can Python; trinh cai tu thu winget).
    Moc 'cai + chay nen (co the mat vai phut neu phai cai Python)'
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $Here 'install_update.ps1') -BinDir $Here
    $ma = $LASTEXITCODE
    if ($ma -ne 0) { Dung $ma "Cai that bai (ma $ma). Gui tep cai.log ve de xem." }

    Hop "Cai xong. huba da chay nen va se tu chay moi khi dang nhap.`nThu: nhan /help cho bot Telegram cua ban." 'OK' 'Information' | Out-Null
    try { Stop-Transcript | Out-Null } catch { }
    exit 0
} catch {
    Dung 1 "Loi: $($_.Exception.Message)"
}
