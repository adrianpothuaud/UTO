//! Filesystem watcher for `--watch` mode auto re-run.
//!
//! Watches a directory recursively for create / modify / remove events using
//! the [`notify`] crate (platform-native backend).  Rapid bursts of events are
//! collapsed with a 300 ms debounce window before the `on_change` callback is
//! invoked.
//!
//! The watcher runs in a background OS thread; [`start_watcher`] returns
//! immediately.

use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Watch `path` recursively and call `on_change` (debounced at 300 ms) whenever
/// files are created, modified, or removed.
///
/// The watcher and its OS resources are kept alive inside the spawned thread for
/// the lifetime of the process.  The function returns an error string if the
/// watcher cannot be initialised or the path cannot be watched.
pub fn start_watcher(path: PathBuf, on_change: impl Fn() + Send + 'static) -> Result<(), String> {
    use notify::{RecursiveMode, Watcher};

    let (sync_tx, sync_rx) = std::sync::mpsc::channel::<()>();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            use notify::EventKind::{Create, Modify, Remove};
            if matches!(event.kind, Modify(_) | Create(_) | Remove(_)) {
                let _ = sync_tx.send(());
            }
        }
    })
    .map_err(|e| format!("Failed to create file watcher: {e}"))?;

    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch '{}': {e}", path.display()))?;

    let debounce = Duration::from_millis(300);
    std::thread::spawn(move || {
        // Move watcher into the thread so it stays alive.
        let _watcher = watcher;
        let mut last_trigger: Option<Instant> = None;
        while let Ok(()) = sync_rx.recv() {
            let now = Instant::now();
            let elapsed = last_trigger
                .map(|t| now.duration_since(t))
                .unwrap_or(debounce);
            if elapsed >= debounce {
                last_trigger = Some(now);
                on_change();
            }
        }
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_watcher_returns_error_for_missing_path() {
        let result = start_watcher(
            PathBuf::from("/tmp/uto-test-nonexistent-watch-dir-xyz"),
            || {},
        );
        assert!(result.is_err(), "should fail for non-existent directory");
    }

    #[test]
    fn start_watcher_succeeds_for_existing_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let result = start_watcher(tmp.path().to_path_buf(), || {});
        assert!(
            result.is_ok(),
            "should succeed for existing directory: {result:?}"
        );
    }
}
