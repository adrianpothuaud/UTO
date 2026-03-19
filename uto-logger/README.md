# uto-logger

**Structured logging and progress indicators for the UTO automation framework.**

`uto-logger` provides consistent logging output across all UTO crates with support for different verbosity levels and progress indicators.

## Features

- **Structured logging** — Consistent format across driver, session, and test execution
- **Progress indicators** — Spinners and progress bars for long-running operations
- **Verbosity control** — Info, debug, and trace levels
- **Color output** — Terminal-friendly colored messages

## Usage

`uto-logger` is used internally by all UTO crates:

```rust
use uto_logger::{info, debug, warn, error};

info!("Starting ChromeDriver on port 9515");
debug!("Chrome version: {}", version);
warn!("AVD boot timeout exceeded, retrying...");
error!("Session creation failed: {}", err);
```

## Log Levels

- **ERROR** — Critical failures (always shown)
- **WARN** — Non-fatal issues (default)
- **INFO** — Key operations (default)
- **DEBUG** — Detailed execution traces (`-v`)
- **TRACE** — Full protocol dumps (`-vv`)

## Related Crates

All UTO crates use `uto-logger` for consistent output.

## License

MIT or Apache-2.0 (dual-licensed)
