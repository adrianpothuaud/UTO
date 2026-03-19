# ADR 0015: Downloadable Install Script and Getting-Started Command

Date: 2026-03-18

## Status

Active — delivered

## Context

UTO has a functional Phase 4 CLI lifecycle (`uto init`, `uto run`, `uto report`) and a Phase 5 UI mode (`uto ui`). However, onboarding still requires users to clone the repository and build from source with `cargo build -p uto-cli`. This is a barrier for newcomers who:

- are not Rust developers but want to adopt UTO as an automation tool;
- expect a one-line install experience comparable to popular toolchains.

Well-known precedents have proven that a **downloadable shell install script** dramatically lowers the first-run friction:

| Tool | Install one-liner |
|---|---|
| **nvm** | `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh \| bash` |
| **rustup** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Bun** | `curl -fsSL https://bun.sh/install \| bash` |
| **Deno** | `curl -fsSL https://deno.land/install.sh \| sh` |

The desired end-user experience for UTO is:

```sh
# Install
curl -sSf https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.sh | sh

# Get started immediately
uto init ./my-tests --template web
uto run --project ./my-tests --target web
uto report --project ./my-tests --html
```

## Decision

UTO will ship two install scripts at the repository root:

1. **`install.sh`** — POSIX sh script for macOS and Linux.
2. **`install.ps1`** — PowerShell script for Windows.

Both scripts follow the same three-phase model used by rustup and nvm:

### Phase 1 — Platform detection

Identify the host OS and abort early with a helpful redirect message if the wrong installer is used (e.g., `install.sh` on Windows redirects to `install.ps1`).

### Phase 2 — Prerequisite resolution

Check whether Rust / `cargo` is already available. If not, install it automatically via the official [rustup](https://rustup.rs) installer (`sh.rustup.rs` on Unix, `win.rustup.rs` on Windows). This mirrors the pattern used by all major Rust tooling and avoids requiring users to pre-install Rust.

### Phase 3 — Binary installation

Use `cargo install --git <repo>` to build and install `uto-cli` (binary name: `uto`) from source. Two ref fallbacks are supported:

1. `--branch <UTO_REF>` — used when `UTO_REF` is a branch name (default: `main`).
2. `--tag <UTO_REF>` — used when `UTO_REF` is a tag (e.g., `v0.1.0`).
3. HEAD fallback — installs from the default branch if neither of the above succeeds.

After installation, both scripts:

- verify the `uto` binary is reachable in `PATH`;
- print actionable PATH-fix instructions if it is not;
- display a **getting-started summary** with the four key commands.

### Configuration via environment variables

| Variable | Default | Description |
|---|---|---|
| `UTO_REF` | `main` | Git branch, tag, or SHA to install from |
| `UTO_INSTALL_DIR` | `$HOME/.cargo` | Cargo `--root` override |
| `UTO_SKIP_RUSTUP` | `0` | Set to `1` to skip the Rust check |

### Getting-started command summary

After the installer completes, users see:

```
════════════════════════════════════════════════════════
  UTO installed — here is how to get started:
════════════════════════════════════════════════════════

  1. Scaffold a new test project

       uto init ./my-tests --template web

  2. Run your tests

       uto run --project ./my-tests --target web

  3. Open a structured HTML report

       uto report --project ./my-tests --html

  4. Launch the interactive UI

       uto ui --project ./my-tests
```

### Documentation surface updates

- **`README.md`**: Install one-liner added prominently at the top of the Quick Start section.
- **`uto-site/content/getting-started.md`**: Install one-liner replaces the manual `cargo build -p uto-cli` approach as the primary path; the manual path is retained as an alternative for contributors.
- **`uto-site/templates/index.html`**: Install one-liner surfaced in the hero code window for maximum visibility.

## Alternatives Considered

### Pre-built binary releases via GitHub Releases

Pre-compiled binaries would eliminate the Rust toolchain dependency and install in seconds. However, this requires a cross-compilation release pipeline (macOS aarch64, macOS x86_64, Linux x86_64, Linux aarch64, Windows x86_64) and signed binary distribution infrastructure. This is deferred to a post-Phase 5 release milestone.

### Cargo crates.io publication

Publishing `uto-cli` to crates.io would enable `cargo install uto-cli` with no URL. This is straightforward but requires a public crates.io release workflow and version pinning discipline. Deferred until the API surface is stable enough for a versioned crate release.

### No install script (status quo)

Requiring users to clone the repo and build from source is adequate for Rust contributors but creates unnecessary friction for test-automation users who care only about the `uto` CLI experience. The install script removes this friction without significant maintenance cost.

## Local Development Install Scripts

For contributors and developers working on UTO itself, two additional install scripts are provided:

- **`install-local.sh`** — POSIX sh script for macOS and Linux.
- **`install-local.ps1`** — PowerShell script for Windows.

These scripts:
- Verify the script is run from the UTO project root directory.
- Check if the currently installed `uto` version matches the local project version.
- Skip installation if versions match (override with `UTO_FORCE=1`).
- Use `cargo install --path uto-cli` to build and install from local source.
- Support the same environment variables as the remote installers (`UTO_INSTALL_DIR`, `UTO_SKIP_RUSTUP`, plus `UTO_FORCE`).

This streamlines the workflow for:
- Testing local changes to the CLI before pushing to GitHub.
- Validating new features in a clean install environment.
- Iterating on the CLI without manually invoking `cargo install --path` each time.

Usage:

```sh
# macOS / Linux
./install-local.sh

# Windows PowerShell
.\install-local.ps1

# Force reinstall even if version matches
UTO_FORCE=1 ./install-local.sh
```

The local installers are documented in the README under "Local Development Install".

## Consequences

### Positive

- First-time onboarding reduces from ~5 manual steps to a single curl command.
- The experience is consistent with industry-standard toolchains (rustup, nvm, bun).
- No new binary distribution infrastructure is required — `cargo install --git` leverages existing CI/CD.
- Environment variables allow pinning to specific releases, making CI usage straightforward.
- Windows users get a first-class install path via PowerShell.

### Negative

- `cargo install --git` performs a full source build, which takes 2–5 minutes on a clean machine. This is comparable to the Rust compilation experience users already expect.
- The `--locked` flag requires `Cargo.lock` to be committed to the repository. This is already the case for the workspace.
- Users without internet access or behind corporate proxies cannot use the install scripts; the manual build path remains the fallback.

### Maintenance implications

- When a new `uto` sub-command is added, the getting-started steps printed by both scripts should be reviewed and updated.
- When a `v1.0.0` tag or pre-built binary release workflow is created, the default `UTO_REF` in both scripts should be updated to point to the stable tag.

## Validation

```sh
# Unix smoke test (does not execute; validates script syntax only)
sh -n install.sh
sh -n install-local.sh

# PowerShell syntax check
pwsh -NoProfile -Command "Get-Content install.ps1 | Out-Null"
pwsh -NoProfile -Command "Get-Content install-local.ps1 | Out-Null"

# Manual end-to-end remote install (requires network)
curl -sSf https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.sh | sh
uto init /tmp/uto-smoke --template web
uto --version

# Manual local install test (from UTO project root)
./install-local.sh
uto --version
```

## References

- ADR 0009: Framework Product Direction — CLI and Reporting-First Experience
- ADR 0013: Getting Started and Troubleshooting
- ADR 0014: UTO UI Mode
- [nvm install script](https://github.com/nvm-sh/nvm/blob/master/install.sh) — inspiration for structure and env-variable API
- [rustup install script](https://sh.rustup.rs) — inspiration for platform detection and rustup bootstrap
