# =============================================================================
# UTO Installer for Windows — inspired by nvm's install experience
#
# Usage (one-liner in PowerShell):
#   irm https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.ps1 | iex
#
# Or, to pin to a specific git ref / tag:
#   $env:UTO_REF = "v0.1.0"
#   irm https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.ps1 | iex
#
# Environment variables:
#   UTO_REF         — git branch, tag, or SHA to install from (default: main)
#   UTO_INSTALL_DIR — cargo --root override for the install directory (default: $HOME\.cargo)
#   UTO_SKIP_RUSTUP — set to "1" to skip the Rust/rustup installation check
# =============================================================================

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$UTO_REPO       = "https://github.com/adrianpothuaud/UTO"
$UTO_REF        = if ($env:UTO_REF) { $env:UTO_REF } else { "main" }
$UTO_INSTALL_DIR = $env:UTO_INSTALL_DIR
$UTO_SKIP_RUSTUP = $env:UTO_SKIP_RUSTUP

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
function Say([string]$msg) {
    Write-Host "[uto] $msg" -ForegroundColor Green
}

function Warn([string]$msg) {
    Write-Warning "[uto warn] $msg"
}

function Fail([string]$msg) {
    Write-Error "[uto error] $msg"
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
# Validate UTO_REF to prevent injection
# ---------------------------------------------------------------------------
function Validate-Ref {
    if ($UTO_REF -notmatch '^[A-Za-z0-9._/\-]+$') {
        Fail "UTO_REF contains invalid characters: '$UTO_REF'
Only alphanumeric characters, dots, hyphens, underscores, and slashes are allowed."
    }
}

# ---------------------------------------------------------------------------
# Build and install the 'uto' binary from source
# ---------------------------------------------------------------------------
function Install-Uto {
    Say "Installing 'uto' from $UTO_REPO @ $UTO_REF…"

    $baseArgs = @("install", "--git", $UTO_REPO, "uto-cli", "--locked")

    if ($UTO_INSTALL_DIR) {
        $baseArgs += @("--root", $UTO_INSTALL_DIR)
    }

    # Try branch ref first
    $branchArgs = $baseArgs + @("--branch", $UTO_REF)
    & cargo @branchArgs
    if ($LASTEXITCODE -eq 0) { return }

    # Fall back to tag ref
    Say "Branch ref failed — retrying with --tag $UTO_REF…"
    $tagArgs = $baseArgs + @("--tag", $UTO_REF)
    & cargo @tagArgs
    if ($LASTEXITCODE -eq 0) { return }

    # Last resort: no ref qualifier (installs HEAD of default branch)
    Say "Tag ref failed — installing from HEAD of default branch…"
    & cargo @baseArgs
    if ($LASTEXITCODE -ne 0) {
        Fail "Failed to install 'uto'. Check that $UTO_REPO is reachable."
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
Try running:  cargo install --git $UTO_REPO uto-cli"
    }
}

# ---------------------------------------------------------------------------
# Print getting-started steps
# ---------------------------------------------------------------------------
function Print-GettingStarted {
    Write-Host ""
    Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "  UTO installed — here is how to get started:"            -ForegroundColor Cyan
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
    Write-Host "  Full documentation: $UTO_REPO#readme"
    Write-Host ""
    Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
function Main {
    Say "UTO Installer (ref: $UTO_REF)"
    Write-Host ""

    Detect-Platform
    Validate-Ref
    Ensure-Rust
    Install-Uto
    Verify-Install
    Print-GettingStarted
}

Main
