# Start hele Industrial Intelligence-stakken i egne PowerShell-vinduer.
# Bruk:  .\start-all.ps1
# Stopp: .\stop-all.ps1

$Root = $PSScriptRoot

function Write-Step($msg) {
    Write-Host "`n==> $msg" -ForegroundColor Cyan
}

function Test-DockerRunning {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw "Docker er ikke installert eller ikke i PATH."
    }

    # Redirect everything; only trust LASTEXITCODE (stderr warnings are harmless)
    cmd /c "docker info >nul 2>&1"
    if ($LASTEXITCODE -ne 0) {
        throw "Docker Desktop kjører ikke. Start Docker Desktop og prøv igjen."
    }
}

function Warn-LocalPostgres {
    $svc = Get-Service -Name "postgresql-x64-17" -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -eq "Running") {
        Write-Host @"

ADVARSEL: Lokal postgresql-x64-17 kjører på port 5432.
Stopp den i en admin-PowerShell før TimescaleDB fungerer:

  Stop-Service postgresql-x64-17

"@ -ForegroundColor Yellow
    }
}

function Start-AppWindow {
    param(
        [string]$Title,
        [string]$WorkingDirectory,
        [string]$Command
    )

    $escapedDir = $WorkingDirectory.Replace("'", "''")
    $ps = @"
`$Host.UI.RawUI.WindowTitle = '$Title'
Set-Location '$escapedDir'
Write-Host 'Starting $Title...' -ForegroundColor Green
$Command
Write-Host ''
Write-Host '$Title stopped. Press Enter to close.' -ForegroundColor Yellow
Read-Host
"@

    Start-Process powershell -ArgumentList @(
        "-NoExit",
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-Command", $ps
    )
}

Write-Step "Sjekker Docker"
Test-DockerRunning
Warn-LocalPostgres

Write-Step "Starter MQTT + TimescaleDB"
Set-Location $Root

$mqtt = (cmd /c "docker ps -a --filter name=industrial-mqtt --format {{.Names}} 2>nul").Trim()
$tsdb = (cmd /c "docker ps -a --filter name=industrial-timescaledb --format {{.Names}} 2>nul").Trim()

if ($mqtt -and $tsdb) {
    cmd /c "docker start industrial-mqtt industrial-timescaledb"
    if ($LASTEXITCODE -ne 0) {
        throw "Klarte ikke starte eksisterende containere."
    }
} else {
    cmd /c "docker compose up -d"
    if ($LASTEXITCODE -ne 0) {
        throw "docker compose up feilet. Fjern gamle containere med: docker rm -f industrial-mqtt industrial-timescaledb"
    }
}

Write-Step "Venter på TimescaleDB"
$ready = $false
for ($i = 0; $i -lt 40; $i++) {
    cmd /c "docker exec industrial-timescaledb pg_isready -U industrial -d industrial_intelligence >nul 2>&1"
    if ($LASTEXITCODE -eq 0) {
        $ready = $true
        break
    }
    Start-Sleep -Seconds 2
}
if (-not $ready) {
    throw "TimescaleDB ble ikke klar i tide."
}
Write-Host "TimescaleDB er klar." -ForegroundColor Green

Write-Step "Starter applikasjoner"
Start-AppWindow "ingestion" (Join-Path $Root "services\ingestion") "cargo run"
Start-Sleep -Seconds 2
Start-AppWindow "factory-simulator" (Join-Path $Root "simulators\factory-simulator") "cargo run"
Start-Sleep -Seconds 1
Start-AppWindow "alerts" (Join-Path $Root "services\alerts") "cargo run"
Start-AppWindow "security-monitor" (Join-Path $Root "services\security-monitor") "cargo run -- --demo"
Start-AppWindow "backend" (Join-Path $Root "apps\backend") "cargo run"
Start-AppWindow "frontend" (Join-Path $Root "apps\frontend") "npx next dev --port 3001"

Write-Host @"

Alt er startet i separate vinduer.

  Dashboard:  http://localhost:3001
  Backend:    http://localhost:3000
  MQTT:       localhost:1883
  TimescaleDB: localhost:5432

Stopp med:  .\stop-all.ps1

"@ -ForegroundColor Green
