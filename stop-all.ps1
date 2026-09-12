# Stopper app-vinduer og Docker-containere.
# Bruk:  .\stop-all.ps1

$ErrorActionPreference = "Continue"
$Root = $PSScriptRoot

Write-Host "==> Stopper app-vinduer (ingestion, simulator, alerts, backend, frontend)..." -ForegroundColor Cyan

$titles = @(
    "ingestion",
    "factory-simulator",
    "alerts",
    "security-monitor",
    "backend",
    "frontend"
)

Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe'" | ForEach-Object {
    try {
        $cmd = $_.CommandLine
        if (-not $cmd) { return }
        foreach ($title in $titles) {
            if ($cmd -like "*WindowTitle = '$title'*") {
                Write-Host "  stopper $title (PID $($_.ProcessId))"
                Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue
            }
        }
    } catch {}
}

# Cargo/node child processes often outlive the shell title trick; kill by path too.
$patterns = @(
    "services\\ingestion",
    "simulators\\factory-simulator",
    "services\\alerts",
    "apps\\backend",
    "apps\\frontend"
)

Get-CimInstance Win32_Process | Where-Object {
    $_.Name -match '^(cargo|ingestion|alerts|backend|node|next|factory-simulator)' -or
    ($_.CommandLine -and ($patterns | Where-Object { $_.CommandLine -like "*$_*" }))
} | ForEach-Object {
    # Be conservative: only kill if command line mentions this repo
    if ($_.CommandLine -and $_.CommandLine -like "*Industrial-Intelligence*") {
        Write-Host "  stopper $($_.Name) (PID $($_.ProcessId))"
        Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "==> Stopper Docker-containere..." -ForegroundColor Cyan
Set-Location $Root
docker compose stop

Write-Host "`nFerdig." -ForegroundColor Green
