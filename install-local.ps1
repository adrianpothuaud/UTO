# =============================================================================
# UTO Local Installer for Windows — install UTO from local project directory
#
# Usage (from UTO project root in PowerShell):
#   .\install-local.ps1
#
# Or with custom install directory:
#   $env:UTO_INSTALL_DIR = "$HOME\.local"
#   .\install-local.ps1
#
# Environment variables:
#   UTO_INSTALL_DIR — cargo --root override for the install directory (default: $HOME\.cargo)
#   UTO_SKIP_RUSTUP — set to "1" to skip the Rust/rustup installation check
#   UTO_FORCE       — set to "1" to force reinstall even if current version matches
# =============================================================================

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$UTO_INSTALL_DIR = $env:UTO_INSTALL_DIR
$UTO_SKIP_RUSTUP = $env:UTO_SKIP_RUSTUP
$UTO_FORCE       = $env:UTO_FORCE

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
function Say([string]$msg) {
    Write-Host "[uto-local] $msg" -ForegroundColor Green
}

function Warn([string]$msg) {
    Write-Warning "[uto-local warn] $msg"
}

function Fail([string]$msg) {
    Write-Error "[uto-local error] $msg"
    exit 1
}

function CommandExists([string]$cmd) {
    $null -ne (Get-Command $cmd -ErrorAction SilentlyContinue)
}

# ---------------------------------------------------------------------------
# Detect platform
# ---------------------------------------------------------------------------
function Detect-Platform {
    Say "Detected platform: Windows"
    if (-not [System.Environment]::Is64BitOperatingSystem) {
        Warn "32-bit Windows detected. The build may be slower or fail on some targets."
    }
}

# ---------------------------------------------------------------------------
# Ensure we're in the UTO project directory
# ---------------------------------------------------------------------------
function Verify-ProjectDirectory {
    if (-not (Test-Path "Cargo.toml")) {
        Fail "Cargo.toml not found. Please run this script from the UTO project root."
    }

    if (-not (Test-Path "uto-cli" -PathType Container)) {
        Fail "uto-cli directory not found. Please run this script from the UTO project root."
    }

    if (-not (Test-Path "uto-cli\Cargo.toml")) {
        Fail "uto-cli\Cargo.toml not found. Are you in the correct directory?"
    }

    $content = Get-Content "uto-cli\Cargo.toml" -Raw
    if ($content -notmatch 'name\s*=\s*"uto-cli"') {
        Fail "uto-cli\Cargo.toml does not appear to be valid. Are you in the correct directory?"
    }

    Say "UTO project directory verified"
}

# ---------------------------------------------------------------------------
# Ensure Rust / cargo is available
# ---------------------------------------------------------------------------
function Ensure-Rust {
    if ($UTO_SKIP_RUSTUP -eq "1") {
        Say "Skipping Rust check (UTO_SKIP_RUSTUP=1)"
        if (-not (CommandExists "cargo")) { Fail "'cargo' not found in PATH." }
        return
    }

    if (CommandExists "cargo") {
        $ver = cargo --version 2>&1
        Say "Rust is already installed: $ver"
        return
    }

    Say "Rust not found — installing via rustup…"

    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"

    Say "Downloading rustup-init.exe from win.rustup.rs (HTTPS)…"
    # The rustup installer is downloaded over HTTPS — the same approach used
    # by the official rustup.rs website. See https://rustup.rs for details.
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe -UseBasicParsing

    Say "Running rustup installer (non-interactive)…"
    & $rustupExe -y --no-modify-path
    if ($LASTEXITCODE -ne 0) { Fail "rustup-init.exe failed with exit code $LASTEXITCODE." }

    # Refresh PATH so cargo is available in this session
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if ($cargoBin -notin ($env:PATH -split ";")) {
        $env:PATH = "$env:PATH;$cargoBin"
    }

    if (-not (CommandExists "cargo")) {
        Fail "Rust was installed but 'cargo' is not yet in PATH.
Open a new PowerShell window and re-run the UTO installer."
    }

    Say "Rust installed: $(cargo --version 2>&1)"
}

# ---------------------------------------------------------------------------
# Check if current version matches local version
# ---------------------------------------------------------------------------
function Check-CurrentVersion {
    if ($UTO_FORCE -eq "1") {
        Say "Forcing reinstall (UTO_FORCE=1)"
        return $false
    }

    if (-not (CommandExists "uto")) {
        Say "uto binary not found in PATH — proceeding with fresh install"
        return $false
    }

    $currentVersion = uto --version 2>&1
    $localVersion = cargo pkgid -p uto-cli 2>&1
    if ($localVersion -match '#(.+)$') {
        $localVersion = "uto $($matches[1])"
    } else {
        $localVersion = "unknown"
    }

    Say "Current installed version: $currentVersion"
    Say "Local project version: $localVersion"

    if ($currentVersion -match $localVersion) {
        Say "Versions match — skipping install (use UTO_FORCE=1 to override)"
        return $true
    }

    return $false
}

# ---------------------------------------------------------------------------
# Build and install the 'uto' binary from local source
# ---------------------------------------------------------------------------
function Install-Uto {
    Say "Building and installing 'uto' from local source…"

    $args = @("install", "--path", "uto-cli", "--locked")

    if ($UTO_INSTALL_DIR) {
        $args += @("--root", $UTO_INSTALL_DIR)
    }

    & cargo @args
    if ($LASTEXITCODE -ne 0) {
        Fail "Failed to install 'uto' from local source."
    }
}

# ---------------------------------------------------------------------------
# Confirm the binary is reachable
# ---------------------------------------------------------------------------
function Verify-Install {
    $defaultBin = if ($UTO_INSTALL_DIR) {
        Join-Path $UTO_INSTALL_DIR "bin"
    } else {
        Join-Path $env:USERPROFILE ".cargo\bin"
    }

    if (CommandExists "uto") {
        $ver = uto --version 2>&1
        Say "Successfully installed: $ver"
    } elseif (Test-Path (Join-Path $defaultBin "uto.exe")) {
        Warn "'uto' was installed to $defaultBin but is not yet in PATH."
        Warn "Add the following to your PATH (System Properties > Environment Variables):"
        Warn "  $defaultBin"
        Warn "Or run in this session: `$env:PATH += `";$defaultBin`""
    } else {
        Fail "Installation appeared to succeed but the 'uto' binary was not found.
Try running:  cargo install --path uto-cli"
    }
}

# ---------------------------------------------------------------------------
# Print getting-started steps
# ---------------------------------------------------------------------------
function Print-GettingStarted {
    Write-Host ""
    Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "  UTO installed from local source"                        -ForegroundColor Cyan
    Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  1. Scaffold a new test project"
    Write-Host ""
    Write-Host "       uto init .\my-tests --template web" -ForegroundColor White
    Write-Host ""
    Write-Host "  2. Run your tests"
    Write-Host ""
    Write-Host "       uto run --project .\my-tests --target web" -ForegroundColor White
    Write-Host ""
    Write-Host "  3. Open a structured HTML report"
    Write-Host ""
    Write-Host "       uto report --project .\my-tests --html" -ForegroundColor White
    Write-Host ""
    Write-Host "  4. Launch the interactive UI"
    Write-Host ""
    Write-Host "       uto ui --project .\my-tests" -ForegroundColor White
    Write-Host ""
    Write-Host "  To reinstall after making changes:"
    Write-Host ""
    Write-Host "       .\install-local.ps1" -ForegroundColor White
    Write-Host "       OR"
    Write-Host "       `$env:UTO_FORCE = `"1`"; .\install-local.ps1" -ForegroundColor White
    Write-Host ""
    Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
function Main {
    Say "UTO Local Installer"
    Write-Host ""

    Detect-Platform
    Verify-ProjectDirectory
    Ensure-Rust

    if (Check-CurrentVersion) {
        Write-Host ""
        Say "Installation skipped — use UTO_FORCE=1 to reinstall"
        exit 0
    }

    Install-Uto
    Verify-Install
    Print-GettingStarted
}

Main
