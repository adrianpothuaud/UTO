#!/usr/bin/env sh
# =============================================================================
# UTO Installer — inspired by nvm's install experience
#
# Usage (one-liner):
#   curl -sSf https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.sh | sh
#
# Or, to pin to a specific git ref / tag:
#   UTO_REF=v0.1.0 curl -sSf https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.sh | sh
#
# Environment variables:
#   UTO_REF         — git branch, tag, or SHA to install from (default: main)
#   UTO_INSTALL_DIR — cargo --root override for the install directory (default: $HOME/.cargo)
#   UTO_SKIP_RUSTUP — set to 1 to skip the Rust/rustup installation check
# =============================================================================

set -eu

UTO_REPO="https://github.com/adrianpothuaud/UTO"
UTO_REF="${UTO_REF:-main}"
UTO_INSTALL_DIR="${UTO_INSTALL_DIR:-}"
UTO_SKIP_RUSTUP="${UTO_SKIP_RUSTUP:-0}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
say() {
    printf '\033[1;32m[uto]\033[0m %s\n' "$1"
}

warn() {
    printf '\033[1;33m[uto warn]\033[0m %s\n' "$1" >&2
}

err() {
    printf '\033[1;31m[uto error]\033[0m %s\n' "$1" >&2
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
  irm https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.ps1 | iex"
            ;;
        *)
            warn "Unknown platform: $_os. Proceeding anyway…"
            ;;
    esac
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
# Validate UTO_REF to prevent command injection
# ---------------------------------------------------------------------------
validate_ref() {
    # Allow only alphanumeric characters, dots, hyphens, underscores, and slashes
    case "$UTO_REF" in
        *[!A-Za-z0-9._/-]*)
            err "UTO_REF contains invalid characters: '${UTO_REF}'
Only alphanumeric characters, dots, hyphens, underscores, and slashes are allowed."
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Build and install the 'uto' binary from source
# ---------------------------------------------------------------------------
install_uto() {
    say "Installing 'uto' from ${UTO_REPO} @ ${UTO_REF}…"

    # The uto-cli package exposes a single binary named 'uto'.
    # Try branch ref, then tag ref, then fall back to HEAD of default branch.
    if [ -n "$UTO_INSTALL_DIR" ]; then
        cargo install --git "$UTO_REPO" uto-cli --locked --root "$UTO_INSTALL_DIR" --branch "$UTO_REF" 2>/dev/null \
        || { say "Branch ref failed — retrying with --tag ${UTO_REF}…" && \
             cargo install --git "$UTO_REPO" uto-cli --locked --root "$UTO_INSTALL_DIR" --tag "$UTO_REF" 2>/dev/null; } \
        || { say "Tag ref failed — installing from HEAD of default branch…" && \
             cargo install --git "$UTO_REPO" uto-cli --locked --root "$UTO_INSTALL_DIR"; } \
        || err "Failed to install 'uto'. Check that ${UTO_REPO} is reachable."
    else
        cargo install --git "$UTO_REPO" uto-cli --locked --branch "$UTO_REF" 2>/dev/null \
        || { say "Branch ref failed — retrying with --tag ${UTO_REF}…" && \
             cargo install --git "$UTO_REPO" uto-cli --locked --tag "$UTO_REF" 2>/dev/null; } \
        || { say "Tag ref failed — installing from HEAD of default branch…" && \
             cargo install --git "$UTO_REPO" uto-cli --locked; } \
        || err "Failed to install 'uto'. Check that ${UTO_REPO} is reachable."
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
Try running:  cargo install --git ${UTO_REPO} uto-cli"
    fi
}

# ---------------------------------------------------------------------------
# Print getting-started steps
# ---------------------------------------------------------------------------
print_getting_started() {
    printf '\n'
    printf '\033[1;36m════════════════════════════════════════════════════════\033[0m\n'
    printf '\033[1;36m  UTO installed — here is how to get started:\033[0m\n'
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
    printf '  Full documentation: %s\n' "${UTO_REPO}#readme"
    printf '\n'
    printf '\033[1;36m════════════════════════════════════════════════════════\033[0m\n'
    printf '\n'
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
main() {
    say "UTO Installer (ref: ${UTO_REF})"
    printf '\n'

    detect_platform
    validate_ref
    ensure_rust
    install_uto
    verify_install
    print_getting_started
}

main "$@"
