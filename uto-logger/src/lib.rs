//! Modern structured logging and progress loaders for UTO.
//!
//! Goals:
//! - consistent multi-crate logging format
//! - process-aware metadata for upcoming multi-session/multi-process runs
//! - reusable spinner/loaders for long setup phases

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static LOGGER_INIT: OnceCell<()> = OnceCell::new();

pub use tracing::{debug, error, info, trace, warn};

/// Initializes global logging with default process-aware formatting.
///
/// Repeated calls are no-ops after the first successful initialization.
pub fn init(component: &str) -> Result<(), String> {
    LOGGER_INIT
        .get_or_try_init(|| {
            let _ = tracing_log::LogTracer::init();
            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"));

            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_ansi(true)
                        .compact(),
                )
                .try_init()
                .map_err(|e| format!("failed to initialize tracing subscriber: {e}"))?;

            tracing::info!(
                component = component,
                pid = std::process::id(),
                "logger initialized"
            );
            Ok(())
        })
        .map(|_| ())
}

/// Returns a stable process label suitable for multi-process reporting/log output.
pub fn process_label(name: &str) -> String {
    format!("{name}[pid={}]", std::process::id())
}

/// Loader manager that supports independent progress bars/spinners.
#[derive(Clone, Debug)]
pub struct LoaderManager {
    inner: Arc<MultiProgress>,
}

impl LoaderManager {
    /// Creates a new loader manager.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MultiProgress::new()),
        }
    }

    /// Starts a spinner with a professional compact style.
    pub fn spinner(&self, message: &str) -> LoaderHandle {
        let pb = self.inner.add(ProgressBar::new_spinner());
        let style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);
        pb.set_style(style);
        pb.enable_steady_tick(std::time::Duration::from_millis(90));
        pb.set_message(message.to_string());
        LoaderHandle { pb }
    }
}

impl Default for LoaderManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for a single spinner/progress item.
#[derive(Clone, Debug)]
pub struct LoaderHandle {
    pb: ProgressBar,
}

impl LoaderHandle {
    /// Updates displayed loader message.
    pub fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }

    /// Marks loader success and leaves a final line.
    pub fn success(&self, message: &str) {
        self.pb.finish_with_message(format!("✓ {message}"));
    }

    /// Marks loader failure and leaves a final line.
    pub fn fail(&self, message: &str) {
        self.pb.abandon_with_message(format!("✗ {message}"));
    }

    /// Clears loader output without final message.
    pub fn clear(&self) {
        self.pb.finish_and_clear();
    }
}
