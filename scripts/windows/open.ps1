$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$AppDir = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$Bin = Join-Path $AppDir "bin\song-request-linux.exe"
$StateDir = Join-Path $env:LOCALAPPDATA "song-request-linux"
$LogDir = Join-Path $StateDir "logs"
$OutLog = Join-Path $LogDir "launcher.log"
$ErrLog = Join-Path $LogDir "launcher.err.log"
$Dashboard = "http://127.0.0.1:7384/"
$Health = "http://127.0.0.1:7384/health"

New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

function Test-SrlHealth {
    try {
        Invoke-WebRequest -UseBasicParsing -Uri $Health -TimeoutSec 1 | Out-Null
        return $true
    } catch {
        return $false
    }
}

if (!(Test-Path $Bin)) {
    Write-Host "Nao encontrei o executavel: $Bin"
    exit 1
}

if (Test-SrlHealth) {
    Start-Process $Dashboard
    exit 0
}

Start-Process -FilePath $Bin -WorkingDirectory $AppDir -WindowStyle Hidden -RedirectStandardOutput $OutLog -RedirectStandardError $ErrLog

for ($i = 0; $i -lt 80; $i++) {
    if (Test-SrlHealth) {
        Start-Process $Dashboard
        exit 0
    }
    Start-Sleep -Milliseconds 150
}

Write-Host "O Song Request Linux iniciou, mas o painel nao respondeu em http://127.0.0.1:7384/."
Write-Host "Veja os logs em: $LogDir"
exit 1
