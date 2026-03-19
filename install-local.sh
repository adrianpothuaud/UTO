#!/usr/bin/env sh
# =============================================================================
# UTO Local Installer — install UTO from local project directory
#
# Usage (from UTO project root):
#   ./install-local.sh
#
# Or with custom install directory:
#   UTO_INSTALL_DIR="$HOME/.local" ./install-local.sh
#
# Environment variables:
#   UTO_INSTALL_DIR — cargo --root override for the install directory (default: $HOME/.cargo)
#   UTO_SKIP_RUSTUP — set to 1 to skip the Rust/rustup installation check
#   UTO_FORCE       — set to 1 to force reinstall even if current version matches
# =============================================================================

set -eu

UTO_INSTALL_DIR="${UTO_INSTALL_DIR:-}"
UTO_SKIP_RUSTUP="${UTO_SKIP_RUSTUP:-0}"
UTO_FORCE="${UTO_FORCE:-0}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
say() {
    printf '\033[1;32m[uto-local]\033[0m %s\n' "$1"
}

warn() {
    printf '\033[1;33m[uto-local warn]\033[0m %s\n' "$1" >&2
}

err() {
    printf '\033[1;31m[uto-local error]\033[0m %s\n' "$1" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "Required command not found: '$1'. Please install it and re-run."
    fi
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

# ---------------------------------------------------------------------------
# Detect OS
# ---------------------------------------------------------------------------
detect_platform() {
    _os="$(uname -s 2>/dev/null || echo unknown)"
    case "$_os" in
        Linux*)  say "Detected platform: Linux" ;;
        Darwin*) say "Detected platform: macOS" ;;
        MINGW* | MSYS* | CYGWIN*)
            err "Windows detected. Please use the PowerShell installer instead:
  .\\install-local.ps1"
            ;;
        *)
            warn "Unknown platform: $_os. Proceeding anyway…"
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Ensure we're in the UTO project directory
# ---------------------------------------------------------------------------
verify_project_directory() {
    if [ ! -f "Cargo.toml" ]; then
        err "Cargo.toml not found. Please run this script from the UTO project root."
    fi

    if [ ! -d "uto-cli" ]; then
        err "uto-cli directory not found. Please run this script from the UTO project root."
    fi

    if ! grep -q 'name = "uto-cli"' uto-cli/Cargo.toml 2>/dev/null; then
        err "uto-cli/Cargo.toml does not appear to be valid. Are you in the correct directory?"
    fi

    say "UTO project directory verified"
}

# ---------------------------------------------------------------------------
# Ensure Rust / cargo is available, installing via rustup if necessary
# ---------------------------------------------------------------------------
ensure_rust() {
    if [ "$UTO_SKIP_RUSTUP" = "1" ]; then
        say "Skipping Rust check (UTO_SKIP_RUSTUP=1)"
        need_cmd cargo
        return
    fi

    if check_cmd cargo; then
        say "Rust is already installed: $(cargo --version 2>&1)"
        return
    fi

    say "Rust not found — installing via rustup…"
    need_cmd curl

    # The rustup installer is fetched over HTTPS with TLS certificate validation
    # (--proto '=https' --tlsv1.2). This is the same pattern used by rustup.rs
    # itself and is the recommended installation method per https://rustup.rs.
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

    # Source cargo env so subsequent commands in this session can find cargo
    # shellcheck source=/dev/null
    if [ -f "$HOME/.cargo/env" ]; then
        . "$HOME/.cargo/env"
    fi

    if ! check_cmd cargo; then
        err "Rust was installed but 'cargo' is not yet in PATH.
Open a new terminal and re-run the UTO installer, or run:
  source \$HOME/.cargo/env"
    fi

    say "Rust installed: $(cargo --version)"
}

# ---------------------------------------------------------------------------
# Check if current version matches local version
# ---------------------------------------------------------------------------
check_current_version() {
    if [ "$UTO_FORCE" = "1" ]; then
        say "Forcing reinstall (UTO_FORCE=1)"
        return 1
    fi

    if ! check_cmd uto; then
        say "uto binary not found in PATH — proceeding with fresh install"
        return 1
    fi

    _current_version="$(uto --version 2>/dev/null || echo 'unknown')"
    _local_version="$(cargo pkgid -p uto-cli 2>/dev/null | sed 's/.*#//' || echo 'unknown')"

    say "Current installed version: $_current_version"
    say "Local project version: uto $_local_version"

    if echo "$_current_version" | grep -q "$_local_version" 2>/dev/null; then
        say "Versions match — skipping install (use UTO_FORCE=1 to override)"
        return 0
    fi

    return 1
}

# ---------------------------------------------------------------------------
# Build and install the 'uto' binary from local source
# ---------------------------------------------------------------------------
install_uto() {
    say "Building and installing 'uto' from local source…"

    # Build with --locked to respect Cargo.lock for reproducible builds
    if [ -n "$UTO_INSTALL_DIR" ]; then
        cargo install --path uto-cli --locked --root "$UTO_INSTALL_DIR" \
            || err "Failed to install 'uto' from local source."
    else
        cargo install --path uto-cli --locked \
            || err "Failed to install 'uto' from local source."
    fi
}

# ---------------------------------------------------------------------------
# Confirm the binary is reachable
# ---------------------------------------------------------------------------
verify_install() {
    _default_bin="${HOME}/.cargo/bin"
    if [ -n "$UTO_INSTALL_DIR" ]; then
        _default_bin="${UTO_INSTALL_DIR}/bin"
    fi

    if check_cmd uto; then
        say "Successfully installed: $(uto --version 2>/dev/null || echo 'uto')"
    elif [ -x "${_default_bin}/uto" ]; then
        warn "'uto' was installed to ${_default_bin} but is not yet in PATH."
        warn "Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        warn "  export PATH=\"\${PATH}:${_default_bin}\""
        warn "Then restart your shell or run:  source \${HOME}/.cargo/env"
    else
        err "Installation appeared to succeed but the 'uto' binary was not found.
Try running:  cargo install --path uto-cli"
    fi
}

# ---------------------------------------------------------------------------
# Print getting-started steps
# ---------------------------------------------------------------------------
print_getting_started() {
    printf '\n'
    printf '\033[1;36m════════════════════════════════════════════════════════\033[0m\n'
    printf '\033[1;36m  UTO installed from local source\033[0m\n'
    printf '\033[1;36m════════════════════════════════════════════════════════\033[0m\n'
    printf '\n'
    printf '  1. Scaffold a new test project\n'
    printf '\n'
    printf '       \033[1muto init ./my-tests --template web\033[0m\n'
    printf '\n'
    printf '  2. Run your tests\n'
    printf '\n'
    printf '       \033[1muto run --project ./my-tests --target web\033[0m\n'
    printf '\n'
    printf '  3. Open a structured HTML report\n'
    printf '\n'
    printf '       \033[1muto report --project ./my-tests --html\033[0m\n'
    printf '\n'
    printf '  4. Launch the interactive UI\n'
    printf '\n'
    printf '       \033[1muto ui --project ./my-tests\033[0m\n'
    printf '\n'
    printf '  To reinstall after making changes:\n'
    printf '\n'
    printf '       \033[1m./install-local.sh\033[0m\n'
    printf '       OR\n'
    printf '       \033[1mUTO_FORCE=1 ./install-local.sh\033[0m\n'
    printf '\n'
    printf '\033[1;36m════════════════════════════════════════════════════════\033[0m\n'
    printf '\n'
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
main() {
    say "UTO Local Installer"
    printf '\n'

    detect_platform
    verify_project_directory
    ensure_rust

    if check_current_version; then
        printf '\n'
        say "Installation skipped — use UTO_FORCE=1 to reinstall"
        exit 0
    fi

    install_uto
    verify_install
    print_getting_started
}

main "$@"
