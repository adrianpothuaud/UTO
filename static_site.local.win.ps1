param(
  [int]$Port = 4173
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$DistDir = Join-Path $RootDir "uto-site\dist"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Error "cargo is required but was not found in PATH."
}

if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
  Write-Error "python is required to serve the static site locally."
}

Write-Host "[INFO] Building static site..."
Push-Location $RootDir
try {
  cargo run -p uto-site
}
finally {
  Pop-Location
}

if (-not (Test-Path $DistDir)) {
  Write-Error "Dist directory not found: $DistDir"
}

$url = "http://127.0.0.1:$Port"
Write-Host "[INFO] Serving at $url"

# Best effort only; do not fail if browser cannot be opened.
try {
  Start-Process $url | Out-Null
}
catch {
}

Push-Location $DistDir
try {
  python -m http.server $Port
}
finally {
  Pop-Location
}
